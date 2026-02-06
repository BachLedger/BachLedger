//! Node orchestration for bach-node

use crate::config::NodeConfig;
use crate::genesis::{
    compute_block_hash, decode_body, decode_header, encode_body, encode_header, encode_receipts,
    GenesisBuilder,
};
use bach_core::{BlockExecutor, ExecutionState};
use bach_primitives::{Address, H256};
use bach_storage::{BlockDb, Database, StateDb, StateReader};
use bach_txpool::{PooledTransaction, TxPool};
use bach_types::{Block, BlockBody, BlockHeader, Bloom};
use bytes::Bytes;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::interval;

/// Node error types
#[derive(Debug, Error)]
pub enum NodeError {
    /// Storage error
    #[error("storage error: {0}")]
    Storage(#[from] bach_storage::StorageError),
    /// Genesis error
    #[error("genesis error: {0}")]
    Genesis(#[from] crate::genesis::GenesisError),
    /// Execution error
    #[error("execution error: {0}")]
    Execution(#[from] bach_core::ExecutionError),
    /// IO error
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result type for node operations
pub type NodeResult<T> = Result<T, NodeError>;

/// BachLedger node
pub struct Node {
    config: NodeConfig,
    state_db: Arc<StateDb>,
    block_db: Arc<BlockDb>,
    txpool: Arc<TxPool>,
    executor: Arc<RwLock<BlockExecutor>>,
    running: Arc<RwLock<bool>>,
}

impl Node {
    /// Create a new node
    pub async fn new(config: NodeConfig) -> NodeResult<Self> {
        // Ensure data directory exists
        std::fs::create_dir_all(&config.datadir)?;

        // Open database
        let db_path = config.datadir.join("db");
        let db = Database::new(db_path.to_str().unwrap_or("./data/db"));
        db.open()?;

        let mut state_db = StateDb::new(db.clone());
        let block_db = BlockDb::new(db);

        // Initialize genesis if needed
        if block_db.get_latest_block()?.is_none() {
            let genesis_builder = GenesisBuilder::new(config.genesis.clone(), config.chain_id);
            genesis_builder.init_genesis(&mut state_db, &block_db)?;
        }

        let state_db = Arc::new(state_db);
        let block_db = Arc::new(block_db);

        // Create transaction pool
        let txpool = Arc::new(TxPool::with_defaults());

        // Create block executor with state from database
        let execution_state = load_state_from_db(&state_db)?;
        let executor = Arc::new(RwLock::new(BlockExecutor::with_state(
            execution_state,
            config.chain_id,
        )));

        Ok(Self {
            config,
            state_db,
            block_db,
            txpool,
            executor,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Get the state database
    pub fn state_db(&self) -> &Arc<StateDb> {
        &self.state_db
    }

    /// Get the block database
    pub fn block_db(&self) -> &Arc<BlockDb> {
        &self.block_db
    }

    /// Get the transaction pool
    pub fn txpool(&self) -> &Arc<TxPool> {
        &self.txpool
    }

    /// Get the block executor
    pub fn executor(&self) -> &Arc<RwLock<BlockExecutor>> {
        &self.executor
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.config.chain_id
    }

    /// Get the RPC config
    pub fn rpc_config(&self) -> &crate::config::RpcConfig {
        &self.config.rpc
    }

    /// Run the node
    pub async fn run(self: Arc<Self>) -> NodeResult<()> {
        tracing::info!("Starting BachLedger node...");
        tracing::info!("Chain ID: {}", self.config.chain_id);
        tracing::info!("Data directory: {:?}", self.config.datadir);

        // Mark as running
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // Log latest block info
        if let Some(number) = self.block_db.get_latest_block()? {
            tracing::info!("Latest block: {}", number);
        }

        // Main event loop
        self.main_loop().await
    }

    /// Main event loop
    async fn main_loop(self: &Arc<Self>) -> NodeResult<()> {
        let block_time = self.config.block.block_time;
        let mut ticker = interval(block_time);

        loop {
            ticker.tick().await;

            // Check if still running
            {
                let running = self.running.read().await;
                if !*running {
                    tracing::info!("Node stopped");
                    break;
                }
            }

            // Check for pending transactions
            let pending = self.txpool.get_pending(100);

            if !pending.is_empty() {
                tracing::debug!("Processing {} pending transactions", pending.len());

                // Build and execute block
                if let Err(e) = self.produce_block(pending).await {
                    tracing::error!("Block production error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Produce a block from pending transactions
    async fn produce_block(&self, transactions: Vec<PooledTransaction>) -> NodeResult<()> {
        let mut executor = self.executor.write().await;

        // Get latest block info
        let latest_number = self.block_db.get_latest_block()?.unwrap_or(0);
        let latest_hash = self
            .block_db
            .get_hash_by_number(latest_number)?
            .unwrap_or(H256::ZERO);

        let new_number = latest_number + 1;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Build block
        let block = Block {
            header: BlockHeader {
                parent_hash: latest_hash,
                ommers_hash: H256::ZERO,
                beneficiary: Address::ZERO, // TODO: Configure coinbase
                state_root: H256::ZERO,
                transactions_root: H256::ZERO,
                receipts_root: H256::ZERO,
                logs_bloom: Bloom::default(),
                difficulty: 0,
                number: new_number,
                gas_limit: self.config.block.gas_limit,
                gas_used: 0,
                timestamp,
                extra_data: Bytes::new(),
                mix_hash: H256::ZERO,
                nonce: 0,
                base_fee_per_gas: Some(1_000_000_000),
            },
            body: BlockBody {
                transactions: transactions.iter().map(|pt| pt.tx.clone()).collect(),
            },
        };

        // Execute block
        let result = executor.execute_block(&block)?;

        // Compute block hash
        let hash = compute_block_hash(&block);

        // Store block and receipts
        let header_bytes = encode_header(&block.header);
        let body_bytes = encode_body(&block.body);
        let receipts_bytes = encode_receipts(&result.receipts);

        self.block_db.put_header(&hash, &header_bytes)?;
        self.block_db.put_body(&hash, &body_bytes)?;
        self.block_db.put_receipts(&hash, &receipts_bytes)?;
        self.block_db.put_hash_by_number(new_number, &hash)?;
        self.block_db.set_latest_block(new_number)?;

        // Update transaction pool nonces
        for tx in &transactions {
            self.txpool.set_nonce(&tx.sender, tx.nonce() + 1);
        }

        tracing::info!(
            "Block {} produced: hash={}, txs={}, gas_used={}",
            new_number,
            hash.to_hex(),
            transactions.len(),
            result.gas_used
        );

        Ok(())
    }

    /// Stop the node
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Node stop requested");
    }

    /// Check if node is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get latest block number
    pub fn get_latest_block_number(&self) -> NodeResult<u64> {
        Ok(self.block_db.get_latest_block()?.unwrap_or(0))
    }

    /// Get block by number
    pub fn get_block_by_number(&self, number: u64) -> NodeResult<Option<Block>> {
        let hash = match self.block_db.get_hash_by_number(number)? {
            Some(h) => h,
            None => return Ok(None),
        };
        self.get_block_by_hash(&hash)
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &H256) -> NodeResult<Option<Block>> {
        let header_bytes = match self.block_db.get_header(hash)? {
            Some(b) => b,
            None => return Ok(None),
        };
        let body_bytes = match self.block_db.get_body(hash)? {
            Some(b) => b,
            None => return Ok(None),
        };

        let header = decode_header(&header_bytes)
            .ok_or_else(|| NodeError::Internal("failed to decode header".to_string()))?;
        let body = decode_body(&body_bytes)
            .ok_or_else(|| NodeError::Internal("failed to decode body".to_string()))?;

        Ok(Some(Block { header, body }))
    }

    /// Get account balance
    pub fn get_balance(&self, address: &Address) -> NodeResult<u128> {
        Ok(self.state_db.get_balance(address)?)
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &Address) -> NodeResult<u64> {
        Ok(self.state_db.get_nonce(address)?)
    }

    /// Get account code
    pub fn get_code(&self, address: &Address) -> NodeResult<Vec<u8>> {
        let code_hash = self.state_db.get_code_hash(address)?;
        if code_hash == bach_storage::EMPTY_CODE_HASH {
            return Ok(vec![]);
        }
        Ok(self.state_db.get_code(&code_hash)?.unwrap_or_default())
    }

    /// Get storage value
    pub fn get_storage(&self, address: &Address, slot: &H256) -> NodeResult<H256> {
        Ok(self.state_db.get_storage(address, slot)?)
    }
}

/// Load state from database into ExecutionState
fn load_state_from_db(_state_db: &StateDb) -> NodeResult<ExecutionState> {
    // For now, start with empty state
    // In a production implementation, this would scan the database
    // and load all accounts into the execution state
    Ok(ExecutionState::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{default_genesis_config, BlockConfig, NodeConfig, RpcConfig};
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cnt = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::path::PathBuf::from(format!("/tmp/bach_node_test_{}_{}", id, cnt))
    }

    fn cleanup(path: &std::path::Path) {
        let _ = fs::remove_dir_all(path);
    }

    fn test_config(datadir: std::path::PathBuf) -> NodeConfig {
        NodeConfig {
            datadir,
            chain_id: 1337,
            rpc: RpcConfig::default(),
            genesis: default_genesis_config(),
            block: BlockConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_node_creation() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        assert_eq!(node.chain_id(), 1337);
        assert_eq!(node.get_latest_block_number().unwrap(), 0);

        cleanup(&datadir);
    }

    #[tokio::test]
    async fn test_node_genesis_initialized() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        // Genesis should be at block 0
        let block = node.get_block_by_number(0).unwrap();
        assert!(block.is_some());

        let genesis = block.unwrap();
        assert_eq!(genesis.header.number, 0);
        assert_eq!(genesis.header.parent_hash, H256::ZERO);

        cleanup(&datadir);
    }

    #[tokio::test]
    async fn test_node_get_balance() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        // Check balance of pre-funded account
        let addr = crate::config::parse_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        let balance = node.get_balance(&addr).unwrap();

        // Should have 10000 ETH
        assert!(balance > 0);

        cleanup(&datadir);
    }

    #[tokio::test]
    async fn test_node_running_state() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        assert!(!node.is_running().await);

        cleanup(&datadir);
    }

    #[tokio::test]
    async fn test_node_stop() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        node.stop().await;
        assert!(!node.is_running().await);

        cleanup(&datadir);
    }

    #[tokio::test]
    async fn test_node_get_nonce() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        let addr = crate::config::parse_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        let nonce = node.get_nonce(&addr).unwrap();

        // Initial nonce should be 0
        assert_eq!(nonce, 0);

        cleanup(&datadir);
    }

    #[tokio::test]
    async fn test_node_get_code_empty() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        // EOA should have no code
        let addr = crate::config::parse_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        let code = node.get_code(&addr).unwrap();

        assert!(code.is_empty());

        cleanup(&datadir);
    }

    #[tokio::test]
    async fn test_node_get_storage_empty() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        let addr = crate::config::parse_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        let slot = H256::ZERO;
        let value = node.get_storage(&addr, &slot).unwrap();

        assert_eq!(value, H256::ZERO);

        cleanup(&datadir);
    }

    #[tokio::test]
    async fn test_node_block_not_found() {
        let datadir = temp_dir();
        let config = test_config(datadir.clone());

        let node = Node::new(config).await.unwrap();

        // Block 999 should not exist
        let block = node.get_block_by_number(999).unwrap();
        assert!(block.is_none());

        cleanup(&datadir);
    }
}
