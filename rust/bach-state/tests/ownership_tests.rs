//! Tests for OwnershipEntry and OwnershipTable
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.
//!
//! These tests implement Algorithm 1 from the Seamless Scheduling paper.

use bach_state::{OwnershipEntry, OwnershipTable};
use bach_primitives::H256;
use bach_types::PriorityCode;
use std::sync::Arc;

// =============================================================================
// OwnershipEntry::new() tests
// =============================================================================

mod ownership_entry_new {
    use super::*;

    #[test]
    fn creates_with_disowned_status() {
        let entry = OwnershipEntry::new();
        // Any priority code should be able to claim ownership of a DISOWNED entry
        let pc = PriorityCode::new(100, H256::zero());
        assert!(entry.check_ownership(&pc));
    }

    #[test]
    fn default_is_same_as_new() {
        let from_new = OwnershipEntry::new();
        let from_default: OwnershipEntry = Default::default();

        let pc = PriorityCode::new(100, H256::zero());
        assert_eq!(from_new.check_ownership(&pc), from_default.check_ownership(&pc));
    }
}

// =============================================================================
// OwnershipEntry::release_ownership() tests
// =============================================================================

mod release_ownership {
    use super::*;

    #[test]
    fn makes_entry_available() {
        let entry = OwnershipEntry::new();
        let pc = PriorityCode::new(100, H256::zero());

        // Claim ownership
        entry.try_set_owner(&pc);

        // Release
        entry.release_ownership();

        // Any other priority code should now be able to claim
        let other_pc = PriorityCode::new(200, H256::zero());
        assert!(entry.check_ownership(&other_pc));
    }

    #[test]
    fn is_idempotent() {
        let entry = OwnershipEntry::new();
        let pc = PriorityCode::new(100, H256::zero());

        entry.try_set_owner(&pc);
        entry.release_ownership();
        entry.release_ownership();
        entry.release_ownership();

        // Should still be released
        let other_pc = PriorityCode::new(999, H256::zero());
        assert!(entry.check_ownership(&other_pc));
    }

    #[test]
    fn released_entry_can_be_reclaimed() {
        let entry = OwnershipEntry::new();
        let pc1 = PriorityCode::new(100, H256::zero());
        let pc2 = PriorityCode::new(200, H256::zero());

        // Claim with pc1
        assert!(entry.try_set_owner(&pc1));

        // Release
        entry.release_ownership();

        // Reclaim with pc2 (even though pc2 has lower priority normally)
        assert!(entry.try_set_owner(&pc2));
    }
}

// =============================================================================
// OwnershipEntry::check_ownership() tests
// =============================================================================

mod check_ownership {
    use super::*;

    #[test]
    fn returns_true_for_disowned_entry() {
        let entry = OwnershipEntry::new();
        let pc = PriorityCode::new(100, H256::zero());
        assert!(entry.check_ownership(&pc));
    }

    #[test]
    fn returns_true_for_higher_priority() {
        let entry = OwnershipEntry::new();
        let low_priority = PriorityCode::new(200, H256::zero());
        let high_priority = PriorityCode::new(100, H256::zero()); // Lower block = higher priority

        // Set owner to low priority
        entry.try_set_owner(&low_priority);

        // High priority can check ownership (would succeed)
        assert!(entry.check_ownership(&high_priority));
    }

    #[test]
    fn returns_true_for_equal_priority() {
        let entry = OwnershipEntry::new();
        let hash = H256::from_hex("0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789").unwrap();
        let pc1 = PriorityCode::new(100, hash);
        let pc2 = PriorityCode::new(100, hash);

        entry.try_set_owner(&pc1);
        assert!(entry.check_ownership(&pc2));
    }

    #[test]
    fn returns_false_for_lower_priority() {
        let entry = OwnershipEntry::new();
        let high_priority = PriorityCode::new(100, H256::zero()); // Lower block = higher priority
        let low_priority = PriorityCode::new(200, H256::zero());

        // Set owner to high priority
        entry.try_set_owner(&high_priority);

        // Low priority cannot check ownership (would fail)
        assert!(!entry.check_ownership(&low_priority));
    }

    #[test]
    fn priority_by_release_bit() {
        let entry = OwnershipEntry::new();
        let hash = H256::zero();

        let owned = PriorityCode::new(100, hash);
        let mut released = PriorityCode::new(100, hash);
        released.release();

        // Set owner to owned (higher priority than released)
        entry.try_set_owner(&owned);

        // Released (lower priority) cannot check ownership
        assert!(!entry.check_ownership(&released));
    }

