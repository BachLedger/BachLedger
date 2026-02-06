//! # bach-evm-tests
//!
//! Ethereum test suite integration for BachLedger EVM.
//!
//! This crate provides:
//! - JSON parsing for ethereum/tests format
//! - VMTest runner for opcode-level testing
//! - StateTest runner for transaction execution testing
//! - Test result aggregation and reporting
//!
//! ## Test Formats
//!
//! ### VMTests
//! Tests individual EVM operations with pre/post state checks.
//!
//! ### GeneralStateTests
//! Tests full transaction execution with state transitions.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod types;
mod vm_test;
mod state_test;
mod runner;

pub use error::{TestError, TestResult};
pub use types::*;
pub use vm_test::VmTestRunner;
pub use state_test::{StateTestRunner, SUPPORTED_FORKS};
pub use runner::{TestRunner, TestStats};
