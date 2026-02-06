//! RPC request and response types

use bach_primitives::{Address, H256, U256};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::JsonRpcError;

/// JSON-RPC request ID (can be number, string, or null)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// Numeric ID
    Number(u64),
    /// String ID
    String(String),
    /// Null ID
    Null,
}

impl Default for JsonRpcId {
    fn default() -> Self {
        Self::Null
    }
}

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    /// Request ID
    #[serde(default)]
    pub id: JsonRpcId,
    /// Method name
    pub method: String,
    /// Method parameters
    #[serde(default)]
    pub params: Vec<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: JsonRpcId,
    /// Result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create success response
    pub fn success(id: JsonRpcId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create error response
    pub fn error(id: JsonRpcId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// Block identifier for RPC calls
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockId {
    /// Block number
    Number(u64),
    /// Block hash
    Hash(H256),
    /// Latest block
    Latest,
    /// Earliest block (genesis)
    Earliest,
    /// Pending block
    Pending,
    /// Safe block
    Safe,
    /// Finalized block
    Finalized,
}

impl Default for BlockId {
    fn default() -> Self {
        Self::Latest
    }
}

/// Parse block ID from JSON value
pub fn parse_block_id(value: &Value) -> Result<BlockId, JsonRpcError> {
    match value {
        Value::String(s) => {
            let s = s.to_lowercase();
            match s.as_str() {
                "latest" => Ok(BlockId::Latest),
                "earliest" => Ok(BlockId::Earliest),
                "pending" => Ok(BlockId::Pending),
                "safe" => Ok(BlockId::Safe),
                "finalized" => Ok(BlockId::Finalized),
                _ if s.starts_with("0x") => {
                    // Could be block number or hash
                    if s.len() == 66 {
                        // 32 bytes = block hash
                        let hash = H256::from_hex(&s).map_err(|e| {
                            JsonRpcError::invalid_params(format!("invalid block hash: {}", e))
                        })?;
                        Ok(BlockId::Hash(hash))
                    } else {
                        // Block number
                        let num = u64::from_str_radix(&s[2..], 16).map_err(|e| {
                            JsonRpcError::invalid_params(format!("invalid block number: {}", e))
                        })?;
                        Ok(BlockId::Number(num))
                    }
                }
                _ => Err(JsonRpcError::invalid_params(format!(
                    "invalid block id: {}",
                    s
                ))),
            }
        }
        Value::Number(n) => {
            let num = n
                .as_u64()
                .ok_or_else(|| JsonRpcError::invalid_params("invalid block number"))?;
            Ok(BlockId::Number(num))
        }
        _ => Err(JsonRpcError::invalid_params("invalid block id type")),
    }
}

/// Parse address from JSON value
pub fn parse_address(value: &Value) -> Result<Address, JsonRpcError> {
    let s = value
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("address must be a string"))?;
    Address::from_hex(s)
        .map_err(|e| JsonRpcError::invalid_params(format!("invalid address: {}", e)))
}

/// Parse H256 from JSON value
pub fn parse_h256(value: &Value) -> Result<H256, JsonRpcError> {
    let s = value
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("hash must be a string"))?;
    H256::from_hex(s).map_err(|e| JsonRpcError::invalid_params(format!("invalid hash: {}", e)))
}

/// Parse hex bytes from JSON value
pub fn parse_hex_bytes(value: &Value) -> Result<Vec<u8>, JsonRpcError> {
    let s = value
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("data must be a hex string"))?;

    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).map_err(|e| JsonRpcError::invalid_params(format!("invalid hex data: {}", e)))
}

/// Parse U256 from JSON value (hex string)
pub fn parse_u256(value: &Value) -> Result<U256, JsonRpcError> {
    let s = value
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("quantity must be a hex string"))?;

    let s = s.strip_prefix("0x").unwrap_or(s);
    U256::from_str_radix(s, 16)
        .map_err(|e| JsonRpcError::invalid_params(format!("invalid quantity: {}", e)))
}

/// Parse u64 from JSON value (hex string)
pub fn parse_u64(value: &Value) -> Result<u64, JsonRpcError> {
    let s = value
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("quantity must be a hex string"))?;

    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16)
        .map_err(|e| JsonRpcError::invalid_params(format!("invalid quantity: {}", e)))
}

/// Format U256 as hex string
pub fn format_u256(value: &U256) -> String {
    format!("0x{:x}", value)
}

