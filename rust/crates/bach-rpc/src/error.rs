//! RPC error types

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// Standard JSON-RPC 2.0 error codes
pub mod error_code {
    /// Parse error: Invalid JSON was received
    pub const PARSE_ERROR: i64 = -32700;
    /// Invalid Request: The JSON is not a valid Request object
    pub const INVALID_REQUEST: i64 = -32600;
    /// Method not found
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// Invalid params
    pub const INVALID_PARAMS: i64 = -32602;
    /// Internal error
    pub const INTERNAL_ERROR: i64 = -32603;

    // Ethereum-specific error codes
    /// Execution error (revert)
    pub const EXECUTION_ERROR: i64 = 3;
    /// Transaction rejected
    pub const TRANSACTION_REJECTED: i64 = -32003;
    /// Resource not found
    pub const RESOURCE_NOT_FOUND: i64 = -32001;
    /// Resource unavailable
    pub const RESOURCE_UNAVAILABLE: i64 = -32002;
}

/// JSON-RPC error response
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i64,
    /// Error message
    pub message: String,
    /// Optional additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create error with additional data
    pub fn with_data(code: i64, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Parse error
    pub fn parse_error() -> Self {
        Self::new(error_code::PARSE_ERROR, "Parse error")
    }

    /// Invalid request
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(error_code::INVALID_REQUEST, message)
    }

    /// Method not found
    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            error_code::METHOD_NOT_FOUND,
            format!("method not found: {}", method),
        )
    }

    /// Invalid params
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(error_code::INVALID_PARAMS, message)
    }

    /// Internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(error_code::INTERNAL_ERROR, message)
    }

    /// Execution error (for eth_call/eth_estimateGas)
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::new(error_code::EXECUTION_ERROR, message)
    }

    /// Transaction rejected
    pub fn transaction_rejected(message: impl Into<String>) -> Self {
        Self::new(error_code::TRANSACTION_REJECTED, message)
    }

    /// Resource not found
    pub fn resource_not_found(message: impl Into<String>) -> Self {
        Self::new(error_code::RESOURCE_NOT_FOUND, message)
    }
}

/// RPC server errors
#[derive(Debug, Error)]
pub enum RpcError {
    /// Server bind error
    #[error("failed to bind server: {0}")]
    Bind(#[from] std::io::Error),

    /// JSON serialization error
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Storage error
    #[error("storage error: {0}")]
    Storage(String),

    /// Execution error
    #[error("execution error: {0}")]
    Execution(String),

    /// Invalid parameter
    #[error("invalid parameter: {0}")]
    InvalidParam(String),
}

impl From<bach_storage::StorageError> for RpcError {
    fn from(e: bach_storage::StorageError) -> Self {
        RpcError::Storage(e.to_string())
    }
}

impl From<bach_core::ExecutionError> for RpcError {
    fn from(e: bach_core::ExecutionError) -> Self {
        RpcError::Execution(e.to_string())
    }
}

/// Result type for RPC operations
pub type RpcResult<T> = Result<T, RpcError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Error Code Tests =====

    #[test]
    fn test_error_codes() {
        assert_eq!(error_code::PARSE_ERROR, -32700);
        assert_eq!(error_code::INVALID_REQUEST, -32600);
        assert_eq!(error_code::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_code::INVALID_PARAMS, -32602);
        assert_eq!(error_code::INTERNAL_ERROR, -32603);
        assert_eq!(error_code::EXECUTION_ERROR, 3);
        assert_eq!(error_code::TRANSACTION_REJECTED, -32003);
        assert_eq!(error_code::RESOURCE_NOT_FOUND, -32001);
        assert_eq!(error_code::RESOURCE_UNAVAILABLE, -32002);
    }

    // ===== JsonRpcError Construction Tests =====

    #[test]
    fn test_json_rpc_error_new() {
        let err = JsonRpcError::new(-32000, "custom error");
        assert_eq!(err.code, -32000);
        assert_eq!(err.message, "custom error");
        assert!(err.data.is_none());
    }

