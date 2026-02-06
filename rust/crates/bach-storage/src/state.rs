//! State database implementation

use crate::db::{cf, Database};
use crate::error::{StorageError, StorageResult};
use crate::traits::{Account, StateReader, StateWriter};
use bach_primitives::{Address, H256};
use std::collections::HashMap;

/// Storage key combining address and slot
fn storage_key(address: &Address, slot: &H256) -> Vec<u8> {
    let mut key = Vec::with_capacity(20 + 32);
    key.extend_from_slice(address.as_bytes());
    key.extend_from_slice(slot.as_bytes());
    key
}

/// State database backed by RocksDB
pub struct StateDb {
    db: Database,
}

impl StateDb {
    /// Create a new state database
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get the underlying database
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Commit changes from a cache
    pub fn commit(&self, cache: &StateCache) -> StorageResult<()> {
        let mut batch = self.db.batch();

        // Commit accounts
        for (address, account) in &cache.accounts {
            if let Some(account) = account {
                batch.put(cf::ACCOUNTS, address.as_bytes(), &account.to_bytes());
            } else {
                batch.delete(cf::ACCOUNTS, address.as_bytes());
            }
        }

        // Commit storage
        for ((address, slot), value) in &cache.storage {
            let key = storage_key(address, slot);
            if *value == H256::ZERO {
                batch.delete(cf::STORAGE, &key);
            } else {
                batch.put(cf::STORAGE, &key, value.as_bytes());
            }
        }

        // Commit code
        for (code_hash, code) in &cache.code {
            batch.put(cf::CODE, code_hash.as_bytes(), code);
        }

        self.db.write_batch(batch)
    }
}

impl StateReader for StateDb {
    fn get_account(&self, address: &Address) -> StorageResult<Option<Account>> {
        let bytes = self.db.get(cf::ACCOUNTS, address.as_bytes())?;
        Ok(bytes.and_then(|b| Account::from_bytes(&b)))
    }

    fn get_storage(&self, address: &Address, key: &H256) -> StorageResult<H256> {
        let storage_key = storage_key(address, key);
        let bytes = self.db.get(cf::STORAGE, &storage_key)?;
        match bytes {
            Some(b) => H256::from_slice(&b)
                .map_err(|e| StorageError::Deserialization(e.to_string())),
            None => Ok(H256::ZERO),
        }
    }

    fn get_code(&self, code_hash: &H256) -> StorageResult<Option<Vec<u8>>> {
        self.db.get(cf::CODE, code_hash.as_bytes())
    }
}

impl StateWriter for StateDb {
    fn set_account(&mut self, address: Address, account: Account) -> StorageResult<()> {
        self.db.put(cf::ACCOUNTS, address.as_bytes(), &account.to_bytes())
    }

    fn delete_account(&mut self, address: &Address) -> StorageResult<()> {
        self.db.delete(cf::ACCOUNTS, address.as_bytes())
    }

    fn set_storage(&mut self, address: Address, key: H256, value: H256) -> StorageResult<()> {
        let storage_key = storage_key(&address, &key);
        if value == H256::ZERO {
            self.db.delete(cf::STORAGE, &storage_key)
        } else {
            self.db.put(cf::STORAGE, &storage_key, value.as_bytes())
        }
    }

    fn set_code(&mut self, code_hash: H256, code: Vec<u8>) -> StorageResult<()> {
        self.db.put(cf::CODE, code_hash.as_bytes(), &code)
    }
}

/// In-memory state cache for batching changes
#[derive(Default)]
pub struct StateCache {
    /// Cached accounts (None = deleted)
    accounts: HashMap<Address, Option<Account>>,
    /// Cached storage
    storage: HashMap<(Address, H256), H256>,
    /// Cached code
    code: HashMap<H256, Vec<u8>>,
}

impl StateCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all cached changes
    pub fn clear(&mut self) {
        self.accounts.clear();
        self.storage.clear();
        self.code.clear();
    }

    /// Get number of cached account changes
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Get number of cached storage changes
    pub fn storage_count(&self) -> usize {
        self.storage.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty() && self.storage.is_empty() && self.code.is_empty()
    }
}

impl StateReader for StateCache {
    fn get_account(&self, address: &Address) -> StorageResult<Option<Account>> {
        Ok(self.accounts.get(address).cloned().flatten())
    }

