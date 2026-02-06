//! # bach-core
//!
//! Core orchestration for BachLedger.
//!
//! This crate ties together all components:
//! - Block execution pipeline
//! - Transaction execution flow
//! - State management
//! - Component coordination
//!
//! ## Architecture
//!
//! ```text
//! +-------------------+
//! |   BlockExecutor   |  <- Executes blocks
//! +-------------------+
//!          |
//! +--------+--------+
//! |  EVM   | State  |  <- Execution + Storage
//! +--------+--------+
//!          |
//! +-------------------+
//! |     Receipts      |  <- Results
//! +-------------------+
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod executor;

pub use error::{ExecutionError, ExecutionResult};
pub use executor::{BlockExecutionResult, BlockExecutor};
