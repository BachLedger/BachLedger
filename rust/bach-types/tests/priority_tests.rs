//! Tests for PriorityCode type
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_types::{PriorityCode, PRIORITY_OWNED, PRIORITY_DISOWNED};
use bach_primitives::H256;

// =============================================================================
// Constants tests
// =============================================================================

mod constants {
    use super::*;

    #[test]
    fn priority_owned_is_zero() {
        assert_eq!(PRIORITY_OWNED, 0);
    }

    #[test]
    fn priority_disowned_is_one() {
        assert_eq!(PRIORITY_DISOWNED, 1);
    }
}

// =============================================================================
// new() tests
// =============================================================================

mod new {
    use super::*;

    #[test]
    fn creates_with_owned_status() {
        let hash = H256::zero();
        let pc = PriorityCode::new(100, hash);
        assert!(!pc.is_released());
    }

    #[test]
    fn stores_block_height() {
        let hash = H256::zero();
        let pc = PriorityCode::new(12345, hash);
        assert_eq!(pc.block_height(), 12345);
    }

    #[test]
    fn stores_hash() {
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let pc = PriorityCode::new(100, hash);
        assert_eq!(pc.hash(), &hash);
    }

    #[test]
    fn zero_block_height() {
        let hash = H256::zero();
        let pc = PriorityCode::new(0, hash);
        assert_eq!(pc.block_height(), 0);
    }

    #[test]
    fn max_block_height() {
        let hash = H256::zero();
        let pc = PriorityCode::new(u64::MAX, hash);
        assert_eq!(pc.block_height(), u64::MAX);
    }
}

// =============================================================================
// release() tests
// =============================================================================

mod release {
    use super::*;

    #[test]
    fn changes_to_disowned() {
        let hash = H256::zero();
        let mut pc = PriorityCode::new(100, hash);
        assert!(!pc.is_released());
        pc.release();
        assert!(pc.is_released());
    }

    #[test]
    fn release_is_idempotent() {
        let hash = H256::zero();
        let mut pc = PriorityCode::new(100, hash);
        pc.release();
        pc.release();
        assert!(pc.is_released());
    }

    #[test]
    fn release_does_not_change_block_height() {
        let hash = H256::zero();
        let mut pc = PriorityCode::new(12345, hash);
        let height_before = pc.block_height();
        pc.release();
        assert_eq!(pc.block_height(), height_before);
    }

    #[test]
    fn release_does_not_change_hash() {
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let mut pc = PriorityCode::new(100, hash);
        pc.release();
        assert_eq!(pc.hash(), &hash);
    }
}

// =============================================================================
// is_released() tests
// =============================================================================

mod is_released {
    use super::*;

    #[test]
    fn false_when_new() {
        let pc = PriorityCode::new(100, H256::zero());
        assert!(!pc.is_released());
    }

    #[test]
    fn true_after_release() {
        let mut pc = PriorityCode::new(100, H256::zero());
        pc.release();
        assert!(pc.is_released());
    }
}

// =============================================================================
// Ordering tests (Ord, PartialOrd)
// =============================================================================

mod ordering {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn owned_has_higher_priority_than_disowned() {
        // Lower value = Higher priority
        // Owned (0) < Disowned (1) means Owned has higher priority
        let hash = H256::zero();
        let owned = PriorityCode::new(100, hash);
        let mut disowned = PriorityCode::new(100, hash);
        disowned.release();

        assert!(owned < disowned, "Owned should have higher priority (lower value)");
    }

    #[test]
    fn lower_block_height_has_higher_priority() {
        let hash = H256::zero();
        let low_height = PriorityCode::new(100, hash);
        let high_height = PriorityCode::new(200, hash);

        assert!(low_height < high_height, "Lower block height should have higher priority");
    }

    #[test]
    fn lower_hash_has_higher_priority() {
        let low_hash = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let high_hash = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        let pc_low = PriorityCode::new(100, low_hash);
        let pc_high = PriorityCode::new(100, high_hash);

        assert!(pc_low < pc_high, "Lower hash should have higher priority");
    }

