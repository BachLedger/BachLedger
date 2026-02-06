//! Ownership Table (OT) - tracks which transaction owns each state key

use crate::state_key::{StateKey, TxId};
use dashmap::DashMap;

/// Ownership Table for tracking state key ownership
///
/// In Seamless Scheduling, each state key can be "owned" by at most one
/// transaction at a time. This prevents write-write conflicts.
pub struct OwnershipTable {
    /// Map from state key to owning transaction
    owners: DashMap<StateKey, TxId>,
}

impl OwnershipTable {
    /// Create a new empty ownership table
    pub fn new() -> Self {
        Self {
            owners: DashMap::new(),
        }
    }

    /// Try to acquire ownership of a state key
    ///
    /// Returns `Ok(())` if ownership was acquired or already owned by this tx.
    /// Returns `Err(owner_tx_id)` if another transaction owns this key.
    pub fn try_acquire(&self, key: &StateKey, tx_id: TxId) -> Result<(), TxId> {
        match self.owners.entry(key.clone()) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                let owner = *entry.get();
                if owner == tx_id {
                    Ok(())
                } else {
                    Err(owner)
                }
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(tx_id);
                Ok(())
            }
        }
    }

    /// Release ownership of a state key
    ///
    /// Uses atomic remove_if to avoid TOCTOU race condition.
    pub fn release(&self, key: &StateKey, tx_id: TxId) -> bool {
        self.owners
            .remove_if(key, |_, owner| *owner == tx_id)
            .is_some()
    }

    /// Release all state keys owned by a transaction
    pub fn release_all(&self, tx_id: TxId) {
        self.owners.retain(|_, &mut owner| owner != tx_id);
    }

    /// Get the owner of a state key
    pub fn get_owner(&self, key: &StateKey) -> Option<TxId> {
        self.owners.get(key).map(|entry| *entry)
    }

    /// Check if a transaction owns a state key
    pub fn is_owner(&self, key: &StateKey, tx_id: TxId) -> bool {
        self.owners.get(key).map(|entry| *entry == tx_id).unwrap_or(false)
    }

    /// Get all keys owned by a transaction
    pub fn get_owned_keys(&self, tx_id: TxId) -> Vec<StateKey> {
        self.owners
            .iter()
            .filter(|entry| *entry.value() == tx_id)
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get the total number of owned keys
    pub fn len(&self) -> usize {
        self.owners.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.owners.is_empty()
    }

    /// Clear all ownership
    pub fn clear(&self) {
        self.owners.clear();
    }
}

