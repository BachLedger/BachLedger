//! Integration tests for bach-storage

use bach_crypto::{keccak256, PrivateKey};
use bach_primitives::{Address, H256, U256};
use bach_storage::{
    Account, BlockHeader, GenesisAccount, GenesisConfig, Log, LogFilter, Storage, StorageError,
    TransactionReceipt, ValidatorConfig,
};
use bach_types::{Block, Transaction};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_temp_storage() -> (Storage, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::open(temp_dir.path()).unwrap();
    (storage, temp_dir)
}

fn create_test_block(height: u64, parent_hash: H256) -> Block {
    Block::new(height, parent_hash, Vec::new(), 1000 + height)
}

fn create_signed_transaction(nonce: u64, to: Option<Address>, value: U256) -> Transaction {
    let private_key = PrivateKey::random();
    let data = vec![];

    // Create unsigned transaction data for signing
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
// Block Store Tests
// =============================================================================

#[test]
fn test_block_store_put_and_get_by_hash() {
    let (storage, _temp) = create_temp_storage();

    let block = create_test_block(1, H256::zero());
    let hash = block.hash();

    storage.blocks.put_block(&block).unwrap();

    let retrieved = storage.blocks.get_block_by_hash(&hash).unwrap();
    assert_eq!(retrieved.height, 1);
    assert_eq!(retrieved.parent_hash, H256::zero());
    assert_eq!(retrieved.timestamp, 1001);
}

#[test]
fn test_block_store_put_and_get_by_height() {
    let (storage, _temp) = create_temp_storage();

    let block = create_test_block(5, H256::zero());
    storage.blocks.put_block(&block).unwrap();

    let retrieved = storage.blocks.get_block_by_height(5).unwrap();
    assert_eq!(retrieved.height, 5);
}

#[test]
fn test_block_store_get_latest_block() {
    let (storage, _temp) = create_temp_storage();

    // No blocks yet
    assert!(storage.blocks.get_latest_block().is_none());

    // Add blocks in order
    let block1 = create_test_block(0, H256::zero());
    storage.blocks.put_block(&block1).unwrap();

    let block2 = create_test_block(1, block1.hash());
    storage.blocks.put_block(&block2).unwrap();

    let block3 = create_test_block(2, block2.hash());
    storage.blocks.put_block(&block3).unwrap();

    let latest = storage.blocks.get_latest_block().unwrap();
    assert_eq!(latest.height, 2);
}

#[test]
fn test_block_store_height_tracking() {
    let (storage, _temp) = create_temp_storage();

    assert_eq!(storage.blocks.get_block_height(), 0);

    let block1 = create_test_block(0, H256::zero());
    storage.blocks.put_block(&block1).unwrap();
    // Height 0 is stored, get_block_height returns 0

    let block2 = create_test_block(1, block1.hash());
    storage.blocks.put_block(&block2).unwrap();
    assert_eq!(storage.blocks.get_block_height(), 1);

    let block3 = create_test_block(10, block2.hash());
    storage.blocks.put_block(&block3).unwrap();
    assert_eq!(storage.blocks.get_block_height(), 10);
}

#[test]
fn test_block_store_nonexistent_block() {
    let (storage, _temp) = create_temp_storage();

    let fake_hash = H256::from([0xab; 32]);
    assert!(storage.blocks.get_block_by_hash(&fake_hash).is_none());
    assert!(storage.blocks.get_block_by_height(999).is_none());
}

#[test]
fn test_block_store_with_transactions() {
    let (storage, _temp) = create_temp_storage();

    let to_addr = Address::from([0x42; 20]);
    let tx = create_signed_transaction(0, Some(to_addr), U256::from_u64(1000));

    let block = Block::new(1, H256::zero(), vec![tx.clone()], 2000);
    let hash = block.hash();

    storage.blocks.put_block(&block).unwrap();

    let retrieved = storage.blocks.get_block_by_hash(&hash).unwrap();
    assert_eq!(retrieved.transactions.len(), 1);
    assert_eq!(retrieved.transactions[0].nonce, tx.nonce);
    assert_eq!(retrieved.transactions[0].to, tx.to);
    assert_eq!(retrieved.transactions[0].value, tx.value);
}

#[test]
fn test_block_header_storage() {
    let (storage, _temp) = create_temp_storage();

    let block = create_test_block(5, H256::zero());
    let hash = block.hash();
    let state_root = H256::from([0xaa; 32]);

    let header = BlockHeader::from_block(&block, state_root);
    storage.blocks.put_block_header(&hash, &header).unwrap();

    let retrieved = storage.blocks.get_block_header(&hash).unwrap();
    assert_eq!(retrieved.height, 5);
    assert_eq!(retrieved.state_root, *state_root.as_bytes());
}

// =============================================================================
// State Store Tests
// =============================================================================

#[test]
fn test_state_store_account_crud() {
    let (storage, _temp) = create_temp_storage();

    let address = Address::from([0x11; 20]);

    // Account doesn't exist yet
    assert!(storage.state.get_account(&address).is_none());

    // Create and store account
    let mut account = Account::new();
    account.nonce = 5;
    account.set_balance(U256::from_u64(1_000_000));

    storage.state.put_account(&address, &account).unwrap();

    // Retrieve and verify
    let retrieved = storage.state.get_account(&address).unwrap();
    assert_eq!(retrieved.nonce, 5);
    assert_eq!(retrieved.balance_u256(), U256::from_u64(1_000_000));

    // Update account
    let mut updated = retrieved;
    updated.nonce = 10;
    storage.state.put_account(&address, &updated).unwrap();

    let retrieved2 = storage.state.get_account(&address).unwrap();
    assert_eq!(retrieved2.nonce, 10);
}

#[test]
fn test_state_store_storage_crud() {
    let (storage, _temp) = create_temp_storage();

    let address = Address::from([0x22; 20]);
    let key = H256::from([0x33; 32]);
    let value = H256::from([0x44; 32]);

    // Storage slot doesn't exist - returns zero
    let initial = storage.state.get_storage(&address, &key);
    assert!(initial.is_zero());

    // Set storage value
    storage.state.put_storage(&address, &key, value).unwrap();

    // Retrieve and verify
    let retrieved = storage.state.get_storage(&address, &key);
    assert_eq!(retrieved, value);

    // Set to zero removes the value
    storage.state.put_storage(&address, &key, H256::zero()).unwrap();
    let after_clear = storage.state.get_storage(&address, &key);
    assert!(after_clear.is_zero());
}

#[test]
fn test_state_store_multiple_storage_slots() {
    let (storage, _temp) = create_temp_storage();

    let address = Address::from([0x55; 20]);

    // Set multiple storage slots
    for i in 0..10u8 {
        let key = H256::from([i; 32]);
        let value = H256::from([i + 100; 32]);
        storage.state.put_storage(&address, &key, value).unwrap();
    }

    // Verify all slots
    for i in 0..10u8 {
        let key = H256::from([i; 32]);
        let expected = H256::from([i + 100; 32]);
        let actual = storage.state.get_storage(&address, &key);
        assert_eq!(actual, expected);
    }
}

#[test]
fn test_state_store_code_storage() {
    let (storage, _temp) = create_temp_storage();

    let code = vec![0x60, 0x00, 0x60, 0x00, 0xf3]; // Simple bytecode

    // Store code
    let code_hash = storage.state.put_code(&code).unwrap();
    assert!(!code_hash.is_zero());

    // Retrieve code
    let retrieved = storage.state.get_code(&code_hash).unwrap();
    assert_eq!(retrieved, code);

    // Empty code returns empty vec
    let empty_hash = keccak256(&[]);
    let empty_code = storage.state.get_code(&empty_hash).unwrap();
    assert!(empty_code.is_empty());
}

#[test]
fn test_state_store_compute_state_root() {
    let (storage, _temp) = create_temp_storage();

    // Empty state has zero root
    let empty_root = storage.state.compute_state_root();
    assert!(empty_root.is_zero());

    // Add some accounts
    let addr1 = Address::from([0x11; 20]);
    let addr2 = Address::from([0x22; 20]);

    let mut account1 = Account::new();
    account1.set_balance(U256::from_u64(100));
    storage.state.put_account(&addr1, &account1).unwrap();

    let mut account2 = Account::new();
    account2.set_balance(U256::from_u64(200));
    storage.state.put_account(&addr2, &account2).unwrap();

    // State root should now be non-zero
    let root = storage.state.compute_state_root();
    assert!(!root.is_zero());

    // Modifying state should change root
    account1.nonce = 1;
    storage.state.put_account(&addr1, &account1).unwrap();
    let new_root = storage.state.compute_state_root();
    assert_ne!(root, new_root);
}

// =============================================================================
// Transaction Store Tests
// =============================================================================

#[test]
fn test_transaction_store_receipt_crud() {
    let (storage, _temp) = create_temp_storage();

    let tx_hash = H256::from([0xaa; 32]);

    // Receipt doesn't exist yet
    assert!(storage.transactions.get_receipt(&tx_hash).is_none());

    // Create and store receipt
    let receipt = TransactionReceipt {
        transaction_hash: *tx_hash.as_bytes(),
        block_hash: [0xbb; 32],
        block_number: 100,
        transaction_index: 0,
        gas_used: 21000,
        status: true,
        logs: vec![],
    };

    storage.transactions.put_receipt(&receipt).unwrap();

    // Retrieve and verify
    let retrieved = storage.transactions.get_receipt(&tx_hash).unwrap();
    assert_eq!(retrieved.block_number, 100);
    assert_eq!(retrieved.gas_used, 21000);
    assert!(retrieved.status);
}

#[test]
fn test_transaction_store_tx_location() {
    let (storage, _temp) = create_temp_storage();

    let tx_hash = H256::from([0xcc; 32]);
    let block_hash = H256::from([0xdd; 32]);

    let receipt = TransactionReceipt {
        transaction_hash: *tx_hash.as_bytes(),
        block_hash: *block_hash.as_bytes(),
        block_number: 50,
        transaction_index: 3,
        gas_used: 50000,
        status: true,
        logs: vec![],
    };

    storage.transactions.put_receipt(&receipt).unwrap();

    let (retrieved_block_hash, retrieved_tx_index) =
        storage.transactions.get_tx_location(&tx_hash).unwrap();
    assert_eq!(retrieved_block_hash, block_hash);
    assert_eq!(retrieved_tx_index, 3);
}

#[test]
fn test_transaction_store_logs() {
    let (storage, _temp) = create_temp_storage();

    let contract_addr = Address::from([0x11; 20]);
    let topic1 = H256::from([0x22; 32]);
    let topic2 = H256::from([0x33; 32]);

    // Create receipt with logs
    let receipt = TransactionReceipt {
        transaction_hash: [0xaa; 32],
        block_hash: [0xbb; 32],
        block_number: 100,
        transaction_index: 0,
        gas_used: 50000,
        status: true,
        logs: vec![
            Log {
                address: *contract_addr.as_bytes(),
                topics: vec![*topic1.as_bytes()],
                data: vec![1, 2, 3],
                block_number: 100,
                transaction_hash: [0xaa; 32],
                transaction_index: 0,
                log_index: 0,
            },
            Log {
                address: *contract_addr.as_bytes(),
                topics: vec![*topic2.as_bytes()],
                data: vec![4, 5, 6],
                block_number: 100,
                transaction_hash: [0xaa; 32],
                transaction_index: 0,
                log_index: 1,
            },
        ],
    };

    storage.transactions.put_receipt(&receipt).unwrap();

    // Query all logs
    let filter = LogFilter {
        from_block: Some(99),
        to_block: Some(101),
        address: None,
        topics: vec![],
    };
    let logs = storage.transactions.get_logs(&filter);
    assert_eq!(logs.len(), 2);

    // Query by address
    let filter_addr = LogFilter {
        from_block: Some(99),
        to_block: Some(101),
        address: Some(contract_addr),
        topics: vec![],
    };
    let logs_addr = storage.transactions.get_logs(&filter_addr);
    assert_eq!(logs_addr.len(), 2);

    // Query by topic
    let filter_topic = LogFilter {
        from_block: Some(99),
        to_block: Some(101),
        address: None,
        topics: vec![Some(topic1)],
    };
    let logs_topic = storage.transactions.get_logs(&filter_topic);
    assert_eq!(logs_topic.len(), 1);
    assert_eq!(logs_topic[0].topics[0], *topic1.as_bytes());
}

#[test]
fn test_transaction_store_log_filter_block_range() {
    let (storage, _temp) = create_temp_storage();

    // Create receipts in different blocks
    for block_num in [50, 100, 150u64] {
        let receipt = TransactionReceipt {
            transaction_hash: [block_num as u8; 32],
            block_hash: [block_num as u8; 32],
            block_number: block_num,
            transaction_index: 0,
            gas_used: 21000,
            status: true,
            logs: vec![Log {
                address: [0x11; 20],
                topics: vec![],
                data: vec![],
                block_number: block_num,
                transaction_hash: [block_num as u8; 32],
                transaction_index: 0,
                log_index: 0,
            }],
        };
        storage.transactions.put_receipt(&receipt).unwrap();
    }

    // Query middle range
    let filter = LogFilter {
        from_block: Some(75),
        to_block: Some(125),
        address: None,
        topics: vec![],
    };
    let logs = storage.transactions.get_logs(&filter);
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].block_number, 100);
}

