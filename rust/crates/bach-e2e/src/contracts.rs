//! Pre-compiled test contract bytecodes for E2E testing.
//!
//! These are minimal hand-crafted EVM bytecode contracts for testing
//! contract deployment and interaction without requiring a Solidity compiler.
//!
//! # Contract Specifications
//!
//! ## SimpleStorage
//! - `store(uint256)`: Store a value at slot 0
//! - `retrieve()`: Return the value at slot 0
//!
//! ## Counter
//! - `increment()`: Increment counter at slot 0
//! - `get()`: Return current count
//!
//! # Bytecode Structure
//!
//! Each contract has:
//! - `*_INIT_CODE`: Constructor bytecode (deploys runtime code)
//! - `*_RUNTIME`: Runtime bytecode (actual contract logic)
//! - `*_SELECTORS`: Function selector constants

use bach_primitives::{Address, H256};

// ============================================================================
// Common Constants
// ============================================================================

/// Zero address for testing
pub const ZERO_ADDRESS: Address = Address::ZERO;

/// Common test value: 1 ether in wei
pub const ONE_ETHER: u128 = 1_000_000_000_000_000_000;

/// Common test value: 1 gwei
pub const ONE_GWEI: u128 = 1_000_000_000;

/// Default gas limit for test transactions
pub const DEFAULT_GAS_LIMIT: u64 = 1_000_000;

/// Minimum gas for simple transfer
pub const TRANSFER_GAS: u64 = 21_000;

/// Storage slot 0
pub const SLOT_ZERO: H256 = H256::ZERO;

// ============================================================================
// SimpleStorage Contract
// ============================================================================
//
// Solidity equivalent:
// ```solidity
// contract SimpleStorage {
//     uint256 value;
//     function store(uint256 v) public { value = v; }
//     function retrieve() public view returns (uint256) { return value; }
// }
// ```

/// Function selector for `store(uint256)` = keccak256("store(uint256)")[:4]
pub const SIMPLE_STORAGE_STORE_SELECTOR: [u8; 4] = [0x60, 0x57, 0x36, 0x1d];

/// Function selector for `retrieve()` = keccak256("retrieve()")[:4]
pub const SIMPLE_STORAGE_RETRIEVE_SELECTOR: [u8; 4] = [0x2e, 0x64, 0xce, 0xc1];

/// SimpleStorage runtime bytecode
///
/// Dispatcher:
/// - If calldata starts with store selector: SSTORE slot 0
/// - If calldata starts with retrieve selector: SLOAD slot 0 and return
///
/// Layout:
/// ```text
/// CALLDATASIZE PUSH1 0x04 LT PUSH1 revert JUMPI  ; require >= 4 bytes
/// PUSH1 0x00 CALLDATALOAD PUSH1 0xe0 SHR         ; get selector
/// DUP1 PUSH4 store_sel EQ PUSH1 store JUMPI     ; dispatch store
/// DUP1 PUSH4 retrieve_sel EQ PUSH1 retrieve JUMPI ; dispatch retrieve
/// PUSH1 0x00 PUSH1 0x00 REVERT                   ; unknown selector
/// store: PUSH1 0x04 CALLDATALOAD PUSH1 0x00 SSTORE STOP
/// retrieve: PUSH1 0x00 SLOAD PUSH1 0x00 MSTORE PUSH1 0x20 PUSH1 0x00 RETURN
/// ```
pub const SIMPLE_STORAGE_RUNTIME: &[u8] = &[
    // Check calldata size >= 4
    0x36,       // CALLDATASIZE
    0x60, 0x04, // PUSH1 4
    0x10,       // LT
    0x60, 0x42, // PUSH1 revert_label (66)
    0x57,       // JUMPI

    // Load selector (first 4 bytes)
    0x60, 0x00, // PUSH1 0
    0x35,       // CALLDATALOAD
    0x60, 0xe0, // PUSH1 224
    0x1c,       // SHR (shift right to get top 4 bytes)

    // Check for store(uint256) selector: 0x6057361d
    0x80,       // DUP1
    0x63, 0x60, 0x57, 0x36, 0x1d, // PUSH4 store_selector
    0x14,       // EQ
    0x60, 0x28, // PUSH1 store_label (40)
    0x57,       // JUMPI

    // Check for retrieve() selector: 0x2e64cec1
    0x80,       // DUP1
    0x63, 0x2e, 0x64, 0xce, 0xc1, // PUSH4 retrieve_selector
    0x14,       // EQ
    0x60, 0x35, // PUSH1 retrieve_label (53)
    0x57,       // JUMPI

    // Unknown selector - revert
    0x60, 0x00, // PUSH1 0
    0x60, 0x00, // PUSH1 0
    0xfd,       // REVERT

    // store: offset 40 (0x28)
    0x5b,       // JUMPDEST
    0x60, 0x04, // PUSH1 4 (skip selector)
    0x35,       // CALLDATALOAD (load uint256 argument)
    0x60, 0x00, // PUSH1 0 (slot 0)
    0x55,       // SSTORE
    0x00,       // STOP

    // retrieve: offset 53 (0x35)
    0x5b,       // JUMPDEST
    0x60, 0x00, // PUSH1 0 (slot 0)
    0x54,       // SLOAD
    0x60, 0x00, // PUSH1 0 (memory offset)
    0x52,       // MSTORE
    0x60, 0x20, // PUSH1 32 (return size)
    0x60, 0x00, // PUSH1 0 (memory offset)
    0xf3,       // RETURN

    // revert: offset 66 (0x42)
    0x5b,       // JUMPDEST
    0x60, 0x00, // PUSH1 0
    0x60, 0x00, // PUSH1 0
    0xfd,       // REVERT
];