impl Default for OwnershipTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bach_primitives::{Address, H256};
    use std::sync::Arc;
    use std::thread;

    fn make_key(addr_byte: u8, slot_byte: u8) -> StateKey {
        StateKey::new(
            Address::from_bytes([addr_byte; 20]),
            H256::from_bytes([slot_byte; 32]),
        )
    }

    #[test]
    fn test_acquire_ownership() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);

        assert!(table.try_acquire(&key, tx1).is_ok());
        assert!(table.is_owner(&key, tx1));
    }

    #[test]
    fn test_acquire_already_owned() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);

        // First acquisition
        assert!(table.try_acquire(&key, tx1).is_ok());
        // Same tx can acquire again
        assert!(table.try_acquire(&key, tx1).is_ok());
    }

    #[test]
    fn test_acquire_conflict() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        assert!(table.try_acquire(&key, tx1).is_ok());
        assert_eq!(table.try_acquire(&key, tx2), Err(tx1));
    }

    #[test]
    fn test_release_ownership() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        table.try_acquire(&key, tx1).unwrap();
        assert!(table.release(&key, tx1));
        assert!(table.try_acquire(&key, tx2).is_ok());
    }

    #[test]
    fn test_release_all() {
        let table = OwnershipTable::new();
        let key1 = make_key(1, 1);
        let key2 = make_key(1, 2);
        let key3 = make_key(2, 1);
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        table.try_acquire(&key1, tx1).unwrap();
        table.try_acquire(&key2, tx1).unwrap();
        table.try_acquire(&key3, tx2).unwrap();

        assert_eq!(table.len(), 3);
        table.release_all(tx1);
        assert_eq!(table.len(), 1);
        assert!(table.is_owner(&key3, tx2));
    }

    #[test]
    fn test_get_owned_keys() {
        let table = OwnershipTable::new();
        let key1 = make_key(1, 1);
        let key2 = make_key(1, 2);
        let tx1 = TxId::new(1);

        table.try_acquire(&key1, tx1).unwrap();
        table.try_acquire(&key2, tx1).unwrap();

        let owned = table.get_owned_keys(tx1);
        assert_eq!(owned.len(), 2);
        assert!(owned.contains(&key1));
        assert!(owned.contains(&key2));
    }

    // ==================== Additional Basic Tests ====================

    #[test]
    fn test_get_owner() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);

        assert_eq!(table.get_owner(&key), None);

        table.try_acquire(&key, tx1).unwrap();
        assert_eq!(table.get_owner(&key), Some(tx1));

        table.release(&key, tx1);
        assert_eq!(table.get_owner(&key), None);
    }

    #[test]
    fn test_is_owner_nonexistent() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);

        // Key not in table
        assert!(!table.is_owner(&key, tx1));
    }

    #[test]
    fn test_release_wrong_owner() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        table.try_acquire(&key, tx1).unwrap();

        // tx2 cannot release tx1's key
        assert!(!table.release(&key, tx2));
        // Key should still be owned by tx1
        assert!(table.is_owner(&key, tx1));
    }

    #[test]
    fn test_release_nonexistent_key() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);

        // Releasing a key that was never acquired
        assert!(!table.release(&key, tx1));
    }

    #[test]
    fn test_len_and_is_empty() {
        let table = OwnershipTable::new();

        assert!(table.is_empty());
        assert_eq!(table.len(), 0);

        table.try_acquire(&make_key(1, 1), TxId::new(1)).unwrap();
        assert!(!table.is_empty());
        assert_eq!(table.len(), 1);

        table.try_acquire(&make_key(2, 2), TxId::new(2)).unwrap();
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn test_clear() {
        let table = OwnershipTable::new();

        for i in 0..10 {
            table.try_acquire(&make_key(i, i), TxId::new(i as u32)).unwrap();
        }

        assert_eq!(table.len(), 10);

        table.clear();

        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_default() {
        let table = OwnershipTable::default();
        assert!(table.is_empty());
    }

    // ==================== Concurrency Tests ====================

    #[test]
    fn test_concurrent_acquire_different_keys() {
        let table = Arc::new(OwnershipTable::new());
        let mut handles = vec![];

        // 10 threads, each acquiring a different key
        for i in 0..10 {
            let table = Arc::clone(&table);
            let handle = thread::spawn(move || {
                let key = make_key(i, i);
                let tx = TxId::new(i as u32);
                table.try_acquire(&key, tx).unwrap();
                assert!(table.is_owner(&key, tx));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(table.len(), 10);
    }

    #[test]
    fn test_concurrent_acquire_same_key() {
        let table = Arc::new(OwnershipTable::new());
        let key = make_key(1, 1);
        let mut handles = vec![];

        use std::sync::atomic::{AtomicUsize, Ordering};
        let success_count = Arc::new(AtomicUsize::new(0));

        // 10 threads all trying to acquire the same key
        for i in 0..10 {
            let table = Arc::clone(&table);
            let key = key.clone();
            let success_count = Arc::clone(&success_count);
            let handle = thread::spawn(move || {
                let tx = TxId::new(i);
                if table.try_acquire(&key, tx).is_ok() {
                    success_count.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Exactly one thread should have succeeded
        assert_eq!(success_count.load(Ordering::SeqCst), 1);
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_concurrent_acquire_release() {
        let table = Arc::new(OwnershipTable::new());
        let key = make_key(1, 1);
        let mut handles = vec![];

        // Multiple rounds of acquire/release
        for round in 0..5 {
            let table = Arc::clone(&table);
            let key = key.clone();
            let handle = thread::spawn(move || {
                let tx = TxId::new(round);
                // Try to acquire
                if table.try_acquire(&key, tx).is_ok() {
                    // Do some work
                    std::thread::sleep(std::time::Duration::from_micros(10));
                    // Release
                    table.release(&key, tx);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // After all threads complete, the key should be released
        // (or owned by the last thread that acquired it)
        // The table should have at most 1 entry
        assert!(table.len() <= 1);
    }

    #[test]
    fn test_concurrent_release_all() {
        let table = Arc::new(OwnershipTable::new());
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        // tx1 acquires 5 keys
        for i in 0..5 {
            table.try_acquire(&make_key(1, i), tx1).unwrap();
        }

        // tx2 acquires 5 different keys
        for i in 5..10 {
            table.try_acquire(&make_key(2, i), tx2).unwrap();
        }

        assert_eq!(table.len(), 10);

        let table1 = Arc::clone(&table);
        let table2 = Arc::clone(&table);

        // Concurrently release all keys for both transactions
        let h1 = thread::spawn(move || {
            table1.release_all(tx1);
        });

        let h2 = thread::spawn(move || {
            table2.release_all(tx2);
        });

        h1.join().unwrap();
        h2.join().unwrap();

        assert!(table.is_empty());
    }

    // ==================== Stress Tests ====================

    #[test]
    fn test_many_keys_single_owner() {
        let table = OwnershipTable::new();
        let tx = TxId::new(1);

        // Acquire 1000 keys
        for i in 0..1000u16 {
            let key = StateKey::new(
                Address::from_bytes([0x42; 20]),
                H256::from_bytes({
                    let mut bytes = [0u8; 32];
                    bytes[0] = (i >> 8) as u8;
                    bytes[1] = (i & 0xff) as u8;
                    bytes
                }),
            );
            table.try_acquire(&key, tx).unwrap();
        }

        assert_eq!(table.len(), 1000);

        let owned = table.get_owned_keys(tx);
        assert_eq!(owned.len(), 1000);

        table.release_all(tx);
        assert!(table.is_empty());
    }

    #[test]
    fn test_many_owners_single_key_each() {
        let table = OwnershipTable::new();

        // 100 different owners, each with one key
        for i in 0..100u8 {
            let key = make_key(i, i);
            let tx = TxId::new(i as u32);
            table.try_acquire(&key, tx).unwrap();
        }

        assert_eq!(table.len(), 100);

        // Verify each owner
        for i in 0..100u8 {
            let key = make_key(i, i);
            let tx = TxId::new(i as u32);
            assert!(table.is_owner(&key, tx));
        }
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_reacquire_after_release() {
        let table = OwnershipTable::new();
        let key = make_key(1, 1);
        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        // tx1 acquires, releases
        table.try_acquire(&key, tx1).unwrap();
        table.release(&key, tx1);

        // tx2 can now acquire
        assert!(table.try_acquire(&key, tx2).is_ok());
        assert!(table.is_owner(&key, tx2));
        assert!(!table.is_owner(&key, tx1));
    }

    #[test]
    fn test_get_owned_keys_empty() {
        let table = OwnershipTable::new();
        let tx = TxId::new(1);

        let owned = table.get_owned_keys(tx);
        assert!(owned.is_empty());
    }

    #[test]
    fn test_multiple_transactions_interleaved() {
        let table = OwnershipTable::new();

        // tx1 acquires key1, key2
        // tx2 acquires key3, key4
        // tx3 acquires key5
        let key1 = make_key(1, 1);
        let key2 = make_key(1, 2);
        let key3 = make_key(2, 1);
        let key4 = make_key(2, 2);
        let key5 = make_key(3, 1);

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);
        let tx3 = TxId::new(3);

        table.try_acquire(&key1, tx1).unwrap();
        table.try_acquire(&key3, tx2).unwrap();
        table.try_acquire(&key2, tx1).unwrap();
        table.try_acquire(&key5, tx3).unwrap();
        table.try_acquire(&key4, tx2).unwrap();

        assert_eq!(table.get_owned_keys(tx1).len(), 2);
        assert_eq!(table.get_owned_keys(tx2).len(), 2);
        assert_eq!(table.get_owned_keys(tx3).len(), 1);

        // Release tx2's keys
        table.release_all(tx2);

        assert_eq!(table.get_owned_keys(tx1).len(), 2);
        assert_eq!(table.get_owned_keys(tx2).len(), 0);
        assert_eq!(table.get_owned_keys(tx3).len(), 1);
        assert_eq!(table.len(), 3);
    }
}
