//! BachLedger Storage
//!
//! Persistent storage layer for the medical blockchain:
//! - `BlockStore`: Block storage by hash and height
//! - `StateStore`: Account state and contract storage
//! - `TransactionStore`: Transaction receipts and logs
//! - `Storage`: Unified storage interface

#![forbid(unsafe_code)]

use bach_crypto::{keccak256, Signature};
use bach_primitives::{Address, H256, U256};
use bach_types::{Block, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Storage operation errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Sled error: {0}")]
    SledError(#[from] sled::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Corrupted data: {0}")]
    CorruptedData(String),

    #[error("Genesis already initialized")]
    GenesisAlreadyInitialized,
}

impl From<bincode::Error> for StorageError {
    fn from(e: bincode::Error) -> Self {
        StorageError::SerializationError(e.to_string())
    }
}

// =============================================================================
// Serializable Types
// =============================================================================

/// Serializable account data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub nonce: u64,
    pub balance: [u8; 32], // U256 as bytes
    pub storage_root: [u8; 32],
    pub code_hash: [u8; 32],
}

impl Account {
    pub fn new() -> Self {
        Self {
            nonce: 0,
            balance: [0u8; 32],
            storage_root: [0u8; 32],
            code_hash: keccak256(&[]).as_bytes().to_owned(),
        }
    }

    pub fn balance_u256(&self) -> U256 {
        U256::from_be_bytes(self.balance)
    }

    pub fn set_balance(&mut self, balance: U256) {
        self.balance = balance.to_be_bytes();
    }

    pub fn storage_root_h256(&self) -> H256 {
        H256::from(self.storage_root)
    }

    pub fn code_hash_h256(&self) -> H256 {
        H256::from(self.code_hash)
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionReceipt {
    pub transaction_hash: [u8; 32],
    pub block_hash: [u8; 32],
    pub block_number: u64,
    pub transaction_index: u32,
    pub gas_used: u64,
    pub status: bool,
    pub logs: Vec<Log>,
}

impl TransactionReceipt {
    pub fn transaction_hash_h256(&self) -> H256 {
        H256::from(self.transaction_hash)
    }

    pub fn block_hash_h256(&self) -> H256 {
        H256::from(self.block_hash)
    }
}

/// Event log
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Log {
    pub address: [u8; 20],
    pub topics: Vec<[u8; 32]>,
    pub data: Vec<u8>,
    pub block_number: u64,
    pub transaction_hash: [u8; 32],
    pub transaction_index: u32,
    pub log_index: u32,
}

impl Log {
    pub fn address_addr(&self) -> Address {
        Address::from(self.address)
    }

    pub fn topics_h256(&self) -> Vec<H256> {
        self.topics.iter().map(|t| H256::from(*t)).collect()
    }
}

/// Log filter for querying logs
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
    pub address: Option<Address>,
    pub topics: Vec<Option<H256>>,
}

/// Serializable block for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredBlock {
    height: u64,
    parent_hash: [u8; 32],
    timestamp: u64,
    transactions: Vec<StoredTransaction>,
}

/// Serializable transaction for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredTransaction {
    nonce: u64,
    to: Option<[u8; 20]>,
    value: [u8; 32],
    data: Vec<u8>,
    signature: Vec<u8>, // 65 bytes stored as Vec for serde compatibility
}

impl From<&Block> for StoredBlock {
    fn from(block: &Block) -> Self {
        Self {
            height: block.height,
            parent_hash: *block.parent_hash.as_bytes(),
            timestamp: block.timestamp,
            transactions: block.transactions.iter().map(StoredTransaction::from).collect(),
        }
    }
}

impl StoredBlock {
    fn to_block(&self) -> Result<Block, StorageError> {
        let transactions: Result<Vec<Transaction>, _> = self
            .transactions
            .iter()
            .map(|tx| tx.to_transaction())
            .collect();

        Ok(Block {
            height: self.height,
            parent_hash: H256::from(self.parent_hash),
            timestamp: self.timestamp,
            transactions: transactions?,
        })
    }
}

impl From<&Transaction> for StoredTransaction {
    fn from(tx: &Transaction) -> Self {
        Self {
            nonce: tx.nonce,
            to: tx.to.map(|a| *a.as_bytes()),
            value: tx.value.to_be_bytes(),
            data: tx.data.clone(),
            signature: tx.signature.to_bytes().to_vec(),
        }
    }
}

