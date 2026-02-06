//! # bach-txpool
//!
//! Transaction pool for BachLedger.
//!
//! This crate provides:
//! - Transaction validation and ordering
//! - Pending/queued transaction separation
//! - Nonce gap handling
//! - Transaction replacement with gas price bumps
//! - Pool size limits and eviction
//!
//! ## Architecture
//!
//! ```text
//! +------------------+
//! |     TxPool       |
//! +------------------+
//!          |
//! +--------+---------+
//! | Pending | Queued |  <- Per-account tx organization
//! +--------+---------+
//!          |
//! +------------------+
//! |   By Hash Index  |  <- Fast lookup by tx hash
//! +------------------+
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use bach_txpool::{TxPool, PoolConfig};
//!
//! let pool = TxPool::with_defaults();
//! pool.add(tx, sender, hash)?;
//! let pending = pool.get_pending(100);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod pool;

pub use error::{TxPoolError, TxPoolResult};
pub use pool::{PoolConfig, PooledTransaction, TxPool};
