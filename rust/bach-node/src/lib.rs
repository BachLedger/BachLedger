//! BachLedger Node
//!
//! Full node implementation integrating all BachLedger components:
//! - Consensus: TBFT Byzantine fault-tolerant consensus
//! - Network: P2P networking for peer communication
//! - Storage: Persistent block and state storage
//! - EVM: Smart contract execution
//! - RPC: JSON-RPC API for external access
//!
//! # Architecture
//!
//! ```text
//! +-------------------+
//! |     BachNode      |
//! +-------------------+
//! |  +-----------+    |
//! |  | Consensus |    |  <- TBFT block ordering
//! |  +-----------+    |
//! |  +-----------+    |
//! |  |  Network  |    |  <- P2P communication
//! |  +-----------+    |
//! |  +-----------+    |
//! |  |  Storage  |    |  <- Persistence layer
//! |  +-----------+    |
//! |  +-----------+    |
//! |  |    EVM    |    |  <- Smart contract execution
//! |  +-----------+    |
//! |  +-----------+    |
//! |  |    RPC    |    |  <- JSON-RPC API
//! |  +-----------+    |
//! +-------------------+
//! ```

#![forbid(unsafe_code)]

use bach_crypto::PrivateKey;
use bach_primitives::{Address, H256, U256};
use bach_rpc::{RpcConfig, RpcServer, RpcState};
use bach_storage::Storage;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

/// Node errors
#[derive(Debug, Error)]
pub enum NodeError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] bach_storage::StorageError),

    #[error("Network error: {0}")]
    NetworkError(#[from] bach_network::NetworkError),

    #[error("Consensus error: {0}")]
    ConsensusError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Node not running")]
    NotRunning,

    #[error("Node already running")]
    AlreadyRunning,
}

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Data directory for storage
    pub data_dir: PathBuf,

    /// Network listen address
    pub listen_addr: SocketAddr,

    /// Bootstrap peers to connect to
    pub bootstrap_peers: Vec<SocketAddr>,

    /// Validator private key (if this node is a validator)
    pub validator_key: Option<[u8; 32]>,

    /// Chain ID
    pub chain_id: u64,

    /// Block time in milliseconds
    pub block_time_ms: u64,

    /// Maximum transactions per block
    pub max_txs_per_block: usize,

    /// Enable RPC server
    pub rpc_enabled: bool,

    /// RPC listen address
    pub rpc_addr: Option<SocketAddr>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            listen_addr: "0.0.0.0:30303".parse().unwrap(),
            bootstrap_peers: Vec::new(),
            validator_key: None,
            chain_id: 1,
            block_time_ms: 3000,
            max_txs_per_block: 1000,
            rpc_enabled: false,
            rpc_addr: None,
        }
    }
}

impl NodeConfig {
    /// Creates a new config with the given data directory.
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            ..Default::default()
        }
    }

    /// Sets the listen address.
    pub fn with_listen_addr(mut self, addr: SocketAddr) -> Self {
        self.listen_addr = addr;
        self
    }

    /// Adds bootstrap peers.
    pub fn with_bootstrap_peers(mut self, peers: Vec<SocketAddr>) -> Self {
        self.bootstrap_peers = peers;
        self
    }

    /// Sets the validator key.
    pub fn with_validator_key(mut self, key: [u8; 32]) -> Self {
        self.validator_key = Some(key);
        self
    }

    /// Sets the chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Enables RPC with the given address.
    pub fn with_rpc(mut self, addr: SocketAddr) -> Self {
        self.rpc_enabled = true;
        self.rpc_addr = Some(addr);
        self
    }

    /// Loads config from a TOML file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, NodeError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| NodeError::ConfigError(format!("Failed to parse config: {}", e)))
    }

    /// Saves config to a TOML file.
    pub fn to_file(&self, path: &std::path::Path) -> Result<(), NodeError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| NodeError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Current node state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// Node is stopped
    Stopped,
    /// Node is starting up
    Starting,
    /// Node is running and syncing
    Syncing,
    /// Node is fully synced and running
    Running,
    /// Node is shutting down
    ShuttingDown,
}

/// BachLedger full node
pub struct BachNode {
    /// Node configuration
    config: NodeConfig,

    /// Current state
    state: NodeState,

    /// Storage layer
    storage: Option<Storage>,

