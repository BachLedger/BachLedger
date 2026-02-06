//! Security tests for bach-sdk
//!
//! Tests for key safety, error message security, and compile-time safety.

// TODO: Uncomment when bach_sdk lib.rs is available
// use bach_sdk::{Wallet, SdkError};

// ==================== Private Key Safety Tests ====================

/// Test that private key is not exposed in Debug output
#[test]
fn test_private_key_not_in_debug() {
    // TODO: Implement
    // let wallet = Wallet::new_random();
    // let debug_str = format!("{:?}", wallet);
    //
    // // Should not contain "private_key" field
    // assert!(!debug_str.to_lowercase().contains("private"));
    // // Should contain address (safe to expose)
    // assert!(debug_str.contains("address"));
}

/// Test that private key bytes are not in Debug output
#[test]
fn test_private_key_bytes_not_in_debug() {
    // TODO: Implement
    // Create wallet from known key, verify those bytes don't appear in debug
    // let known_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    // let wallet = Wallet::from_private_key_hex(known_key).unwrap();
    // let debug_str = format!("{:?}", wallet);
    // assert!(!debug_str.contains("ac0974"));
}

// ==================== Error Message Security Tests ====================

/// Test invalid key error doesn't leak input
#[test]
fn test_invalid_key_error_no_leak() {
    // TODO: Implement
    // let bad_key = "sensitive_secret_key_data";
    // let result = Wallet::from_private_key_hex(bad_key);
    // if let Err(e) = result {
    //     let error_msg = e.to_string();
    //     assert!(!error_msg.contains("sensitive"));
    //     assert!(!error_msg.contains("secret"));
    // }
}

/// Test signing error doesn't leak private key
#[test]
fn test_signing_error_no_key_leak() {
    // TODO: Implement
    // Force a signing error and verify key isn't in message
}

/// Test all SdkError variants for sensitive data
#[test]
fn test_all_error_variants_safe() {
    // TODO: Implement
    // Create each error variant and verify no sensitive data in Display/Debug
}

// ==================== Compile-Time Safety Tests ====================
//
// These tests verify that certain unsafe patterns don't compile.
// They are implemented as trybuild compile-fail tests.
//
// To use trybuild:
// 1. Add trybuild as dev-dependency
// 2. Create ui/ directory with .rs files that should fail to compile
// 3. Create .stderr files with expected error messages

/// Marker test for compile-fail tests
///
/// The actual compile-fail tests are in tests/ui/ directory:
/// - ui/wallet_not_clone.rs - Wallet should not implement Clone
/// - ui/wallet_not_copy.rs - Wallet should not implement Copy
#[test]
#[ignore = "Compile-fail tests run separately via trybuild"]
fn test_compile_fail_tests_exist() {
    // This is a placeholder to document that compile-fail tests exist.
    // Run with: cargo test --test ui_tests
}

// ==================== Memory Safety Tests ====================

/// Test that wallet memory is properly handled
#[test]
fn test_wallet_drop() {
    // TODO: Implement if zeroize is used
    // This is hard to test directly, but we can verify the wallet
    // goes out of scope without issues
    // {
    //     let _wallet = Wallet::new_random();
    // }
    // Wallet should be dropped here
}

// ==================== Input Validation Security Tests ====================

/// Test rejection of oversized data
#[test]
fn test_reject_oversized_input() {
    // TODO: Implement
    // Very large inputs should be rejected or handled safely
}

/// Test handling of malformed hex input
#[test]
fn test_malformed_hex_input() {
    // TODO: Implement
    // Various malformed inputs should produce errors, not panics
    // let test_cases = vec![
    //     "0x",           // Just prefix
    //     "0xZZZZ",       // Invalid hex chars
    //     "0x123",        // Odd length
    //     "0x" + "ff".repeat(100), // Too long
    // ];
}

/// Test null byte handling
#[test]
fn test_null_bytes_in_input() {
    // TODO: Implement
    // Inputs with null bytes should be handled safely
}

// ==================== Signature Malleability Tests ====================

/// Test that high-s signatures are rejected during verification
#[test]
fn test_reject_high_s_signature() {
    // TODO: Implement
    // Manually create a signature with s > n/2 and verify it's rejected
}

/// Test that our signatures are always low-s (EIP-2)
#[test]
fn test_signatures_always_low_s() {
    // TODO: Implement
    // Sign many messages and verify all have s <= n/2
}

// ==================== Chain ID Security Tests ====================

/// Test replay protection with different chain IDs
#[test]
fn test_chain_id_replay_protection() {
    // TODO: Implement
    // Transaction signed for chain 1 should not validate on chain 5
}

/// Test zero chain ID is rejected
#[test]
fn test_zero_chain_id_rejected() {
    // TODO: Implement
    // Chain ID 0 might indicate missing replay protection
}
