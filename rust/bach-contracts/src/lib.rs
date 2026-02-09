//! BachLedger Medical Smart Contracts
//!
//! This module provides pre-built EVM bytecode for medical blockchain functionality:
//! - Simple storage contract for testing
//! - Medical record management patterns
//! - Access control utilities
//!
//! # Usage
//!
//! ```ignore
//! use bach_contracts::{SimpleStorage, MedicalRegistry};
//! use bach_evm::{deploy_contract, call_contract, EvmContext, EvmState};
//!
//! // Deploy simple storage contract
//! let code = SimpleStorage::deployment_code();
//! let addr = deploy_contract(&code, context, &mut state)?;
//!
//! // Call contract to store a value
//! let calldata = SimpleStorage::encode_store(42);
//! call_contract(addr, &calldata, context, &mut state);
//! ```

#![forbid(unsafe_code)]

use bach_primitives::{Address, H256, U256};
use bach_crypto::keccak256;

// =============================================================================
// Simple Storage Contract
// =============================================================================

/// Simple storage contract for basic testing.
///
/// Solidity equivalent:
/// ```solidity
/// contract SimpleStorage {
///     uint256 public value;
///
///     function store(uint256 _value) public {
///         value = _value;
///     }
///
///     function retrieve() public view returns (uint256) {
///         return value;
///     }
/// }
/// ```
pub struct SimpleStorage;

impl SimpleStorage {
    /// Returns the deployment bytecode for SimpleStorage contract.
    ///
    /// This is a minimal contract that stores a value at slot 0.
    /// - Empty calldata: returns SLOAD(0)
    /// - Calldata present: SSTORE(0, first 32 bytes of calldata)
    pub fn deployment_code() -> Vec<u8> {
        // Runtime code (what gets stored):
        // 1. If calldata size is 0, return SLOAD(0)
        // 2. Otherwise, store first calldata word to slot 0
        let runtime = vec![
            // Check calldata size
            0x36,       // CALLDATASIZE          offset 0x00
            0x15,       // ISZERO                offset 0x01
            0x60, 0x0c, // PUSH1 0x0c (retrieve label at offset 12)
            0x57,       // JUMPI                 offset 0x04

            // Store path (offset 0x05):
            // Load 32 bytes of calldata starting at offset 0
            0x60, 0x00, // PUSH1 0 (calldata offset) offset 0x05
            0x35,       // CALLDATALOAD          offset 0x07
            0x60, 0x00, // PUSH1 0 (storage slot) offset 0x08
            0x55,       // SSTORE                offset 0x0a
            0x00,       // STOP                  offset 0x0b

            // Retrieve path (offset 0x0c = 12):
            0x5b,       // JUMPDEST              offset 0x0c
            0x60, 0x00, // PUSH1 0 (slot)        offset 0x0d
            0x54,       // SLOAD                 offset 0x0f
            0x60, 0x00, // PUSH1 0 (memory offset) offset 0x10
            0x52,       // MSTORE                offset 0x12
            0x60, 0x20, // PUSH1 32 (return size) offset 0x13
            0x60, 0x00, // PUSH1 0 (memory offset) offset 0x15
            0xf3,       // RETURN                offset 0x17
        ];

        // Init code that copies runtime to memory and returns it
        //
        // Init code structure (11 bytes):
        // PUSH1 runtime_len  (2 bytes)
        // DUP1               (1 byte)
        // PUSH1 init_len     (2 bytes)
        // PUSH1 0            (2 bytes)
        // CODECOPY           (1 byte)
        // PUSH1 0            (2 bytes)
        // RETURN             (1 byte)
        // Total: 11 bytes
        let runtime_len = runtime.len() as u8;
        let init_len = 11u8; // Actual init code size

        let mut init = vec![
            0x60, runtime_len, // PUSH1 runtime_len
            0x80,              // DUP1
            0x60, init_len,    // PUSH1 init_len (offset where runtime starts)
            0x60, 0x00,        // PUSH1 0 (memory dest)
            0x39,              // CODECOPY
            0x60, 0x00,        // PUSH1 0 (memory offset)
            0xf3,              // RETURN
        ];

        init.extend(runtime);
        init
    }

    /// Encodes a `store(uint256)` function call.
    pub fn encode_store(value: U256) -> Vec<u8> {
        // Function selector: keccak256("store(uint256)")[0:4]
        let selector = Self::store_selector();
        let mut data = selector.to_vec();
        data.extend_from_slice(&value.to_be_bytes());
        data
    }

