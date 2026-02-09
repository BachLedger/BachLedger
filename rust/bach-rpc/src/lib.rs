//! BachLedger JSON-RPC API
//!
//! Ethereum-compatible JSON-RPC interface for the medical blockchain.
//!
//! Implements standard Ethereum RPC methods:
//! - Transaction submission: `eth_sendTransaction`, `eth_sendRawTransaction`
//! - State queries: `eth_call`, `eth_getBalance`, `eth_getStorageAt`, `eth_getCode`
//! - Block queries: `eth_getBlockByNumber`, `eth_getBlockByHash`
//! - Receipt/Log queries: `eth_getTransactionReceipt`, `eth_getLogs`

#![forbid(unsafe_code)]

use bach_primitives::{Address, H256, U256};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// RPC error codes following Ethereum JSON-RPC conventions
#[derive(Debug, Clone, Copy)]
pub enum RpcErrorCode {
    /// Invalid JSON was received
    ParseError = -32700,
    /// The JSON sent is not a valid Request object
    InvalidRequest = -32600,
    /// The method does not exist / is not available
    MethodNotFound = -32601,
    /// Invalid method parameter(s)
    InvalidParams = -32602,
    /// Internal JSON-RPC error
    InternalError = -32603,
    /// Server error (reserved for implementation-defined server-errors)
    ServerError = -32000,
    /// Transaction rejected
    TransactionRejected = -32003,
    /// Resource not found
    ResourceNotFound = -32001,
    /// Execution error (revert, out of gas, etc.)
    ExecutionError = -32015,
}