    #[test]
    fn priority_by_block_height() {
        let entry = OwnershipEntry::new();
        let hash = H256::zero();

        let low_block = PriorityCode::new(100, hash); // Higher priority
        let high_block = PriorityCode::new(200, hash); // Lower priority

        entry.try_set_owner(&low_block);

        // High block (lower priority) cannot check
        assert!(!entry.check_ownership(&high_block));
        // Low block (current owner) can check
        assert!(entry.check_ownership(&low_block));
    }

    #[test]
    fn priority_by_hash() {
        let entry = OwnershipEntry::new();

        let low_hash = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let high_hash = H256::from_hex("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let low_hash_pc = PriorityCode::new(100, low_hash); // Higher priority
        let high_hash_pc = PriorityCode::new(100, high_hash); // Lower priority

        entry.try_set_owner(&low_hash_pc);

        // High hash (lower priority) cannot check
        assert!(!entry.check_ownership(&high_hash_pc));
    }
}

// =============================================================================
// OwnershipEntry::try_set_owner() tests
// =============================================================================

mod try_set_owner {
    use super::*;

    #[test]
    fn succeeds_for_disowned_entry() {
        let entry = OwnershipEntry::new();
        let pc = PriorityCode::new(100, H256::zero());
        assert!(entry.try_set_owner(&pc));
    }

    #[test]
    fn succeeds_for_higher_priority() {
        let entry = OwnershipEntry::new();
        let low_priority = PriorityCode::new(200, H256::zero());
        let high_priority = PriorityCode::new(100, H256::zero());

        entry.try_set_owner(&low_priority);
        assert!(entry.try_set_owner(&high_priority));
    }

    #[test]
    fn succeeds_for_equal_priority() {
        let entry = OwnershipEntry::new();
        let hash = H256::from_hex("0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789").unwrap();
        let pc1 = PriorityCode::new(100, hash);
        let pc2 = PriorityCode::new(100, hash);

        entry.try_set_owner(&pc1);
        // Equal priority should succeed (<=)
        assert!(entry.try_set_owner(&pc2));
    }

    #[test]
    fn fails_for_lower_priority() {
        let entry = OwnershipEntry::new();
        let high_priority = PriorityCode::new(100, H256::zero());
        let low_priority = PriorityCode::new(200, H256::zero());

        entry.try_set_owner(&high_priority);
        assert!(!entry.try_set_owner(&low_priority));
    }

    #[test]
    fn updates_owner_on_success() {
        let entry = OwnershipEntry::new();
        let pc1 = PriorityCode::new(200, H256::zero());
        let pc2 = PriorityCode::new(100, H256::zero());

        entry.try_set_owner(&pc1);
        entry.try_set_owner(&pc2);

        // Owner should now be pc2
        let owner = entry.current_owner();
        assert_eq!(owner.block_height(), 100);
    }

    #[test]
    fn does_not_update_owner_on_failure() {
        let entry = OwnershipEntry::new();
        let high = PriorityCode::new(100, H256::zero());
        let low = PriorityCode::new(200, H256::zero());

        entry.try_set_owner(&high);
        entry.try_set_owner(&low); // Should fail

        // Owner should still be high
        let owner = entry.current_owner();
        assert_eq!(owner.block_height(), 100);
    }

    #[test]
    fn after_release_any_can_claim() {
        let entry = OwnershipEntry::new();
        let high = PriorityCode::new(100, H256::zero());
        let low = PriorityCode::new(999, H256::zero());

        entry.try_set_owner(&high);
        entry.release_ownership();

        // Even low priority can claim after release
        assert!(entry.try_set_owner(&low));
    }
}

// =============================================================================
// OwnershipEntry::current_owner() tests
// =============================================================================

mod current_owner {
    use super::*;

    #[test]
    fn returns_disowned_priority_for_new() {
        let entry = OwnershipEntry::new();
        let owner = entry.current_owner();
        // DISOWNED entries have release_bit = 1 (lowest priority)
        assert!(owner.is_released());
    }

    #[test]
    fn returns_set_owner() {
        let entry = OwnershipEntry::new();
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let pc = PriorityCode::new(12345, hash);

        entry.try_set_owner(&pc);
        let owner = entry.current_owner();

        assert_eq!(owner.block_height(), 12345);
        assert_eq!(owner.hash(), &hash);
        assert!(!owner.is_released());
    }

    #[test]
    fn returns_released_after_release() {
        let entry = OwnershipEntry::new();
        let pc = PriorityCode::new(100, H256::zero());

        entry.try_set_owner(&pc);
        entry.release_ownership();

        let owner = entry.current_owner();
        assert!(owner.is_released());
    }
}

// =============================================================================
// OwnershipEntry Clone tests
// =============================================================================

mod ownership_entry_clone {
    use super::*;

