//! Tests for ECDSA Signature, PrivateKey, and PublicKey
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_crypto::{keccak256, CryptoError, PrivateKey, PublicKey, Signature, SIGNATURE_LENGTH};
use bach_primitives::H256;

// =============================================================================
// CryptoError tests
// =============================================================================

mod crypto_error {
    use super::*;

    #[test]
    fn error_variants_exist() {
        let _ = CryptoError::InvalidPrivateKey;
        let _ = CryptoError::InvalidSignature;
        let _ = CryptoError::RecoveryFailed;
        let _ = CryptoError::InvalidPublicKey;
    }

    #[test]
    fn error_is_debug() {
        let err = CryptoError::InvalidPrivateKey;
        let debug = format!("{:?}", err);
        assert!(debug.contains("InvalidPrivateKey"));
    }

    #[test]
    fn error_is_clone() {
        let err = CryptoError::InvalidSignature;
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn error_is_eq() {
        assert_eq!(CryptoError::InvalidPrivateKey, CryptoError::InvalidPrivateKey);
        assert_ne!(CryptoError::InvalidPrivateKey, CryptoError::InvalidSignature);
    }
}

// =============================================================================
// PrivateKey tests
// =============================================================================

mod private_key {
    use super::*;

    #[test]
    fn random_generates_valid_key() {
        let key = PrivateKey::random();
        // Should be able to derive public key
        let _pubkey = key.public_key();
    }

    #[test]
    fn random_generates_different_keys() {
        let key1 = PrivateKey::random();
        let key2 = PrivateKey::random();
        // Extremely unlikely to be the same
        assert_ne!(key1.to_bytes(), key2.to_bytes());
    }

    #[test]
    fn from_bytes_valid_key() {
        // A known valid secp256k1 private key (any 32-byte value < curve order is valid)
        // This is a simple test key - DO NOT use in production
        let bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let result = PrivateKey::from_bytes(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn from_bytes_zero_is_invalid() {
        // Zero is not a valid private key
        let bytes = [0u8; 32];
        let result = PrivateKey::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CryptoError::InvalidPrivateKey);
    }

    #[test]
    fn from_bytes_order_is_invalid() {
        // The secp256k1 curve order (n) is:
        // 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
        // Values >= n are invalid
        let bytes: [u8; 32] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b,
            0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
        ];
        let result = PrivateKey::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CryptoError::InvalidPrivateKey);
    }