/// RPC operation errors
#[derive(Debug, Error)]
pub enum RpcError {
    #[error("Invalid params: {0}")]
    InvalidParams(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Transaction rejected: {0}")]
    TransactionRejected(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

impl From<RpcError> for jsonrpsee::types::ErrorObjectOwned {
    fn from(err: RpcError) -> Self {
        let (code, message) = match &err {
            RpcError::InvalidParams(msg) => (RpcErrorCode::InvalidParams as i32, msg.clone()),
            RpcError::NotFound(msg) => (RpcErrorCode::ResourceNotFound as i32, msg.clone()),
            RpcError::TransactionRejected(msg) => (RpcErrorCode::TransactionRejected as i32, msg.clone()),
            RpcError::ExecutionError(msg) => (RpcErrorCode::ExecutionError as i32, msg.clone()),
            RpcError::InternalError(msg) => (RpcErrorCode::InternalError as i32, msg.clone()),
            RpcError::StorageError(msg) => (RpcErrorCode::ServerError as i32, msg.clone()),
        };
        jsonrpsee::types::ErrorObjectOwned::owned(code, message, None::<()>)
    }
}

// =============================================================================
// RPC Types - Request/Response structures
// =============================================================================

/// Block number parameter - can be a number or tag
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockNumberOrTag {
    /// Block number as hex string
    Number(String),
    /// Block tag: "latest", "earliest", "pending"
    Tag(BlockTag),
}

/// Block tags for querying
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockTag {
    Latest,
    Earliest,
    Pending,
    Safe,
    Finalized,
}

impl Default for BlockNumberOrTag {
    fn default() -> Self {
        BlockNumberOrTag::Tag(BlockTag::Latest)
    }
}

impl BlockNumberOrTag {
    /// Parse to block number (None for pending)
    pub fn to_block_number(&self, latest_height: u64) -> Option<u64> {
        match self {
            BlockNumberOrTag::Number(hex) => {
                let hex = hex.strip_prefix("0x").unwrap_or(hex);
                u64::from_str_radix(hex, 16).ok()
            }
            BlockNumberOrTag::Tag(tag) => match tag {
                BlockTag::Latest | BlockTag::Safe | BlockTag::Finalized => Some(latest_height),
                BlockTag::Earliest => Some(0),
                BlockTag::Pending => None,
            },
        }
    }
}

/// Transaction call object for eth_call and eth_sendTransaction
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRequest {
    /// Sender address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<String>,
    /// Gas price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    /// Transfer value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Input data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

impl CallRequest {
    /// Parse the 'from' field as Address
    pub fn from_address(&self) -> Result<Option<Address>, RpcError> {
        match &self.from {
            Some(s) => {
                let addr = parse_address(s)?;
                Ok(Some(addr))
            }
            None => Ok(None),
        }
    }

    /// Parse the 'to' field as Address
    pub fn to_address(&self) -> Result<Option<Address>, RpcError> {
        match &self.to {
            Some(s) => {
                let addr = parse_address(s)?;
                Ok(Some(addr))
            }
            None => Ok(None),
        }
    }

    /// Parse the 'value' field as U256
    pub fn value_u256(&self) -> Result<U256, RpcError> {
        match &self.value {
            Some(s) => parse_u256(s),
            None => Ok(U256::ZERO),
        }
    }

    /// Parse the 'data' field as bytes
    pub fn input_data(&self) -> Result<Vec<u8>, RpcError> {
        match &self.data {
            Some(s) => parse_bytes(s),
            None => Ok(Vec::new()),
        }
    }

    /// Parse the 'gas' field as u64
    pub fn gas_limit(&self) -> Result<Option<u64>, RpcError> {
        match &self.gas {
            Some(s) => {
                let val = parse_u64(s)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    /// Parse the 'nonce' field as u64
    pub fn nonce_u64(&self) -> Result<Option<u64>, RpcError> {
        match &self.nonce {
            Some(s) => {
                let val = parse_u64(s)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }
}

/// Log filter for eth_getLogs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFilterRequest {
    /// Start block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_block: Option<BlockNumberOrTag>,
    /// End block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_block: Option<BlockNumberOrTag>,
    /// Contract address(es)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<AddressFilter>,
    /// Topic filters (up to 4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<Option<TopicFilter>>>,
    /// Block hash (alternative to from/to block)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<String>,
}

/// Address filter - single or multiple addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressFilter {
    Single(String),
    Multiple(Vec<String>),
}

/// Topic filter - single or multiple topics (OR)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TopicFilter {
    Single(String),
    Multiple(Vec<String>),
}

// =============================================================================
// RPC Response Types
// =============================================================================

/// Block response object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockResponse {
    /// Block number
    pub number: String,
    /// Block hash
    pub hash: String,
    /// Parent block hash
    pub parent_hash: String,
    /// Nonce (PoW)
    pub nonce: String,
    /// SHA3 of uncles
    pub sha3_uncles: String,
    /// Logs bloom filter
    pub logs_bloom: String,
    /// Transactions root
    pub transactions_root: String,
    /// State root
    pub state_root: String,
    /// Receipts root
    pub receipts_root: String,
    /// Miner/validator address
    pub miner: String,
    /// Difficulty
    pub difficulty: String,
    /// Total difficulty
    pub total_difficulty: String,
    /// Extra data
    pub extra_data: String,
    /// Block size in bytes
    pub size: String,
    /// Gas limit
    pub gas_limit: String,
    /// Gas used
    pub gas_used: String,
    /// Block timestamp
    pub timestamp: String,
    /// Transactions (hashes or full objects)
    pub transactions: TransactionsResponse,
    /// Uncles
    pub uncles: Vec<String>,
}

/// Transactions in block - either hashes only or full objects
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionsResponse {
    Hashes(Vec<String>),
    Full(Vec<TransactionResponse>),
}

/// Transaction response object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    /// Block hash
    pub block_hash: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Sender address
    pub from: String,
    /// Gas limit
    pub gas: String,
    /// Gas price
    pub gas_price: String,
    /// Transaction hash
    pub hash: String,
    /// Input data
    pub input: String,
    /// Nonce
    pub nonce: String,
    /// Recipient address
    pub to: Option<String>,
    /// Transaction index in block
    pub transaction_index: Option<String>,
    /// Transfer value
    pub value: String,
    /// ECDSA recovery id
    pub v: String,
    /// ECDSA signature r
    pub r: String,
    /// ECDSA signature s
    pub s: String,
}

/// Transaction receipt response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptResponse {
    /// Transaction hash
    pub transaction_hash: String,
    /// Transaction index
    pub transaction_index: String,
    /// Block hash
    pub block_hash: String,
    /// Block number
    pub block_number: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: Option<String>,
    /// Cumulative gas used
    pub cumulative_gas_used: String,
    /// Gas used by this transaction
    pub gas_used: String,
    /// Contract address (if contract creation)
    pub contract_address: Option<String>,
    /// Logs
    pub logs: Vec<LogResponse>,
    /// Logs bloom filter
    pub logs_bloom: String,
    /// Transaction type
    #[serde(rename = "type")]
    pub tx_type: String,
    /// Status (1 = success, 0 = failure)
    pub status: String,
}

/// Log response object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogResponse {
    /// Whether log was removed (reorg)
    pub removed: bool,
    /// Log index in block
    pub log_index: String,
    /// Transaction index
    pub transaction_index: String,
    /// Transaction hash
    pub transaction_hash: String,
    /// Block hash
    pub block_hash: String,
    /// Block number
    pub block_number: String,
    /// Contract address
    pub address: String,
    /// Log data
    pub data: String,
    /// Indexed topics
    pub topics: Vec<String>,
}

// =============================================================================
// RPC Trait Definition
// =============================================================================

/// Ethereum-compatible JSON-RPC API
#[rpc(server, namespace = "eth")]
pub trait EthApi {
    /// Sends a signed transaction to the network
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, data: String) -> RpcResult<String>;

    /// Creates and sends a transaction (requires unlocked account)
    #[method(name = "sendTransaction")]
    async fn send_transaction(&self, tx: CallRequest) -> RpcResult<String>;

    /// Executes a call without creating a transaction
    #[method(name = "call")]
    async fn call(
        &self,
        tx: CallRequest,
        block: Option<BlockNumberOrTag>,
    ) -> RpcResult<String>;

    /// Returns the balance of an account
    #[method(name = "getBalance")]
    async fn get_balance(
        &self,
        address: String,
        block: Option<BlockNumberOrTag>,
    ) -> RpcResult<String>;

    /// Returns the storage value at a position
    #[method(name = "getStorageAt")]
    async fn get_storage_at(
        &self,
        address: String,
        position: String,
        block: Option<BlockNumberOrTag>,
    ) -> RpcResult<String>;

    /// Returns the code at an address
    #[method(name = "getCode")]
    async fn get_code(
        &self,
        address: String,
        block: Option<BlockNumberOrTag>,
    ) -> RpcResult<String>;

    /// Returns a block by number
    #[method(name = "getBlockByNumber")]
    async fn get_block_by_number(
        &self,
        block: BlockNumberOrTag,
        full_transactions: bool,
    ) -> RpcResult<Option<BlockResponse>>;

    /// Returns a block by hash
    #[method(name = "getBlockByHash")]
    async fn get_block_by_hash(
        &self,
        hash: String,
        full_transactions: bool,
    ) -> RpcResult<Option<BlockResponse>>;

    /// Returns the receipt of a transaction
    #[method(name = "getTransactionReceipt")]
    async fn get_transaction_receipt(
        &self,
        hash: String,
    ) -> RpcResult<Option<ReceiptResponse>>;

    /// Returns logs matching a filter
    #[method(name = "getLogs")]
    async fn get_logs(&self, filter: LogFilterRequest) -> RpcResult<Vec<LogResponse>>;

    /// Returns the current block number
    #[method(name = "blockNumber")]
    async fn block_number(&self) -> RpcResult<String>;

    /// Returns the chain ID
    #[method(name = "chainId")]
    async fn chain_id(&self) -> RpcResult<String>;

    /// Returns the transaction count (nonce) for an address
    #[method(name = "getTransactionCount")]
    async fn get_transaction_count(
        &self,
        address: String,
        block: Option<BlockNumberOrTag>,
    ) -> RpcResult<String>;

    /// Estimates gas for a transaction
    #[method(name = "estimateGas")]
    async fn estimate_gas(
        &self,
        tx: CallRequest,
        block: Option<BlockNumberOrTag>,
    ) -> RpcResult<String>;

    /// Returns the current gas price
    #[method(name = "gasPrice")]
    async fn gas_price(&self) -> RpcResult<String>;
}

/// Net namespace RPC methods
#[rpc(server, namespace = "net")]
pub trait NetApi {
    /// Returns the network ID
    #[method(name = "version")]
    async fn version(&self) -> RpcResult<String>;