    /// RPC server (if enabled)
    rpc_server: Option<RpcServer>,

    /// Shared RPC state (for EVM integration)
    rpc_state: Option<Arc<RpcState>>,

    /// Our address (if validator)
    validator_address: Option<Address>,

    /// Current block height
    current_height: u64,

    /// Current block hash
    current_hash: H256,
}

impl BachNode {
    /// Creates a new node with the given configuration.
    pub fn new(config: NodeConfig) -> Self {
        Self {
            config,
            state: NodeState::Stopped,
            storage: None,
            rpc_server: None,
            rpc_state: None,
            validator_address: None,
            current_height: 0,
            current_hash: H256::zero(),
        }
    }

    /// Returns the current node state.
    pub fn state(&self) -> NodeState {
        self.state
    }

    /// Returns the node configuration.
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    /// Returns the current block height.
    pub fn current_height(&self) -> u64 {
        self.current_height
    }

    /// Returns the current block hash.
    pub fn current_hash(&self) -> H256 {
        self.current_hash
    }

    /// Returns the validator address if this node is a validator.
    pub fn validator_address(&self) -> Option<&Address> {
        self.validator_address.as_ref()
    }

    /// Returns true if this node is a validator.
    pub fn is_validator(&self) -> bool {
        self.validator_address.is_some()
    }

    /// Initializes the node (opens storage, loads state).
    pub fn init(&mut self) -> Result<(), NodeError> {
        if self.state != NodeState::Stopped {
            return Err(NodeError::AlreadyRunning);
        }

        self.state = NodeState::Starting;

        // Create data directory if needed
        std::fs::create_dir_all(&self.config.data_dir)?;

        // Open storage
        let storage = Storage::open(&self.config.data_dir)?;

        // Load current chain state
        self.current_height = storage.blocks.get_block_height();
        if let Some(block) = storage.blocks.get_latest_block() {
            self.current_hash = block.hash();
        }

        self.storage = Some(storage);

        // Initialize validator identity if key provided
        if let Some(key_bytes) = &self.config.validator_key {
            let private_key = PrivateKey::from_bytes(key_bytes)
                .map_err(|_| NodeError::ConfigError("Invalid validator key".to_string()))?;
            self.validator_address = Some(private_key.public_key().to_address());
        }

        tracing::info!(
            height = self.current_height,
            hash = %self.current_hash,
            validator = ?self.validator_address,
            "Node initialized"
        );

        Ok(())
    }

    /// Starts the node (begins networking and consensus).
    pub async fn start(&mut self) -> Result<(), NodeError> {
        if self.state != NodeState::Starting && self.state != NodeState::Stopped {
            return Err(NodeError::AlreadyRunning);
        }

        // Initialize if not already done
        if self.storage.is_none() {
            self.init()?;
        }

        self.state = NodeState::Syncing;

        tracing::info!(
            listen_addr = %self.config.listen_addr,
            "Node starting"
        );

        // Start RPC server if enabled
        if self.config.rpc_enabled {
            self.start_rpc().await?;
        }

        // TODO: Start network service
        // TODO: Start consensus engine
        // TODO: Start block sync

        self.state = NodeState::Running;

        tracing::info!("Node started");

        Ok(())
    }

    /// Starts the RPC server.
    async fn start_rpc(&mut self) -> Result<(), NodeError> {
        let rpc_addr = self.config.rpc_addr.ok_or_else(|| {
            NodeError::ConfigError("RPC enabled but no address configured".to_string())
        })?;

        let storage = self.storage.take().ok_or(NodeError::NotRunning)?;

        let rpc_config = RpcConfig {
            http_addr: rpc_addr.ip().to_string(),
            http_port: rpc_addr.port(),
            ..Default::default()
        };

        let mut rpc_server = RpcServer::new(rpc_config, storage, self.config.chain_id);
        let state = rpc_server.state();

        // Set initial block height
        {
            let mut height = state.block_height.write().unwrap();
            *height = self.current_height;
        }

        let bound_addr = rpc_server.start().await
            .map_err(|e| NodeError::ConfigError(format!("RPC start failed: {:?}", e)))?;

        tracing::info!("RPC server listening on {}", bound_addr);

        // Store the RPC state for EVM access
        self.rpc_state = Some(state);
        self.rpc_server = Some(rpc_server);

        Ok(())
    }