// =============================================================================
// Genesis Initialization Tests
// =============================================================================

#[test]
fn test_genesis_initialization() {
    let (mut storage, _temp) = create_temp_storage();

    let addr1 = Address::from([0x11; 20]);
    let addr2 = Address::from([0x22; 20]);

    let mut alloc = HashMap::new();
    alloc.insert(
        addr1,
        GenesisAccount {
            balance: U256::from_u64(1_000_000),
            code: None,
            storage: None,
        },
    );
    alloc.insert(
        addr2,
        GenesisAccount {
            balance: U256::from_u64(2_000_000),
            code: Some(vec![0x60, 0x00]),
            storage: Some({
                let mut s = HashMap::new();
                s.insert(H256::from([0x01; 32]), H256::from([0x02; 32]));
                s
            }),
        },
    );

    let genesis_config = GenesisConfig {
        chain_id: 12345,
        timestamp: 1700000000,
        validators: vec![],
        alloc,
    };

    let genesis_block = storage.init_genesis(&genesis_config).unwrap();

    // Verify genesis block
    assert_eq!(genesis_block.height, 0);
    assert!(genesis_block.parent_hash.is_zero());
    assert!(genesis_block.transactions.is_empty());
    assert_eq!(genesis_block.timestamp, 1700000000);

    // Verify accounts were created
    let account1 = storage.state.get_account(&addr1).unwrap();
    assert_eq!(account1.balance_u256(), U256::from_u64(1_000_000));

    let account2 = storage.state.get_account(&addr2).unwrap();
    assert_eq!(account2.balance_u256(), U256::from_u64(2_000_000));

    // Verify code was stored
    let code_hash = account2.code_hash_h256();
    let code = storage.state.get_code(&code_hash).unwrap();
    assert_eq!(code, vec![0x60, 0x00]);

    // Verify storage was set
    let storage_value = storage.state.get_storage(&addr2, &H256::from([0x01; 32]));
    assert_eq!(storage_value, H256::from([0x02; 32]));

    // Genesis block should be retrievable
    let retrieved = storage.blocks.get_block_by_height(0).unwrap();
    assert_eq!(retrieved.height, 0);
}