    /// Returns true if client is listening for connections
    #[method(name = "listening")]
    async fn listening(&self) -> RpcResult<bool>;

    /// Returns number of peers
    #[method(name = "peerCount")]
    async fn peer_count(&self) -> RpcResult<String>;
}

/// Web3 namespace RPC methods
#[rpc(server, namespace = "web3")]
pub trait Web3Api {
    /// Returns the client version
    #[method(name = "clientVersion")]
    async fn client_version(&self) -> RpcResult<String>;

    /// Returns Keccak-256 hash of the given data
    #[method(name = "sha3")]
    async fn sha3(&self, data: String) -> RpcResult<String>;
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse hex string to Address
pub fn parse_address(s: &str) -> Result<Address, RpcError> {
    Address::from_hex(s).map_err(|e| RpcError::InvalidParams(format!("Invalid address: {:?}", e)))
}

/// Parse hex string to H256
pub fn parse_h256(s: &str) -> Result<H256, RpcError> {
    H256::from_hex(s).map_err(|e| RpcError::InvalidParams(format!("Invalid hash: {:?}", e)))
}

/// Parse hex string to U256
pub fn parse_u256(s: &str) -> Result<U256, RpcError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.is_empty() {
        return Ok(U256::ZERO);
    }

    // Pad to 64 chars (32 bytes)
    let padded = format!("{:0>64}", s);
    let bytes = hex::decode(&padded)
        .map_err(|e| RpcError::InvalidParams(format!("Invalid hex: {}", e)))?;

