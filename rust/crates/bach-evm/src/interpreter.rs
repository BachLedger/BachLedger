//! EVM bytecode interpreter

use crate::context::Environment;
use crate::error::{EvmError, EvmResult, ExecutionResult, Log};
use crate::gas;
use crate::memory::Memory;
use crate::opcode::Opcode;
use crate::stack::{self, Stack, U256, U256_ONE, U256_ZERO};
use bach_crypto::keccak256;
use bach_primitives::{Address, H256};
use std::collections::{HashMap, HashSet};

/// Trait for EVM state access (storage, balance, code)
pub trait StateAccess {
    /// Read storage value
    fn get_storage(&self, address: &Address, key: &H256) -> H256;
    /// Write storage value
    fn set_storage(&mut self, address: Address, key: H256, value: H256);
    /// Get account balance
    fn get_balance(&self, address: &Address) -> u128;
    /// Get account code
    fn get_code(&self, address: &Address) -> Vec<u8>;
    /// Get account code size
    fn get_code_size(&self, address: &Address) -> usize {
        self.get_code(address).len()
    }
    /// Get account code hash
    fn get_code_hash(&self, address: &Address) -> H256;
    /// Check if account exists
    fn account_exists(&self, address: &Address) -> bool;
    /// Transfer value between accounts
    fn transfer(&mut self, from: &Address, to: &Address, value: u128) -> Result<(), EvmError>;
    /// Get account nonce
    fn get_nonce(&self, address: &Address) -> u64;
    /// Increment nonce and return the old value
    fn increment_nonce(&mut self, address: &Address) -> u64;
    /// Mark an address in the access list (warm)
    fn mark_warm(&mut self, address: &Address);
    /// Check if address is warm
    fn is_warm(&self, address: &Address) -> bool;
    /// Mark a storage slot as warm
    fn mark_storage_warm(&mut self, address: &Address, key: &H256);
    /// Check if storage slot is warm
    fn is_storage_warm(&self, address: &Address, key: &H256) -> bool;
    /// Create a sub-state snapshot for nested calls
    fn snapshot(&self) -> usize;
    /// Revert to a snapshot
    fn revert_to_snapshot(&mut self, snapshot: usize);
    /// Commit a snapshot (discard revert point)
    fn commit_snapshot(&mut self, snapshot: usize);
}

/// Null state that returns defaults (used for pure computation / testing)
#[derive(Default)]
pub struct NullState;

impl StateAccess for NullState {
    fn get_storage(&self, _address: &Address, _key: &H256) -> H256 { H256::ZERO }
    fn set_storage(&mut self, _address: Address, _key: H256, _value: H256) {}
    fn get_balance(&self, _address: &Address) -> u128 { 0 }
    fn get_code(&self, _address: &Address) -> Vec<u8> { vec![] }
    fn get_code_hash(&self, _address: &Address) -> H256 { H256::ZERO }
    fn account_exists(&self, _address: &Address) -> bool { false }
    fn transfer(&mut self, _from: &Address, _to: &Address, _value: u128) -> Result<(), EvmError> { Ok(()) }
    fn get_nonce(&self, _address: &Address) -> u64 { 0 }
    fn increment_nonce(&mut self, _address: &Address) -> u64 { 0 }
    fn mark_warm(&mut self, _address: &Address) {}
    fn is_warm(&self, _address: &Address) -> bool { true }
    fn mark_storage_warm(&mut self, _address: &Address, _key: &H256) {}
    fn is_storage_warm(&self, _address: &Address, _key: &H256) -> bool { true }
    fn snapshot(&self) -> usize { 0 }
    fn revert_to_snapshot(&mut self, _snapshot: usize) {}
    fn commit_snapshot(&mut self, _snapshot: usize) {}
}

/// Interpreter state
#[derive(Clone, Debug)]
pub struct Interpreter {
    /// Bytecode being executed
    code: Vec<u8>,
    /// Program counter
    pc: usize,
    /// Stack
    stack: Stack,
    /// Memory
    memory: Memory,
    /// Return data from last call
    return_data: Vec<u8>,
    /// Gas remaining
    gas: u64,
    /// Valid jump destinations
    jump_dests: HashSet<usize>,
    /// Execution stopped
    stopped: bool,
    /// Logs emitted
    logs: Vec<Log>,
    /// Transient storage (EIP-1153, per-transaction)
    transient_storage: HashMap<(Address, H256), H256>,
}

impl Interpreter {
    /// Create a new interpreter with bytecode and gas
    pub fn new(code: Vec<u8>, gas: u64) -> Self {
        let jump_dests = Self::analyze_jump_dests(&code);
        Self {
            code,
            pc: 0,
            stack: Stack::new(),
            memory: Memory::new(),
            return_data: Vec::new(),
            gas,
            jump_dests,
            stopped: false,
            logs: Vec::new(),
            transient_storage: HashMap::new(),
        }
    }

    /// Analyze bytecode for valid jump destinations
    fn analyze_jump_dests(code: &[u8]) -> HashSet<usize> {
        let mut dests = HashSet::new();
        let mut i = 0;

        while i < code.len() {
            let opcode = code[i];
            if opcode == Opcode::JUMPDEST as u8 {
                dests.insert(i);
            }
            // Skip PUSH operands
            if (0x60..=0x7F).contains(&opcode) {
                let push_size = (opcode - 0x5F) as usize;
                i += push_size;
            }
            i += 1;
        }

        dests
    }

    /// Execute until completion or error (backward compatible - uses NullState)
    pub fn run(&mut self, env: &Environment) -> ExecutionResult {
        let mut state = NullState;
        self.run_with_state(env, &mut state)
    }

