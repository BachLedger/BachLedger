# bach-crypto Test Coverage Summary

## Overview
This document summarizes the test coverage for the `bach-crypto` module.
Tests are written using TDD (Test-Driven Development) - they are written BEFORE implementation.

**Status**: Tests compile but FAIL (Red phase - implementation needed)

## Test Files

| File | Type | Tests |
|------|------|-------|
| `tests/keccak_tests.rs` | keccak256, keccak256_concat | 28 tests |
| `tests/signature_tests.rs` | PrivateKey, PublicKey, Signature, CryptoError | 66 tests |
| **Total** | | **94 tests** |

---

## Keccak Tests (`tests/keccak_tests.rs`)

### keccak256 (14 tests)
- `empty_input` - keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
- `single_byte_zero` - keccak256([0x00])
- `hello_world` - keccak256("hello world")
- `abc` - keccak256("abc") standard test vector
- `deterministic` - Same input produces same output
- `different_inputs_different_outputs` - Different inputs produce different hashes
- `single_bit_change_affects_output` - Avalanche effect
- `long_input` - 1KB of zeros
- `result_is_32_bytes` - Output length check
- `ethereum_address_style_input` - 20-byte input
- `all_ones_input` - [0xff; 32]
- `sequential_bytes` - 0..255
- `hash_of_hash` - Nested hashing

### keccak256_concat (12 tests)
- `empty_slices` - No slices = hash of empty
- `single_empty_slice` - [&[]] = hash of empty
- `single_slice` - Single slice = keccak256
- `two_slices_equals_concatenated` - ["hello", "world"] = keccak256("helloworld")
- `three_slices` - Three-way concatenation
- `order_matters` - ["a", "b"] != ["b", "a"]
- `empty_slice_in_middle` - Empty slice is no-op
- `multiple_empty_slices` - Multiple empties = empty
- `single_byte_slices` - Byte-by-byte
- `binary_data_slices` - 32-byte hashes concatenated
- `mixed_length_slices` - Various lengths
- `many_slices` - 100 single-byte slices

### Thread Safety (1 test)
- `keccak256_can_be_called_from_multiple_threads`

### Test Vectors (5 tests)
- `nist_like_test_vector_short` - "The quick brown fox..."
- `nist_like_test_vector_with_period` - Same with period
- `ethereum_transfer_encoding_style` - 0xa9059cbb...
- `solidity_function_signature` - transfer(address,uint256)
- `solidity_event_signature` - Transfer event topic

---

## Signature Tests (`tests/signature_tests.rs`)

### CryptoError (4 tests)
- `error_variants_exist` - All variants defined
- `error_is_debug` - Debug trait
- `error_is_clone` - Clone trait
- `error_is_eq` - PartialEq/Eq traits

### PrivateKey (15 tests)
- `random_generates_valid_key` - random() works
- `random_generates_different_keys` - Not deterministic
- `from_bytes_valid_key` - Valid scalar accepted
- `from_bytes_zero_is_invalid` - Zero rejected
- `from_bytes_order_is_invalid` - Curve order rejected
- `from_bytes_above_order_is_invalid` - Above order rejected
- `to_bytes_roundtrip` - from_bytes/to_bytes cycle
- `to_bytes_random_roundtrip` - Random key roundtrip
- `public_key_derivation` - Derives 64-byte pubkey
- `public_key_deterministic` - Same key = same pubkey
- `sign_produces_signature` - Produces 65-byte sig
- `sign_deterministic_with_rfc6979` - Deterministic signing
- `sign_different_messages_different_signatures` - Different msgs
- `debug_does_not_reveal_key` - Key bytes redacted

### PublicKey (13 tests)
- `from_bytes_valid_point` - Valid point accepted
- `from_bytes_invalid_point_zeros` - Zeros rejected
- `from_bytes_invalid_point_random` - Random rejected
- `to_bytes_length` - 64 bytes
- `to_bytes_roundtrip` - from/to cycle
- `to_address_format` - Address is 20 bytes
- `to_address_is_last_20_bytes_of_keccak` - keccak256(pubkey)[12..32]
- `to_address_deterministic` - Same pubkey = same address
- `verify_valid_signature` - Valid sig verifies
- `verify_wrong_message` - Wrong message fails
- `verify_wrong_key` - Wrong key fails
- `debug_is_implemented` - Debug trait
- `clone_works` - Clone trait
- `eq_works` - Eq trait

### Signature (19 tests)
- `from_bytes_valid_signature` - Valid sig accepted
- `from_bytes_all_zeros_invalid` - Zeros rejected
- `to_bytes_length` - 65 bytes
- `to_bytes_roundtrip` - from/to cycle
- `verify_valid` - Valid verification
- `verify_invalid_message` - Wrong message fails
- `verify_invalid_pubkey` - Wrong key fails
- `recover_returns_correct_pubkey` - Recovery works
- `recover_fails_with_wrong_message` - Wrong message gives wrong key
- `r_component` - r() returns first 32 bytes
- `s_component` - s() returns bytes 32-64
- `v_component` - v() is 27 or 28
- `v_matches_last_byte` - v() = bytes[64]
- `r_s_v_compose_full_signature` - Components compose correctly
- `debug_is_implemented` - Debug trait
- `clone_works` - Clone trait
- `eq_works` - Eq trait

### Integration (4 tests)
- `full_sign_verify_recover_cycle` - Complete workflow
- `ethereum_style_transaction_signing` - TX signing simulation
- `multiple_signatures_same_key` - 10 signatures from one key
- `known_test_vector` - Private key 1 (generator point)

### Thread Safety (7 tests)
- `private_key_is_send` - Send trait
- `private_key_is_sync` - Sync trait
- `public_key_is_send` - Send trait
- `public_key_is_sync` - Sync trait
- `signature_is_send` - Send trait
- `signature_is_sync` - Sync trait
- `concurrent_signing` - 4 threads signing same message

### Constants (1 test)
- `signature_length_is_65` - SIGNATURE_LENGTH == 65

---

## Acceptance Criteria

For implementation to pass:
1. All 94 tests must pass
2. No panics from `todo!()` macros
3. All error types must match `CryptoError` variants
4. Known test vectors must match exactly
5. Thread safety (Send + Sync) must be verified
6. Private key Debug must not reveal bytes

## Dependencies Needed

The implementation will need:
- `sha3` crate for Keccak-256
- `k256` crate for secp256k1 ECDSA

## Next Steps

1. Coder agent implements `bach-crypto/src/lib.rs`
2. Run `cargo test -p bach-crypto` to verify
3. All tests should transition from FAIL to PASS