    if bytes.len() != 32 {
        return Err(RpcError::InvalidParams("Invalid U256 length".to_string()));
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(U256::from_be_bytes(arr))
}

/// Parse hex string to u64
pub fn parse_u64(s: &str) -> Result<u64, RpcError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).map_err(|e| RpcError::InvalidParams(format!("Invalid number: {}", e)))
}

/// Parse hex string to bytes
pub fn parse_bytes(s: &str) -> Result<Vec<u8>, RpcError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.is_empty() {
        return Ok(Vec::new());
    }
    hex::decode(s).map_err(|e| RpcError::InvalidParams(format!("Invalid hex data: {}", e)))
}

/// Format Address as hex string
pub fn format_address(addr: &Address) -> String {
    format!("0x{}", hex::encode(addr.as_bytes()))
}

/// Format H256 as hex string
pub fn format_h256(hash: &H256) -> String {
    format!("0x{}", hex::encode(hash.as_bytes()))
}

/// Format U256 as hex string (minimal representation, no leading zeros)
pub fn format_u256(val: &U256) -> String {
    if val.is_zero() {
        return "0x0".to_string();
    }
    let bytes = val.to_be_bytes();
    // Skip leading zero bytes
    let first_nonzero = bytes.iter().position(|&b| b != 0).unwrap_or(31);
    let hex_str = hex::encode(&bytes[first_nonzero..]);
    // Also strip leading zero from first byte if present
    let trimmed = hex_str.trim_start_matches('0');
    if trimmed.is_empty() {
        "0x0".to_string()
    } else {
        format!("0x{}", trimmed)
    }
}

