//! Tests for StateDB, MemoryStateDB, and Snapshot
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_state::{StateDB, MemoryStateDB, Snapshot, StateError};
use bach_primitives::H256;

// =============================================================================
// StateError tests
// =============================================================================

mod state_error {
    use super::*;

    #[test]
    fn error_variants_exist() {
        let key = H256::zero();
        let _ = StateError::KeyNotFound(key);
        let _ = StateError::SnapshotExpired;
        let _ = StateError::LockError("test".to_string());
    }

    #[test]
    fn error_is_debug() {
        let err = StateError::SnapshotExpired;
        let debug = format!("{:?}", err);
        assert!(debug.contains("SnapshotExpired"));
    }

    #[test]
    fn error_is_clone() {
        let err = StateError::SnapshotExpired;
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn error_is_eq() {
        assert_eq!(StateError::SnapshotExpired, StateError::SnapshotExpired);
        let key = H256::zero();
        assert_eq!(StateError::KeyNotFound(key), StateError::KeyNotFound(key));
        assert_ne!(StateError::SnapshotExpired, StateError::LockError("".to_string()));
    }

    #[test]
    fn key_not_found_contains_key() {
        let key = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        if let StateError::KeyNotFound(k) = StateError::KeyNotFound(key) {
            assert_eq!(k, key);
        } else {
            panic!("Expected KeyNotFound");
        }
    }

    #[test]
    fn lock_error_contains_message() {
        let msg = "mutex poisoned".to_string();
        if let StateError::LockError(m) = StateError::LockError(msg.clone()) {
            assert_eq!(m, msg);
        } else {
            panic!("Expected LockError");
        }
    }
}

// =============================================================================
// MemoryStateDB::new() tests
// =============================================================================

mod memory_state_db_new {
    use super::*;

    #[test]
    fn creates_empty_db() {
        let db = MemoryStateDB::new();
        assert!(db.keys().is_empty());
    }

    #[test]
    fn get_returns_none_for_empty_db() {
        let db = MemoryStateDB::new();
        let key = H256::zero();
        assert_eq!(db.get(&key), None);
    }
}

// =============================================================================
// StateDB::get() tests
// =============================================================================

mod state_db_get {
    use super::*;

    #[test]
    fn returns_none_for_nonexistent_key() {
        let db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(db.get(&key), None);
    }

    #[test]
    fn returns_some_for_existing_key() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![1, 2, 3, 4];

        db.set(key, value.clone());
        assert_eq!(db.get(&key), Some(value));
    }

    #[test]
    fn returns_correct_value() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![0xde, 0xad, 0xbe, 0xef];

        db.set(key, value.clone());
        assert_eq!(db.get(&key), Some(value));
    }

    #[test]
    fn returns_latest_value() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1, 2, 3]);
        db.set(key, vec![4, 5, 6]);

        assert_eq!(db.get(&key), Some(vec![4, 5, 6]));
    }

    #[test]
    fn different_keys_different_values() {
        let mut db = MemoryStateDB::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        db.set(key1, vec![1, 1, 1]);
        db.set(key2, vec![2, 2, 2]);

        assert_eq!(db.get(&key1), Some(vec![1, 1, 1]));
        assert_eq!(db.get(&key2), Some(vec![2, 2, 2]));
    }
}

// =============================================================================
// StateDB::set() tests
// =============================================================================

mod state_db_set {
    use super::*;

    #[test]
    fn adds_new_key() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![1, 2, 3];

        db.set(key, value.clone());
        assert_eq!(db.get(&key), Some(value));
    }

    #[test]
    fn overwrites_existing_key() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1, 2, 3]);
        db.set(key, vec![4, 5, 6, 7]);

        assert_eq!(db.get(&key), Some(vec![4, 5, 6, 7]));
    }

    #[test]
    fn empty_value() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![]);
        assert_eq!(db.get(&key), Some(vec![]));
    }

    #[test]
    fn large_value() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![0xffu8; 10000]; // 10KB

        db.set(key, value.clone());
        assert_eq!(db.get(&key), Some(value));
    }

    #[test]
    fn multiple_keys() {
        let mut db = MemoryStateDB::new();

        for i in 0..100 {
            let mut key_bytes = [0u8; 32];
            key_bytes[31] = i as u8;
            let key = H256::from_slice(&key_bytes).unwrap();
            db.set(key, vec![i as u8]);
        }

        assert_eq!(db.keys().len(), 100);
    }
}

