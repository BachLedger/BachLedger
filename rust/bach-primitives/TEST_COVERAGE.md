# bach-primitives Test Coverage Summary

## Overview
This document summarizes the test coverage for the `bach-primitives` module.
Tests are written using TDD (Test-Driven Development) - they are written BEFORE implementation.

**Status**: Tests compile but FAIL (Red phase - implementation needed)

## Test Files

| File | Type | Tests |
|------|------|-------|
| `tests/address_tests.rs` | Address / H160 | 46 tests |
| `tests/h256_tests.rs` | H256 | 43 tests |
| `tests/u256_tests.rs` | U256 | 82 tests |
| **Total** | | **171 tests** |

---

## Address Tests (`tests/address_tests.rs`)

### from_slice (6 tests)
- `valid_slice_exactly_20_bytes` - Happy path
- `valid_slice_with_data` - Non-zero data
- `invalid_slice_too_short` - 19 bytes
- `invalid_slice_too_long` - 21 bytes
- `invalid_slice_empty` - 0 bytes
- `invalid_slice_much_too_long` - 100 bytes

### from_hex (15 tests)
- `valid_hex_with_0x_prefix` - Standard format
- `valid_hex_without_0x_prefix` - No prefix
- `valid_hex_with_data` - Non-zero data
- `valid_hex_uppercase` - UPPERCASE input
- `valid_hex_mixed_case` - MiXeD case
- `invalid_hex_wrong_length_too_short` - 4 bytes
- `invalid_hex_wrong_length_too_long` - 21 bytes
- `invalid_hex_chars` - Non-hex chars (g)
- `invalid_hex_special_chars` - Special characters
- `invalid_hex_empty_string` - Empty input
- `invalid_hex_only_prefix` - Just "0x"
- `invalid_hex_spaces` - Contains spaces
- `invalid_hex_odd_length` - Odd char count
- `valid_known_ethereum_address` - Vitalik's address

### zero (4 tests)
- `zero_returns_all_zeros` - All bytes are 0
- `zero_is_zero` - is_zero() returns true
- `zero_equals_from_slice_zeros` - Equality check
- `zero_equals_from_hex_zeros` - Equality check

### as_bytes (2 tests)
- `as_bytes_returns_correct_reference` - Correct data
- `as_bytes_length_is_correct` - Length = 20

### is_zero (4 tests)
- `is_zero_true_for_zero` - Zero address
- `is_zero_false_for_non_zero` - First byte non-zero
- `is_zero_false_for_last_byte_non_zero` - Last byte
- `is_zero_false_for_all_ones` - All 0xff

### Display/LowerHex (6 tests)
- `display_outputs_0x_prefix` - Starts with "0x"
- `display_outputs_lowercase_hex` - Lowercase output
- `display_outputs_correct_length` - 42 chars
- `lowerhex_outputs_0x_prefix` - Starts with "0x"
- `lowerhex_outputs_lowercase` - Lowercase output
- `display_and_lowerhex_are_equivalent` - Same output

### AsRef<[u8]> (2 tests)
- `as_ref_returns_slice` - Correct slice
- `as_ref_length_is_correct` - Length = 20

### From<[u8; 20]> (3 tests)
- `from_array_creates_address` - Basic conversion
- `from_array_with_data` - With data
- `from_array_equals_from_slice` - Equivalence

### Derived Traits (8 tests)
- `debug_is_implemented` - Debug trait
- `clone_produces_equal_copy` - Clone
- `copy_produces_equal_copy` - Copy
- `partial_eq_works` - PartialEq
- `hash_is_consistent` - Hash (HashSet)
- `hash_different_for_different_addresses` - Hash uniqueness
- `ord_ordering_is_consistent` - Ord
- `default_is_zero` - Default

### H160 Alias (4 tests)
- `h160_is_address` - Type equivalence
- `h160_from_hex_works` - from_hex
- `h160_from_slice_works` - from_slice
- `h160_zero_works` - zero()

### Thread Safety (2 tests)
- `address_is_send` - Send trait
- `address_is_sync` - Sync trait

---

## H256 Tests (`tests/h256_tests.rs`)

### from_slice (7 tests)
- `valid_slice_exactly_32_bytes` - Happy path
- `valid_slice_with_data` - Non-zero data
- `invalid_slice_too_short` - 31 bytes
- `invalid_slice_too_long` - 33 bytes
- `invalid_slice_empty` - 0 bytes
- `invalid_slice_20_bytes` - Address length rejection
- `invalid_slice_much_too_long` - 100 bytes

