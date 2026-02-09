//! Tests for Block type
//!
//! Test-driven development: these tests are written BEFORE implementation.
//! All tests should FAIL until implementation is complete.

use bach_types::{Block, Transaction};
use bach_primitives::{Address, H256, U256};
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

/// Creates a test block with transactions
fn create_test_block(
    height: u64,
    parent_hash: H256,
    tx_count: usize,
    timestamp: u64,
) -> Block {
    let priv_key = PrivateKey::random();
    let to = Address::zero();

    let transactions: Vec<Transaction> = (0..tx_count)
        .map(|i| create_test_transaction(i as u64, Some(to), U256::from_u64(i as u64 * 100), vec![], &priv_key))
        .collect();

    Block::new(height, parent_hash, transactions, timestamp)
}

// =============================================================================
// new() tests
// =============================================================================

mod new {
    use super::*;

    #[test]
    fn creates_block_with_fields() {
        let parent_hash = H256::from_hex("0xdeadbeef00112233445566778899aabbccddeeff00112233445566778899aabb").unwrap();
        let block = create_test_block(100, parent_hash, 5, 1234567890);

        assert_eq!(block.height, 100);
        assert_eq!(block.parent_hash, parent_hash);
        assert_eq!(block.transactions.len(), 5);
        assert_eq!(block.timestamp, 1234567890);
    }

    #[test]
    fn genesis_block() {
        let parent_hash = H256::zero();
        let block = Block::new(0, parent_hash, vec![], 0);

        assert_eq!(block.height, 0);
        assert_eq!(block.parent_hash, H256::zero());
        assert!(block.transactions.is_empty());
        assert_eq!(block.timestamp, 0);
    }

    #[test]
    fn empty_transactions() {
        let parent_hash = H256::zero();
        let block = Block::new(1, parent_hash, vec![], 1000);

        assert!(block.transactions.is_empty());
        assert_eq!(block.transaction_count(), 0);
    }

    #[test]
    fn max_height() {
        let parent_hash = H256::zero();
        let block = Block::new(u64::MAX, parent_hash, vec![], 0);

        assert_eq!(block.height, u64::MAX);
    }

    #[test]
    fn max_timestamp() {
        let parent_hash = H256::zero();
        let block = Block::new(0, parent_hash, vec![], u64::MAX);

        assert_eq!(block.timestamp, u64::MAX);
    }

    #[test]
    fn many_transactions() {
        let parent_hash = H256::zero();
        let block = create_test_block(1, parent_hash, 100, 1000);

        assert_eq!(block.transactions.len(), 100);
    }
}

// =============================================================================
// hash() tests
// =============================================================================

mod hash {
    use super::*;