/// Format u64 as hex string
pub fn format_u64(val: u64) -> String {
    format!("0x{:x}", val)
}

/// Format bytes as hex string
pub fn format_bytes(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

// =============================================================================
// Server Configuration
// =============================================================================

/// RPC server configuration
#[derive(Debug, Clone)]
pub struct RpcConfig {
    /// HTTP listen address
    pub http_addr: String,
    /// HTTP port
    pub http_port: u16,
    /// WebSocket listen address (optional)
    pub ws_addr: Option<String>,
    /// WebSocket port (optional)
    pub ws_port: Option<u16>,
    /// Maximum connections
    pub max_connections: u32,
    /// Enable CORS
    pub cors_enabled: bool,
    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            http_addr: "127.0.0.1".to_string(),
            http_port: 8545,
            ws_addr: None,
            ws_port: None,
            max_connections: 100,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0x742d35Cc6634C0532925a3b844Bc9e7595f1b0E0").unwrap();
        assert!(!addr.is_zero());
    }

    #[test]
    fn test_parse_u256() {
        let val = parse_u256("0x1").unwrap();
        assert_eq!(val, U256::from_u64(1));

        let val = parse_u256("0x10").unwrap();
        assert_eq!(val, U256::from_u64(16));

        let val = parse_u256("0x0").unwrap();
        assert_eq!(val, U256::ZERO);
    }

    #[test]
    fn test_parse_u64() {
        assert_eq!(parse_u64("0x1").unwrap(), 1);
        assert_eq!(parse_u64("0x10").unwrap(), 16);
        assert_eq!(parse_u64("0xff").unwrap(), 255);
    }

    #[test]
    fn test_parse_bytes() {
        let bytes = parse_bytes("0x1234").unwrap();
        assert_eq!(bytes, vec![0x12, 0x34]);

        let empty = parse_bytes("0x").unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_format_u256() {
        assert_eq!(format_u256(&U256::ZERO), "0x0");
        assert_eq!(format_u256(&U256::from_u64(1)), "0x1");
        assert_eq!(format_u256(&U256::from_u64(16)), "0x10");
        assert_eq!(format_u256(&U256::from_u64(255)), "0xff");
    }

    #[test]
    fn test_format_u64() {
        assert_eq!(format_u64(0), "0x0");
        assert_eq!(format_u64(1), "0x1");
        assert_eq!(format_u64(255), "0xff");
    }

    #[test]
    fn test_block_number_or_tag() {
        let latest = BlockNumberOrTag::Tag(BlockTag::Latest);
        assert_eq!(latest.to_block_number(100), Some(100));

        let earliest = BlockNumberOrTag::Tag(BlockTag::Earliest);
        assert_eq!(earliest.to_block_number(100), Some(0));

        let pending = BlockNumberOrTag::Tag(BlockTag::Pending);
        assert_eq!(pending.to_block_number(100), None);

        let number = BlockNumberOrTag::Number("0x10".to_string());
        assert_eq!(number.to_block_number(100), Some(16));
    }

    #[test]
    fn test_call_request_parsing() {
        let req = CallRequest {
            from: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f1b0E0".to_string()),
            to: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f1b0E1".to_string()),
            value: Some("0x1".to_string()),
            data: Some("0x1234".to_string()),
            gas: Some("0x5208".to_string()),
            nonce: Some("0x1".to_string()),
            ..Default::default()
        };

        assert!(req.from_address().unwrap().is_some());
        assert!(req.to_address().unwrap().is_some());
        assert_eq!(req.value_u256().unwrap(), U256::from_u64(1));
        assert_eq!(req.input_data().unwrap(), vec![0x12, 0x34]);
        assert_eq!(req.gas_limit().unwrap(), Some(21000));
        assert_eq!(req.nonce_u64().unwrap(), Some(1));
    }
}
