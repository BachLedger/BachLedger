//! BachLedger EVM - Ethereum Virtual Machine Implementation
//!
//! A complete EVM interpreter for the BachLedger medical blockchain.

#![forbid(unsafe_code)]

use bach_crypto::keccak256;
use bach_primitives::{Address, H256, U256};
use std::collections::HashMap;

// =============================================================================
// Constants
// =============================================================================

/// Maximum stack depth
pub const MAX_STACK_SIZE: usize = 1024;

/// Maximum code size (24KB per EIP-170)
pub const MAX_CODE_SIZE: usize = 24576;

/// Maximum call depth
pub const MAX_CALL_DEPTH: usize = 1024;

// Gas costs
pub const GAS_ZERO: u64 = 0;
pub const GAS_BASE: u64 = 2;
pub const GAS_VERY_LOW: u64 = 3;
pub const GAS_LOW: u64 = 5;
pub const GAS_MID: u64 = 8;
pub const GAS_HIGH: u64 = 10;
pub const GAS_JUMPDEST: u64 = 1;
pub const GAS_SLOAD: u64 = 200;
pub const GAS_SSTORE_SET: u64 = 20000;
pub const GAS_SSTORE_RESET: u64 = 5000;
pub const GAS_SSTORE_CLEAR_REFUND: u64 = 15000;
pub const GAS_SELFDESTRUCT: u64 = 5000;
pub const GAS_CREATE: u64 = 32000;
pub const GAS_CALL: u64 = 700;
pub const GAS_CALL_VALUE: u64 = 9000;
pub const GAS_CALL_NEW_ACCOUNT: u64 = 25000;
pub const GAS_EXP: u64 = 10;
pub const GAS_EXP_BYTE: u64 = 50;
pub const GAS_MEMORY: u64 = 3;
pub const GAS_COPY: u64 = 3;
pub const GAS_BLOCKHASH: u64 = 20;
pub const GAS_LOG: u64 = 375;
pub const GAS_LOG_DATA: u64 = 8;
pub const GAS_LOG_TOPIC: u64 = 375;
pub const GAS_KECCAK256: u64 = 30;
pub const GAS_KECCAK256_WORD: u64 = 6;
pub const GAS_BALANCE: u64 = 400;
pub const GAS_EXTCODE: u64 = 700;

// =============================================================================
// Opcodes
// =============================================================================

#[allow(dead_code)]
mod opcode {
    // Stop and Arithmetic
    pub const STOP: u8 = 0x00;
    pub const ADD: u8 = 0x01;
    pub const MUL: u8 = 0x02;
    pub const SUB: u8 = 0x03;
    pub const DIV: u8 = 0x04;
    pub const SDIV: u8 = 0x05;
    pub const MOD: u8 = 0x06;
    pub const SMOD: u8 = 0x07;
    pub const ADDMOD: u8 = 0x08;
    pub const MULMOD: u8 = 0x09;
    pub const EXP: u8 = 0x0A;
    pub const SIGNEXTEND: u8 = 0x0B;

    // Comparison & Bitwise Logic
    pub const LT: u8 = 0x10;
    pub const GT: u8 = 0x11;
    pub const SLT: u8 = 0x12;
    pub const SGT: u8 = 0x13;
    pub const EQ: u8 = 0x14;
    pub const ISZERO: u8 = 0x15;
    pub const AND: u8 = 0x16;
    pub const OR: u8 = 0x17;
    pub const XOR: u8 = 0x18;
    pub const NOT: u8 = 0x19;
    pub const BYTE: u8 = 0x1A;
    pub const SHL: u8 = 0x1B;
    pub const SHR: u8 = 0x1C;
    pub const SAR: u8 = 0x1D;

    // Keccak256
    pub const KECCAK256: u8 = 0x20;

    // Environmental Information
    pub const ADDRESS: u8 = 0x30;
    pub const BALANCE: u8 = 0x31;
    pub const ORIGIN: u8 = 0x32;
    pub const CALLER: u8 = 0x33;
    pub const CALLVALUE: u8 = 0x34;
    pub const CALLDATALOAD: u8 = 0x35;
    pub const CALLDATASIZE: u8 = 0x36;
    pub const CALLDATACOPY: u8 = 0x37;
    pub const CODESIZE: u8 = 0x38;
    pub const CODECOPY: u8 = 0x39;
    pub const GASPRICE: u8 = 0x3A;
    pub const EXTCODESIZE: u8 = 0x3B;
    pub const EXTCODECOPY: u8 = 0x3C;
    pub const RETURNDATASIZE: u8 = 0x3D;
    pub const RETURNDATACOPY: u8 = 0x3E;
    pub const EXTCODEHASH: u8 = 0x3F;

    // Block Information
    pub const BLOCKHASH: u8 = 0x40;
    pub const COINBASE: u8 = 0x41;
    pub const TIMESTAMP: u8 = 0x42;
    pub const NUMBER: u8 = 0x43;
    pub const DIFFICULTY: u8 = 0x44;
    pub const GASLIMIT: u8 = 0x45;
    pub const CHAINID: u8 = 0x46;
    pub const SELFBALANCE: u8 = 0x47;
    pub const BASEFEE: u8 = 0x48;

    // Stack, Memory, Storage and Flow Operations
    pub const POP: u8 = 0x50;
    pub const MLOAD: u8 = 0x51;
    pub const MSTORE: u8 = 0x52;
    pub const MSTORE8: u8 = 0x53;
    pub const SLOAD: u8 = 0x54;
    pub const SSTORE: u8 = 0x55;
    pub const JUMP: u8 = 0x56;
    pub const JUMPI: u8 = 0x57;
    pub const PC: u8 = 0x58;
    pub const MSIZE: u8 = 0x59;
    pub const GAS: u8 = 0x5A;
    pub const JUMPDEST: u8 = 0x5B;

    // Push Operations
    pub const PUSH0: u8 = 0x5F;
    pub const PUSH1: u8 = 0x60;
    pub const PUSH32: u8 = 0x7F;

    // Duplication Operations
    pub const DUP1: u8 = 0x80;
    pub const DUP16: u8 = 0x8F;

    // Exchange Operations
    pub const SWAP1: u8 = 0x90;
    pub const SWAP16: u8 = 0x9F;

    // Logging Operations
    pub const LOG0: u8 = 0xA0;
    pub const LOG1: u8 = 0xA1;
    pub const LOG2: u8 = 0xA2;
    pub const LOG3: u8 = 0xA3;
    pub const LOG4: u8 = 0xA4;

    // System Operations
    pub const CREATE: u8 = 0xF0;
    pub const CALL: u8 = 0xF1;
    pub const CALLCODE: u8 = 0xF2;
    pub const RETURN: u8 = 0xF3;
    pub const DELEGATECALL: u8 = 0xF4;
    pub const CREATE2: u8 = 0xF5;
    pub const STATICCALL: u8 = 0xFA;
    pub const REVERT: u8 = 0xFD;
    pub const INVALID: u8 = 0xFE;
    pub const SELFDESTRUCT: u8 = 0xFF;
}

// =============================================================================
// Error Types
// =============================================================================

/// EVM execution errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmError {
    /// Out of gas
    OutOfGas,
    /// Stack underflow
    StackUnderflow,
    /// Stack overflow
    StackOverflow,
    /// Invalid jump destination
    InvalidJump,
    /// Invalid opcode
    InvalidOpcode(u8),
    /// Invalid memory access
    InvalidMemoryAccess,
    /// Write in static context
    WriteInStaticContext,
    /// Call depth exceeded
    CallDepthExceeded,
    /// Insufficient balance
    InsufficientBalance,
    /// Contract creation failed
    CreateFailed,
    /// Return data out of bounds
    ReturnDataOutOfBounds,
    /// Code size limit exceeded
    CodeSizeExceeded,
    /// Invalid code (starts with 0xEF)
    InvalidCode,
    /// Revert with data
    Revert(Vec<u8>),
}

