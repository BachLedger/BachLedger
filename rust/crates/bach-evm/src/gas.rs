//! Gas cost calculations

use crate::opcode::Opcode;

/// Gas costs for EVM operations
pub mod cost {
    /// Zero gas
    pub const ZERO: u64 = 0;
    /// Base gas
    pub const BASE: u64 = 2;
    /// Very low gas
    pub const VERYLOW: u64 = 3;
    /// Low gas
    pub const LOW: u64 = 5;
    /// Mid gas
    pub const MID: u64 = 8;
    /// High gas
    pub const HIGH: u64 = 10;

    /// Jump dest gas
    pub const JUMPDEST: u64 = 1;
    /// Exp gas
    pub const EXP: u64 = 10;
    /// Exp byte gas
    pub const EXP_BYTE: u64 = 50;
    /// SHA3 base gas
    pub const SHA3: u64 = 30;
    /// SHA3 word gas
    pub const SHA3_WORD: u64 = 6;

    /// Balance gas (EIP-2929 cold)
    pub const BALANCE_COLD: u64 = 2600;
    /// Balance gas (warm)
    pub const BALANCE_WARM: u64 = 100;
    /// Ext code size gas (cold)
    pub const EXTCODESIZE_COLD: u64 = 2600;
    /// Ext code size gas (warm)
    pub const EXTCODESIZE_WARM: u64 = 100;
    /// Ext code copy base (cold)
    pub const EXTCODECOPY_COLD: u64 = 2600;
    /// Ext code copy base (warm)
    pub const EXTCODECOPY_WARM: u64 = 100;
    /// Ext code hash (cold)
    pub const EXTCODEHASH_COLD: u64 = 2600;
    /// Ext code hash (warm)
    pub const EXTCODEHASH_WARM: u64 = 100;

    /// Sload gas (cold)
    pub const SLOAD_COLD: u64 = 2100;
    /// Sload gas (warm)
    pub const SLOAD_WARM: u64 = 100;
    /// Sstore set gas
    pub const SSTORE_SET: u64 = 20000;
    /// Sstore reset gas
    pub const SSTORE_RESET: u64 = 2900;
    /// Sstore clear refund
    pub const SSTORE_CLEAR_REFUND: u64 = 4800;

    /// Log gas
    pub const LOG: u64 = 375;
    /// Log topic gas
    pub const LOG_TOPIC: u64 = 375;
    /// Log data gas (per byte)
    pub const LOG_DATA: u64 = 8;

    /// Create gas
    pub const CREATE: u64 = 32000;
    /// Create2 gas
    pub const CREATE2: u64 = 32000;
    /// Call gas (cold)
    pub const CALL_COLD: u64 = 2600;
    /// Call gas (warm)
    pub const CALL_WARM: u64 = 100;
    /// Call value transfer gas
    pub const CALL_VALUE: u64 = 9000;
    /// Call new account gas
    pub const CALL_NEW_ACCOUNT: u64 = 25000;
    /// Call stipend
    pub const CALL_STIPEND: u64 = 2300;

    /// Memory gas per word
    pub const MEMORY: u64 = 3;
    /// Copy gas per word
    pub const COPY: u64 = 3;

    /// Transaction gas
    pub const TX: u64 = 21000;
    /// Transaction create gas
    pub const TX_CREATE: u64 = 32000;
    /// Transaction data zero byte
    pub const TX_DATA_ZERO: u64 = 4;
    /// Transaction data non-zero byte
    pub const TX_DATA_NONZERO: u64 = 16;
    /// Access list address gas
    pub const ACCESS_LIST_ADDRESS: u64 = 2400;
    /// Access list storage key gas
    pub const ACCESS_LIST_STORAGE_KEY: u64 = 1900;

    /// Selfdestruct gas
    pub const SELFDESTRUCT: u64 = 5000;
    /// Selfdestruct new account gas
    pub const SELFDESTRUCT_NEW_ACCOUNT: u64 = 25000;

    /// Max call depth
    pub const MAX_CALL_DEPTH: usize = 1024;
    /// Max stack size
    pub const MAX_STACK_SIZE: usize = 1024;
    /// Max code size (EIP-170)
    pub const MAX_CODE_SIZE: usize = 24576;
    /// Max init code size (EIP-3860)
    pub const MAX_INIT_CODE_SIZE: usize = 49152;
}

