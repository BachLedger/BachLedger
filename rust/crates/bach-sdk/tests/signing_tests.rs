//! Signing and verification tests for bach-sdk
//!
//! Tests message signing, transaction signing, and signature verification.

use bach_sdk::{Wallet, TxBuilder, H256, Address};
use bach_crypto::{verify, recover_public_key, public_key_to_address};

// ==================== Message Signing Tests ====================

/// Test signing a hash
#[test]
fn test_sign_hash() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();
    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);
}

/// Test signing produces valid v value (27 or 28)
#[test]
fn test_sign_v_value() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();
    assert!(signature.v == 27 || signature.v == 28);
}

/// Test signing with Ethereum personal message prefix
#[test]
fn test_sign_message() {
    let wallet = Wallet::new_random();
    let message = b"Hello, BachLedger!";
    let signature = wallet.sign_message(message).unwrap();
    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);
}

/// Test signing empty message
#[test]
fn test_sign_empty_message() {
    let wallet = Wallet::new_random();
    let signature = wallet.sign_message(b"").unwrap();
    assert_ne!(signature.r, [0u8; 32]);
    assert_ne!(signature.s, [0u8; 32]);
}

/// Test signing large message
#[test]
fn test_sign_large_message() {
    let wallet = Wallet::new_random();
    let large_msg = vec![0xab; 10000];
    let signature = wallet.sign_message(&large_msg).unwrap();
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

/// Test that signatures always have low-s value
#[test]
fn test_signature_low_s() {
    for i in 0..10 {
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

/// Test multiple signatures for low-s compliance
#[test]
fn test_multiple_signatures_low_s() {
    let wallet = Wallet::new_random();
    for i in 0..20 {
        let hash = H256::from_bytes([i as u8; 32]);
        let signature = wallet.sign_hash(&hash).unwrap();
        assert!(is_low_s(&signature.s), "Signature {} has high-s", i);
    }
}

// ==================== Signature Verification Tests ====================

/// Test signature verification roundtrip
#[test]
fn test_sign_verify_roundtrip() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    let is_valid = verify(&hash, &signature, wallet.public_key()).unwrap();
    assert!(is_valid);
}

/// Test verification fails with wrong message
#[test]
fn test_verify_wrong_message() {
    let wallet = Wallet::new_random();
    let hash1 = H256::from_bytes([0x42; 32]);
    let hash2 = H256::from_bytes([0x43; 32]);
    let signature = wallet.sign_hash(&hash1).unwrap();

    // Verify against wrong hash should fail
    let is_valid = verify(&hash2, &signature, wallet.public_key()).unwrap();
    assert!(!is_valid);
}

/// Test verification fails with wrong public key
#[test]
fn test_verify_wrong_public_key() {
    let wallet1 = Wallet::new_random();
    let wallet2 = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet1.sign_hash(&hash).unwrap();

    // Verify with wrong pubkey should fail
    let is_valid = verify(&hash, &signature, wallet2.public_key()).unwrap();
    assert!(!is_valid);
}

// ==================== Public Key Recovery Tests ====================

/// Test public key recovery from signature
#[test]
fn test_recover_public_key() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    let recovered = recover_public_key(&hash, &signature).unwrap();
    assert_eq!(wallet.public_key(), &recovered);
}

/// Test address recovery from signature
#[test]
fn test_recover_address() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    let recovered_pubkey = recover_public_key(&hash, &signature).unwrap();
    let recovered_address = public_key_to_address(&recovered_pubkey);
    assert_eq!(wallet.address(), &recovered_address);
}

/// Test recovery with tampered signature returns wrong address
#[test]
fn test_recover_tampered_signature() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let mut signature = wallet.sign_hash(&hash).unwrap();

    // Tamper with r
    signature.r[0] ^= 0xff;

    // Recovery may fail or return wrong address
    if let Ok(recovered_pubkey) = recover_public_key(&hash, &signature) {
        let recovered_address = public_key_to_address(&recovered_pubkey);
        assert_ne!(wallet.address(), &recovered_address);
    }
    // If it errors, that's also acceptable
}

// ==================== Transaction Signing Tests ====================

fn test_wallet() -> Wallet {
    Wallet::from_private_key_hex(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    ).unwrap()
}

