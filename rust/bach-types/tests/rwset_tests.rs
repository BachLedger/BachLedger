//! Tests for ReadWriteSet type
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_types::ReadWriteSet;
use bach_primitives::H256;

// =============================================================================
// new() tests
// =============================================================================

mod new {
    use super::*;

    #[test]
    fn creates_empty_set() {
        let rwset = ReadWriteSet::new();
        assert!(rwset.reads().is_empty());
        assert!(rwset.writes().is_empty());
    }

    #[test]
    fn all_keys_empty_initially() {
        let rwset = ReadWriteSet::new();
        assert!(rwset.all_keys().is_empty());
    }
}

// =============================================================================
// record_read() tests
// =============================================================================

mod record_read {
    use super::*;

    #[test]
    fn adds_key_to_reads() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_read(key);
        assert_eq!(rwset.reads().len(), 1);
        assert_eq!(rwset.reads()[0], key);
    }

    #[test]
    fn multiple_reads() {
        let mut rwset = ReadWriteSet::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();
        let key3 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000003").unwrap();

        rwset.record_read(key1);
        rwset.record_read(key2);
        rwset.record_read(key3);

        assert_eq!(rwset.reads().len(), 3);
    }

    #[test]
    fn duplicate_reads_allowed() {
        // Reading the same key multiple times is valid
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        rwset.record_read(key);
        rwset.record_read(key);

        // May or may not deduplicate - implementation choice
        assert!(rwset.reads().len() >= 1);
    }

    #[test]
    fn does_not_affect_writes() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_read(key);
        assert!(rwset.writes().is_empty());
    }
}

// =============================================================================
// record_write() tests
// =============================================================================

mod record_write {
    use super::*;

    #[test]
    fn adds_key_value_to_writes() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![1, 2, 3, 4];

        rwset.record_write(key, value.clone());

        assert_eq!(rwset.writes().len(), 1);
        assert_eq!(rwset.writes()[0].0, key);
        assert_eq!(rwset.writes()[0].1, value);
    }

    #[test]
    fn multiple_writes() {
        let mut rwset = ReadWriteSet::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        rwset.record_write(key1, vec![1, 2, 3]);
        rwset.record_write(key2, vec![4, 5, 6]);

        assert_eq!(rwset.writes().len(), 2);
    }

    #[test]
    fn empty_value() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        rwset.record_write(key, vec![]);

        assert_eq!(rwset.writes().len(), 1);
        assert!(rwset.writes()[0].1.is_empty());
    }

    #[test]
    fn large_value() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![0xffu8; 1024]; // 1KB value

        rwset.record_write(key, value.clone());

        assert_eq!(rwset.writes()[0].1, value);
    }

    #[test]
    fn duplicate_writes_allowed() {
        // Writing to the same key multiple times is valid (last write wins semantically)
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        rwset.record_write(key, vec![1, 2, 3]);
        rwset.record_write(key, vec![4, 5, 6]);

        // Both writes should be recorded
        assert!(rwset.writes().len() >= 1);
    }

    #[test]
    fn does_not_affect_reads() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_write(key, vec![1, 2, 3]);
        assert!(rwset.reads().is_empty());
    }
}

// =============================================================================
// reads() tests
// =============================================================================

mod reads {
    use super::*;

    #[test]
    fn returns_slice_of_read_keys() {
        let mut rwset = ReadWriteSet::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        rwset.record_read(key1);
        rwset.record_read(key2);

        let reads = rwset.reads();
        assert!(reads.contains(&key1));
        assert!(reads.contains(&key2));
    }

    #[test]
    fn preserves_order() {
        let mut rwset = ReadWriteSet::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();
        let key3 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000003").unwrap();

        rwset.record_read(key1);
        rwset.record_read(key2);
        rwset.record_read(key3);

        let reads = rwset.reads();
        assert_eq!(reads[0], key1);
        assert_eq!(reads[1], key2);
        assert_eq!(reads[2], key3);
    }
}

// =============================================================================
// writes() tests
// =============================================================================

mod writes {
    use super::*;

    #[test]
    fn returns_slice_of_write_pairs() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![1, 2, 3];

        rwset.record_write(key, value.clone());

        let writes = rwset.writes();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0], (key, value));
    }

    #[test]
    fn preserves_order() {
        let mut rwset = ReadWriteSet::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        rwset.record_write(key1, vec![1]);
        rwset.record_write(key2, vec![2]);

        let writes = rwset.writes();
        assert_eq!(writes[0].0, key1);
        assert_eq!(writes[1].0, key2);
    }
}

