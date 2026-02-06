//! Consensus error types

use bach_primitives::{Address, H256};
use thiserror::Error;

/// Consensus errors
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// Invalid block proposal
    #[error("invalid proposal: {0}")]
    InvalidProposal(String),

    /// Invalid vote
    #[error("invalid vote from {voter:?}: {reason}")]
    InvalidVote {
        /// Voter address
        voter: Address,
        /// Failure reason
        reason: String,
    },

    /// Not a validator
    #[error("not a validator: {0:?}")]
    NotValidator(Address),

    /// Duplicate vote
    #[error("duplicate vote from {0:?}")]
    DuplicateVote(Address),

    /// Invalid signature
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// Wrong height
    #[error("wrong height: expected {expected}, got {got}")]
    WrongHeight {
        /// Expected height
        expected: u64,
        /// Actual height
        got: u64,
    },

    /// Wrong round
    #[error("wrong round: expected {expected}, got {got}")]
    WrongRound {
        /// Expected round
        expected: u32,
        /// Actual round
        got: u32,
    },

    /// Block not found
    #[error("block not found: {0:?}")]
    BlockNotFound(H256),

    /// Timeout
    #[error("timeout at height {height}, round {round}")]
    Timeout {
        /// Height
        height: u64,
        /// Round
        round: u32,
    },

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result type for consensus operations
pub type ConsensusResult<T> = Result<T, ConsensusError>;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_address(n: u8) -> Address {
        Address::from_bytes([n; 20])
    }

    #[test]
    fn test_error_display_invalid_proposal() {
        let err = ConsensusError::InvalidProposal("bad block".to_string());
        assert!(format!("{}", err).contains("bad block"));
    }

    #[test]
    fn test_error_display_invalid_vote() {
        let err = ConsensusError::InvalidVote {
            voter: test_address(1),
            reason: "signature invalid".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("signature invalid"));
    }

    #[test]
    fn test_error_display_not_validator() {
        let err = ConsensusError::NotValidator(test_address(99));
        assert!(format!("{}", err).contains("not a validator"));
    }

    #[test]
    fn test_error_display_duplicate_vote() {
        let err = ConsensusError::DuplicateVote(test_address(1));
        assert!(format!("{}", err).contains("duplicate vote"));
    }

    #[test]
    fn test_error_display_invalid_signature() {
        let err = ConsensusError::InvalidSignature("bad sig".to_string());
        assert!(format!("{}", err).contains("bad sig"));
    }

    #[test]
    fn test_error_display_wrong_height() {
        let err = ConsensusError::WrongHeight { expected: 5, got: 3 };
        let msg = format!("{}", err);
        assert!(msg.contains("5"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn test_error_display_wrong_round() {
        let err = ConsensusError::WrongRound { expected: 2, got: 0 };
        let msg = format!("{}", err);
        assert!(msg.contains("2"));
        assert!(msg.contains("0"));
    }

    #[test]
    fn test_error_display_block_not_found() {
        let hash = H256::from_bytes([0x42; 32]);
        let err = ConsensusError::BlockNotFound(hash);
        assert!(format!("{}", err).contains("block not found"));
    }

    #[test]
    fn test_error_display_timeout() {
        let err = ConsensusError::Timeout { height: 10, round: 2 };
        let msg = format!("{}", err);
        assert!(msg.contains("10"));
        assert!(msg.contains("2"));
    }

    #[test]
    fn test_error_display_internal() {
        let err = ConsensusError::Internal("something failed".to_string());
        assert!(format!("{}", err).contains("something failed"));
    }
}
