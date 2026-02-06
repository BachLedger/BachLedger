//! Read/Write set tracking for transactions

use crate::state_key::StateKey;
use std::collections::HashSet;

/// Read/Write set for a transaction
///
/// Tracks which state keys a transaction reads and writes.
/// Used for conflict detection and dependency analysis.
#[derive(Clone, Debug, Default)]
pub struct RWSet {
    /// State keys that were read
    pub reads: HashSet<StateKey>,
    /// State keys that were written
    pub writes: HashSet<StateKey>,
}

impl RWSet {
    /// Create a new empty RW set
    pub fn new() -> Self {
        Self {
            reads: HashSet::new(),
            writes: HashSet::new(),
        }
    }

    /// Record a read access
    pub fn record_read(&mut self, key: StateKey) {
        self.reads.insert(key);
    }

    /// Record a write access
    pub fn record_write(&mut self, key: StateKey) {
        self.writes.insert(key);
    }

    /// Check if this transaction reads a key
    pub fn reads_key(&self, key: &StateKey) -> bool {
        self.reads.contains(key)
    }

    /// Check if this transaction writes a key
    pub fn writes_key(&self, key: &StateKey) -> bool {
        self.writes.contains(key)
    }

    /// Check for read-write conflict with another transaction
    ///
    /// Returns true if `self` reads a key that `other` writes,
    /// indicating `self` depends on `other` (read-after-write).
    pub fn has_raw_dependency(&self, other: &RWSet) -> bool {
        self.reads.iter().any(|key| other.writes.contains(key))
    }

    /// Check for write-write conflict with another transaction
    ///
    /// Returns true if both transactions write to the same key.
    pub fn has_waw_conflict(&self, other: &RWSet) -> bool {
        self.writes.iter().any(|key| other.writes.contains(key))
    }

    /// Check for write-read conflict (anti-dependency)
    ///
    /// Returns true if `self` writes a key that `other` reads.
    pub fn has_war_conflict(&self, other: &RWSet) -> bool {
        self.writes.iter().any(|key| other.reads.contains(key))
    }

    /// Get all conflicting keys with another RW set
    pub fn get_conflicts(&self, other: &RWSet) -> ConflictSet {
        let raw: HashSet<_> = self.reads.intersection(&other.writes).cloned().collect();
        let waw: HashSet<_> = self.writes.intersection(&other.writes).cloned().collect();
        let war: HashSet<_> = self.writes.intersection(&other.reads).cloned().collect();

        ConflictSet { raw, waw, war }
    }

    /// Get number of read keys
    pub fn read_count(&self) -> usize {
        self.reads.len()
    }

    /// Get number of write keys
    pub fn write_count(&self) -> usize {
        self.writes.len()
    }

    /// Check if the RW set is empty
    pub fn is_empty(&self) -> bool {
        self.reads.is_empty() && self.writes.is_empty()
    }

    /// Merge another RW set into this one
    pub fn merge(&mut self, other: &RWSet) {
        self.reads.extend(other.reads.iter().cloned());
        self.writes.extend(other.writes.iter().cloned());
    }

    /// Clear the RW set
    pub fn clear(&mut self) {
        self.reads.clear();
        self.writes.clear();
    }
}

/// Set of conflicting keys between two transactions
#[derive(Clone, Debug, Default)]
pub struct ConflictSet {
    /// Read-after-write conflicts
    pub raw: HashSet<StateKey>,
    /// Write-after-write conflicts
    pub waw: HashSet<StateKey>,
    /// Write-after-read conflicts (anti-dependency)
    pub war: HashSet<StateKey>,
}