#[test]
fn test_genesis_with_validators() {
    let (mut storage, _temp) = create_temp_storage();

    let validator_addr = Address::from([0x99; 20]);

    let genesis_config = GenesisConfig {
        chain_id: 1,
        timestamp: 0,
        validators: vec![ValidatorConfig {
            address: validator_addr,
            stake: U256::from_u64(32_000_000),
        }],
        alloc: HashMap::new(),
    };

    storage.init_genesis(&genesis_config).unwrap();

    // Validator should have stake as balance
    let validator_account = storage.state.get_account(&validator_addr).unwrap();
    assert_eq!(validator_account.balance_u256(), U256::from_u64(32_000_000));
}

#[test]
fn test_genesis_already_initialized() {
    let (mut storage, _temp) = create_temp_storage();

    let genesis_config = GenesisConfig::default();

    // First init succeeds
    storage.init_genesis(&genesis_config).unwrap();

    // Second init fails
    let result = storage.init_genesis(&genesis_config);
    assert!(matches!(result, Err(StorageError::GenesisAlreadyInitialized)));
}

// =============================================================================
// Persistence Tests
// =============================================================================

#[test]
fn test_persistence_across_restarts() {
    let temp_dir = TempDir::new().unwrap();

    // Create storage, add data, and close
    {
        let storage = Storage::open(temp_dir.path()).unwrap();

        let block = create_test_block(1, H256::zero());
        storage.blocks.put_block(&block).unwrap();

        let address = Address::from([0x42; 20]);
        let mut account = Account::new();
        account.nonce = 100;
        account.set_balance(U256::from_u64(999));
        storage.state.put_account(&address, &account).unwrap();

        let receipt = TransactionReceipt {
            transaction_hash: [0xee; 32],
            block_hash: [0xff; 32],
            block_number: 1,
            transaction_index: 0,
            gas_used: 21000,
            status: true,
            logs: vec![],
        };
        storage.transactions.put_receipt(&receipt).unwrap();

        storage.close().unwrap();
    }

    // Reopen storage and verify data persists
    {
        let storage = Storage::open(temp_dir.path()).unwrap();

        // Block should persist
        let block = storage.blocks.get_block_by_height(1).unwrap();
        assert_eq!(block.height, 1);

        // Account should persist
        let address = Address::from([0x42; 20]);
        let account = storage.state.get_account(&address).unwrap();
        assert_eq!(account.nonce, 100);
        assert_eq!(account.balance_u256(), U256::from_u64(999));

        // Receipt should persist
        let tx_hash = H256::from([0xee; 32]);
        let receipt = storage.transactions.get_receipt(&tx_hash).unwrap();
        assert_eq!(receipt.gas_used, 21000);
    }
}