// =============================================================================
// StateDB::delete() tests
// =============================================================================

mod state_db_delete {
    use super::*;

    #[test]
    fn removes_existing_key() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1, 2, 3]);
        db.delete(&key);

        assert_eq!(db.get(&key), None);
    }

    #[test]
    fn delete_nonexistent_key_is_ok() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        // Should not panic
        db.delete(&key);
        assert_eq!(db.get(&key), None);
    }

    #[test]
    fn delete_does_not_affect_other_keys() {
        let mut db = MemoryStateDB::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        db.set(key1, vec![1, 1, 1]);
        db.set(key2, vec![2, 2, 2]);
        db.delete(&key1);

        assert_eq!(db.get(&key1), None);
        assert_eq!(db.get(&key2), Some(vec![2, 2, 2]));
    }

    #[test]
    fn can_set_after_delete() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1, 2, 3]);
        db.delete(&key);
        db.set(key, vec![4, 5, 6]);

        assert_eq!(db.get(&key), Some(vec![4, 5, 6]));
    }
}

// =============================================================================
// StateDB::snapshot() tests
// =============================================================================

mod state_db_snapshot {
    use super::*;

    #[test]
    fn snapshot_returns_snapshot() {
        let db = MemoryStateDB::new();
        let _snapshot = db.snapshot();
        // Should not panic
    }

    #[test]
    fn snapshot_sees_existing_data() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![1, 2, 3];

        db.set(key, value.clone());
        let snapshot = db.snapshot();

        assert_eq!(snapshot.get(&key), Some(value));
    }

    #[test]
    fn snapshot_isolation_from_later_writes() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1, 2, 3]);
        let snapshot = db.snapshot();

        // Write after snapshot
        db.set(key, vec![4, 5, 6]);

        // Snapshot should still see old value
        assert_eq!(snapshot.get(&key), Some(vec![1, 2, 3]));
        // DB should see new value
        assert_eq!(db.get(&key), Some(vec![4, 5, 6]));
    }

    #[test]
    fn snapshot_isolation_from_later_deletes() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1, 2, 3]);
        let snapshot = db.snapshot();

        // Delete after snapshot
        db.delete(&key);

        // Snapshot should still see value
        assert_eq!(snapshot.get(&key), Some(vec![1, 2, 3]));
        // DB should not see value
        assert_eq!(db.get(&key), None);
    }

    #[test]
    fn snapshot_does_not_see_nonexistent_keys() {
        let db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let snapshot = db.snapshot();
        assert_eq!(snapshot.get(&key), None);
    }

    #[test]
    fn multiple_snapshots_independent() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1]);
        let snap1 = db.snapshot();

        db.set(key, vec![2]);
        let snap2 = db.snapshot();

        db.set(key, vec![3]);

        assert_eq!(snap1.get(&key), Some(vec![1]));
        assert_eq!(snap2.get(&key), Some(vec![2]));
        assert_eq!(db.get(&key), Some(vec![3]));
    }
}

// =============================================================================
// StateDB::commit() tests
// =============================================================================

mod state_db_commit {
    use super::*;

    #[test]
    fn commit_empty_writes() {
        let mut db = MemoryStateDB::new();
        db.commit(&[]);
        assert!(db.keys().is_empty());
    }

    #[test]
    fn commit_single_write() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![1, 2, 3];