/// Format u128 as hex string
pub fn format_u128(value: u128) -> String {
    format!("0x{:x}", value)
}

/// Format u64 as hex string
pub fn format_u64(value: u64) -> String {
    format!("0x{:x}", value)
}

/// Format bytes as hex string
pub fn format_bytes(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Call request for eth_call and eth_estimateGas (raw JSON form)
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallRequestRaw {
    /// From address (optional, defaults to zero address)
    pub from: Option<String>,
    /// To address (required for calls, None for contract creation)
    pub to: Option<String>,
    /// Gas limit (optional)
    pub gas: Option<String>,
    /// Gas price (optional, legacy)
    pub gas_price: Option<String>,
    /// Max fee per gas (optional, EIP-1559)
    pub max_fee_per_gas: Option<String>,
    /// Max priority fee per gas (optional, EIP-1559)
    pub max_priority_fee_per_gas: Option<String>,
    /// Value to send (optional)
    pub value: Option<String>,
    /// Input data (optional)
    #[serde(default, alias = "input")]
    pub data: Option<String>,
    /// Nonce (optional)
    pub nonce: Option<String>,
}

/// Call request for eth_call and eth_estimateGas (parsed form)
#[derive(Debug, Clone, Default)]
pub struct CallRequest {
    /// From address (optional, defaults to zero address)
    pub from: Option<Address>,
    /// To address (required for calls, None for contract creation)
    pub to: Option<Address>,
    /// Gas limit (optional)
    pub gas: Option<u64>,
    /// Gas price (optional, legacy)
    pub gas_price: Option<u128>,
    /// Max fee per gas (optional, EIP-1559)
    pub max_fee_per_gas: Option<u128>,
    /// Max priority fee per gas (optional, EIP-1559)
    pub max_priority_fee_per_gas: Option<u128>,
    /// Value to send (optional)
    pub value: Option<U256>,
    /// Input data (optional)
    pub data: Option<Bytes>,
    /// Nonce (optional)
    pub nonce: Option<u64>,
}

impl CallRequest {
    /// Parse from raw JSON form
    pub fn from_raw(raw: CallRequestRaw) -> Result<Self, crate::error::JsonRpcError> {
        Ok(Self {
            from: raw.from.map(|s| Address::from_hex(&s)).transpose()
                .map_err(|e| crate::error::JsonRpcError::invalid_params(format!("invalid from: {}", e)))?,
            to: raw.to.map(|s| Address::from_hex(&s)).transpose()
                .map_err(|e| crate::error::JsonRpcError::invalid_params(format!("invalid to: {}", e)))?,
            gas: raw.gas.map(|s| parse_hex_u64(&s)).transpose()?,
            gas_price: raw.gas_price.map(|s| parse_hex_u128(&s)).transpose()?,
            max_fee_per_gas: raw.max_fee_per_gas.map(|s| parse_hex_u128(&s)).transpose()?,
            max_priority_fee_per_gas: raw.max_priority_fee_per_gas.map(|s| parse_hex_u128(&s)).transpose()?,
            value: raw.value.map(|s| {
                let s = s.strip_prefix("0x").unwrap_or(&s);
                U256::from_str_radix(s, 16)
                    .map_err(|e| crate::error::JsonRpcError::invalid_params(format!("invalid value: {}", e)))
            }).transpose()?,
            data: raw.data.map(|s| {
                let s = s.strip_prefix("0x").unwrap_or(&s);
                hex::decode(s)
                    .map(Bytes::from)
                    .map_err(|e| crate::error::JsonRpcError::invalid_params(format!("invalid data: {}", e)))
            }).transpose()?,
            nonce: raw.nonce.map(|s| parse_hex_u64(&s)).transpose()?,
        })
    }
}

fn parse_hex_u64(s: &str) -> Result<u64, crate::error::JsonRpcError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16)
        .map_err(|e| crate::error::JsonRpcError::invalid_params(format!("invalid hex u64: {}", e)))
}

fn parse_hex_u128(s: &str) -> Result<u128, crate::error::JsonRpcError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u128::from_str_radix(s, 16)
        .map_err(|e| crate::error::JsonRpcError::invalid_params(format!("invalid hex u128: {}", e)))
}