/// SimpleStorage init code (constructor)
///
/// Copies runtime code to memory and returns it.
/// ```text
/// PUSH1 runtime_size PUSH1 0x0c PUSH1 0x00 CODECOPY
/// PUSH1 runtime_size PUSH1 0x00 RETURN
/// <runtime_code>
/// ```
pub const SIMPLE_STORAGE_INIT_CODE: &[u8] = &[
    // Copy runtime code to memory
    0x60, 0x47, // PUSH1 runtime_size (71 bytes)
    0x60, 0x0c, // PUSH1 12 (offset where runtime starts)
    0x60, 0x00, // PUSH1 0 (memory destination)
    0x39,       // CODECOPY

    // Return runtime code
    0x60, 0x47, // PUSH1 runtime_size
    0x60, 0x00, // PUSH1 0
    0xf3,       // RETURN

    // Runtime code follows (71 bytes = 0x47)
    // Check calldata size >= 4
    0x36,       // CALLDATASIZE
    0x60, 0x04, // PUSH1 4
    0x10,       // LT
    0x60, 0x42, // PUSH1 revert_label
    0x57,       // JUMPI

    // Load selector
    0x60, 0x00, // PUSH1 0
    0x35,       // CALLDATALOAD
    0x60, 0xe0, // PUSH1 224
    0x1c,       // SHR

    // Check store selector
    0x80,       // DUP1
    0x63, 0x60, 0x57, 0x36, 0x1d, // PUSH4
    0x14,       // EQ
    0x60, 0x28, // PUSH1 store_label
    0x57,       // JUMPI

    // Check retrieve selector
    0x80,       // DUP1
    0x63, 0x2e, 0x64, 0xce, 0xc1, // PUSH4
    0x14,       // EQ
    0x60, 0x35, // PUSH1 retrieve_label
    0x57,       // JUMPI

    // Unknown selector
    0x60, 0x00, // PUSH1 0
    0x60, 0x00, // PUSH1 0
    0xfd,       // REVERT

    // store:
    0x5b,       // JUMPDEST
    0x60, 0x04, // PUSH1 4
    0x35,       // CALLDATALOAD
    0x60, 0x00, // PUSH1 0
    0x55,       // SSTORE
    0x00,       // STOP

    // retrieve:
    0x5b,       // JUMPDEST
    0x60, 0x00, // PUSH1 0
    0x54,       // SLOAD
    0x60, 0x00, // PUSH1 0
    0x52,       // MSTORE
    0x60, 0x20, // PUSH1 32
    0x60, 0x00, // PUSH1 0
    0xf3,       // RETURN

    // revert:
    0x5b,       // JUMPDEST
    0x60, 0x00, // PUSH1 0
    0x60, 0x00, // PUSH1 0
    0xfd,       // REVERT
];

// ============================================================================
// Counter Contract
// ============================================================================
//
// Solidity equivalent:
// ```solidity
// contract Counter {
//     uint256 count;
//     function increment() public { count++; }
//     function get() public view returns (uint256) { return count; }
// }
// ```