/// Get static gas cost for an opcode
pub fn static_gas(opcode: Opcode) -> u64 {
    match opcode {
        // Zero cost
        Opcode::STOP | Opcode::RETURN | Opcode::REVERT => cost::ZERO,

        // Base cost (2)
        Opcode::ADDRESS | Opcode::ORIGIN | Opcode::CALLER | Opcode::CALLVALUE |
        Opcode::CALLDATASIZE | Opcode::CODESIZE | Opcode::GASPRICE |
        Opcode::COINBASE | Opcode::TIMESTAMP | Opcode::NUMBER |
        Opcode::PREVRANDAO | Opcode::GASLIMIT | Opcode::CHAINID |
        Opcode::RETURNDATASIZE | Opcode::POP | Opcode::PC |
        Opcode::MSIZE | Opcode::GAS | Opcode::BASEFEE | Opcode::PUSH0 => cost::BASE,

        // Very low cost (3)
        Opcode::ADD | Opcode::SUB | Opcode::NOT | Opcode::LT | Opcode::GT |
        Opcode::SLT | Opcode::SGT | Opcode::EQ | Opcode::ISZERO |
        Opcode::AND | Opcode::OR | Opcode::XOR | Opcode::BYTE |
        Opcode::SHL | Opcode::SHR | Opcode::SAR |
        Opcode::CALLDATALOAD | Opcode::MLOAD | Opcode::MSTORE | Opcode::MSTORE8 |
        Opcode::PUSH1 | Opcode::PUSH2 | Opcode::PUSH3 | Opcode::PUSH4 |
        Opcode::PUSH5 | Opcode::PUSH6 | Opcode::PUSH7 | Opcode::PUSH8 |
        Opcode::PUSH9 | Opcode::PUSH10 | Opcode::PUSH11 | Opcode::PUSH12 |
        Opcode::PUSH13 | Opcode::PUSH14 | Opcode::PUSH15 | Opcode::PUSH16 |
        Opcode::PUSH17 | Opcode::PUSH18 | Opcode::PUSH19 | Opcode::PUSH20 |
        Opcode::PUSH21 | Opcode::PUSH22 | Opcode::PUSH23 | Opcode::PUSH24 |
        Opcode::PUSH25 | Opcode::PUSH26 | Opcode::PUSH27 | Opcode::PUSH28 |
        Opcode::PUSH29 | Opcode::PUSH30 | Opcode::PUSH31 | Opcode::PUSH32 |
        Opcode::DUP1 | Opcode::DUP2 | Opcode::DUP3 | Opcode::DUP4 |
        Opcode::DUP5 | Opcode::DUP6 | Opcode::DUP7 | Opcode::DUP8 |
        Opcode::DUP9 | Opcode::DUP10 | Opcode::DUP11 | Opcode::DUP12 |
        Opcode::DUP13 | Opcode::DUP14 | Opcode::DUP15 | Opcode::DUP16 |
        Opcode::SWAP1 | Opcode::SWAP2 | Opcode::SWAP3 | Opcode::SWAP4 |
        Opcode::SWAP5 | Opcode::SWAP6 | Opcode::SWAP7 | Opcode::SWAP8 |
        Opcode::SWAP9 | Opcode::SWAP10 | Opcode::SWAP11 | Opcode::SWAP12 |
        Opcode::SWAP13 | Opcode::SWAP14 | Opcode::SWAP15 | Opcode::SWAP16 => cost::VERYLOW,

        // Low cost (5)
        Opcode::MUL | Opcode::DIV | Opcode::SDIV | Opcode::MOD |
        Opcode::SMOD | Opcode::SIGNEXTEND | Opcode::SELFBALANCE => cost::LOW,

        // Mid cost (8)
        Opcode::ADDMOD | Opcode::MULMOD | Opcode::JUMP => cost::MID,

        // High cost (10)
        Opcode::JUMPI => cost::HIGH,

        // Jump destination
        Opcode::JUMPDEST => cost::JUMPDEST,

        // Special costs (dynamic)
        Opcode::EXP => cost::EXP,
        Opcode::KECCAK256 => cost::SHA3,
        Opcode::BALANCE => cost::BALANCE_WARM, // Default warm, adjust if cold
        Opcode::EXTCODESIZE => cost::EXTCODESIZE_WARM,
        Opcode::EXTCODECOPY => cost::EXTCODECOPY_WARM,
        Opcode::EXTCODEHASH => cost::EXTCODEHASH_WARM,
        Opcode::BLOCKHASH => 20,
        Opcode::SLOAD => cost::SLOAD_WARM,
        Opcode::SSTORE => 0, // Dynamic based on storage state
        Opcode::LOG0 => cost::LOG,
        Opcode::LOG1 => cost::LOG + cost::LOG_TOPIC,
        Opcode::LOG2 => cost::LOG + 2 * cost::LOG_TOPIC,
        Opcode::LOG3 => cost::LOG + 3 * cost::LOG_TOPIC,
        Opcode::LOG4 => cost::LOG + 4 * cost::LOG_TOPIC,
        Opcode::CREATE => cost::CREATE,
        Opcode::CREATE2 => cost::CREATE2,
        Opcode::CALL | Opcode::CALLCODE | Opcode::DELEGATECALL |
        Opcode::STATICCALL => cost::CALL_WARM,
        Opcode::SELFDESTRUCT => cost::SELFDESTRUCT,
        Opcode::CALLDATACOPY | Opcode::CODECOPY |
        Opcode::RETURNDATACOPY | Opcode::MCOPY => cost::VERYLOW,
        Opcode::TLOAD | Opcode::TSTORE => cost::SLOAD_WARM,
        Opcode::INVALID => 0,
    }
}

