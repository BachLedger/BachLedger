# BachLedger SDK & CLI Test Plan

## Overview

This document outlines the comprehensive test strategy for `bach-sdk` and `bach-cli` crates.
Tests are organized by component and priority, with focus on both happy paths and edge cases.

---

## Part 1: bach-sdk Tests

### 1.1 Client Lifecycle Tests

#### Construction & Configuration
```rust
#[cfg(test)]
mod client_tests {
    // Basic client creation
    - test_client_new_with_default_config()
    - test_client_new_with_custom_endpoint()
    - test_client_new_with_invalid_url() -> Error
    - test_client_new_with_custom_timeout()
    - test_client_new_with_chain_id()

    // Client builder pattern
    - test_client_builder_all_options()
    - test_client_builder_minimal()
    - test_client_builder_missing_required_field() -> Error

    // Connection handling
    - test_client_connect_success()
    - test_client_connect_timeout()
    - test_client_connect_invalid_endpoint()
    - test_client_reconnect_after_disconnect()
}
```

#### Health & Status
```rust
    - test_client_health_check()
    - test_client_get_chain_id()
    - test_client_get_block_number()
    - test_client_is_syncing()
```

### 1.2 Account & Wallet Tests

#### Key Management
```rust
#[cfg(test)]
mod wallet_tests {
    // Key generation
    - test_generate_random_keypair()
    - test_keypair_determinism_from_seed()
    - test_private_key_from_hex()
    - test_private_key_from_invalid_hex() -> Error
    - test_private_key_from_bytes()
    - test_private_key_from_wrong_length_bytes() -> Error

    // Address derivation
    - test_address_from_private_key()
    - test_address_from_public_key()
    - test_address_derivation_matches_ethereum() // Known test vector
    - test_known_private_key_address() // ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 -> 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

    // Key serialization (SECURITY CRITICAL)
    - test_private_key_to_hex()
    - test_private_key_debug_does_not_leak() // Debug impl should mask key
    - test_private_key_display_does_not_leak()
    - test_private_key_clone()
    - test_private_key_zeroize_on_drop() // If using zeroize
}
```

#### Account State
```rust
#[cfg(test)]
mod account_tests {
    - test_get_balance()
    - test_get_balance_zero_address()
    - test_get_balance_nonexistent_account()
    - test_get_nonce()
    - test_get_nonce_fresh_account()
    - test_get_code_contract()
    - test_get_code_eoa() // Should be empty
    - test_get_storage_at()
}
```

### 1.3 Transaction Building Tests

#### Legacy Transactions (Type 0)
```rust
#[cfg(test)]
mod legacy_tx_tests {
    // Basic construction
    - test_legacy_tx_simple_transfer()
    - test_legacy_tx_contract_call()
    - test_legacy_tx_contract_creation()

    // Field validation
    - test_legacy_tx_default_gas_limit() // Should be 21000 for transfers
    - test_legacy_tx_custom_gas_limit()
    - test_legacy_tx_zero_gas_price()
    - test_legacy_tx_max_gas_price()
    - test_legacy_tx_zero_value()
    - test_legacy_tx_max_value()
    - test_legacy_tx_with_data()
    - test_legacy_tx_empty_data()

    // Address handling
    - test_legacy_tx_to_none_is_contract_creation()
    - test_legacy_tx_to_zero_address()
    - test_legacy_tx_to_checksum_address()
    - test_legacy_tx_to_lowercase_address()
}
```

#### EIP-1559 Transactions (Type 2)
```rust
#[cfg(test)]
mod eip1559_tx_tests {
    // Basic construction
    - test_eip1559_tx_simple_transfer()
    - test_eip1559_tx_contract_call()
    - test_eip1559_tx_contract_creation()

    // Fee parameters
    - test_eip1559_tx_max_fee_validation()
    - test_eip1559_tx_priority_fee_validation()
    - test_eip1559_tx_priority_exceeds_max_fee() -> Error
    - test_eip1559_tx_zero_priority_fee()
    - test_eip1559_tx_effective_gas_price_calculation()
    - test_eip1559_tx_effective_gas_price_capped()
    - test_eip1559_tx_base_fee_too_high() -> None

    // Chain ID
    - test_eip1559_tx_requires_chain_id()
    - test_eip1559_tx_chain_id_mismatch() -> Error

    // Access list
    - test_eip1559_tx_empty_access_list()
    - test_eip1559_tx_with_access_list()
    - test_eip1559_tx_access_list_multiple_entries()
    - test_eip1559_tx_access_list_storage_keys()
}
```