        db.commit(&[(key, value.clone())]);
        assert_eq!(db.get(&key), Some(value));
    }

    #[test]
    fn commit_multiple_writes() {
        let mut db = MemoryStateDB::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        db.commit(&[
            (key1, vec![1, 1, 1]),
            (key2, vec![2, 2, 2]),
        ]);

        assert_eq!(db.get(&key1), Some(vec![1, 1, 1]));
        assert_eq!(db.get(&key2), Some(vec![2, 2, 2]));
    }

    #[test]
    fn commit_overwrites_existing() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1, 2, 3]);
        db.commit(&[(key, vec![4, 5, 6])]);

        assert_eq!(db.get(&key), Some(vec![4, 5, 6]));
    }

    #[test]
    fn commit_duplicate_keys_last_wins() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.commit(&[
            (key, vec![1]),
            (key, vec![2]),
            (key, vec![3]),
        ]);

        assert_eq!(db.get(&key), Some(vec![3]));
    }

    #[test]
    fn commit_many_writes() {
        let mut db = MemoryStateDB::new();

        let writes: Vec<(H256, Vec<u8>)> = (0..100).map(|i| {
            let mut key_bytes = [0u8; 32];
            key_bytes[31] = i as u8;
            let key = H256::from_slice(&key_bytes).unwrap();
            (key, vec![i as u8])
        }).collect();

        db.commit(&writes);
        assert_eq!(db.keys().len(), 100);
    }
}

// =============================================================================
// StateDB::keys() tests
// =============================================================================

mod state_db_keys {
    use super::*;

    #[test]
    fn empty_for_new_db() {
        let db = MemoryStateDB::new();
        assert!(db.keys().is_empty());
    }

    #[test]
    fn contains_set_keys() {
        let mut db = MemoryStateDB::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        db.set(key1, vec![1]);
        db.set(key2, vec![2]);

        let keys = db.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&key1));
        assert!(keys.contains(&key2));
    }

    #[test]
    fn does_not_contain_deleted_keys() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1]);
        db.delete(&key);

        assert!(!db.keys().contains(&key));
    }

    #[test]
    fn no_duplicates() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        db.set(key, vec![1]);
        db.set(key, vec![2]);
        db.set(key, vec![3]);

        let keys = db.keys();
        let count = keys.iter().filter(|&k| *k == key).count();
        assert_eq!(count, 1);
    }
}

// =============================================================================
// Snapshot tests
// =============================================================================

mod snapshot {
    use super::*;

    #[test]
    fn get_returns_none_for_nonexistent() {
        let db = MemoryStateDB::new();
        let snapshot = db.snapshot();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        assert_eq!(snapshot.get(&key), None);
    }

    #[test]
    fn get_returns_value() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![1, 2, 3];

        db.set(key, value.clone());
        let snapshot = db.snapshot();

        assert_eq!(snapshot.get(&key), Some(value));
    }

    #[test]
    fn clone_works() {
        let mut db = MemoryStateDB::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let value = vec![1, 2, 3];

        db.set(key, value.clone());
        let snapshot = db.snapshot();
        let cloned = snapshot.clone();

        assert_eq!(cloned.get(&key), Some(value));
    }

    #[test]
    fn debug_is_implemented() {
        let db = MemoryStateDB::new();
        let snapshot = db.snapshot();
        let debug = format!("{:?}", snapshot);
        assert!(!debug.is_empty());
    }
}

// =============================================================================
// Thread safety tests
// =============================================================================

mod thread_safety {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn memory_state_db_is_send() {
        assert_send::<MemoryStateDB>();
    }

    #[test]
    fn memory_state_db_is_sync() {
        assert_sync::<MemoryStateDB>();
    }

    #[test]
    fn snapshot_is_send() {
        assert_send::<Snapshot>();
    }

    #[test]
    fn snapshot_is_sync() {
        assert_sync::<Snapshot>();
    }
}

// =============================================================================
// Default trait tests
// =============================================================================

mod default_trait {
    use super::*;

    #[test]
    fn memory_state_db_default() {
        let db: MemoryStateDB = Default::default();
        assert!(db.keys().is_empty());
    }
}

// =============================================================================
// Debug trait tests
// =============================================================================

mod debug_trait {
    use super::*;

    #[test]
    fn memory_state_db_debug() {
        let db = MemoryStateDB::new();
        let debug = format!("{:?}", db);
        assert!(!debug.is_empty());
    }
}