    /// Execute until completion with state access
    pub fn run_with_state(&mut self, env: &Environment, state: &mut dyn StateAccess) -> ExecutionResult {
        let initial_gas = self.gas;

        while !self.stopped && self.pc < self.code.len() {
            match self.step_with_state(env, state) {
                Ok(()) => {}
                Err(EvmError::Revert(data)) => {
                    return ExecutionResult::revert(
                        initial_gas - self.gas,
                        data,
                    );
                }
                Err(_e) => {
                    // Consume all gas on error
                    return ExecutionResult::failure(initial_gas, Vec::new());
                }
            }
        }

        // Normal completion (STOP, RETURN, or end of code)
        ExecutionResult::success(
            initial_gas - self.gas,
            self.return_data.clone(),
            std::mem::take(&mut self.logs),
        )
    }

    /// Execute a single step (backward compatible)
    pub fn step(&mut self, env: &Environment) -> EvmResult<()> {
        let mut state = NullState;
        self.step_with_state(env, &mut state)
    }

    /// Execute a single step with state access
    pub fn step_with_state(&mut self, env: &Environment, state: &mut dyn StateAccess) -> EvmResult<()> {
        if self.pc >= self.code.len() {
            self.stopped = true;
            return Ok(());
        }

        let opcode_byte = self.code[self.pc];
        let opcode = Opcode::from_byte(opcode_byte)
            .ok_or(EvmError::InvalidOpcode(opcode_byte))?;

        // Check and consume static gas
        let static_gas = gas::static_gas(opcode);
        self.use_gas(static_gas)?;

        // Execute opcode
        self.execute(opcode, env, state)?;

        Ok(())
    }

    /// Use gas, returning error if insufficient
    fn use_gas(&mut self, amount: u64) -> EvmResult<()> {
        if self.gas < amount {
            return Err(EvmError::OutOfGas);
        }
        self.gas -= amount;
        Ok(())
    }

