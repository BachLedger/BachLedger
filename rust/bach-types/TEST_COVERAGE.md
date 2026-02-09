# bach-types Test Coverage Summary

## Overview
This document summarizes the test coverage for the `bach-types` module.
Tests are written using TDD (Test-Driven Development) - they are written BEFORE implementation.

**Status**: Tests compile but FAIL (Red phase - implementation needed)

## Test Files

| File | Type | Tests |
|------|------|-------|
| `tests/priority_tests.rs` | PriorityCode | 32 tests |
| `tests/rwset_tests.rs` | ReadWriteSet | 28 tests |
| `tests/transaction_tests.rs` | Transaction, TypeError | 28 tests |
| `tests/block_tests.rs` | Block | 27 tests |
| **Total** | | **115 tests** |

---

## PriorityCode Tests (`tests/priority_tests.rs`)

### Constants (2 tests)
- `priority_owned_is_zero` - PRIORITY_OWNED == 0
- `priority_disowned_is_one` - PRIORITY_DISOWNED == 1

### new() (5 tests)
- `creates_with_owned_status` - Starts not released
- `stores_block_height` - Correct height storage
- `stores_hash` - Correct hash storage
- `zero_block_height` - Height = 0
- `max_block_height` - Height = u64::MAX

### release() (4 tests)
- `changes_to_disowned` - Sets release bit
- `release_is_idempotent` - Multiple releases OK
- `release_does_not_change_block_height` - Height preserved
- `release_does_not_change_hash` - Hash preserved

### is_released() (2 tests)
- `false_when_new` - Initially not released
- `true_after_release` - Released after release()

### Ordering (11 tests)
- `owned_has_higher_priority_than_disowned` - Owned < Disowned
- `lower_block_height_has_higher_priority` - Lower height wins
- `lower_hash_has_higher_priority` - Lower hash wins
- `ordering_release_bit_first` - Release bit is primary key
- `ordering_block_height_second` - Height is secondary key
- `ordering_hash_third` - Hash is tertiary key
- `equal_priority_codes` - Equality check
- `partial_ord_consistent_with_ord` - Consistency
- `can_be_sorted` - Vec sorting works

### Serialization (9 tests)
- `to_bytes_length` - 41 bytes
- `from_bytes_roundtrip` - Roundtrip owned
- `from_bytes_roundtrip_released` - Roundtrip released
- `bytes_structure_release_bit_first` - Byte 0 = release_bit
- `bytes_structure_release_bit_disowned` - DISOWNED = 1
- `bytes_structure_block_height` - Bytes 1-8 = height BE
- `bytes_structure_hash` - Bytes 9-40 = hash
- `from_bytes_zero_block_height` - Roundtrip height=0
- `from_bytes_max_block_height` - Roundtrip height=MAX

### Traits (3 tests)
- `debug_is_implemented`, `clone_works`, `eq_works`

### Thread Safety (2 tests)
- `priority_code_is_send`, `priority_code_is_sync`

---

## ReadWriteSet Tests (`tests/rwset_tests.rs`)

### new() (2 tests)
- `creates_empty_set` - Empty reads/writes
- `all_keys_empty_initially` - No keys

### record_read() (4 tests)
- `adds_key_to_reads` - Single read
- `multiple_reads` - Multiple reads
- `duplicate_reads_allowed` - Same key multiple times
- `does_not_affect_writes` - Reads don't affect writes

### record_write() (6 tests)
- `adds_key_value_to_writes` - Single write
- `multiple_writes` - Multiple writes
- `empty_value` - Write empty Vec
- `large_value` - Write 1KB value
- `duplicate_writes_allowed` - Same key multiple times
- `does_not_affect_reads` - Writes don't affect reads

### reads() (2 tests)
- `returns_slice_of_read_keys` - Returns &[H256]
- `preserves_order` - Insertion order

### writes() (2 tests)
- `returns_slice_of_write_pairs` - Returns &[(H256, Vec<u8>)]
- `preserves_order` - Insertion order

### all_keys() (5 tests)
- `includes_read_keys` - Reads included
- `includes_write_keys` - Writes included
- `combines_reads_and_writes` - Both combined
- `returns_unique_keys` - Deduplicated
- `handles_multiple_reads_of_same_key` - Dedup works

