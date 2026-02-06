//! # bach-scheduler
//!
//! Seamless Scheduling algorithm for BachLedger.
//!
//! This crate implements the novel scheduling algorithm that enables
//! parallel transaction execution while maintaining determinism.
//!
//! ## Architecture
//!
//! The Seamless Scheduling algorithm consists of several key components:
//!
//! - **StateKey**: Identifies individual state slots (address + storage slot)
//! - **OwnershipTable**: Tracks which transaction owns each state key
//! - **RWSet**: Tracks read/write sets for conflict detection
//! - **DependencyGraph**: Builds transaction dependency relationships
//! - **Scheduler**: Orchestrates parallel execution batches
//!
//! ## Algorithm Overview
//!
//! 1. **Pre-execution analysis**: Analyze transactions to predict state access
//! 2. **Dependency graph construction**: Build DAG based on state conflicts
//! 3. **Batch generation**: Group independent transactions for parallel execution
//! 4. **Execution**: Execute batches in topological order
//! 5. **Validation**: Verify no conflicts occurred during execution

#![warn(missing_docs)]
#![warn(clippy::all)]

mod state_key;
mod ownership;
mod rw_set;
mod dependency;
mod scheduler;
mod error;

pub use state_key::{StateKey, TxId};
pub use ownership::OwnershipTable;
pub use rw_set::{RWSet, ConflictSet};
pub use dependency::{DependencyGraph, DependencyType};
pub use scheduler::{Scheduler, ExecutionBatch, ScheduleResult};
pub use error::{SchedulerError, SchedulerResult};
