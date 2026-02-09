//! Tests for Transaction type
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.
//!
//! Note: These tests require bach-crypto to be implemented for signing.

use bach_types::{Transaction, TypeError};
use bach_primitives::{Address, U256};
use bach_crypto::{PrivateKey, keccak256};

// =============================================================================
// Helper functions
// =============================================================================

/// Creates a signed transaction for testing
fn create_test_transaction(
    nonce: u64,
    to: Option<Address>,
    value: U256,
    data: Vec<u8>,
    private_key: &PrivateKey,
) -> Transaction {
    // Create unsigned transaction data for signing
    // This is a simplified version - real implementation may differ
    let mut signing_data = Vec::new();
    signing_data.extend_from_slice(&nonce.to_be_bytes());
    if let Some(addr) = &to {
        signing_data.extend_from_slice(addr.as_bytes());
    }
    signing_data.extend_from_slice(&value.to_be_bytes());
    signing_data.extend_from_slice(&data);

    let signing_hash = keccak256(&signing_data);
    let signature = private_key.sign(&signing_hash);

    Transaction::new(nonce, to, value, data, signature)
}

// =============================================================================
// new() tests
// =============================================================================

mod new {
    use super::*;

    #[test]
    fn creates_transaction_with_fields() {
        let priv_key = PrivateKey::random();
        let to = Address::from_hex("0xdeadbeef00112233445566778899aabbccddeeff").unwrap();
        let value = U256::from_u64(1000);
        let data = vec![0x12, 0x34];

        let tx = create_test_transaction(1, Some(to), value, data.clone(), &priv_key);

        assert_eq!(tx.nonce, 1);
        assert_eq!(tx.to, Some(to));
        assert_eq!(tx.value, value);
        assert_eq!(tx.data, data);
    }

    #[test]
    fn creates_contract_creation_transaction() {
        let priv_key = PrivateKey::random();
        let value = U256::ZERO;
        let data = vec![0x60, 0x80, 0x60, 0x40]; // Contract bytecode

        let tx = create_test_transaction(0, None, value, data.clone(), &priv_key);

        assert_eq!(tx.to, None);
        assert_eq!(tx.data, data);
    }

    #[test]
    fn zero_nonce() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);
        assert_eq!(tx.nonce, 0);
    }

    #[test]
    fn max_nonce() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(u64::MAX, Some(to), U256::ZERO, vec![], &priv_key);
        assert_eq!(tx.nonce, u64::MAX);
    }

    #[test]
    fn large_value() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let value = U256::MAX;
        let tx = create_test_transaction(0, Some(to), value, vec![], &priv_key);
        assert_eq!(tx.value, U256::MAX);
    }

    #[test]
    fn empty_data() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);
        assert!(tx.data.is_empty());
    }

    #[test]
    fn large_data() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let data = vec![0xffu8; 10000]; // 10KB of data
        let tx = create_test_transaction(0, Some(to), U256::ZERO, data.clone(), &priv_key);
        assert_eq!(tx.data.len(), 10000);
    }
}

// =============================================================================
// hash() tests
// =============================================================================

mod hash {
    use super::*;

    #[test]
    fn returns_32_byte_hash() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        let hash = tx.hash();
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn deterministic() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        let hash1 = tx.hash();
        let hash2 = tx.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn different_nonce_different_hash() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx1 = create_test_transaction(1, Some(to), U256::ZERO, vec![], &priv_key);
        let tx2 = create_test_transaction(2, Some(to), U256::ZERO, vec![], &priv_key);

        assert_ne!(tx1.hash(), tx2.hash());
    }

    #[test]
    fn different_to_different_hash() {
        let priv_key = PrivateKey::random();
        let to1 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();
        let to2 = Address::from_hex("0x0000000000000000000000000000000000000002").unwrap();

        let tx1 = create_test_transaction(0, Some(to1), U256::ZERO, vec![], &priv_key);
        let tx2 = create_test_transaction(0, Some(to2), U256::ZERO, vec![], &priv_key);

        assert_ne!(tx1.hash(), tx2.hash());
    }

    #[test]
    fn different_value_different_hash() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx1 = create_test_transaction(0, Some(to), U256::from_u64(100), vec![], &priv_key);
        let tx2 = create_test_transaction(0, Some(to), U256::from_u64(200), vec![], &priv_key);

        assert_ne!(tx1.hash(), tx2.hash());
    }

    #[test]
    fn different_data_different_hash() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx1 = create_test_transaction(0, Some(to), U256::ZERO, vec![1, 2, 3], &priv_key);
        let tx2 = create_test_transaction(0, Some(to), U256::ZERO, vec![4, 5, 6], &priv_key);

        assert_ne!(tx1.hash(), tx2.hash());
    }

    #[test]
    fn contract_creation_vs_transfer_different_hash() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx1 = create_test_transaction(0, None, U256::ZERO, vec![], &priv_key);
        let tx2 = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        assert_ne!(tx1.hash(), tx2.hash());
    }
}

