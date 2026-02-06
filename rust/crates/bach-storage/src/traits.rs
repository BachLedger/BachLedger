//! Storage traits for state access

use crate::error::StorageResult;
use bach_primitives::{Address, H256};

/// Account data
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Account {
    /// Account nonce
    pub nonce: u64,
    /// Account balance
    pub balance: u128,
    /// Code hash (keccak256 of code, or EMPTY_CODE_HASH if no code)
    pub code_hash: H256,
    /// Storage root (merkle root of storage trie)
    pub storage_root: H256,
}

/// Empty code hash (keccak256 of empty bytes)
pub const EMPTY_CODE_HASH: H256 = H256::from_bytes([
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c,
    0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b,
    0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);

/// Empty storage root (keccak256 of RLP encoded empty list)
pub const EMPTY_STORAGE_ROOT: H256 = H256::from_bytes([
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6,
    0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0,
    0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
]);

impl Account {
    /// Create a new empty account
    pub fn new() -> Self {
        Self {
            nonce: 0,
            balance: 0,
            code_hash: EMPTY_CODE_HASH,
            storage_root: EMPTY_STORAGE_ROOT,
        }
    }

    /// Check if account is empty (EIP-161)
    pub fn is_empty(&self) -> bool {
        self.nonce == 0 && self.balance == 0 && self.code_hash == EMPTY_CODE_HASH
    }

    /// Check if account has code
    pub fn has_code(&self) -> bool {
        self.code_hash != EMPTY_CODE_HASH
    }

    /// Serialize account to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + 16 + 32 + 32);
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.balance.to_le_bytes());
        bytes.extend_from_slice(self.code_hash.as_bytes());
        bytes.extend_from_slice(self.storage_root.as_bytes());
        bytes
    }

    /// Deserialize account from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 8 + 16 + 32 + 32 {
            return None;
        }
        let nonce = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let balance = u128::from_le_bytes(bytes[8..24].try_into().ok()?);
        let code_hash = H256::from_slice(&bytes[24..56]).ok()?;
        let storage_root = H256::from_slice(&bytes[56..88]).ok()?;
        Some(Self {
            nonce,
            balance,
            code_hash,
            storage_root,
        })
    }
}

/// Read access to state
pub trait StateReader {
    /// Get account by address
    fn get_account(&self, address: &Address) -> StorageResult<Option<Account>>;

    /// Get storage value
    fn get_storage(&self, address: &Address, key: &H256) -> StorageResult<H256>;

    /// Get contract code by hash
    fn get_code(&self, code_hash: &H256) -> StorageResult<Option<Vec<u8>>>;

    /// Check if account exists
    fn account_exists(&self, address: &Address) -> StorageResult<bool> {
        Ok(self.get_account(address)?.is_some())
    }

    /// Get account nonce
    fn get_nonce(&self, address: &Address) -> StorageResult<u64> {
        Ok(self.get_account(address)?.map(|a| a.nonce).unwrap_or(0))
    }

    /// Get account balance
    fn get_balance(&self, address: &Address) -> StorageResult<u128> {
        Ok(self.get_account(address)?.map(|a| a.balance).unwrap_or(0))
    }

    /// Get account code hash
    fn get_code_hash(&self, address: &Address) -> StorageResult<H256> {
        Ok(self.get_account(address)?.map(|a| a.code_hash).unwrap_or(EMPTY_CODE_HASH))
    }
}

/// Write access to state
pub trait StateWriter {
    /// Set account
    fn set_account(&mut self, address: Address, account: Account) -> StorageResult<()>;

    /// Delete account
    fn delete_account(&mut self, address: &Address) -> StorageResult<()>;

    /// Set storage value
    fn set_storage(&mut self, address: Address, key: H256, value: H256) -> StorageResult<()>;

    /// Set contract code
    fn set_code(&mut self, code_hash: H256, code: Vec<u8>) -> StorageResult<()>;

    /// Increment nonce
    fn increment_nonce(&mut self, address: &Address) -> StorageResult<u64>
    where
        Self: StateReader,
    {
        let mut account = self.get_account(address)?.unwrap_or_default();
        account.nonce = account.nonce
            .checked_add(1)
            .ok_or_else(|| crate::error::StorageError::InvalidFormat("nonce overflow".into()))?;
        let new_nonce = account.nonce;
        self.set_account(*address, account)?;
        Ok(new_nonce)
    }

    /// Add to balance
    fn add_balance(&mut self, address: &Address, amount: u128) -> StorageResult<()>
    where
        Self: StateReader,
    {
        let mut account = self.get_account(address)?.unwrap_or_default();
        account.balance = account.balance.saturating_add(amount);
        self.set_account(*address, account)
    }

    /// Subtract from balance (returns false if insufficient)
    fn sub_balance(&mut self, address: &Address, amount: u128) -> StorageResult<bool>
    where
        Self: StateReader,
    {
        let mut account = self.get_account(address)?.unwrap_or_default();
        if account.balance < amount {
            return Ok(false);
        }
        account.balance -= amount;
        self.set_account(*address, account)?;
        Ok(true)
    }
}

/// Combined read/write state access
pub trait State: StateReader + StateWriter {}