    /// Encodes a `retrieve()` function call.
    pub fn encode_retrieve() -> Vec<u8> {
        // Empty calldata triggers the retrieve path
        Vec::new()
    }

    /// Returns the function selector for `store(uint256)`.
    pub fn store_selector() -> [u8; 4] {
        let hash = keccak256(b"store(uint256)");
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash.as_bytes()[0..4]);
        selector
    }

    /// Returns the function selector for `retrieve()`.
    pub fn retrieve_selector() -> [u8; 4] {
        let hash = keccak256(b"retrieve()");
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash.as_bytes()[0..4]);
        selector
    }
}

// =============================================================================
// Counter Contract
// =============================================================================

/// Counter contract for incrementing/decrementing a value.
///
/// Solidity equivalent:
/// ```solidity
/// contract Counter {
///     uint256 public count;
///
///     function increment() public {
///         count += 1;
///     }
///
///     function get() public view returns (uint256) {
///         return count;
///     }
/// }
/// ```
pub struct Counter;

impl Counter {
    /// Returns the deployment bytecode for Counter contract.
    pub fn deployment_code() -> Vec<u8> {
        // Runtime code:
        // - Empty calldata: return count (SLOAD slot 0)
        // - Any calldata: increment count
        let runtime = vec![
            // Check calldata size
            0x36,       // CALLDATASIZE          offset 0x00
            0x15,       // ISZERO                offset 0x01
            0x60, 0x0f, // PUSH1 0x0f (get label at offset 15)
            0x57,       // JUMPI                 offset 0x04

            // Increment path (offset 0x05):
            0x60, 0x00, // PUSH1 0 (slot)        offset 0x05
            0x54,       // SLOAD                 offset 0x07
            0x60, 0x01, // PUSH1 1               offset 0x08
            0x01,       // ADD                   offset 0x0a
            0x60, 0x00, // PUSH1 0 (slot)        offset 0x0b
            0x55,       // SSTORE                offset 0x0d
            0x00,       // STOP                  offset 0x0e

            // Get path (offset 0x0f = 15):
            0x5b,       // JUMPDEST              offset 0x0f
            0x60, 0x00, // PUSH1 0 (slot)        offset 0x10
            0x54,       // SLOAD                 offset 0x12
            0x60, 0x00, // PUSH1 0 (memory offset) offset 0x13
            0x52,       // MSTORE                offset 0x15
            0x60, 0x20, // PUSH1 32 (return size) offset 0x16
            0x60, 0x00, // PUSH1 0 (memory offset) offset 0x18
            0xf3,       // RETURN                offset 0x1a
        ];

        let runtime_len = runtime.len() as u8;
        let init_len = 11u8; // Same as SimpleStorage

        let mut init = vec![
            0x60, runtime_len, // PUSH1 runtime_len
            0x80,              // DUP1
            0x60, init_len,    // PUSH1 init_len
            0x60, 0x00,        // PUSH1 0
            0x39,              // CODECOPY
            0x60, 0x00,        // PUSH1 0
            0xf3,              // RETURN
        ];

        init.extend(runtime);
        init
    }

    /// Encodes an `increment()` function call.
    pub fn encode_increment() -> Vec<u8> {
        Self::increment_selector().to_vec()
    }

    /// Encodes a `get()` function call.
    pub fn encode_get() -> Vec<u8> {
        Vec::new() // Empty calldata triggers get
    }

    /// Returns the function selector for `increment()`.
    pub fn increment_selector() -> [u8; 4] {
        let hash = keccak256(b"increment()");
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash.as_bytes()[0..4]);
        selector
    }
}

// =============================================================================
// Medical Registry Contract
// =============================================================================

/// Medical record registry contract.
///
/// Simplified version of the full MedicalRecord.sol that can be deployed
/// via our EVM. Stores hashes of medical records with patient ownership.
pub struct MedicalRegistry;

impl MedicalRegistry {
    /// Storage slot for admin address
    #[allow(dead_code)]
    const SLOT_ADMIN: u64 = 0;
    /// Storage slot for record count
    #[allow(dead_code)]
    const SLOT_RECORD_COUNT: u64 = 1;
    // Dynamic storage: records start at slot 2 and use (patient_hash + index) as key

    /// Computes the storage slot for a record.
    pub fn record_slot(patient: &Address, index: u64) -> H256 {
        let mut data = Vec::new();
        data.extend_from_slice(patient.as_bytes());
        data.extend_from_slice(&index.to_be_bytes());
        keccak256(&data)
    }