/// Calculate memory expansion cost
pub fn memory_gas(current_size: usize, new_size: usize) -> u64 {
    if new_size <= current_size {
        return 0;
    }

    let new_words = new_size.div_ceil(32);
    let old_words = current_size.div_ceil(32);

    let new_cost = memory_word_cost(new_words);
    let old_cost = memory_word_cost(old_words);

    new_cost.saturating_sub(old_cost)
}

/// Calculate memory cost for a number of words
fn memory_word_cost(words: usize) -> u64 {
    let words = words as u64;
    cost::MEMORY * words + words * words / 512
}

/// Calculate copy cost (for CALLDATACOPY, CODECOPY, etc.)
pub fn copy_gas(length: usize) -> u64 {
    let words = length.div_ceil(32);
    cost::COPY * words as u64
}

/// Calculate EXP gas cost
pub fn exp_gas(exponent: &[u8; 32]) -> u64 {
    let mut byte_size = 32;
    for (i, &b) in exponent.iter().enumerate() {
        if b != 0 {
            byte_size = 32 - i;
            break;
        }
    }
    if exponent.iter().all(|&b| b == 0) {
        byte_size = 0;
    }
    cost::EXP + cost::EXP_BYTE * byte_size as u64
}

/// Calculate SHA3/KECCAK256 gas cost
pub fn sha3_gas(length: usize) -> u64 {
    let words = length.div_ceil(32);
    cost::SHA3 + cost::SHA3_WORD * words as u64
}