impl StoredTransaction {
    fn to_transaction(&self) -> Result<Transaction, StorageError> {
        let sig_bytes: [u8; 65] = self.signature.as_slice().try_into()
            .map_err(|_| StorageError::CorruptedData("Invalid signature length in stored transaction".into()))?;
        let signature = Signature::from_bytes(&sig_bytes)
            .map_err(|_| StorageError::CorruptedData("Invalid signature in stored transaction".into()))?;

        Ok(Transaction {
            nonce: self.nonce,
            to: self.to.map(Address::from),
            value: U256::from_be_bytes(self.value),
            data: self.data.clone(),
            signature,
        })
    }
}

/// Serializable block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub parent_hash: [u8; 32],
    pub timestamp: u64,
    pub transactions_hash: [u8; 32],
    pub state_root: [u8; 32],
}

impl BlockHeader {
    pub fn from_block(block: &Block, state_root: H256) -> Self {
        Self {
            height: block.height,
            parent_hash: *block.parent_hash.as_bytes(),
            timestamp: block.timestamp,
            transactions_hash: *block.transactions_hash().as_bytes(),
            state_root: *state_root.as_bytes(),
        }
    }
}

// =============================================================================
// Genesis Configuration
// =============================================================================

/// Genesis account allocation
#[derive(Debug, Clone)]
pub struct GenesisAccount {
    pub balance: U256,
    pub code: Option<Vec<u8>>,
    pub storage: Option<HashMap<H256, H256>>,
}

/// Validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub address: Address,
    pub stake: U256,
}

/// Genesis block configuration
#[derive(Debug, Clone)]
pub struct GenesisConfig {
    pub chain_id: u64,
    pub timestamp: u64,
    pub validators: Vec<ValidatorConfig>,
    pub alloc: HashMap<Address, GenesisAccount>,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            chain_id: 1,
            timestamp: 0,
            validators: Vec::new(),
            alloc: HashMap::new(),
        }
    }
}

// =============================================================================
// Block Store
// =============================================================================

/// Block storage with indexing by hash and height
pub struct BlockStore {
    db: sled::Db,
    blocks_by_hash: sled::Tree,
    blocks_by_height: sled::Tree,
    block_headers: sled::Tree,
    metadata: sled::Tree,
}

const LATEST_HEIGHT_KEY: &[u8] = b"latest_height";

impl BlockStore {
    /// Opens or creates a block store at the given path
    pub fn new(path: &Path) -> Result<Self, StorageError> {
        let db = sled::open(path.join("blocks"))?;
        let blocks_by_hash = db.open_tree("blocks_by_hash")?;
        let blocks_by_height = db.open_tree("blocks_by_height")?;
        let block_headers = db.open_tree("block_headers")?;
        let metadata = db.open_tree("metadata")?;

        Ok(Self {
            db,
            blocks_by_hash,
            blocks_by_height,
            block_headers,
            metadata,
        })
    }

    /// Stores a block
    pub fn put_block(&self, block: &Block) -> Result<(), StorageError> {
        let hash = block.hash();
        let height = block.height;

        let stored = StoredBlock::from(block);
        let encoded = bincode::serialize(&stored)?;

        // Store block by hash
        self.blocks_by_hash.insert(hash.as_bytes(), encoded)?;

        // Store hash by height
        self.blocks_by_height.insert(height.to_be_bytes(), hash.as_bytes())?;

        // Update latest height if this is higher
        let current_height = self.get_block_height();
        if height > current_height || current_height == 0 {
            self.metadata.insert(LATEST_HEIGHT_KEY, &height.to_be_bytes())?;
        }

        Ok(())
    }

    /// Stores a block header
    pub fn put_block_header(&self, hash: &H256, header: &BlockHeader) -> Result<(), StorageError> {
        let encoded = bincode::serialize(header)?;
        self.block_headers.insert(hash.as_bytes(), encoded)?;
        Ok(())
    }

    /// Retrieves a block by hash
    pub fn get_block_by_hash(&self, hash: &H256) -> Option<Block> {
        let data = self.blocks_by_hash.get(hash.as_bytes()).ok()??;
        let stored: StoredBlock = bincode::deserialize(&data).ok()?;
        stored.to_block().ok()
    }