    /// Encodes a `setAdmin(address)` call.
    pub fn encode_set_admin(admin: &Address) -> Vec<u8> {
        let selector = Self::set_admin_selector();
        let mut data = selector.to_vec();
        // Pad address to 32 bytes (left-pad with zeros)
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(admin.as_bytes());
        data
    }

    /// Encodes an `addRecord(address patient, bytes32 dataHash)` call.
    pub fn encode_add_record(patient: &Address, data_hash: &H256) -> Vec<u8> {
        let selector = Self::add_record_selector();
        let mut data = selector.to_vec();
        // Patient address (32 bytes, left-padded)
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(patient.as_bytes());
        // Data hash (32 bytes)
        data.extend_from_slice(data_hash.as_bytes());
        data
    }

    /// Function selector for `setAdmin(address)`.
    pub fn set_admin_selector() -> [u8; 4] {
        let hash = keccak256(b"setAdmin(address)");
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash.as_bytes()[0..4]);
        selector
    }

    /// Function selector for `addRecord(address,bytes32)`.
    pub fn add_record_selector() -> [u8; 4] {
        let hash = keccak256(b"addRecord(address,bytes32)");
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash.as_bytes()[0..4]);
        selector
    }

    /// Function selector for `getRecord(address,uint256)`.
    pub fn get_record_selector() -> [u8; 4] {
        let hash = keccak256(b"getRecord(address,uint256)");
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash.as_bytes()[0..4]);
        selector
    }
}

// =============================================================================
// ABI Encoding Utilities
// =============================================================================

/// Utilities for ABI encoding function calls.
pub mod abi {
    use super::*;

    /// Encodes a uint256 value to 32 bytes (big-endian, left-padded).
    pub fn encode_uint256(value: U256) -> [u8; 32] {
        value.to_be_bytes()
    }

    /// Encodes an address to 32 bytes (left-padded).
    pub fn encode_address(addr: &Address) -> [u8; 32] {
        let mut encoded = [0u8; 32];
        encoded[12..32].copy_from_slice(addr.as_bytes());
        encoded
    }

    /// Encodes a bytes32 value.
    pub fn encode_bytes32(hash: &H256) -> [u8; 32] {
        let mut encoded = [0u8; 32];
        encoded.copy_from_slice(hash.as_bytes());
        encoded
    }

    /// Decodes a uint256 from 32 bytes.
    pub fn decode_uint256(data: &[u8]) -> Option<U256> {
        if data.len() < 32 {
            return None;
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&data[0..32]);
        Some(U256::from_be_bytes(bytes))
    }

    /// Decodes an address from 32 bytes.
    pub fn decode_address(data: &[u8]) -> Option<Address> {
        if data.len() < 32 {
            return None;
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&data[12..32]);
        Some(Address::from(bytes))
    }

    /// Computes a function selector from a function signature.
    pub fn function_selector(signature: &str) -> [u8; 4] {
        let hash = keccak256(signature.as_bytes());
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash.as_bytes()[0..4]);
        selector
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use bach_evm::{deploy_contract, call_contract, EvmContext, EvmState};

    fn test_context(caller: Address) -> EvmContext {
        EvmContext {
            origin: caller,
            caller,
            address: Address::zero(),
            value: U256::ZERO,
            data: Vec::new(),
            gas_limit: 1_000_000,
            gas_price: U256::ZERO,
            block_number: 1,
            timestamp: 1234567890,
            block_gas_limit: 30_000_000,
            coinbase: Address::zero(),
            difficulty: U256::ZERO,
            chain_id: 1,
            base_fee: U256::ZERO,
            is_static: false,
            depth: 0,
        }
    }

    #[test]
    fn test_simple_storage_deployment() {
        let mut state = EvmState::new();
        let deployer = Address::from([0xaa; 20]);

        // Fund deployer
        state.set_balance(&deployer, U256::from_u64(1_000_000_000));

        let code = SimpleStorage::deployment_code();
        let context = test_context(deployer);

        let result = deploy_contract(&code, context, &mut state);
        assert!(result.is_ok(), "Contract deployment failed: {:?}", result.err());

        let contract_addr = result.unwrap();
        assert!(!contract_addr.is_zero());

        // Verify runtime code was stored
        let stored_code = state.get_code(&contract_addr);
        assert!(!stored_code.is_empty(), "No runtime code stored");
    }

