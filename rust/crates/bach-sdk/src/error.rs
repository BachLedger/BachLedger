//! SDK error types

use thiserror::Error;

/// SDK error type
#[derive(Debug, Error)]
pub enum SdkError {
    /// Transport/network error
    #[error("Transport error: {0}")]
    Transport(String),

    /// RPC error from node
    #[error("RPC error: {code} - {message}")]
    Rpc {
        /// Error code
        code: i64,
        /// Error message
        message: String,
    },

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid private key
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    /// Signing failed
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// ABI encoding error
    #[error("ABI encoding error: {0}")]
    AbiEncode(String),

    /// ABI decoding error
    #[error("ABI decoding error: {0}")]
    AbiDecode(String),

    /// Transaction build error
    #[error("Transaction build error: {0}")]
    TxBuild(String),

    /// Invalid hex string
    #[error("Invalid hex: {0}")]
    InvalidHex(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid chain ID
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(String),
}

impl From<hex::FromHexError> for SdkError {
    fn from(e: hex::FromHexError) -> Self {
        SdkError::InvalidHex(e.to_string())
    }
}

impl From<serde_json::Error> for SdkError {
    fn from(e: serde_json::Error) -> Self {
        SdkError::Serialization(e.to_string())
    }
}

impl From<bach_crypto::CryptoError> for SdkError {
    fn from(e: bach_crypto::CryptoError) -> Self {
        SdkError::SigningFailed(e.to_string())
    }
}

impl From<bach_primitives::PrimitiveError> for SdkError {
    fn from(e: bach_primitives::PrimitiveError) -> Self {
        SdkError::InvalidAddress(e.to_string())
    }
}