### from_hex (14 tests)
- `valid_hex_with_0x_prefix` - Standard format
- `valid_hex_without_0x_prefix` - No prefix
- `valid_hex_with_data` - Non-zero data
- `valid_hex_uppercase` - UPPERCASE input
- `valid_hex_mixed_case` - MiXeD case
- `invalid_hex_wrong_length_too_short` - 4 bytes
- `invalid_hex_wrong_length_20_bytes` - Address length
- `invalid_hex_wrong_length_too_long` - 33 bytes
- `invalid_hex_chars` - Non-hex chars
- `invalid_hex_special_chars` - Special characters
- `invalid_hex_empty_string` - Empty input
- `invalid_hex_only_prefix` - Just "0x"
- `invalid_hex_spaces` - Contains spaces
- `invalid_hex_odd_length` - Odd char count
- `valid_known_hash` - Keccak256 of empty string

### zero (4 tests)
- `zero_returns_all_zeros` - All bytes are 0
- `zero_is_zero` - is_zero() returns true
- `zero_equals_from_slice_zeros` - Equality check
- `zero_equals_from_hex_zeros` - Equality check

### as_bytes (2 tests)
- `as_bytes_returns_correct_reference` - Correct data
- `as_bytes_length_is_correct` - Length = 32

### is_zero (5 tests)
- `is_zero_true_for_zero` - Zero hash
- `is_zero_false_for_non_zero` - First byte non-zero
- `is_zero_false_for_last_byte_non_zero` - Last byte
- `is_zero_false_for_all_ones` - All 0xff
- `is_zero_false_for_middle_byte_non_zero` - Middle byte

### Display/LowerHex (6 tests)
- `display_outputs_0x_prefix` - Starts with "0x"
- `display_outputs_lowercase_hex` - Lowercase output
- `display_outputs_correct_length` - 66 chars
- `lowerhex_outputs_0x_prefix` - Starts with "0x"
- `lowerhex_outputs_lowercase` - Lowercase output
- `display_and_lowerhex_are_equivalent` - Same output

### AsRef<[u8]> (2 tests)
- `as_ref_returns_slice` - Correct slice
- `as_ref_length_is_correct` - Length = 32

### From<[u8; 32]> (3 tests)
- `from_array_creates_h256` - Basic conversion
- `from_array_with_data` - With data
- `from_array_equals_from_slice` - Equivalence

### Derived Traits (8 tests)
- `debug_is_implemented` - Debug trait
- `clone_produces_equal_copy` - Clone
- `copy_produces_equal_copy` - Copy
- `partial_eq_works` - PartialEq
- `hash_is_consistent` - Hash (HashSet)
- `hash_different_for_different_hashes` - Hash uniqueness
- `ord_ordering_is_consistent` - Ord
- `default_is_zero` - Default

### Thread Safety (2 tests)
- `h256_is_send` - Send trait
- `h256_is_sync` - Sync trait

---

## U256 Tests (`tests/u256_tests.rs`)

### Constants (6 tests)
- `zero_constant_is_zero` - ZERO.is_zero()
- `one_constant_is_not_zero` - ONE is non-zero
- `max_constant_is_not_zero` - MAX is non-zero
- `one_constant_value` - ONE = 1
- `max_constant_value` - MAX = all 0xff
- `zero_constant_value` - ZERO = all 0x00

### Big Endian (7 tests)
- `from_be_bytes_zero` - All zeros
- `from_be_bytes_one` - Value 1
- `from_be_bytes_max` - All 0xff
- `to_be_bytes_zero` - ZERO to bytes
- `to_be_bytes_one` - ONE to bytes
- `to_be_bytes_max` - MAX to bytes
- `roundtrip_be_bytes` - from/to roundtrip

### Little Endian (8 tests)
- `from_le_bytes_zero` - All zeros
- `from_le_bytes_one` - Value 1
- `from_le_bytes_max` - All 0xff
- `to_le_bytes_zero` - ZERO to bytes
- `to_le_bytes_one` - ONE to bytes
- `to_le_bytes_max` - MAX to bytes
- `roundtrip_le_bytes` - from/to roundtrip
- `be_and_le_differ_for_non_symmetric_value` - BE vs LE

### from_u64 / From<u64> (8 tests)
- `from_u64_zero` - Zero value
- `from_u64_one` - One value
- `from_u64_max` - u64::MAX
- `from_u64_arbitrary` - 0xdeadbeef
- `from_trait_u64` - From<u64> trait
- `from_trait_u64_zero` - From<u64> zero
- `from_trait_u64_max` - From<u64> max

### From<u128> (4 tests)
- `from_u128_zero` - Zero value
- `from_u128_one` - One value
- `from_u128_max` - u128::MAX
- `from_u128_arbitrary` - Large value