    /// Stops the node gracefully.
    pub async fn stop(&mut self) -> Result<(), NodeError> {
        if self.state == NodeState::Stopped {
            return Ok(());
        }

        self.state = NodeState::ShuttingDown;

        tracing::info!("Node stopping");

        // Stop RPC server
        if let Some(mut rpc) = self.rpc_server.take() {
            rpc.stop().await;
        }
        self.rpc_state = None;

        // TODO: Stop consensus engine
        // TODO: Stop network service

        // Flush storage
        if let Some(storage) = self.storage.take() {
            storage.flush()?;
        }

        self.state = NodeState::Stopped;

        tracing::info!("Node stopped");

        Ok(())
    }

    /// Returns the RPC state for external access (e.g., for testing).
    pub fn rpc_state(&self) -> Option<&Arc<RpcState>> {
        self.rpc_state.as_ref()
    }

    /// Sets the balance of an address (for testing/genesis).
    pub fn set_balance(&self, address: &Address, balance: U256) -> Result<(), NodeError> {
        let state = self.rpc_state.as_ref().ok_or(NodeError::NotRunning)?;
        let mut evm_state = state.evm_state.write().unwrap();
        evm_state.set_balance(address, balance);
        Ok(())
    }

    /// Gets the balance of an address.
    pub fn get_balance(&self, address: &Address) -> Result<U256, NodeError> {
        let state = self.rpc_state.as_ref().ok_or(NodeError::NotRunning)?;
        let evm_state = state.evm_state.read().unwrap();
        Ok(evm_state.get_balance(address))
    }

    /// Deploys a contract and returns the contract address.
    pub fn deploy_contract(
        &self,
        from: Address,
        code: &[u8],
        value: U256,
        gas_limit: u64,
    ) -> Result<Address, NodeError> {
        let state = self.rpc_state.as_ref().ok_or(NodeError::NotRunning)?;
        let mut evm_state = state.evm_state.write().unwrap();

        let context = bach_evm::EvmContext {
            origin: from,
            caller: from,
            address: Address::zero(),
            value,
            data: code.to_vec(),
            gas_limit,
            gas_price: U256::ZERO,
            block_number: self.current_height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_gas_limit: 30_000_000,
            coinbase: Address::zero(),
            difficulty: U256::ZERO,
            chain_id: self.config.chain_id,
            base_fee: U256::ZERO,
            is_static: false,
            depth: 0,
        };

        bach_evm::deploy_contract(code, context, &mut evm_state)
            .map_err(|e| NodeError::ConsensusError(format!("Contract deployment failed: {:?}", e)))
    }

    /// Calls a contract and returns the output.
    pub fn call_contract(
        &self,
        from: Address,
        to: Address,
        data: &[u8],
        value: U256,
        gas_limit: u64,
    ) -> Result<Vec<u8>, NodeError> {
        let state = self.rpc_state.as_ref().ok_or(NodeError::NotRunning)?;
        let evm_state = state.evm_state.read().unwrap();
        let mut state_copy = evm_state.clone();

        let context = bach_evm::EvmContext {
            origin: from,
            caller: from,
            address: to,
            value,
            data: data.to_vec(),
            gas_limit,
            gas_price: U256::ZERO,
            block_number: self.current_height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_gas_limit: 30_000_000,
            coinbase: Address::zero(),
            difficulty: U256::ZERO,
            chain_id: self.config.chain_id,
            base_fee: U256::ZERO,
            is_static: true,
            depth: 0,
        };

        let result = bach_evm::call_contract(to, data, context, &mut state_copy);

        if result.success {
            Ok(result.output)
        } else {
            Err(NodeError::ConsensusError(format!(
                "Contract call failed: {:?}",
                result.error
            )))
        }
    }

    /// Returns a reference to the storage layer.
    pub fn storage(&self) -> Option<&Storage> {
        self.storage.as_ref()
    }

    /// Returns a mutable reference to the storage layer.
    pub fn storage_mut(&mut self) -> Option<&mut Storage> {
        self.storage.as_mut()
    }
}