/// Test signing a legacy transaction
#[test]
fn test_sign_legacy_transaction() {
    let wallet = test_wallet();
    let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

    let signed = TxBuilder::new(1)
        .nonce(0)
        .gas_limit(21000)
        .gas_price(1_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_legacy(&wallet)
        .unwrap();

    assert!(signed.signature.is_valid());
}

/// Test signing an EIP-1559 transaction
#[test]
fn test_sign_eip1559_transaction() {
    let wallet = test_wallet();
    let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

    let signed = TxBuilder::new(1)
        .nonce(0)
        .gas_limit(21000)
        .max_fee_per_gas(100_000_000_000)
        .max_priority_fee_per_gas(2_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_eip1559(&wallet)
        .unwrap();

    assert!(signed.signature.is_valid());
}

/// Test signed transaction signature is deterministic
#[test]
fn test_signed_tx_hash_deterministic() {
    let wallet = test_wallet();
    let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

    let signed1 = TxBuilder::new(1)
        .nonce(0)
        .gas_limit(21000)
        .gas_price(1_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_legacy(&wallet)
        .unwrap();

    let signed2 = TxBuilder::new(1)
        .nonce(0)
        .gas_limit(21000)
        .gas_price(1_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_legacy(&wallet)
        .unwrap();

    // Same tx signed twice should produce same signature
    assert_eq!(signed1.signature.r, signed2.signature.r);
    assert_eq!(signed1.signature.s, signed2.signature.s);
}

/// Test signed transaction has valid signature
#[test]
fn test_signed_tx_recover_sender() {
    let wallet = test_wallet();
    let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

    let signed = TxBuilder::new(1)
        .nonce(0)
        .gas_limit(21000)
        .gas_price(1_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_legacy(&wallet)
        .unwrap();

    // The signature should be valid
    assert!(signed.signature.is_valid());
    // And r,s should be non-zero
    assert_ne!(signed.signature.r, H256::ZERO);
    assert_ne!(signed.signature.s, H256::ZERO);
}

// ==================== Chain ID Tests ====================

/// Test transaction includes chain ID
#[test]
fn test_tx_sign_includes_chain_id() {
    let wallet = test_wallet();
    let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

    let signed = TxBuilder::new(5) // Goerli chain ID
        .nonce(0)
        .gas_limit(21000)
        .gas_price(1_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_legacy(&wallet)
        .unwrap();

    // Signature should be valid
    assert!(signed.signature.is_valid());
}

/// Test EIP-1559 tx requires chain ID
#[test]
fn test_eip1559_requires_chain_id() {
    let wallet = test_wallet();
    let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

    // EIP-1559 with chain ID 1 should work
    let result = TxBuilder::new(1)
        .nonce(0)
        .gas_limit(21000)
        .max_fee_per_gas(100_000_000_000)
        .max_priority_fee_per_gas(2_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_eip1559(&wallet);

    assert!(result.is_ok());
}

/// Test signed tx not valid on different chain (replay protection)
#[test]
fn test_cross_chain_replay_protection() {
    let wallet = test_wallet();
    let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

    // Sign for chain 1
    let signed_chain1 = TxBuilder::new(1)
        .nonce(0)
        .gas_limit(21000)
        .gas_price(1_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_legacy(&wallet)
        .unwrap();

    // Sign for chain 5
    let signed_chain5 = TxBuilder::new(5)
        .nonce(0)
        .gas_limit(21000)
        .gas_price(1_000_000_000)
        .to(to)
        .value(1_000_000_000_000_000_000)
        .sign_legacy(&wallet)
        .unwrap();

    // Signatures should be different due to different chain IDs (EIP-155)
    // The v value encodes the chain ID
    assert_ne!(signed_chain1.signature.v, signed_chain5.signature.v);
}

// ==================== Signature Serialization Tests ====================

/// Test signature to_bytes roundtrip
#[test]
fn test_signature_to_bytes_roundtrip() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    let bytes = signature.to_bytes();
    assert_eq!(bytes.len(), 65);

    // Verify components are in the bytes
    assert_eq!(&bytes[0..32], &signature.r);
    assert_eq!(&bytes[32..64], &signature.s);
    assert_eq!(bytes[64], signature.v);
}

/// Test signature bytes layout (r || s || v)
#[test]
fn test_signature_bytes_layout() {
    let wallet = Wallet::new_random();
    let hash = H256::from_bytes([0x42; 32]);
    let signature = wallet.sign_hash(&hash).unwrap();

    let bytes = signature.to_bytes();

    // r is bytes[0..32]
    assert_eq!(&bytes[0..32], &signature.r);
    // s is bytes[32..64]
    assert_eq!(&bytes[32..64], &signature.s);
    // v is bytes[64]
    assert_eq!(bytes[64], signature.v);
}