// =============================================================================
// Execution Result
// =============================================================================

/// Result of EVM execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Gas used
    pub gas_used: u64,
    /// Return data
    pub output: Vec<u8>,
    /// Error if any
    pub error: Option<EvmError>,
    /// Logs emitted
    pub logs: Vec<Log>,
}

/// A log entry emitted by LOG opcodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Log {
    /// Contract address that emitted the log
    pub address: Address,
    /// Indexed topics
    pub topics: Vec<H256>,
    /// Non-indexed data
    pub data: Vec<u8>,
}

// =============================================================================
// EVM Context
// =============================================================================

/// Execution context for EVM
#[derive(Debug, Clone)]
pub struct EvmContext {
    /// Transaction origin
    pub origin: Address,
    /// Current caller
    pub caller: Address,
    /// Current contract address
    pub address: Address,
    /// Call value
    pub value: U256,
    /// Call data
    pub data: Vec<u8>,
    /// Gas limit
    pub gas_limit: u64,
    /// Gas price
    pub gas_price: U256,
    /// Current block number
    pub block_number: u64,
    /// Block timestamp
    pub timestamp: u64,
    /// Block gas limit
    pub block_gas_limit: u64,
    /// Coinbase address
    pub coinbase: Address,
    /// Block difficulty
    pub difficulty: U256,
    /// Chain ID
    pub chain_id: u64,
    /// Base fee
    pub base_fee: U256,
    /// Is static call (no state modifications allowed)
    pub is_static: bool,
    /// Call depth
    pub depth: usize,
}

impl Default for EvmContext {
    fn default() -> Self {
        Self {
            origin: Address::zero(),
            caller: Address::zero(),
            address: Address::zero(),
            value: U256::ZERO,
            data: Vec::new(),
            gas_limit: 1_000_000,
            gas_price: U256::ZERO,
            block_number: 0,
            timestamp: 0,
            block_gas_limit: 30_000_000,
            coinbase: Address::zero(),
            difficulty: U256::ZERO,
            chain_id: 1,
            base_fee: U256::ZERO,
            is_static: false,
            depth: 0,
        }
    }
}

// =============================================================================
// Account and State
// =============================================================================

/// EVM account
#[derive(Debug, Clone, Default)]
pub struct Account {
    /// Account balance
    pub balance: U256,
    /// Account nonce
    pub nonce: u64,
    /// Contract code
    pub code: Vec<u8>,
    /// Storage
    pub storage: HashMap<H256, H256>,
}

/// EVM state
#[derive(Debug, Clone, Default)]
pub struct EvmState {
    /// Accounts
    accounts: HashMap<Address, Account>,
    /// Block hashes (block_number -> hash)
    block_hashes: HashMap<u64, H256>,
}

impl EvmState {
    /// Creates a new empty state
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets an account (creates empty one if doesn't exist)
    pub fn get_account(&self, address: &Address) -> Account {
        self.accounts.get(address).cloned().unwrap_or_default()
    }

    /// Gets mutable account reference
    pub fn get_account_mut(&mut self, address: &Address) -> &mut Account {
        self.accounts.entry(*address).or_default()
    }

    /// Sets account balance
    pub fn set_balance(&mut self, address: &Address, balance: U256) {
        self.get_account_mut(address).balance = balance;
    }

    /// Gets account balance
    pub fn get_balance(&self, address: &Address) -> U256 {
        self.accounts.get(address).map(|a| a.balance).unwrap_or(U256::ZERO)
    }

    /// Gets account code
    pub fn get_code(&self, address: &Address) -> Vec<u8> {
        self.accounts.get(address).map(|a| a.code.clone()).unwrap_or_default()
    }

    /// Sets account code
    pub fn set_code(&mut self, address: &Address, code: Vec<u8>) {
        self.get_account_mut(address).code = code;
    }

    /// Gets storage value
    pub fn get_storage(&self, address: &Address, key: &H256) -> H256 {
        self.accounts
            .get(address)
            .and_then(|a| a.storage.get(key))
            .copied()
            .unwrap_or(H256::zero())
    }

    /// Sets storage value
    pub fn set_storage(&mut self, address: &Address, key: H256, value: H256) {
        self.get_account_mut(address).storage.insert(key, value);
    }

    /// Gets account nonce
    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.accounts.get(address).map(|a| a.nonce).unwrap_or(0)
    }

    /// Increments account nonce
    pub fn increment_nonce(&mut self, address: &Address) {
        self.get_account_mut(address).nonce += 1;
    }

    /// Sets a block hash
    pub fn set_block_hash(&mut self, number: u64, hash: H256) {
        self.block_hashes.insert(number, hash);
    }

    /// Gets a block hash
    pub fn get_block_hash(&self, number: u64) -> H256 {
        self.block_hashes.get(&number).copied().unwrap_or(H256::zero())
    }

    /// Checks if account exists
    pub fn account_exists(&self, address: &Address) -> bool {
        self.accounts.contains_key(address)
    }

    /// Removes an account
    pub fn remove_account(&mut self, address: &Address) {
        self.accounts.remove(address);
    }

    /// Transfers value between accounts
    pub fn transfer(&mut self, from: &Address, to: &Address, value: U256) -> Result<(), EvmError> {
        let from_balance = self.get_balance(from);
        if from_balance < value {
            return Err(EvmError::InsufficientBalance);
        }

        let to_balance = self.get_balance(to);

        // Perform transfer
        self.set_balance(from, from_balance.checked_sub(&value).unwrap());
        self.set_balance(to, to_balance.checked_add(&value).unwrap_or(U256::MAX));

        Ok(())
    }
}

// =============================================================================
// EVM Interpreter
// =============================================================================

/// The EVM interpreter
pub struct Evm {
    /// Operand stack
    stack: Vec<U256>,
    /// Memory
    memory: Vec<u8>,
    /// Program counter
    pc: usize,
    /// Gas remaining
    gas_remaining: u64,
    /// Return data from last call
    returndata: Vec<u8>,
    /// Emitted logs
    logs: Vec<Log>,
    /// Valid jump destinations
    jumpdests: Vec<bool>,
}