/// Function selector for `increment()` = keccak256("increment()")[:4]
pub const COUNTER_INCREMENT_SELECTOR: [u8; 4] = [0xd0, 0x9d, 0xe0, 0x8a];

/// Function selector for `get()` = keccak256("get()")[:4]
pub const COUNTER_GET_SELECTOR: [u8; 4] = [0x6d, 0x4c, 0xe6, 0x3c];

/// Counter runtime bytecode
pub const COUNTER_RUNTIME: &[u8] = &[
    // Check calldata size >= 4
    0x36,       // CALLDATASIZE
    0x60, 0x04, // PUSH1 4
    0x10,       // LT
    0x60, 0x3e, // PUSH1 revert_label (62)
    0x57,       // JUMPI

    // Load selector
    0x60, 0x00, // PUSH1 0
    0x35,       // CALLDATALOAD
    0x60, 0xe0, // PUSH1 224
    0x1c,       // SHR

    // Check increment() selector: 0xd09de08a
    0x80,       // DUP1
    0x63, 0xd0, 0x9d, 0xe0, 0x8a, // PUSH4
    0x14,       // EQ
    0x60, 0x28, // PUSH1 increment_label (40)
    0x57,       // JUMPI

    // Check get() selector: 0x6d4ce63c
    0x80,       // DUP1
    0x63, 0x6d, 0x4c, 0xe6, 0x3c, // PUSH4
    0x14,       // EQ
    0x60, 0x31, // PUSH1 get_label (49)
    0x57,       // JUMPI

    // Unknown selector - revert
    0x60, 0x00, // PUSH1 0
    0x60, 0x00, // PUSH1 0
    0xfd,       // REVERT

    // increment: offset 40 (0x28)
    0x5b,       // JUMPDEST
    0x60, 0x00, // PUSH1 0 (slot)
    0x54,       // SLOAD
    0x60, 0x01, // PUSH1 1
    0x01,       // ADD
    0x60, 0x00, // PUSH1 0 (slot)
    0x55,       // SSTORE
    0x00,       // STOP

    // get: offset 49 (0x31)
    0x5b,       // JUMPDEST
    0x60, 0x00, // PUSH1 0 (slot)
    0x54,       // SLOAD
    0x60, 0x00, // PUSH1 0 (memory offset)
    0x52,       // MSTORE
    0x60, 0x20, // PUSH1 32
    0x60, 0x00, // PUSH1 0
    0xf3,       // RETURN

    // revert: offset 62 (0x3e)
    0x5b,       // JUMPDEST
    0x60, 0x00, // PUSH1 0
    0x60, 0x00, // PUSH1 0
    0xfd,       // REVERT
];

/// Counter init code (constructor)
pub const COUNTER_INIT_CODE: &[u8] = &[
    // Copy runtime code to memory
    0x60, 0x43, // PUSH1 runtime_size (67 bytes)
    0x60, 0x0c, // PUSH1 12 (offset where runtime starts)
    0x60, 0x00, // PUSH1 0 (memory destination)
    0x39,       // CODECOPY

    // Return runtime code
    0x60, 0x43, // PUSH1 runtime_size
    0x60, 0x00, // PUSH1 0
    0xf3,       // RETURN

    // Runtime code follows (67 bytes = 0x43)
    0x36, 0x60, 0x04, 0x10, 0x60, 0x3e, 0x57,
    0x60, 0x00, 0x35, 0x60, 0xe0, 0x1c,
    0x80, 0x63, 0xd0, 0x9d, 0xe0, 0x8a, 0x14, 0x60, 0x28, 0x57,
    0x80, 0x63, 0x6d, 0x4c, 0xe6, 0x3c, 0x14, 0x60, 0x31, 0x57,
    0x60, 0x00, 0x60, 0x00, 0xfd,
    0x5b, 0x60, 0x00, 0x54, 0x60, 0x01, 0x01, 0x60, 0x00, 0x55, 0x00,
    0x5b, 0x60, 0x00, 0x54, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3,
    0x5b, 0x60, 0x00, 0x60, 0x00, 0xfd,
];

// ============================================================================
// Minimal Return Contract (for simple testing)
// ============================================================================
//
// Just returns a constant value - useful for basic deployment tests

/// Contract that always returns 0x42 (66 in decimal)
pub const RETURN_42_RUNTIME: &[u8] = &[
    0x60, 0x42, // PUSH1 0x42
    0x60, 0x00, // PUSH1 0 (memory offset)
    0x52,       // MSTORE
    0x60, 0x20, // PUSH1 32 (return size)
    0x60, 0x00, // PUSH1 0 (memory offset)
    0xf3,       // RETURN
];