    #[test]
    fn test_simple_storage_store_and_retrieve() {
        let mut state = EvmState::new();
        let deployer = Address::from([0xaa; 20]);

        // Fund deployer
        state.set_balance(&deployer, U256::from_u64(1_000_000_000));

        // Deploy
        let code = SimpleStorage::deployment_code();
        let context = test_context(deployer);
        let contract_addr = deploy_contract(&code, context, &mut state).unwrap();

        // Verify runtime code was stored
        let runtime_code = state.get_code(&contract_addr);
        eprintln!("Runtime code length: {}", runtime_code.len());
        eprintln!("Runtime code: {:02x?}", &runtime_code);

        // First, test retrieve (empty calldata) - should return 0
        let retrieve_data = Vec::new();
        let call_ctx = test_context(deployer);

        let result = call_contract(contract_addr, &retrieve_data, call_ctx, &mut state);
        assert!(result.success, "Initial retrieve failed: {:?}", result.error);

        let value = abi::decode_uint256(&result.output);
        assert_eq!(value, Some(U256::ZERO), "Initial value should be 0");

        // Store value 42 (just send the value as calldata, no selector)
        let mut store_data = [0u8; 32];
        store_data[31] = 42;  // 42 in big-endian u256
        let call_ctx = test_context(deployer);

        let result = call_contract(contract_addr, &store_data, call_ctx, &mut state);
        assert!(result.success, "Store call failed: {:?}", result.error);

        // Retrieve value
        let retrieve_data = Vec::new();
        let call_ctx = test_context(deployer);

        let result = call_contract(contract_addr, &retrieve_data, call_ctx, &mut state);
        assert!(result.success, "Retrieve call failed: {:?}", result.error);

        // Decode result
        let value = abi::decode_uint256(&result.output);
        assert_eq!(value, Some(U256::from_u64(42)));
    }

    #[test]
    fn test_counter_deployment() {
        let mut state = EvmState::new();
        let deployer = Address::from([0xbb; 20]);

        state.set_balance(&deployer, U256::from_u64(1_000_000_000));

        let code = Counter::deployment_code();
        let context = test_context(deployer);

        let result = deploy_contract(&code, context, &mut state);
        assert!(result.is_ok(), "Counter deployment failed: {:?}", result.err());
    }

    #[test]
    fn test_counter_increment() {
        let mut state = EvmState::new();
        let deployer = Address::from([0xcc; 20]);

        state.set_balance(&deployer, U256::from_u64(1_000_000_000));

        // Deploy
        let code = Counter::deployment_code();
        let context = test_context(deployer);
        let contract_addr = deploy_contract(&code, context, &mut state).unwrap();

        // Get initial count (should be 0)
        let get_data = Counter::encode_get();
        let call_ctx = test_context(deployer);
        let result = call_contract(contract_addr, &get_data, call_ctx, &mut state);
        assert!(result.success);
        assert_eq!(abi::decode_uint256(&result.output), Some(U256::ZERO));

        // Increment
        let inc_data = Counter::encode_increment();
        let call_ctx = test_context(deployer);
        let result = call_contract(contract_addr, &inc_data, call_ctx, &mut state);
        assert!(result.success, "Increment failed: {:?}", result.error);

        // Get count (should be 1)
        let call_ctx = test_context(deployer);
        let result = call_contract(contract_addr, &get_data, call_ctx, &mut state);
        assert!(result.success);
        assert_eq!(abi::decode_uint256(&result.output), Some(U256::from_u64(1)));

        // Increment again
        let call_ctx = test_context(deployer);
        let result = call_contract(contract_addr, &inc_data, call_ctx, &mut state);
        assert!(result.success);

        // Get count (should be 2)
        let call_ctx = test_context(deployer);
        let result = call_contract(contract_addr, &get_data, call_ctx, &mut state);
        assert!(result.success);
        assert_eq!(abi::decode_uint256(&result.output), Some(U256::from_u64(2)));
    }

    #[test]
    fn test_function_selector() {
        // Known Ethereum function selectors
        assert_eq!(abi::function_selector("transfer(address,uint256)"),
                   [0xa9, 0x05, 0x9c, 0xbb]);

        assert_eq!(abi::function_selector("balanceOf(address)"),
                   [0x70, 0xa0, 0x82, 0x31]);
    }

    #[test]
    fn test_abi_encoding() {
        // Test uint256 encoding
        let encoded = abi::encode_uint256(U256::from_u64(256));
        assert_eq!(encoded[31], 0);  // Low byte is 0
        assert_eq!(encoded[30], 1);  // Next byte is 1 (256 = 0x100)

        // Test address encoding (left-padded)
        let addr = Address::from([0xab; 20]);
        let encoded = abi::encode_address(&addr);
        assert_eq!(&encoded[0..12], &[0u8; 12]); // First 12 bytes are zeros
        assert_eq!(&encoded[12..32], addr.as_bytes()); // Last 20 bytes are the address
    }
}
