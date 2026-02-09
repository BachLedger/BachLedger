//! Comprehensive test suite for bach-evm
//!
//! Tests cover:
//! - Basic opcodes (STOP, PUSH, POP, ADD, SUB, etc.)
//! - Arithmetic operations
//! - Comparison and bitwise operations
//! - Memory and storage operations
//! - Control flow (JUMP, JUMPI)
//! - Contract deployment and calls
//! - Gas metering
//! - Error handling (stack underflow, invalid jump, out of gas, revert)

use bach_crypto::keccak256;
use bach_evm::{deploy_contract, execute, opcode, EvmContext, EvmError, EvmState};
use bach_primitives::{Address, U256};

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
        opcode::PUSH1,
        0x42,
        opcode::PUSH1,
        0x01,
        opcode::ADD,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
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
        opcode::PUSH1,
        5,
        opcode::PUSH1,
        10,
        opcode::SUB,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
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
        opcode::PUSH1,
        10,
        opcode::PUSH1,
        5,
        opcode::LT,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
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
        opcode::PUSH1, 0x42, // value
        opcode::PUSH1, 0x00, // key
        opcode::SSTORE, opcode::PUSH1, 0x00, // key
        opcode::SLOAD, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1,
        0x00, opcode::RETURN,
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
        opcode::PUSH1, 0x04, // offset 0: jump to 4
        opcode::JUMP,        // offset 2
        opcode::INVALID,     // offset 3: should be skipped
        opcode::JUMPDEST,    // offset 4
        opcode::PUSH1, 0x42, // offset 5
        opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1, 0x00,
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
        opcode::PUSH1, 0x04, // size
        opcode::PUSH1, 0x00, // offset
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
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
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
        opcode::PUSH1, 0x00, // size
        opcode::PUSH1, 0x00, // offset
        opcode::KECCAK256, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1,
        0x00, opcode::RETURN,
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
        opcode::PUSH1,
        1,
        opcode::PUSH1,
        2,
        opcode::DUP1 + 1, // DUP2
        opcode::SWAP1,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
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
        opcode::PUSH1, 0x42, // data
        opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH32, // topic
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, opcode::PUSH1, 0x20, // size
        opcode::PUSH1, 0x00, // offset
        opcode::LOG1, opcode::STOP,
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
    let code = vec![opcode::PUSH1, 0x42, opcode::PUSH1, 0x00, opcode::MSTORE];

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
        opcode::PUSH1, 0xFF, // Invalid destination
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
        opcode::PUSH1,
        0x42,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    // Init code that returns the runtime code
    let mut init_code = vec![
        opcode::PUSH1,
        runtime_code.len() as u8, // size
        opcode::PUSH1,
        0x0C, // offset of runtime code
        opcode::PUSH1,
        0x00, // destOffset
        opcode::CODECOPY,
        opcode::PUSH1,
        runtime_code.len() as u8, // size
        opcode::PUSH1,
        0x00, // offset
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
    let call_result = bach_evm::call_contract(contract_addr, &[], context, &mut state);
    assert!(call_result.success);
    assert_eq!(call_result.output[31], 0x42);
}

#[test]
fn test_bitwise_operations() {
    // Test AND: 0xFF AND 0x0F = 0x0F
    let code = vec![
        opcode::PUSH1,
        0x0F,
        opcode::PUSH1,
        0xFF,
        opcode::AND,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
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
        opcode::PUSH1, 0x01, // value
        opcode::PUSH1, 0x04, // shift amount
        opcode::SHL, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1, 0x00,
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 16);
}

// ============================================================================
// Additional comprehensive tests for ICDD compliance
// ============================================================================

#[test]
fn test_mul_operation() {
    // Test MUL: 7 * 6 = 42
    let code = vec![
        opcode::PUSH1, 6, opcode::PUSH1, 7, opcode::MUL, opcode::PUSH1, 0x00, opcode::MSTORE,
        opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 42);
}

#[test]
fn test_div_operation() {
    // Test DIV: 42 / 6 = 7
    let code = vec![
        opcode::PUSH1, 6, opcode::PUSH1, 42, opcode::DIV, opcode::PUSH1, 0x00, opcode::MSTORE,
        opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 7);
}