    fn get_storage(&self, address: &Address, key: &H256) -> StorageResult<H256> {
        Ok(self.storage.get(&(*address, *key)).cloned().unwrap_or(H256::ZERO))
    }

    fn get_code(&self, code_hash: &H256) -> StorageResult<Option<Vec<u8>>> {
        Ok(self.code.get(code_hash).cloned())
    }
}

impl StateWriter for StateCache {
    fn set_account(&mut self, address: Address, account: Account) -> StorageResult<()> {
        self.accounts.insert(address, Some(account));
        Ok(())
    }

    fn delete_account(&mut self, address: &Address) -> StorageResult<()> {
        self.accounts.insert(*address, None);
        Ok(())
    }

    fn set_storage(&mut self, address: Address, key: H256, value: H256) -> StorageResult<()> {
        self.storage.insert((address, key), value);
        Ok(())
    }

    fn set_code(&mut self, code_hash: H256, code: Vec<u8>) -> StorageResult<()> {
        self.code.insert(code_hash, code);
        Ok(())
    }
}

/// Layered state with fallback to underlying storage
pub struct CachedState<'a> {
    cache: StateCache,
    underlying: &'a dyn StateReader,
}

impl<'a> CachedState<'a> {
    /// Create a new cached state layer
    pub fn new(underlying: &'a dyn StateReader) -> Self {
        Self {
            cache: StateCache::new(),
            underlying,
        }
    }

    /// Get the cache
    pub fn cache(&self) -> &StateCache {
        &self.cache
    }

    /// Take ownership of the cache
    pub fn into_cache(self) -> StateCache {
        self.cache
    }
}

impl StateReader for CachedState<'_> {
    fn get_account(&self, address: &Address) -> StorageResult<Option<Account>> {
        // Check cache first
        if let Some(cached) = self.cache.accounts.get(address) {
            return Ok(cached.clone());
        }
        // Fall back to underlying
        self.underlying.get_account(address)
    }

    fn get_storage(&self, address: &Address, key: &H256) -> StorageResult<H256> {
        // Check cache first
        if let Some(cached) = self.cache.storage.get(&(*address, *key)) {
            return Ok(*cached);
        }
        // Fall back to underlying
        self.underlying.get_storage(address, key)
    }

    fn get_code(&self, code_hash: &H256) -> StorageResult<Option<Vec<u8>>> {
        // Check cache first
        if let Some(cached) = self.cache.code.get(code_hash) {
            return Ok(Some(cached.clone()));
        }
        // Fall back to underlying
        self.underlying.get_code(code_hash)
    }
}