impl Evm {
    /// Creates a new EVM instance
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(MAX_STACK_SIZE),
            memory: Vec::new(),
            pc: 0,
            gas_remaining: 0,
            returndata: Vec::new(),
            logs: Vec::new(),
            jumpdests: Vec::new(),
        }
    }

    /// Resets the EVM state for a new execution
    fn reset(&mut self, gas_limit: u64) {
        self.stack.clear();
        self.memory.clear();
        self.pc = 0;
        self.gas_remaining = gas_limit;
        self.returndata.clear();
        self.logs.clear();
        self.jumpdests.clear();
    }

    /// Analyzes code to find valid jump destinations
    fn analyze_jumpdests(&mut self, code: &[u8]) {
        self.jumpdests = vec![false; code.len()];
        let mut i = 0;
        while i < code.len() {
            let op = code[i];
            if op == opcode::JUMPDEST {
                self.jumpdests[i] = true;
            }
            // Skip PUSH data
            if op >= opcode::PUSH1 && op <= opcode::PUSH32 {
                let push_size = (op - opcode::PUSH1 + 1) as usize;
                i += push_size;
            }
            i += 1;
        }
    }

    // Stack operations
    fn push(&mut self, value: U256) -> Result<(), EvmError> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(EvmError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<U256, EvmError> {
        self.stack.pop().ok_or(EvmError::StackUnderflow)
    }

    fn peek(&self, n: usize) -> Result<U256, EvmError> {
        let len = self.stack.len();
        if n >= len {
            return Err(EvmError::StackUnderflow);
        }
        Ok(self.stack[len - 1 - n])
    }

    fn swap(&mut self, n: usize) -> Result<(), EvmError> {
        let len = self.stack.len();
        if n >= len {
            return Err(EvmError::StackUnderflow);
        }
        let top = len - 1;
        let other = len - 1 - n;
        self.stack.swap(top, other);
        Ok(())
    }

    fn dup(&mut self, n: usize) -> Result<(), EvmError> {
        let value = self.peek(n - 1)?;
        self.push(value)
    }

    // Gas operations
    fn use_gas(&mut self, gas: u64) -> Result<(), EvmError> {
        if self.gas_remaining < gas {
            return Err(EvmError::OutOfGas);
        }
        self.gas_remaining -= gas;
        Ok(())
    }

    // Memory operations
    fn memory_gas_cost(&self, offset: usize, size: usize) -> u64 {
        if size == 0 {
            return 0;
        }
        let new_size = offset.saturating_add(size);
        let new_words = (new_size + 31) / 32;
        let current_words = (self.memory.len() + 31) / 32;

        if new_words <= current_words {
            return 0;
        }

        let new_cost = (new_words as u64) * GAS_MEMORY + (new_words as u64).pow(2) / 512;
        let current_cost = (current_words as u64) * GAS_MEMORY + (current_words as u64).pow(2) / 512;

        new_cost.saturating_sub(current_cost)
    }

    fn expand_memory(&mut self, offset: usize, size: usize) {
        if size == 0 {
            return;
        }
        let new_size = offset.saturating_add(size);
        if new_size > self.memory.len() {
            self.memory.resize(new_size, 0);
        }
    }

    fn mload(&mut self, offset: usize) -> Result<U256, EvmError> {
        let gas = self.memory_gas_cost(offset, 32);
        self.use_gas(gas)?;
        self.expand_memory(offset, 32);

        let mut bytes = [0u8; 32];
        let end = (offset + 32).min(self.memory.len());
        let len = end.saturating_sub(offset);
        bytes[..len].copy_from_slice(&self.memory[offset..end]);

        Ok(U256::from_be_bytes(bytes))
    }

    fn mstore(&mut self, offset: usize, value: U256) -> Result<(), EvmError> {
        let gas = self.memory_gas_cost(offset, 32);
        self.use_gas(gas)?;
        self.expand_memory(offset, 32);

        let bytes = value.to_be_bytes();
        self.memory[offset..offset + 32].copy_from_slice(&bytes);
        Ok(())
    }

    fn mstore8(&mut self, offset: usize, value: u8) -> Result<(), EvmError> {
        let gas = self.memory_gas_cost(offset, 1);
        self.use_gas(gas)?;
        self.expand_memory(offset, 1);

        self.memory[offset] = value;
        Ok(())
    }

    /// Execute bytecode
    pub fn execute(
        &mut self,
        code: &[u8],
        context: &EvmContext,
        state: &mut EvmState,
    ) -> ExecutionResult {
        self.reset(context.gas_limit);
        self.analyze_jumpdests(code);

        let result = self.run(code, context, state);

        let gas_used = context.gas_limit.saturating_sub(self.gas_remaining);

        match result {
            Ok(output) => ExecutionResult {
                success: true,
                gas_used,
                output,
                error: None,
                logs: std::mem::take(&mut self.logs),
            },
            Err(EvmError::Revert(data)) => ExecutionResult {
                success: false,
                gas_used,
                output: data.clone(),
                error: Some(EvmError::Revert(data)),
                logs: Vec::new(),
            },
            Err(e) => ExecutionResult {
                success: false,
                gas_used: context.gas_limit, // Consume all gas on error
                output: Vec::new(),
                error: Some(e),
                logs: Vec::new(),
            },
        }
    }

    fn run(
        &mut self,
        code: &[u8],
        context: &EvmContext,
        state: &mut EvmState,
    ) -> Result<Vec<u8>, EvmError> {
        while self.pc < code.len() {
            let op = code[self.pc];
            self.pc += 1;

            match op {
                opcode::STOP => return Ok(Vec::new()),

                // Arithmetic
                opcode::ADD => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a.wrapping_add(&b))?;
                }
                opcode::MUL => {
                    self.use_gas(GAS_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a.wrapping_mul(&b))?;
                }
                opcode::SUB => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a.wrapping_sub(&b))?;
                }
                opcode::DIV => {
                    self.use_gas(GAS_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    if b.is_zero() {
                        self.push(U256::ZERO)?;
                    } else {
                        self.push(a.checked_div(&b).unwrap_or(U256::ZERO))?;
                    }
                }
                opcode::SDIV => {
                    self.use_gas(GAS_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(signed_div(a, b))?;
                }
                opcode::MOD => {
                    self.use_gas(GAS_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    if b.is_zero() {
                        self.push(U256::ZERO)?;
                    } else {
                        self.push(a.wrapping_mod(&b))?;
                    }
                }
                opcode::SMOD => {
                    self.use_gas(GAS_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(signed_mod(a, b))?;
                }
                opcode::ADDMOD => {
                    self.use_gas(GAS_MID)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    let n = self.pop()?;
                    if n.is_zero() {
                        self.push(U256::ZERO)?;
                    } else {
                        self.push(addmod(a, b, n))?;
                    }
                }
                opcode::MULMOD => {
                    self.use_gas(GAS_MID)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    let n = self.pop()?;
                    if n.is_zero() {
                        self.push(U256::ZERO)?;
                    } else {
                        self.push(mulmod(a, b, n))?;
                    }
                }
                opcode::EXP => {
                    let base = self.pop()?;
                    let exponent = self.pop()?;
                    let byte_size = byte_size(&exponent);
                    let gas = GAS_EXP + GAS_EXP_BYTE * byte_size as u64;
                    self.use_gas(gas)?;
                    self.push(exp(base, exponent))?;
                }
                opcode::SIGNEXTEND => {
                    self.use_gas(GAS_LOW)?;
                    let b = self.pop()?;
                    let x = self.pop()?;
                    self.push(signextend(b, x))?;
                }

                // Comparison
                opcode::LT => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(if a < b { U256::ONE } else { U256::ZERO })?;
                }
                opcode::GT => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(if a > b { U256::ONE } else { U256::ZERO })?;
                }
                opcode::SLT => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(if signed_lt(a, b) { U256::ONE } else { U256::ZERO })?;
                }
                opcode::SGT => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(if signed_gt(a, b) { U256::ONE } else { U256::ZERO })?;
                }
                opcode::EQ => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(if a == b { U256::ONE } else { U256::ZERO })?;
                }
                opcode::ISZERO => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    self.push(if a.is_zero() { U256::ONE } else { U256::ZERO })?;
                }

                // Bitwise
                opcode::AND => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a.bitand(&b))?;
                }
                opcode::OR => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a.bitor(&b))?;
                }
                opcode::XOR => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    let b = self.pop()?;
                    self.push(a.bitxor(&b))?;
                }
                opcode::NOT => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let a = self.pop()?;
                    self.push(a.bitnot())?;
                }
                opcode::BYTE => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let i = self.pop()?;
                    let x = self.pop()?;
                    self.push(byte_at(i, x))?;
                }
                opcode::SHL => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let shift = self.pop()?;
                    let value = self.pop()?;
                    self.push(shl(shift, value))?;
                }
                opcode::SHR => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let shift = self.pop()?;
                    let value = self.pop()?;
                    self.push(shr(shift, value))?;
                }
                opcode::SAR => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let shift = self.pop()?;
                    let value = self.pop()?;
                    self.push(sar(shift, value))?;
                }

                // Keccak256
                opcode::KECCAK256 => {
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();
                    let words = (size + 31) / 32;
                    let gas = GAS_KECCAK256 + GAS_KECCAK256_WORD * words as u64;
                    self.use_gas(gas)?;

                    let mem_gas = self.memory_gas_cost(offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(offset, size);

                    let data = if size > 0 {
                        &self.memory[offset..offset + size]
                    } else {
                        &[]
                    };
                    let hash = keccak256(data);
                    self.push(U256::from_be_bytes(*hash.as_bytes()))?;
                }

                // Environmental Information
                opcode::ADDRESS => {
                    self.use_gas(GAS_BASE)?;
                    self.push(address_to_u256(&context.address))?;
                }
                opcode::BALANCE => {
                    self.use_gas(GAS_BALANCE)?;
                    let addr = u256_to_address(self.pop()?);
                    let balance = state.get_balance(&addr);
                    self.push(balance)?;
                }
                opcode::ORIGIN => {
                    self.use_gas(GAS_BASE)?;
                    self.push(address_to_u256(&context.origin))?;
                }
                opcode::CALLER => {
                    self.use_gas(GAS_BASE)?;
                    self.push(address_to_u256(&context.caller))?;
                }
                opcode::CALLVALUE => {
                    self.use_gas(GAS_BASE)?;
                    self.push(context.value)?;
                }
                opcode::CALLDATALOAD => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let offset = self.pop()?.as_usize();
                    let mut bytes = [0u8; 32];
                    for i in 0..32 {
                        if offset + i < context.data.len() {
                            bytes[i] = context.data[offset + i];
                        }
                    }
                    self.push(U256::from_be_bytes(bytes))?;
                }
                opcode::CALLDATASIZE => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64(context.data.len() as u64))?;
                }
                opcode::CALLDATACOPY => {
                    let dest_offset = self.pop()?.as_usize();
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();

                    let words = (size + 31) / 32;
                    let gas = GAS_VERY_LOW + GAS_COPY * words as u64;
                    self.use_gas(gas)?;

                    let mem_gas = self.memory_gas_cost(dest_offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(dest_offset, size);

                    for i in 0..size {
                        let byte = if offset + i < context.data.len() {
                            context.data[offset + i]
                        } else {
                            0
                        };
                        self.memory[dest_offset + i] = byte;
                    }
                }
                opcode::CODESIZE => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64(code.len() as u64))?;
                }
                opcode::CODECOPY => {
                    let dest_offset = self.pop()?.as_usize();
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();

                    let words = (size + 31) / 32;
                    let gas = GAS_VERY_LOW + GAS_COPY * words as u64;
                    self.use_gas(gas)?;

                    let mem_gas = self.memory_gas_cost(dest_offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(dest_offset, size);

                    for i in 0..size {
                        let byte = if offset + i < code.len() {
                            code[offset + i]
                        } else {
                            0
                        };
                        self.memory[dest_offset + i] = byte;
                    }
                }
                opcode::GASPRICE => {
                    self.use_gas(GAS_BASE)?;
                    self.push(context.gas_price)?;
                }
                opcode::EXTCODESIZE => {
                    self.use_gas(GAS_EXTCODE)?;
                    let addr = u256_to_address(self.pop()?);
                    let code_size = state.get_code(&addr).len();
                    self.push(U256::from_u64(code_size as u64))?;
                }
                opcode::EXTCODECOPY => {
                    let addr = u256_to_address(self.pop()?);
                    let dest_offset = self.pop()?.as_usize();
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();

                    let words = (size + 31) / 32;
                    let gas = GAS_EXTCODE + GAS_COPY * words as u64;
                    self.use_gas(gas)?;

                    let mem_gas = self.memory_gas_cost(dest_offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(dest_offset, size);

                    let ext_code = state.get_code(&addr);
                    for i in 0..size {
                        let byte = if offset + i < ext_code.len() {
                            ext_code[offset + i]
                        } else {
                            0
                        };
                        self.memory[dest_offset + i] = byte;
                    }
                }
                opcode::RETURNDATASIZE => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64(self.returndata.len() as u64))?;
                }
                opcode::RETURNDATACOPY => {
                    let dest_offset = self.pop()?.as_usize();
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();

                    // Check bounds
                    if offset.checked_add(size).map_or(true, |end| end > self.returndata.len()) {
                        return Err(EvmError::ReturnDataOutOfBounds);
                    }

                    let words = (size + 31) / 32;
                    let gas = GAS_VERY_LOW + GAS_COPY * words as u64;
                    self.use_gas(gas)?;

                    let mem_gas = self.memory_gas_cost(dest_offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(dest_offset, size);

                    self.memory[dest_offset..dest_offset + size]
                        .copy_from_slice(&self.returndata[offset..offset + size]);
                }
                opcode::EXTCODEHASH => {
                    self.use_gas(GAS_EXTCODE)?;
                    let addr = u256_to_address(self.pop()?);
                    let code = state.get_code(&addr);
                    if code.is_empty() && state.get_balance(&addr).is_zero() && state.get_nonce(&addr) == 0 {
                        self.push(U256::ZERO)?;
                    } else {
                        let hash = keccak256(&code);
                        self.push(U256::from_be_bytes(*hash.as_bytes()))?;
                    }
                }

                // Block Information
                opcode::BLOCKHASH => {
                    self.use_gas(GAS_BLOCKHASH)?;
                    let block_num = self.pop()?.as_u64();
                    let hash = if block_num < context.block_number
                        && context.block_number - block_num <= 256
                    {
                        state.get_block_hash(block_num)
                    } else {
                        H256::zero()
                    };
                    self.push(U256::from_be_bytes(*hash.as_bytes()))?;
                }
                opcode::COINBASE => {
                    self.use_gas(GAS_BASE)?;
                    self.push(address_to_u256(&context.coinbase))?;
                }
                opcode::TIMESTAMP => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64(context.timestamp))?;
                }
                opcode::NUMBER => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64(context.block_number))?;
                }
                opcode::DIFFICULTY => {
                    self.use_gas(GAS_BASE)?;
                    self.push(context.difficulty)?;
                }
                opcode::GASLIMIT => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64(context.block_gas_limit))?;
                }
                opcode::CHAINID => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64(context.chain_id))?;
                }
                opcode::SELFBALANCE => {
                    self.use_gas(GAS_LOW)?;
                    let balance = state.get_balance(&context.address);
                    self.push(balance)?;
                }
                opcode::BASEFEE => {
                    self.use_gas(GAS_BASE)?;
                    self.push(context.base_fee)?;
                }

                // Stack, Memory, Storage and Flow Operations
                opcode::POP => {
                    self.use_gas(GAS_BASE)?;
                    self.pop()?;
                }
                opcode::MLOAD => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let offset = self.pop()?.as_usize();
                    let value = self.mload(offset)?;
                    self.push(value)?;
                }
                opcode::MSTORE => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let offset = self.pop()?.as_usize();
                    let value = self.pop()?;
                    self.mstore(offset, value)?;
                }
                opcode::MSTORE8 => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let offset = self.pop()?.as_usize();
                    let value = self.pop()?;
                    self.mstore8(offset, value.to_be_bytes()[31])?;
                }
                opcode::SLOAD => {
                    self.use_gas(GAS_SLOAD)?;
                    let key = self.pop()?;
                    let key_h256 = H256::from(key.to_be_bytes());
                    let value = state.get_storage(&context.address, &key_h256);
                    self.push(U256::from_be_bytes(*value.as_bytes()))?;
                }
                opcode::SSTORE => {
                    if context.is_static {
                        return Err(EvmError::WriteInStaticContext);
                    }

                    let key = self.pop()?;
                    let value = self.pop()?;
                    let key_h256 = H256::from(key.to_be_bytes());
                    let value_h256 = H256::from(value.to_be_bytes());

                    let current = state.get_storage(&context.address, &key_h256);

                    // Gas calculation (simplified)
                    let gas = if current.is_zero() && !value_h256.is_zero() {
                        GAS_SSTORE_SET
                    } else {
                        GAS_SSTORE_RESET
                    };
                    self.use_gas(gas)?;

                    state.set_storage(&context.address, key_h256, value_h256);
                }
                opcode::JUMP => {
                    self.use_gas(GAS_MID)?;
                    let dest = self.pop()?.as_usize();
                    if dest >= code.len() || !self.jumpdests[dest] {
                        return Err(EvmError::InvalidJump);
                    }
                    self.pc = dest;
                }
                opcode::JUMPI => {
                    self.use_gas(GAS_HIGH)?;
                    let dest = self.pop()?.as_usize();
                    let cond = self.pop()?;
                    if !cond.is_zero() {
                        if dest >= code.len() || !self.jumpdests[dest] {
                            return Err(EvmError::InvalidJump);
                        }
                        self.pc = dest;
                    }
                }
                opcode::PC => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64((self.pc - 1) as u64))?;
                }
                opcode::MSIZE => {
                    self.use_gas(GAS_BASE)?;
                    let size = ((self.memory.len() + 31) / 32) * 32;
                    self.push(U256::from_u64(size as u64))?;
                }
                opcode::GAS => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::from_u64(self.gas_remaining))?;
                }
                opcode::JUMPDEST => {
                    self.use_gas(GAS_JUMPDEST)?;
                    // No operation, just marks valid jump destination
                }

                // Push Operations
                opcode::PUSH0 => {
                    self.use_gas(GAS_BASE)?;
                    self.push(U256::ZERO)?;
                }
                op if op >= opcode::PUSH1 && op <= opcode::PUSH32 => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let n = (op - opcode::PUSH1 + 1) as usize;
                    let mut bytes = [0u8; 32];
                    let start = 32 - n;
                    for i in 0..n {
                        if self.pc + i < code.len() {
                            bytes[start + i] = code[self.pc + i];
                        }
                    }
                    self.pc += n;
                    self.push(U256::from_be_bytes(bytes))?;
                }

                // Dup Operations
                op if op >= opcode::DUP1 && op <= opcode::DUP16 => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let n = (op - opcode::DUP1 + 1) as usize;
                    self.dup(n)?;
                }

                // Swap Operations
                op if op >= opcode::SWAP1 && op <= opcode::SWAP16 => {
                    self.use_gas(GAS_VERY_LOW)?;
                    let n = (op - opcode::SWAP1 + 1) as usize;
                    self.swap(n)?;
                }

                // Log Operations
                op if op >= opcode::LOG0 && op <= opcode::LOG4 => {
                    if context.is_static {
                        return Err(EvmError::WriteInStaticContext);
                    }

                    let n = (op - opcode::LOG0) as usize;
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();

                    let mut topics = Vec::with_capacity(n);
                    for _ in 0..n {
                        let topic = self.pop()?;
                        topics.push(H256::from(topic.to_be_bytes()));
                    }

                    let gas = GAS_LOG + GAS_LOG_TOPIC * n as u64 + GAS_LOG_DATA * size as u64;
                    self.use_gas(gas)?;

                    let mem_gas = self.memory_gas_cost(offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(offset, size);

                    let data = self.memory[offset..offset + size].to_vec();

                    self.logs.push(Log {
                        address: context.address,
                        topics,
                        data,
                    });
                }

                // System Operations
                opcode::CREATE | opcode::CREATE2 => {
                    if context.is_static {
                        return Err(EvmError::WriteInStaticContext);
                    }
                    if context.depth >= MAX_CALL_DEPTH {
                        return Err(EvmError::CallDepthExceeded);
                    }

                    let value = self.pop()?;
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();

                    let salt = if op == opcode::CREATE2 {
                        Some(self.pop()?)
                    } else {
                        None
                    };

                    self.use_gas(GAS_CREATE)?;
                    let mem_gas = self.memory_gas_cost(offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(offset, size);

                    let init_code = self.memory[offset..offset + size].to_vec();

                    // Check code size limit
                    if init_code.len() > MAX_CODE_SIZE * 2 {
                        self.push(U256::ZERO)?;
                        continue;
                    }

                    // Calculate contract address
                    let new_address = if let Some(salt) = salt {
                        create2_address(&context.address, &salt, &init_code)
                    } else {
                        create_address(&context.address, state.get_nonce(&context.address))
                    };

                    // Transfer value
                    if !value.is_zero() {
                        if state.get_balance(&context.address) < value {
                            self.push(U256::ZERO)?;
                            continue;
                        }
                        state.transfer(&context.address, &new_address, value)?;
                    }

                    // Increment nonce for CREATE
                    if op == opcode::CREATE {
                        state.increment_nonce(&context.address);
                    }

                    // Execute init code
                    let mut create_context = context.clone();
                    create_context.caller = context.address;
                    create_context.address = new_address;
                    create_context.value = value;
                    create_context.data = Vec::new();
                    create_context.depth += 1;
                    create_context.gas_limit = self.gas_remaining - self.gas_remaining / 64;

                    let mut create_evm = Evm::new();
                    let result = create_evm.execute(&init_code, &create_context, state);

                    self.gas_remaining -= result.gas_used;
                    self.returndata = result.output.clone();

                    if result.success {
                        // Check returned code size
                        if result.output.len() > MAX_CODE_SIZE {
                            self.push(U256::ZERO)?;
                        } else if !result.output.is_empty() && result.output[0] == 0xEF {
                            // EIP-3541: reject code starting with 0xEF
                            self.push(U256::ZERO)?;
                        } else {
                            state.set_code(&new_address, result.output);
                            self.push(address_to_u256(&new_address))?;
                        }
                    } else {
                        self.push(U256::ZERO)?;
                    }
                }

                opcode::CALL | opcode::CALLCODE | opcode::DELEGATECALL | opcode::STATICCALL => {
                    if context.depth >= MAX_CALL_DEPTH {
                        return Err(EvmError::CallDepthExceeded);
                    }

                    let gas_limit = self.pop()?.as_u64();
                    let to = u256_to_address(self.pop()?);

                    let value = if op == opcode::CALL || op == opcode::CALLCODE {
                        self.pop()?
                    } else {
                        U256::ZERO
                    };

                    let args_offset = self.pop()?.as_usize();
                    let args_size = self.pop()?.as_usize();
                    let ret_offset = self.pop()?.as_usize();
                    let ret_size = self.pop()?.as_usize();

                    // Check static context for value transfer
                    if context.is_static && !value.is_zero() {
                        return Err(EvmError::WriteInStaticContext);
                    }

                    // Calculate gas
                    let mut gas = GAS_CALL;
                    if !value.is_zero() {
                        gas += GAS_CALL_VALUE;
                        if !state.account_exists(&to) {
                            gas += GAS_CALL_NEW_ACCOUNT;
                        }
                    }
                    self.use_gas(gas)?;

                    // Memory expansion
                    let mem_gas = self.memory_gas_cost(args_offset, args_size)
                        .max(self.memory_gas_cost(ret_offset, ret_size));
                    self.use_gas(mem_gas)?;
                    self.expand_memory(args_offset, args_size);
                    self.expand_memory(ret_offset, ret_size);

                    let input = self.memory[args_offset..args_offset + args_size].to_vec();

                    // Transfer value for CALL
                    if op == opcode::CALL && !value.is_zero() {
                        if state.get_balance(&context.address) < value {
                            self.push(U256::ZERO)?;
                            continue;
                        }
                        state.transfer(&context.address, &to, value)?;
                    }

                    // Get code to execute
                    let target_code = state.get_code(&to);

                    // Set up call context
                    let mut call_context = context.clone();
                    call_context.depth += 1;
                    call_context.data = input;

                    match op {
                        opcode::CALL => {
                            call_context.caller = context.address;
                            call_context.address = to;
                            call_context.value = value;
                        }
                        opcode::CALLCODE => {
                            call_context.caller = context.address;
                            call_context.address = context.address;
                            call_context.value = value;
                        }
                        opcode::DELEGATECALL => {
                            // Keep caller and value from parent context
                            call_context.address = context.address;
                        }
                        opcode::STATICCALL => {
                            call_context.caller = context.address;
                            call_context.address = to;
                            call_context.value = U256::ZERO;
                            call_context.is_static = true;
                        }
                        _ => unreachable!(),
                    }

                    let stipend = if !value.is_zero() { 2300 } else { 0 };
                    let available_gas = self.gas_remaining - self.gas_remaining / 64;
                    call_context.gas_limit = gas_limit.min(available_gas) + stipend;

                    // Execute call
                    let mut call_evm = Evm::new();
                    let result = call_evm.execute(&target_code, &call_context, state);

                    self.gas_remaining -= result.gas_used.saturating_sub(stipend);
                    self.returndata = result.output.clone();

                    // Copy return data to memory
                    let copy_size = ret_size.min(result.output.len());
                    for i in 0..copy_size {
                        self.memory[ret_offset + i] = result.output[i];
                    }

                    self.push(if result.success { U256::ONE } else { U256::ZERO })?;
                }

                opcode::RETURN => {
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();

                    let mem_gas = self.memory_gas_cost(offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(offset, size);

                    return Ok(self.memory[offset..offset + size].to_vec());
                }

                opcode::REVERT => {
                    let offset = self.pop()?.as_usize();
                    let size = self.pop()?.as_usize();

                    let mem_gas = self.memory_gas_cost(offset, size);
                    self.use_gas(mem_gas)?;
                    self.expand_memory(offset, size);

                    let data = self.memory[offset..offset + size].to_vec();
                    return Err(EvmError::Revert(data));
                }

                opcode::INVALID => {
                    return Err(EvmError::InvalidOpcode(op));
                }

                opcode::SELFDESTRUCT => {
                    if context.is_static {
                        return Err(EvmError::WriteInStaticContext);
                    }

                    self.use_gas(GAS_SELFDESTRUCT)?;

                    let beneficiary = u256_to_address(self.pop()?);
                    let balance = state.get_balance(&context.address);

                    // Transfer remaining balance
                    if !balance.is_zero() {
                        let ben_balance = state.get_balance(&beneficiary);
                        state.set_balance(&beneficiary, ben_balance.checked_add(&balance).unwrap_or(U256::MAX));
                    }

                    // Clear account
                    state.set_balance(&context.address, U256::ZERO);

                    return Ok(Vec::new());
                }

                _ => {
                    return Err(EvmError::InvalidOpcode(op));
                }
            }
        }

        Ok(Vec::new())
    }
}

