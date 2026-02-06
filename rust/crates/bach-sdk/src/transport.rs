//! Transport layer for RPC communication

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::SdkError;

/// Transport trait for RPC communication (object-safe)
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send an RPC request and get JSON response
    async fn request_json(
        &self,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Value, SdkError>;
}

/// Helper to deserialize response
pub fn deserialize_response<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, SdkError> {
    serde_json::from_value(value).map_err(|e| SdkError::Serialization(e.to_string()))
}

/// Mock transport for testing
pub struct MockTransport {
    responses: Arc<Mutex<HashMap<String, Value>>>,
    default_responses: Arc<Mutex<HashMap<String, Value>>>,
}

impl MockTransport {
    /// Create a new mock transport
    pub fn new() -> Self {
        let mut defaults = HashMap::new();

        // Default responses for common methods
        defaults.insert("eth_chainId".to_string(), Value::String("0x1".to_string()));
        defaults.insert("eth_gasPrice".to_string(), Value::String("0x3b9aca00".to_string())); // 1 gwei
        defaults.insert("eth_blockNumber".to_string(), Value::String("0x100".to_string())); // Block 256
        defaults.insert("eth_getBalance".to_string(), Value::String("0xde0b6b3a7640000".to_string())); // 1 ETH
        defaults.insert("eth_getTransactionCount".to_string(), Value::String("0x0".to_string()));
        defaults.insert("eth_estimateGas".to_string(), Value::String("0x5208".to_string())); // 21000
        defaults.insert("eth_sendRawTransaction".to_string(), Value::String(
            "0x88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b".to_string()
        ));
        defaults.insert("eth_call".to_string(), Value::String("0x".to_string()));
        defaults.insert("eth_getCode".to_string(), Value::String("0x".to_string()));

        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            default_responses: Arc::new(Mutex::new(defaults)),
        }
    }

    /// Set a mock response for a specific method
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned (only possible if another thread panicked while holding the lock).
    pub fn set_response(&self, method: &str, response: Value) {
        // Using expect here is acceptable as mutex poisoning indicates a serious bug
        // that should not be silently ignored in tests
        self.responses
            .lock()
            .expect("MockTransport mutex poisoned")
            .insert(method.to_string(), response);
    }

    /// Clear custom responses
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned.
    pub fn clear_responses(&self) {
        self.responses
            .lock()
            .expect("MockTransport mutex poisoned")
            .clear();
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn request_json(
        &self,
        method: &str,
        _params: Vec<Value>,
    ) -> Result<Value, SdkError> {
        // Check custom responses first
        let custom_response = self
            .responses
            .lock()
            .map_err(|_| SdkError::Transport("MockTransport mutex poisoned".to_string()))?
            .get(method)
            .cloned();

        if let Some(response) = custom_response {
            return Ok(response);
        }

        // Fall back to defaults
        let default_response = self
            .default_responses
            .lock()
            .map_err(|_| SdkError::Transport("MockTransport mutex poisoned".to_string()))?
            .get(method)
            .cloned();

        if let Some(response) = default_response {
            return Ok(response);
        }

        Err(SdkError::Rpc {
            code: -32601,
            message: format!("Method not found: {}", method),
        })
    }
}

/// HTTP transport for real RPC communication
#[cfg(feature = "http")]
pub struct HttpTransport {
    client: reqwest::Client,
    url: String,
    request_id: std::sync::atomic::AtomicU64,
}

#[cfg(feature = "http")]
impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(feature = "http")]
#[async_trait]
impl Transport for HttpTransport {
    async fn request_json(
        &self,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Value, SdkError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": method,
            "params": params,
        });

        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SdkError::Transport(e.to_string()))?;

        let response: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| SdkError::Transport(e.to_string()))?;

        if let Some(error) = response.error {
            return Err(SdkError::Rpc {
                code: error.code,
                message: error.message,
            });
        }

        response.result.ok_or_else(|| SdkError::Rpc {
            code: -32603,
            message: "No result in response".to_string(),
        })
    }
}

#[cfg(feature = "http")]
#[derive(serde::Deserialize)]
struct JsonRpcResponse {
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[cfg(feature = "http")]
#[derive(serde::Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_transport_default_responses() {
        let transport = MockTransport::new();

        let result = transport
            .request_json("eth_chainId", vec![])
            .await
            .unwrap();
        assert_eq!(result, Value::String("0x1".to_string()));

        let result = transport
            .request_json("eth_gasPrice", vec![])
            .await
            .unwrap();
        assert_eq!(result, Value::String("0x3b9aca00".to_string()));
    }

    #[tokio::test]
    async fn test_mock_transport_custom_response() {
        let transport = MockTransport::new();
        transport.set_response("eth_chainId", Value::String("0x5".to_string()));

        let result = transport
            .request_json("eth_chainId", vec![])
            .await
            .unwrap();
        assert_eq!(result, Value::String("0x5".to_string()));
    }

    #[tokio::test]
    async fn test_mock_transport_unknown_method() {
        let transport = MockTransport::new();
        let result = transport
            .request_json("unknown_method", vec![])
            .await;
        assert!(result.is_err());
    }
}