#### Transaction Builder
```rust
#[cfg(test)]
mod tx_builder_tests {
    // Builder pattern
    - test_tx_builder_minimal()
    - test_tx_builder_all_fields()
    - test_tx_builder_missing_to_for_transfer() -> Error
    - test_tx_builder_auto_nonce() // Fetches from chain
    - test_tx_builder_manual_nonce()
    - test_tx_builder_auto_gas_estimation()
    - test_tx_builder_manual_gas()

    // Type inference
    - test_tx_builder_infers_legacy_type()
    - test_tx_builder_infers_eip1559_type()
    - test_tx_builder_explicit_type_override()
}
```

### 1.4 Signing & Verification Tests

#### Message Signing
```rust
#[cfg(test)]
mod signing_tests {
    // Basic signing
    - test_sign_message_hash()
    - test_sign_produces_low_s() // EIP-2 compliance
    - test_sign_deterministic_for_same_input()
    - test_sign_different_for_different_messages()
    - test_sign_zero_hash()
    - test_sign_max_hash()

    // Signature format
    - test_signature_v_value_27_or_28()
    - test_signature_r_not_zero()
    - test_signature_s_not_zero()
    - test_signature_to_bytes_65()
    - test_signature_from_bytes_roundtrip()

    // Verification
    - test_verify_valid_signature()
    - test_verify_wrong_message() -> false
    - test_verify_wrong_public_key() -> false
    - test_verify_tampered_r() -> false/Error
    - test_verify_tampered_s() -> false/Error
    - test_verify_high_s_rejected() // EIP-2

    // Recovery
    - test_recover_public_key()
    - test_recover_address()
    - test_recover_invalid_recovery_id() -> Error
    - test_recover_matches_original_signer()
}
```

#### Transaction Signing
```rust
#[cfg(test)]
mod tx_signing_tests {
    // Sign transactions
    - test_sign_legacy_tx()
    - test_sign_eip1559_tx()
    - test_sign_contract_creation_tx()

    // Signed transaction properties
    - test_signed_tx_hash_deterministic()
    - test_signed_tx_recoverable_sender()
    - test_signed_tx_signature_valid()

    // RLP encoding
    - test_signed_tx_rlp_encode()
    - test_signed_tx_rlp_decode_roundtrip()
    - test_signed_tx_raw_bytes()
}
```

### 1.5 RPC Method Tests (with mocks)

#### Block Methods
```rust
#[cfg(test)]
mod rpc_block_tests {
    - test_get_block_by_number()
    - test_get_block_by_hash()
    - test_get_block_latest()
    - test_get_block_pending()
    - test_get_block_not_found() -> None
    - test_get_block_with_transactions()
    - test_get_block_transaction_count()
}
```

#### Transaction Methods
```rust
#[cfg(test)]
mod rpc_tx_tests {
    - test_send_raw_transaction()
    - test_send_raw_transaction_invalid() -> Error
    - test_get_transaction_by_hash()
    - test_get_transaction_not_found() -> None
    - test_get_transaction_receipt()
    - test_get_transaction_receipt_pending() -> None
    - test_estimate_gas()
    - test_estimate_gas_revert() -> Error with reason
}
```

#### Call Methods
```rust
#[cfg(test)]
mod rpc_call_tests {
    - test_eth_call()
    - test_eth_call_revert() -> Error with reason
    - test_eth_call_at_block_number()
    - test_eth_call_state_override()
}
```

### 1.6 Error Handling Tests

```rust
#[cfg(test)]
mod error_tests {
    // Network errors
    - test_error_connection_refused()
    - test_error_timeout()
    - test_error_invalid_response()

    // RPC errors
    - test_error_rpc_method_not_found()
    - test_error_rpc_invalid_params()
    - test_error_nonce_too_low()
    - test_error_insufficient_funds()
    - test_error_gas_too_low()
    - test_error_execution_reverted()

    // Validation errors
    - test_error_invalid_address()
    - test_error_invalid_private_key()
    - test_error_invalid_signature()

    // Error messages
    - test_error_display_human_readable()
    - test_error_source_chain()
}
```