/// RPC block representation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlock {
    /// Block hash
    pub hash: String,
    /// Parent block hash
    pub parent_hash: String,
    /// Sha3 uncles hash
    pub sha3_uncles: String,
    /// Miner/coinbase address
    pub miner: String,
    /// State root
    pub state_root: String,
    /// Transactions root
    pub transactions_root: String,
    /// Receipts root
    pub receipts_root: String,
    /// Logs bloom
    pub logs_bloom: String,
    /// Difficulty
    pub difficulty: String,
    /// Block number
    pub number: String,
    /// Gas limit
    pub gas_limit: String,
    /// Gas used
    pub gas_used: String,
    /// Timestamp
    pub timestamp: String,
    /// Extra data
    pub extra_data: String,
    /// Mix hash
    pub mix_hash: String,
    /// Nonce
    pub nonce: String,
    /// Base fee per gas (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee_per_gas: Option<String>,
    /// Total difficulty
    pub total_difficulty: String,
    /// Block size
    pub size: String,
    /// Transactions (hashes or full objects)
    pub transactions: Vec<Value>,
    /// Uncles
    pub uncles: Vec<String>,
}

/// RPC transaction representation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransaction {
    /// Transaction hash
    pub hash: String,
    /// Nonce
    pub nonce: String,
    /// Block hash (None if pending)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<String>,
    /// Block number (None if pending)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<String>,
    /// Transaction index (None if pending)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<String>,
    /// From address
    pub from: String,
    /// To address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Value
    pub value: String,
    /// Gas limit
    pub gas: String,
    /// Gas price
    pub gas_price: String,
    /// Input data
    pub input: String,
    /// V
    pub v: String,
    /// R
    pub r: String,
    /// S
    pub s: String,
    /// Transaction type
    #[serde(rename = "type")]
    pub tx_type: String,
    /// Chain ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
    /// Max fee per gas (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,
    /// Max priority fee per gas (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,
}

/// RPC receipt representation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcReceipt {
    /// Transaction hash
    pub transaction_hash: String,
    /// Transaction index
    pub transaction_index: String,
    /// Block hash
    pub block_hash: String,
    /// Block number
    pub block_number: String,
    /// From address
    pub from: String,
    /// To address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Cumulative gas used
    pub cumulative_gas_used: String,
    /// Gas used
    pub gas_used: String,
    /// Contract address (if contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_address: Option<String>,
    /// Logs
    pub logs: Vec<RpcLog>,
    /// Logs bloom
    pub logs_bloom: String,
    /// Transaction type
    #[serde(rename = "type")]
    pub tx_type: String,
    /// Status (1 = success, 0 = failure)
    pub status: String,
    /// Effective gas price
    pub effective_gas_price: String,
}