impl ConflictSet {
    /// Check if there are any conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.raw.is_empty() || !self.waw.is_empty() || !self.war.is_empty()
    }

    /// Get total number of conflicting keys
    pub fn total_conflicts(&self) -> usize {
        self.raw.len() + self.waw.len() + self.war.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bach_primitives::{Address, H256};

    fn make_key(id: u8) -> StateKey {
        StateKey::new(
            Address::from_bytes([id; 20]),
            H256::from_bytes([id; 32]),
        )
    }

    #[test]
    fn test_rw_set_basic() {
        let mut rw = RWSet::new();
        let key1 = make_key(1);
        let key2 = make_key(2);

        rw.record_read(key1.clone());
        rw.record_write(key2.clone());

        assert!(rw.reads_key(&key1));
        assert!(!rw.reads_key(&key2));
        assert!(rw.writes_key(&key2));
        assert!(!rw.writes_key(&key1));
    }

    #[test]
    fn test_raw_dependency() {
        let key = make_key(1);

        let mut tx1 = RWSet::new();
        tx1.record_write(key.clone());

        let mut tx2 = RWSet::new();
        tx2.record_read(key.clone());

        // tx2 reads what tx1 writes -> tx2 depends on tx1
        assert!(tx2.has_raw_dependency(&tx1));
        assert!(!tx1.has_raw_dependency(&tx2));
    }

    #[test]
    fn test_waw_conflict() {
        let key = make_key(1);

        let mut tx1 = RWSet::new();
        tx1.record_write(key.clone());

        let mut tx2 = RWSet::new();
        tx2.record_write(key.clone());

        assert!(tx1.has_waw_conflict(&tx2));
        assert!(tx2.has_waw_conflict(&tx1));
    }

    #[test]
    fn test_war_conflict() {
        let key = make_key(1);

        let mut tx1 = RWSet::new();
        tx1.record_write(key.clone());

        let mut tx2 = RWSet::new();
        tx2.record_read(key.clone());

        // tx1 writes what tx2 reads
        assert!(tx1.has_war_conflict(&tx2));
        assert!(!tx2.has_war_conflict(&tx1));
    }

    #[test]
    fn test_no_conflict() {
        let mut tx1 = RWSet::new();
        tx1.record_write(make_key(1));

        let mut tx2 = RWSet::new();
        tx2.record_write(make_key(2));

        assert!(!tx1.has_raw_dependency(&tx2));
        assert!(!tx1.has_waw_conflict(&tx2));
        assert!(!tx1.has_war_conflict(&tx2));
    }

    #[test]
    fn test_conflict_set() {
        let key1 = make_key(1);
        let key2 = make_key(2);
        let key3 = make_key(3);

        let mut tx1 = RWSet::new();
        tx1.record_read(key1.clone());
        tx1.record_write(key2.clone());
        tx1.record_write(key3.clone());

        let mut tx2 = RWSet::new();
        tx2.record_write(key1.clone()); // RAW conflict for tx1
        tx2.record_write(key2.clone()); // WAW conflict
        tx2.record_read(key3.clone());  // WAR conflict for tx1

        let conflicts = tx1.get_conflicts(&tx2);
        assert!(conflicts.has_conflicts());
        assert!(conflicts.raw.contains(&key1));
        assert!(conflicts.waw.contains(&key2));
        assert!(conflicts.war.contains(&key3));
    }

    #[test]
    fn test_merge() {
        let mut rw1 = RWSet::new();
        rw1.record_read(make_key(1));
        rw1.record_write(make_key(2));

        let mut rw2 = RWSet::new();
        rw2.record_read(make_key(3));
        rw2.record_write(make_key(4));

        rw1.merge(&rw2);

        assert_eq!(rw1.read_count(), 2);
        assert_eq!(rw1.write_count(), 2);
    }

    // ==================== Additional RWSet Tests ====================

    #[test]
    fn test_rw_set_new() {
        let rw = RWSet::new();
        assert!(rw.is_empty());
        assert_eq!(rw.read_count(), 0);
        assert_eq!(rw.write_count(), 0);
    }

    #[test]
    fn test_rw_set_default() {
        let rw = RWSet::default();
        assert!(rw.is_empty());
    }

    #[test]
    fn test_rw_set_clear() {
        let mut rw = RWSet::new();
        rw.record_read(make_key(1));
        rw.record_write(make_key(2));

        assert!(!rw.is_empty());

        rw.clear();

        assert!(rw.is_empty());
        assert_eq!(rw.read_count(), 0);
        assert_eq!(rw.write_count(), 0);
    }

    #[test]
    fn test_duplicate_read() {
        let mut rw = RWSet::new();
        let key = make_key(1);

        rw.record_read(key.clone());
        rw.record_read(key.clone());
        rw.record_read(key.clone());

        // HashSet deduplicates
        assert_eq!(rw.read_count(), 1);
    }

    #[test]
    fn test_duplicate_write() {
        let mut rw = RWSet::new();
        let key = make_key(1);

        rw.record_write(key.clone());
        rw.record_write(key.clone());
        rw.record_write(key.clone());

        assert_eq!(rw.write_count(), 1);
    }

    #[test]
    fn test_read_then_write_same_key() {
        let mut rw = RWSet::new();
        let key = make_key(1);

        rw.record_read(key.clone());
        rw.record_write(key.clone());

        // Both sets can contain the same key
        assert!(rw.reads_key(&key));
        assert!(rw.writes_key(&key));
        assert_eq!(rw.read_count(), 1);
        assert_eq!(rw.write_count(), 1);
    }

    // ==================== Complex Conflict Scenarios ====================

    #[test]
    fn test_multiple_raw_dependencies() {
        // tx1 writes key1, key2, key3
        // tx2 reads key1, key2
        let mut tx1 = RWSet::new();
        tx1.record_write(make_key(1));
        tx1.record_write(make_key(2));
        tx1.record_write(make_key(3));

        let mut tx2 = RWSet::new();
        tx2.record_read(make_key(1));
        tx2.record_read(make_key(2));

        assert!(tx2.has_raw_dependency(&tx1));

        let conflicts = tx2.get_conflicts(&tx1);
        assert_eq!(conflicts.raw.len(), 2);
    }

    #[test]
    fn test_multiple_waw_conflicts() {
        let mut tx1 = RWSet::new();
        tx1.record_write(make_key(1));
        tx1.record_write(make_key(2));
        tx1.record_write(make_key(3));

        let mut tx2 = RWSet::new();
        tx2.record_write(make_key(2));
        tx2.record_write(make_key(3));
        tx2.record_write(make_key(4));

        assert!(tx1.has_waw_conflict(&tx2));

        let conflicts = tx1.get_conflicts(&tx2);
        assert_eq!(conflicts.waw.len(), 2); // keys 2, 3
    }

    #[test]
    fn test_all_three_conflict_types() {
        // Comprehensive test with all conflict types
        let key_raw = make_key(1);
        let key_waw = make_key(2);
        let key_war = make_key(3);
        let key_no_conflict = make_key(4);

        let mut tx1 = RWSet::new();
        tx1.record_read(key_raw.clone());    // Will conflict with tx2's write (RAW)
        tx1.record_write(key_waw.clone());   // Will conflict with tx2's write (WAW)
        tx1.record_write(key_war.clone());   // Will conflict with tx2's read (WAR)
        tx1.record_read(key_no_conflict.clone());

        let mut tx2 = RWSet::new();
        tx2.record_write(key_raw.clone());   // RAW: tx1 reads, tx2 writes
        tx2.record_write(key_waw.clone());   // WAW: both write
        tx2.record_read(key_war.clone());    // WAR: tx1 writes, tx2 reads

        let conflicts = tx1.get_conflicts(&tx2);

        assert!(conflicts.has_conflicts());
        assert_eq!(conflicts.raw.len(), 1);
        assert_eq!(conflicts.waw.len(), 1);
        assert_eq!(conflicts.war.len(), 1);
        assert_eq!(conflicts.total_conflicts(), 3);

        assert!(conflicts.raw.contains(&key_raw));
        assert!(conflicts.waw.contains(&key_waw));
        assert!(conflicts.war.contains(&key_war));
    }

    #[test]
    fn test_conflict_set_no_conflicts() {
        let mut tx1 = RWSet::new();
        tx1.record_read(make_key(1));
        tx1.record_write(make_key(2));

        let mut tx2 = RWSet::new();
        tx2.record_read(make_key(3));
        tx2.record_write(make_key(4));

        let conflicts = tx1.get_conflicts(&tx2);

        assert!(!conflicts.has_conflicts());
        assert_eq!(conflicts.total_conflicts(), 0);
    }

    #[test]
    fn test_conflict_set_default() {
        let conflicts = ConflictSet::default();
        assert!(!conflicts.has_conflicts());
        assert_eq!(conflicts.total_conflicts(), 0);
    }

    // ==================== Merge Tests ====================

    #[test]
    fn test_merge_overlapping() {
        let mut rw1 = RWSet::new();
        rw1.record_read(make_key(1));
        rw1.record_read(make_key(2));
        rw1.record_write(make_key(3));

        let mut rw2 = RWSet::new();
        rw2.record_read(make_key(2));  // overlaps
        rw2.record_read(make_key(4));
        rw2.record_write(make_key(3)); // overlaps
        rw2.record_write(make_key(5));

        rw1.merge(&rw2);

        // key2 and key3 overlap, so counts should be 3 and 2
        assert_eq!(rw1.read_count(), 3);  // keys 1, 2, 4
        assert_eq!(rw1.write_count(), 2); // keys 3, 5
    }

    #[test]
    fn test_merge_with_empty() {
        let mut rw1 = RWSet::new();
        rw1.record_read(make_key(1));

        let rw2 = RWSet::new();

        rw1.merge(&rw2);

        assert_eq!(rw1.read_count(), 1);
        assert_eq!(rw1.write_count(), 0);
    }

    #[test]
    fn test_merge_into_empty() {
        let mut rw1 = RWSet::new();

        let mut rw2 = RWSet::new();
        rw2.record_read(make_key(1));
        rw2.record_write(make_key(2));

        rw1.merge(&rw2);

        assert_eq!(rw1.read_count(), 1);
        assert_eq!(rw1.write_count(), 1);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_rw_set_clone() {
        let mut rw = RWSet::new();
        rw.record_read(make_key(1));
        rw.record_write(make_key(2));

        let cloned = rw.clone();

        assert_eq!(cloned.read_count(), rw.read_count());
        assert_eq!(cloned.write_count(), rw.write_count());
        assert!(cloned.reads_key(&make_key(1)));
        assert!(cloned.writes_key(&make_key(2)));
    }

    // ==================== Ethereum-like Scenarios ====================

    #[test]
    fn test_erc20_transfer_rw_set() {
        // Simulating ERC20 transfer: reads sender balance, writes sender and recipient balances
        let sender = Address::from_bytes([0x01; 20]);
        let recipient = Address::from_bytes([0x02; 20]);

        let sender_balance = StateKey::balance(sender);
        let recipient_balance = StateKey::balance(recipient);

        let mut rw = RWSet::new();
        rw.record_read(sender_balance.clone());    // Check sender balance
        rw.record_write(sender_balance.clone());   // Deduct from sender
        rw.record_write(recipient_balance.clone()); // Add to recipient

        assert_eq!(rw.read_count(), 1);
        assert_eq!(rw.write_count(), 2);
    }

    #[test]
    fn test_two_transfers_same_sender_conflict() {
        // Two transfers from the same sender should have WAW conflict on sender balance
        let sender = Address::from_bytes([0x01; 20]);
        let recipient1 = Address::from_bytes([0x02; 20]);
        let recipient2 = Address::from_bytes([0x03; 20]);

        let sender_balance = StateKey::balance(sender);

        let mut tx1 = RWSet::new();
        tx1.record_read(sender_balance.clone());
        tx1.record_write(sender_balance.clone());
        tx1.record_write(StateKey::balance(recipient1));

        let mut tx2 = RWSet::new();
        tx2.record_read(sender_balance.clone());
        tx2.record_write(sender_balance.clone());
        tx2.record_write(StateKey::balance(recipient2));

        // Both WAW and RAW conflicts on sender_balance
        assert!(tx1.has_waw_conflict(&tx2));
        assert!(tx2.has_raw_dependency(&tx1));
    }

    #[test]
    fn test_two_transfers_different_senders_no_conflict() {
        // Two transfers from different senders to different recipients: no conflict
        let sender1 = Address::from_bytes([0x01; 20]);
        let sender2 = Address::from_bytes([0x02; 20]);
        let recipient1 = Address::from_bytes([0x03; 20]);
        let recipient2 = Address::from_bytes([0x04; 20]);

        let mut tx1 = RWSet::new();
        tx1.record_read(StateKey::balance(sender1));
        tx1.record_write(StateKey::balance(sender1));
        tx1.record_write(StateKey::balance(recipient1));

        let mut tx2 = RWSet::new();
        tx2.record_read(StateKey::balance(sender2));
        tx2.record_write(StateKey::balance(sender2));
        tx2.record_write(StateKey::balance(recipient2));

        // No conflicts - can execute in parallel
        assert!(!tx1.has_raw_dependency(&tx2));
        assert!(!tx1.has_waw_conflict(&tx2));
        assert!(!tx1.has_war_conflict(&tx2));
    }

    #[test]
    fn test_storage_slot_conflict() {
        // Two transactions accessing the same contract storage slot
        let contract = Address::from_bytes([0x42; 20]);
        let slot = H256::from_bytes([0x01; 32]);
        let key = StateKey::new(contract, slot);

        let mut tx1 = RWSet::new();
        tx1.record_read(key.clone());
        tx1.record_write(key.clone());

        let mut tx2 = RWSet::new();
        tx2.record_read(key.clone());
        tx2.record_write(key.clone());

        assert!(tx1.has_waw_conflict(&tx2));
        assert!(tx2.has_raw_dependency(&tx1));
    }

    // ==================== Large RW Sets ====================

    #[test]
    fn test_large_rw_set() {
        let mut rw = RWSet::new();

        // 100 reads, 100 writes
        for i in 0..100 {
            rw.record_read(make_key(i));
        }
        for i in 100..200 {
            rw.record_write(make_key(i));
        }

        assert_eq!(rw.read_count(), 100);
        assert_eq!(rw.write_count(), 100);
        assert!(!rw.is_empty());
    }

    #[test]
    fn test_conflict_detection_large_sets() {
        let mut tx1 = RWSet::new();
        let mut tx2 = RWSet::new();

        // tx1 writes keys 0-49
        for i in 0..50 {
            tx1.record_write(make_key(i));
        }

        // tx2 reads keys 25-74 (overlap: 25-49)
        for i in 25..75 {
            tx2.record_read(make_key(i));
        }

        assert!(tx2.has_raw_dependency(&tx1));

        let conflicts = tx2.get_conflicts(&tx1);
        assert_eq!(conflicts.raw.len(), 25); // keys 25-49
    }
}