### 1.7 Security Tests

```rust
#[cfg(test)]
mod security_tests {
    // Key safety
    - test_private_key_not_in_debug_output()
    - test_private_key_not_in_error_messages()
    - test_private_key_not_logged() // Check tracing output
    - test_private_key_zeroed_on_drop() // Memory safety - verify key is zeroed

    // Wallet type safety (compile-time tests)
    - test_wallet_not_clone() // Compile-fail test: Wallet should NOT impl Clone
    - test_wallet_not_copy()  // Compile-fail test: Wallet should NOT impl Copy

    // Signature malleability
    - test_reject_high_s_signatures()
    - test_signatures_always_low_s()

    // Input validation
    - test_reject_oversized_data()
    - test_reject_negative_values() // If using signed types
}
```

### 1.8 Chain ID & Replay Protection Tests

```rust
#[cfg(test)]
mod chain_id_tests {
    // Chain ID validation
    - test_tx_sign_includes_chain_id()
    - test_tx_sign_wrong_chain_id_rejected()
    - test_tx_verify_chain_id_mismatch_fails()

    // Replay protection
    - test_signed_tx_not_valid_on_different_chain()
    - test_legacy_tx_replay_protection() // EIP-155
    - test_eip1559_tx_chain_id_required()

    // Cross-chain scenarios
    - test_mainnet_tx_rejected_on_testnet()
    - test_chain_id_zero_rejected()
    - test_chain_id_max_value()
}
```

### 1.9 Error Message Security Tests

```rust
#[cfg(test)]
mod error_security_tests {
    // Private key must NEVER appear in errors
    - test_invalid_key_error_no_key_leak()
    - test_signing_error_no_key_leak()
    - test_wallet_creation_error_no_key_leak()

    // Error message inspection
    - test_all_error_variants_no_sensitive_data()
    - test_error_display_safe()
    - test_error_debug_safe()
    - test_error_source_chain_safe()
}
```

---

## Part 2: bach-cli Tests

### 2.1 Command Parsing Tests

#### Global Options
```rust
#[cfg(test)]
mod cli_global_tests {
    - test_cli_help()
    - test_cli_version()
    - test_cli_endpoint_flag()
    - test_cli_endpoint_env_var()
    - test_cli_config_file()
    - test_cli_output_json_flag()
    - test_cli_output_text_default()
}
```

#### Account Commands
```rust
#[cfg(test)]
mod cli_account_tests {
    // bach account new
    - test_account_new()
    - test_account_new_json_output()
    - test_account_new_does_not_print_key_by_default()
    - test_account_new_show_private_key_flag()

    // bach account balance
    - test_account_balance_address()
    - test_account_balance_invalid_address() -> Error
    - test_account_balance_json_output()

    // bach account nonce
    - test_account_nonce()
}
```

#### Transaction Commands
```rust
#[cfg(test)]
mod cli_tx_tests {
    // bach tx send
    - test_tx_send_simple()
    - test_tx_send_with_data()
    - test_tx_send_with_value()
    - test_tx_send_missing_to() -> Error
    - test_tx_send_missing_private_key() -> Error
    - test_tx_send_json_output()

    // bach tx call
    - test_tx_call()
    - test_tx_call_with_data()

    // bach tx decode
    - test_tx_decode_raw()
    - test_tx_decode_invalid() -> Error

    // bach tx sign
    - test_tx_sign_offline()
    - test_tx_sign_output_raw()
}
```

#### Block Commands
```rust
#[cfg(test)]
mod cli_block_tests {
    // bach block get
    - test_block_get_by_number()
    - test_block_get_latest()
    - test_block_get_by_hash()
    - test_block_get_json_output()
}
```

#### Key Commands
```rust
#[cfg(test)]
mod cli_key_tests {
    // bach key generate
    - test_key_generate()
    - test_key_generate_json()

    // bach key inspect
    - test_key_inspect_private_key()
    - test_key_inspect_public_key()
    - test_key_inspect_address()
    - test_key_inspect_invalid() -> Error

    // bach key sign
    - test_key_sign_message()
    - test_key_sign_hash()

    // bach key verify
    - test_key_verify_valid()
    - test_key_verify_invalid() -> false
}
```

