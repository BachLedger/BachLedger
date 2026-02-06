//! # bach-e2e
//!
//! End-to-end integration testing framework for BachLedger.
//!
//! ## Design Philosophy
//!
//! 1. **Simple**: One command to run all tests
//! 2. **Declarative**: Tests describe WHAT, not HOW
//! 3. **Isolated**: Each test gets fresh state
//! 4. **Fast**: No network, no disk (use tempdir)
//!
//! ## Usage
//!
//! ```ignore
//! cargo test -p bach-e2e
//! ```

mod harness;
mod builder;
pub mod contracts;
mod scenarios;

pub use harness::{TestHarness, TestAccount, ReceiptAssertions};
pub use builder::{TxBuilder, EtherDenom, ExecutionResult};

/// Test result
pub type E2EResult<T> = Result<T, E2EError>;

/// E2E test errors
#[derive(Debug, thiserror::Error)]
pub enum E2EError {
    /// Setup failed
    #[error("setup failed: {0}")]
    Setup(String),

    /// Transaction failed
    #[error("transaction failed: {0}")]
    Transaction(String),

    /// Assertion failed
    #[error("assertion failed: {0}")]
    Assertion(String),

    /// Storage error
    #[error("storage error: {0}")]
    Storage(#[from] bach_storage::StorageError),

    /// EVM error
    #[error("evm error: {0}")]
    Evm(String),
}