### checked_add (9 tests)
- `add_zero_to_zero` - 0 + 0
- `add_zero_to_one` - 1 + 0
- `add_one_to_zero` - 0 + 1
- `add_one_to_one` - 1 + 1
- `add_overflow_max_plus_one` - MAX + 1 = None
- `add_overflow_max_plus_max` - MAX + MAX = None
- `add_large_values_no_overflow` - (MAX-1) + 1
- `add_arbitrary_values` - 100 + 200
- `add_u64_max_twice_no_overflow` - u64::MAX + u64::MAX

### checked_sub (8 tests)
- `sub_zero_from_zero` - 0 - 0
- `sub_zero_from_one` - 1 - 0
- `sub_one_from_one` - 1 - 1
- `sub_underflow_zero_minus_one` - 0 - 1 = None
- `sub_underflow_one_minus_two` - 1 - 2 = None
- `sub_max_from_max` - MAX - MAX
- `sub_one_from_max` - MAX - 1
- `sub_arbitrary_values` - 300 - 100

### checked_mul (11 tests)
- `mul_zero_by_zero` - 0 * 0
- `mul_zero_by_one` - 0 * 1
- `mul_one_by_zero` - 1 * 0
- `mul_one_by_one` - 1 * 1
- `mul_max_by_zero` - MAX * 0
- `mul_zero_by_max` - 0 * MAX
- `mul_max_by_one` - MAX * 1
- `mul_overflow_max_by_two` - MAX * 2 = None
- `mul_overflow_large_values` - 2^128 * 2^128 = None
- `mul_arbitrary_values` - 100 * 200
- `mul_small_values_no_overflow` - 1000 * 1000

### checked_div (11 tests)
- `div_by_zero` - 1 / 0 = None
- `div_zero_by_zero` - 0 / 0 = None
- `div_max_by_zero` - MAX / 0 = None
- `div_zero_by_one` - 0 / 1
- `div_one_by_one` - 1 / 1
- `div_max_by_one` - MAX / 1
- `div_max_by_max` - MAX / MAX
- `div_arbitrary_exact` - 100 / 10
- `div_arbitrary_with_remainder` - 100 / 30
- `div_smaller_by_larger` - 5 / 10
- `div_one_by_max` - 1 / MAX

### is_zero (8 tests)
- `is_zero_for_zero_constant` - ZERO
- `is_zero_for_one_constant` - ONE
- `is_zero_for_max_constant` - MAX
- `is_zero_for_from_u64_zero` - from_u64(0)
- `is_zero_for_from_u64_one` - from_u64(1)
- `is_zero_after_subtraction_to_zero` - 1 - 1
- `is_zero_from_be_bytes_zero` - from BE zeros
- `is_zero_from_le_bytes_zero` - from LE zeros

### Display (5 tests)
- `display_zero` - "0"
- `display_one` - "1"
- `display_small_number` - "12345"
- `display_u64_max` - u64::MAX decimal
- `display_max` - 2^256-1 decimal

### LowerHex (6 tests)
- `lowerhex_zero` - "0x0"
- `lowerhex_one` - "0x1"
- `lowerhex_small_number` - "0xdeadbeef"
- `lowerhex_outputs_lowercase` - lowercase
- `lowerhex_max` - 64 f's
- `lowerhex_u64_max` - 16 f's

### Derived Traits (10 tests)
- `debug_is_implemented` - Debug trait
- `clone_produces_equal_copy` - Clone
- `copy_produces_equal_copy` - Copy
- `partial_eq_works` - PartialEq
- `hash_is_consistent` - Hash (HashSet)
- `hash_different_for_different_values` - Hash uniqueness
- `ord_ordering_is_consistent` - Ord
- `ord_zero_less_than_one` - ZERO < ONE
- `ord_one_less_than_max` - ONE < MAX
- `ord_max_is_greatest` - MAX > large
- `default_is_zero` - Default

### Thread Safety (2 tests)
- `u256_is_send` - Send trait
- `u256_is_sync` - Sync trait

### Edge Cases (7 tests)
- `arithmetic_identity_add_zero` - x + 0 = x
- `arithmetic_identity_sub_zero` - x - 0 = x
- `arithmetic_identity_mul_one` - x * 1 = x
- `arithmetic_identity_div_one` - x / 1 = x
- `add_sub_inverse` - (a + b) - b = a
- `mul_div_inverse_exact` - (a * b) / b = a
- `power_of_two_boundary` - Around 2^64

---

## Acceptance Criteria

For implementation to pass:
1. All 171 tests must pass
2. No panics from `todo!()` macros
3. All error types must match `PrimitiveError` variants
4. Thread safety (Send + Sync) must be verified

## Next Steps

1. Coder agent implements `bach-primitives/src/lib.rs`
2. Run `cargo test -p bach-primitives` to verify
3. All tests should transition from FAIL to PASS