    /// Retrieves a block by height
    pub fn get_block_by_height(&self, height: u64) -> Option<Block> {
        let hash_bytes = self.blocks_by_height.get(height.to_be_bytes()).ok()??;
        let hash = H256::from_slice(&hash_bytes).ok()?;
        self.get_block_by_hash(&hash)
    }

    /// Retrieves the latest block
    pub fn get_latest_block(&self) -> Option<Block> {
        let height = self.get_block_height();
        if height == 0 {
            // Check if there's a genesis block at height 0
            self.get_block_by_height(0)
        } else {
            self.get_block_by_height(height)
        }
    }

    /// Returns the current block height (0 if no blocks)
    pub fn get_block_height(&self) -> u64 {
        self.metadata
            .get(LATEST_HEIGHT_KEY)
            .ok()
            .flatten()
            .map(|v| {
                let bytes: [u8; 8] = v.as_ref().try_into().unwrap_or([0u8; 8]);
                u64::from_be_bytes(bytes)
            })
            .unwrap_or(0)
    }

    /// Retrieves a block header
    pub fn get_block_header(&self, hash: &H256) -> Option<BlockHeader> {
        let data = self.block_headers.get(hash.as_bytes()).ok()??;
        bincode::deserialize(&data).ok()
    }

    /// Flushes data to disk
    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()?;
        Ok(())
    }
}

// =============================================================================
// State Store
// =============================================================================

/// Account state and contract storage
pub struct StateStore {
    db: sled::Db,
    accounts: sled::Tree,
    storage: sled::Tree,
    code: sled::Tree,
}

impl StateStore {
    /// Opens or creates a state store at the given path
    pub fn new(path: &Path) -> Result<Self, StorageError> {
        let db = sled::open(path.join("state"))?;
        let accounts = db.open_tree("accounts")?;
        let storage = db.open_tree("storage")?;
        let code = db.open_tree("code")?;

        Ok(Self {
            db,
            accounts,
            storage,
            code,
        })
    }

    /// Retrieves an account
    pub fn get_account(&self, address: &Address) -> Option<Account> {
        let data = self.accounts.get(address.as_bytes()).ok()??;
        bincode::deserialize(&data).ok()
    }

    /// Stores an account
    pub fn put_account(&self, address: &Address, account: &Account) -> Result<(), StorageError> {
        let encoded = bincode::serialize(account)?;
        self.accounts.insert(address.as_bytes(), encoded)?;
        Ok(())
    }

    /// Retrieves storage value for an address and key
    pub fn get_storage(&self, address: &Address, key: &H256) -> H256 {
        let storage_key = Self::make_storage_key(address, key);
        self.storage
            .get(storage_key)
            .ok()
            .flatten()
            .and_then(|v| {
                let bytes: [u8; 32] = v.as_ref().try_into().ok()?;
                Some(H256::from(bytes))
            })
            .unwrap_or_else(H256::zero)
    }

    /// Stores a storage value
    pub fn put_storage(&self, address: &Address, key: &H256, value: H256) -> Result<(), StorageError> {
        let storage_key = Self::make_storage_key(address, key);
        if value.is_zero() {
            // Remove zero values to save space
            self.storage.remove(storage_key)?;
        } else {
            self.storage.insert(storage_key, value.as_bytes())?;
        }
        Ok(())
    }

    /// Retrieves contract code by hash
    pub fn get_code(&self, code_hash: &H256) -> Option<Vec<u8>> {
        if code_hash.is_zero() || *code_hash == keccak256(&[]) {
            return Some(Vec::new());
        }
        let data = self.code.get(code_hash.as_bytes()).ok()??;
        Some(data.to_vec())
    }

    /// Stores contract code, returns its hash
    pub fn put_code(&self, code: &[u8]) -> Result<H256, StorageError> {
        let hash = keccak256(code);
        if !code.is_empty() {
            self.code.insert(hash.as_bytes(), code)?;
        }
        Ok(hash)
    }

    /// Computes a simple state root (hash of all account hashes)
    pub fn compute_state_root(&self) -> H256 {
        let mut all_data = Vec::new();

        for (key, value) in self.accounts.iter().flatten() {
            all_data.extend_from_slice(&key);
            all_data.extend_from_slice(&value);
        }

        if all_data.is_empty() {
            return H256::zero();
        }

        keccak256(&all_data)
    }

