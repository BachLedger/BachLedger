//! Error types for EVM tests

use thiserror::Error;

/// Test error type
#[derive(Error, Debug)]
pub enum TestError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Hex decoding error
    #[error("Hex error: {0}")]
    Hex(String),

    /// Test fixture parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Test execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Assertion failed
    #[error("Assertion failed: {0}")]
    Assertion(String),

    /// Unsupported test type
    #[error("Unsupported: {0}")]
    Unsupported(String),
}

impl From<hex::FromHexError> for TestError {
    fn from(e: hex::FromHexError) -> Self {
        TestError::Hex(e.to_string())
    }
}

/// Test result type
pub type TestResult<T> = Result<T, TestError>;