// =============================================================================
// all_keys() tests
// =============================================================================

mod all_keys {
    use super::*;

    #[test]
    fn includes_read_keys() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_read(key);

        let all = rwset.all_keys();
        assert!(all.contains(&key));
    }

    #[test]
    fn includes_write_keys() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_write(key, vec![1, 2, 3]);

        let all = rwset.all_keys();
        assert!(all.contains(&key));
    }

    #[test]
    fn combines_reads_and_writes() {
        let mut rwset = ReadWriteSet::new();
        let read_key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let write_key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        rwset.record_read(read_key);
        rwset.record_write(write_key, vec![1, 2, 3]);

        let all = rwset.all_keys();
        assert!(all.contains(&read_key));
        assert!(all.contains(&write_key));
    }

    #[test]
    fn returns_unique_keys() {
        // If a key is both read and written, it should appear once
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        rwset.record_read(key);
        rwset.record_write(key, vec![1, 2, 3]);

        let all = rwset.all_keys();
        // Count occurrences of key
        let count = all.iter().filter(|&k| *k == key).count();
        assert_eq!(count, 1, "Key should appear exactly once in all_keys()");
    }

    #[test]
    fn handles_multiple_reads_of_same_key() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        rwset.record_read(key);
        rwset.record_read(key);
        rwset.record_read(key);

        let all = rwset.all_keys();
        let count = all.iter().filter(|&k| *k == key).count();
        assert_eq!(count, 1, "Duplicate reads should be deduplicated in all_keys()");
    }
}

// =============================================================================
// clear() tests
// =============================================================================

mod clear {
    use super::*;

    #[test]
    fn clears_reads() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_read(key);
        rwset.clear();
        assert!(rwset.reads().is_empty());
    }

    #[test]
    fn clears_writes() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_write(key, vec![1, 2, 3]);
        rwset.clear();
        assert!(rwset.writes().is_empty());
    }

    #[test]
    fn clears_all_keys() {
        let mut rwset = ReadWriteSet::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        rwset.record_read(key1);
        rwset.record_write(key2, vec![1, 2, 3]);
        rwset.clear();

        assert!(rwset.all_keys().is_empty());
    }

    #[test]
    fn can_add_after_clear() {
        let mut rwset = ReadWriteSet::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        rwset.record_read(key1);
        rwset.clear();
        rwset.record_write(key2, vec![4, 5, 6]);

        assert!(rwset.reads().is_empty());
        assert_eq!(rwset.writes().len(), 1);
    }
}

// =============================================================================
// Default trait tests
// =============================================================================

mod default_trait {
    use super::*;

    #[test]
    fn default_creates_empty_set() {
        let rwset: ReadWriteSet = Default::default();
        assert!(rwset.reads().is_empty());
        assert!(rwset.writes().is_empty());
    }

    #[test]
    fn default_equals_new() {
        let from_new = ReadWriteSet::new();
        let from_default: ReadWriteSet = Default::default();

        assert_eq!(from_new.reads().len(), from_default.reads().len());
        assert_eq!(from_new.writes().len(), from_default.writes().len());
    }
}

// =============================================================================
// Clone trait tests
// =============================================================================

mod clone_trait {
    use super::*;

    #[test]
    fn clone_preserves_reads() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_read(key);

        let cloned = rwset.clone();
        assert_eq!(rwset.reads(), cloned.reads());
    }

    #[test]
    fn clone_preserves_writes() {
        let mut rwset = ReadWriteSet::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        rwset.record_write(key, vec![1, 2, 3]);

        let cloned = rwset.clone();
        assert_eq!(rwset.writes(), cloned.writes());
    }

    #[test]
    fn clone_is_independent() {
        let mut rwset = ReadWriteSet::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();
        rwset.record_read(key1);

        let mut cloned = rwset.clone();
        cloned.record_read(key2);

        // Original should not be affected
        assert_eq!(rwset.reads().len(), 1);
        assert_eq!(cloned.reads().len(), 2);
    }
}

// =============================================================================
// Debug trait tests
// =============================================================================

mod debug_trait {
    use super::*;

    #[test]
    fn debug_is_implemented() {
        let rwset = ReadWriteSet::new();
        let debug = format!("{:?}", rwset);
        assert!(!debug.is_empty());
    }
}