    /// Execute an opcode
    fn execute(&mut self, opcode: Opcode, env: &Environment, state: &mut dyn StateAccess) -> EvmResult<()> {
        match opcode {
            // ==================== Stop ====================
            Opcode::STOP => {
                self.stopped = true;
            }

            // ==================== Arithmetic ====================
            Opcode::ADD => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_add(&a, &b))?;
                self.pc += 1;
            }
            Opcode::MUL => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_mul(&a, &b))?;
                self.pc += 1;
            }
            Opcode::SUB => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_sub(&a, &b))?;
                self.pc += 1;
            }
            Opcode::DIV => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_div(&a, &b))?;
                self.pc += 1;
            }
            Opcode::SDIV => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_sdiv(&a, &b))?;
                self.pc += 1;
            }
            Opcode::MOD => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_mod(&a, &b))?;
                self.pc += 1;
            }
            Opcode::SMOD => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_smod(&a, &b))?;
                self.pc += 1;
            }
            Opcode::ADDMOD => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let n = self.stack.pop()?;
                self.stack.push(stack::u256_addmod(&a, &b, &n))?;
                self.pc += 1;
            }
            Opcode::MULMOD => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let n = self.stack.pop()?;
                self.stack.push(stack::u256_mulmod(&a, &b, &n))?;
                self.pc += 1;
            }
            Opcode::EXP => {
                let base = self.stack.pop()?;
                let exponent = self.stack.pop()?;
                // Dynamic gas for EXP
                self.use_gas(gas::exp_gas(&exponent) - gas::cost::EXP)?; // static already charged
                self.stack.push(stack::u256_exp(&base, &exponent))?;
                self.pc += 1;
            }
            Opcode::SIGNEXTEND => {
                let b = self.stack.pop()?;
                let x = self.stack.pop()?;
                self.stack.push(stack::u256_signextend(&b, &x))?;
                self.pc += 1;
            }

            // ==================== Comparison ====================
            Opcode::LT => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let result = if stack::u256_lt(&a, &b) { U256_ONE } else { U256_ZERO };
                self.stack.push(result)?;
                self.pc += 1;
            }
            Opcode::GT => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let result = if stack::u256_gt(&a, &b) { U256_ONE } else { U256_ZERO };
                self.stack.push(result)?;
                self.pc += 1;
            }
            Opcode::SLT => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let result = if stack::u256_slt(&a, &b) { U256_ONE } else { U256_ZERO };
                self.stack.push(result)?;
                self.pc += 1;
            }
            Opcode::SGT => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let result = if stack::u256_sgt(&a, &b) { U256_ONE } else { U256_ZERO };
                self.stack.push(result)?;
                self.pc += 1;
            }
            Opcode::EQ => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let result = if a == b { U256_ONE } else { U256_ZERO };
                self.stack.push(result)?;
                self.pc += 1;
            }
            Opcode::ISZERO => {
                let a = self.stack.pop()?;
                let result = if stack::u256_is_zero(&a) { U256_ONE } else { U256_ZERO };
                self.stack.push(result)?;
                self.pc += 1;
            }

            // ==================== Bitwise ====================
            Opcode::AND => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_and(&a, &b))?;
                self.pc += 1;
            }
            Opcode::OR => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_or(&a, &b))?;
                self.pc += 1;
            }
            Opcode::XOR => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_xor(&a, &b))?;
                self.pc += 1;
            }
            Opcode::NOT => {
                let a = self.stack.pop()?;
                self.stack.push(stack::u256_not(&a))?;
                self.pc += 1;
            }
            Opcode::BYTE => {
                let i = self.stack.pop()?;
                let x = self.stack.pop()?;
                self.stack.push(stack::u256_byte(&i, &x))?;
                self.pc += 1;
            }
            Opcode::SHL => {
                let shift = self.stack.pop()?;
                let value = self.stack.pop()?;
                self.stack.push(stack::u256_shl(&shift, &value))?;
                self.pc += 1;
            }
            Opcode::SHR => {
                let shift = self.stack.pop()?;
                let value = self.stack.pop()?;
                self.stack.push(stack::u256_shr(&shift, &value))?;
                self.pc += 1;
            }
            Opcode::SAR => {
                let shift = self.stack.pop()?;
                let value = self.stack.pop()?;
                self.stack.push(stack::u256_sar(&shift, &value))?;
                self.pc += 1;
            }

            // ==================== SHA3 ====================
            Opcode::KECCAK256 => {
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                let new_size = self.memory.expand(offset, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.use_gas(gas::sha3_gas(size))?;

                let data = self.memory.load_slice(offset, size);
                let hash = keccak256(&data);
                self.stack.push(*hash.as_bytes())?;
                self.pc += 1;
            }

            // ==================== Environment ====================
            Opcode::ADDRESS => {
                let mut addr = U256_ZERO;
                addr[12..32].copy_from_slice(env.call.address.as_bytes());
                self.stack.push(addr)?;
                self.pc += 1;
            }
            Opcode::BALANCE => {
                let addr_u256 = self.stack.pop()?;
                let address = u256_to_address(&addr_u256);
                let balance = state.get_balance(&address);
                self.stack.push(stack::u128_to_u256(balance))?;
                self.pc += 1;
            }
            Opcode::ORIGIN => {
                let mut addr = U256_ZERO;
                addr[12..32].copy_from_slice(env.tx.origin.as_bytes());
                self.stack.push(addr)?;
                self.pc += 1;
            }
            Opcode::CALLER => {
                let mut addr = U256_ZERO;
                addr[12..32].copy_from_slice(env.call.caller.as_bytes());
                self.stack.push(addr)?;
                self.pc += 1;
            }
            Opcode::CALLVALUE => {
                self.stack.push(stack::u128_to_u256(env.call.value))?;
                self.pc += 1;
            }
            Opcode::CALLDATALOAD => {
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .unwrap_or(usize::MAX);
                let mut result = U256_ZERO;
                for (i, byte) in result.iter_mut().enumerate().take(32) {
                    if offset.wrapping_add(i) < env.call.data.len() {
                        *byte = env.call.data[offset + i];
                    }
                }
                self.stack.push(result)?;
                self.pc += 1;
            }
            Opcode::CALLDATASIZE => {
                self.stack.push(stack::u64_to_u256(env.call.data.len() as u64))?;
                self.pc += 1;
            }
            Opcode::CALLDATACOPY => {
                let dest = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .unwrap_or(usize::MAX);
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                let new_size = self.memory.expand(dest, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.use_gas(gas::copy_gas(size))?;

                let mut data = vec![0u8; size];
                for (i, byte) in data.iter_mut().enumerate() {
                    if offset.wrapping_add(i) < env.call.data.len() {
                        *byte = env.call.data[offset + i];
                    }
                }
                self.memory.store_slice(dest, &data);
                self.pc += 1;
            }
            Opcode::CODESIZE => {
                self.stack.push(stack::u64_to_u256(self.code.len() as u64))?;
                self.pc += 1;
            }
            Opcode::CODECOPY => {
                let dest = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .unwrap_or(usize::MAX);
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                let new_size = self.memory.expand(dest, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.use_gas(gas::copy_gas(size))?;

                let mut data = vec![0u8; size];
                for (i, byte) in data.iter_mut().enumerate() {
                    if offset.wrapping_add(i) < self.code.len() {
                        *byte = self.code[offset + i];
                    }
                }
                self.memory.store_slice(dest, &data);
                self.pc += 1;
            }
            Opcode::GASPRICE => {
                self.stack.push(stack::u128_to_u256(env.tx.gas_price))?;
                self.pc += 1;
            }
            Opcode::EXTCODESIZE => {
                let addr_u256 = self.stack.pop()?;
                let address = u256_to_address(&addr_u256);
                let size = state.get_code_size(&address);
                self.stack.push(stack::u64_to_u256(size as u64))?;
                self.pc += 1;
            }
            Opcode::EXTCODECOPY => {
                let addr_u256 = self.stack.pop()?;
                let address = u256_to_address(&addr_u256);
                let dest = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .unwrap_or(usize::MAX);
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                let new_size = self.memory.expand(dest, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.use_gas(gas::copy_gas(size))?;

                let ext_code = state.get_code(&address);
                let mut data = vec![0u8; size];
                for (i, byte) in data.iter_mut().enumerate() {
                    if offset.wrapping_add(i) < ext_code.len() {
                        *byte = ext_code[offset + i];
                    }
                }
                self.memory.store_slice(dest, &data);
                self.pc += 1;
            }
            Opcode::RETURNDATASIZE => {
                self.stack.push(stack::u64_to_u256(self.return_data.len() as u64))?;
                self.pc += 1;
            }
            Opcode::RETURNDATACOPY => {
                let dest = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::ReturnDataOutOfBounds)?;
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                if offset.saturating_add(size) > self.return_data.len() {
                    return Err(EvmError::ReturnDataOutOfBounds);
                }

                let new_mem_size = self.memory.expand(dest, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_mem_size))?;
                self.use_gas(gas::copy_gas(size))?;

                self.memory.store_slice(dest, &self.return_data[offset..offset + size]);
                self.pc += 1;
            }
            Opcode::EXTCODEHASH => {
                let addr_u256 = self.stack.pop()?;
                let address = u256_to_address(&addr_u256);
                if !state.account_exists(&address) {
                    self.stack.push(U256_ZERO)?;
                } else {
                    let hash = state.get_code_hash(&address);
                    self.stack.push(*hash.as_bytes())?;
                }
                self.pc += 1;
            }

            // ==================== Block info ====================
            Opcode::BLOCKHASH => {
                let _block_num = self.stack.pop()?;
                // Return zero for now - needs block history access
                self.stack.push(U256_ZERO)?;
                self.pc += 1;
            }
            Opcode::COINBASE => {
                let mut addr = U256_ZERO;
                addr[12..32].copy_from_slice(env.block.coinbase.as_bytes());
                self.stack.push(addr)?;
                self.pc += 1;
            }
            Opcode::TIMESTAMP => {
                self.stack.push(stack::u64_to_u256(env.block.timestamp))?;
                self.pc += 1;
            }
            Opcode::NUMBER => {
                self.stack.push(stack::u64_to_u256(env.block.number))?;
                self.pc += 1;
            }
            Opcode::PREVRANDAO => {
                self.stack.push(*env.block.prevrandao.as_bytes())?;
                self.pc += 1;
            }
            Opcode::GASLIMIT => {
                self.stack.push(stack::u64_to_u256(env.block.gas_limit))?;
                self.pc += 1;
            }
            Opcode::CHAINID => {
                self.stack.push(stack::u64_to_u256(env.block.chain_id))?;
                self.pc += 1;
            }
            Opcode::SELFBALANCE => {
                let balance = state.get_balance(&env.call.address);
                self.stack.push(stack::u128_to_u256(balance))?;
                self.pc += 1;
            }
            Opcode::BASEFEE => {
                self.stack.push(stack::u128_to_u256(env.block.base_fee))?;
                self.pc += 1;
            }

            // ==================== Stack, Memory, Flow ====================
            Opcode::POP => {
                self.stack.pop()?;
                self.pc += 1;
            }
            Opcode::MLOAD => {
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let new_size = self.memory.expand(offset, 32);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                let value = self.memory.load(offset);
                self.stack.push(value)?;
                self.pc += 1;
            }
            Opcode::MSTORE => {
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let value = self.stack.pop()?;
                let new_size = self.memory.expand(offset, 32);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.memory.store(offset, &value);
                self.pc += 1;
            }
            Opcode::MSTORE8 => {
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let value = self.stack.pop()?;
                let new_size = self.memory.expand(offset, 1);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.memory.store8(offset, value[31]);
                self.pc += 1;
            }

            // ==================== Storage ====================
            Opcode::SLOAD => {
                let key_u256 = self.stack.pop()?;
                let key = H256::from_bytes(key_u256);
                let value = state.get_storage(&env.call.address, &key);
                self.stack.push(*value.as_bytes())?;
                self.pc += 1;
            }
            Opcode::SSTORE => {
                if env.call.is_static {
                    return Err(EvmError::StaticCallViolation);
                }
                let key_u256 = self.stack.pop()?;
                let value_u256 = self.stack.pop()?;
                let key = H256::from_bytes(key_u256);
                let value = H256::from_bytes(value_u256);
                // Dynamic gas for SSTORE (simplified: charge set cost for non-zero, reset for zero)
                let current = state.get_storage(&env.call.address, &key);
                let gas_cost = if current == H256::ZERO && value != H256::ZERO {
                    gas::cost::SSTORE_SET
                } else if current != H256::ZERO && value == H256::ZERO {
                    gas::cost::SSTORE_RESET
                } else {
                    gas::cost::SLOAD_WARM
                };
                self.use_gas(gas_cost)?;
                state.set_storage(env.call.address, key, value);
                self.pc += 1;
            }

            // ==================== Flow control ====================
            Opcode::JUMP => {
                let dest = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidJump(usize::MAX))?;
                if !self.jump_dests.contains(&dest) {
                    return Err(EvmError::InvalidJump(dest));
                }
                self.pc = dest;
            }
            Opcode::JUMPI => {
                let dest = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidJump(usize::MAX))?;
                let cond = self.stack.pop()?;
                if !stack::u256_is_zero(&cond) {
                    if !self.jump_dests.contains(&dest) {
                        return Err(EvmError::InvalidJump(dest));
                    }
                    self.pc = dest;
                } else {
                    self.pc += 1;
                }
            }
            Opcode::PC => {
                self.stack.push(stack::u64_to_u256(self.pc as u64))?;
                self.pc += 1;
            }
            Opcode::MSIZE => {
                self.stack.push(stack::u64_to_u256(self.memory.size() as u64))?;
                self.pc += 1;
            }
            Opcode::GAS => {
                self.stack.push(stack::u64_to_u256(self.gas))?;
                self.pc += 1;
            }
            Opcode::JUMPDEST => {
                self.pc += 1;
            }

            // ==================== Transient storage (EIP-1153) ====================
            Opcode::TLOAD => {
                let key_u256 = self.stack.pop()?;
                let key = H256::from_bytes(key_u256);
                let value = self.transient_storage
                    .get(&(env.call.address, key))
                    .copied()
                    .unwrap_or(H256::ZERO);
                self.stack.push(*value.as_bytes())?;
                self.pc += 1;
            }
            Opcode::TSTORE => {
                if env.call.is_static {
                    return Err(EvmError::StaticCallViolation);
                }
                let key_u256 = self.stack.pop()?;
                let value_u256 = self.stack.pop()?;
                let key = H256::from_bytes(key_u256);
                let value = H256::from_bytes(value_u256);
                self.transient_storage.insert((env.call.address, key), value);
                self.pc += 1;
            }

            // ==================== MCOPY (EIP-5656) ====================
            Opcode::MCOPY => {
                let dest = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let src = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                let new_size_dest = self.memory.expand(dest, size);
                let new_size_src = self.memory.expand(src, size);
                let new_size = std::cmp::max(new_size_dest, new_size_src);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.use_gas(gas::copy_gas(size))?;

                self.memory.copy(dest, src, size);
                self.pc += 1;
            }

            // ==================== Push operations ====================
            Opcode::PUSH0 => {
                self.stack.push(U256_ZERO)?;
                self.pc += 1;
            }
            op if op.is_push() => {
                let size = op.push_size();
                let mut value = U256_ZERO;
                for i in 0..size {
                    let idx = self.pc + 1 + i;
                    if idx < self.code.len() {
                        value[32 - size + i] = self.code[idx];
                    }
                }
                self.stack.push(value)?;
                self.pc += 1 + size;
            }

            // ==================== Dup operations ====================
            op if op.dup_depth() > 0 => {
                self.stack.dup(op.dup_depth())?;
                self.pc += 1;
            }

            // ==================== Swap operations ====================
            op if op.swap_depth() > 0 => {
                self.stack.swap(op.swap_depth())?;
                self.pc += 1;
            }

            // ==================== LOG operations ====================
            op if op.log_topics() > 0 || op == Opcode::LOG0 => {
                if env.call.is_static {
                    return Err(EvmError::StaticCallViolation);
                }
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                let topic_count = op.log_topics();
                let mut topics = Vec::with_capacity(topic_count);
                for _ in 0..topic_count {
                    let topic = self.stack.pop()?;
                    topics.push(H256::from_bytes(topic));
                }

                let new_mem_size = self.memory.expand(offset, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_mem_size))?;
                self.use_gas(gas::log_gas(topic_count, size))?;

                let data = self.memory.load_slice(offset, size);
                self.logs.push(Log {
                    address: env.call.address,
                    topics,
                    data,
                });
                self.pc += 1;
            }

            // ==================== System: CALL/STATICCALL/DELEGATECALL/CALLCODE ====================
            Opcode::CALL | Opcode::CALLCODE => {
                let gas_limit = self.stack.pop()?;
                let addr_u256 = self.stack.pop()?;
                let value_u256 = self.stack.pop()?;
                let args_offset = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let args_size = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let ret_offset = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let ret_size = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;

                let address = u256_to_address(&addr_u256);
                let value = u256_to_u128_saturating(&value_u256);

                if env.call.is_static && value > 0 {
                    return Err(EvmError::StaticCallViolation);
                }

                // Memory expansion
                let new_size_args = self.memory.expand(args_offset, args_size);
                let new_size_ret = self.memory.expand(ret_offset, ret_size);
                let new_size = std::cmp::max(new_size_args, new_size_ret);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;

                // Additional gas for value transfer
                if value > 0 {
                    self.use_gas(gas::cost::CALL_VALUE)?;
                }

                let call_gas = stack::u256_to_u64(&gas_limit).unwrap_or(self.gas);
                let call_gas = std::cmp::min(call_gas, self.gas - self.gas / 64);

                let call_data = self.memory.load_slice(args_offset, args_size);
                let code = state.get_code(&address);

                if code.is_empty() {
                    // No code to execute - simple transfer if value > 0
                    if value > 0 && opcode == Opcode::CALL {
                        let _ = state.transfer(&env.call.address, &address, value);
                    }
                    self.return_data = vec![];
                    self.stack.push(U256_ONE)?; // success
                } else {
                    let snapshot = state.snapshot();
                    if value > 0 && opcode == Opcode::CALL {
                        if let Err(_) = state.transfer(&env.call.address, &address, value) {
                            self.return_data = vec![];
                            self.stack.push(U256_ZERO)?;
                            self.pc += 1;
                            return Ok(());
                        }
                    }

                    let (call_addr, call_caller) = if opcode == Opcode::CALL {
                        (address, env.call.address)
                    } else {
                        // CALLCODE: execute code at address but in our context
                        (env.call.address, env.call.caller)
                    };

                    let sub_env = Environment {
                        call: crate::context::CallContext {
                            address: call_addr,
                            caller: call_caller,
                            value,
                            data: call_data,
                            gas: call_gas,
                            is_static: env.call.is_static,
                            depth: env.call.depth + 1,
                        },
                        block: env.block.clone(),
                        tx: env.tx.clone(),
                    };

                    let mut sub_interp = Interpreter::new(code, call_gas);
                    let result = sub_interp.run_with_state(&sub_env, state);

                    self.gas -= call_gas;
                    self.gas += call_gas.saturating_sub(result.gas_used);

                    if result.success {
                        state.commit_snapshot(snapshot);
                        self.logs.extend(result.logs);
                        self.stack.push(U256_ONE)?;
                    } else {
                        state.revert_to_snapshot(snapshot);
                        self.stack.push(U256_ZERO)?;
                    }

                    self.return_data = result.output.clone();
                    let copy_size = std::cmp::min(ret_size, result.output.len());
                    if copy_size > 0 {
                        self.memory.store_slice(ret_offset, &result.output[..copy_size]);
                    }
                }
                self.pc += 1;
            }

            Opcode::DELEGATECALL | Opcode::STATICCALL => {
                let gas_limit = self.stack.pop()?;
                let addr_u256 = self.stack.pop()?;
                let args_offset = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let args_size = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let ret_offset = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let ret_size = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;

                let address = u256_to_address(&addr_u256);

                let new_size_args = self.memory.expand(args_offset, args_size);
                let new_size_ret = self.memory.expand(ret_offset, ret_size);
                let new_size = std::cmp::max(new_size_args, new_size_ret);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;

                let call_gas = stack::u256_to_u64(&gas_limit).unwrap_or(self.gas);
                let call_gas = std::cmp::min(call_gas, self.gas - self.gas / 64);

                let call_data = self.memory.load_slice(args_offset, args_size);
                let code = state.get_code(&address);

                if code.is_empty() {
                    self.return_data = vec![];
                    self.stack.push(U256_ONE)?;
                } else {
                    let snapshot = state.snapshot();

                    let (call_addr, call_caller, call_value, is_static) = if opcode == Opcode::DELEGATECALL {
                        (env.call.address, env.call.caller, env.call.value, env.call.is_static)
                    } else {
                        // STATICCALL
                        (address, env.call.address, 0u128, true)
                    };

                    let sub_env = Environment {
                        call: crate::context::CallContext {
                            address: call_addr,
                            caller: call_caller,
                            value: call_value,
                            data: call_data,
                            gas: call_gas,
                            is_static,
                            depth: env.call.depth + 1,
                        },
                        block: env.block.clone(),
                        tx: env.tx.clone(),
                    };

                    let mut sub_interp = Interpreter::new(code, call_gas);
                    let result = sub_interp.run_with_state(&sub_env, state);

                    self.gas -= call_gas;
                    self.gas += call_gas.saturating_sub(result.gas_used);

                    if result.success {
                        state.commit_snapshot(snapshot);
                        self.logs.extend(result.logs);
                        self.stack.push(U256_ONE)?;
                    } else {
                        state.revert_to_snapshot(snapshot);
                        self.stack.push(U256_ZERO)?;
                    }

                    self.return_data = result.output.clone();
                    let copy_size = std::cmp::min(ret_size, result.output.len());
                    if copy_size > 0 {
                        self.memory.store_slice(ret_offset, &result.output[..copy_size]);
                    }
                }
                self.pc += 1;
            }

            // ==================== System: CREATE/CREATE2 ====================
            Opcode::CREATE => {
                if env.call.is_static {
                    return Err(EvmError::StaticCallViolation);
                }
                let value_u256 = self.stack.pop()?;
                let offset = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let size = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;

                let new_size = self.memory.expand(offset, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;

                let value = u256_to_u128_saturating(&value_u256);
                let init_code = self.memory.load_slice(offset, size);

                let nonce = state.get_nonce(&env.call.address);
                let contract_address = create_address(&env.call.address, nonce);
                state.increment_nonce(&env.call.address);

                let call_gas = self.gas - self.gas / 64;
                let snapshot = state.snapshot();

                if value > 0 {
                    if let Err(_) = state.transfer(&env.call.address, &contract_address, value) {
                        self.stack.push(U256_ZERO)?;
                        self.pc += 1;
                        return Ok(());
                    }
                }

                let sub_env = Environment {
                    call: crate::context::CallContext {
                        address: contract_address,
                        caller: env.call.address,
                        value,
                        data: vec![],
                        gas: call_gas,
                        is_static: false,
                        depth: env.call.depth + 1,
                    },
                    block: env.block.clone(),
                    tx: env.tx.clone(),
                };

                let mut sub_interp = Interpreter::new(init_code, call_gas);
                let result = sub_interp.run_with_state(&sub_env, state);

                self.gas -= call_gas;
                self.gas += call_gas.saturating_sub(result.gas_used);

                if result.success && !result.output.is_empty() {
                    if result.output.len() > gas::cost::MAX_CODE_SIZE {
                        state.revert_to_snapshot(snapshot);
                        self.stack.push(U256_ZERO)?;
                    } else {
                        state.commit_snapshot(snapshot);
                        // Store contract code via the state's set_storage mechanism
                        // (the executor layer handles actual code storage)
                        self.logs.extend(result.logs);
                        let mut addr_val = U256_ZERO;
                        addr_val[12..32].copy_from_slice(contract_address.as_bytes());
                        self.stack.push(addr_val)?;
                    }
                } else {
                    state.revert_to_snapshot(snapshot);
                    self.stack.push(U256_ZERO)?;
                }

                self.return_data = result.output;
                self.pc += 1;
            }

            Opcode::CREATE2 => {
                if env.call.is_static {
                    return Err(EvmError::StaticCallViolation);
                }
                let value_u256 = self.stack.pop()?;
                let offset = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let size = stack::u256_to_usize(&self.stack.pop()?).ok_or(EvmError::InvalidMemoryAccess)?;
                let salt = self.stack.pop()?;

                let new_size = self.memory.expand(offset, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.use_gas(gas::copy_gas(size))?; // init code hashing cost

                let value = u256_to_u128_saturating(&value_u256);
                let init_code = self.memory.load_slice(offset, size);

                let code_hash = keccak256(&init_code);
                let contract_address = create2_address(&env.call.address, &H256::from_bytes(salt), &code_hash);
                state.increment_nonce(&env.call.address);

                let call_gas = self.gas - self.gas / 64;
                let snapshot = state.snapshot();

                if value > 0 {
                    if let Err(_) = state.transfer(&env.call.address, &contract_address, value) {
                        self.stack.push(U256_ZERO)?;
                        self.pc += 1;
                        return Ok(());
                    }
                }

                let sub_env = Environment {
                    call: crate::context::CallContext {
                        address: contract_address,
                        caller: env.call.address,
                        value,
                        data: vec![],
                        gas: call_gas,
                        is_static: false,
                        depth: env.call.depth + 1,
                    },
                    block: env.block.clone(),
                    tx: env.tx.clone(),
                };

                let mut sub_interp = Interpreter::new(init_code, call_gas);
                let result = sub_interp.run_with_state(&sub_env, state);

                self.gas -= call_gas;
                self.gas += call_gas.saturating_sub(result.gas_used);

                if result.success && !result.output.is_empty() {
                    if result.output.len() > gas::cost::MAX_CODE_SIZE {
                        state.revert_to_snapshot(snapshot);
                        self.stack.push(U256_ZERO)?;
                    } else {
                        state.commit_snapshot(snapshot);
                        self.logs.extend(result.logs);
                        let mut addr_val = U256_ZERO;
                        addr_val[12..32].copy_from_slice(contract_address.as_bytes());
                        self.stack.push(addr_val)?;
                    }
                } else {
                    state.revert_to_snapshot(snapshot);
                    self.stack.push(U256_ZERO)?;
                }

                self.return_data = result.output;
                self.pc += 1;
            }

            // ==================== Return/Revert ====================
            Opcode::RETURN => {
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                let new_mem_size = self.memory.expand(offset, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_mem_size))?;

                self.return_data = self.memory.load_slice(offset, size);
                self.stopped = true;
            }
            Opcode::REVERT => {
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                let new_mem_size = self.memory.expand(offset, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_mem_size))?;

                let data = self.memory.load_slice(offset, size);
                return Err(EvmError::Revert(data));
            }
            Opcode::INVALID => {
                return Err(EvmError::InvalidOpcode(0xFE));
            }
            Opcode::SELFDESTRUCT => {
                if env.call.is_static {
                    return Err(EvmError::StaticCallViolation);
                }
                let beneficiary_u256 = self.stack.pop()?;
                let beneficiary = u256_to_address(&beneficiary_u256);
                let balance = state.get_balance(&env.call.address);
                if balance > 0 {
                    let _ = state.transfer(&env.call.address, &beneficiary, balance);
                }
                self.stopped = true;
            }

            // ==================== Unimplemented opcodes ====================
            _ => {
                return Err(EvmError::InvalidOpcode(opcode as u8));
            }
        }

        Ok(())
    }

    /// Get remaining gas
    pub fn gas_remaining(&self) -> u64 {
        self.gas
    }

    /// Get return data
    pub fn return_data(&self) -> &[u8] {
        &self.return_data
    }
}