    #[test]
    fn from_bytes_above_order_is_invalid() {
        // Value greater than curve order
        let bytes: [u8; 32] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        let result = PrivateKey::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CryptoError::InvalidPrivateKey);
    }

    #[test]
    fn to_bytes_roundtrip() {
        let original_bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let key = PrivateKey::from_bytes(&original_bytes).unwrap();
        let exported = key.to_bytes();
        assert_eq!(original_bytes, exported);
    }

    #[test]
    fn to_bytes_random_roundtrip() {
        let key = PrivateKey::random();
        let bytes = key.to_bytes();
        let restored = PrivateKey::from_bytes(&bytes).unwrap();
        assert_eq!(key.to_bytes(), restored.to_bytes());
    }

    #[test]
    fn public_key_derivation() {
        let key = PrivateKey::random();
        let pubkey = key.public_key();
        // Public key should be 64 bytes
        assert_eq!(pubkey.to_bytes().len(), 64);
    }

    #[test]
    fn public_key_deterministic() {
        let bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let key1 = PrivateKey::from_bytes(&bytes).unwrap();
        let key2 = PrivateKey::from_bytes(&bytes).unwrap();
        assert_eq!(key1.public_key().to_bytes(), key2.public_key().to_bytes());
    }

    #[test]
    fn sign_produces_signature() {
        let key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = key.sign(&message);
        // Signature should be 65 bytes
        assert_eq!(signature.to_bytes().len(), SIGNATURE_LENGTH);
    }

    #[test]
    fn sign_deterministic_with_rfc6979() {
        // RFC6979 makes ECDSA deterministic
        let bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let key = PrivateKey::from_bytes(&bytes).unwrap();
        let message = H256::from_hex("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();

        let sig1 = key.sign(&message);
        let sig2 = key.sign(&message);
        assert_eq!(sig1.to_bytes(), sig2.to_bytes());
    }

    #[test]
    fn sign_different_messages_different_signatures() {
        let key = PrivateKey::random();
        let msg1 = H256::from_hex("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
        let msg2 = H256::from_hex("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();

        let sig1 = key.sign(&msg1);
        let sig2 = key.sign(&msg2);
        assert_ne!(sig1.to_bytes(), sig2.to_bytes());
    }

    #[test]
    fn debug_does_not_reveal_key() {
        let key = PrivateKey::random();
        let debug = format!("{:?}", key);
        // Should contain "REDACTED" or similar, not the actual key bytes
        assert!(debug.contains("REDACTED") || !debug.contains(&format!("{:02x}", key.to_bytes()[0])));
    }
}

// =============================================================================
// PublicKey tests
// =============================================================================

mod public_key {
    use super::*;

    #[test]
    fn from_bytes_valid_point() {
        // Generate a valid public key from a private key
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let bytes = pub_key.to_bytes();

        // Should be able to recreate from bytes
        let restored = PublicKey::from_bytes(&bytes);
        assert!(restored.is_ok());
        assert_eq!(restored.unwrap().to_bytes(), bytes);
    }

    #[test]
    fn from_bytes_invalid_point_zeros() {
        // All zeros is not a valid point on the curve
        let bytes = [0u8; 64];
        let result = PublicKey::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CryptoError::InvalidPublicKey);
    }

    #[test]
    fn from_bytes_invalid_point_random() {
        // Random bytes are extremely unlikely to be a valid point
        let bytes = [0xffu8; 64];
        let result = PublicKey::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CryptoError::InvalidPublicKey);
    }

    #[test]
    fn to_bytes_length() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        assert_eq!(pub_key.to_bytes().len(), 64);
    }

    #[test]
    fn to_bytes_roundtrip() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let bytes = pub_key.to_bytes();
        let restored = PublicKey::from_bytes(&bytes).unwrap();
        assert_eq!(pub_key.to_bytes(), restored.to_bytes());
    }

    #[test]
    fn to_address_format() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let address = pub_key.to_address();
        // Address should be 20 bytes
        assert_eq!(address.as_bytes().len(), 20);
    }

    #[test]
    fn to_address_is_last_20_bytes_of_keccak() {
        // Address = keccak256(public_key)[12..32]
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let pub_bytes = pub_key.to_bytes();

        let hash = keccak256(&pub_bytes);
        let expected_address_bytes = &hash.as_bytes()[12..32];

        let address = pub_key.to_address();
        assert_eq!(address.as_bytes(), expected_address_bytes);
    }

    #[test]
    fn to_address_deterministic() {
        let bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let key = PrivateKey::from_bytes(&bytes).unwrap();
        let pub_key = key.public_key();

        let addr1 = pub_key.to_address();
        let addr2 = pub_key.to_address();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn verify_valid_signature() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();

        let signature = priv_key.sign(&message);
        assert!(pub_key.verify(&signature, &message));
    }

    #[test]
    fn verify_wrong_message() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let message1 = H256::from_hex("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
        let message2 = H256::from_hex("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();

        let signature = priv_key.sign(&message1);
        assert!(!pub_key.verify(&signature, &message2));
    }

    #[test]
    fn verify_wrong_key() {
        let priv_key1 = PrivateKey::random();
        let priv_key2 = PrivateKey::random();
        let pub_key2 = priv_key2.public_key();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();

        let signature = priv_key1.sign(&message);
        assert!(!pub_key2.verify(&signature, &message));
    }

    #[test]
    fn debug_is_implemented() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let debug = format!("{:?}", pub_key);
        assert!(!debug.is_empty());
    }

    #[test]
    fn clone_works() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let cloned = pub_key.clone();
        assert_eq!(pub_key, cloned);
    }

    #[test]
    fn eq_works() {
        let priv_key = PrivateKey::random();
        let pub_key1 = priv_key.public_key();
        let pub_key2 = priv_key.public_key();
        assert_eq!(pub_key1, pub_key2);
    }
}

// =============================================================================
// Signature tests
// =============================================================================

mod signature {
    use super::*;

    #[test]
    fn from_bytes_valid_signature() {
        // Create a valid signature first
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);
        let bytes = signature.to_bytes();