impl Default for Evm {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Convert Address to U256
fn address_to_u256(addr: &Address) -> U256 {
    let mut bytes = [0u8; 32];
    bytes[12..32].copy_from_slice(addr.as_bytes());
    U256::from_be_bytes(bytes)
}

/// Convert U256 to Address
fn u256_to_address(val: U256) -> Address {
    let bytes = val.to_be_bytes();
    let mut addr_bytes = [0u8; 20];
    addr_bytes.copy_from_slice(&bytes[12..32]);
    Address::from(addr_bytes)
}

/// Create contract address (CREATE)
fn create_address(sender: &Address, nonce: u64) -> Address {
    // RLP encode [sender, nonce]
    let mut data = Vec::new();

    // Simple RLP encoding
    let sender_bytes = sender.as_bytes();
    let nonce_bytes = if nonce == 0 {
        vec![0x80]  // Empty byte for nonce 0
    } else {
        let mut n = nonce;
        let mut bytes = Vec::new();
        while n > 0 {
            bytes.push((n & 0xFF) as u8);
            n >>= 8;
        }
        bytes.reverse();
        if bytes.len() == 1 && bytes[0] < 0x80 {
            bytes
        } else {
            let mut result = vec![0x80 + bytes.len() as u8];
            result.extend(bytes);
            result
        }
    };

    let total_len = 21 + nonce_bytes.len();
    if total_len < 56 {
        data.push(0xC0 + total_len as u8);
    } else {
        // For longer lists (unlikely for address+nonce)
        data.push(0xF7 + 1);
        data.push(total_len as u8);
    }

    // Address (always 20 bytes)
    data.push(0x80 + 20);
    data.extend_from_slice(sender_bytes);

    // Nonce
    data.extend(nonce_bytes);

    let hash = keccak256(&data);
    let mut addr_bytes = [0u8; 20];
    addr_bytes.copy_from_slice(&hash.as_bytes()[12..32]);
    Address::from(addr_bytes)
}

/// Create2 contract address
fn create2_address(sender: &Address, salt: &U256, init_code: &[u8]) -> Address {
    let code_hash = keccak256(init_code);

    let mut data = Vec::with_capacity(1 + 20 + 32 + 32);
    data.push(0xFF);
    data.extend_from_slice(sender.as_bytes());
    data.extend_from_slice(&salt.to_be_bytes());
    data.extend_from_slice(code_hash.as_bytes());

    let hash = keccak256(&data);
    let mut addr_bytes = [0u8; 20];
    addr_bytes.copy_from_slice(&hash.as_bytes()[12..32]);
    Address::from(addr_bytes)
}

// =============================================================================
// Signed Arithmetic
// =============================================================================

fn signed_div(a: U256, b: U256) -> U256 {
    if b.is_zero() {
        return U256::ZERO;
    }

    let a_neg = a.is_negative();
    let b_neg = b.is_negative();

    let a_abs = if a_neg { a.twos_complement() } else { a };
    let b_abs = if b_neg { b.twos_complement() } else { b };

    let result = a_abs.checked_div(&b_abs).unwrap_or(U256::ZERO);

    if a_neg != b_neg && !result.is_zero() {
        result.twos_complement()
    } else {
        result
    }
}

fn signed_mod(a: U256, b: U256) -> U256 {
    if b.is_zero() {
        return U256::ZERO;
    }

    let a_neg = a.is_negative();

    let a_abs = if a_neg { a.twos_complement() } else { a };
    let b_abs = if b.is_negative() { b.twos_complement() } else { b };

    let result = a_abs.wrapping_mod(&b_abs);

    if a_neg && !result.is_zero() {
        result.twos_complement()
    } else {
        result
    }
}

fn signed_lt(a: U256, b: U256) -> bool {
    let a_neg = a.is_negative();
    let b_neg = b.is_negative();

    match (a_neg, b_neg) {
        (true, false) => true,   // negative < positive
        (false, true) => false,  // positive >= negative
        _ => a < b,              // same sign: compare normally
    }
}

fn signed_gt(a: U256, b: U256) -> bool {
    signed_lt(b, a)
}

fn signextend(b: U256, x: U256) -> U256 {
    let b_val = b.as_u64();
    if b_val >= 31 {
        return x;
    }

    let byte_pos = b_val as usize;
    let bit_pos = byte_pos * 8 + 7;
    let bytes = x.to_be_bytes();
    let sign_byte_idx = 31 - byte_pos;
    let sign_bit = (bytes[sign_byte_idx] >> 7) & 1;

    if sign_bit == 0 {
        // Clear upper bits
        let mut result_bytes = [0u8; 32];
        result_bytes[sign_byte_idx..].copy_from_slice(&bytes[sign_byte_idx..]);
        U256::from_be_bytes(result_bytes)
    } else {
        // Set upper bits to 1
        let mut result_bytes = [0xFFu8; 32];
        result_bytes[sign_byte_idx..].copy_from_slice(&bytes[sign_byte_idx..]);
        // Set bits above sign bit in sign byte
        let mask = 0xFF << ((bit_pos % 8) + 1);
        result_bytes[sign_byte_idx] |= mask as u8;
        U256::from_be_bytes(result_bytes)
    }
}

// =============================================================================
// Modular Arithmetic
// =============================================================================

fn addmod(a: U256, b: U256, n: U256) -> U256 {
    if n.is_zero() {
        return U256::ZERO;
    }

    // Use 512-bit arithmetic to avoid overflow
    let (sum, overflow) = add_512(a, b);
    mod_512(sum, overflow, n)
}

fn mulmod(a: U256, b: U256, n: U256) -> U256 {
    if n.is_zero() {
        return U256::ZERO;
    }

    // Use 512-bit arithmetic
    let product = mul_512(a, b);
    mod_512_full(product, n)
}

// 512-bit addition, returns (low 256 bits, high 256 bits as bool for overflow)
fn add_512(a: U256, b: U256) -> (U256, bool) {
    let a_limbs = a.limbs();
    let b_limbs = b.limbs();
    let mut result = [0u64; 4];
    let mut carry = 0u64;

    for i in 0..4 {
        let (sum1, c1) = a_limbs[i].overflowing_add(b_limbs[i]);
        let (sum2, c2) = sum1.overflowing_add(carry);
        result[i] = sum2;
        carry = (c1 as u64) + (c2 as u64);
    }

    (U256::from_limbs(result), carry != 0)
}

// 512-bit multiplication
fn mul_512(a: U256, b: U256) -> [u64; 8] {
    let a_limbs = a.limbs();
    let b_limbs = b.limbs();
    let mut result = [0u64; 8];

    for i in 0..4 {
        if a_limbs[i] == 0 {
            continue;
        }
        let mut carry: u64 = 0;
        for j in 0..4 {
            let product = (a_limbs[i] as u128) * (b_limbs[j] as u128)
                + (result[i + j] as u128)
                + (carry as u128);
            result[i + j] = product as u64;
            carry = (product >> 64) as u64;
        }
        result[i + 4] = carry;
    }

    result
}

// Modulo for 256-bit + overflow bit
fn mod_512(low: U256, overflow: bool, n: U256) -> U256 {
    if !overflow {
        return low.wrapping_mod(&n);
    }

    // Need to compute (low + 2^256) mod n
    // 2^256 mod n = (2^256 - n) mod n when n > 0
    let two_256_mod_n = U256::ZERO.wrapping_sub(&n).wrapping_mod(&n);
    let result = low.wrapping_mod(&n);
    result.wrapping_add(&two_256_mod_n).wrapping_mod(&n)
}

// Modulo for full 512-bit number
fn mod_512_full(value: [u64; 8], n: U256) -> U256 {
    let n_limbs = n.limbs();

    // Check if high part is zero
    if value[4] == 0 && value[5] == 0 && value[6] == 0 && value[7] == 0 {
        let low = U256::from_limbs([value[0], value[1], value[2], value[3]]);
        return low.wrapping_mod(&n);
    }

    // Use shift and subtract for full 512-bit mod
    let mut result = [0u64; 8];
    result.copy_from_slice(&value);

    // Find highest bit in result
    let mut high_bit = 0;
    for i in (0..8).rev() {
        if result[i] != 0 {
            high_bit = i * 64 + (64 - result[i].leading_zeros() as usize);
            break;
        }
    }

    // Find highest bit in n
    let n_bits = {
        let mut bits = 0;
        for i in (0..4).rev() {
            if n_limbs[i] != 0 {
                bits = i * 64 + (64 - n_limbs[i].leading_zeros() as usize);
                break;
            }
        }
        bits
    };

    if high_bit < n_bits {
        return U256::from_limbs([result[0], result[1], result[2], result[3]]);
    }

    // Shift n left and subtract
    for shift in (0..=(high_bit - n_bits)).rev() {
        // Check if result >= n << shift
        let mut shifted = [0u64; 8];
        let limb_shift = shift / 64;
        let bit_shift = shift % 64;

        for i in 0..4 {
            if i + limb_shift < 8 {
                shifted[i + limb_shift] = n_limbs[i] << bit_shift;
                if bit_shift > 0 && i + limb_shift + 1 < 8 && i > 0 {
                    shifted[i + limb_shift] |= n_limbs[i - 1] >> (64 - bit_shift);
                }
            }
        }
        if bit_shift > 0 && limb_shift < 8 {
            for i in (1..4).rev() {
                if i + limb_shift < 8 {
                    shifted[i + limb_shift] |= n_limbs[i - 1] >> (64 - bit_shift);
                }
            }
        }

        // Compare and subtract if result >= shifted
        let mut ge = true;
        for i in (0..8).rev() {
            if result[i] < shifted[i] {
                ge = false;
                break;
            } else if result[i] > shifted[i] {
                break;
            }
        }

        if ge {
            let mut borrow = 0u64;
            for i in 0..8 {
                let (diff1, b1) = result[i].overflowing_sub(shifted[i]);
                let (diff2, b2) = diff1.overflowing_sub(borrow);
                result[i] = diff2;
                borrow = (b1 as u64) + (b2 as u64);
            }
        }
    }

    U256::from_limbs([result[0], result[1], result[2], result[3]])
}

// =============================================================================
// Other Helpers
// =============================================================================

fn exp(base: U256, exponent: U256) -> U256 {
    if exponent.is_zero() {
        return U256::ONE;
    }
    if base.is_zero() {
        return U256::ZERO;
    }
    if base == U256::ONE {
        return U256::ONE;
    }

    let mut result = U256::ONE;
    let mut base = base;
    let mut exp = exponent;

    while !exp.is_zero() {
        if exp.limbs()[0] & 1 == 1 {
            result = result.wrapping_mul(&base);
        }
        base = base.wrapping_mul(&base);
        exp = exp.shr(1);
    }

    result
}

fn byte_size(val: &U256) -> usize {
    let bytes = val.to_be_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b != 0 {
            return 32 - i;
        }
    }
    0
}

