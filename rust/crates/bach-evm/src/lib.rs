//! # bach-evm
//!
//! EVM execution engine for BachLedger.
//!
//! This crate provides:
//! - EVM bytecode interpreter
//! - Stack and memory management
//! - Gas metering
//! - Opcode implementations
//!
//! ## Architecture
//!
//! ```text
//! +------------------+
//! |   Interpreter    |  <- Main execution loop
//! +------------------+
//!          |
//! +--------+--------+
//! |  Stack | Memory |  <- Runtime state
//! +--------+--------+
//!          |
//! +------------------+
//! |    Environment   |  <- Call/Block/Tx context
//! +------------------+
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use bach_evm::{Interpreter, Environment};
//!
//! let code = vec![0x60, 0x01, 0x60, 0x02, 0x01, 0x00]; // PUSH1 1, PUSH1 2, ADD, STOP
//! let mut interp = Interpreter::new(code, 10000);
//! let env = Environment::default();
//! let result = interp.run(&env);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod opcode;
mod gas;
mod stack;
mod memory;
mod context;
mod interpreter;

pub use error::{EvmError, EvmResult, ExecutionResult, Log};
pub use opcode::Opcode;
pub use gas::{cost, static_gas, memory_gas, copy_gas, exp_gas, sha3_gas, log_gas};
pub use stack::{Stack, U256, U256_ZERO, U256_ONE, U256_MAX};
pub use stack::{u64_to_u256, u128_to_u256, u256_to_u64, u256_to_usize, u256_is_zero};
pub use stack::{u256_add, u256_sub, u256_and, u256_or, u256_xor, u256_not, u256_lt, u256_gt, u256_cmp};
pub use memory::Memory;
pub use context::{CallContext, BlockContext, TxContext, Environment};
pub use interpreter::Interpreter;
