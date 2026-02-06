//! # bach-storage
//!
//! Storage layer for BachLedger using RocksDB.
//!
//! This crate provides:
//! - Key-value storage abstraction over RocksDB
//! - State database for accounts and contract storage
//! - Block database for headers, bodies, and receipts
//! - Batch write support for atomic commits
//! - In-memory caching layer for efficient state access
//!
//! ## Architecture
//!
//! ```text
//! +------------------+
//! |   CachedState    |  <- In-memory write cache
//! +------------------+
//!          |
//! +------------------+
//! |     StateDb      |  <- Account/Storage/Code access
//! +------------------+
//!          |
//! +------------------+
//! |    Database      |  <- RocksDB wrapper
//! +------------------+
//! ```
//!
//! ## Column Families
//!
//! - `accounts` - Account state (nonce, balance, code_hash, storage_root)
//! - `storage` - Contract storage (address + slot -> value)
//! - `code` - Contract bytecode (code_hash -> bytecode)
//! - `headers` - Block headers (hash -> header)
//! - `bodies` - Block bodies (hash -> body)
//! - `receipts` - Transaction receipts (hash -> receipts)
//! - `block_index` - Block number to hash mapping
//! - `meta` - Database metadata

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod traits;
mod db;
mod state;
mod block;

pub use error::{StorageError, StorageResult};
pub use traits::{Account, StateReader, StateWriter, State, EMPTY_CODE_HASH, EMPTY_STORAGE_ROOT};
pub use db::{Database, DbConfig, WriteBatchWrapper, cf, ALL_CFS};
pub use state::{StateDb, StateCache, CachedState};
pub use block::BlockDb;
