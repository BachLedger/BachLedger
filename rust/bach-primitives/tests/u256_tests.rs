//! Tests for U256 type
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_primitives::U256;

// =============================================================================
// Constants tests
// =============================================================================

mod constants {
    use super::*;

    #[test]
    fn zero_constant_is_zero() {
        assert!(U256::ZERO.is_zero());
    }

    #[test]
    fn one_constant_is_not_zero() {
        assert!(!U256::ONE.is_zero());
    }

    #[test]
    fn max_constant_is_not_zero() {
        assert!(!U256::MAX.is_zero());
    }

    #[test]
    fn one_constant_value() {
        let one = U256::ONE;
        let bytes = one.to_be_bytes();
        // Only last byte should be 1
        assert_eq!(bytes[31], 1);
        for i in 0..31 {
            assert_eq!(bytes[i], 0, "byte {} should be 0", i);
        }
    }

    #[test]
    fn max_constant_value() {
        let max = U256::MAX;
        let bytes = max.to_be_bytes();
        // All bytes should be 0xff
        for (i, &byte) in bytes.iter().enumerate() {
            assert_eq!(byte, 0xff, "byte {} should be 0xff", i);
        }
    }

    #[test]
    fn zero_constant_value() {
        let zero = U256::ZERO;
        let bytes = zero.to_be_bytes();
        // All bytes should be 0
        for (i, &byte) in bytes.iter().enumerate() {
            assert_eq!(byte, 0, "byte {} should be 0", i);
        }
    }
}

// =============================================================================
// from_be_bytes / to_be_bytes tests
// =============================================================================

mod big_endian {
    use super::*;

    #[test]
    fn from_be_bytes_zero() {
        let bytes = [0u8; 32];
        let val = U256::from_be_bytes(bytes);
        assert!(val.is_zero());
    }

    #[test]
    fn from_be_bytes_one() {
        let mut bytes = [0u8; 32];
        bytes[31] = 1;
        let val = U256::from_be_bytes(bytes);
        assert_eq!(val, U256::ONE);
    }

    #[test]
    fn from_be_bytes_max() {
        let bytes = [0xffu8; 32];
        let val = U256::from_be_bytes(bytes);
        assert_eq!(val, U256::MAX);
    }

    #[test]
    fn to_be_bytes_zero() {
        let val = U256::ZERO;
        let bytes = val.to_be_bytes();
        assert_eq!(bytes, [0u8; 32]);
    }

    #[test]
    fn to_be_bytes_one() {
        let val = U256::ONE;
        let bytes = val.to_be_bytes();
        let mut expected = [0u8; 32];
        expected[31] = 1;
        assert_eq!(bytes, expected);
    }

    #[test]
    fn to_be_bytes_max() {
        let val = U256::MAX;
        let bytes = val.to_be_bytes();
        assert_eq!(bytes, [0xffu8; 32]);
    }

    #[test]
    fn roundtrip_be_bytes() {
        let original: [u8; 32] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
        ];
        let val = U256::from_be_bytes(original);
        let roundtrip = val.to_be_bytes();
        assert_eq!(original, roundtrip);
    }
}

// =============================================================================
// from_le_bytes / to_le_bytes tests
// =============================================================================

mod little_endian {
    use super::*;

    #[test]
    fn from_le_bytes_zero() {
        let bytes = [0u8; 32];
        let val = U256::from_le_bytes(bytes);
        assert!(val.is_zero());
    }

    #[test]
    fn from_le_bytes_one() {
        let mut bytes = [0u8; 32];
        bytes[0] = 1; // Little-endian: LSB first
        let val = U256::from_le_bytes(bytes);
        assert_eq!(val, U256::ONE);
    }

    #[test]
    fn from_le_bytes_max() {
        let bytes = [0xffu8; 32];
        let val = U256::from_le_bytes(bytes);
        assert_eq!(val, U256::MAX);
    }

