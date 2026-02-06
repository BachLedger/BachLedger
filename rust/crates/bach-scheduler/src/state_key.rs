//! State key types for ownership tracking

use bach_primitives::{Address, H256};

/// A unique identifier for a piece of state (storage slot)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateKey {
    /// Contract address
    pub address: Address,
    /// Storage slot
    pub slot: H256,
}

impl StateKey {
    /// Create a new state key
    pub fn new(address: Address, slot: H256) -> Self {
        Self { address, slot }
    }

    /// Create state key for account balance
    pub fn balance(address: Address) -> Self {
        Self {
            address,
            slot: H256::ZERO,
        }
    }

    /// Create state key for account nonce
    pub fn nonce(address: Address) -> Self {
        Self {
            address,
            slot: H256::from_bytes([0x01; 32]),
        }
    }

    /// Create state key for contract code
    pub fn code(address: Address) -> Self {
        Self {
            address,
            slot: H256::from_bytes([0x02; 32]),
        }
    }
}

/// Transaction identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TxId(pub u32);

impl TxId {
    /// Create a new transaction ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for TxId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<usize> for TxId {
    fn from(id: usize) -> Self {
        Self(id as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_state_key_creation() {
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);
        let key = StateKey::new(addr, slot);

        assert_eq!(key.address, addr);
        assert_eq!(key.slot, slot);
    }

    #[test]
    fn test_state_key_equality() {
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);

        let key1 = StateKey::new(addr, slot);
        let key2 = StateKey::new(addr, slot);
        let key3 = StateKey::new(addr, H256::ZERO);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_tx_id() {
        let id1 = TxId::new(1);
        let id2 = TxId::from(2u32);
        let id3 = TxId::from(3usize);

        assert_eq!(id1.as_u32(), 1);
        assert_eq!(id2.as_u32(), 2);
        assert_eq!(id3.as_u32(), 3);
        assert!(id1 < id2);
    }

    // ==================== StateKey Special Constructors ====================

    #[test]
    fn test_state_key_balance() {
        let addr = Address::from_bytes([0x42; 20]);
        let key = StateKey::balance(addr);

        assert_eq!(key.address, addr);
        assert_eq!(key.slot, H256::ZERO);
    }

    #[test]
    fn test_state_key_nonce() {
        let addr = Address::from_bytes([0x42; 20]);
        let key = StateKey::nonce(addr);

        assert_eq!(key.address, addr);
        assert_eq!(key.slot, H256::from_bytes([0x01; 32]));
    }

    #[test]
    fn test_state_key_code() {
        let addr = Address::from_bytes([0x42; 20]);
        let key = StateKey::code(addr);

        assert_eq!(key.address, addr);
        assert_eq!(key.slot, H256::from_bytes([0x02; 32]));
    }

    #[test]
    fn test_state_key_special_types_distinct() {
        let addr = Address::from_bytes([0x42; 20]);

        let balance = StateKey::balance(addr);
        let nonce = StateKey::nonce(addr);
        let code = StateKey::code(addr);

        // All three should be different
        assert_ne!(balance, nonce);
        assert_ne!(balance, code);
        assert_ne!(nonce, code);
    }

    // ==================== StateKey Hash/Clone ====================

    #[test]
    fn test_state_key_hash_consistency() {
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);

        let key1 = StateKey::new(addr, slot);
        let key2 = StateKey::new(addr, slot);

        let mut set = HashSet::new();
        set.insert(key1.clone());

        // Same key should be found
        assert!(set.contains(&key2));
        // Insert duplicate should not increase size
        set.insert(key2);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_state_key_as_hashmap_key() {
        let addr1 = Address::from_bytes([0x01; 20]);
        let addr2 = Address::from_bytes([0x02; 20]);

        let key1 = StateKey::balance(addr1);
        let key2 = StateKey::balance(addr2);

        let mut map: HashMap<StateKey, u64> = HashMap::new();
        map.insert(key1.clone(), 100);
        map.insert(key2.clone(), 200);

        assert_eq!(map.get(&key1), Some(&100));
        assert_eq!(map.get(&key2), Some(&200));
    }

    #[test]
    fn test_state_key_clone() {
        let addr = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);
        let key = StateKey::new(addr, slot);
        let cloned = key.clone();

        assert_eq!(key, cloned);
        assert_eq!(key.address, cloned.address);
        assert_eq!(key.slot, cloned.slot);
    }

    // ==================== TxId Tests ====================

    #[test]
    fn test_tx_id_ordering() {
        let ids: Vec<TxId> = (0..10).map(TxId::new).collect();

        for i in 0..9 {
            assert!(ids[i] < ids[i + 1]);
            assert!(ids[i + 1] > ids[i]);
            assert!(ids[i] <= ids[i]);
            assert!(ids[i] >= ids[i]);
        }
    }

    #[test]
    fn test_tx_id_boundary_values() {
        let min = TxId::new(0);
        let max = TxId::new(u32::MAX);

        assert_eq!(min.as_u32(), 0);
        assert_eq!(max.as_u32(), u32::MAX);
        assert!(min < max);
    }

    #[test]
    fn test_tx_id_copy() {
        let id = TxId::new(42);
        let copied = id; // Copy, not move
        let also_copied = id; // Can use again because Copy

        assert_eq!(id, copied);
        assert_eq!(id, also_copied);
    }

    #[test]
    fn test_tx_id_hash_consistency() {
        let mut set: HashSet<TxId> = HashSet::new();

        for i in 0..100 {
            set.insert(TxId::new(i));
        }

        assert_eq!(set.len(), 100);

        // Duplicates should not increase size
        for i in 0..100 {
            set.insert(TxId::new(i));
        }
        assert_eq!(set.len(), 100);
    }

    #[test]
    fn test_tx_id_from_usize_truncation() {
        // Test that large usize values are truncated to u32
        let large_usize = (u32::MAX as usize) + 1;
        let id = TxId::from(large_usize);
        // Should wrap around to 0
        assert_eq!(id.as_u32(), 0);
    }

    // ==================== StateKey Different Addresses/Slots ====================

    #[test]
    fn test_state_key_different_address_same_slot() {
        let addr1 = Address::from_bytes([0x01; 20]);
        let addr2 = Address::from_bytes([0x02; 20]);
        let slot = H256::from_bytes([0x42; 32]);

        let key1 = StateKey::new(addr1, slot);
        let key2 = StateKey::new(addr2, slot);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_state_key_same_address_different_slot() {
        let addr = Address::from_bytes([0x42; 20]);
        let slot1 = H256::from_bytes([0x01; 32]);
        let slot2 = H256::from_bytes([0x02; 32]);

        let key1 = StateKey::new(addr, slot1);
        let key2 = StateKey::new(addr, slot2);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_state_key_zero_values() {
        let addr = Address::ZERO;
        let slot = H256::ZERO;
        let key = StateKey::new(addr, slot);

        assert_eq!(key.address, Address::ZERO);
        assert_eq!(key.slot, H256::ZERO);

        // Should equal balance key for zero address
        assert_eq!(key, StateKey::balance(Address::ZERO));
    }
}