// =============================================================================
// Concurrent Access Tests
// =============================================================================

#[test]
fn test_concurrent_reads() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(Storage::open(temp_dir.path()).unwrap());

    // Add some test data
    for i in 0..10u64 {
        let block = create_test_block(i, H256::zero());
        storage.blocks.put_block(&block).unwrap();
    }
    storage.flush().unwrap();

    // Spawn multiple reader threads
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let storage_clone = Arc::clone(&storage);
            thread::spawn(move || {
                for i in 0..10u64 {
                    let block = storage_clone.blocks.get_block_by_height(i);
                    assert!(block.is_some());
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_zero_values() {
    let (storage, _temp) = create_temp_storage();

    // Zero balance account
    let address = Address::zero();
    let account = Account::new();
    storage.state.put_account(&address, &account).unwrap();

    let retrieved = storage.state.get_account(&address).unwrap();
    assert!(retrieved.balance_u256().is_zero());
    assert_eq!(retrieved.nonce, 0);
}

#[test]
fn test_max_values() {
    let (storage, _temp) = create_temp_storage();

    let address = Address::from([0xff; 20]);
    let mut account = Account::new();
    account.nonce = u64::MAX;
    account.set_balance(U256::MAX);

    storage.state.put_account(&address, &account).unwrap();

    let retrieved = storage.state.get_account(&address).unwrap();
    assert_eq!(retrieved.nonce, u64::MAX);
    assert_eq!(retrieved.balance_u256(), U256::MAX);
}

#[test]
fn test_large_code() {
    let (storage, _temp) = create_temp_storage();

    // 24KB of bytecode (near Ethereum's contract size limit)
    let large_code: Vec<u8> = (0..24576).map(|i| (i % 256) as u8).collect();

    let code_hash = storage.state.put_code(&large_code).unwrap();
    let retrieved = storage.state.get_code(&code_hash).unwrap();

    assert_eq!(retrieved.len(), 24576);
    assert_eq!(retrieved, large_code);
}

#[test]
fn test_many_storage_slots() {
    let (storage, _temp) = create_temp_storage();

    let address = Address::from([0x42; 20]);

    // Store 1000 slots
    for i in 0..1000u32 {
        let mut key_bytes = [0u8; 32];
        key_bytes[28..32].copy_from_slice(&i.to_be_bytes());
        let key = H256::from(key_bytes);

        let mut value_bytes = [0u8; 32];
        value_bytes[28..32].copy_from_slice(&(i + 1000).to_be_bytes());
        let value = H256::from(value_bytes);

        storage.state.put_storage(&address, &key, value).unwrap();
    }

    storage.flush().unwrap();

    // Verify all slots
    for i in 0..1000u32 {
        let mut key_bytes = [0u8; 32];
        key_bytes[28..32].copy_from_slice(&i.to_be_bytes());
        let key = H256::from(key_bytes);

        let mut expected_bytes = [0u8; 32];
        expected_bytes[28..32].copy_from_slice(&(i + 1000).to_be_bytes());
        let expected = H256::from(expected_bytes);

        let actual = storage.state.get_storage(&address, &key);
        assert_eq!(actual, expected);
    }
}
