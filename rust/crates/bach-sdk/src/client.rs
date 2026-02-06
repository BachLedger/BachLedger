//! BachClient - main RPC client

use bach_primitives::{Address, H256, U256};
use bach_types::{Block, Receipt, SignedTransaction, TransactionBody, TxType};
use bytes::Bytes;
use serde_json::Value;

use crate::transport::{deserialize_response, MockTransport, Transport};
use crate::types::{BlockId, CallRequest, PendingTransaction};
use crate::SdkError;

#[cfg(feature = "http")]
use crate::transport::HttpTransport;

/// BachLedger client for RPC communication
pub struct BachClient {
    transport: Box<dyn Transport>,
    chain_id: Option<u64>,
}

impl BachClient {
    /// Create a new client with HTTP transport
    #[cfg(feature = "http")]
    pub async fn connect(url: &str) -> Result<Self, SdkError> {
        let transport = HttpTransport::new(url);
        let mut client = Self {
            transport: Box::new(transport),
            chain_id: None,
        };

        // Fetch and cache chain ID
        let chain_id = client.fetch_chain_id().await?;
        client.chain_id = Some(chain_id);

        Ok(client)
    }

    /// Create a new client with mock transport (for testing)
    pub fn new_mock() -> Self {
        Self {
            transport: Box::new(MockTransport::new()),
            chain_id: Some(1),
        }
    }

    /// Create a client with a custom transport
    pub fn with_transport(transport: impl Transport + 'static) -> Self {
        Self {
            transport: Box::new(transport),
            chain_id: None,
        }
    }