#[test]
fn test_div_by_zero() {
    // DIV by zero should return 0
    let code = vec![
        opcode::PUSH1, 0, opcode::PUSH1, 42, opcode::DIV, opcode::PUSH1, 0x00, opcode::MSTORE,
        opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0);
}

#[test]
fn test_mod_operation() {
    // Test MOD: 10 % 3 = 1
    let code = vec![
        opcode::PUSH1, 3, opcode::PUSH1, 10, opcode::MOD, opcode::PUSH1, 0x00, opcode::MSTORE,
        opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 1);
}

#[test]
fn test_gt_operation() {
    // Test GT: 10 > 5 = 1
    let code = vec![
        opcode::PUSH1, 5, opcode::PUSH1, 10, opcode::GT, opcode::PUSH1, 0x00, opcode::MSTORE,
        opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 1);
}

#[test]
fn test_eq_operation() {
    // Test EQ: 42 == 42 = 1
    let code = vec![
        opcode::PUSH1, 42, opcode::PUSH1, 42, opcode::EQ, opcode::PUSH1, 0x00, opcode::MSTORE,
        opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 1);
}

#[test]
fn test_iszero_operation() {
    // Test ISZERO: iszero(0) = 1
    let code = vec![
        opcode::PUSH1, 0, opcode::ISZERO, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20,
        opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 1);
}

#[test]
fn test_or_operation() {
    // Test OR: 0xF0 | 0x0F = 0xFF
    let code = vec![
        opcode::PUSH1, 0x0F, opcode::PUSH1, 0xF0, opcode::OR, opcode::PUSH1, 0x00, opcode::MSTORE,
        opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0xFF);
}

#[test]
fn test_xor_operation() {
    // Test XOR: 0xFF ^ 0x0F = 0xF0
    let code = vec![
        opcode::PUSH1, 0x0F, opcode::PUSH1, 0xFF, opcode::XOR, opcode::PUSH1, 0x00, opcode::MSTORE,
        opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0xF0);
}