    #[test]
    fn test_json_rpc_error_with_data() {
        let data = serde_json::json!({"key": "value"});
        let err = JsonRpcError::with_data(-32000, "custom error", data.clone());
        assert_eq!(err.code, -32000);
        assert_eq!(err.message, "custom error");
        assert_eq!(err.data, Some(data));
    }

    #[test]
    fn test_json_rpc_error_parse_error() {
        let err = JsonRpcError::parse_error();
        assert_eq!(err.code, error_code::PARSE_ERROR);
        assert_eq!(err.message, "Parse error");
    }

    #[test]
    fn test_json_rpc_error_invalid_request() {
        let err = JsonRpcError::invalid_request("test message");
        assert_eq!(err.code, error_code::INVALID_REQUEST);
        assert_eq!(err.message, "test message");
    }

    #[test]
    fn test_json_rpc_error_method_not_found() {
        let err = JsonRpcError::method_not_found("eth_unknown");
        assert_eq!(err.code, error_code::METHOD_NOT_FOUND);
        assert!(err.message.contains("eth_unknown"));
    }

    #[test]
    fn test_json_rpc_error_invalid_params() {
        let err = JsonRpcError::invalid_params("missing address");
        assert_eq!(err.code, error_code::INVALID_PARAMS);
        assert_eq!(err.message, "missing address");
    }

    #[test]
    fn test_json_rpc_error_internal_error() {
        let err = JsonRpcError::internal_error("database failure");
        assert_eq!(err.code, error_code::INTERNAL_ERROR);
        assert_eq!(err.message, "database failure");
    }

    #[test]
    fn test_json_rpc_error_execution_error() {
        let err = JsonRpcError::execution_error("revert");
        assert_eq!(err.code, error_code::EXECUTION_ERROR);
        assert_eq!(err.message, "revert");
    }

    #[test]
    fn test_json_rpc_error_transaction_rejected() {
        let err = JsonRpcError::transaction_rejected("nonce too low");
        assert_eq!(err.code, error_code::TRANSACTION_REJECTED);
        assert_eq!(err.message, "nonce too low");
    }

    #[test]
    fn test_json_rpc_error_resource_not_found() {
        let err = JsonRpcError::resource_not_found("block not found");
        assert_eq!(err.code, error_code::RESOURCE_NOT_FOUND);
        assert_eq!(err.message, "block not found");
    }

    // ===== JsonRpcError Serialization Tests =====

    #[test]
    fn test_json_rpc_error_serialize_without_data() {
        let err = JsonRpcError::invalid_params("test");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"code\":-32602"));
        assert!(json.contains("\"message\":\"test\""));
        assert!(!json.contains("\"data\""));
    }

    #[test]
    fn test_json_rpc_error_serialize_with_data() {
        let err = JsonRpcError::with_data(-32000, "error", serde_json::json!("extra info"));
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"data\":\"extra info\""));
    }

    // ===== RpcError Tests =====

    #[test]
    fn test_rpc_error_display_storage() {
        let err = RpcError::Storage("database error".to_string());
        assert!(err.to_string().contains("storage error"));
        assert!(err.to_string().contains("database error"));
    }

    #[test]
    fn test_rpc_error_display_execution() {
        let err = RpcError::Execution("out of gas".to_string());
        assert!(err.to_string().contains("execution error"));
        assert!(err.to_string().contains("out of gas"));
    }

    #[test]
    fn test_rpc_error_display_invalid_param() {
        let err = RpcError::InvalidParam("invalid address".to_string());
        assert!(err.to_string().contains("invalid parameter"));
        assert!(err.to_string().contains("invalid address"));
    }

    #[test]
    fn test_rpc_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: RpcError = io_err.into();
        assert!(matches!(err, RpcError::Bind(_)));
    }

    #[test]
    fn test_rpc_error_from_serde() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: RpcError = json_err.into();
        assert!(matches!(err, RpcError::Serialization(_)));
    }
}