    /// Helper method to make RPC request and deserialize
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Vec<Value>,
    ) -> Result<T, SdkError> {
        let value = self.transport.request_json(method, params).await?;
        deserialize_response(value)
    }

    // ==================== Chain Info ====================

    /// Get the chain ID
    pub async fn chain_id(&self) -> Result<u64, SdkError> {
        if let Some(id) = self.chain_id {
            return Ok(id);
        }
        self.fetch_chain_id().await
    }

    async fn fetch_chain_id(&self) -> Result<u64, SdkError> {
        let result: String = self.request("eth_chainId", vec![]).await?;
        parse_hex_u64(&result)
    }

    /// Get the current gas price
    pub async fn gas_price(&self) -> Result<u128, SdkError> {
        let result: String = self.request("eth_gasPrice", vec![]).await?;
        parse_hex_u128(&result)
    }

    /// Get the current block number
    pub async fn block_number(&self) -> Result<u64, SdkError> {
        let result: String = self.request("eth_blockNumber", vec![]).await?;
        parse_hex_u64(&result)
    }

    // ==================== Account Queries ====================

    /// Get the balance of an address
    pub async fn get_balance(&self, address: &Address, block: BlockId) -> Result<U256, SdkError> {
        let result: String = self
            .request(
                "eth_getBalance",
                vec![
                    Value::String(address.to_hex()),
                    serde_json::to_value(block)?,
                ],
            )
            .await?;
        parse_hex_u256(&result)
    }

    /// Get the nonce (transaction count) of an address
    pub async fn get_nonce(&self, address: &Address, block: BlockId) -> Result<u64, SdkError> {
        let result: String = self
            .request(
                "eth_getTransactionCount",
                vec![
                    Value::String(address.to_hex()),
                    serde_json::to_value(block)?,
                ],
            )
            .await?;
        parse_hex_u64(&result)
    }

    /// Get the code at an address
    pub async fn get_code(&self, address: &Address, block: BlockId) -> Result<Bytes, SdkError> {
        let result: String = self
            .request(
                "eth_getCode",
                vec![
                    Value::String(address.to_hex()),
                    serde_json::to_value(block)?,
                ],
            )
            .await?;
        parse_hex_bytes(&result)
    }

    // ==================== Block Queries ====================

    /// Get a block by number
    pub async fn get_block(&self, block: BlockId) -> Result<Option<Block>, SdkError> {
        let _result: Option<Value> = self
            .request(
                "eth_getBlockByNumber",
                vec![serde_json::to_value(block)?, Value::Bool(true)],
            )
            .await?;

        // TODO: Parse block from JSON
        // For now, return None as we need proper deserialization
        Ok(None)
    }

    /// Get a block by hash
    pub async fn get_block_by_hash(&self, hash: &H256) -> Result<Option<Block>, SdkError> {
        let _result: Option<Value> = self
            .request(
                "eth_getBlockByHash",
                vec![Value::String(hash.to_hex()), Value::Bool(true)],
            )
            .await?;

        // TODO: Parse block from JSON
        Ok(None)
    }

    // ==================== Transaction Queries ====================

    /// Get a transaction by hash
    pub async fn get_transaction(
        &self,
        hash: &H256,
    ) -> Result<Option<SignedTransaction>, SdkError> {
        let _result: Option<Value> = self
            .request("eth_getTransactionByHash", vec![Value::String(hash.to_hex())])
            .await?;

        // TODO: Parse transaction from JSON
        Ok(None)
    }

    /// Get a transaction receipt
    pub async fn get_receipt(&self, hash: &H256) -> Result<Option<Receipt>, SdkError> {
        let _result: Option<Value> = self
            .request(
                "eth_getTransactionReceipt",
                vec![Value::String(hash.to_hex())],
            )
            .await?;

        // TODO: Parse receipt from JSON
        Ok(None)
    }

    // ==================== Transaction Submission ====================

    /// Send a raw transaction (RLP-encoded bytes)
    pub async fn send_raw_transaction(&self, tx: &[u8]) -> Result<PendingTransaction, SdkError> {
        let hex = format!("0x{}", hex::encode(tx));
        let result: String = self
            .request("eth_sendRawTransaction", vec![Value::String(hex)])
            .await?;

        let hash = H256::from_hex(&result).map_err(|e| SdkError::InvalidHex(e.to_string()))?;
        Ok(PendingTransaction::new(hash))
    }

    /// Send a signed transaction
    pub async fn send_transaction(
        &self,
        tx: &SignedTransaction,
    ) -> Result<PendingTransaction, SdkError> {
        let encoded = encode_signed_transaction(tx);
        self.send_raw_transaction(&encoded).await
    }

    // ==================== Call & Estimation ====================

    /// Execute a call (read-only, does not create transaction)
    pub async fn call(&self, request: &CallRequest, block: BlockId) -> Result<Bytes, SdkError> {
        let result: String = self
            .request(
                "eth_call",
                vec![serde_json::to_value(request)?, serde_json::to_value(block)?],
            )
            .await?;
        parse_hex_bytes(&result)
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, request: &CallRequest) -> Result<u64, SdkError> {
        let result: String = self
            .request("eth_estimateGas", vec![serde_json::to_value(request)?])
            .await?;
        parse_hex_u64(&result)
    }
}

// ==================== Helper Functions ====================

/// RLP encode a signed transaction for sending to the network
fn encode_signed_transaction(tx: &SignedTransaction) -> Vec<u8> {
    match tx.tx_type {
        TxType::Legacy => encode_legacy_transaction(tx),
        TxType::DynamicFee => encode_eip1559_transaction(tx),
        _ => encode_legacy_transaction(tx), // Fallback for other types
    }
}