    #[test]
    fn to_le_bytes_zero() {
        let val = U256::ZERO;
        let bytes = val.to_le_bytes();
        assert_eq!(bytes, [0u8; 32]);
    }

    #[test]
    fn to_le_bytes_one() {
        let val = U256::ONE;
        let bytes = val.to_le_bytes();
        let mut expected = [0u8; 32];
        expected[0] = 1; // Little-endian: LSB first
        assert_eq!(bytes, expected);
    }

    #[test]
    fn to_le_bytes_max() {
        let val = U256::MAX;
        let bytes = val.to_le_bytes();
        assert_eq!(bytes, [0xffu8; 32]);
    }

    #[test]
    fn roundtrip_le_bytes() {
        let original: [u8; 32] = [
            0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
        ];
        let val = U256::from_le_bytes(original);
        let roundtrip = val.to_le_bytes();
        assert_eq!(original, roundtrip);
    }

    #[test]
    fn be_and_le_differ_for_non_symmetric_value() {
        let mut be_bytes = [0u8; 32];
        be_bytes[31] = 0x01;
        be_bytes[30] = 0x02;

        let from_be = U256::from_be_bytes(be_bytes);
        let be_roundtrip = from_be.to_be_bytes();
        let le_roundtrip = from_be.to_le_bytes();

        // BE and LE representations should be reversed
        assert_ne!(be_roundtrip, le_roundtrip);
    }
}

// =============================================================================
// from_u64 / From<u64> tests
// =============================================================================

mod from_u64 {
    use super::*;

    #[test]
    fn from_u64_zero() {
        let val = U256::from_u64(0);
        assert!(val.is_zero());
    }

    #[test]
    fn from_u64_one() {
        let val = U256::from_u64(1);
        assert_eq!(val, U256::ONE);
    }

    #[test]
    fn from_u64_max() {
        let val = U256::from_u64(u64::MAX);
        assert!(!val.is_zero());
        let bytes = val.to_be_bytes();
        // First 24 bytes should be 0
        for i in 0..24 {
            assert_eq!(bytes[i], 0, "byte {} should be 0", i);
        }
        // Last 8 bytes should be u64::MAX in big-endian
        for i in 24..32 {
            assert_eq!(bytes[i], 0xff, "byte {} should be 0xff", i);
        }
    }

    #[test]
    fn from_u64_arbitrary() {
        let val = U256::from_u64(0xdeadbeef);
        let bytes = val.to_be_bytes();
        assert_eq!(bytes[28], 0xde);
        assert_eq!(bytes[29], 0xad);
        assert_eq!(bytes[30], 0xbe);
        assert_eq!(bytes[31], 0xef);
    }

    #[test]
    fn from_trait_u64() {
        let val: U256 = 42u64.into();
        let expected = U256::from_u64(42);
        assert_eq!(val, expected);
    }

    #[test]
    fn from_trait_u64_zero() {
        let val: U256 = 0u64.into();
        assert!(val.is_zero());
    }

    #[test]
    fn from_trait_u64_max() {
        let val: U256 = u64::MAX.into();
        assert_eq!(val, U256::from_u64(u64::MAX));
    }
}

// =============================================================================
// From<u128> tests
// =============================================================================

mod from_u128 {
    use super::*;

    #[test]
    fn from_u128_zero() {
        let val: U256 = 0u128.into();
        assert!(val.is_zero());
    }

    #[test]
    fn from_u128_one() {
        let val: U256 = 1u128.into();
        assert_eq!(val, U256::ONE);
    }

    #[test]
    fn from_u128_max() {
        let val: U256 = u128::MAX.into();
        assert!(!val.is_zero());
        let bytes = val.to_be_bytes();
        // First 16 bytes should be 0
        for i in 0..16 {
            assert_eq!(bytes[i], 0, "byte {} should be 0", i);
        }
        // Last 16 bytes should be u128::MAX in big-endian
        for i in 16..32 {
            assert_eq!(bytes[i], 0xff, "byte {} should be 0xff", i);
        }
    }