#[test]
fn test_not_operation() {
    // Test NOT: ~0 = 0xFF...FF (all ones)
    let code = vec![
        opcode::PUSH1, 0, opcode::NOT, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20,
        opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    // All bytes should be 0xFF
    for byte in &result.output {
        assert_eq!(*byte, 0xFF);
    }
}

#[test]
fn test_shr_operation() {
    // Test SHR: 16 >> 4 = 1
    let code = vec![
        opcode::PUSH1, 16, // value
        opcode::PUSH1, 4,  // shift amount
        opcode::SHR, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1, 0x00,
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 1);
}

#[test]
fn test_jumpi_taken() {
    // JUMPI with condition true (1)
    // offset 0: PUSH1, offset 1: 1, offset 2: PUSH1, offset 3: 0x06
    // offset 4: JUMPI, offset 5: INVALID, offset 6: JUMPDEST
    let code = vec![
        opcode::PUSH1, 1,    // condition = true (offset 0-1)
        opcode::PUSH1, 0x06, // destination = 6 (offset 2-3)
        opcode::JUMPI,       // offset 4
        opcode::INVALID,     // offset 5: should be skipped
        opcode::JUMPDEST,    // offset 6
        opcode::PUSH1, 0x42, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20,
        opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0x42);
}

#[test]
fn test_jumpi_not_taken() {
    // JUMPI with condition false (0) - should not jump
    let code = vec![
        opcode::PUSH1, 0,    // condition = false
        opcode::PUSH1, 0x08, // destination (doesn't matter)
        opcode::JUMPI,       // offset 4: should NOT jump
        opcode::PUSH1, 0x42, // offset 5: should execute
        opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1, 0x00,
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0x42);
}

#[test]
fn test_mstore8() {
    // MSTORE8: store single byte
    let code = vec![
        opcode::PUSH1, 0x42, // value
        opcode::PUSH1, 0x1F, // offset (last byte of first word)
        opcode::MSTORE8, opcode::PUSH1, 0x20, opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0x42);
}

#[test]
fn test_calldataload() {
    // CALLDATALOAD at offset 0
    let code = vec![
        opcode::PUSH1, 0x00, // offset
        opcode::CALLDATALOAD, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20,
        opcode::PUSH1, 0x00, opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    let mut calldata = [0u8; 32];
    calldata[31] = 0x42;
    context.data = calldata.to_vec();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0x42);
}

#[test]
fn test_calldatasize() {
    let code = vec![
        opcode::CALLDATASIZE,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    context.data = vec![1, 2, 3, 4, 5]; // 5 bytes
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 5);
}

#[test]
fn test_address_opcode() {
    let code = vec![
        opcode::ADDRESS,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    context.address = Address::from_hex("0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap();
    let mut state = EvmState::new();

    let result = execute(&code, context.clone(), &mut state);
    assert!(result.success);

    let mut expected = [0u8; 32];
    expected[12..32].copy_from_slice(context.address.as_bytes());
    assert_eq!(result.output, expected);
}

#[test]
fn test_origin_opcode() {
    let code = vec![
        opcode::ORIGIN,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    context.origin = Address::from_hex("0xabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd").unwrap();
    let mut state = EvmState::new();

    let result = execute(&code, context.clone(), &mut state);
    assert!(result.success);

    let mut expected = [0u8; 32];
    expected[12..32].copy_from_slice(context.origin.as_bytes());
    assert_eq!(result.output, expected);
}

#[test]
fn test_callvalue() {
    let code = vec![
        opcode::CALLVALUE,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    context.value = U256::from_u64(1000);
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    // Check last 2 bytes for value 1000 = 0x03E8
    assert_eq!(result.output[30], 0x03);
    assert_eq!(result.output[31], 0xE8);
}

#[test]
fn test_gas_opcode() {
    let code = vec![
        opcode::GAS,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    context.gas_limit = 100000;
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    // Gas should be less than initial due to executed opcodes
    assert!(result.gas_used > 0);
}

#[test]
fn test_pc_opcode() {
    let code = vec![
        opcode::PC, // PC at offset 0
        opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1, 0x00,
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0); // PC was 0 when executed
}

#[test]
fn test_msize_opcode() {
    let code = vec![
        opcode::PUSH1, 0x42, opcode::PUSH1, 0x00, opcode::MSTORE, // This expands memory to 32
        opcode::MSIZE, opcode::PUSH1, 0x20, opcode::MSTORE, opcode::PUSH1, 0x40, opcode::PUSH1,
        0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    // MSIZE should be 32 (0x20) after first MSTORE
    assert_eq!(result.output[63], 0x20);
}

#[test]
fn test_push2() {
    // PUSH2 = 0x61
    let code = vec![
        0x61, // PUSH2
        0x01,
        0x00, // PUSH2 0x0100 = 256
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[30], 0x01);
    assert_eq!(result.output[31], 0x00);
}

#[test]
fn test_push32() {
    let code = vec![
        opcode::PUSH32, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
        0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
        0x1C, 0x1D, 0x1E, 0x1F, 0x20, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20,
        opcode::PUSH1, 0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[0], 0x01);
    assert_eq!(result.output[31], 0x20);
}

#[test]
fn test_pop_operation() {
    let code = vec![
        opcode::PUSH1, 0x42, opcode::PUSH1, 0xFF, opcode::POP, // Pop 0xFF, leave 0x42
        opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1, 0x00,
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0x42);
}

#[test]
fn test_log0() {
    let code = vec![
        opcode::PUSH1, 0x42, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, // size
        opcode::PUSH1, 0x00, // offset
        opcode::LOG0, opcode::STOP,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.logs.len(), 1);
    assert_eq!(result.logs[0].topics.len(), 0);
    assert_eq!(result.logs[0].data.len(), 32);
}

#[test]
fn test_return_with_data() {
    let code = vec![
        opcode::PUSH1, 0xAB, opcode::PUSH1, 0x00, opcode::MSTORE8, opcode::PUSH1, 0xCD,
        opcode::PUSH1, 0x01, opcode::MSTORE8, opcode::PUSH1, 0x02, // size = 2
        opcode::PUSH1, 0x00, // offset = 0
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output.len(), 2);
    assert_eq!(result.output[0], 0xAB);
    assert_eq!(result.output[1], 0xCD);
}

#[test]
fn test_exp_operation() {
    // 2^8 = 256
    let code = vec![
        opcode::PUSH1, 8, // exponent
        opcode::PUSH1, 2, // base
        opcode::EXP, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1, 0x00,
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[30], 0x01);
    assert_eq!(result.output[31], 0x00); // 256 = 0x0100
}

#[test]
fn test_addmod_operation() {
    // (10 + 5) % 7 = 1
    let code = vec![
        opcode::PUSH1, 7,  // N
        opcode::PUSH1, 5,  // b
        opcode::PUSH1, 10, // a
        opcode::ADDMOD, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1,
        0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 1);
}

#[test]
fn test_mulmod_operation() {
    // (3 * 4) % 5 = 2
    let code = vec![
        opcode::PUSH1, 5, // N
        opcode::PUSH1, 4, // b
        opcode::PUSH1, 3, // a
        opcode::MULMOD, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1,
        0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 2);
}

#[test]
fn test_byte_operation() {
    // Get byte 31 (least significant) of 0x42
    let code = vec![
        opcode::PUSH1, 0x42, opcode::PUSH1, 31, // byte index
        opcode::BYTE, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1,
        0x00, opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 0x42);
}

#[test]
fn test_timestamp() {
    let code = vec![
        opcode::TIMESTAMP,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    context.timestamp = 1234567890;
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    // Timestamp should be in output
    assert!(result.output[31] > 0 || result.output[27] > 0);
}

#[test]
fn test_number() {
    let code = vec![
        opcode::NUMBER,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    context.block_number = 12345;
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    // 12345 = 0x3039
    assert_eq!(result.output[30], 0x30);
    assert_eq!(result.output[31], 0x39);
}

#[test]
fn test_chainid() {
    let code = vec![
        opcode::CHAINID,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];

    let mut context = EvmContext::default();
    context.chain_id = 1; // Mainnet
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    assert_eq!(result.output[31], 1);
}

#[test]
fn test_codesize() {
    let code = vec![
        opcode::CODESIZE,
        opcode::PUSH1,
        0x00,
        opcode::MSTORE,
        opcode::PUSH1,
        0x20,
        opcode::PUSH1,
        0x00,
        opcode::RETURN,
    ];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(result.success);
    // Code size should be 9 bytes (CODESIZE + PUSH1 + 0x00 + MSTORE + PUSH1 + 0x20 + PUSH1 + 0x00 + RETURN)
    assert_eq!(result.output[31], 9);
}

#[test]
fn test_stack_overflow() {
    // Try to push more than 1024 items
    let mut code = Vec::new();
    for _ in 0..1025 {
        code.push(opcode::PUSH1);
        code.push(0x42);
    }
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(!result.success);
    assert!(matches!(result.error, Some(EvmError::StackOverflow)));
}

#[test]
fn test_invalid_opcode() {
    let code = vec![opcode::INVALID];
    let context = EvmContext::default();
    let mut state = EvmState::new();

    let result = execute(&code, context, &mut state);
    assert!(!result.success);
    assert!(matches!(result.error, Some(EvmError::InvalidOpcode(_))));
}

#[test]
fn test_balance_opcode() {
    let code = vec![
        opcode::PUSH1, 0x00, opcode::MLOAD, // Load address from memory (will be 0)
        opcode::BALANCE, opcode::PUSH1, 0x00, opcode::MSTORE, opcode::PUSH1, 0x20, opcode::PUSH1,
        0x00, opcode::RETURN,
    ];

    let context = EvmContext::default();
    let mut state = EvmState::new();

    // Set balance for the zero address
    state.set_balance(&Address::zero(), U256::from_u64(1000000));

    let result = execute(&code, context, &mut state);
    assert!(result.success);
}