fn byte_at(i: U256, x: U256) -> U256 {
    let i_val = i.as_u64();
    if i_val >= 32 {
        return U256::ZERO;
    }
    let bytes = x.to_be_bytes();
    U256::from_u64(bytes[i_val as usize] as u64)
}

fn shl(shift: U256, value: U256) -> U256 {
    let shift_val = shift.as_u64();
    if shift_val >= 256 {
        return U256::ZERO;
    }
    value.shl(shift_val as usize)
}

fn shr(shift: U256, value: U256) -> U256 {
    let shift_val = shift.as_u64();
    if shift_val >= 256 {
        return U256::ZERO;
    }
    value.shr(shift_val as usize)
}

fn sar(shift: U256, value: U256) -> U256 {
    let shift_val = shift.as_u64();
    if shift_val >= 256 {
        return if value.is_negative() {
            U256::MAX
        } else {
            U256::ZERO
        };
    }

    if !value.is_negative() {
        return value.shr(shift_val as usize);
    }

    // Arithmetic shift right for negative numbers
    let shifted = value.shr(shift_val as usize);
    // Fill high bits with 1s
    let mask = U256::MAX.shl(256 - shift_val as usize);
    shifted.bitor(&mask)
}

// =============================================================================
// Public API
// =============================================================================