    #[test]
    fn from_u128_arbitrary() {
        let val: U256 = 0xdeadbeefcafebabe12345678u128.into();
        let bytes = val.to_be_bytes();
        // Should be in last 12 bytes (96 bits)
        assert_eq!(bytes[20], 0xde);
        assert_eq!(bytes[21], 0xad);
        assert_eq!(bytes[22], 0xbe);
        assert_eq!(bytes[23], 0xef);
        assert_eq!(bytes[24], 0xca);
        assert_eq!(bytes[25], 0xfe);
        assert_eq!(bytes[26], 0xba);
        assert_eq!(bytes[27], 0xbe);
        assert_eq!(bytes[28], 0x12);
        assert_eq!(bytes[29], 0x34);
        assert_eq!(bytes[30], 0x56);
        assert_eq!(bytes[31], 0x78);
    }
}

// =============================================================================
// checked_add tests
// =============================================================================

mod checked_add {
    use super::*;

    #[test]
    fn add_zero_to_zero() {
        let result = U256::ZERO.checked_add(&U256::ZERO);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn add_zero_to_one() {
        let result = U256::ONE.checked_add(&U256::ZERO);
        assert_eq!(result, Some(U256::ONE));
    }

    #[test]
    fn add_one_to_zero() {
        let result = U256::ZERO.checked_add(&U256::ONE);
        assert_eq!(result, Some(U256::ONE));
    }

    #[test]
    fn add_one_to_one() {
        let result = U256::ONE.checked_add(&U256::ONE);
        assert!(result.is_some());
        let val = result.unwrap();
        let bytes = val.to_be_bytes();
        assert_eq!(bytes[31], 2);
    }

    #[test]
    fn add_overflow_max_plus_one() {
        let result = U256::MAX.checked_add(&U256::ONE);
        assert_eq!(result, None);
    }

    #[test]
    fn add_overflow_max_plus_max() {
        let result = U256::MAX.checked_add(&U256::MAX);
        assert_eq!(result, None);
    }

    #[test]
    fn add_large_values_no_overflow() {
        // (MAX - 1) + 1 should not overflow
        let max_minus_one = U256::MAX.checked_sub(&U256::ONE).unwrap();
        let result = max_minus_one.checked_add(&U256::ONE);
        assert_eq!(result, Some(U256::MAX));
    }

    #[test]
    fn add_arbitrary_values() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(200);
        let result = a.checked_add(&b);
        assert_eq!(result, Some(U256::from_u64(300)));
    }

    #[test]
    fn add_u64_max_twice_no_overflow() {
        let a = U256::from_u64(u64::MAX);
        let b = U256::from_u64(u64::MAX);
        let result = a.checked_add(&b);
        assert!(result.is_some());
    }
}

// =============================================================================
// checked_sub tests
// =============================================================================

mod checked_sub {
    use super::*;

