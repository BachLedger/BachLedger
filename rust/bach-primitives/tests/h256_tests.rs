//! Tests for H256 type
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_primitives::{H256, PrimitiveError, HASH_LENGTH};

// =============================================================================
// from_slice tests
// =============================================================================

mod from_slice {
    use super::*;

    #[test]
    fn valid_slice_exactly_32_bytes() {
        let bytes = [0u8; HASH_LENGTH];
        let result = H256::from_slice(&bytes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), &bytes);
    }

    #[test]
    fn valid_slice_with_data() {
        let bytes: [u8; 32] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
        ];
        let result = H256::from_slice(&bytes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), &bytes);
    }

    #[test]
    fn invalid_slice_too_short() {
        let bytes = [0u8; 31];
        let result = H256::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, HASH_LENGTH);
                assert_eq!(actual, 31);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_slice_too_long() {
        let bytes = [0u8; 33];
        let result = H256::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, HASH_LENGTH);
                assert_eq!(actual, 33);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_slice_empty() {
        let bytes: [u8; 0] = [];
        let result = H256::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, HASH_LENGTH);
                assert_eq!(actual, 0);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_slice_20_bytes() {
        // Test that H256 rejects Address-length input
        let bytes = [0u8; 20];
        let result = H256::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, HASH_LENGTH);
                assert_eq!(actual, 20);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_slice_much_too_long() {
        let bytes = [0u8; 100];
        let result = H256::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, HASH_LENGTH);
                assert_eq!(actual, 100);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }
}

// =============================================================================
// from_hex tests
// =============================================================================

mod from_hex {
    use super::*;

    #[test]
    fn valid_hex_with_0x_prefix() {
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000000";
        let result = H256::from_hex(hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn valid_hex_without_0x_prefix() {
        let hex = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = H256::from_hex(hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn valid_hex_with_data() {
        let hex = "0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb";
        let result = H256::from_hex(hex);
        assert!(result.is_ok());
        let expected: [u8; 32] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
        ];
        assert_eq!(result.unwrap().as_bytes(), &expected);
    }

    #[test]
    fn valid_hex_uppercase() {
        let hex = "0xDEADBEEF00112233445566778899AABBCCDDEEFF00112233445566778899AABB";
        let result = H256::from_hex(hex);
        assert!(result.is_ok());
    }

    #[test]
    fn valid_hex_mixed_case() {
        let hex = "0xDeAdBeEf00112233445566778899AaBbCcDdEeFf00112233445566778899AaBb";
        let result = H256::from_hex(hex);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_hex_wrong_length_too_short() {
        let hex = "0xdeadbeef"; // Only 4 bytes
        let result = H256::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, HASH_LENGTH);
                assert_eq!(actual, 4);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_hex_wrong_length_20_bytes() {
        // Address-length hex should fail for H256
        let hex = "0xdeadbeef00112233445566778899aabbccddeeff";
        let result = H256::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, HASH_LENGTH);
                assert_eq!(actual, 20);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_hex_wrong_length_too_long() {
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000000ff"; // 33 bytes
        let result = H256::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, HASH_LENGTH);
                assert_eq!(actual, 33);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_hex_chars() {
        let hex = "0xgggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg";
        let result = H256::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidHex(_)) => (),
            _ => panic!("Expected InvalidHex error"),
        }
    }

    #[test]
    fn invalid_hex_special_chars() {
        let hex = "0x!@#$%^&*()!@#$%^&*()!@#$%^&*()!@#$%^&*()!@#$%^&*()!@#$%^&*()!@#$";
        let result = H256::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidHex(_)) => (),
            _ => panic!("Expected InvalidHex error"),
        }
    }