/// Execute EVM bytecode
pub fn execute(code: &[u8], context: EvmContext, state: &mut EvmState) -> ExecutionResult {
    let mut evm = Evm::new();
    evm.execute(code, &context, state)
}

/// Deploy a contract
pub fn deploy_contract(
    code: &[u8],
    context: EvmContext,
    state: &mut EvmState,
) -> Result<Address, EvmError> {
    let sender = context.caller;
    let nonce = state.get_nonce(&sender);
    let contract_address = create_address(&sender, nonce);

    // Increment sender nonce
    state.increment_nonce(&sender);

    // Transfer value if any
    if !context.value.is_zero() {
        state.transfer(&sender, &contract_address, context.value)?;
    }

    // Execute init code
    let mut deploy_context = context;
    deploy_context.address = contract_address;

    let mut evm = Evm::new();
    let result = evm.execute(code, &deploy_context, state);

    if !result.success {
        return Err(result.error.unwrap_or(EvmError::CreateFailed));
    }

    // Check code size limit
    if result.output.len() > MAX_CODE_SIZE {
        return Err(EvmError::CodeSizeExceeded);
    }

    // Check for EIP-3541 (reject code starting with 0xEF)
    if !result.output.is_empty() && result.output[0] == 0xEF {
        return Err(EvmError::InvalidCode);
    }

    // Store the deployed code
    state.set_code(&contract_address, result.output);

    Ok(contract_address)
}