/// Encode a legacy (Type 0) transaction
fn encode_legacy_transaction(tx: &SignedTransaction) -> Vec<u8> {
    let body = match &tx.tx {
        TransactionBody::Legacy(legacy) => legacy,
        _ => return vec![], // Invalid state
    };

    // RLP encode: [nonce, gasPrice, gasLimit, to, value, data, v, r, s]
    let mut result = Vec::new();

    // Build the list items
    let nonce = encode_u64_minimal(body.nonce);
    let gas_price = encode_u128_minimal(body.gas_price);
    let gas_limit = encode_u64_minimal(body.gas_limit);
    let to = if let Some(addr) = &body.to {
        encode_bytes(addr.as_bytes())
    } else {
        vec![0x80] // Empty for contract creation
    };
    let value = encode_u128_minimal(body.value);
    let data = encode_bytes(&body.data);
    let v = encode_u64_minimal(tx.signature.v);
    let r = encode_bytes(tx.signature.r.as_bytes());
    let s = encode_bytes(tx.signature.s.as_bytes());

    // Calculate total payload length
    let payload_len = nonce.len() + gas_price.len() + gas_limit.len() + to.len()
        + value.len() + data.len() + v.len() + r.len() + s.len();

    // Add list header
    if payload_len < 56 {
        result.push(0xc0 + payload_len as u8);
    } else {
        let len_bytes = encode_length(payload_len);
        result.push(0xf7 + len_bytes.len() as u8);
        result.extend(len_bytes);
    }

    // Add items
    result.extend(nonce);
    result.extend(gas_price);
    result.extend(gas_limit);
    result.extend(to);
    result.extend(value);
    result.extend(data);
    result.extend(v);
    result.extend(r);
    result.extend(s);

    result
}

/// Encode an EIP-1559 (Type 2) transaction
fn encode_eip1559_transaction(tx: &SignedTransaction) -> Vec<u8> {
    let body = match &tx.tx {
        TransactionBody::DynamicFee(dynamic) => dynamic,
        _ => return vec![], // Invalid state
    };

    // Build the list: [chain_id, nonce, max_priority_fee, max_fee, gas_limit, to, value, data, access_list, v, r, s]
    let chain_id = encode_u64_minimal(body.chain_id);
    let nonce = encode_u64_minimal(body.nonce);
    let max_priority = encode_u128_minimal(body.max_priority_fee_per_gas);
    let max_fee = encode_u128_minimal(body.max_fee_per_gas);
    let gas_limit = encode_u64_minimal(body.gas_limit);
    let to = if let Some(addr) = &body.to {
        encode_bytes(addr.as_bytes())
    } else {
        vec![0x80]
    };
    let value = encode_u128_minimal(body.value);
    let data = encode_bytes(&body.data);
    let access_list = vec![0xc0]; // Empty access list
    let v = encode_u64_minimal(tx.signature.v);
    let r = encode_bytes(tx.signature.r.as_bytes());
    let s = encode_bytes(tx.signature.s.as_bytes());

    let payload_len = chain_id.len() + nonce.len() + max_priority.len() + max_fee.len()
        + gas_limit.len() + to.len() + value.len() + data.len() + access_list.len()
        + v.len() + r.len() + s.len();

    let mut inner = Vec::new();
    if payload_len < 56 {
        inner.push(0xc0 + payload_len as u8);
    } else {
        let len_bytes = encode_length(payload_len);
        inner.push(0xf7 + len_bytes.len() as u8);
        inner.extend(len_bytes);
    }

    inner.extend(chain_id);
    inner.extend(nonce);
    inner.extend(max_priority);
    inner.extend(max_fee);
    inner.extend(gas_limit);
    inner.extend(to);
    inner.extend(value);
    inner.extend(data);
    inner.extend(access_list);
    inner.extend(v);
    inner.extend(r);
    inner.extend(s);

    // EIP-1559 transactions are prefixed with 0x02
    let mut result = vec![0x02];
    result.extend(inner);
    result
}

/// Encode a u64 with minimal bytes (RLP encoding)
fn encode_u64_minimal(value: u64) -> Vec<u8> {
    if value == 0 {
        return vec![0x80];
    }
    if value < 128 {
        return vec![value as u8];
    }
    let bytes = value.to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(8);
    let len = 8 - start;
    let mut result = vec![0x80 + len as u8];
    result.extend_from_slice(&bytes[start..]);
    result
}

/// Encode a u128 with minimal bytes (RLP encoding)
fn encode_u128_minimal(value: u128) -> Vec<u8> {
    if value == 0 {
        return vec![0x80];
    }
    if value < 128 {
        return vec![value as u8];
    }
    let bytes = value.to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(16);
    let len = 16 - start;
    let mut result = vec![0x80 + len as u8];
    result.extend_from_slice(&bytes[start..]);
    result
}

