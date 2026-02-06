//! Wallet and account tests for bach-sdk
//!
//! Tests key generation, import/export, and address derivation.

use bach_sdk::{Wallet, SdkError, Address, H256};
use bach_crypto::{verify, recover_public_key, public_key_to_address};

// ==================== Key Generation Tests ====================

#[test]
fn test_wallet_new_random() {
    let wallet = Wallet::new_random();
    assert_ne!(wallet.address(), &Address::ZERO);
}

#[test]
fn test_wallet_random_unique() {
    let wallet1 = Wallet::new_random();
    let wallet2 = Wallet::new_random();
    assert_ne!(wallet1.address(), wallet2.address());
}

#[test]
fn test_wallet_random_multiple_unique() {
    // Generate 10 wallets and ensure all addresses are unique
    let wallets: Vec<_> = (0..10).map(|_| Wallet::new_random()).collect();
    for i in 0..wallets.len() {
        for j in (i + 1)..wallets.len() {
            assert_ne!(wallets[i].address(), wallets[j].address());
        }
    }
}

// ==================== Key Import Tests ====================

#[test]
fn test_wallet_from_private_key_bytes() {
    let key = [0x42u8; 32];
    let wallet = Wallet::from_private_key(&key).unwrap();
    assert_ne!(wallet.address(), &Address::ZERO);
}

#[test]
fn test_wallet_from_hex_with_prefix() {
    let wallet = Wallet::from_private_key_hex(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    ).unwrap();
    assert_eq!(
        wallet.address().to_hex().to_lowercase(),
        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
    );
}

#[test]
fn test_wallet_from_hex_without_prefix() {
    let wallet = Wallet::from_private_key_hex(
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    ).unwrap();
    assert_eq!(
        wallet.address().to_hex().to_lowercase(),
        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
    );
}

#[test]
fn test_wallet_from_hex_invalid_length_short() {
    let result = Wallet::from_private_key_hex("0x1234");
    assert!(result.is_err());
    if let Err(SdkError::InvalidPrivateKey(msg)) = result {
        assert!(msg.contains("32 bytes"));
    }
}

#[test]
fn test_wallet_from_hex_invalid_length_long() {
    // 33 bytes (66 hex chars)
    let result = Wallet::from_private_key_hex(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80ff"
    );
    assert!(result.is_err());
}

#[test]
fn test_wallet_from_hex_invalid_chars() {
    let result = Wallet::from_private_key_hex("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG");
    assert!(result.is_err());
}

#[test]
fn test_wallet_from_hex_empty() {
    let result = Wallet::from_private_key_hex("");
    assert!(result.is_err());
}

#[test]
fn test_wallet_from_hex_only_prefix() {
    let result = Wallet::from_private_key_hex("0x");
    assert!(result.is_err());
}

// ==================== Address Derivation Tests ====================

/// Test known Ethereum test vector (Hardhat/Foundry first account)
#[test]
fn test_known_address_derivation_hardhat() {
    // Private: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
    // Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    let wallet = Wallet::from_private_key_hex(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    ).unwrap();

    assert_eq!(
        wallet.address().to_hex().to_lowercase(),
        "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
    );
}

#[test]
fn test_address_derivation_deterministic() {
    let key = [0x42u8; 32];
    let wallet1 = Wallet::from_private_key(&key).unwrap();
    let wallet2 = Wallet::from_private_key(&key).unwrap();
    assert_eq!(wallet1.address(), wallet2.address());
}

#[test]
fn test_address_is_20_bytes() {
    let wallet = Wallet::new_random();
    assert_eq!(wallet.address().as_bytes().len(), 20);
}

// ==================== Public Key Tests ====================

#[test]
fn test_wallet_public_key_consistent() {
    let key = [0x42u8; 32];
    let wallet1 = Wallet::from_private_key(&key).unwrap();
    let wallet2 = Wallet::from_private_key(&key).unwrap();

    // Public keys should be identical for same private key
    let pk1 = wallet1.public_key();
    let pk2 = wallet2.public_key();
    assert_eq!(pk1, pk2);
}

#[test]
fn test_address_matches_public_key() {
    let wallet = Wallet::new_random();
    let derived_address = public_key_to_address(wallet.public_key());
    assert_eq!(wallet.address(), &derived_address);
}