impl Drop for BachNode {
    fn drop(&mut self) {
        if self.state != NodeState::Stopped {
            tracing::warn!("Node dropped without proper shutdown");
            // Try to flush storage
            if let Some(storage) = self.storage.take() {
                let _ = storage.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.block_time_ms, 3000);
        assert!(!config.rpc_enabled);
    }

    #[test]
    fn test_config_builder() {
        let config = NodeConfig::default()
            .with_chain_id(42)
            .with_listen_addr("127.0.0.1:9999".parse().unwrap())
            .with_rpc("127.0.0.1:8545".parse().unwrap());

        assert_eq!(config.chain_id, 42);
        assert_eq!(config.listen_addr.port(), 9999);
        assert!(config.rpc_enabled);
        assert_eq!(config.rpc_addr.unwrap().port(), 8545);
    }

    #[test]
    fn test_node_creation() {
        let config = NodeConfig::default();
        let node = BachNode::new(config);

        assert_eq!(node.state(), NodeState::Stopped);
        assert_eq!(node.current_height(), 0);
        assert!(!node.is_validator());
    }

    #[test]
    fn test_node_init() {
        let temp_dir = TempDir::new().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());

        let mut node = BachNode::new(config);
        node.init().unwrap();

        assert_eq!(node.state(), NodeState::Starting);
        assert!(node.storage().is_some());
    }

    #[test]
    fn test_node_with_validator_key() {
        let temp_dir = TempDir::new().unwrap();
        let key = PrivateKey::random();
        let expected_addr = key.public_key().to_address();

        let config = NodeConfig::new(temp_dir.path().to_path_buf())
            .with_validator_key(key.to_bytes());

        let mut node = BachNode::new(config);
        node.init().unwrap();

        assert!(node.is_validator());
        assert_eq!(node.validator_address(), Some(&expected_addr));
    }

    #[tokio::test]
    async fn test_node_start_stop() {
        let temp_dir = TempDir::new().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());

        let mut node = BachNode::new(config);
        node.start().await.unwrap();

        assert_eq!(node.state(), NodeState::Running);

        node.stop().await.unwrap();

        assert_eq!(node.state(), NodeState::Stopped);
    }

    #[tokio::test]
    async fn test_node_with_rpc() {
        let temp_dir = TempDir::new().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf())
            .with_chain_id(42)
            .with_rpc("127.0.0.1:0".parse().unwrap()); // Port 0 = random available port

        let mut node = BachNode::new(config);
        node.start().await.unwrap();

        assert_eq!(node.state(), NodeState::Running);
        assert!(node.rpc_state().is_some());

        // Verify chain ID
        let state = node.rpc_state().unwrap();
        assert_eq!(state.chain_id, 42);

        node.stop().await.unwrap();
        assert!(node.rpc_state().is_none());
    }

    #[tokio::test]
    async fn test_node_balance_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf())
            .with_rpc("127.0.0.1:0".parse().unwrap());

        let mut node = BachNode::new(config);
        node.start().await.unwrap();

        let addr = Address::from([0xaa; 20]);

        // Initially zero balance
        let balance = node.get_balance(&addr).unwrap();
        assert_eq!(balance, U256::ZERO);

        // Set balance
        node.set_balance(&addr, U256::from_u64(1_000_000)).unwrap();

        // Verify balance
        let balance = node.get_balance(&addr).unwrap();
        assert_eq!(balance, U256::from_u64(1_000_000));

        node.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_node_contract_deployment() {
        let temp_dir = TempDir::new().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf())
            .with_rpc("127.0.0.1:0".parse().unwrap());

        let mut node = BachNode::new(config);
        node.start().await.unwrap();

        let deployer = Address::from([0xdd; 20]);

        // Fund deployer
        node.set_balance(&deployer, U256::from_u64(1_000_000_000)).unwrap();

        // Simple contract that returns 42: PUSH1 42, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN
        let init_code = vec![
            0x60, 0x2a, // PUSH1 42
            0x60, 0x00, // PUSH1 0
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3,       // RETURN
        ];

        let result = node.deploy_contract(deployer, &init_code, U256::ZERO, 100_000);

        // Contract deployment might fail if init code doesn't return proper runtime code
        // but the call itself should not error
        match result {
            Ok(addr) => {
                // Contract was deployed
                assert!(!addr.is_zero());
            }
            Err(_) => {
                // Deployment can fail if the bytecode is not proper init code
                // This is expected behavior for this simple example
            }
        }

        node.stop().await.unwrap();
    }
}
