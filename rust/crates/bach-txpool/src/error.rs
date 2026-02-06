//! Transaction pool error types

use thiserror::Error;

/// Transaction pool errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TxPoolError {
    /// Invalid signature
    #[error("invalid signature")]
    InvalidSignature,

    /// Nonce too low
    #[error("nonce too low: expected {expected}, got {got}")]
    NonceTooLow {
        /// Expected nonce
        expected: u64,
        /// Actual nonce
        got: u64,
    },

    /// Nonce gap too large
    #[error("nonce gap too large: current {current}, tx nonce {tx_nonce}")]
    NonceGapTooLarge {
        /// Current account nonce
        current: u64,
        /// Transaction nonce
        tx_nonce: u64,
    },

    /// Insufficient balance for gas + value
    #[error("insufficient balance: required {required}, available {available}")]
    InsufficientBalance {
        /// Required balance
        required: u128,
        /// Available balance
        available: u128,
    },

    /// Gas limit too low
    #[error("gas limit too low: {0}")]
    GasLimitTooLow(u64),

    /// Gas limit exceeds block limit
    #[error("gas limit exceeds block limit: {gas_limit} > {block_limit}")]
    GasLimitExceedsBlock {
        /// Transaction gas limit
        gas_limit: u64,
        /// Block gas limit
        block_limit: u64,
    },

    /// Gas price too low
    #[error("gas price too low: {0}")]
    GasPriceTooLow(u128),

    /// Transaction already exists
    #[error("transaction already exists: {0:?}")]
    AlreadyExists(bach_primitives::H256),

    /// Pool is full
    #[error("pool is full (max size: {0})")]
    PoolFull(usize),

    /// Transaction underpriced for replacement
    #[error("replacement transaction underpriced: old {old}, new {new}")]
    Underpriced {
        /// Old gas price
        old: u128,
        /// New gas price
        new: u128,
    },

    /// Failed to recover sender
    #[error("failed to recover sender: {0}")]
    RecoveryFailed(String),
}

/// Result type for transaction pool operations
pub type TxPoolResult<T> = Result<T, TxPoolError>;