    #[test]
    fn ordering_release_bit_first() {
        // Even with lower block height and hash, released should be lower priority
        let hash1 = H256::zero();
        let hash2 = H256::from_hex("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let owned = PriorityCode::new(1000, hash2); // Higher block height, higher hash, but OWNED
        let mut disowned = PriorityCode::new(1, hash1); // Lower block height, lower hash, but DISOWNED
        disowned.release();

        assert!(owned < disowned, "Owned should have higher priority regardless of other fields");
    }

    #[test]
    fn ordering_block_height_second() {
        // Same release status, different block heights
        let low_hash = H256::zero();
        let high_hash = H256::from_hex("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let low_height_high_hash = PriorityCode::new(100, high_hash);
        let high_height_low_hash = PriorityCode::new(200, low_hash);

        assert!(low_height_high_hash < high_height_low_hash,
                "Lower block height should have higher priority regardless of hash");
    }

    #[test]
    fn ordering_hash_third() {
        // Same release status, same block height, different hashes
        let low_hash = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let high_hash = H256::from_hex("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let pc_low = PriorityCode::new(100, low_hash);
        let pc_high = PriorityCode::new(100, high_hash);

        assert!(pc_low < pc_high, "Lower hash should have higher priority");
    }

    #[test]
    fn equal_priority_codes() {
        let hash = H256::from_hex("0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789").unwrap();
        let pc1 = PriorityCode::new(100, hash);
        let pc2 = PriorityCode::new(100, hash);

        assert_eq!(pc1.cmp(&pc2), Ordering::Equal);
        assert_eq!(pc1, pc2);
    }

    #[test]
    fn partial_ord_consistent_with_ord() {
        let hash1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let hash2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        let pc1 = PriorityCode::new(100, hash1);
        let pc2 = PriorityCode::new(100, hash2);

        assert_eq!(pc1.partial_cmp(&pc2), Some(pc1.cmp(&pc2)));
    }

    #[test]
    fn can_be_sorted() {
        let hash1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let hash2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();
        let hash3 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000003").unwrap();

        let mut pcs = vec![
            PriorityCode::new(200, hash2),
            PriorityCode::new(100, hash3),
            PriorityCode::new(100, hash1),
        ];

        pcs.sort();

        // After sorting: lowest (highest priority) first
        assert_eq!(pcs[0].block_height(), 100);
        assert_eq!(pcs[0].hash(), &hash1);
        assert_eq!(pcs[1].block_height(), 100);
        assert_eq!(pcs[1].hash(), &hash3);
        assert_eq!(pcs[2].block_height(), 200);
    }
}

// =============================================================================
// Serialization tests (to_bytes / from_bytes)
// =============================================================================

mod serialization {
    use super::*;

    #[test]
    fn to_bytes_length() {
        let pc = PriorityCode::new(100, H256::zero());
        let bytes = pc.to_bytes();
        assert_eq!(bytes.len(), 41);
    }

    #[test]
    fn from_bytes_roundtrip() {
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let pc = PriorityCode::new(12345, hash);
        let bytes = pc.to_bytes();
        let restored = PriorityCode::from_bytes(&bytes);

        assert_eq!(pc.block_height(), restored.block_height());
        assert_eq!(pc.hash(), restored.hash());
        assert_eq!(pc.is_released(), restored.is_released());
    }

    #[test]
    fn from_bytes_roundtrip_released() {
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let mut pc = PriorityCode::new(12345, hash);
        pc.release();
        let bytes = pc.to_bytes();
        let restored = PriorityCode::from_bytes(&bytes);

        assert!(restored.is_released());
        assert_eq!(pc.block_height(), restored.block_height());
        assert_eq!(pc.hash(), restored.hash());
    }

    #[test]
    fn bytes_structure_release_bit_first() {
        let pc = PriorityCode::new(100, H256::zero());
        let bytes = pc.to_bytes();
        // First byte should be release_bit (0 for OWNED)
        assert_eq!(bytes[0], PRIORITY_OWNED);
    }

    #[test]
    fn bytes_structure_release_bit_disowned() {
        let mut pc = PriorityCode::new(100, H256::zero());
        pc.release();
        let bytes = pc.to_bytes();
        // First byte should be release_bit (1 for DISOWNED)
        assert_eq!(bytes[0], PRIORITY_DISOWNED);
    }

    #[test]
    fn bytes_structure_block_height() {
        let pc = PriorityCode::new(0x0102030405060708u64, H256::zero());
        let bytes = pc.to_bytes();
        // Bytes 1-8 should be block_height in big-endian
        assert_eq!(&bytes[1..9], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }

    #[test]
    fn bytes_structure_hash() {
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let pc = PriorityCode::new(100, hash);
        let bytes = pc.to_bytes();
        // Bytes 9-40 should be the hash
        assert_eq!(&bytes[9..41], hash.as_bytes());
    }

    #[test]
    fn from_bytes_zero_block_height() {
        let pc = PriorityCode::new(0, H256::zero());
        let bytes = pc.to_bytes();
        let restored = PriorityCode::from_bytes(&bytes);
        assert_eq!(restored.block_height(), 0);
    }

    #[test]
    fn from_bytes_max_block_height() {
        let pc = PriorityCode::new(u64::MAX, H256::zero());
        let bytes = pc.to_bytes();
        let restored = PriorityCode::from_bytes(&bytes);
        assert_eq!(restored.block_height(), u64::MAX);
    }
}

// =============================================================================
// Derived trait tests
// =============================================================================

mod derived_traits {
    use super::*;

    #[test]
    fn debug_is_implemented() {
        let pc = PriorityCode::new(100, H256::zero());
        let debug = format!("{:?}", pc);
        assert!(!debug.is_empty());
    }

    #[test]
    fn clone_works() {
        let pc = PriorityCode::new(100, H256::zero());
        let cloned = pc.clone();
        assert_eq!(pc, cloned);
    }

    #[test]
    fn eq_works() {
        let hash = H256::from_hex("0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789").unwrap();
        let pc1 = PriorityCode::new(100, hash);
        let pc2 = PriorityCode::new(100, hash);
        let pc3 = PriorityCode::new(200, hash);

        assert_eq!(pc1, pc2);
        assert_ne!(pc1, pc3);
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
    fn priority_code_is_send() {
        assert_send::<PriorityCode>();
    }

    #[test]
    fn priority_code_is_sync() {
        assert_sync::<PriorityCode>();
    }
}
