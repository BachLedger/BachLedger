//! Block execution error types

use bach_primitives::H256;
use thiserror::Error;

/// Block execution errors
#[derive(Debug, Error)]
pub enum ExecutionError {
    /// Invalid block structure
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    /// Invalid transaction
    #[error("invalid transaction {tx_hash:?}: {reason}")]
    InvalidTransaction {
        /// Transaction hash
        tx_hash: H256,
        /// Failure reason
        reason: String,
    },

    /// EVM execution error
    #[error("EVM error: {0}")]
    Evm(#[from] bach_evm::EvmError),

    /// Storage error
    #[error("storage error: {0}")]
    Storage(#[from] bach_storage::StorageError),

    /// Insufficient gas
    #[error("insufficient gas: required {required}, available {available}")]
    InsufficientGas {
        /// Required gas
        required: u64,
        /// Available gas
        available: u64,
    },

    /// Insufficient balance
    #[error("insufficient balance: required {required}, available {available}")]
    InsufficientBalance {
        /// Required balance
        required: u128,
        /// Available balance
        available: u128,
    },

    /// Nonce mismatch
    #[error("nonce mismatch: expected {expected}, got {got}")]
    NonceMismatch {
        /// Expected nonce
        expected: u64,
        /// Actual nonce
        got: u64,
    },

    /// Sender recovery failed
    #[error("sender recovery failed: {0}")]
    SenderRecovery(String),

    /// Block gas limit exceeded
    #[error("block gas limit exceeded: {used} > {limit}")]
    BlockGasLimitExceeded {
        /// Gas used
        used: u64,
        /// Block gas limit
        limit: u64,
    },

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result type for execution operations
pub type ExecutionResult<T> = Result<T, ExecutionError>;