// ==================== Helper functions ====================

/// Extract Address from U256 (last 20 bytes)
fn u256_to_address(v: &U256) -> Address {
    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&v[12..32]);
    Address::from_bytes(bytes)
}

/// Saturating convert U256 to u128
fn u256_to_u128_saturating(v: &U256) -> u128 {
    // Check if any bytes above u128 range are set
    if v[0..16].iter().any(|&b| b != 0) {
        return u128::MAX;
    }
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&v[16..32]);
    u128::from_be_bytes(bytes)
}

/// Calculate CREATE address: keccak256(RLP([sender, nonce]))[12:]
fn create_address(sender: &Address, nonce: u64) -> Address {
    // RLP encode [sender, nonce]
    let mut rlp_data = Vec::new();

    // Encode sender (20 bytes -> prefix 0x94)
    let sender_bytes = sender.as_bytes();
    rlp_data.push(0x80 + 20);
    rlp_data.extend_from_slice(sender_bytes);

    // Encode nonce
    if nonce == 0 {
        rlp_data.push(0x80);
    } else if nonce < 128 {
        rlp_data.push(nonce as u8);
    } else {
        let nonce_bytes = nonce.to_be_bytes();
        let start = nonce_bytes.iter().position(|&b| b != 0).unwrap_or(8);
        let len = 8 - start;
        rlp_data.push(0x80 + len as u8);
        rlp_data.extend_from_slice(&nonce_bytes[start..]);
    }

    // Wrap in list
    let payload_len = rlp_data.len();
    let mut list = Vec::new();
    if payload_len < 56 {
        list.push(0xc0 + payload_len as u8);
    } else {
        let len_bytes = payload_len.to_be_bytes();
        let start = len_bytes.iter().position(|&b| b != 0).unwrap_or(7);
        list.push(0xf7 + (8 - start) as u8);
        list.extend_from_slice(&len_bytes[start..]);
    }
    list.extend_from_slice(&rlp_data);

    let hash = keccak256(&list);
    Address::from_slice(&hash.as_bytes()[12..]).unwrap_or(Address::ZERO)
}