    /// Creates a composite storage key from address and slot
    fn make_storage_key(address: &Address, key: &H256) -> [u8; 52] {
        let mut storage_key = [0u8; 52];
        storage_key[0..20].copy_from_slice(address.as_bytes());
        storage_key[20..52].copy_from_slice(key.as_bytes());
        storage_key
    }

    /// Flushes data to disk
    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()?;
        Ok(())
    }
}

// =============================================================================
// Transaction Store
// =============================================================================

/// Transaction receipt and log storage
pub struct TransactionStore {
    db: sled::Db,
    tx_locations: sled::Tree,
    receipts: sled::Tree,
    logs_by_block: sled::Tree,
}

impl TransactionStore {
    /// Opens or creates a transaction store at the given path
    pub fn new(path: &Path) -> Result<Self, StorageError> {
        let db = sled::open(path.join("transactions"))?;
        let tx_locations = db.open_tree("tx_locations")?;
        let receipts = db.open_tree("receipts")?;
        let logs_by_block = db.open_tree("logs_by_block")?;

        Ok(Self {
            db,
            tx_locations,
            receipts,
            logs_by_block,
        })
    }

    /// Stores a transaction receipt
    pub fn put_receipt(&self, receipt: &TransactionReceipt) -> Result<(), StorageError> {
        let tx_hash = receipt.transaction_hash;
        let encoded = bincode::serialize(receipt)?;

        // Store receipt
        self.receipts.insert(tx_hash, encoded)?;

        // Store transaction location (block_hash, tx_index)
        let mut location = [0u8; 36];
        location[0..32].copy_from_slice(&receipt.block_hash);
        location[32..36].copy_from_slice(&receipt.transaction_index.to_be_bytes());
        self.tx_locations.insert(tx_hash, location.as_slice())?;

        // Store logs indexed by block
        if !receipt.logs.is_empty() {
            let logs_key = Self::make_logs_key(receipt.block_number, receipt.transaction_index);
            let logs_encoded = bincode::serialize(&receipt.logs)?;
            self.logs_by_block.insert(logs_key, logs_encoded)?;
        }

        Ok(())
    }

    /// Retrieves a transaction receipt
    pub fn get_receipt(&self, tx_hash: &H256) -> Option<TransactionReceipt> {
        let data = self.receipts.get(tx_hash.as_bytes()).ok()??;
        bincode::deserialize(&data).ok()
    }

    /// Gets transaction location (block_hash, tx_index)
    pub fn get_tx_location(&self, tx_hash: &H256) -> Option<(H256, u32)> {
        let data = self.tx_locations.get(tx_hash.as_bytes()).ok()??;
        if data.len() != 36 {
            return None;
        }
        let block_hash = H256::from_slice(&data[0..32]).ok()?;
        let tx_index = u32::from_be_bytes(data[32..36].try_into().ok()?);
        Some((block_hash, tx_index))
    }

    /// Queries logs matching a filter
    pub fn get_logs(&self, filter: &LogFilter) -> Vec<Log> {
        let mut results = Vec::new();

        let from_block = filter.from_block.unwrap_or(0);
        let to_block = filter.to_block.unwrap_or(u64::MAX);

        // Iterate through logs by block range
        let start_key = Self::make_logs_key(from_block, 0);
        let end_key = Self::make_logs_key(to_block.saturating_add(1), 0);

        for (_key, value) in self.logs_by_block.range(start_key..end_key).flatten() {
            if let Ok(logs) = bincode::deserialize::<Vec<Log>>(&value) {
                for log in logs {
                    if Self::log_matches_filter(&log, filter) {
                        results.push(log);
                    }
                }
            }
        }

        results
    }

    /// Checks if a log matches the filter
    fn log_matches_filter(log: &Log, filter: &LogFilter) -> bool {
        // Check address filter
        if let Some(ref addr) = filter.address {
            if log.address != *addr.as_bytes() {
                return false;
            }
        }

        // Check topic filters
        for (i, topic_filter) in filter.topics.iter().enumerate() {
            if let Some(ref expected_topic) = topic_filter {
                if i >= log.topics.len() {
                    return false;
                }
                if log.topics[i] != *expected_topic.as_bytes() {
                    return false;
                }
            }
        }

        true
    }