    #[test]
    fn sub_zero_from_zero() {
        let result = U256::ZERO.checked_sub(&U256::ZERO);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn sub_zero_from_one() {
        let result = U256::ONE.checked_sub(&U256::ZERO);
        assert_eq!(result, Some(U256::ONE));
    }

    #[test]
    fn sub_one_from_one() {
        let result = U256::ONE.checked_sub(&U256::ONE);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn sub_underflow_zero_minus_one() {
        let result = U256::ZERO.checked_sub(&U256::ONE);
        assert_eq!(result, None);
    }

    #[test]
    fn sub_underflow_one_minus_two() {
        let two = U256::ONE.checked_add(&U256::ONE).unwrap();
        let result = U256::ONE.checked_sub(&two);
        assert_eq!(result, None);
    }

    #[test]
    fn sub_max_from_max() {
        let result = U256::MAX.checked_sub(&U256::MAX);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn sub_one_from_max() {
        let result = U256::MAX.checked_sub(&U256::ONE);
        assert!(result.is_some());
        let val = result.unwrap();
        // MAX - 1 should have last byte as 0xfe
        let bytes = val.to_be_bytes();
        assert_eq!(bytes[31], 0xfe);
    }

    #[test]
    fn sub_arbitrary_values() {
        let a = U256::from_u64(300);
        let b = U256::from_u64(100);
        let result = a.checked_sub(&b);
        assert_eq!(result, Some(U256::from_u64(200)));
    }
}

// =============================================================================
// checked_mul tests
// =============================================================================

mod checked_mul {
    use super::*;

    #[test]
    fn mul_zero_by_zero() {
        let result = U256::ZERO.checked_mul(&U256::ZERO);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn mul_zero_by_one() {
        let result = U256::ZERO.checked_mul(&U256::ONE);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn mul_one_by_zero() {
        let result = U256::ONE.checked_mul(&U256::ZERO);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn mul_one_by_one() {
        let result = U256::ONE.checked_mul(&U256::ONE);
        assert_eq!(result, Some(U256::ONE));
    }

    #[test]
    fn mul_max_by_zero() {
        let result = U256::MAX.checked_mul(&U256::ZERO);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn mul_zero_by_max() {
        let result = U256::ZERO.checked_mul(&U256::MAX);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn mul_max_by_one() {
        let result = U256::MAX.checked_mul(&U256::ONE);
        assert_eq!(result, Some(U256::MAX));
    }

    #[test]
    fn mul_overflow_max_by_two() {
        let two = U256::ONE.checked_add(&U256::ONE).unwrap();
        let result = U256::MAX.checked_mul(&two);
        assert_eq!(result, None);
    }

    #[test]
    fn mul_overflow_large_values() {
        // (2^128)^2 = 2^256 which overflows (> U256::MAX = 2^256 - 1)
        // Note: u128::MAX = 2^128 - 1, and (2^128 - 1)^2 = 2^256 - 2^129 + 1 which FITS
        // So we need to construct 2^128 explicitly
        let two_pow_128 = {
            let mut bytes = [0u8; 32];
            bytes[15] = 1; // Big-endian: byte 15 is the 128th bit position
            U256::from_be_bytes(bytes)
        };
        let result = two_pow_128.checked_mul(&two_pow_128);
        assert_eq!(result, None);
    }

    #[test]
    fn mul_u128_max_squared_does_not_overflow() {
        // (2^128 - 1)^2 = 2^256 - 2^129 + 1, which fits in 256 bits
        let large: U256 = u128::MAX.into();
        let result = large.checked_mul(&large);
        assert!(result.is_some(), "u128::MAX squared should fit in U256");
    }

    #[test]
    fn mul_arbitrary_values() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(200);
        let result = a.checked_mul(&b);
        assert_eq!(result, Some(U256::from_u64(20000)));
    }

    #[test]
    fn mul_small_values_no_overflow() {
        let a = U256::from_u64(1000);
        let b = U256::from_u64(1000);
        let result = a.checked_mul(&b);
        assert_eq!(result, Some(U256::from_u64(1_000_000)));
    }
}

// =============================================================================
// checked_div tests
// =============================================================================

mod checked_div {
    use super::*;

    #[test]
    fn div_by_zero() {
        let result = U256::ONE.checked_div(&U256::ZERO);
        assert_eq!(result, None);
    }

    #[test]
    fn div_zero_by_zero() {
        let result = U256::ZERO.checked_div(&U256::ZERO);
        assert_eq!(result, None);
    }

    #[test]
    fn div_max_by_zero() {
        let result = U256::MAX.checked_div(&U256::ZERO);
        assert_eq!(result, None);
    }

    #[test]
    fn div_zero_by_one() {
        let result = U256::ZERO.checked_div(&U256::ONE);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn div_one_by_one() {
        let result = U256::ONE.checked_div(&U256::ONE);
        assert_eq!(result, Some(U256::ONE));
    }

    #[test]
    fn div_max_by_one() {
        let result = U256::MAX.checked_div(&U256::ONE);
        assert_eq!(result, Some(U256::MAX));
    }

    #[test]
    fn div_max_by_max() {
        let result = U256::MAX.checked_div(&U256::MAX);
        assert_eq!(result, Some(U256::ONE));
    }

    #[test]
    fn div_arbitrary_exact() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(10);
        let result = a.checked_div(&b);
        assert_eq!(result, Some(U256::from_u64(10)));
    }

    #[test]
    fn div_arbitrary_with_remainder() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(30);
        let result = a.checked_div(&b);
        // 100 / 30 = 3 (integer division)
        assert_eq!(result, Some(U256::from_u64(3)));
    }

    #[test]
    fn div_smaller_by_larger() {
        let a = U256::from_u64(5);
        let b = U256::from_u64(10);
        let result = a.checked_div(&b);
        assert_eq!(result, Some(U256::ZERO));
    }

    #[test]
    fn div_one_by_max() {
        let result = U256::ONE.checked_div(&U256::MAX);
        assert_eq!(result, Some(U256::ZERO));
    }
}

// =============================================================================
// is_zero tests
// =============================================================================

mod is_zero {
    use super::*;

    #[test]
    fn is_zero_for_zero_constant() {
        assert!(U256::ZERO.is_zero());
    }

    #[test]
    fn is_zero_for_one_constant() {
        assert!(!U256::ONE.is_zero());
    }

    #[test]
    fn is_zero_for_max_constant() {
        assert!(!U256::MAX.is_zero());
    }

    #[test]
    fn is_zero_for_from_u64_zero() {
        assert!(U256::from_u64(0).is_zero());
    }

    #[test]
    fn is_zero_for_from_u64_one() {
        assert!(!U256::from_u64(1).is_zero());
    }

    #[test]
    fn is_zero_after_subtraction_to_zero() {
        let result = U256::ONE.checked_sub(&U256::ONE).unwrap();
        assert!(result.is_zero());
    }

    #[test]
    fn is_zero_from_be_bytes_zero() {
        let val = U256::from_be_bytes([0u8; 32]);
        assert!(val.is_zero());
    }

    #[test]
    fn is_zero_from_le_bytes_zero() {
        let val = U256::from_le_bytes([0u8; 32]);
        assert!(val.is_zero());
    }
}

// =============================================================================
// Display trait tests
// =============================================================================

mod display {
    use super::*;

    #[test]
    fn display_zero() {
        let display = format!("{}", U256::ZERO);
        assert_eq!(display, "0");
    }

    #[test]
    fn display_one() {
        let display = format!("{}", U256::ONE);
        assert_eq!(display, "1");
    }

    #[test]
    fn display_small_number() {
        let val = U256::from_u64(12345);
        let display = format!("{}", val);
        assert_eq!(display, "12345");
    }

    #[test]
    fn display_u64_max() {
        let val = U256::from_u64(u64::MAX);
        let display = format!("{}", val);
        assert_eq!(display, u64::MAX.to_string());
    }

    #[test]
    fn display_max() {
        let display = format!("{}", U256::MAX);
        // 2^256 - 1 in decimal
        let expected = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        assert_eq!(display, expected);
    }
}

// =============================================================================
// LowerHex trait tests
// =============================================================================

mod lowerhex {
    use super::*;

    #[test]
    fn lowerhex_zero() {
        let display = format!("{:x}", U256::ZERO);
        assert_eq!(display, "0x0");
    }

    #[test]
    fn lowerhex_one() {
        let display = format!("{:x}", U256::ONE);
        assert_eq!(display, "0x1");
    }

    #[test]
    fn lowerhex_small_number() {
        let val = U256::from_u64(0xdeadbeef);
        let display = format!("{:x}", val);
        assert_eq!(display, "0xdeadbeef");
    }

    #[test]
    fn lowerhex_outputs_lowercase() {
        let val = U256::from_u64(0xABCDEF);
        let display = format!("{:x}", val);
        assert_eq!(display, "0xabcdef");
    }

    #[test]
    fn lowerhex_max() {
        let display = format!("{:x}", U256::MAX);
        // 64 f's
        let expected = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        assert_eq!(display, expected);
    }

    #[test]
    fn lowerhex_u64_max() {
        let val = U256::from_u64(u64::MAX);
        let display = format!("{:x}", val);
        assert_eq!(display, "0xffffffffffffffff");
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
        let val = U256::ZERO;
        let debug = format!("{:?}", val);
        assert!(!debug.is_empty());
    }

    #[test]
    fn clone_produces_equal_copy() {
        let val = U256::from_u64(12345);
        let cloned = val.clone();
        assert_eq!(val, cloned);
    }

    #[test]
    fn copy_produces_equal_copy() {
        let val = U256::from_u64(12345);
        let copied = val;
        assert_eq!(val, copied);
    }

    #[test]
    fn partial_eq_works() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(100);
        let c = U256::from_u64(200);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn hash_is_consistent() {
        let val = U256::from_u64(12345);
        let mut set = HashSet::new();
        set.insert(val);
        assert!(set.contains(&val));
    }

    #[test]
    fn hash_different_for_different_values() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(200);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn ord_ordering_is_consistent() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(200);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn ord_zero_less_than_one() {
        assert!(U256::ZERO < U256::ONE);
    }

    #[test]
    fn ord_one_less_than_max() {
        assert!(U256::ONE < U256::MAX);
    }

    #[test]
    fn ord_max_is_greatest() {
        let large: U256 = u128::MAX.into();
        assert!(large < U256::MAX);
    }

    #[test]
    fn default_is_zero() {
        let default_val: U256 = Default::default();
        assert!(default_val.is_zero());
        assert_eq!(default_val, U256::ZERO);
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
    fn u256_is_send() {
        assert_send::<U256>();
    }

    #[test]
    fn u256_is_sync() {
        assert_sync::<U256>();
    }
}

// =============================================================================
// Edge case tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn arithmetic_identity_add_zero() {
        let val = U256::from_u64(12345);
        let result = val.checked_add(&U256::ZERO).unwrap();
        assert_eq!(val, result);
    }

    #[test]
    fn arithmetic_identity_sub_zero() {
        let val = U256::from_u64(12345);
        let result = val.checked_sub(&U256::ZERO).unwrap();
        assert_eq!(val, result);
    }

    #[test]
    fn arithmetic_identity_mul_one() {
        let val = U256::from_u64(12345);
        let result = val.checked_mul(&U256::ONE).unwrap();
        assert_eq!(val, result);
    }

    #[test]
    fn arithmetic_identity_div_one() {
        let val = U256::from_u64(12345);
        let result = val.checked_div(&U256::ONE).unwrap();
        assert_eq!(val, result);
    }

    #[test]
    fn add_sub_inverse() {
        let a = U256::from_u64(12345);
        let b = U256::from_u64(6789);
        let sum = a.checked_add(&b).unwrap();
        let back = sum.checked_sub(&b).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn mul_div_inverse_exact() {
        let a = U256::from_u64(100);
        let b = U256::from_u64(10);
        let product = a.checked_mul(&b).unwrap();
        let back = product.checked_div(&b).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn power_of_two_boundary() {
        // Test values around 2^64 boundary
        let just_below = U256::from_u64(u64::MAX);
        let one = U256::ONE;
        let result = just_below.checked_add(&one);
        assert!(result.is_some());
        // Result should be 2^64
        let val = result.unwrap();
        assert!(val > just_below);
    }
}
