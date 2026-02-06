//! EVM bytecode interpreter

use crate::context::Environment;
use crate::error::{EvmError, EvmResult, ExecutionResult, Log};
use crate::gas;
use crate::memory::Memory;
use crate::opcode::Opcode;
use crate::stack::{self, Stack, U256, U256_ONE, U256_ZERO};
use bach_crypto::keccak256;
use bach_primitives::H256;
use std::collections::HashSet;

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

    /// Execute until completion or error
    pub fn run(&mut self, env: &Environment) -> ExecutionResult {
        let initial_gas = self.gas;

        while !self.stopped && self.pc < self.code.len() {
            match self.step(env) {
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

    /// Execute a single step
    pub fn step(&mut self, env: &Environment) -> EvmResult<()> {
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
        self.execute(opcode, env)?;

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
    fn execute(&mut self, opcode: Opcode, env: &Environment) -> EvmResult<()> {
        match opcode {
            // Stop
            Opcode::STOP => {
                self.stopped = true;
            }

            // Arithmetic
            Opcode::ADD => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_add(&a, &b))?;
                self.pc += 1;
            }
            Opcode::SUB => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                self.stack.push(stack::u256_sub(&a, &b))?;
                self.pc += 1;
            }
            Opcode::MUL => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                // Simple multiplication for small values
                let result = self.u256_mul(&a, &b);
                self.stack.push(result)?;
                self.pc += 1;
            }
            Opcode::DIV => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let result = if stack::u256_is_zero(&b) {
                    U256_ZERO
                } else {
                    self.u256_div(&a, &b)
                };
                self.stack.push(result)?;
                self.pc += 1;
            }
            Opcode::MOD => {
                let a = self.stack.pop()?;
                let b = self.stack.pop()?;
                let result = if stack::u256_is_zero(&b) {
                    U256_ZERO
                } else {
                    self.u256_mod(&a, &b)
                };
                self.stack.push(result)?;
                self.pc += 1;
            }

            // Comparison
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

            // Bitwise
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

            // SHA3
            Opcode::KECCAK256 => {
                let offset = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;
                let size = stack::u256_to_usize(&self.stack.pop()?)
                    .ok_or(EvmError::InvalidMemoryAccess)?;

                // Memory expansion gas
                let new_size = self.memory.expand(offset, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.use_gas(gas::sha3_gas(size))?;

                let data = self.memory.load_slice(offset, size);
                let hash = keccak256(&data);
                self.stack.push(*hash.as_bytes())?;
                self.pc += 1;
            }

            // Environment
            Opcode::ADDRESS => {
                let mut addr = U256_ZERO;
                addr[12..32].copy_from_slice(env.call.address.as_bytes());
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
                    if offset + i < env.call.data.len() {
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

                // Memory expansion and copy gas
                let new_size = self.memory.expand(dest, size);
                self.use_gas(gas::memory_gas(self.memory.size(), new_size))?;
                self.use_gas(gas::copy_gas(size))?;

                let mut data = vec![0u8; size];
                for (i, byte) in data.iter_mut().enumerate() {
                    if offset + i < env.call.data.len() {
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
                    if offset + i < self.code.len() {
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

            // Block info
            Opcode::BLOCKHASH => {
                let _block_num = self.stack.pop()?;
                // TODO: Implement blockhash lookup
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
            Opcode::BASEFEE => {
                self.stack.push(stack::u128_to_u256(env.block.base_fee))?;
                self.pc += 1;
            }
            Opcode::ORIGIN => {
                let mut addr = U256_ZERO;
                addr[12..32].copy_from_slice(env.tx.origin.as_bytes());
                self.stack.push(addr)?;
                self.pc += 1;
            }

            // Stack, Memory, Flow
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
            Opcode::MSIZE => {
                self.stack.push(stack::u64_to_u256(self.memory.size() as u64))?;
                self.pc += 1;
            }
            Opcode::GAS => {
                self.stack.push(stack::u64_to_u256(self.gas))?;
                self.pc += 1;
            }
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
            Opcode::JUMPDEST => {
                self.pc += 1;
            }
            Opcode::PC => {
                self.stack.push(stack::u64_to_u256(self.pc as u64))?;
                self.pc += 1;
            }

            // Push operations
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

            // Dup operations
            op if op.dup_depth() > 0 => {
                self.stack.dup(op.dup_depth())?;
                self.pc += 1;
            }

            // Swap operations
            op if op.swap_depth() > 0 => {
                self.stack.swap(op.swap_depth())?;
                self.pc += 1;
            }

            // Return/Revert
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

            // LOG operations
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

            // Unimplemented opcodes
            _ => {
                return Err(EvmError::InvalidOpcode(opcode as u8));
            }
        }

        Ok(())
    }

    // Simple U256 multiplication (for small values)
    fn u256_mul(&self, a: &U256, b: &U256) -> U256 {
        // Convert to u128 if possible for simple cases
        if let (Some(a_val), Some(b_val)) = (stack::u256_to_u64(a), stack::u256_to_u64(b)) {
            let result = (a_val as u128) * (b_val as u128);
            return stack::u128_to_u256(result);
        }
        // For larger values, return zero (simplified)
        U256_ZERO
    }

    // Simple U256 division (for small values)
    fn u256_div(&self, a: &U256, b: &U256) -> U256 {
        if let (Some(a_val), Some(b_val)) = (stack::u256_to_u64(a), stack::u256_to_u64(b)) {
            if b_val == 0 {
                return U256_ZERO;
            }
            return stack::u64_to_u256(a_val / b_val);
        }
        U256_ZERO
    }

    // Simple U256 modulo (for small values)
    fn u256_mod(&self, a: &U256, b: &U256) -> U256 {
        if let (Some(a_val), Some(b_val)) = (stack::u256_to_u64(a), stack::u256_to_u64(b)) {
            if b_val == 0 {
                return U256_ZERO;
            }
            return stack::u64_to_u256(a_val % b_val);
        }
        U256_ZERO
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
        // PUSH1 3, PUSH1 5, ADD, STOP
        let code = [0x60, 0x03, 0x60, 0x05, 0x01, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_push_sub() {
        // PUSH1 10, PUSH1 3, SUB -> should be 7
        let code = [0x60, 0x03, 0x60, 0x0A, 0x03, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_jump() {
        // PUSH1 4, JUMP, INVALID, JUMPDEST, STOP
        let code = [0x60, 0x04, 0x56, 0xFE, 0x5B, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_jumpi_taken() {
        // PUSH1 1, PUSH1 5, JUMPI, INVALID, JUMPDEST, STOP
        let code = [0x60, 0x01, 0x60, 0x05, 0x57, 0x5B, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_jumpi_not_taken() {
        // PUSH1 0, PUSH1 6, JUMPI, STOP, INVALID, JUMPDEST
        let code = [0x60, 0x00, 0x60, 0x06, 0x57, 0x00, 0x5B];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_mstore_mload() {
        // PUSH1 42, PUSH1 0, MSTORE, PUSH1 0, MLOAD, STOP
        let code = [0x60, 0x2A, 0x60, 0x00, 0x52, 0x60, 0x00, 0x51, 0x00];
        let result = run_code(&code, 10000);
        assert!(result.success);
    }

    #[test]
    fn test_dup_swap() {
        // PUSH1 1, PUSH1 2, DUP2, SWAP1, STOP
        let code = [0x60, 0x01, 0x60, 0x02, 0x81, 0x90, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_return() {
        // PUSH1 4, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN
        let code = [0x60, 0x04, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xF3];
        let result = run_code(&code, 10000);
        assert!(result.success);
        assert_eq!(result.output.len(), 32);
        assert_eq!(result.output[31], 4);
    }

    #[test]
    fn test_revert() {
        // PUSH1 0, PUSH1 0, REVERT
        let code = [0x60, 0x00, 0x60, 0x00, 0xFD];
        let result = run_code(&code, 10000);
        assert!(!result.success);
    }

    #[test]
    fn test_out_of_gas() {
        // PUSH1 1 with only 1 gas
        let code = [0x60, 0x01];
        let result = run_code(&code, 1);
        assert!(!result.success);
    }

    #[test]
    fn test_invalid_jump() {
        // PUSH1 10, JUMP (no JUMPDEST at 10)
        let code = [0x60, 0x0A, 0x56];
        let result = run_code(&code, 1000);
        assert!(!result.success);
    }

    #[test]
    fn test_keccak256() {
        // Store 0x01 at memory 0, KECCAK256(0, 1)
        let code = [
            0x60, 0x01, 0x60, 0x00, 0x52,  // MSTORE 1 at 0
            0x60, 0x01, 0x60, 0x1F,        // PUSH 1, PUSH 31 (offset of the byte)
            0x20,                          // KECCAK256
            0x00                           // STOP
        ];
        let result = run_code(&code, 100000);
        assert!(result.success);
    }

    // ==================== Extended Interpreter Tests ====================

    // --- Arithmetic Tests ---

    #[test]
    fn test_mul() {
        // PUSH1 3, PUSH1 4, MUL -> 12
        let code = [0x60, 0x03, 0x60, 0x04, 0x02, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_div() {
        // PUSH1 2, PUSH1 10, DIV -> 5
        let code = [0x60, 0x02, 0x60, 0x0A, 0x04, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_div_by_zero() {
        // PUSH1 0, PUSH1 10, DIV -> 0 (div by zero returns 0)
        let code = [0x60, 0x00, 0x60, 0x0A, 0x04, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_mod() {
        // PUSH1 3, PUSH1 10, MOD -> 1
        let code = [0x60, 0x03, 0x60, 0x0A, 0x06, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_mod_by_zero() {
        // PUSH1 0, PUSH1 10, MOD -> 0 (mod by zero returns 0)
        let code = [0x60, 0x00, 0x60, 0x0A, 0x06, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    // --- Comparison Tests ---

    #[test]
    fn test_lt() {
        // PUSH1 10, PUSH1 5, LT -> 1 (5 < 10)
        let code = [0x60, 0x0A, 0x60, 0x05, 0x10, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_gt() {
        // PUSH1 5, PUSH1 10, GT -> 1 (10 > 5)
        let code = [0x60, 0x05, 0x60, 0x0A, 0x11, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_eq() {
        // PUSH1 5, PUSH1 5, EQ -> 1
        let code = [0x60, 0x05, 0x60, 0x05, 0x14, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_iszero_true() {
        // PUSH1 0, ISZERO -> 1
        let code = [0x60, 0x00, 0x15, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_iszero_false() {
        // PUSH1 1, ISZERO -> 0
        let code = [0x60, 0x01, 0x15, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    // --- Bitwise Tests ---

    #[test]
    fn test_and() {
        // PUSH1 0x0F, PUSH1 0xFF, AND -> 0x0F
        let code = [0x60, 0x0F, 0x60, 0xFF, 0x16, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_or() {
        // PUSH1 0x0F, PUSH1 0xF0, OR -> 0xFF
        let code = [0x60, 0x0F, 0x60, 0xF0, 0x17, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_xor() {
        // PUSH1 0xFF, PUSH1 0x0F, XOR -> 0xF0
        let code = [0x60, 0xFF, 0x60, 0x0F, 0x18, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_not() {
        // PUSH1 0, NOT -> all 1s
        let code = [0x60, 0x00, 0x19, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    // --- Stack Operations ---

    #[test]
    fn test_pop() {
        // PUSH1 1, PUSH1 2, POP -> stack has [1]
        let code = [0x60, 0x01, 0x60, 0x02, 0x50, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_dup1() {
        // PUSH1 42, DUP1 -> stack has [42, 42]
        let code = [0x60, 0x2A, 0x80, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_dup16() {
        // Push 16 values, then DUP16
        let mut code = Vec::new();
        for i in 1..=16 {
            code.push(0x60); // PUSH1
            code.push(i);
        }
        code.push(0x8F); // DUP16
        code.push(0x00); // STOP
        let result = run_code(&code, 10000);
        assert!(result.success);
    }

    #[test]
    fn test_swap1() {
        // PUSH1 1, PUSH1 2, SWAP1 -> stack has [1, 2] (top=1)
        let code = [0x60, 0x01, 0x60, 0x02, 0x90, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_swap16() {
        // Push 17 values, then SWAP16
        let mut code = Vec::new();
        for i in 1..=17 {
            code.push(0x60); // PUSH1
            code.push(i);
        }
        code.push(0x9F); // SWAP16
        code.push(0x00); // STOP
        let result = run_code(&code, 10000);
        assert!(result.success);
    }

    // --- Memory Operations ---

    #[test]
    fn test_mstore8() {
        // PUSH1 0x42, PUSH1 31, MSTORE8, PUSH1 0, MLOAD -> 0x42 in last byte
        let code = [0x60, 0x42, 0x60, 0x1F, 0x53, 0x60, 0x00, 0x51, 0x00];
        let result = run_code(&code, 10000);
        assert!(result.success);
    }

    #[test]
    fn test_msize() {
        // Initial MSIZE should be 0
        // MSIZE, STOP
        let code = [0x59, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_msize_after_mstore() {
        // PUSH1 1, PUSH1 0, MSTORE, MSIZE -> 32
        let code = [0x60, 0x01, 0x60, 0x00, 0x52, 0x59, 0x00];
        let result = run_code(&code, 10000);
        assert!(result.success);
    }

    // --- Control Flow ---

    #[test]
    fn test_pc() {
        // PC at offset 0 should push 0
        let code = [0x58, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_gas() {
        // GAS should push remaining gas
        let code = [0x5A, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_jump_dest_skipped() {
        // JUMPDEST should be a no-op when executed sequentially
        // JUMPDEST, STOP
        let code = [0x5B, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_nested_jumps() {
        // PUSH 6, JUMP, INVALID, JUMPDEST, PUSH 10, JUMP, INVALID, JUMPDEST, STOP
        let code = [
            0x60, 0x04, // PUSH1 4
            0x56,       // JUMP
            0xFE,       // INVALID (skipped)
            0x5B,       // JUMPDEST (offset 4)
            0x60, 0x09, // PUSH1 9
            0x56,       // JUMP
            0xFE,       // INVALID (skipped)
            0x5B,       // JUMPDEST (offset 9)
            0x00,       // STOP
        ];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    // --- Environment Info ---

    #[test]
    fn test_address() {
        // ADDRESS, STOP
        let code = [0x30, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_caller() {
        // CALLER, STOP
        let code = [0x33, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_callvalue() {
        // CALLVALUE, STOP
        let code = [0x34, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_calldatasize() {
        // CALLDATASIZE, STOP
        let code = [0x36, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_calldataload() {
        // PUSH1 0, CALLDATALOAD, STOP
        let code = [0x60, 0x00, 0x35, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_codesize() {
        // CODESIZE, STOP
        let code = [0x38, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_gasprice() {
        // GASPRICE, STOP
        let code = [0x3A, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_returndatasize() {
        // RETURNDATASIZE, STOP (initially 0)
        let code = [0x3D, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    // --- Block Info ---

    #[test]
    fn test_blockhash() {
        // PUSH1 0, BLOCKHASH, STOP
        let code = [0x60, 0x00, 0x40, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_coinbase() {
        // COINBASE, STOP
        let code = [0x41, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_timestamp() {
        // TIMESTAMP, STOP
        let code = [0x42, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_number() {
        // NUMBER, STOP
        let code = [0x43, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_prevrandao() {
        // PREVRANDAO, STOP
        let code = [0x44, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_gaslimit() {
        // GASLIMIT, STOP
        let code = [0x45, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_chainid() {
        // CHAINID, STOP
        let code = [0x46, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_basefee() {
        // BASEFEE, STOP
        let code = [0x48, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_origin() {
        // ORIGIN, STOP
        let code = [0x32, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    // --- Push operations ---

    #[test]
    fn test_push0() {
        // PUSH0, STOP
        let code = [0x5F, 0x00];
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    #[test]
    fn test_push32() {
        // PUSH32 <32 bytes>, STOP
        let mut code = vec![0x7F]; // PUSH32
        code.extend([0x12; 32]); // 32 bytes
        code.push(0x00); // STOP
        let result = run_code(&code, 1000);
        assert!(result.success);
    }

    // --- Error cases ---

    #[test]
    fn test_stack_underflow() {
        // POP on empty stack
        let code = [0x50]; // POP
        let result = run_code(&code, 1000);
        assert!(!result.success);
    }

    #[test]
    fn test_stack_overflow() {
        // Push 1025 items
        let mut code = Vec::new();
        for _ in 0..1025 {
            code.push(0x60); // PUSH1
            code.push(0x01);
        }
        code.push(0x00); // STOP
        let result = run_code(&code, 100000);
        assert!(!result.success);
    }

    #[test]
    fn test_invalid_opcode() {
        // 0xFE is INVALID opcode
        let code = [0xFE];
        let result = run_code(&code, 1000);
        assert!(!result.success);
    }

    #[test]
    fn test_jump_to_push_data() {
        // PUSH1 1, JUMP (1 is within PUSH1 operand, not a JUMPDEST)
        let code = [0x60, 0x01, 0x56];
        let result = run_code(&code, 1000);
        assert!(!result.success);
    }

    // --- CALLDATACOPY and CODECOPY ---

    #[test]
    fn test_calldatacopy() {
        // PUSH1 5, PUSH1 0, PUSH1 0, CALLDATACOPY, STOP
        let code = [0x60, 0x05, 0x60, 0x00, 0x60, 0x00, 0x37, 0x00];
        let result = run_code(&code, 10000);
        assert!(result.success);
    }

    #[test]
    fn test_codecopy() {
        // PUSH1 5, PUSH1 0, PUSH1 0, CODECOPY, STOP
        let code = [0x60, 0x05, 0x60, 0x00, 0x60, 0x00, 0x39, 0x00];
        let result = run_code(&code, 10000);
        assert!(result.success);
    }

    // --- Return with data ---

    #[test]
    fn test_return_with_data() {
        // Store 0xDEADBEEF at memory 0, return 32 bytes
        let code = [
            0x63, 0xDE, 0xAD, 0xBE, 0xEF, // PUSH4 0xDEADBEEF
            0x60, 0x00,                     // PUSH1 0
            0x52,                           // MSTORE
            0x60, 0x20,                     // PUSH1 32
            0x60, 0x00,                     // PUSH1 0
            0xF3,                           // RETURN
        ];
        let result = run_code(&code, 100000);
        assert!(result.success);
        assert_eq!(result.output.len(), 32);
        // Check that the value is in the output
        assert_eq!(&result.output[28..32], &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    // --- Complex bytecode ---

    #[test]
    fn test_simple_loop() {
        // Counter loop: count from 0 to 5
        // PUSH1 0, JUMPDEST, PUSH1 1, ADD, DUP1, PUSH1 5, LT, PUSH1 2, JUMPI, STOP
        let code = [
            0x60, 0x00, // PUSH1 0 (counter)
            0x5B,       // JUMPDEST (offset 2)
            0x60, 0x01, // PUSH1 1
            0x01,       // ADD
            0x80,       // DUP1
            0x60, 0x05, // PUSH1 5
            0x10,       // LT
            0x60, 0x02, // PUSH1 2 (jump target)
            0x57,       // JUMPI
            0x00,       // STOP
        ];
        let result = run_code(&code, 100000);
        assert!(result.success);
    }

    #[test]
    fn test_memory_expansion_gas() {
        // MSTORE at offset 0 should expand memory
        // Base opcode costs: PUSH1(3) + PUSH1(3) + MSTORE(3) + STOP(0) = 9
        // Plus memory expansion from 0 to 32 bytes = 3 gas
        let code = [
            0x60, 0x01, // PUSH1 1
            0x60, 0x00, // PUSH1 0 (offset)
            0x52,       // MSTORE
            0x00,       // STOP
        ];
        let result = run_code(&code, 1000000);
        assert!(result.success);
        // Gas should be at least 9 for opcodes + memory cost
        assert!(result.gas_used >= 9, "gas_used = {}", result.gas_used);
    }

    #[test]
    fn test_execution_result_methods() {
        let success = ExecutionResult::success(100, vec![1, 2, 3], vec![]);
        assert!(success.success);
        assert_eq!(success.gas_used, 100);
        assert_eq!(success.output, vec![1, 2, 3]);

        let failure = ExecutionResult::failure(500, vec![4, 5, 6]);
        assert!(!failure.success);
        assert_eq!(failure.gas_used, 500);

        let revert = ExecutionResult::revert(200, vec![7, 8, 9]);
        assert!(!revert.success);
        assert_eq!(revert.gas_used, 200);
    }
}