// =============================================================================
// sender() tests
// =============================================================================

mod sender {
    use super::*;

    #[test]
    fn recovers_correct_address() {
        let priv_key = PrivateKey::random();
        let expected_address = priv_key.public_key().to_address();
        let to = Address::zero();

        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);
        let sender = tx.sender().unwrap();

        assert_eq!(sender, expected_address);
    }

    #[test]
    fn consistent_recovery() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        let sender1 = tx.sender().unwrap();
        let sender2 = tx.sender().unwrap();

        assert_eq!(sender1, sender2);
    }

    #[test]
    fn different_transactions_same_sender() {
        let priv_key = PrivateKey::random();
        let expected_address = priv_key.public_key().to_address();
        let to = Address::zero();

        let tx1 = create_test_transaction(1, Some(to), U256::from_u64(100), vec![1, 2], &priv_key);
        let tx2 = create_test_transaction(2, Some(to), U256::from_u64(200), vec![3, 4], &priv_key);

        assert_eq!(tx1.sender().unwrap(), expected_address);
        assert_eq!(tx2.sender().unwrap(), expected_address);
    }

    #[test]
    fn different_signers_different_senders() {
        let priv_key1 = PrivateKey::random();
        let priv_key2 = PrivateKey::random();
        let to = Address::zero();

        let tx1 = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key1);
        let tx2 = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key2);

        assert_ne!(tx1.sender().unwrap(), tx2.sender().unwrap());
    }

    #[test]
    fn contract_creation_sender() {
        let priv_key = PrivateKey::random();
        let expected_address = priv_key.public_key().to_address();

        let tx = create_test_transaction(0, None, U256::ZERO, vec![0x60, 0x80], &priv_key);
        let sender = tx.sender().unwrap();

        assert_eq!(sender, expected_address);
    }
}

// =============================================================================
// signing_hash() tests
// =============================================================================

mod signing_hash {
    use super::*;

    #[test]
    fn returns_32_byte_hash() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        let hash = tx.signing_hash();
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn deterministic() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        let hash1 = tx.signing_hash();
        let hash2 = tx.signing_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn different_nonce_different_signing_hash() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx1 = create_test_transaction(1, Some(to), U256::ZERO, vec![], &priv_key);
        let tx2 = create_test_transaction(2, Some(to), U256::ZERO, vec![], &priv_key);

        assert_ne!(tx1.signing_hash(), tx2.signing_hash());
    }

    #[test]
    fn signing_hash_different_from_tx_hash() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        // Signing hash and transaction hash should be different
        // (signing hash doesn't include the signature, tx hash does)
        assert_ne!(tx.signing_hash(), tx.hash());
    }
}

// =============================================================================
// TypeError tests
// =============================================================================

mod type_error {
    use super::*;

    #[test]
    fn error_variants_exist() {
        let _ = TypeError::InvalidSignature;
        let _ = TypeError::RecoveryFailed;
        let _ = TypeError::InvalidTransaction("test".to_string());
    }

    #[test]
    fn error_is_debug() {
        let err = TypeError::InvalidSignature;
        let debug = format!("{:?}", err);
        assert!(debug.contains("InvalidSignature"));
    }

    #[test]
    fn error_is_clone() {
        let err = TypeError::RecoveryFailed;
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn error_is_eq() {
        assert_eq!(TypeError::InvalidSignature, TypeError::InvalidSignature);
        assert_ne!(TypeError::InvalidSignature, TypeError::RecoveryFailed);
    }

    #[test]
    fn invalid_transaction_contains_message() {
        let err = TypeError::InvalidTransaction("missing field".to_string());
        if let TypeError::InvalidTransaction(msg) = err {
            assert_eq!(msg, "missing field");
        } else {
            panic!("Expected InvalidTransaction");
        }
    }
}

// =============================================================================
// Derived trait tests
// =============================================================================

mod derived_traits {
    use super::*;

    #[test]
    fn debug_is_implemented() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);
        let debug = format!("{:?}", tx);
        assert!(!debug.is_empty());
    }

    #[test]
    fn clone_works() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);
        let cloned = tx.clone();

        assert_eq!(tx.nonce, cloned.nonce);
        assert_eq!(tx.to, cloned.to);
        assert_eq!(tx.value, cloned.value);
        assert_eq!(tx.data, cloned.data);
    }

    #[test]
    fn eq_works() {
        let priv_bytes: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ];
        let priv_key = PrivateKey::from_bytes(&priv_bytes).unwrap();
        let to = Address::zero();

        let tx1 = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);
        let tx2 = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        // Same inputs should produce same transaction
        assert_eq!(tx1, tx2);
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
    fn transaction_is_send() {
        assert_send::<Transaction>();
    }

    #[test]
    fn transaction_is_sync() {
        assert_sync::<Transaction>();
    }
}
