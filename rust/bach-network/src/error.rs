//! Network error types

use thiserror::Error;

/// Errors that can occur in the network layer.
#[derive(Error, Debug)]
pub enum NetworkError {
    /// IO error during network operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Message encoding/decoding error
    #[error("Codec error: {0}")]
    Codec(String),

    /// Peer not found
    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Handshake failed
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    /// Maximum peer limit reached
    #[error("Maximum peers reached: {0}")]
    MaxPeersReached(usize),

    /// Invalid message received
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Genesis hash mismatch during handshake
    #[error("Genesis mismatch: expected {expected}, got {actual}")]
    GenesisMismatch { expected: String, actual: String },

    /// Protocol version mismatch
    #[error("Version mismatch: our version {our_version}, peer version {peer_version}")]
    VersionMismatch { our_version: u32, peer_version: u32 },

    /// Channel send error
    #[error("Channel send error")]
    ChannelSend,

    /// Service not running
    #[error("Service not running")]
    NotRunning,

    /// Already connected to peer
    #[error("Already connected to peer: {0}")]
    AlreadyConnected(String),
}

/// Result type for network operations.
pub type NetworkResult<T> = Result<T, NetworkError>;