### clear() (4 tests)
- `clears_reads` - Reads cleared
- `clears_writes` - Writes cleared
- `clears_all_keys` - all_keys() empty
- `can_add_after_clear` - Reuse after clear

### Traits (3 tests)
- `default_creates_empty_set`, `default_equals_new`, `debug_is_implemented`

### Clone (3 tests)
- `clone_preserves_reads`, `clone_preserves_writes`, `clone_is_independent`

---

## Transaction Tests (`tests/transaction_tests.rs`)

### new() (7 tests)
- `creates_transaction_with_fields` - All fields set
- `creates_contract_creation_transaction` - to = None
- `zero_nonce`, `max_nonce` - Edge cases
- `large_value` - U256::MAX
- `empty_data`, `large_data` - Data variations

### hash() (7 tests)
- `returns_32_byte_hash` - Correct length
- `deterministic` - Same tx = same hash
- `different_nonce_different_hash` - Nonce affects hash
- `different_to_different_hash` - Recipient affects hash
- `different_value_different_hash` - Value affects hash
- `different_data_different_hash` - Data affects hash
- `contract_creation_vs_transfer_different_hash` - to=None differs

### sender() (5 tests)
- `recovers_correct_address` - Correct address recovery
- `consistent_recovery` - Multiple calls same result
- `different_transactions_same_sender` - Same signer
- `different_signers_different_senders` - Different signers
- `contract_creation_sender` - Works for to=None

### signing_hash() (4 tests)
- `returns_32_byte_hash` - Correct length
- `deterministic` - Same tx = same hash
- `different_nonce_different_signing_hash` - Nonce affects
- `signing_hash_different_from_tx_hash` - Different from tx.hash()

### TypeError (5 tests)
- `error_variants_exist` - All variants
- `error_is_debug`, `error_is_clone`, `error_is_eq`
- `invalid_transaction_contains_message` - String message

### Traits (3 tests)
- `debug_is_implemented`, `clone_works`, `eq_works`

### Thread Safety (2 tests)
- `transaction_is_send`, `transaction_is_sync`

---

## Block Tests (`tests/block_tests.rs`)

### new() (6 tests)
- `creates_block_with_fields` - All fields set
- `genesis_block` - Height=0, parent=zero
- `empty_transactions` - No transactions
- `max_height`, `max_timestamp` - u64::MAX
- `many_transactions` - 100 transactions

### hash() (8 tests)
- `returns_32_byte_hash` - Correct length
- `deterministic` - Same block = same hash
- `different_height_different_hash` - Height affects hash
- `different_parent_hash_different_hash` - Parent affects hash
- `different_timestamp_different_hash` - Timestamp affects
- `different_transactions_different_hash` - Transactions affect
- `empty_vs_nonempty_transactions_different_hash`
- `genesis_block_hash` - Genesis has non-zero hash

### transactions_hash() (6 tests)
- `returns_32_byte_hash` - Correct length
- `deterministic` - Same txs = same hash
- `empty_transactions_hash` - Works for empty
- `different_transactions_different_hash` - Txs affect hash
- `order_matters` - Transaction order matters
- `independent_of_block_metadata` - Only txs matter

### transaction_count() (4 tests)
- `zero_for_empty_block` - 0 for empty
- `correct_for_single_transaction` - 1 tx
- `correct_for_multiple_transactions` - N txs
- `matches_transactions_len` - == transactions.len()

### Traits (4 tests)
- `debug_is_implemented`, `clone_works`, `eq_works`, `not_equal_different_height`

### Thread Safety (2 tests)
- `block_is_send`, `block_is_sync`

### Integration (2 tests)
- `chain_of_blocks` - Parent hash chaining
- `block_hash_includes_transactions` - Txs affect block hash

---

## Acceptance Criteria

For implementation to pass:
1. All 115 tests must pass
2. No panics from `todo!()` macros
3. All error types must match `TypeError` variants
4. Thread safety (Send + Sync) must be verified
5. PriorityCode ordering: release_bit > block_height > hash

## Dependencies

- `bach-primitives`: Address, H256, U256
- `bach-crypto`: Signature, PrivateKey, keccak256

## Next Steps

1. Coder agent implements `bach-types/src/lib.rs`
2. Run `cargo test -p bach-types` to verify
3. All tests should transition from FAIL to PASS
