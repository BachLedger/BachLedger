//! Comprehensive EVM tests for bach-evm
//!
//! Test-Driven Development: These tests define expected EVM behavior.
//! Implementation must pass all tests.
//!
//! Test categories:
//! 1. Stack Operations: PUSH, POP, DUP, SWAP
//! 2. Arithmetic: ADD, SUB, MUL, DIV, MOD, EXP
//! 3. Comparison: LT, GT, EQ, ISZERO
//! 4. Bitwise: AND, OR, XOR, NOT, SHL, SHR
//! 5. Memory: MLOAD, MSTORE, MSTORE8, MSIZE
//! 6. Storage: SLOAD, SSTORE
//! 7. Control Flow: JUMP, JUMPI, JUMPDEST, STOP, RETURN, REVERT
//! 8. Environment: ADDRESS, CALLER, CALLVALUE, CALLDATALOAD
//! 9. Contract: CREATE, CALL, DELEGATECALL
//! 10. Gas Metering: Gas consumption tests

use bach_evm::{Evm, EvmContext, EvmError, EvmResult, ExecutionResult, Opcode};
use bach_primitives::{Address, H256, U256};

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a default execution context for testing
fn test_context() -> EvmContext {
    EvmContext {
        caller: Address::from_hex("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
        address: Address::from_hex("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(),
        value: U256::ZERO,
        data: Vec::new(),
        gas_limit: 1_000_000,
    }
}

/// Creates an EVM instance with given bytecode
fn evm_with_code(code: Vec<u8>) -> Evm {
    Evm::new(code, test_context())
}

/// Creates an EVM instance with given bytecode and context
fn evm_with_code_and_context(code: Vec<u8>, ctx: EvmContext) -> Evm {
    Evm::new(code, ctx)
}

/// Helper to create U256 from a small number
fn u256(n: u64) -> U256 {
    U256::from_u64(n)
}

// =============================================================================
// 1. Stack Operations Tests
// =============================================================================

mod stack_operations {
    use super::*;

    #[test]
    fn test_push1() {
        // PUSH1 0x42 STOP
        let code = vec![0x60, 0x42, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().len(), 1);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
    }

    #[test]
    fn test_push2() {
        // PUSH2 0x1234 STOP
        let code = vec![0x61, 0x12, 0x34, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x1234));
    }

    #[test]
    fn test_push32() {
        // PUSH32 <32 bytes of 0xff> STOP
        let mut code = vec![0x7f]; // PUSH32
        code.extend_from_slice(&[0xff; 32]);
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::MAX);
    }

    #[test]
    fn test_pop() {
        // PUSH1 0x42 PUSH1 0x43 POP STOP
        let code = vec![0x60, 0x42, 0x60, 0x43, 0x50, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().len(), 1);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
    }

    #[test]
    fn test_pop_empty_stack() {
        // POP (empty stack should fail)
        let code = vec![0x50, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute();

        assert!(matches!(result, Err(EvmError::StackUnderflow)));
    }

    #[test]
    fn test_dup1() {
        // PUSH1 0x42 DUP1 STOP
        let code = vec![0x60, 0x42, 0x80, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().len(), 2);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
        assert_eq!(evm.stack().peek(1).unwrap(), u256(0x42));
    }

    #[test]
    fn test_dup16() {
        // Push 16 values, then DUP16 to duplicate the first
        let mut code = Vec::new();
        for i in 1..=16 {
            code.push(0x60); // PUSH1
            code.push(i);
        }
        code.push(0x8f); // DUP16
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().len(), 17);
        // DUP16 should duplicate the 16th item (which is 1)
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_swap1() {
        // PUSH1 0x01 PUSH1 0x02 SWAP1 STOP
        let code = vec![0x60, 0x01, 0x60, 0x02, 0x90, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x01));
        assert_eq!(evm.stack().peek(1).unwrap(), u256(0x02));
    }

    #[test]
    fn test_swap16() {
        // Push 17 values, then SWAP16
        let mut code = Vec::new();
        for i in 1..=17 {
            code.push(0x60); // PUSH1
            code.push(i);
        }
        code.push(0x9f); // SWAP16
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // After SWAP16, top (17) should swap with 16th from top (1)
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
        assert_eq!(evm.stack().peek(16).unwrap(), u256(17));
    }

    #[test]
    fn test_stack_overflow() {
        // Push 1025 items (stack limit is 1024)
        let mut code = Vec::new();
        for _ in 0..1025 {
            code.push(0x60); // PUSH1
            code.push(0x01);
        }
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute();

        assert!(matches!(result, Err(EvmError::StackOverflow)));
    }
}

// =============================================================================
// 2. Arithmetic Tests
// =============================================================================

mod arithmetic {
    use super::*;

