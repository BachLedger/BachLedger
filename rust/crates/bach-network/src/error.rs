//! Network error types

use thiserror::Error;

/// Network errors
#[derive(Debug, Error)]
pub enum NetworkError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Connection failed
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Peer not found
    #[error("peer not found: {0}")]
    PeerNotFound(String),

    /// Already connected
    #[error("already connected to peer: {0}")]
    AlreadyConnected(String),

    /// Invalid message
    #[error("invalid message: {0}")]
    InvalidMessage(String),

    /// Protocol error
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Timeout
    #[error("timeout: {0}")]
    Timeout(String),

    /// Not running
    #[error("network service not running")]
    NotRunning,

    /// Already running
    #[error("network service already running")]
    AlreadyRunning,

    /// Send error
    #[error("send error: {0}")]
    Send(String),

    /// Channel closed
    #[error("channel closed")]
    ChannelClosed,
}

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: NetworkError = io_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("IO error"));
    }

    #[test]
    fn test_error_connection_failed() {
        let err = NetworkError::ConnectionFailed("timeout".into());
        let msg = format!("{}", err);
        assert!(msg.contains("connection failed"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_error_peer_not_found() {
        let err = NetworkError::PeerNotFound("abc123".into());
        let msg = format!("{}", err);
        assert!(msg.contains("peer not found"));
        assert!(msg.contains("abc123"));
    }

    #[test]
    fn test_error_already_connected() {
        let err = NetworkError::AlreadyConnected("peer1".into());
        let msg = format!("{}", err);
        assert!(msg.contains("already connected"));
        assert!(msg.contains("peer1"));
    }

    #[test]
    fn test_error_invalid_message() {
        let err = NetworkError::InvalidMessage("bad format".into());
        let msg = format!("{}", err);
        assert!(msg.contains("invalid message"));
        assert!(msg.contains("bad format"));
    }

    #[test]
    fn test_error_protocol() {
        let err = NetworkError::Protocol("version mismatch".into());
        let msg = format!("{}", err);
        assert!(msg.contains("protocol error"));
        assert!(msg.contains("version mismatch"));
    }

    #[test]
    fn test_error_timeout() {
        let err = NetworkError::Timeout("connect".into());
        let msg = format!("{}", err);
        assert!(msg.contains("timeout"));
        assert!(msg.contains("connect"));
    }

    #[test]
    fn test_error_not_running() {
        let err = NetworkError::NotRunning;
        let msg = format!("{}", err);
        assert!(msg.contains("not running"));
    }

    #[test]
    fn test_error_already_running() {
        let err = NetworkError::AlreadyRunning;
        let msg = format!("{}", err);
        assert!(msg.contains("already running"));
    }

    #[test]
    fn test_error_send() {
        let err = NetworkError::Send("queue full".into());
        let msg = format!("{}", err);
        assert!(msg.contains("send error"));
        assert!(msg.contains("queue full"));
    }

    #[test]
    fn test_error_channel_closed() {
        let err = NetworkError::ChannelClosed;
        let msg = format!("{}", err);
        assert!(msg.contains("channel closed"));
    }

    #[test]
    fn test_error_debug() {
        let err = NetworkError::NotRunning;
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotRunning"));
    }

    #[test]
    fn test_network_result_ok() {
        let result: NetworkResult<u32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_network_result_err() {
        let result: NetworkResult<u32> = Err(NetworkError::ChannelClosed);
        assert!(result.is_err());
    }
}