// ==================== Security Tests ====================

#[test]
fn test_wallet_debug_hides_private_key() {
    let wallet = Wallet::new_random();
    let debug_output = format!("{:?}", wallet);

    // Should contain Wallet and address
    assert!(debug_output.contains("Wallet"));
    assert!(debug_output.contains("address"));

    // Should NOT contain private_key
    assert!(!debug_output.to_lowercase().contains("private"));
}

#[test]
fn test_wallet_debug_hides_key_bytes() {
    // Create wallet from known key
    let known_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let wallet = Wallet::from_private_key_hex(known_key).unwrap();
    let debug_output = format!("{:?}", wallet);

    // Key bytes should not appear in debug output
    assert!(!debug_output.contains("ac0974"));
    assert!(!debug_output.contains("f2ff80"));
}

#[test]
fn test_error_message_no_key_leak() {
    // Try to create wallet with invalid input
    let bad_input = "sensitive_secret_data_here";
    let result = Wallet::from_private_key_hex(bad_input);

    if let Err(e) = result {
        let error_msg = e.to_string();
        // Error should not echo back the sensitive input
        assert!(!error_msg.contains("sensitive"));
        assert!(!error_msg.contains("secret"));
    }
}

// ==================== Signing Tests ====================

#[test]
fn test_wallet_sign_hash() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    // Signature components should be non-zero
    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);

    // v should be 27 or 28
    assert!(signature.v == 27 || signature.v == 28);
}

#[test]
fn test_wallet_sign_message() {
    let wallet = Wallet::new_random();
    let message = b"Hello, BachLedger!";
    let signature = wallet.sign_message(message).unwrap();

    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);
}

#[test]
fn test_wallet_sign_empty_message() {
    let wallet = Wallet::new_random();
    let signature = wallet.sign_message(b"").unwrap();

    // Empty message signing should succeed
    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);
}

#[test]
fn test_wallet_sign_large_message() {
    let wallet = Wallet::new_random();
    let large_msg = vec![0xab; 10000];
    let signature = wallet.sign_message(&large_msg).unwrap();

    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);
}

#[test]
fn test_wallet_sign_zero_hash() {
    let wallet = Wallet::new_random();
    let zero_hash = H256::ZERO;
    let signature = wallet.sign_hash(&zero_hash).unwrap();

    // Should succeed with valid signature
    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);
}

#[test]
fn test_wallet_sign_max_hash() {
    let wallet = Wallet::new_random();
    let max_hash = H256::from_bytes([0xff; 32]);
    let signature = wallet.sign_hash(&max_hash).unwrap();

    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);
}

// ==================== EIP-2 Low-S Tests ====================

/// secp256k1_n/2 constant for low-s verification
const SECP256K1_N_DIV_2: [u8; 32] = [
    0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D,
    0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
];

fn is_low_s(s: &[u8; 32]) -> bool {
    for i in 0..32 {
        match s[i].cmp(&SECP256K1_N_DIV_2[i]) {
            std::cmp::Ordering::Less => return true,
            std::cmp::Ordering::Greater => return false,
            std::cmp::Ordering::Equal => continue,
        }
    }
    true // s == n/2 is considered low-s
}

#[test]
fn test_signature_always_low_s() {
    // Sign multiple messages and verify all have low-s
    for i in 0..20 {
        let wallet = Wallet::new_random();
        let hash = H256::from_bytes([i as u8; 32]);
        let signature = wallet.sign_hash(&hash).unwrap();

        assert!(
            is_low_s(&signature.s),
            "Signature {} has high-s value (EIP-2 violation)",
            i
        );
    }
}

// ==================== Signature Verification Tests ====================

#[test]
fn test_sign_verify_roundtrip() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    let is_valid = verify(&hash, &signature, wallet.public_key()).unwrap();
    assert!(is_valid);
}

#[test]
fn test_sign_recover_roundtrip() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    let recovered = recover_public_key(&hash, &signature).unwrap();
    assert_eq!(wallet.public_key(), &recovered);
}

#[test]
fn test_sign_recover_address() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    let recovered_pubkey = recover_public_key(&hash, &signature).unwrap();
    let recovered_address = public_key_to_address(&recovered_pubkey);
    assert_eq!(wallet.address(), &recovered_address);
}