impl<T: StateReader + StateWriter> State for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_account() {
        let account = Account::new();
        assert!(account.is_empty());
        assert!(!account.has_code());
    }

    #[test]
    fn test_account_with_balance() {
        let mut account = Account::new();
        account.balance = 100;
        assert!(!account.is_empty());
    }

    #[test]
    fn test_account_serialization() {
        let account = Account {
            nonce: 42,
            balance: 1000,
            code_hash: H256::from_bytes([0x01; 32]),
            storage_root: H256::from_bytes([0x02; 32]),
        };

        let bytes = account.to_bytes();
        let recovered = Account::from_bytes(&bytes).unwrap();

        assert_eq!(account, recovered);
    }

    #[test]
    fn test_account_serialization_length() {
        let account = Account::new();
        let bytes = account.to_bytes();
        assert_eq!(bytes.len(), 88); // 8 + 16 + 32 + 32
    }

    // ==================== Additional Account Tests ====================

    #[test]
    fn test_account_default() {
        // Default trait derives zero values, not the semantic "empty" values
        let account = Account::default();
        assert_eq!(account.nonce, 0);
        assert_eq!(account.balance, 0);
        // Note: Default gives H256::ZERO, use Account::new() for EMPTY_CODE_HASH
        assert_eq!(account.code_hash, H256::ZERO);
        assert_eq!(account.storage_root, H256::ZERO);
    }

    #[test]
    fn test_account_with_nonce() {
        let mut account = Account::new();
        account.nonce = 1;
        assert!(!account.is_empty());
    }

    #[test]
    fn test_account_with_code() {
        let mut account = Account::new();
        account.code_hash = H256::from_bytes([0x42; 32]);
        assert!(!account.is_empty());
        assert!(account.has_code());
    }

    #[test]
    fn test_account_has_code_empty() {
        let account = Account::new();
        assert!(!account.has_code());
        assert_eq!(account.code_hash, EMPTY_CODE_HASH);
    }

    #[test]
    fn test_account_clone() {
        let account = Account {
            nonce: 10,
            balance: 500,
            code_hash: H256::from_bytes([0xaa; 32]),
            storage_root: H256::from_bytes([0xbb; 32]),
        };
        let cloned = account.clone();
        assert_eq!(account, cloned);
    }

    #[test]
    fn test_account_equality() {
        let account1 = Account {
            nonce: 1,
            balance: 100,
            code_hash: EMPTY_CODE_HASH,
            storage_root: EMPTY_STORAGE_ROOT,
        };
        let account2 = account1.clone();
        let mut account3 = account1.clone();
        account3.nonce = 2;

        assert_eq!(account1, account2);
        assert_ne!(account1, account3);
    }

    #[test]
    fn test_account_serialization_max_values() {
        let account = Account {
            nonce: u64::MAX,
            balance: u128::MAX,
            code_hash: H256::from_bytes([0xff; 32]),
            storage_root: H256::from_bytes([0xff; 32]),
        };

        let bytes = account.to_bytes();
        let recovered = Account::from_bytes(&bytes).unwrap();

        assert_eq!(account, recovered);
        assert_eq!(recovered.nonce, u64::MAX);
        assert_eq!(recovered.balance, u128::MAX);
    }

    #[test]
    fn test_account_from_bytes_invalid_length() {
        let short_bytes = vec![0u8; 50]; // Too short
        assert!(Account::from_bytes(&short_bytes).is_none());

        let long_bytes = vec![0u8; 100]; // Too long
        assert!(Account::from_bytes(&long_bytes).is_none());

        let empty_bytes: Vec<u8> = vec![];
        assert!(Account::from_bytes(&empty_bytes).is_none());
    }

    // ==================== Constants Tests ====================

    #[test]
    fn test_empty_code_hash_constant() {
        // keccak256("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        assert_eq!(EMPTY_CODE_HASH.as_bytes()[0], 0xc5);
        assert_eq!(EMPTY_CODE_HASH.as_bytes()[1], 0xd2);
    }

    #[test]
    fn test_empty_storage_root_constant() {
        // keccak256(RLP([]))
        assert_eq!(EMPTY_STORAGE_ROOT.as_bytes()[0], 0x56);
        assert_eq!(EMPTY_STORAGE_ROOT.as_bytes()[1], 0xe8);
    }

    // ==================== Empty Account EIP-161 Tests ====================

    #[test]
    fn test_eip161_empty_account() {
        // EIP-161: Account is empty if nonce = 0, balance = 0, code = empty
        let account = Account::new();
        assert!(account.is_empty());

        // Any non-zero field makes it non-empty
        let mut with_nonce = Account::new();
        with_nonce.nonce = 1;
        assert!(!with_nonce.is_empty());

        let mut with_balance = Account::new();
        with_balance.balance = 1;
        assert!(!with_balance.is_empty());

        let mut with_code = Account::new();
        with_code.code_hash = H256::from_bytes([0x42; 32]);
        assert!(!with_code.is_empty());
    }

    #[test]
    fn test_account_storage_root_does_not_affect_empty() {
        // Storage root alone doesn't affect is_empty (only nonce, balance, code)
        let mut account = Account::new();
        account.storage_root = H256::from_bytes([0x42; 32]);
        assert!(account.is_empty()); // Still empty by EIP-161 rules
    }
}