    #[test]
    fn clone_preserves_state() {
        let entry = OwnershipEntry::new();
        let pc = PriorityCode::new(100, H256::zero());
        entry.try_set_owner(&pc);

        let cloned = entry.clone();
        let owner = cloned.current_owner();
        assert_eq!(owner.block_height(), 100);
    }
}

// =============================================================================
// OwnershipTable::new() tests
// =============================================================================

mod ownership_table_new {
    use super::*;

    #[test]
    fn creates_empty_table() {
        let table = OwnershipTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn default_is_same_as_new() {
        let from_new = OwnershipTable::new();
        let from_default: OwnershipTable = Default::default();

        assert_eq!(from_new.len(), from_default.len());
    }
}

// =============================================================================
// OwnershipTable::get_or_create() tests
// =============================================================================

mod get_or_create {
    use super::*;

    #[test]
    fn creates_new_entry_for_unknown_key() {
        let table = OwnershipTable::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let entry = table.get_or_create(&key);
        assert_eq!(table.len(), 1);

        // New entry should be DISOWNED
        let pc = PriorityCode::new(100, H256::zero());
        assert!(entry.check_ownership(&pc));
    }

    #[test]
    fn returns_existing_entry() {
        let table = OwnershipTable::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let entry1 = table.get_or_create(&key);
        let pc = PriorityCode::new(100, H256::zero());
        entry1.try_set_owner(&pc);

        let entry2 = table.get_or_create(&key);
        // Should be the same entry with the same owner
        assert_eq!(entry2.current_owner().block_height(), 100);
    }

    #[test]
    fn different_keys_different_entries() {
        let table = OwnershipTable::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        let entry1 = table.get_or_create(&key1);
        let entry2 = table.get_or_create(&key2);

        let pc = PriorityCode::new(100, H256::zero());
        entry1.try_set_owner(&pc);

        // entry2 should still be DISOWNED
        assert!(entry2.current_owner().is_released());
    }

    #[test]
    fn returns_arc() {
        let table = OwnershipTable::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let entry: Arc<OwnershipEntry> = table.get_or_create(&key);
        // Should be able to clone the Arc
        let _cloned = Arc::clone(&entry);
    }
}

// =============================================================================
// OwnershipTable::release_all() tests
// =============================================================================

mod release_all {
    use super::*;

    #[test]
    fn releases_all_specified_keys() {
        let table = OwnershipTable::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        let pc = PriorityCode::new(100, H256::zero());
        table.get_or_create(&key1).try_set_owner(&pc);
        table.get_or_create(&key2).try_set_owner(&pc);

        table.release_all(&[key1, key2]);

        // Both should be released
        assert!(table.get_or_create(&key1).current_owner().is_released());
        assert!(table.get_or_create(&key2).current_owner().is_released());
    }

    #[test]
    fn does_not_affect_unspecified_keys() {
        let table = OwnershipTable::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        let pc = PriorityCode::new(100, H256::zero());
        table.get_or_create(&key1).try_set_owner(&pc);
        table.get_or_create(&key2).try_set_owner(&pc);

        table.release_all(&[key1]); // Only release key1

        assert!(table.get_or_create(&key1).current_owner().is_released());
        assert!(!table.get_or_create(&key2).current_owner().is_released());
    }

    #[test]
    fn handles_empty_list() {
        let table = OwnershipTable::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let pc = PriorityCode::new(100, H256::zero());
        table.get_or_create(&key).try_set_owner(&pc);

        table.release_all(&[]);

        // Should not be affected
        assert!(!table.get_or_create(&key).current_owner().is_released());
    }

    #[test]
    fn handles_nonexistent_keys() {
        let table = OwnershipTable::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        // Should not panic
        table.release_all(&[key]);
    }
}

// =============================================================================
// OwnershipTable::clear() tests
// =============================================================================

mod clear {
    use super::*;

    #[test]
    fn removes_all_entries() {
        let table = OwnershipTable::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        table.get_or_create(&key1);
        table.get_or_create(&key2);
        assert_eq!(table.len(), 2);

        table.clear();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn clear_empty_table_is_ok() {
        let table = OwnershipTable::new();
        table.clear();
        assert!(table.is_empty());
    }

    #[test]
    fn can_add_after_clear() {
        let table = OwnershipTable::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        table.get_or_create(&key);
        table.clear();
        table.get_or_create(&key);

        assert_eq!(table.len(), 1);
    }
}

// =============================================================================
// OwnershipTable::len() and is_empty() tests
// =============================================================================

mod len_and_is_empty {
    use super::*;

