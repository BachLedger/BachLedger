//! Error types for the scheduler

use crate::state_key::TxId;
use thiserror::Error;

/// Scheduler errors
#[derive(Debug, Error)]
pub enum SchedulerError {
    /// Transaction not found
    #[error("transaction {0:?} not found")]
    TxNotFound(TxId),

    /// Circular dependency detected
    #[error("circular dependency detected involving transaction {0:?}")]
    CircularDependency(TxId),

    /// Ownership conflict
    #[error("ownership conflict: key owned by {owner:?}, requested by {requester:?}")]
    OwnershipConflict {
        /// Current owner
        owner: TxId,
        /// Transaction requesting ownership
        requester: TxId,
    },

    /// Maximum retries exceeded
    #[error("transaction {0:?} exceeded maximum retry count")]
    MaxRetriesExceeded(TxId),

    /// Invalid transaction index
    #[error("invalid transaction index: {0}")]
    InvalidIndex(usize),

    /// Execution aborted
    #[error("execution aborted: {0}")]
    Aborted(String),
}

/// Result type for scheduler operations
pub type SchedulerResult<T> = Result<T, SchedulerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SchedulerError::TxNotFound(TxId::new(42));
        assert!(err.to_string().contains("42"));

        let err = SchedulerError::CircularDependency(TxId::new(1));
        assert!(err.to_string().contains("circular"));

        let err = SchedulerError::OwnershipConflict {
            owner: TxId::new(1),
            requester: TxId::new(2),
        };
        assert!(err.to_string().contains("ownership"));
    }
}