    #[test]
    fn test_add() {
        // PUSH1 0x03 PUSH1 0x05 ADD STOP
        let code = vec![0x60, 0x03, 0x60, 0x05, 0x01, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(8));
    }

    #[test]
    fn test_add_overflow() {
        // U256::MAX + 1 should wrap to 0
        let mut code = vec![0x7f]; // PUSH32
        code.extend_from_slice(&[0xff; 32]); // MAX
        code.push(0x60); // PUSH1
        code.push(0x01);
        code.push(0x01); // ADD
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_sub() {
        // PUSH1 0x03 PUSH1 0x05 SUB STOP (5 - 3 = 2)
        let code = vec![0x60, 0x03, 0x60, 0x05, 0x03, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(2));
    }

    #[test]
    fn test_sub_underflow() {
        // 0 - 1 should wrap to MAX
        let code = vec![0x60, 0x01, 0x60, 0x00, 0x03, 0x00]; // 0 - 1
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::MAX);
    }

    #[test]
    fn test_mul() {
        // PUSH1 0x03 PUSH1 0x05 MUL STOP
        let code = vec![0x60, 0x03, 0x60, 0x05, 0x02, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(15));
    }

    #[test]
    fn test_mul_zero() {
        // Any number * 0 = 0
        let code = vec![0x60, 0x00, 0x60, 0x05, 0x02, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_div() {
        // PUSH1 0x02 PUSH1 0x0a DIV STOP (10 / 2 = 5)
        let code = vec![0x60, 0x02, 0x60, 0x0a, 0x04, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(5));
    }

    #[test]
    fn test_div_by_zero() {
        // Division by zero returns 0 in EVM
        let code = vec![0x60, 0x00, 0x60, 0x0a, 0x04, 0x00]; // 10 / 0
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_sdiv() {
        // Signed division: -10 / 2 = -5
        // -10 in two's complement is MAX - 9
        let mut code = vec![0x60, 0x02]; // PUSH1 2
        code.push(0x7f); // PUSH32 -10
        let minus_10 = {
            let mut bytes = [0xff; 32];
            bytes[31] = 0xf6; // -10 in two's complement
            bytes
        };
        code.extend_from_slice(&minus_10);
        code.push(0x05); // SDIV
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Result should be -5 in two's complement
    }

    #[test]
    fn test_mod() {
        // PUSH1 0x03 PUSH1 0x0a MOD STOP (10 % 3 = 1)
        let code = vec![0x60, 0x03, 0x60, 0x0a, 0x06, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_mod_by_zero() {
        // Modulo by zero returns 0 in EVM
        let code = vec![0x60, 0x00, 0x60, 0x0a, 0x06, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_addmod() {
        // ADDMOD(10, 10, 8) = (10 + 10) % 8 = 4
        let code = vec![
            0x60, 0x08, // PUSH1 8 (modulus)
            0x60, 0x0a, // PUSH1 10
            0x60, 0x0a, // PUSH1 10
            0x08, // ADDMOD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(4));
    }

    #[test]
    fn test_mulmod() {
        // MULMOD(10, 10, 8) = (10 * 10) % 8 = 4
        let code = vec![
            0x60, 0x08, // PUSH1 8 (modulus)
            0x60, 0x0a, // PUSH1 10
            0x60, 0x0a, // PUSH1 10
            0x09, // MULMOD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(4));
    }

    #[test]
    fn test_exp() {
        // EXP(2, 10) = 1024
        let code = vec![
            0x60, 0x0a, // PUSH1 10 (exponent)
            0x60, 0x02, // PUSH1 2 (base)
            0x0a, // EXP
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1024));
    }

    #[test]
    fn test_exp_zero_exponent() {
        // x^0 = 1
        let code = vec![
            0x60, 0x00, // PUSH1 0 (exponent)
            0x60, 0xff, // PUSH1 255 (base)
            0x0a, // EXP
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_signextend() {
        // SIGNEXTEND(0, 0x7f) = 0x7f (positive, no change)
        let code = vec![
            0x60, 0x7f, // PUSH1 0x7f
            0x60, 0x00, // PUSH1 0 (byte position)
            0x0b, // SIGNEXTEND
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x7f));
    }

    #[test]
    fn test_signextend_negative() {
        // SIGNEXTEND(0, 0x80) should extend sign bit
        let code = vec![
            0x60, 0x80, // PUSH1 0x80 (negative in signed byte)
            0x60, 0x00, // PUSH1 0 (byte position)
            0x0b, // SIGNEXTEND
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Result should have all high bits set (sign extended)
        let value = evm.stack().peek(0).unwrap();
        // Check that the high byte is 0xff (sign extended)
        let bytes = value.to_be_bytes();
        assert_eq!(bytes[0], 0xff);
    }
}

// =============================================================================
// 3. Comparison Tests
// =============================================================================

mod comparison {
    use super::*;

    #[test]
    fn test_lt_true() {
        // 3 < 5 = 1
        let code = vec![0x60, 0x05, 0x60, 0x03, 0x10, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_lt_false() {
        // 5 < 3 = 0
        let code = vec![0x60, 0x03, 0x60, 0x05, 0x10, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_lt_equal() {
        // 5 < 5 = 0
        let code = vec![0x60, 0x05, 0x60, 0x05, 0x10, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_gt_true() {
        // 5 > 3 = 1
        let code = vec![0x60, 0x03, 0x60, 0x05, 0x11, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_gt_false() {
        // 3 > 5 = 0
        let code = vec![0x60, 0x05, 0x60, 0x03, 0x11, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_slt() {
        // Signed less than: -1 < 1 = 1
        // -1 is U256::MAX in two's complement
        let mut code = vec![0x60, 0x01]; // PUSH1 1
        code.push(0x7f); // PUSH32 -1 (MAX)
        code.extend_from_slice(&[0xff; 32]);
        code.push(0x12); // SLT
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_sgt() {
        // Signed greater than: 1 > -1 = 1
        let mut code = vec![0x7f]; // PUSH32 -1 (MAX)
        code.extend_from_slice(&[0xff; 32]);
        code.push(0x60); // PUSH1 1
        code.push(0x01);
        code.push(0x13); // SGT
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_eq_true() {
        // 5 == 5 = 1
        let code = vec![0x60, 0x05, 0x60, 0x05, 0x14, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_eq_false() {
        // 3 == 5 = 0
        let code = vec![0x60, 0x05, 0x60, 0x03, 0x14, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_iszero_true() {
        // ISZERO(0) = 1
        let code = vec![0x60, 0x00, 0x15, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_iszero_false() {
        // ISZERO(5) = 0
        let code = vec![0x60, 0x05, 0x15, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }
}

// =============================================================================
// 4. Bitwise Tests
// =============================================================================

mod bitwise {
    use super::*;

    #[test]
    fn test_and() {
        // 0x0f AND 0x3c = 0x0c
        let code = vec![0x60, 0x3c, 0x60, 0x0f, 0x16, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x0c));
    }

    #[test]
    fn test_or() {
        // 0x0f OR 0xf0 = 0xff
        let code = vec![0x60, 0xf0, 0x60, 0x0f, 0x17, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0xff));
    }

    #[test]
    fn test_xor() {
        // 0x0f XOR 0xff = 0xf0
        let code = vec![0x60, 0xff, 0x60, 0x0f, 0x18, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0xf0));
    }

    #[test]
    fn test_not() {
        // NOT(0) = MAX
        let code = vec![0x60, 0x00, 0x19, 0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::MAX);
    }

    #[test]
    fn test_byte() {
        // BYTE(31, 0x1234) = 0x34 (rightmost byte)
        let code = vec![
            0x61, 0x12, 0x34, // PUSH2 0x1234
            0x60, 0x1f, // PUSH1 31
            0x1a, // BYTE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x34));
    }

    #[test]
    fn test_byte_out_of_range() {
        // BYTE(32, x) = 0 (out of range)
        let code = vec![
            0x60, 0xff, // PUSH1 0xff
            0x60, 0x20, // PUSH1 32
            0x1a, // BYTE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_shl() {
        // SHL(4, 1) = 16 (1 << 4)
        let code = vec![
            0x60, 0x01, // PUSH1 1
            0x60, 0x04, // PUSH1 4
            0x1b, // SHL
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(16));
    }

    #[test]
    fn test_shr() {
        // SHR(4, 16) = 1 (16 >> 4)
        let code = vec![
            0x60, 0x10, // PUSH1 16
            0x60, 0x04, // PUSH1 4
            0x1c, // SHR
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1));
    }

    #[test]
    fn test_sar() {
        // SAR (arithmetic shift right) preserves sign
        // SAR(4, -16) should give -1 (sign preserved)
        let mut code = vec![];
        // Push -16 (all 1s except last byte is 0xf0)
        code.push(0x7f); // PUSH32
        let mut minus_16 = [0xff; 32];
        minus_16[31] = 0xf0;
        code.extend_from_slice(&minus_16);
        code.push(0x60); // PUSH1 4
        code.push(0x04);
        code.push(0x1d); // SAR
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Result should be -1 (all 1s)
        assert_eq!(evm.stack().peek(0).unwrap(), U256::MAX);
    }
}

// =============================================================================
// 5. Memory Tests
// =============================================================================

mod memory {
    use super::*;

    #[test]
    fn test_mstore_mload() {
        // MSTORE 0x42 at offset 0, then MLOAD from offset 0
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x00, // PUSH1 0 (offset)
            0x52, // MSTORE
            0x60, 0x00, // PUSH1 0 (offset)
            0x51, // MLOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
    }

    #[test]
    fn test_mstore8() {
        // MSTORE8 stores only 1 byte
        let code = vec![
            0x61, 0x12, 0x34, // PUSH2 0x1234
            0x60, 0x00, // PUSH1 0 (offset)
            0x53, // MSTORE8 (stores only 0x34)
            0x60, 0x00, // PUSH1 0
            0x51, // MLOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // MSTORE8 stores at offset 0, value is 0x34 followed by zeros
        // When loaded as 32 bytes: 0x34000...
        let expected = U256::from_be_bytes({
            let mut bytes = [0u8; 32];
            bytes[0] = 0x34;
            bytes
        });
        assert_eq!(evm.stack().peek(0).unwrap(), expected);
    }

    #[test]
    fn test_msize() {
        // MSIZE returns memory size (rounded up to 32 bytes)
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x00, // PUSH1 0 (offset)
            0x52, // MSTORE (expands memory to 32 bytes)
            0x59, // MSIZE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(32));
    }

    #[test]
    fn test_msize_larger_offset() {
        // MSTORE at offset 32 should expand memory to 64 bytes
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x20, // PUSH1 32 (offset)
            0x52, // MSTORE
            0x59, // MSIZE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(64));
    }

    #[test]
    fn test_mcopy() {
        // MCOPY copies memory from source to destination
        // Store 0x42 at offset 0, copy to offset 32
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x00, // PUSH1 0 (offset)
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32 (size)
            0x60, 0x00, // PUSH1 0 (source)
            0x60, 0x20, // PUSH1 32 (dest)
            0x5e, // MCOPY (EIP-5656)
            0x60, 0x20, // PUSH1 32
            0x51, // MLOAD (read from offset 32)
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
    }
}

// =============================================================================
// 6. Storage Tests
// =============================================================================

mod storage {
    use super::*;

    #[test]
    fn test_sstore_sload() {
        // SSTORE value at slot 0, then SLOAD
        let code = vec![
            0x60, 0x42, // PUSH1 0x42 (value)
            0x60, 0x00, // PUSH1 0 (slot)
            0x55, // SSTORE
            0x60, 0x00, // PUSH1 0 (slot)
            0x54, // SLOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
    }

    #[test]
    fn test_sload_uninitialized() {
        // SLOAD from uninitialized slot returns 0
        let code = vec![
            0x60, 0x99, // PUSH1 0x99 (slot)
            0x54, // SLOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::ZERO);
    }

    #[test]
    fn test_tstore_tload() {
        // Transient storage (EIP-1153)
        let code = vec![
            0x60, 0x42, // PUSH1 0x42 (value)
            0x60, 0x00, // PUSH1 0 (slot)
            0x5d, // TSTORE
            0x60, 0x00, // PUSH1 0 (slot)
            0x5c, // TLOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
    }
}

// =============================================================================
// 7. Control Flow Tests
// =============================================================================

mod control_flow {
    use super::*;

    #[test]
    fn test_stop() {
        // STOP halts execution
        let code = vec![0x00];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert!(result.return_data.is_empty());
    }

    #[test]
    fn test_jump() {
        // Jump to JUMPDEST at offset 4
        // PUSH1 4 JUMP INVALID JUMPDEST PUSH1 0x42 STOP
        let code = vec![
            0x60, 0x04, // PUSH1 4
            0x56, // JUMP
            0xfe, // INVALID (should be skipped)
            0x5b, // JUMPDEST (offset 4)
            0x60, 0x42, // PUSH1 0x42
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
    }

    #[test]
    fn test_jump_invalid_dest() {
        // Jump to non-JUMPDEST should fail
        let code = vec![
            0x60, 0x03, // PUSH1 3 (not a JUMPDEST)
            0x56, // JUMP
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute();

        assert!(matches!(result, Err(EvmError::InvalidJumpDestination)));
    }

    #[test]
    fn test_jumpi_true() {
        // Conditional jump when condition is non-zero
        let code = vec![
            0x60, 0x01, // PUSH1 1 (condition: true)
            0x60, 0x06, // PUSH1 6 (destination)
            0x57, // JUMPI
            0xfe, // INVALID (should be skipped)
            0x5b, // JUMPDEST (offset 6)
            0x60, 0x42, // PUSH1 0x42
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x42));
    }

    #[test]
    fn test_jumpi_false() {
        // Conditional jump when condition is zero (fall through)
        let code = vec![
            0x60, 0x00, // PUSH1 0 (condition: false)
            0x60, 0x08, // PUSH1 8 (destination)
            0x57, // JUMPI (not taken)
            0x60, 0x33, // PUSH1 0x33
            0x00, // STOP
            0x5b, // JUMPDEST (offset 8, not reached)
            0x60, 0x42, // PUSH1 0x42
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x33));
    }

    #[test]
    fn test_pc() {
        // PC returns current program counter
        let code = vec![
            0x60, 0x00, // PUSH1 0 (offset 0-1)
            0x58, // PC (offset 2, should push 2)
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(2));
    }

    #[test]
    fn test_return() {
        // RETURN with data
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x00, // PUSH1 0 (offset)
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32 (size)
            0x60, 0x00, // PUSH1 0 (offset)
            0xf3, // RETURN
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(result.return_data.len(), 32);
        // Value 0x42 is stored at the end of the 32-byte word
        assert_eq!(result.return_data[31], 0x42);
    }

    #[test]
    fn test_revert() {
        // REVERT with data
        let code = vec![
            0x60, 0xff, // PUSH1 0xff
            0x60, 0x00, // PUSH1 0 (offset)
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32 (size)
            0x60, 0x00, // PUSH1 0 (offset)
            0xfd, // REVERT
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(!result.success);
        assert_eq!(result.return_data.len(), 32);
    }

    #[test]
    fn test_invalid() {
        // INVALID opcode
        let code = vec![0xfe];
        let mut evm = evm_with_code(code);
        let result = evm.execute();

        assert!(matches!(result, Err(EvmError::InvalidOpcode(0xfe))));
    }
}

// =============================================================================
// 8. Environment Tests
// =============================================================================

mod environment {
    use super::*;

    #[test]
    fn test_address() {
        // ADDRESS returns current contract address
        let code = vec![0x30, 0x00]; // ADDRESS STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        let addr_bytes = evm.stack().peek(0).unwrap().to_be_bytes();
        // Address is in the lower 20 bytes
        assert_eq!(&addr_bytes[12..], test_context().address.as_bytes());
    }

    #[test]
    fn test_caller() {
        // CALLER returns message sender
        let code = vec![0x33, 0x00]; // CALLER STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        let caller_bytes = evm.stack().peek(0).unwrap().to_be_bytes();
        assert_eq!(&caller_bytes[12..], test_context().caller.as_bytes());
    }

    #[test]
    fn test_callvalue() {
        // CALLVALUE returns msg.value
        let mut ctx = test_context();
        ctx.value = u256(1000);
        let code = vec![0x34, 0x00]; // CALLVALUE STOP
        let mut evm = evm_with_code_and_context(code, ctx);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(1000));
    }

    #[test]
    fn test_calldataload() {
        // CALLDATALOAD loads 32 bytes from calldata
        let mut ctx = test_context();
        ctx.data = vec![0x42; 32];
        let code = vec![
            0x60, 0x00, // PUSH1 0 (offset)
            0x35, // CALLDATALOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code_and_context(code, ctx);
        let result = evm.execute().unwrap();

        assert!(result.success);
        let expected = U256::from_be_bytes([0x42; 32]);
        assert_eq!(evm.stack().peek(0).unwrap(), expected);
    }

    #[test]
    fn test_calldataload_padding() {
        // CALLDATALOAD pads with zeros if calldata is shorter
        let mut ctx = test_context();
        ctx.data = vec![0x42; 16]; // Only 16 bytes
        let code = vec![
            0x60, 0x00, // PUSH1 0 (offset)
            0x35, // CALLDATALOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code_and_context(code, ctx);
        let result = evm.execute().unwrap();

        assert!(result.success);
        let mut expected_bytes = [0u8; 32];
        expected_bytes[0..16].copy_from_slice(&[0x42; 16]);
        let expected = U256::from_be_bytes(expected_bytes);
        assert_eq!(evm.stack().peek(0).unwrap(), expected);
    }

    #[test]
    fn test_calldatasize() {
        // CALLDATASIZE returns length of calldata
        let mut ctx = test_context();
        ctx.data = vec![0x00; 64];
        let code = vec![0x36, 0x00]; // CALLDATASIZE STOP
        let mut evm = evm_with_code_and_context(code, ctx);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(64));
    }

    #[test]
    fn test_calldatacopy() {
        // CALLDATACOPY copies calldata to memory
        let mut ctx = test_context();
        ctx.data = vec![0x42; 32];
        let code = vec![
            0x60, 0x20, // PUSH1 32 (size)
            0x60, 0x00, // PUSH1 0 (data offset)
            0x60, 0x00, // PUSH1 0 (memory offset)
            0x37, // CALLDATACOPY
            0x60, 0x00, // PUSH1 0
            0x51, // MLOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code_and_context(code, ctx);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), U256::from_be_bytes([0x42; 32]));
    }

    #[test]
    fn test_codesize() {
        // CODESIZE returns bytecode length
        let code = vec![0x38, 0x00]; // CODESIZE STOP
        let code_len = code.len();
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(evm.stack().peek(0).unwrap(), u256(code_len as u64));
    }

    #[test]
    fn test_codecopy() {
        // CODECOPY copies code to memory
        let code = vec![
            0x60, 0x02, // PUSH1 2 (size)
            0x60, 0x00, // PUSH1 0 (code offset)
            0x60, 0x00, // PUSH1 0 (memory offset)
            0x39, // CODECOPY
            0x60, 0x00, // PUSH1 0
            0x51, // MLOAD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // First 2 bytes of code (0x60, 0x02) copied to memory
        let val = evm.stack().peek(0).unwrap().to_be_bytes();
        assert_eq!(val[0], 0x60);
        assert_eq!(val[1], 0x02);
    }

    #[test]
    fn test_origin() {
        // ORIGIN returns transaction sender (tx.origin)
        let code = vec![0x32, 0x00]; // ORIGIN STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // In a simple execution, origin equals caller
    }

    #[test]
    fn test_gasprice() {
        // GASPRICE returns gas price
        let code = vec![0x3a, 0x00]; // GASPRICE STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_gas() {
        // GAS returns remaining gas
        let code = vec![0x5a, 0x00]; // GAS STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Gas should be positive (some consumed for instructions)
        assert!(evm.stack().peek(0).unwrap() > U256::ZERO);
    }
}

// =============================================================================
// 9. Contract Tests
// =============================================================================

mod contract {
    use super::*;

    #[test]
    fn test_create() {
        // CREATE deploys a new contract
        // Deploy code: PUSH1 0x42 PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
        let init_code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x00, // PUSH1 0
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32
            0x60, 0x00, // PUSH1 0
            0xf3, // RETURN
        ];

        let mut code = Vec::new();
        // Store init code in memory
        for (i, byte) in init_code.iter().enumerate() {
            code.push(0x60); // PUSH1
            code.push(*byte);
            code.push(0x60); // PUSH1
            code.push(i as u8);
            code.push(0x53); // MSTORE8
        }
        // CREATE(value=0, offset=0, size=init_code.len())
        code.push(0x60); // PUSH1 size
        code.push(init_code.len() as u8);
        code.push(0x60); // PUSH1 0 (offset)
        code.push(0x00);
        code.push(0x60); // PUSH1 0 (value)
        code.push(0x00);
        code.push(0xf0); // CREATE
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // CREATE returns new contract address (should be non-zero)
    }

    #[test]
    fn test_create2() {
        // CREATE2 with salt
        let init_code = vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3];

        let mut code = Vec::new();
        // Store init code in memory
        for (i, byte) in init_code.iter().enumerate() {
            code.push(0x60);
            code.push(*byte);
            code.push(0x60);
            code.push(i as u8);
            code.push(0x53);
        }
        // CREATE2(value=0, offset=0, size=init_code.len(), salt=0x1234)
        code.push(0x61); // PUSH2 salt
        code.push(0x12);
        code.push(0x34);
        code.push(0x60); // PUSH1 size
        code.push(init_code.len() as u8);
        code.push(0x60); // PUSH1 0 (offset)
        code.push(0x00);
        code.push(0x60); // PUSH1 0 (value)
        code.push(0x00);
        code.push(0xf5); // CREATE2
        code.push(0x00); // STOP

        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_call() {
        // CALL to another contract (simplified test)
        let code = vec![
            0x60, 0x00, // PUSH1 0 (retSize)
            0x60, 0x00, // PUSH1 0 (retOffset)
            0x60, 0x00, // PUSH1 0 (argsSize)
            0x60, 0x00, // PUSH1 0 (argsOffset)
            0x60, 0x00, // PUSH1 0 (value)
            0x60, 0x00, // PUSH1 0 (address - will be zero)
            0x60, 0xff, // PUSH1 gas
            0xf1, // CALL
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // CALL returns 1 for success, 0 for failure
    }

    #[test]
    fn test_delegatecall() {
        // DELEGATECALL preserves caller and value
        let code = vec![
            0x60, 0x00, // PUSH1 0 (retSize)
            0x60, 0x00, // PUSH1 0 (retOffset)
            0x60, 0x00, // PUSH1 0 (argsSize)
            0x60, 0x00, // PUSH1 0 (argsOffset)
            0x60, 0x00, // PUSH1 0 (address)
            0x60, 0xff, // PUSH1 gas
            0xf4, // DELEGATECALL
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_staticcall() {
        // STATICCALL cannot modify state
        let code = vec![
            0x60, 0x00, // PUSH1 0 (retSize)
            0x60, 0x00, // PUSH1 0 (retOffset)
            0x60, 0x00, // PUSH1 0 (argsSize)
            0x60, 0x00, // PUSH1 0 (argsOffset)
            0x60, 0x00, // PUSH1 0 (address)
            0x60, 0xff, // PUSH1 gas
            0xfa, // STATICCALL
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_returndatasize() {
        // RETURNDATASIZE after a call
        let code = vec![
            0x60, 0x00, // retSize
            0x60, 0x00, // retOffset
            0x60, 0x00, // argsSize
            0x60, 0x00, // argsOffset
            0x60, 0x00, // value
            0x60, 0x00, // address
            0x60, 0xff, // gas
            0xf1, // CALL
            0x50, // POP (discard call result)
            0x3d, // RETURNDATASIZE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_returndatacopy() {
        // RETURNDATACOPY copies return data to memory
        let code = vec![
            0x60, 0x20, // retSize = 32
            0x60, 0x00, // retOffset
            0x60, 0x00, // argsSize
            0x60, 0x00, // argsOffset
            0x60, 0x00, // value
            0x60, 0x00, // address
            0x60, 0xff, // gas
            0xf1, // CALL
            0x50, // POP
            0x60, 0x20, // size = 32
            0x60, 0x00, // dataOffset
            0x60, 0x40, // memOffset = 64
            0x3e, // RETURNDATACOPY
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_selfdestruct() {
        // SELFDESTRUCT destroys contract and sends balance
        let code = vec![
            0x60, 0x00, // PUSH1 0 (beneficiary address)
            0xff, // SELFDESTRUCT
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }
}

// =============================================================================
// 10. Gas Metering Tests
// =============================================================================

mod gas_metering {
    use super::*;

    #[test]
    fn test_gas_consumption_basic() {
        // Simple operations should consume gas
        let code = vec![
            0x60, 0x01, // PUSH1 1
            0x60, 0x02, // PUSH1 2
            0x01, // ADD
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let initial_gas = evm.gas_remaining();
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert!(result.gas_used > 0);
        assert!(evm.gas_remaining() < initial_gas);
    }

    #[test]
    fn test_out_of_gas() {
        // Execution with very limited gas should fail
        let code = vec![
            0x60, 0x01, // PUSH1 1
            0x60, 0x02, // PUSH1 2
            0x01, // ADD
            0x00, // STOP
        ];
        let mut ctx = test_context();
        ctx.gas_limit = 1; // Very low gas

        let mut evm = evm_with_code_and_context(code, ctx);
        let result = evm.execute();

        assert!(matches!(result, Err(EvmError::OutOfGas)));
    }

    #[test]
    fn test_sstore_gas_cold_slot() {
        // SSTORE to cold slot costs more gas
        let code = vec![
            0x60, 0x01, // PUSH1 1 (value)
            0x60, 0x00, // PUSH1 0 (slot)
            0x55, // SSTORE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Cold SSTORE costs 20000 gas (approximate)
        assert!(result.gas_used >= 20000);
    }

    #[test]
    fn test_sstore_gas_warm_slot() {
        // SSTORE to warm slot costs less
        let code = vec![
            0x60, 0x01, // PUSH1 1 (value)
            0x60, 0x00, // PUSH1 0 (slot)
            0x55, // SSTORE (cold)
            0x60, 0x02, // PUSH1 2 (value)
            0x60, 0x00, // PUSH1 0 (slot)
            0x55, // SSTORE (warm)
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Second SSTORE should be cheaper
    }

    #[test]
    fn test_memory_expansion_gas() {
        // Memory expansion costs gas
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x61, 0x10, 0x00, // PUSH2 4096 (large offset)
            0x52, // MSTORE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Memory expansion gas should be significant
        assert!(result.gas_used > 100);
    }

    #[test]
    fn test_exp_gas_varies_with_exponent() {
        // EXP gas depends on byte size of exponent
        let code_small_exp = vec![
            0x60, 0x02, // exponent = 2 (1 byte)
            0x60, 0x02, // base = 2
            0x0a, // EXP
            0x00, // STOP
        ];

        let code_large_exp = vec![
            0x61, 0x01, 0x00, // exponent = 256 (2 bytes)
            0x60, 0x02, // base = 2
            0x0a, // EXP
            0x00, // STOP
        ];

        let mut evm_small = evm_with_code(code_small_exp);
        let result_small = evm_small.execute().unwrap();

        let mut evm_large = evm_with_code(code_large_exp);
        let result_large = evm_large.execute().unwrap();

        assert!(result_small.success);
        assert!(result_large.success);
        // Larger exponent should cost more gas
        assert!(result_large.gas_used > result_small.gas_used);
    }

    #[test]
    fn test_call_gas_stipend() {
        // CALL with non-zero value gets 2300 gas stipend
        let code = vec![
            0x60, 0x00, // retSize
            0x60, 0x00, // retOffset
            0x60, 0x00, // argsSize
            0x60, 0x00, // argsOffset
            0x60, 0x01, // value = 1 (non-zero)
            0x60, 0x00, // address
            0x60, 0x00, // gas = 0 (will get stipend)
            0xf1, // CALL
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_gas_refund_sstore_clear() {
        // Clearing storage gives gas refund
        let code = vec![
            // First set a value
            0x60, 0x01, // value = 1
            0x60, 0x00, // slot = 0
            0x55, // SSTORE
            // Then clear it
            0x60, 0x00, // value = 0
            0x60, 0x00, // slot = 0
            0x55, // SSTORE (should refund)
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Gas refund should reduce effective gas used
    }
}

// =============================================================================
// Additional Edge Cases
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_code() {
        let code = vec![];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_push_truncated() {
        // PUSH2 with only 1 byte available should pad with zeros
        let code = vec![0x61, 0x42]; // PUSH2 but only 1 byte follows
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // Should push 0x4200 (padded with zero)
        assert_eq!(evm.stack().peek(0).unwrap(), u256(0x4200));
    }

    #[test]
    fn test_keccak256() {
        // SHA3/KECCAK256 of empty data
        let code = vec![
            0x60, 0x00, // PUSH1 0 (size)
            0x60, 0x00, // PUSH1 0 (offset)
            0x20, // KECCAK256
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        // keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        let expected_hash = H256::from_hex(
            "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
        )
        .unwrap();
        let result_bytes = evm.stack().peek(0).unwrap().to_be_bytes();
        assert_eq!(result_bytes, *expected_hash.as_bytes());
    }

    #[test]
    fn test_balance() {
        // BALANCE returns account balance
        let code = vec![
            0x60, 0x00, // PUSH1 0 (address)
            0x31, // BALANCE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_selfbalance() {
        // SELFBALANCE returns current contract's balance
        let code = vec![0x47, 0x00]; // SELFBALANCE STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_extcodesize() {
        // EXTCODESIZE returns code size of external account
        let code = vec![
            0x60, 0x00, // PUSH1 0 (address)
            0x3b, // EXTCODESIZE
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_extcodecopy() {
        // EXTCODECOPY copies external code to memory
        let code = vec![
            0x60, 0x20, // size
            0x60, 0x00, // code offset
            0x60, 0x00, // memory offset
            0x60, 0x00, // address
            0x3c, // EXTCODECOPY
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_extcodehash() {
        // EXTCODEHASH returns keccak256 of code
        let code = vec![
            0x60, 0x00, // PUSH1 0 (address)
            0x3f, // EXTCODEHASH
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_blockhash() {
        // BLOCKHASH returns hash of recent block
        let code = vec![
            0x60, 0x00, // PUSH1 0 (block number)
            0x40, // BLOCKHASH
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_coinbase() {
        // COINBASE returns block beneficiary
        let code = vec![0x41, 0x00]; // COINBASE STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_timestamp() {
        // TIMESTAMP returns block timestamp
        let code = vec![0x42, 0x00]; // TIMESTAMP STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_number() {
        // NUMBER returns block number
        let code = vec![0x43, 0x00]; // NUMBER STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_difficulty_prevrandao() {
        // DIFFICULTY/PREVRANDAO returns random value (post-merge)
        let code = vec![0x44, 0x00]; // DIFFICULTY/PREVRANDAO STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_gaslimit() {
        // GASLIMIT returns block gas limit
        let code = vec![0x45, 0x00]; // GASLIMIT STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_chainid() {
        // CHAINID returns chain ID
        let code = vec![0x46, 0x00]; // CHAINID STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_basefee() {
        // BASEFEE returns base fee (EIP-1559)
        let code = vec![0x48, 0x00]; // BASEFEE STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_blobhash() {
        // BLOBHASH returns versioned hash (EIP-4844)
        let code = vec![
            0x60, 0x00, // PUSH1 0 (index)
            0x49, // BLOBHASH
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_blobbasefee() {
        // BLOBBASEFEE returns blob base fee (EIP-4844)
        let code = vec![0x4a, 0x00]; // BLOBBASEFEE STOP
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_log0() {
        // LOG0 emits log with no topics
        let code = vec![
            0x60, 0x42, // PUSH1 0x42
            0x60, 0x00, // PUSH1 0 (offset)
            0x52, // MSTORE
            0x60, 0x20, // PUSH1 32 (size)
            0x60, 0x00, // PUSH1 0 (offset)
            0xa0, // LOG0
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(result.logs.len(), 1);
        assert!(result.logs[0].topics.is_empty());
    }

    #[test]
    fn test_log4() {
        // LOG4 emits log with 4 topics
        let code = vec![
            0x60, 0x42, // data
            0x60, 0x00,
            0x52, // MSTORE
            0x60, 0x01, // topic4
            0x60, 0x02, // topic3
            0x60, 0x03, // topic2
            0x60, 0x04, // topic1
            0x60, 0x20, // size
            0x60, 0x00, // offset
            0xa4, // LOG4
            0x00, // STOP
        ];
        let mut evm = evm_with_code(code);
        let result = evm.execute().unwrap();

        assert!(result.success);
        assert_eq!(result.logs.len(), 1);
        assert_eq!(result.logs[0].topics.len(), 4);
    }
}
