//! Tests for Address type
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_primitives::{Address, PrimitiveError, ADDRESS_LENGTH, H160};

// =============================================================================
// from_slice tests
// =============================================================================

mod from_slice {
    use super::*;

    #[test]
    fn valid_slice_exactly_20_bytes() {
        let bytes = [0u8; ADDRESS_LENGTH];
        let result = Address::from_slice(&bytes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), &bytes);
    }

    #[test]
    fn valid_slice_with_data() {
        let bytes: [u8; 20] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        ];
        let result = Address::from_slice(&bytes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), &bytes);
    }

    #[test]
    fn invalid_slice_too_short() {
        let bytes = [0u8; 19];
        let result = Address::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, ADDRESS_LENGTH);
                assert_eq!(actual, 19);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_slice_too_long() {
        let bytes = [0u8; 21];
        let result = Address::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, ADDRESS_LENGTH);
                assert_eq!(actual, 21);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_slice_empty() {
        let bytes: [u8; 0] = [];
        let result = Address::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, ADDRESS_LENGTH);
                assert_eq!(actual, 0);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_slice_much_too_long() {
        let bytes = [0u8; 100];
        let result = Address::from_slice(&bytes);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, ADDRESS_LENGTH);
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
        let hex = "0x0000000000000000000000000000000000000000";
        let result = Address::from_hex(hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), &[0u8; 20]);
    }

    #[test]
    fn valid_hex_without_0x_prefix() {
        let hex = "0000000000000000000000000000000000000000";
        let result = Address::from_hex(hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_bytes(), &[0u8; 20]);
    }

    #[test]
    fn valid_hex_with_data() {
        let hex = "0xdeadbeef00112233445566778899aabbccddeeff";
        let result = Address::from_hex(hex);
        assert!(result.is_ok());
        let expected: [u8; 20] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        ];
        assert_eq!(result.unwrap().as_bytes(), &expected);
    }

    #[test]
    fn valid_hex_uppercase() {
        let hex = "0xDEADBEEF00112233445566778899AABBCCDDEEFF";
        let result = Address::from_hex(hex);
        assert!(result.is_ok());
        let expected: [u8; 20] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        ];
        assert_eq!(result.unwrap().as_bytes(), &expected);
    }

    #[test]
    fn valid_hex_mixed_case() {
        let hex = "0xDeAdBeEf00112233445566778899AaBbCcDdEeFf";
        let result = Address::from_hex(hex);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_hex_wrong_length_too_short() {
        let hex = "0xdeadbeef"; // Only 4 bytes
        let result = Address::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, ADDRESS_LENGTH);
                assert_eq!(actual, 4);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_hex_wrong_length_too_long() {
        let hex = "0x0000000000000000000000000000000000000000ff"; // 21 bytes
        let result = Address::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidLength { expected, actual }) => {
                assert_eq!(expected, ADDRESS_LENGTH);
                assert_eq!(actual, 21);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn invalid_hex_chars() {
        let hex = "0xgggggggggggggggggggggggggggggggggggggggg";
        let result = Address::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidHex(_)) => (),
            _ => panic!("Expected InvalidHex error"),
        }
    }

    #[test]
    fn invalid_hex_special_chars() {
        let hex = "0x!@#$%^&*()!@#$%^&*()!@#$%^&*()!@#$%^&*()";
        let result = Address::from_hex(hex);
        assert!(result.is_err());
        match result {
            Err(PrimitiveError::InvalidHex(_)) => (),
            _ => panic!("Expected InvalidHex error"),
        }
    }

    #[test]
    fn invalid_hex_empty_string() {
        let hex = "";
        let result = Address::from_hex(hex);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_hex_only_prefix() {
        let hex = "0x";
        let result = Address::from_hex(hex);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_hex_spaces() {
        let hex = "0x dead beef 0011 2233 4455 6677 8899 aabb ccdd eeff";
        let result = Address::from_hex(hex);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_hex_odd_length() {
        let hex = "0xdeadbee"; // 7 chars = 3.5 bytes, invalid
        let result = Address::from_hex(hex);
        assert!(result.is_err());
    }

    #[test]
    fn valid_known_ethereum_address() {
        // Vitalik's address
        let hex = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        let result = Address::from_hex(hex);
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
        let zero = Address::zero();
        assert_eq!(zero.as_bytes(), &[0u8; ADDRESS_LENGTH]);
    }

    #[test]
    fn zero_is_zero() {
        let zero = Address::zero();
        assert!(zero.is_zero());
    }

    #[test]
    fn zero_equals_from_slice_zeros() {
        let from_slice = Address::from_slice(&[0u8; ADDRESS_LENGTH]).unwrap();
        let zero = Address::zero();
        assert_eq!(zero, from_slice);
    }

    #[test]
    fn zero_equals_from_hex_zeros() {
        let from_hex = Address::from_hex("0x0000000000000000000000000000000000000000").unwrap();
        let zero = Address::zero();
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
        let bytes: [u8; 20] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
        let addr = Address::from_slice(&bytes).unwrap();
        assert_eq!(addr.as_bytes(), &bytes);
    }

    #[test]
    fn as_bytes_length_is_correct() {
        let addr = Address::zero();
        assert_eq!(addr.as_bytes().len(), ADDRESS_LENGTH);
    }
}

// =============================================================================
// is_zero tests
// =============================================================================

mod is_zero {
    use super::*;

    #[test]
    fn is_zero_true_for_zero() {
        let addr = Address::zero();
        assert!(addr.is_zero());
    }

    #[test]
    fn is_zero_false_for_non_zero() {
        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes[0] = 1;
        let addr = Address::from_slice(&bytes).unwrap();
        assert!(!addr.is_zero());
    }

    #[test]
    fn is_zero_false_for_last_byte_non_zero() {
        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes[19] = 1;
        let addr = Address::from_slice(&bytes).unwrap();
        assert!(!addr.is_zero());
    }

    #[test]
    fn is_zero_false_for_all_ones() {
        let bytes = [0xffu8; ADDRESS_LENGTH];
        let addr = Address::from_slice(&bytes).unwrap();
        assert!(!addr.is_zero());
    }
}

// =============================================================================
// Display and LowerHex trait tests
// =============================================================================

mod display {
    use super::*;

    #[test]
    fn display_outputs_0x_prefix() {
        let addr = Address::zero();
        let display = format!("{}", addr);
        assert!(display.starts_with("0x"));
    }

    #[test]
    fn display_outputs_lowercase_hex() {
        let hex = "0xdeadbeef00112233445566778899aabbccddeeff";
        let addr = Address::from_hex(hex).unwrap();
        let display = format!("{}", addr);
        assert_eq!(display, hex);
    }

    #[test]
    fn display_outputs_correct_length() {
        let addr = Address::zero();
        let display = format!("{}", addr);
        // 0x + 40 hex chars = 42
        assert_eq!(display.len(), 42);
    }

    #[test]
    fn lowerhex_outputs_0x_prefix() {
        let addr = Address::zero();
        let display = format!("{:x}", addr);
        assert!(display.starts_with("0x"));
    }

    #[test]
    fn lowerhex_outputs_lowercase() {
        let hex = "0xDEADBEEF00112233445566778899AABBCCDDEEFF";
        let addr = Address::from_hex(hex).unwrap();
        let display = format!("{:x}", addr);
        assert_eq!(display, hex.to_lowercase());
    }

    #[test]
    fn display_and_lowerhex_are_equivalent() {
        let addr = Address::from_hex("0xaabbccdd00112233445566778899aabbccddeeff").unwrap();
        let display = format!("{}", addr);
        let lowerhex = format!("{:x}", addr);
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
        let bytes: [u8; 20] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
        let addr = Address::from_slice(&bytes).unwrap();
        let slice: &[u8] = addr.as_ref();
        assert_eq!(slice, &bytes);
    }

    #[test]
    fn as_ref_length_is_correct() {
        let addr = Address::zero();
        let slice: &[u8] = addr.as_ref();
        assert_eq!(slice.len(), ADDRESS_LENGTH);
    }
}

// =============================================================================
// From<[u8; ADDRESS_LENGTH]> trait tests
// =============================================================================

mod from_array {
    use super::*;

    #[test]
    fn from_array_creates_address() {
        let bytes: [u8; ADDRESS_LENGTH] = [0u8; ADDRESS_LENGTH];
        let addr: Address = bytes.into();
        assert_eq!(addr.as_bytes(), &bytes);
    }

    #[test]
    fn from_array_with_data() {
        let bytes: [u8; 20] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        ];
        let addr: Address = bytes.into();
        assert_eq!(addr.as_bytes(), &bytes);
    }

    #[test]
    fn from_array_equals_from_slice() {
        let bytes: [u8; ADDRESS_LENGTH] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
        let from_array: Address = bytes.into();
        let from_slice = Address::from_slice(&bytes).unwrap();
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
        let addr = Address::zero();
        let debug = format!("{:?}", addr);
        assert!(!debug.is_empty());
    }

    #[test]
    fn clone_produces_equal_copy() {
        let addr = Address::from_hex("0xdeadbeef00112233445566778899aabbccddeeff").unwrap();
        let cloned = addr.clone();
        assert_eq!(addr, cloned);
    }

    #[test]
    fn copy_produces_equal_copy() {
        let addr = Address::from_hex("0xdeadbeef00112233445566778899aabbccddeeff").unwrap();
        let copied = addr;
        assert_eq!(addr, copied);
    }

    #[test]
    fn partial_eq_works() {
        let addr1 = Address::zero();
        let addr2 = Address::zero();
        let addr3 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);
    }

    #[test]
    fn hash_is_consistent() {
        let addr = Address::from_hex("0xdeadbeef00112233445566778899aabbccddeeff").unwrap();
        let mut set = HashSet::new();
        set.insert(addr);
        assert!(set.contains(&addr));
    }

    #[test]
    fn hash_different_for_different_addresses() {
        let addr1 = Address::from_hex("0xdeadbeef00112233445566778899aabbccddeeff").unwrap();
        let addr2 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let mut set = HashSet::new();
        set.insert(addr1);
        set.insert(addr2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn ord_ordering_is_consistent() {
        let addr1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let addr2 = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();
        assert!(addr1 < addr2);
        assert!(addr2 > addr1);
    }

    #[test]
    fn default_is_zero() {
        let default_addr: Address = Default::default();
        let zero_addr = Address::zero();
        assert_eq!(default_addr, zero_addr);
    }
}

// =============================================================================
// H160 type alias tests
// =============================================================================

mod h160_alias {
    use super::*;

    #[test]
    fn h160_is_address() {
        let addr: Address = Address::zero();
        let h160: H160 = addr;
        assert_eq!(addr, h160);
    }

    #[test]
    fn h160_from_hex_works() {
        let h160 = H160::from_hex("0xdeadbeef00112233445566778899aabbccddeeff");
        assert!(h160.is_ok());
    }

    #[test]
    fn h160_from_slice_works() {
        let bytes = [0u8; ADDRESS_LENGTH];
        let h160 = H160::from_slice(&bytes);
        assert!(h160.is_ok());
    }

    #[test]
    fn h160_zero_works() {
        let h160 = H160::zero();
        assert!(h160.is_zero());
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
    fn address_is_send() {
        assert_send::<Address>();
    }

    #[test]
    fn address_is_sync() {
        assert_sync::<Address>();
    }
}
