//! # bach-types
//!
//! Core blockchain types for BachLedger.
//!
//! This crate provides:
//! - [`Transaction`](transaction::SignedTransaction) - Signed transactions
//! - [`Block`](block::Block) - Block with header and body
//! - [`Receipt`](receipt::Receipt) - Transaction execution receipts

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod block;
pub mod codec;
pub mod receipt;
pub mod transaction;

// Re-export commonly used types
pub use block::{Block, BlockBody, BlockHeader, Bloom};
pub use receipt::{Log, Receipt, TxStatus};
pub use transaction::{
    AccessListItem, DynamicFeeTx, LegacyTx, SignedTransaction, TransactionBody, TxSignature,
    TxType,
};