/// Call a contract
pub fn call_contract(
    address: Address,
    data: &[u8],
    context: EvmContext,
    state: &mut EvmState,
) -> ExecutionResult {
    let code = state.get_code(&address);

    let mut call_context = context;
    call_context.address = address;
    call_context.data = data.to_vec();

    let mut evm = Evm::new();
    evm.execute(&code, &call_context, state)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop() {
        let code = vec![opcode::STOP];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert!(result.output.is_empty());
    }

    #[test]
    fn test_push_pop() {
        // PUSH1 0x42, PUSH1 0x01, ADD, PUSH1 0x00, MSTORE, PUSH1 0x20, PUSH1 0x00, RETURN
        let code = vec![
            opcode::PUSH1, 0x42,
            opcode::PUSH1, 0x01,
            opcode::ADD,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.output.len(), 32);
        assert_eq!(result.output[31], 0x43); // 0x42 + 0x01
    }

    #[test]
    fn test_arithmetic() {
        // Test SUB: PUSH1 5, PUSH1 10, SUB -> 5
        let code = vec![
            opcode::PUSH1, 5,
            opcode::PUSH1, 10,
            opcode::SUB,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.output[31], 5);
    }

    #[test]
    fn test_comparison() {
        // Test LT: PUSH1 10, PUSH1 5, LT -> 1 (5 < 10)
        let code = vec![
            opcode::PUSH1, 10,
            opcode::PUSH1, 5,
            opcode::LT,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.output[31], 1);
    }

    #[test]
    fn test_storage() {
        // SSTORE then SLOAD
        let code = vec![
            opcode::PUSH1, 0x42,     // value
            opcode::PUSH1, 0x00,     // key
            opcode::SSTORE,
            opcode::PUSH1, 0x00,     // key
            opcode::SLOAD,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.output[31], 0x42);
    }

    #[test]
    fn test_jump() {
        // PUSH1 0x05, JUMP, INVALID, JUMPDEST, PUSH1 0x42, ...
        let code = vec![
            opcode::PUSH1, 0x04,     // offset 0: jump to 4
            opcode::JUMP,            // offset 2
            opcode::INVALID,         // offset 3: should be skipped
            opcode::JUMPDEST,        // offset 4
            opcode::PUSH1, 0x42,     // offset 5
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.output[31], 0x42);
    }

    #[test]
    fn test_revert() {
        let code = vec![
            opcode::PUSH1, 0x04,     // size
            opcode::PUSH1, 0x00,     // offset
            opcode::REVERT,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(!result.success);
        assert!(matches!(result.error, Some(EvmError::Revert(_))));
    }

    #[test]
    fn test_caller_address() {
        let code = vec![
            opcode::CALLER,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];

        let mut context = EvmContext::default();
        context.caller = Address::from_hex("0x1234567890123456789012345678901234567890").unwrap();
        let mut state = EvmState::new();

        let result = execute(&code, context.clone(), &mut state);
        assert!(result.success);

        // Check that caller address is in output
        let mut expected = [0u8; 32];
        expected[12..32].copy_from_slice(context.caller.as_bytes());
        assert_eq!(result.output, expected);
    }

    #[test]
    fn test_keccak256() {
        // Hash empty data
        let code = vec![
            opcode::PUSH1, 0x00,     // size
            opcode::PUSH1, 0x00,     // offset
            opcode::KECCAK256,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);

        // keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        let expected = keccak256(&[]);
        assert_eq!(&result.output[..], expected.as_bytes());
    }

    #[test]
    fn test_dup_swap() {
        // PUSH1 1, PUSH1 2, DUP2 -> stack: [1, 2, 1]
        // SWAP1 -> stack: [1, 1, 2]
        let code = vec![
            opcode::PUSH1, 1,
            opcode::PUSH1, 2,
            opcode::DUP1 + 1,        // DUP2
            opcode::SWAP1,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.output[31], 2);
    }

    #[test]
    fn test_log() {
        let code = vec![
            opcode::PUSH1, 0x42,     // data
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH32,          // topic
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            opcode::PUSH1, 0x20,     // size
            opcode::PUSH1, 0x00,     // offset
            opcode::LOG1,
            opcode::STOP,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.logs.len(), 1);
        assert_eq!(result.logs[0].topics.len(), 1);
    }

    #[test]
    fn test_out_of_gas() {
        let code = vec![
            opcode::PUSH1, 0x42,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
        ];

        let mut context = EvmContext::default();
        context.gas_limit = 1; // Very low gas
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(!result.success);
        assert!(matches!(result.error, Some(EvmError::OutOfGas)));
    }

    #[test]
    fn test_stack_underflow() {
        let code = vec![opcode::ADD]; // No values on stack
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(!result.success);
        assert!(matches!(result.error, Some(EvmError::StackUnderflow)));
    }

    #[test]
    fn test_invalid_jump() {
        let code = vec![
            opcode::PUSH1, 0xFF,     // Invalid destination
            opcode::JUMP,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(!result.success);
        assert!(matches!(result.error, Some(EvmError::InvalidJump)));
    }

    #[test]
    fn test_deploy_and_call() {
        // Simple contract that returns 0x42
        // PUSH1 0x42, PUSH1 0x00, MSTORE, PUSH1 0x20, PUSH1 0x00, RETURN
        let runtime_code = vec![
            opcode::PUSH1, 0x42,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];

        // Init code that returns the runtime code
        let mut init_code = vec![
            opcode::PUSH1, runtime_code.len() as u8,  // size
            opcode::PUSH1, 0x0C,                       // offset of runtime code
            opcode::PUSH1, 0x00,                       // destOffset
            opcode::CODECOPY,
            opcode::PUSH1, runtime_code.len() as u8,  // size
            opcode::PUSH1, 0x00,                       // offset
            opcode::RETURN,
        ];
        init_code.extend(&runtime_code);

        let mut context = EvmContext::default();
        context.caller = Address::from_hex("0x1234567890123456789012345678901234567890").unwrap();
        let mut state = EvmState::new();
        state.set_balance(&context.caller, U256::from_u64(1_000_000));

        // Deploy
        let contract_addr = deploy_contract(&init_code, context.clone(), &mut state).unwrap();

        // Verify code was stored
        let stored_code = state.get_code(&contract_addr);
        assert_eq!(stored_code, runtime_code);

        // Call the contract
        let call_result = call_contract(contract_addr, &[], context, &mut state);
        assert!(call_result.success);
        assert_eq!(call_result.output[31], 0x42);
    }

    #[test]
    fn test_bitwise_operations() {
        // Test AND: 0xFF AND 0x0F = 0x0F
        let code = vec![
            opcode::PUSH1, 0x0F,
            opcode::PUSH1, 0xFF,
            opcode::AND,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.output[31], 0x0F);
    }

    #[test]
    fn test_shift_operations() {
        // Test SHL: 1 << 4 = 16
        let code = vec![
            opcode::PUSH1, 0x01,     // value
            opcode::PUSH1, 0x04,     // shift amount
            opcode::SHL,
            opcode::PUSH1, 0x00,
            opcode::MSTORE,
            opcode::PUSH1, 0x20,
            opcode::PUSH1, 0x00,
            opcode::RETURN,
        ];
        let context = EvmContext::default();
        let mut state = EvmState::new();

        let result = execute(&code, context, &mut state);
        assert!(result.success);
        assert_eq!(result.output[31], 16);
    }
}