/// Calculate CREATE2 address: keccak256(0xff ++ sender ++ salt ++ keccak256(init_code))[12:]
fn create2_address(sender: &Address, salt: &H256, code_hash: &H256) -> Address {
    let mut data = Vec::with_capacity(1 + 20 + 32 + 32);
    data.push(0xff);
    data.extend_from_slice(sender.as_bytes());
    data.extend_from_slice(salt.as_bytes());
    data.extend_from_slice(code_hash.as_bytes());
    let hash = keccak256(&data);
    Address::from_slice(&hash.as_bytes()[12..]).unwrap_or(Address::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_code(code: &[u8], gas: u64) -> ExecutionResult {
        let mut interp = Interpreter::new(code.to_vec(), gas);
        let env = Environment::default();
        interp.run(&env)
    }

    #[test]
    fn test_stop() {
        let result = run_code(&[0x00], 1000);
        assert!(result.success);
    }

    #[test]
    fn test_push_add() {
        let code = [0x60, 0x03, 0x60, 0x05, 0x01, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_push_sub() {
        let code = [0x60, 0x03, 0x60, 0x0A, 0x03, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_mul() {
        let code = [0x60, 0x03, 0x60, 0x04, 0x02, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_div() {
        let code = [0x60, 0x02, 0x60, 0x0A, 0x04, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_return() {
        let code = [0x60, 0x04, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3];
        let result = run_code(&code, 10000);
        assert!(result.success);
        assert_eq!(result.output.len(), 32);
        assert_eq!(result.output[31], 4);
    }

    #[test]
    fn test_revert() {
        let code = [0x60, 0x00, 0x60, 0x00, 0xFD];
        let result = run_code(&code, 10000);
        assert!(!result.success);
    }

    #[test]
    fn test_shl() {
        // PUSH1 1, PUSH1 1, SHL -> 2 (1 << 1 = 2)
        let code = [0x60, 0x01, 0x60, 0x01, 0x1B, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_shr() {
        // PUSH1 4, PUSH1 1, SHR -> 2 (4 >> 1 = 2)
        let code = [0x60, 0x04, 0x60, 0x01, 0x1C, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_byte_opcode() {
        // PUSH1 0xFF, PUSH1 31, BYTE -> 0xFF
        let code = [0x60, 0xFF, 0x60, 0x1F, 0x1A, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_jump() {
        let code = [0x60, 0x04, 0x56, 0xFE, 0x5B, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_out_of_gas() {
        let code = [0x60, 0x01];
        let result = run_code(&code, 1);
        assert!(!result.success);
    }

    #[test]
    fn test_sload_sstore_with_state() {
        // PUSH1 42, PUSH1 0, SSTORE (store 42 at slot 0)
        // PUSH1 0, SLOAD (load from slot 0)
        // PUSH1 0, MSTORE (store result in memory)
        // PUSH1 32, PUSH1 0, RETURN (return 32 bytes from memory)
        let code = [
            0x60, 0x2A, // PUSH1 42
            0x60, 0x00, // PUSH1 0 (slot)
            0x55,       // SSTORE
            0x60, 0x00, // PUSH1 0 (slot)
            0x54,       // SLOAD
            0x60, 0x00, // PUSH1 0 (memory offset)
            0x52,       // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xF3,       // RETURN
        ];

        // Create a simple in-memory state
        use std::collections::HashMap;
        struct TestState {
            storage: HashMap<(Address, H256), H256>,
        }
        impl StateAccess for TestState {
            fn get_storage(&self, address: &Address, key: &H256) -> H256 {
                self.storage.get(&(*address, *key)).copied().unwrap_or(H256::ZERO)
            }
            fn set_storage(&mut self, address: Address, key: H256, value: H256) {
                self.storage.insert((address, key), value);
            }
            fn get_balance(&self, _: &Address) -> u128 { 0 }
            fn get_code(&self, _: &Address) -> Vec<u8> { vec![] }
            fn get_code_hash(&self, _: &Address) -> H256 { H256::ZERO }
            fn account_exists(&self, _: &Address) -> bool { false }
            fn transfer(&mut self, _: &Address, _: &Address, _: u128) -> Result<(), EvmError> { Ok(()) }
            fn get_nonce(&self, _: &Address) -> u64 { 0 }
            fn increment_nonce(&mut self, _: &Address) -> u64 { 0 }
            fn mark_warm(&mut self, _: &Address) {}
            fn is_warm(&self, _: &Address) -> bool { true }
            fn mark_storage_warm(&mut self, _: &Address, _: &H256) {}
            fn is_storage_warm(&self, _: &Address, _: &H256) -> bool { true }
            fn snapshot(&self) -> usize { 0 }
            fn revert_to_snapshot(&mut self, _: usize) {}
            fn commit_snapshot(&mut self, _: usize) {}
        }

        let mut state = TestState { storage: HashMap::new() };
        let mut interp = Interpreter::new(code.to_vec(), 100_000);
        let env = Environment::default();
        let result = interp.run_with_state(&env, &mut state);

        assert!(result.success);
        assert_eq!(result.output.len(), 32);
        assert_eq!(result.output[31], 42); // Value we stored
    }

    #[test]
    fn test_create_address_calculation() {
        let sender = Address::from_bytes([0x42; 20]);
        let addr0 = create_address(&sender, 0);
        let addr1 = create_address(&sender, 1);
        assert_ne!(addr0, addr1);
        assert_ne!(addr0, Address::ZERO);
    }

    #[test]
    fn test_create2_address_calculation() {
        let sender = Address::from_bytes([0x42; 20]);
        let salt = H256::from_bytes([0x01; 32]);
        let code_hash = H256::from_bytes([0xAA; 32]);
        let addr = create2_address(&sender, &salt, &code_hash);
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn test_mul_large_values() {
        // Test that MUL works with values larger than u64
        // PUSH32 (large value), PUSH32 (2), MUL, STOP
        let mut code = Vec::new();
        // PUSH32 0x00...0100000000 (2^32)
        code.push(0x7F);
        let mut val = [0u8; 32];
        val[27] = 1; // 2^32 = 0x100000000
        code.extend_from_slice(&val);
        // PUSH32 0x00...0100000000 (2^32)
        code.push(0x7F);
        code.extend_from_slice(&val);
        // MUL
        code.push(0x02);
        // Store result and return
        code.push(0x60); code.push(0x00); // PUSH1 0
        code.push(0x52); // MSTORE
        code.push(0x60); code.push(0x20); // PUSH1 32
        code.push(0x60); code.push(0x00); // PUSH1 0
        code.push(0xF3); // RETURN

        let result = run_code(&code, 100000);
        assert!(result.success);
        // 2^32 * 2^32 = 2^64 = 0x10000000000000000
        assert_eq!(result.output[23], 1); // byte 23 = 2^64
        // Verify it's not zero (the old broken behavior)
        assert!(result.output.iter().any(|&b| b != 0));
    }
}