    /// Creates a key for logs indexed by block number and tx index
    fn make_logs_key(block_number: u64, tx_index: u32) -> [u8; 12] {
        let mut key = [0u8; 12];
        key[0..8].copy_from_slice(&block_number.to_be_bytes());
        key[8..12].copy_from_slice(&tx_index.to_be_bytes());
        key
    }

    /// Flushes data to disk
    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()?;
        Ok(())
    }
}

// =============================================================================
// Unified Storage
// =============================================================================

/// Unified storage interface for the blockchain
pub struct Storage {
    pub blocks: BlockStore,
    pub state: StateStore,
    pub transactions: TransactionStore,
    path: std::path::PathBuf,
}

impl Storage {
    /// Opens or creates storage at the given path
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        std::fs::create_dir_all(path)?;

        let blocks = BlockStore::new(path)?;
        let state = StateStore::new(path)?;
        let transactions = TransactionStore::new(path)?;

        Ok(Self {
            blocks,
            state,
            transactions,
            path: path.to_path_buf(),
        })
    }

    /// Returns the storage path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Initializes the genesis block and state
    pub fn init_genesis(&mut self, genesis: &GenesisConfig) -> Result<Block, StorageError> {
        // Check if genesis already exists
        if self.blocks.get_block_by_height(0).is_some() {
            return Err(StorageError::GenesisAlreadyInitialized);
        }

        // Initialize accounts from allocation
        for (address, alloc) in &genesis.alloc {
            let mut account = Account::new();
            account.set_balance(alloc.balance);

            // Store code if present
            if let Some(ref code) = alloc.code {
                let code_hash = self.state.put_code(code)?;
                account.code_hash = *code_hash.as_bytes();
            }

            // Store storage if present
            if let Some(ref storage) = alloc.storage {
                for (key, value) in storage {
                    self.state.put_storage(address, key, *value)?;
                }
            }

            self.state.put_account(address, &account)?;
        }

        // Initialize validators (as accounts with stake)
        for validator in &genesis.validators {
            let mut account = self.state.get_account(&validator.address)
                .unwrap_or_default();
            // Add stake to balance
            let current_balance = account.balance_u256();
            if let Some(new_balance) = current_balance.checked_add(&validator.stake) {
                account.set_balance(new_balance);
                self.state.put_account(&validator.address, &account)?;
            }
        }

        // Compute state root
        let state_root = self.state.compute_state_root();

        // Create genesis block (height 0, no parent, no transactions)
        let genesis_block = Block::new(
            0,
            H256::zero(),
            Vec::new(),
            genesis.timestamp,
        );

        // Store genesis block
        self.blocks.put_block(&genesis_block)?;

        // Store genesis block header
        let header = BlockHeader::from_block(&genesis_block, state_root);
        self.blocks.put_block_header(&genesis_block.hash(), &header)?;

        // Flush all data
        self.flush()?;

        Ok(genesis_block)
    }

    /// Closes the storage (flushes all data)
    pub fn close(&self) -> Result<(), StorageError> {
        self.flush()
    }

    /// Flushes all pending writes to disk
    pub fn flush(&self) -> Result<(), StorageError> {
        self.blocks.flush()?;
        self.state.flush()?;
        self.transactions.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_serialization() {
        let mut account = Account::new();
        account.nonce = 42;
        account.set_balance(U256::from_u64(1_000_000));

        let encoded = bincode::serialize(&account).unwrap();
        let decoded: Account = bincode::deserialize(&encoded).unwrap();

        assert_eq!(account, decoded);
        assert_eq!(decoded.nonce, 42);
        assert_eq!(decoded.balance_u256(), U256::from_u64(1_000_000));
    }

    #[test]
    fn test_receipt_serialization() {
        let receipt = TransactionReceipt {
            transaction_hash: [1u8; 32],
            block_hash: [2u8; 32],
            block_number: 100,
            transaction_index: 0,
            gas_used: 21000,
            status: true,
            logs: vec![Log {
                address: [3u8; 20],
                topics: vec![[4u8; 32]],
                data: vec![5, 6, 7],
                block_number: 100,
                transaction_hash: [1u8; 32],
                transaction_index: 0,
                log_index: 0,
            }],
        };

        let encoded = bincode::serialize(&receipt).unwrap();
        let decoded: TransactionReceipt = bincode::deserialize(&encoded).unwrap();

        assert_eq!(receipt, decoded);
    }
}