    #[test]
    fn returns_32_byte_hash() {
        let block = create_test_block(1, H256::zero(), 1, 1000);
        let hash = block.hash();
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn deterministic() {
        let block = create_test_block(1, H256::zero(), 1, 1000);
        let hash1 = block.hash();
        let hash2 = block.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn different_height_different_hash() {
        let block1 = create_test_block(1, H256::zero(), 0, 1000);
        let block2 = create_test_block(2, H256::zero(), 0, 1000);
        assert_ne!(block1.hash(), block2.hash());
    }

    #[test]
    fn different_parent_hash_different_hash() {
        let parent1 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let parent2 = H256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002").unwrap();

        let block1 = Block::new(1, parent1, vec![], 1000);
        let block2 = Block::new(1, parent2, vec![], 1000);

        assert_ne!(block1.hash(), block2.hash());
    }

    #[test]
    fn different_timestamp_different_hash() {
        let block1 = create_test_block(1, H256::zero(), 0, 1000);
        let block2 = create_test_block(1, H256::zero(), 0, 2000);
        assert_ne!(block1.hash(), block2.hash());
    }

    #[test]
    fn different_transactions_different_hash() {
        let parent = H256::zero();
        let priv_key = PrivateKey::random();
        let to = Address::zero();

        let tx1 = create_test_transaction(0, Some(to), U256::from_u64(100), vec![], &priv_key);
        let tx2 = create_test_transaction(0, Some(to), U256::from_u64(200), vec![], &priv_key);

        let block1 = Block::new(1, parent, vec![tx1], 1000);
        let block2 = Block::new(1, parent, vec![tx2], 1000);

        assert_ne!(block1.hash(), block2.hash());
    }

    #[test]
    fn empty_vs_nonempty_transactions_different_hash() {
        let parent = H256::zero();
        let block1 = Block::new(1, parent, vec![], 1000);
        let block2 = create_test_block(1, parent, 1, 1000);

        assert_ne!(block1.hash(), block2.hash());
    }

    #[test]
    fn genesis_block_hash() {
        let genesis = Block::new(0, H256::zero(), vec![], 0);
        let hash = genesis.hash();
        assert!(!hash.is_zero()); // Genesis block should have a non-zero hash
    }
}

// =============================================================================
// transactions_hash() tests
// =============================================================================

mod transactions_hash {
    use super::*;

    #[test]
    fn returns_32_byte_hash() {
        let block = create_test_block(1, H256::zero(), 3, 1000);
        let hash = block.transactions_hash();
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn deterministic() {
        let block = create_test_block(1, H256::zero(), 3, 1000);
        let hash1 = block.transactions_hash();
        let hash2 = block.transactions_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn empty_transactions_hash() {
        let block = Block::new(1, H256::zero(), vec![], 1000);
        let hash = block.transactions_hash();
        // Should return some consistent value for empty transactions
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn different_transactions_different_hash() {
        let parent = H256::zero();
        let priv_key = PrivateKey::random();
        let to = Address::zero();

        let tx1 = create_test_transaction(0, Some(to), U256::from_u64(100), vec![], &priv_key);
        let tx2 = create_test_transaction(0, Some(to), U256::from_u64(200), vec![], &priv_key);

        let block1 = Block::new(1, parent, vec![tx1], 1000);
        let block2 = Block::new(1, parent, vec![tx2], 1000);

        assert_ne!(block1.transactions_hash(), block2.transactions_hash());
    }

    #[test]
    fn order_matters() {
        let parent = H256::zero();
        let priv_key = PrivateKey::random();
        let to = Address::zero();

        let tx1 = create_test_transaction(1, Some(to), U256::from_u64(100), vec![], &priv_key);
        let tx2 = create_test_transaction(2, Some(to), U256::from_u64(200), vec![], &priv_key);

        let block1 = Block::new(1, parent, vec![tx1.clone(), tx2.clone()], 1000);
        let block2 = Block::new(1, parent, vec![tx2, tx1], 1000);

        assert_ne!(block1.transactions_hash(), block2.transactions_hash());
    }

    #[test]
    fn independent_of_block_metadata() {
        let priv_key = PrivateKey::random();
        let to = Address::zero();
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        // Same transaction in blocks with different heights/timestamps
        let block1 = Block::new(1, H256::zero(), vec![tx.clone()], 1000);
        let block2 = Block::new(2, H256::zero(), vec![tx], 2000);

        // transactions_hash should be the same since transactions are identical
        assert_eq!(block1.transactions_hash(), block2.transactions_hash());
    }
}

// =============================================================================
// transaction_count() tests
// =============================================================================

mod transaction_count {
    use super::*;

    #[test]
    fn zero_for_empty_block() {
        let block = Block::new(1, H256::zero(), vec![], 1000);
        assert_eq!(block.transaction_count(), 0);
    }

    #[test]
    fn correct_for_single_transaction() {
        let block = create_test_block(1, H256::zero(), 1, 1000);
        assert_eq!(block.transaction_count(), 1);
    }

    #[test]
    fn correct_for_multiple_transactions() {
        let block = create_test_block(1, H256::zero(), 10, 1000);
        assert_eq!(block.transaction_count(), 10);
    }

    #[test]
    fn matches_transactions_len() {
        let block = create_test_block(1, H256::zero(), 50, 1000);
        assert_eq!(block.transaction_count(), block.transactions.len());
    }
}

// =============================================================================
// Derived trait tests
// =============================================================================

mod derived_traits {
    use super::*;

    #[test]
    fn debug_is_implemented() {
        let block = create_test_block(1, H256::zero(), 1, 1000);
        let debug = format!("{:?}", block);
        assert!(!debug.is_empty());
    }

    #[test]
    fn clone_works() {
        let block = create_test_block(1, H256::zero(), 2, 1000);
        let cloned = block.clone();

        assert_eq!(block.height, cloned.height);
        assert_eq!(block.parent_hash, cloned.parent_hash);
        assert_eq!(block.timestamp, cloned.timestamp);
        assert_eq!(block.transactions.len(), cloned.transactions.len());
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
        let tx = create_test_transaction(0, Some(to), U256::ZERO, vec![], &priv_key);

        let block1 = Block::new(1, H256::zero(), vec![tx.clone()], 1000);
        let block2 = Block::new(1, H256::zero(), vec![tx], 1000);

        assert_eq!(block1, block2);
    }

    #[test]
    fn not_equal_different_height() {
        let block1 = Block::new(1, H256::zero(), vec![], 1000);
        let block2 = Block::new(2, H256::zero(), vec![], 1000);

        assert_ne!(block1, block2);
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
    fn block_is_send() {
        assert_send::<Block>();
    }

    #[test]
    fn block_is_sync() {
        assert_sync::<Block>();
    }
}

// =============================================================================
// Integration tests
// =============================================================================

mod integration {
    use super::*;

    #[test]
    fn chain_of_blocks() {
        let genesis = Block::new(0, H256::zero(), vec![], 0);
        let genesis_hash = genesis.hash();

        let block1 = create_test_block(1, genesis_hash, 2, 1000);
        let block1_hash = block1.hash();

        let block2 = create_test_block(2, block1_hash, 3, 2000);

        assert_eq!(block1.parent_hash, genesis_hash);
        assert_eq!(block2.parent_hash, block1_hash);
    }

    #[test]
    fn block_hash_includes_transactions() {
        let parent = H256::zero();
        let priv_key = PrivateKey::random();
        let to = Address::zero();

        // Block with transaction
        let tx = create_test_transaction(0, Some(to), U256::from_u64(1000), vec![], &priv_key);
        let block_with_tx = Block::new(1, parent, vec![tx], 1000);

        // Empty block
        let empty_block = Block::new(1, parent, vec![], 1000);

        // Block hash should differ based on transactions
        assert_ne!(block_with_tx.hash(), empty_block.hash());
    }
}