/// RPC log representation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcLog {
    /// Log address
    pub address: String,
    /// Log topics
    pub topics: Vec<String>,
    /// Log data
    pub data: String,
    /// Block hash
    pub block_hash: String,
    /// Block number
    pub block_number: String,
    /// Transaction hash
    pub transaction_hash: String,
    /// Transaction index
    pub transaction_index: String,
    /// Log index
    pub log_index: String,
    /// Removed (always false for confirmed logs)
    pub removed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== BlockId Parsing Tests =====

    #[test]
    fn test_parse_block_id_latest() {
        let result = parse_block_id(&Value::String("latest".to_string()));
        assert_eq!(result.unwrap(), BlockId::Latest);
    }

    #[test]
    fn test_parse_block_id_latest_case_insensitive() {
        let result = parse_block_id(&Value::String("LATEST".to_string()));
        assert_eq!(result.unwrap(), BlockId::Latest);
        let result = parse_block_id(&Value::String("Latest".to_string()));
        assert_eq!(result.unwrap(), BlockId::Latest);
    }

    #[test]
    fn test_parse_block_id_number() {
        let result = parse_block_id(&Value::String("0x10".to_string()));
        assert_eq!(result.unwrap(), BlockId::Number(16));
    }

    #[test]
    fn test_parse_block_id_number_zero() {
        let result = parse_block_id(&Value::String("0x0".to_string()));
        assert_eq!(result.unwrap(), BlockId::Number(0));
    }

    #[test]
    fn test_parse_block_id_number_large() {
        let result = parse_block_id(&Value::String("0xffffffff".to_string()));
        assert_eq!(result.unwrap(), BlockId::Number(0xffffffff));
    }

    #[test]
    fn test_parse_block_id_earliest() {
        let result = parse_block_id(&Value::String("earliest".to_string()));
        assert_eq!(result.unwrap(), BlockId::Earliest);
    }

    #[test]
    fn test_parse_block_id_pending() {
        let result = parse_block_id(&Value::String("pending".to_string()));
        assert_eq!(result.unwrap(), BlockId::Pending);
    }

    #[test]
    fn test_parse_block_id_safe() {
        let result = parse_block_id(&Value::String("safe".to_string()));
        assert_eq!(result.unwrap(), BlockId::Safe);
    }

    #[test]
    fn test_parse_block_id_finalized() {
        let result = parse_block_id(&Value::String("finalized".to_string()));
        assert_eq!(result.unwrap(), BlockId::Finalized);
    }

    #[test]
    fn test_parse_block_id_json_number() {
        let result = parse_block_id(&Value::Number(serde_json::Number::from(100u64)));
        assert_eq!(result.unwrap(), BlockId::Number(100));
    }

    #[test]
    fn test_parse_block_id_hash() {
        let hash_str = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_block_id(&Value::String(hash_str.to_string()));
        assert!(matches!(result.unwrap(), BlockId::Hash(_)));
    }

    #[test]
    fn test_parse_block_id_invalid() {
        let result = parse_block_id(&Value::String("invalid".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_block_id_invalid_type() {
        let result = parse_block_id(&Value::Bool(true));
        assert!(result.is_err());
    }

    // ===== Address Parsing Tests =====

    #[test]
    fn test_parse_address_valid() {
        let addr_str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
        let result = parse_address(&Value::String(addr_str.to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_address_lowercase() {
        let addr_str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
        let result = parse_address(&Value::String(addr_str.to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_address_invalid_not_string() {
        let result = parse_address(&Value::Number(serde_json::Number::from(123)));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_address_invalid_hex() {
        let result = parse_address(&Value::String("0xGGGG".to_string()));
        assert!(result.is_err());
    }

    // ===== H256 Parsing Tests =====

    #[test]
    fn test_parse_h256_valid() {
        let hash_str = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_h256(&Value::String(hash_str.to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_h256_invalid_not_string() {
        let result = parse_h256(&Value::Number(serde_json::Number::from(123)));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_h256_invalid_length() {
        let result = parse_h256(&Value::String("0x1234".to_string()));
        assert!(result.is_err());
    }

    // ===== Hex Bytes Parsing Tests =====

    #[test]
    fn test_parse_hex_bytes_empty() {
        let result = parse_hex_bytes(&Value::String("0x".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_hex_bytes_valid() {
        let result = parse_hex_bytes(&Value::String("0xabcd".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0xab, 0xcd]);
    }

    #[test]
    fn test_parse_hex_bytes_without_prefix() {
        let result = parse_hex_bytes(&Value::String("abcd".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0xab, 0xcd]);
    }

    #[test]
    fn test_parse_hex_bytes_invalid() {
        let result = parse_hex_bytes(&Value::String("0xGGGG".to_string()));
        assert!(result.is_err());
    }

    // ===== U256 Parsing Tests =====

    #[test]
    fn test_parse_u256_zero() {
        let result = parse_u256(&Value::String("0x0".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), U256::zero());
    }

    #[test]
    fn test_parse_u256_hex() {
        let result = parse_u256(&Value::String("0x10".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), U256::from(16u64));
    }

    #[test]
    fn test_parse_u256_large() {
        let result = parse_u256(&Value::String("0xde0b6b3a7640000".to_string()));
        assert!(result.is_ok());
        // 1 ETH = 10^18 wei
        assert_eq!(result.unwrap(), U256::from(1_000_000_000_000_000_000u64));
    }

    // ===== u64 Parsing Tests =====

    #[test]
    fn test_parse_u64_zero() {
        let result = parse_u64(&Value::String("0x0".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0u64);
    }

    #[test]
    fn test_parse_u64_hex() {
        let result = parse_u64(&Value::String("0xff".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 255u64);
    }

    // ===== Formatting Tests =====

    #[test]
    fn test_format_u64() {
        assert_eq!(format_u64(0), "0x0");
        assert_eq!(format_u64(16), "0x10");
        assert_eq!(format_u64(255), "0xff");
        assert_eq!(format_u64(1_000_000), "0xf4240");
    }

    #[test]
    fn test_format_u128() {
        assert_eq!(format_u128(0), "0x0");
        assert_eq!(format_u128(1_000_000_000), "0x3b9aca00"); // 1 gwei
        assert_eq!(format_u128(1_000_000_000_000_000_000), "0xde0b6b3a7640000"); // 1 ETH
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(&[]), "0x");
        assert_eq!(format_bytes(&[0xab, 0xcd]), "0xabcd");
        assert_eq!(format_bytes(&[0x00, 0x01, 0xff]), "0x0001ff");
    }

    // ===== JSON-RPC ID Tests =====

    #[test]
    fn test_json_rpc_id_default() {
        let id = JsonRpcId::default();
        assert_eq!(id, JsonRpcId::Null);
    }

    #[test]
    fn test_json_rpc_id_number_serde() {
        let id = JsonRpcId::Number(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
        let parsed: JsonRpcId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn test_json_rpc_id_string_serde() {
        let id = JsonRpcId::String("test-id".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"test-id\"");
        let parsed: JsonRpcId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn test_json_rpc_id_null_serde() {
        let id = JsonRpcId::Null;
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "null");
        let parsed: JsonRpcId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    // ===== JSON-RPC Request Tests =====

    #[test]
    fn test_json_rpc_request_deserialize() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": []
        }"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.id, JsonRpcId::Number(1));
        assert_eq!(req.method, "eth_blockNumber");
        assert!(req.params.is_empty());
    }

    #[test]
    fn test_json_rpc_request_with_params() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": "test",
            "method": "eth_getBalance",
            "params": ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "latest"]
        }"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id, JsonRpcId::String("test".to_string()));
        assert_eq!(req.method, "eth_getBalance");
        assert_eq!(req.params.len(), 2);
    }

    #[test]
    fn test_json_rpc_request_no_params() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_chainId"
        }"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert!(req.params.is_empty());
    }

    #[test]
    fn test_json_rpc_request_null_id() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": null,
            "method": "eth_chainId"
        }"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id, JsonRpcId::Null);
    }

    // ===== JSON-RPC Response Tests =====

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(JsonRpcId::Number(1), Value::String("0x10".into()));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.jsonrpc, "2.0");
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(
            JsonRpcId::Number(1),
            JsonRpcError::method_not_found("unknown"),
        );
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.jsonrpc, "2.0");
    }

    #[test]
    fn test_json_rpc_response_success_serialize() {
        let response = JsonRpcResponse::success(JsonRpcId::Number(1), Value::String("0x10".into()));
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"result\":\"0x10\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_json_rpc_response_error_serialize() {
        let response = JsonRpcResponse::error(
            JsonRpcId::Number(1),
            JsonRpcError::invalid_params("missing parameter"),
        );
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"error\""));
        assert!(json.contains("\"code\":-32602"));
        assert!(!json.contains("\"result\""));
    }

    // ===== CallRequestRaw Tests =====

    #[test]
    fn test_call_request_raw_deserialize_minimal() {
        let json = r#"{}"#;
        let req: CallRequestRaw = serde_json::from_str(json).unwrap();
        assert!(req.from.is_none());
        assert!(req.to.is_none());
        assert!(req.gas.is_none());
        assert!(req.value.is_none());
        assert!(req.data.is_none());
    }

    #[test]
    fn test_call_request_raw_deserialize_full() {
        let json = r#"{
            "from": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "to": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            "gas": "0x5208",
            "gasPrice": "0x3b9aca00",
            "value": "0xde0b6b3a7640000",
            "data": "0xabcd"
        }"#;
        let req: CallRequestRaw = serde_json::from_str(json).unwrap();
        assert!(req.from.is_some());
        assert!(req.to.is_some());
        assert!(req.gas.is_some());
        assert!(req.gas_price.is_some());
        assert!(req.value.is_some());
        assert!(req.data.is_some());
    }

    #[test]
    fn test_call_request_raw_input_alias() {
        let json = r#"{
            "input": "0xabcd"
        }"#;
        let req: CallRequestRaw = serde_json::from_str(json).unwrap();
        assert_eq!(req.data, Some("0xabcd".to_string()));
    }

    #[test]
    fn test_call_request_from_raw() {
        let raw = CallRequestRaw {
            from: Some("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string()),
            to: Some("0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string()),
            gas: Some("0x5208".to_string()),
            gas_price: Some("0x3b9aca00".to_string()),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            value: Some("0x0".to_string()),
            data: Some("0xabcd".to_string()),
            nonce: Some("0x1".to_string()),
        };
        let req = CallRequest::from_raw(raw).unwrap();
        assert!(req.from.is_some());
        assert!(req.to.is_some());
        assert_eq!(req.gas, Some(21000));
        assert_eq!(req.gas_price, Some(1_000_000_000));
        assert_eq!(req.nonce, Some(1));
    }

    #[test]
    fn test_call_request_from_raw_invalid_from() {
        let raw = CallRequestRaw {
            from: Some("invalid".to_string()),
            to: None,
            gas: None,
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            value: None,
            data: None,
            nonce: None,
        };
        let result = CallRequest::from_raw(raw);
        assert!(result.is_err());
    }

    // ===== BlockId Default Tests =====

    #[test]
    fn test_block_id_default() {
        assert_eq!(BlockId::default(), BlockId::Latest);
    }

    // ===== RpcBlock/RpcTransaction Serialization Tests =====

    #[test]
    fn test_rpc_block_serialize() {
        let block = RpcBlock {
            hash: "0x0".to_string(),
            parent_hash: "0x0".to_string(),
            sha3_uncles: "0x0".to_string(),
            miner: "0x0".to_string(),
            state_root: "0x0".to_string(),
            transactions_root: "0x0".to_string(),
            receipts_root: "0x0".to_string(),
            logs_bloom: "0x0".to_string(),
            difficulty: "0x0".to_string(),
            number: "0x1".to_string(),
            gas_limit: "0x1c9c380".to_string(),
            gas_used: "0x0".to_string(),
            timestamp: "0x0".to_string(),
            extra_data: "0x".to_string(),
            mix_hash: "0x0".to_string(),
            nonce: "0x0".to_string(),
            base_fee_per_gas: Some("0x3b9aca00".to_string()),
            total_difficulty: "0x0".to_string(),
            size: "0x0".to_string(),
            transactions: vec![],
            uncles: vec![],
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"number\":\"0x1\""));
        assert!(json.contains("\"baseFeePerGas\":\"0x3b9aca00\""));
    }

    #[test]
    fn test_rpc_transaction_serialize() {
        let tx = RpcTransaction {
            hash: "0x123".to_string(),
            nonce: "0x0".to_string(),
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
            to: Some("0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string()),
            value: "0x0".to_string(),
            gas: "0x5208".to_string(),
            gas_price: "0x3b9aca00".to_string(),
            input: "0x".to_string(),
            v: "0x1b".to_string(),
            r: "0x0".to_string(),
            s: "0x0".to_string(),
            tx_type: "0x0".to_string(),
            chain_id: Some("0x539".to_string()),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
        };
        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"hash\":\"0x123\""));
        assert!(json.contains("\"from\":"));
        assert!(json.contains("\"to\":"));
        // blockHash should be skipped when None
        assert!(!json.contains("\"blockHash\":null"));
    }

    #[test]
    fn test_rpc_receipt_serialize() {
        let receipt = RpcReceipt {
            transaction_hash: "0x123".to_string(),
            transaction_index: "0x0".to_string(),
            block_hash: "0xabc".to_string(),
            block_number: "0x1".to_string(),
            from: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
            to: Some("0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string()),
            cumulative_gas_used: "0x5208".to_string(),
            gas_used: "0x5208".to_string(),
            contract_address: None,
            logs: vec![],
            logs_bloom: "0x0".to_string(),
            tx_type: "0x0".to_string(),
            status: "0x1".to_string(),
            effective_gas_price: "0x3b9aca00".to_string(),
        };
        let json = serde_json::to_string(&receipt).unwrap();
        assert!(json.contains("\"status\":\"0x1\""));
        // contractAddress should be skipped when None
        assert!(!json.contains("\"contractAddress\":null"));
    }

    #[test]
    fn test_rpc_log_serialize() {
        let log = RpcLog {
            address: "0x123".to_string(),
            topics: vec!["0xabc".to_string()],
            data: "0x".to_string(),
            block_hash: "0x456".to_string(),
            block_number: "0x1".to_string(),
            transaction_hash: "0x789".to_string(),
            transaction_index: "0x0".to_string(),
            log_index: "0x0".to_string(),
            removed: false,
        };
        let json = serde_json::to_string(&log).unwrap();
        assert!(json.contains("\"address\":\"0x123\""));
        assert!(json.contains("\"removed\":false"));
    }
}