impl StateWriter for CachedState<'_> {
    fn set_account(&mut self, address: Address, account: Account) -> StorageResult<()> {
        self.cache.set_account(address, account)
    }

    fn delete_account(&mut self, address: &Address) -> StorageResult<()> {
        self.cache.delete_account(address)
    }

    fn set_storage(&mut self, address: Address, key: H256, value: H256) -> StorageResult<()> {
        self.cache.set_storage(address, key, value)
    }

    fn set_code(&mut self, code_hash: H256, code: Vec<u8>) -> StorageResult<()> {
        self.cache.set_code(code_hash, code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::EMPTY_CODE_HASH;
    use std::fs;

    fn temp_db_path() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cnt = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/bach_state_test_{}_{}", id, cnt)
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_state_db_account() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);

        // Initially no account
        assert!(state.get_account(&addr).unwrap().is_none());

        // Set account
        let account = Account {
            nonce: 5,
            balance: 1000,
            code_hash: EMPTY_CODE_HASH,
            storage_root: H256::ZERO,
        };
        state.set_account(addr, account.clone()).unwrap();

        // Read back
        let read = state.get_account(&addr).unwrap().unwrap();
        assert_eq!(read.nonce, 5);
        assert_eq!(read.balance, 1000);

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_state_db_storage() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);
        let value = H256::from_bytes([0xff; 32]);

        // Initially zero
        assert_eq!(state.get_storage(&addr, &slot).unwrap(), H256::ZERO);

        // Set value
        state.set_storage(addr, slot, value).unwrap();

        // Read back
        assert_eq!(state.get_storage(&addr, &slot).unwrap(), value);

        // Set to zero (delete)
        state.set_storage(addr, slot, H256::ZERO).unwrap();
        assert_eq!(state.get_storage(&addr, &slot).unwrap(), H256::ZERO);

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_state_cache() {
        let mut cache = StateCache::new();
        let addr = Address::from_bytes([0x42; 20]);

        // Set account in cache
        let account = Account {
            nonce: 1,
            balance: 500,
            ..Default::default()
        };
        cache.set_account(addr, account).unwrap();

        assert_eq!(cache.account_count(), 1);
        let read = cache.get_account(&addr).unwrap().unwrap();
        assert_eq!(read.balance, 500);
    }

    #[test]
    fn test_cached_state_layering() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);

        // Set account in underlying
        let account = Account {
            nonce: 1,
            balance: 100,
            ..Default::default()
        };
        state.set_account(addr, account).unwrap();

        // Create cached layer
        let mut cached = CachedState::new(&state);

        // Should read from underlying
        let read = cached.get_account(&addr).unwrap().unwrap();
        assert_eq!(read.balance, 100);

        // Modify in cache
        let modified = Account {
            nonce: 2,
            balance: 200,
            ..Default::default()
        };
        cached.set_account(addr, modified).unwrap();

        // Should read from cache now
        let read = cached.get_account(&addr).unwrap().unwrap();
        assert_eq!(read.balance, 200);

        // Underlying unchanged
        let underlying = state.get_account(&addr).unwrap().unwrap();
        assert_eq!(underlying.balance, 100);

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_commit_cache() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);

        // Build up cache
        let mut cache = StateCache::new();
        cache.set_account(addr, Account {
            nonce: 5,
            balance: 1000,
            ..Default::default()
        }).unwrap();
        cache.set_storage(addr, slot, H256::from_bytes([0xff; 32])).unwrap();

        // Commit
        state.commit(&cache).unwrap();

        // Verify committed
        let read = state.get_account(&addr).unwrap().unwrap();
        assert_eq!(read.nonce, 5);
        assert_eq!(state.get_storage(&addr, &slot).unwrap(), H256::from_bytes([0xff; 32]));

        state.database().close();
        cleanup(&path);
    }

    // ==================== Additional StateDb Tests ====================

    #[test]
    fn test_state_db_code() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let code_hash = H256::from_bytes([0x42; 32]);
        let code = vec![0x60, 0x00, 0x60, 0x00, 0xf3]; // Simple bytecode

        // Initially no code
        assert!(state.get_code(&code_hash).unwrap().is_none());

        // Set code
        state.set_code(code_hash, code.clone()).unwrap();

        // Read back
        let read = state.get_code(&code_hash).unwrap().unwrap();
        assert_eq!(read, code);

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_state_db_delete_account() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);

        let account = Account {
            nonce: 1,
            balance: 100,
            ..Default::default()
        };
        state.set_account(addr, account).unwrap();
        assert!(state.get_account(&addr).unwrap().is_some());

        state.delete_account(&addr).unwrap();
        assert!(state.get_account(&addr).unwrap().is_none());

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_state_db_multiple_accounts() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);

        // Create multiple accounts
        for i in 0..10u8 {
            let addr = Address::from_bytes([i; 20]);
            let account = Account {
                nonce: i as u64,
                balance: (i as u128) * 100,
                ..Default::default()
            };
            state.set_account(addr, account).unwrap();
        }

        // Verify all accounts
        for i in 0..10u8 {
            let addr = Address::from_bytes([i; 20]);
            let account = state.get_account(&addr).unwrap().unwrap();
            assert_eq!(account.nonce, i as u64);
            assert_eq!(account.balance, (i as u128) * 100);
        }

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_state_db_multiple_storage_slots() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);

        // Set multiple storage slots
        for i in 0..10u8 {
            let slot = H256::from_bytes([i; 32]);
            let value = H256::from_bytes([i + 100; 32]);
            state.set_storage(addr, slot, value).unwrap();
        }

        // Verify all slots
        for i in 0..10u8 {
            let slot = H256::from_bytes([i; 32]);
            let expected = H256::from_bytes([i + 100; 32]);
            assert_eq!(state.get_storage(&addr, &slot).unwrap(), expected);
        }

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_state_db_different_address_storage_isolation() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr1 = Address::from_bytes([0x01; 20]);
        let addr2 = Address::from_bytes([0x02; 20]);
        let slot = H256::from_bytes([0x00; 32]);

        state.set_storage(addr1, slot, H256::from_bytes([0xaa; 32])).unwrap();
        state.set_storage(addr2, slot, H256::from_bytes([0xbb; 32])).unwrap();

        // Same slot, different addresses should have different values
        assert_eq!(state.get_storage(&addr1, &slot).unwrap(), H256::from_bytes([0xaa; 32]));
        assert_eq!(state.get_storage(&addr2, &slot).unwrap(), H256::from_bytes([0xbb; 32]));

        state.database().close();
        cleanup(&path);
    }

    // ==================== Additional StateCache Tests ====================

    #[test]
    fn test_state_cache_default() {
        let cache = StateCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.account_count(), 0);
        assert_eq!(cache.storage_count(), 0);
    }

    #[test]
    fn test_state_cache_clear() {
        let mut cache = StateCache::new();
        let addr = Address::from_bytes([0x42; 20]);

        cache.set_account(addr, Account::default()).unwrap();
        cache.set_storage(addr, H256::ZERO, H256::from_bytes([0x01; 32])).unwrap();
        cache.set_code(H256::from_bytes([0x42; 32]), vec![0x60]).unwrap();

        assert!(!cache.is_empty());

        cache.clear();

        assert!(cache.is_empty());
        assert_eq!(cache.account_count(), 0);
        assert_eq!(cache.storage_count(), 0);
    }

    #[test]
    fn test_state_cache_delete_account() {
        let mut cache = StateCache::new();
        let addr = Address::from_bytes([0x42; 20]);

        cache.set_account(addr, Account {
            nonce: 1,
            balance: 100,
            ..Default::default()
        }).unwrap();

        assert!(cache.get_account(&addr).unwrap().is_some());

        cache.delete_account(&addr).unwrap();

        // After delete, get_account returns None (marked as deleted)
        assert!(cache.get_account(&addr).unwrap().is_none());
        assert_eq!(cache.account_count(), 1); // Still tracked as deleted
    }

    #[test]
    fn test_state_cache_storage_operations() {
        let mut cache = StateCache::new();
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);

        // Initially zero
        assert_eq!(cache.get_storage(&addr, &slot).unwrap(), H256::ZERO);

        // Set value
        cache.set_storage(addr, slot, H256::from_bytes([0xff; 32])).unwrap();
        assert_eq!(cache.storage_count(), 1);
        assert_eq!(cache.get_storage(&addr, &slot).unwrap(), H256::from_bytes([0xff; 32]));
    }

    #[test]
    fn test_state_cache_code_operations() {
        let mut cache = StateCache::new();
        let code_hash = H256::from_bytes([0x42; 32]);
        let code = vec![0x60, 0x00, 0x60, 0x00, 0xf3];

        assert!(cache.get_code(&code_hash).unwrap().is_none());

        cache.set_code(code_hash, code.clone()).unwrap();
        assert_eq!(cache.get_code(&code_hash).unwrap(), Some(code));
    }

    // ==================== Additional CachedState Tests ====================

    #[test]
    fn test_cached_state_storage_layering() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);

        // Set storage in underlying
        state.set_storage(addr, slot, H256::from_bytes([0xaa; 32])).unwrap();

        // Create cached layer
        let mut cached = CachedState::new(&state);

        // Should read from underlying
        assert_eq!(cached.get_storage(&addr, &slot).unwrap(), H256::from_bytes([0xaa; 32]));

        // Modify in cache
        cached.set_storage(addr, slot, H256::from_bytes([0xbb; 32])).unwrap();

        // Should read from cache now
        assert_eq!(cached.get_storage(&addr, &slot).unwrap(), H256::from_bytes([0xbb; 32]));

        // Underlying unchanged
        assert_eq!(state.get_storage(&addr, &slot).unwrap(), H256::from_bytes([0xaa; 32]));

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_cached_state_code_layering() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let code_hash = H256::from_bytes([0x42; 32]);
        let code1 = vec![0x60, 0x00];
        let code2 = vec![0x60, 0x01, 0x60, 0x02];

        // Set code in underlying
        state.set_code(code_hash, code1.clone()).unwrap();

        // Create cached layer
        let mut cached = CachedState::new(&state);

        // Should read from underlying
        assert_eq!(cached.get_code(&code_hash).unwrap(), Some(code1.clone()));

        // Set new code in cache (same hash, different code - unusual but testing layer behavior)
        cached.set_code(code_hash, code2.clone()).unwrap();

        // Should read from cache now
        assert_eq!(cached.get_code(&code_hash).unwrap(), Some(code2));

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_cached_state_into_cache() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);

        let mut cached = CachedState::new(&state);
        cached.set_account(addr, Account {
            nonce: 5,
            balance: 500,
            ..Default::default()
        }).unwrap();

        let cache = cached.into_cache();
        assert_eq!(cache.account_count(), 1);
        assert_eq!(cache.get_account(&addr).unwrap().unwrap().balance, 500);

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_cached_state_cache_accessor() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);

        let mut cached = CachedState::new(&state);
        cached.set_account(addr, Account::default()).unwrap();

        let cache_ref = cached.cache();
        assert_eq!(cache_ref.account_count(), 1);

        state.database().close();
        cleanup(&path);
    }

    // ==================== Commit Tests ====================

    #[test]
    fn test_commit_delete_account() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);

        // First create account directly
        state.set_account(addr, Account {
            nonce: 1,
            balance: 100,
            ..Default::default()
        }).unwrap();

        // Then delete via cache commit
        let mut cache = StateCache::new();
        cache.delete_account(&addr).unwrap();
        state.commit(&cache).unwrap();

        // Should be deleted
        assert!(state.get_account(&addr).unwrap().is_none());

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_commit_zero_storage() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);

        // First set non-zero
        state.set_storage(addr, slot, H256::from_bytes([0xff; 32])).unwrap();

        // Then set to zero via cache (should delete)
        let mut cache = StateCache::new();
        cache.set_storage(addr, slot, H256::ZERO).unwrap();
        state.commit(&cache).unwrap();

        // Should be zero (deleted)
        assert_eq!(state.get_storage(&addr, &slot).unwrap(), H256::ZERO);

        state.database().close();
        cleanup(&path);
    }

    #[test]
    fn test_commit_multiple_changes() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let state = StateDb::new(db);

        let mut cache = StateCache::new();

        // Add multiple accounts and storage
        for i in 0..5u8 {
            let addr = Address::from_bytes([i; 20]);
            cache.set_account(addr, Account {
                nonce: i as u64,
                balance: (i as u128) * 100,
                ..Default::default()
            }).unwrap();

            for j in 0..3u8 {
                let slot = H256::from_bytes([j; 32]);
                let value = H256::from_bytes([i + j; 32]);
                cache.set_storage(addr, slot, value).unwrap();
            }
        }

        // Commit all at once
        state.commit(&cache).unwrap();

        // Verify all changes
        for i in 0..5u8 {
            let addr = Address::from_bytes([i; 20]);
            let account = state.get_account(&addr).unwrap().unwrap();
            assert_eq!(account.nonce, i as u64);
            assert_eq!(account.balance, (i as u128) * 100);

            for j in 0..3u8 {
                let slot = H256::from_bytes([j; 32]);
                let expected = H256::from_bytes([i + j; 32]);
                assert_eq!(state.get_storage(&addr, &slot).unwrap(), expected);
            }
        }

        state.database().close();
        cleanup(&path);
    }

    // ==================== StateReader Helper Methods ====================

    #[test]
    fn test_state_reader_helpers() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state = StateDb::new(db);
        let addr = Address::from_bytes([0x42; 20]);

        // account_exists
        assert!(!state.account_exists(&addr).unwrap());

        state.set_account(addr, Account {
            nonce: 5,
            balance: 1000,
            code_hash: H256::from_bytes([0xcc; 32]),
            ..Default::default()
        }).unwrap();

        assert!(state.account_exists(&addr).unwrap());

        // get_nonce
        assert_eq!(state.get_nonce(&addr).unwrap(), 5);

        // get_balance
        assert_eq!(state.get_balance(&addr).unwrap(), 1000);

        // get_code_hash
        assert_eq!(state.get_code_hash(&addr).unwrap(), H256::from_bytes([0xcc; 32]));

        // Non-existent account
        let addr2 = Address::from_bytes([0x00; 20]);
        assert_eq!(state.get_nonce(&addr2).unwrap(), 0);
        assert_eq!(state.get_balance(&addr2).unwrap(), 0);
        assert_eq!(state.get_code_hash(&addr2).unwrap(), EMPTY_CODE_HASH);

        state.database().close();
        cleanup(&path);
    }
}