### 2.2 Output Format Tests

```rust
#[cfg(test)]
mod cli_output_tests {
    // Text output
    - test_text_output_readable()
    - test_text_output_aligned()
    - test_text_output_colored() // If using colors

    // JSON output
    - test_json_output_valid()
    - test_json_output_parseable()
    - test_json_output_consistent_keys()

    // Error output
    - test_error_output_to_stderr()
    - test_error_exit_code_nonzero()
    - test_error_message_helpful()
}
```

### 2.3 Config File Tests

```rust
#[cfg(test)]
mod cli_config_tests {
    - test_config_file_load()
    - test_config_file_not_found() // Should use defaults
    - test_config_file_invalid_format() -> Error
    - test_config_env_override()
    - test_config_flag_override_env()
    - test_config_default_values()
}
```

### 2.4 Integration Tests

```rust
#[cfg(test)]
mod cli_integration_tests {
    // End-to-end workflows
    - test_workflow_generate_key_and_get_address()
    - test_workflow_sign_and_verify_message()
    - test_workflow_build_sign_send_transaction()

    // Piping
    - test_pipe_key_generate_to_inspect()
    - test_pipe_tx_sign_to_send()
}
```

---

## Part 3: Test Infrastructure

### 3.1 Mock Server

```rust
// Mock RPC server for testing without real node
struct MockRpcServer {
    responses: HashMap<String, Value>,
}

impl MockRpcServer {
    fn new() -> Self;
    fn expect(method: &str, params: Value, response: Value);
    fn start() -> String; // Returns URL
}
```

### 3.2 Test Fixtures

```rust
// Common test data
mod fixtures {
    // Known test keys (DO NOT USE IN PRODUCTION)
    const TEST_PRIVATE_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const TEST_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    // Sample transactions
    fn sample_legacy_tx() -> LegacyTx;
    fn sample_eip1559_tx() -> DynamicFeeTx;
    fn sample_signed_tx() -> SignedTransaction;

    // Sample blocks
    fn sample_block() -> Block;
    fn sample_block_header() -> BlockHeader;
}
```

### 3.3 Property-Based Tests

```rust
// Using proptest for fuzzing
mod proptest_tests {
    // Roundtrip properties
    - proptest_signature_roundtrip()
    - proptest_transaction_rlp_roundtrip()
    - proptest_address_hex_roundtrip()

    // Invariants
    - proptest_signed_tx_recoverable()
    - proptest_signature_always_low_s()
}
```

---

## Part 4: Test Priorities

### P0 - Critical (Must pass before merge)
- Key generation and address derivation
- Transaction signing
- Signature verification
- Private key security (no leaks)

### P1 - High (Required for release)
- All transaction building
- RPC method wrappers
- CLI command parsing
- Error handling

### P2 - Medium (Nice to have)
- Property-based tests
- Performance benchmarks
- Edge case coverage

### P3 - Low (Future)
- Stress tests
- Compatibility tests with other clients

---

## Part 5: Coverage Goals

| Component | Target Coverage |
|-----------|-----------------|
| bach-sdk core | 90% |
| bach-sdk RPC | 80% |
| bach-cli commands | 85% |
| Error handling | 95% |

---

## Part 6: Known Test Vectors

### Ethereum Compatibility

```
Private Key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
Expected Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

Private Key: 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
Verify address derivation is consistent

Keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
Keccak256("hello") = 0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
```

### EIP-2 Low-S Test Vectors

All signatures must have s <= secp256k1_n/2 where:
```
secp256k1_n = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
secp256k1_n/2 = 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0
```

---

## Appendix: Test File Structure

```
crates/bach-sdk/
  src/
    lib.rs
    client.rs
    wallet.rs
    transaction.rs
    ...
  tests/
    client_tests.rs
    wallet_tests.rs
    transaction_tests.rs
    signing_tests.rs
    rpc_tests.rs
    security_tests.rs
    integration_tests.rs

crates/bach-cli/
  src/
    main.rs
    commands/
      account.rs
      tx.rs
      block.rs
      key.rs
  tests/
    cli_tests.rs
    command_tests.rs
    output_tests.rs
```
