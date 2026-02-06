//! Storage error types

use thiserror::Error;

/// Storage errors
#[derive(Debug, Error)]
pub enum StorageError {
    /// RocksDB error
    #[error("rocksdb error: {0}")]
    RocksDb(#[from] rocksdb::Error),

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Key not found
    #[error("key not found")]
    NotFound,

    /// Invalid column family
    #[error("invalid column family: {0}")]
    InvalidColumnFamily(String),

    /// Database not open
    #[error("database not open")]
    NotOpen,

    /// Database already open
    #[error("database already open")]
    AlreadyOpen,

    /// Invalid data format
    #[error("invalid data format: {0}")]
    InvalidFormat(String),

    /// IO error
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;