    #[test]
    fn len_zero_for_new() {
        let table = OwnershipTable::new();
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn is_empty_true_for_new() {
        let table = OwnershipTable::new();
        assert!(table.is_empty());
    }

    #[test]
    fn len_increases_with_entries() {
        let table = OwnershipTable::new();

        for i in 0..10 {
            let mut key_bytes = [0u8; 32];
            key_bytes[31] = i;
            let key = H256::from_slice(&key_bytes).unwrap();
            table.get_or_create(&key);
            assert_eq!(table.len(), (i + 1) as usize);
        }
    }

    #[test]
    fn is_empty_false_after_add() {
        let table = OwnershipTable::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        table.get_or_create(&key);
        assert!(!table.is_empty());
    }

    #[test]
    fn get_or_create_same_key_does_not_increase_len() {
        let table = OwnershipTable::new();
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        table.get_or_create(&key);
        table.get_or_create(&key);
        table.get_or_create(&key);

        assert_eq!(table.len(), 1);
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
    fn ownership_entry_is_send() {
        assert_send::<OwnershipEntry>();
    }

    #[test]
    fn ownership_entry_is_sync() {
        assert_sync::<OwnershipEntry>();
    }

    #[test]
    fn ownership_table_is_send() {
        assert_send::<OwnershipTable>();
    }

    #[test]
    fn ownership_table_is_sync() {
        assert_sync::<OwnershipTable>();
    }

    #[test]
    fn concurrent_get_or_create() {
        use std::thread;

        let table = Arc::new(OwnershipTable::new());
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();

        let handles: Vec<_> = (0..4).map(|_| {
            let table = Arc::clone(&table);
            thread::spawn(move || {
                table.get_or_create(&key)
            })
        }).collect();

        for handle in handles {
            let _ = handle.join().unwrap();
        }

        // Only one entry should exist
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn concurrent_try_set_owner() {
        use std::thread;

        let table = Arc::new(OwnershipTable::new());
        let key = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let entry = table.get_or_create(&key);

        let handles: Vec<_> = (0..4).map(|i| {
            let entry = Arc::clone(&entry);
            thread::spawn(move || {
                let pc = PriorityCode::new(100 + i as u64, H256::zero());
                entry.try_set_owner(&pc)
            })
        }).collect();

        let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // At least one should succeed (the highest priority one)
        // Only one can be the final owner, others may have succeeded then been preempted
        let successes = results.iter().filter(|&&r| r).count();
        assert!(successes >= 1);
    }
}

// =============================================================================
// Integration tests - Algorithm 1 scenarios
// =============================================================================

mod algorithm1_scenarios {
    use super::*;

    #[test]
    fn scenario_single_owner() {
        // Single transaction claims ownership
        let entry = OwnershipEntry::new();
        let tx_priority = PriorityCode::new(100, H256::zero());

        assert!(entry.try_set_owner(&tx_priority));
        assert!(entry.check_ownership(&tx_priority));
    }

    #[test]
    fn scenario_conflict_higher_priority_wins() {
        // Two transactions compete, higher priority wins
        let entry = OwnershipEntry::new();
        let low_priority = PriorityCode::new(200, H256::zero());
        let high_priority = PriorityCode::new(100, H256::zero());

        // Low priority claims first
        assert!(entry.try_set_owner(&low_priority));

        // High priority can steal
        assert!(entry.try_set_owner(&high_priority));

        // Low priority now fails
        assert!(!entry.check_ownership(&low_priority));
    }

    #[test]
    fn scenario_release_then_reclaim() {
        // Transaction releases ownership, another claims
        let entry = OwnershipEntry::new();
        let tx1 = PriorityCode::new(100, H256::zero());
        let tx2 = PriorityCode::new(200, H256::zero());

        entry.try_set_owner(&tx1);
        entry.release_ownership();

        // tx2 can now claim even with lower priority
        assert!(entry.try_set_owner(&tx2));
    }

    #[test]
    fn scenario_multiple_keys() {
        // Transaction accesses multiple keys
        let table = OwnershipTable::new();
        let key1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let key2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();
        let key3 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000003").unwrap();

        let tx_priority = PriorityCode::new(100, H256::zero());

        // Claim all keys
        for key in &[key1, key2, key3] {
            assert!(table.get_or_create(key).try_set_owner(&tx_priority));
        }

        // Release all
        table.release_all(&[key1, key2, key3]);

        // All should be released
        for key in &[key1, key2, key3] {
            assert!(table.get_or_create(key).current_owner().is_released());
        }
    }
}