/// Encode bytes with RLP length prefix
fn encode_bytes(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return vec![0x80];
    }
    if data.len() == 1 && data[0] < 128 {
        return data.to_vec();
    }
    if data.len() < 56 {
        let mut result = vec![0x80 + data.len() as u8];
        result.extend_from_slice(data);
        return result;
    }
    let len_bytes = encode_length(data.len());
    let mut result = vec![0xb7 + len_bytes.len() as u8];
    result.extend(len_bytes);
    result.extend_from_slice(data);
    result
}

/// Encode length as minimal big-endian bytes
fn encode_length(len: usize) -> Vec<u8> {
    let bytes = (len as u64).to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
    bytes[start..].to_vec()
}

fn parse_hex_u64(s: &str) -> Result<u64, SdkError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).map_err(|e| SdkError::InvalidHex(e.to_string()))
}

fn parse_hex_u128(s: &str) -> Result<u128, SdkError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u128::from_str_radix(s, 16).map_err(|e| SdkError::InvalidHex(e.to_string()))
}

fn parse_hex_u256(s: &str) -> Result<U256, SdkError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    // Pad to 64 hex chars (32 bytes)
    let padded = format!("{:0>64}", s);
    let bytes = hex::decode(&padded)?;
    Ok(U256::from_big_endian(&bytes))
}

fn parse_hex_bytes(s: &str) -> Result<Bytes, SdkError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.is_empty() {
        return Ok(Bytes::new());
    }
    let bytes = hex::decode(s)?;
    Ok(Bytes::from(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_mock_chain_id() {
        let client = BachClient::new_mock();
        let chain_id = client.chain_id().await.unwrap();
        assert_eq!(chain_id, 1);
    }

    #[tokio::test]
    async fn test_client_mock_gas_price() {
        let client = BachClient::new_mock();
        let gas_price = client.gas_price().await.unwrap();
        assert_eq!(gas_price, 1_000_000_000); // 1 gwei
    }

    #[tokio::test]
    async fn test_client_mock_block_number() {
        let client = BachClient::new_mock();
        let block_number = client.block_number().await.unwrap();
        assert_eq!(block_number, 256);
    }

    #[tokio::test]
    async fn test_client_mock_balance() {
        let client = BachClient::new_mock();
        let balance = client
            .get_balance(&Address::ZERO, BlockId::Latest)
            .await
            .unwrap();
        assert_eq!(balance, U256::from(1_000_000_000_000_000_000u128)); // 1 ETH
    }

    #[tokio::test]
    async fn test_client_mock_nonce() {
        let client = BachClient::new_mock();
        let nonce = client
            .get_nonce(&Address::ZERO, BlockId::Latest)
            .await
            .unwrap();
        assert_eq!(nonce, 0);
    }

    #[tokio::test]
    async fn test_client_mock_estimate_gas() {
        let client = BachClient::new_mock();
        let gas = client
            .estimate_gas(&CallRequest::default())
            .await
            .unwrap();
        assert_eq!(gas, 21000);
    }

    #[test]
    fn test_parse_hex_u64() {
        assert_eq!(parse_hex_u64("0x1").unwrap(), 1);
        assert_eq!(parse_hex_u64("0x100").unwrap(), 256);
        assert_eq!(parse_hex_u64("100").unwrap(), 256);
    }

    #[test]
    fn test_parse_hex_u128() {
        assert_eq!(parse_hex_u128("0x3b9aca00").unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_parse_hex_u256() {
        let result = parse_hex_u256("0xde0b6b3a7640000").unwrap();
        assert_eq!(result, U256::from(1_000_000_000_000_000_000u128));
    }

    #[test]
    fn test_parse_hex_bytes() {
        let result = parse_hex_bytes("0x1234").unwrap();
        assert_eq!(result.as_ref(), &[0x12, 0x34]);
    }

    #[test]
    fn test_parse_hex_bytes_empty() {
        let result = parse_hex_bytes("0x").unwrap();
        assert!(result.is_empty());
    }
}