        // Should be able to recreate from bytes
        let restored = Signature::from_bytes(&bytes);
        assert!(restored.is_ok());
    }

    #[test]
    fn from_bytes_all_zeros_invalid() {
        // All zeros is not a valid signature (r and s cannot be 0)
        let bytes = [0u8; SIGNATURE_LENGTH];
        let result = Signature::from_bytes(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CryptoError::InvalidSignature);
    }

    #[test]
    fn to_bytes_length() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);
        assert_eq!(signature.to_bytes().len(), SIGNATURE_LENGTH);
    }

    #[test]
    fn to_bytes_roundtrip() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);
        let bytes = signature.to_bytes();
        let restored = Signature::from_bytes(&bytes).unwrap();
        assert_eq!(signature.to_bytes(), restored.to_bytes());
    }

    #[test]
    fn verify_valid() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();

        let signature = priv_key.sign(&message);
        assert!(signature.verify(&pub_key, &message));
    }

    #[test]
    fn verify_invalid_message() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let message1 = H256::from_hex("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
        let message2 = H256::from_hex("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();

        let signature = priv_key.sign(&message1);
        assert!(!signature.verify(&pub_key, &message2));
    }

    #[test]
    fn verify_invalid_pubkey() {
        let priv_key1 = PrivateKey::random();
        let priv_key2 = PrivateKey::random();
        let pub_key2 = priv_key2.public_key();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();

        let signature = priv_key1.sign(&message);
        assert!(!signature.verify(&pub_key2, &message));
    }

    #[test]
    fn recover_returns_correct_pubkey() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();

        let signature = priv_key.sign(&message);
        let recovered = signature.recover(&message).unwrap();

        assert_eq!(recovered.to_bytes(), pub_key.to_bytes());
    }

    #[test]
    fn recover_fails_with_wrong_message() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let message1 = H256::from_hex("0x1111111111111111111111111111111111111111111111111111111111111111").unwrap();
        let message2 = H256::from_hex("0x2222222222222222222222222222222222222222222222222222222222222222").unwrap();

        let signature = priv_key.sign(&message1);
        let recovered = signature.recover(&message2);

        // Recovery might succeed but give wrong key, or might fail
        if let Ok(wrong_key) = recovered {
            assert_ne!(wrong_key.to_bytes(), pub_key.to_bytes());
        }
    }

    #[test]
    fn r_component() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);

        let r = signature.r();
        assert_eq!(r.len(), 32);

        // r should match first 32 bytes of signature
        let bytes = signature.to_bytes();
        assert_eq!(r, &bytes[0..32]);
    }

    #[test]
    fn s_component() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);

        let s = signature.s();
        assert_eq!(s.len(), 32);

        // s should match bytes 32-64 of signature
        let bytes = signature.to_bytes();
        assert_eq!(s, &bytes[32..64]);
    }

    #[test]
    fn v_component() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);

        let v = signature.v();
        // Ethereum-style v is either 27 or 28
        assert!(v == 27 || v == 28, "v should be 27 or 28, got {}", v);
    }

    #[test]
    fn v_matches_last_byte() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);

        let v = signature.v();
        let bytes = signature.to_bytes();
        assert_eq!(v, bytes[64]);
    }

    #[test]
    fn r_s_v_compose_full_signature() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);

        let mut composed = [0u8; SIGNATURE_LENGTH];
        composed[0..32].copy_from_slice(signature.r());
        composed[32..64].copy_from_slice(signature.s());
        composed[64] = signature.v();

        assert_eq!(composed, signature.to_bytes());
    }

    #[test]
    fn debug_is_implemented() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);
        let debug = format!("{:?}", signature);
        assert!(!debug.is_empty());
    }

    #[test]
    fn clone_works() {
        let priv_key = PrivateKey::random();
        let message = H256::from_hex("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let signature = priv_key.sign(&message);
        let cloned = signature.clone();
        assert_eq!(signature, cloned);
    }

    #[test]
    fn eq_works() {
        let bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let priv_key = PrivateKey::from_bytes(&bytes).unwrap();
        let message = H256::from_hex("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();

        let sig1 = priv_key.sign(&message);
        let sig2 = priv_key.sign(&message);
        assert_eq!(sig1, sig2);
    }
}

// =============================================================================
// Integration tests
// =============================================================================

mod integration {
    use super::*;

    #[test]
    fn full_sign_verify_recover_cycle() {
        // 1. Generate key pair
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();

        // 2. Create message hash
        let message_data = b"hello blockchain world";
        let message_hash = keccak256(message_data);

        // 3. Sign
        let signature = priv_key.sign(&message_hash);

        // 4. Verify with public key
        assert!(pub_key.verify(&signature, &message_hash));
        assert!(signature.verify(&pub_key, &message_hash));

        // 5. Recover public key from signature
        let recovered = signature.recover(&message_hash).unwrap();
        assert_eq!(recovered.to_bytes(), pub_key.to_bytes());

        // 6. Verify address matches
        assert_eq!(recovered.to_address(), pub_key.to_address());
    }

    #[test]
    fn ethereum_style_transaction_signing() {
        // Simulate signing a transaction hash
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();
        let address = pub_key.to_address();

        // Create transaction hash (would normally be RLP encoded tx hash)
        let tx_data = b"nonce:1,to:0x...,value:1000000000000000000,data:0x";
        let tx_hash = keccak256(tx_data);

        // Sign
        let signature = priv_key.sign(&tx_hash);

        // Recover sender
        let recovered_pubkey = signature.recover(&tx_hash).unwrap();
        let recovered_address = recovered_pubkey.to_address();

        assert_eq!(address, recovered_address);
    }

    #[test]
    fn multiple_signatures_same_key() {
        let priv_key = PrivateKey::random();
        let pub_key = priv_key.public_key();

        for i in 0..10 {
            let message = keccak256(&[i as u8; 32]);
            let signature = priv_key.sign(&message);

            assert!(pub_key.verify(&signature, &message));
            let recovered = signature.recover(&message).unwrap();
            assert_eq!(recovered.to_bytes(), pub_key.to_bytes());
        }
    }

    #[test]
    fn known_test_vector() {
        // Private key: 0x0000...0001
        let priv_bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let priv_key = PrivateKey::from_bytes(&priv_bytes).unwrap();
        let pub_key = priv_key.public_key();

        // The public key for private key = 1 is the generator point G
        // X = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
        // Y = 0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8
        let pub_bytes = pub_key.to_bytes();

        // Check first byte of X coordinate
        assert_eq!(pub_bytes[0], 0x79);
        // Check first byte of Y coordinate
        assert_eq!(pub_bytes[32], 0x48);

        // Known address for private key 1
        // 0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf
        let address = pub_key.to_address();
        let addr_hex = format!("{}", address);
        assert!(addr_hex.to_lowercase().contains("7e5f4552091a69125d5dfcb7b8c2659029395bdf"));
    }
}

// =============================================================================
// Thread safety tests
// =============================================================================

mod thread_safety {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn private_key_is_send() {
        assert_send::<PrivateKey>();
    }

    #[test]
    fn private_key_is_sync() {
        assert_sync::<PrivateKey>();
    }

    #[test]
    fn public_key_is_send() {
        assert_send::<PublicKey>();
    }

    #[test]
    fn public_key_is_sync() {
        assert_sync::<PublicKey>();
    }

    #[test]
    fn signature_is_send() {
        assert_send::<Signature>();
    }

    #[test]
    fn signature_is_sync() {
        assert_sync::<Signature>();
    }

    #[test]
    fn concurrent_signing() {
        use std::thread;

        let priv_bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];

        let handles: Vec<_> = (0..4).map(|_| {
            let bytes = priv_bytes;
            thread::spawn(move || {
                let key = PrivateKey::from_bytes(&bytes).unwrap();
                let message = H256::from_hex("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
                key.sign(&message)
            })
        }).collect();

        let signatures: Vec<Signature> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All signatures should be identical (deterministic signing)
        for sig in &signatures {
            assert_eq!(sig.to_bytes(), signatures[0].to_bytes());
        }
    }
}

// =============================================================================
// SIGNATURE_LENGTH constant test
// =============================================================================

mod constants {
    use super::*;

    #[test]
    fn signature_length_is_65() {
        assert_eq!(SIGNATURE_LENGTH, 65);
    }
}