/// Init code for return-42 contract
pub const RETURN_42_INIT_CODE: &[u8] = &[
    0x60, 0x09, // PUSH1 9 (runtime size)
    0x60, 0x0c, // PUSH1 12 (code offset)
    0x60, 0x00, // PUSH1 0 (memory dest)
    0x39,       // CODECOPY
    0x60, 0x09, // PUSH1 9
    0x60, 0x00, // PUSH1 0
    0xf3,       // RETURN
    // Runtime:
    0x60, 0x42, // PUSH1 0x42
    0x60, 0x00, // PUSH1 0
    0x52,       // MSTORE
    0x60, 0x20, // PUSH1 32
    0x60, 0x00, // PUSH1 0
    0xf3,       // RETURN
];

// ============================================================================
// Helper Functions
// ============================================================================

/// Encode a store(uint256) call
pub fn encode_store(value: u128) -> Vec<u8> {
    let mut data = Vec::with_capacity(36);
    data.extend_from_slice(&SIMPLE_STORAGE_STORE_SELECTOR);
    // Pad value to 32 bytes (big-endian)
    let mut padded = [0u8; 32];
    padded[16..].copy_from_slice(&value.to_be_bytes());
    data.extend_from_slice(&padded);
    data
}

/// Encode a retrieve() call
pub fn encode_retrieve() -> Vec<u8> {
    SIMPLE_STORAGE_RETRIEVE_SELECTOR.to_vec()
}

/// Encode an increment() call
pub fn encode_increment() -> Vec<u8> {
    COUNTER_INCREMENT_SELECTOR.to_vec()
}

/// Encode a get() call
pub fn encode_get() -> Vec<u8> {
    COUNTER_GET_SELECTOR.to_vec()
}

/// Decode a uint256 return value
pub fn decode_uint256(data: &[u8]) -> Option<u128> {
    if data.len() < 32 {
        return None;
    }
    // Take last 16 bytes (assuming value fits in u128)
    let bytes: [u8; 16] = data[16..32].try_into().ok()?;
    Some(u128::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_store() {
        let data = encode_store(42);
        assert_eq!(data.len(), 36); // 4 selector + 32 value
        assert_eq!(&data[0..4], &SIMPLE_STORAGE_STORE_SELECTOR);
        assert_eq!(data[35], 42); // last byte is the value
    }

    #[test]
    fn test_encode_retrieve() {
        let data = encode_retrieve();
        assert_eq!(data.len(), 4);
        assert_eq!(&data[..], &SIMPLE_STORAGE_RETRIEVE_SELECTOR);
    }

    #[test]
    fn test_encode_increment() {
        let data = encode_increment();
        assert_eq!(data.len(), 4);
        assert_eq!(&data[..], &COUNTER_INCREMENT_SELECTOR);
    }

    #[test]
    fn test_encode_get() {
        let data = encode_get();
        assert_eq!(data.len(), 4);
        assert_eq!(&data[..], &COUNTER_GET_SELECTOR);
    }

    #[test]
    fn test_decode_uint256() {
        let mut data = [0u8; 32];
        data[31] = 42;
        assert_eq!(decode_uint256(&data), Some(42));
    }

    #[test]
    fn test_decode_uint256_large() {
        let mut data = [0u8; 32];
        data[16..].copy_from_slice(&ONE_ETHER.to_be_bytes());
        assert_eq!(decode_uint256(&data), Some(ONE_ETHER));
    }

    #[test]
    fn test_decode_uint256_short() {
        let data = [0u8; 16];
        assert_eq!(decode_uint256(&data), None);
    }

    #[test]
    fn test_init_code_sizes() {
        // Verify init code is larger than runtime (contains both)
        assert!(SIMPLE_STORAGE_INIT_CODE.len() > SIMPLE_STORAGE_RUNTIME.len());
        assert!(COUNTER_INIT_CODE.len() > COUNTER_RUNTIME.len());
        assert!(RETURN_42_INIT_CODE.len() > RETURN_42_RUNTIME.len());
    }

    #[test]
    fn test_constants() {
        assert_eq!(ONE_ETHER, 1_000_000_000_000_000_000);
        assert_eq!(ONE_GWEI, 1_000_000_000);
        assert_eq!(TRANSFER_GAS, 21_000);
    }
}