    #[test]
    fn invalid_hex_empty_string() {
        let hex = "";
        let result = H256::from_hex(hex);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_hex_only_prefix() {
        let hex = "0x";
        let result = H256::from_hex(hex);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_hex_spaces() {
        let hex = "0x dead beef 0011 2233 4455 6677 8899 aabb ccdd eeff 0011 2233 4455 6677 8899 aabb";
        let result = H256::from_hex(hex);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_hex_odd_length() {
        let hex = "0xdeadbee"; // 7 chars = 3.5 bytes, invalid
        let result = H256::from_hex(hex);
        assert!(result.is_err());
    }

    #[test]
    fn valid_known_hash() {
        // Keccak256 of empty string
        let hex = "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
        let result = H256::from_hex(hex);
        assert!(result.is_ok());
    }
}

// =============================================================================
// zero() tests
// =============================================================================

mod zero {
    use super::*;

    #[test]
    fn zero_returns_all_zeros() {
        let zero = H256::zero();
        assert_eq!(zero.as_bytes(), &[0u8; HASH_LENGTH]);
    }

    #[test]
    fn zero_is_zero() {
        let zero = H256::zero();
        assert!(zero.is_zero());
    }

    #[test]
    fn zero_equals_from_slice_zeros() {
        let from_slice = H256::from_slice(&[0u8; HASH_LENGTH]).unwrap();
        let zero = H256::zero();
        assert_eq!(zero, from_slice);
    }

    #[test]
    fn zero_equals_from_hex_zeros() {
        let from_hex = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let zero = H256::zero();
        assert_eq!(zero, from_hex);
    }
}

// =============================================================================
// as_bytes tests
// =============================================================================

mod as_bytes {
    use super::*;

    #[test]
    fn as_bytes_returns_correct_reference() {
        let bytes: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let hash = H256::from_slice(&bytes).unwrap();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn as_bytes_length_is_correct() {
        let hash = H256::zero();
        assert_eq!(hash.as_bytes().len(), HASH_LENGTH);
    }
}

// =============================================================================
// is_zero tests
// =============================================================================

mod is_zero {
    use super::*;

    #[test]
    fn is_zero_true_for_zero() {
        let hash = H256::zero();
        assert!(hash.is_zero());
    }

    #[test]
    fn is_zero_false_for_non_zero() {
        let mut bytes = [0u8; HASH_LENGTH];
        bytes[0] = 1;
        let hash = H256::from_slice(&bytes).unwrap();
        assert!(!hash.is_zero());
    }

    #[test]
    fn is_zero_false_for_last_byte_non_zero() {
        let mut bytes = [0u8; HASH_LENGTH];
        bytes[31] = 1;
        let hash = H256::from_slice(&bytes).unwrap();
        assert!(!hash.is_zero());
    }

    #[test]
    fn is_zero_false_for_all_ones() {
        let bytes = [0xffu8; HASH_LENGTH];
        let hash = H256::from_slice(&bytes).unwrap();
        assert!(!hash.is_zero());
    }

    #[test]
    fn is_zero_false_for_middle_byte_non_zero() {
        let mut bytes = [0u8; HASH_LENGTH];
        bytes[15] = 0xff;
        let hash = H256::from_slice(&bytes).unwrap();
        assert!(!hash.is_zero());
    }
}

// =============================================================================
// Display and LowerHex trait tests
// =============================================================================

mod display {
    use super::*;

    #[test]
    fn display_outputs_0x_prefix() {
        let hash = H256::zero();
        let display = format!("{}", hash);
        assert!(display.starts_with("0x"));
    }

    #[test]
    fn display_outputs_lowercase_hex() {
        let hex = "0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb";
        let hash = H256::from_hex(hex).unwrap();
        let display = format!("{}", hash);
        assert_eq!(display, hex);
    }

    #[test]
    fn display_outputs_correct_length() {
        let hash = H256::zero();
        let display = format!("{}", hash);
        // 0x + 64 hex chars = 66
        assert_eq!(display.len(), 66);
    }

    #[test]
    fn lowerhex_outputs_0x_prefix() {
        let hash = H256::zero();
        let display = format!("{:x}", hash);
        assert!(display.starts_with("0x"));
    }

    #[test]
    fn lowerhex_outputs_lowercase() {
        let hex = "0xDEADBEEF00112233445566778899AABBCCDDEEFF00112233445566778899AABB";
        let hash = H256::from_hex(hex).unwrap();
        let display = format!("{:x}", hash);
        assert_eq!(display, hex.to_lowercase());
    }

    #[test]
    fn display_and_lowerhex_are_equivalent() {
        let hash = H256::from_hex("0xaabbccdd00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let display = format!("{}", hash);
        let lowerhex = format!("{:x}", hash);
        assert_eq!(display, lowerhex);
    }
}

// =============================================================================
// AsRef<[u8]> trait tests
// =============================================================================

mod as_ref {
    use super::*;

    #[test]
    fn as_ref_returns_slice() {
        let bytes: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let hash = H256::from_slice(&bytes).unwrap();
        let slice: &[u8] = hash.as_ref();
        assert_eq!(slice, &bytes);
    }

    #[test]
    fn as_ref_length_is_correct() {
        let hash = H256::zero();
        let slice: &[u8] = hash.as_ref();
        assert_eq!(slice.len(), HASH_LENGTH);
    }
}

// =============================================================================
// From<[u8; HASH_LENGTH]> trait tests
// =============================================================================

mod from_array {
    use super::*;

    #[test]
    fn from_array_creates_h256() {
        let bytes: [u8; HASH_LENGTH] = [0u8; HASH_LENGTH];
        let hash: H256 = bytes.into();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn from_array_with_data() {
        let bytes: [u8; 32] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
        ];
        let hash: H256 = bytes.into();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn from_array_equals_from_slice() {
        let bytes: [u8; HASH_LENGTH] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let from_array: H256 = bytes.into();
        let from_slice = H256::from_slice(&bytes).unwrap();
        assert_eq!(from_array, from_slice);
    }
}

// =============================================================================
// Derived trait tests (Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)
// =============================================================================

mod derived_traits {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn debug_is_implemented() {
        let hash = H256::zero();
        let debug = format!("{:?}", hash);
        assert!(!debug.is_empty());
    }

    #[test]
    fn clone_produces_equal_copy() {
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let cloned = hash.clone();
        assert_eq!(hash, cloned);
    }

    #[test]
    fn copy_produces_equal_copy() {
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let copied = hash;
        assert_eq!(hash, copied);
    }

    #[test]
    fn partial_eq_works() {
        let hash1 = H256::zero();
        let hash2 = H256::zero();
        let hash3 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn hash_is_consistent() {
        let hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let mut set = HashSet::new();
        set.insert(hash);
        assert!(set.contains(&hash));
    }

    #[test]
    fn hash_different_for_different_hashes() {
        let hash1 = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let hash2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let mut set = HashSet::new();
        set.insert(hash1);
        set.insert(hash2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn ord_ordering_is_consistent() {
        let hash1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let hash2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();
        assert!(hash1 < hash2);
        assert!(hash2 > hash1);
    }

    #[test]
    fn default_is_zero() {
        let default_hash: H256 = Default::default();
        let zero_hash = H256::zero();
        assert_eq!(default_hash, zero_hash);
    }
}

// =============================================================================
// Thread safety tests (Send + Sync)
// =============================================================================

mod thread_safety {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn h256_is_send() {
        assert_send::<H256>();
    }

    #[test]
    fn h256_is_sync() {
        assert_sync::<H256>();
    }
}