/// Calculate LOG gas cost
pub fn log_gas(topics: usize, data_size: usize) -> u64 {
    cost::LOG + cost::LOG_TOPIC * topics as u64 + cost::LOG_DATA * data_size as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_gas() {
        assert_eq!(static_gas(Opcode::STOP), 0);
        assert_eq!(static_gas(Opcode::ADD), 3);
        assert_eq!(static_gas(Opcode::MUL), 5);
        assert_eq!(static_gas(Opcode::JUMP), 8);
        assert_eq!(static_gas(Opcode::JUMPI), 10);
    }

    #[test]
    fn test_memory_gas() {
        // No expansion
        assert_eq!(memory_gas(32, 32), 0);
        assert_eq!(memory_gas(64, 32), 0);

        // Expansion
        assert!(memory_gas(0, 32) > 0);
        assert!(memory_gas(32, 64) > 0);
    }

    #[test]
    fn test_copy_gas() {
        assert_eq!(copy_gas(0), 0);
        assert_eq!(copy_gas(32), 3);
        assert_eq!(copy_gas(64), 6);
        assert_eq!(copy_gas(33), 6); // 2 words
    }

    #[test]
    fn test_exp_gas() {
        let zero = [0u8; 32];
        assert_eq!(exp_gas(&zero), cost::EXP);

        let mut one = [0u8; 32];
        one[31] = 1;
        assert_eq!(exp_gas(&one), cost::EXP + cost::EXP_BYTE);

        let mut big = [0u8; 32];
        big[0] = 1;
        assert_eq!(exp_gas(&big), cost::EXP + 32 * cost::EXP_BYTE);
    }

    #[test]
    fn test_sha3_gas() {
        assert_eq!(sha3_gas(0), cost::SHA3);
        assert_eq!(sha3_gas(32), cost::SHA3 + cost::SHA3_WORD);
        assert_eq!(sha3_gas(64), cost::SHA3 + 2 * cost::SHA3_WORD);
    }

    #[test]
    fn test_log_gas() {
        assert_eq!(log_gas(0, 0), cost::LOG);
        assert_eq!(log_gas(1, 0), cost::LOG + cost::LOG_TOPIC);
        assert_eq!(log_gas(0, 32), cost::LOG + 32 * cost::LOG_DATA);
        assert_eq!(log_gas(2, 64), cost::LOG + 2 * cost::LOG_TOPIC + 64 * cost::LOG_DATA);
    }

    // ==================== Extended Gas Tests ====================

    #[test]
    fn test_static_gas_all_categories() {
        // Zero cost
        assert_eq!(static_gas(Opcode::STOP), cost::ZERO);
        assert_eq!(static_gas(Opcode::RETURN), cost::ZERO);
        assert_eq!(static_gas(Opcode::REVERT), cost::ZERO);

        // Base cost (2)
        assert_eq!(static_gas(Opcode::ADDRESS), cost::BASE);
        assert_eq!(static_gas(Opcode::ORIGIN), cost::BASE);
        assert_eq!(static_gas(Opcode::CALLER), cost::BASE);
        assert_eq!(static_gas(Opcode::CALLVALUE), cost::BASE);
        assert_eq!(static_gas(Opcode::CALLDATASIZE), cost::BASE);
        assert_eq!(static_gas(Opcode::CODESIZE), cost::BASE);
        assert_eq!(static_gas(Opcode::GASPRICE), cost::BASE);
        assert_eq!(static_gas(Opcode::COINBASE), cost::BASE);
        assert_eq!(static_gas(Opcode::TIMESTAMP), cost::BASE);
        assert_eq!(static_gas(Opcode::NUMBER), cost::BASE);
        assert_eq!(static_gas(Opcode::PREVRANDAO), cost::BASE);
        assert_eq!(static_gas(Opcode::GASLIMIT), cost::BASE);
        assert_eq!(static_gas(Opcode::CHAINID), cost::BASE);
        assert_eq!(static_gas(Opcode::RETURNDATASIZE), cost::BASE);
        assert_eq!(static_gas(Opcode::POP), cost::BASE);
        assert_eq!(static_gas(Opcode::PC), cost::BASE);
        assert_eq!(static_gas(Opcode::MSIZE), cost::BASE);
        assert_eq!(static_gas(Opcode::GAS), cost::BASE);
        assert_eq!(static_gas(Opcode::BASEFEE), cost::BASE);
        assert_eq!(static_gas(Opcode::PUSH0), cost::BASE);

        // Very low cost (3)
        assert_eq!(static_gas(Opcode::ADD), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::SUB), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::NOT), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::LT), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::GT), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::SLT), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::SGT), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::EQ), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::ISZERO), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::AND), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::OR), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::XOR), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::BYTE), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::SHL), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::SHR), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::SAR), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::CALLDATALOAD), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::MLOAD), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::MSTORE), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::MSTORE8), cost::VERYLOW);

        // Low cost (5)
        assert_eq!(static_gas(Opcode::MUL), cost::LOW);
        assert_eq!(static_gas(Opcode::DIV), cost::LOW);
        assert_eq!(static_gas(Opcode::SDIV), cost::LOW);
        assert_eq!(static_gas(Opcode::MOD), cost::LOW);
        assert_eq!(static_gas(Opcode::SMOD), cost::LOW);
        assert_eq!(static_gas(Opcode::SIGNEXTEND), cost::LOW);
        assert_eq!(static_gas(Opcode::SELFBALANCE), cost::LOW);

        // Mid cost (8)
        assert_eq!(static_gas(Opcode::ADDMOD), cost::MID);
        assert_eq!(static_gas(Opcode::MULMOD), cost::MID);
        assert_eq!(static_gas(Opcode::JUMP), cost::MID);

        // High cost (10)
        assert_eq!(static_gas(Opcode::JUMPI), cost::HIGH);

        // Jump dest (1)
        assert_eq!(static_gas(Opcode::JUMPDEST), cost::JUMPDEST);
    }

    #[test]
    fn test_static_gas_push_operations() {
        // All PUSH operations have VERYLOW cost
        assert_eq!(static_gas(Opcode::PUSH1), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::PUSH16), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::PUSH32), cost::VERYLOW);
    }

    #[test]
    fn test_static_gas_dup_operations() {
        // All DUP operations have VERYLOW cost
        assert_eq!(static_gas(Opcode::DUP1), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::DUP8), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::DUP16), cost::VERYLOW);
    }

    #[test]
    fn test_static_gas_swap_operations() {
        // All SWAP operations have VERYLOW cost
        assert_eq!(static_gas(Opcode::SWAP1), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::SWAP8), cost::VERYLOW);
        assert_eq!(static_gas(Opcode::SWAP16), cost::VERYLOW);
    }

    #[test]
    fn test_static_gas_log_operations() {
        assert_eq!(static_gas(Opcode::LOG0), cost::LOG);
        assert_eq!(static_gas(Opcode::LOG1), cost::LOG + cost::LOG_TOPIC);
        assert_eq!(static_gas(Opcode::LOG2), cost::LOG + 2 * cost::LOG_TOPIC);
        assert_eq!(static_gas(Opcode::LOG3), cost::LOG + 3 * cost::LOG_TOPIC);
        assert_eq!(static_gas(Opcode::LOG4), cost::LOG + 4 * cost::LOG_TOPIC);
    }

    #[test]
    fn test_static_gas_system_operations() {
        assert_eq!(static_gas(Opcode::CREATE), cost::CREATE);
        assert_eq!(static_gas(Opcode::CREATE2), cost::CREATE2);
        assert_eq!(static_gas(Opcode::CALL), cost::CALL_WARM);
        assert_eq!(static_gas(Opcode::CALLCODE), cost::CALL_WARM);
        assert_eq!(static_gas(Opcode::DELEGATECALL), cost::CALL_WARM);
        assert_eq!(static_gas(Opcode::STATICCALL), cost::CALL_WARM);
        assert_eq!(static_gas(Opcode::SELFDESTRUCT), cost::SELFDESTRUCT);
    }

    #[test]
    fn test_memory_gas_quadratic() {
        // Memory cost is linear + quadratic: 3*words + words^2/512
        // First 32 bytes (1 word): 3*1 + 1/512 = 3
        assert_eq!(memory_gas(0, 32), 3);

        // 64 bytes (2 words): 3*2 + 4/512 = 6
        assert_eq!(memory_gas(0, 64), 6);

        // 32 words (1024 bytes): 3*32 + 32*32/512 = 96 + 2 = 98
        assert_eq!(memory_gas(0, 1024), 98);

        // 512 words (16384 bytes): 3*512 + 512*512/512 = 1536 + 512 = 2048
        assert_eq!(memory_gas(0, 16384), 2048);
    }

    #[test]
    fn test_memory_gas_incremental() {
        // Expanding from 32 to 64 bytes
        let cost_32 = memory_gas(0, 32);
        let cost_64 = memory_gas(0, 64);
        let cost_32_to_64 = memory_gas(32, 64);

        assert_eq!(cost_32_to_64, cost_64 - cost_32);
    }

    #[test]
    fn test_copy_gas_word_rounding() {
        // 0 bytes = 0 words
        assert_eq!(copy_gas(0), 0);

        // 1-32 bytes = 1 word
        assert_eq!(copy_gas(1), cost::COPY);
        assert_eq!(copy_gas(32), cost::COPY);

        // 33-64 bytes = 2 words
        assert_eq!(copy_gas(33), 2 * cost::COPY);
        assert_eq!(copy_gas(64), 2 * cost::COPY);

        // 100 bytes = 4 words
        assert_eq!(copy_gas(100), 4 * cost::COPY);
    }

    #[test]
    fn test_exp_gas_byte_count() {
        // Exponent 0 = 0 bytes
        let zero = [0u8; 32];
        assert_eq!(exp_gas(&zero), cost::EXP);

        // Exponent 1 (in last byte) = 1 byte
        let mut one = [0u8; 32];
        one[31] = 1;
        assert_eq!(exp_gas(&one), cost::EXP + cost::EXP_BYTE);

        // Exponent 0xFF (in last byte) = 1 byte
        let mut byte_ff = [0u8; 32];
        byte_ff[31] = 0xFF;
        assert_eq!(exp_gas(&byte_ff), cost::EXP + cost::EXP_BYTE);

        // Exponent 0x100 (in last 2 bytes) = 2 bytes
        let mut two_bytes = [0u8; 32];
        two_bytes[30] = 1;
        assert_eq!(exp_gas(&two_bytes), cost::EXP + 2 * cost::EXP_BYTE);

        // Exponent with first byte set = 32 bytes
        let mut full = [0u8; 32];
        full[0] = 1;
        assert_eq!(exp_gas(&full), cost::EXP + 32 * cost::EXP_BYTE);
    }

    #[test]
    fn test_sha3_gas_word_rounding() {
        // 0 bytes
        assert_eq!(sha3_gas(0), cost::SHA3);

        // 1-32 bytes = 1 word
        assert_eq!(sha3_gas(1), cost::SHA3 + cost::SHA3_WORD);
        assert_eq!(sha3_gas(32), cost::SHA3 + cost::SHA3_WORD);

        // 33-64 bytes = 2 words
        assert_eq!(sha3_gas(33), cost::SHA3 + 2 * cost::SHA3_WORD);
        assert_eq!(sha3_gas(64), cost::SHA3 + 2 * cost::SHA3_WORD);
    }

    #[test]
    fn test_log_gas_all_topics() {
        // LOG0: 375
        assert_eq!(log_gas(0, 0), cost::LOG);

        // LOG1: 375 + 375 = 750
        assert_eq!(log_gas(1, 0), cost::LOG + cost::LOG_TOPIC);

        // LOG2: 375 + 750 = 1125
        assert_eq!(log_gas(2, 0), cost::LOG + 2 * cost::LOG_TOPIC);

        // LOG3: 375 + 1125 = 1500
        assert_eq!(log_gas(3, 0), cost::LOG + 3 * cost::LOG_TOPIC);

        // LOG4: 375 + 1500 = 1875
        assert_eq!(log_gas(4, 0), cost::LOG + 4 * cost::LOG_TOPIC);
    }

    #[test]
    fn test_log_gas_with_data() {
        // LOG0 with 100 bytes of data
        assert_eq!(log_gas(0, 100), cost::LOG + 100 * cost::LOG_DATA);

        // LOG2 with 50 bytes of data
        assert_eq!(log_gas(2, 50), cost::LOG + 2 * cost::LOG_TOPIC + 50 * cost::LOG_DATA);
    }

    #[test]
    fn test_gas_constants() {
        // Verify important constants
        assert_eq!(cost::MAX_STACK_SIZE, 1024);
        assert_eq!(cost::MAX_CALL_DEPTH, 1024);
        assert_eq!(cost::MAX_CODE_SIZE, 24576);
        assert_eq!(cost::MAX_INIT_CODE_SIZE, 49152);
        assert_eq!(cost::TX, 21000);
        assert_eq!(cost::TX_CREATE, 32000);
    }

    #[test]
    fn test_eip2929_cold_warm_costs() {
        // EIP-2929 access costs
        assert!(cost::BALANCE_COLD > cost::BALANCE_WARM);
        assert!(cost::EXTCODESIZE_COLD > cost::EXTCODESIZE_WARM);
        assert!(cost::EXTCODECOPY_COLD > cost::EXTCODECOPY_WARM);
        assert!(cost::EXTCODEHASH_COLD > cost::EXTCODEHASH_WARM);
        assert!(cost::SLOAD_COLD > cost::SLOAD_WARM);
        assert!(cost::CALL_COLD > cost::CALL_WARM);

        // Specific values
        assert_eq!(cost::BALANCE_COLD, 2600);
        assert_eq!(cost::BALANCE_WARM, 100);
        assert_eq!(cost::SLOAD_COLD, 2100);
        assert_eq!(cost::SLOAD_WARM, 100);
    }

    #[test]
    fn test_sstore_costs() {
        assert_eq!(cost::SSTORE_SET, 20000);
        assert_eq!(cost::SSTORE_RESET, 2900);
        assert_eq!(cost::SSTORE_CLEAR_REFUND, 4800);
    }

    #[test]
    fn test_call_costs() {
        assert_eq!(cost::CALL_VALUE, 9000);
        assert_eq!(cost::CALL_NEW_ACCOUNT, 25000);
        assert_eq!(cost::CALL_STIPEND, 2300);
    }

    #[test]
    fn test_transient_storage_gas() {
        // EIP-1153: TLOAD and TSTORE have same cost as warm SLOAD
        assert_eq!(static_gas(Opcode::TLOAD), cost::SLOAD_WARM);
        assert_eq!(static_gas(Opcode::TSTORE), cost::SLOAD_WARM);
    }

    #[test]
    fn test_mcopy_gas() {
        // EIP-5656: MCOPY has VERYLOW static cost + copy cost
        assert_eq!(static_gas(Opcode::MCOPY), cost::VERYLOW);
    }
}
