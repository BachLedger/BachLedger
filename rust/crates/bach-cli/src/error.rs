//! CLI error types

use thiserror::Error;

/// CLI error type
#[derive(Debug, Error)]
pub enum CliError {
    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid private key
    #[error("Invalid private key: {0}")]
    InvalidKey(String),

    /// Invalid hex string
    #[error("Invalid hex: {0}")]
    InvalidHex(String),

    /// Invalid amount
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// SDK error
    #[error("SDK error: {0}")]
    Sdk(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Config error
    #[error("Config error: {0}")]
    Config(String),

    /// Cryptographic error
    #[error("Crypto error: {0}")]
    Crypto(String),
}
