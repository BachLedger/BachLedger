//! Cryptographic errors

use thiserror::Error;

/// Cryptographic operation error
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Signing failed
    #[error("signing failed: {0}")]
    SigningFailed(String),

    /// Invalid signature
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid recovery ID
    #[error("invalid recovery id: {0}")]
    InvalidRecoveryId(u8),

    /// Recovery failed
    #[error("public key recovery failed: {0}")]
    RecoveryFailed(String),

    /// Invalid private key
    #[error("invalid private key")]
    InvalidPrivateKey,
}
