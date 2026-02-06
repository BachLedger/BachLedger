//! EVM opcode definitions

/// EVM opcodes (see Yellow Paper Appendix H)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum Opcode {
    // Stop and Arithmetic
    STOP = 0x00,
    ADD = 0x01,
    MUL = 0x02,
    SUB = 0x03,
    DIV = 0x04,
    SDIV = 0x05,
    MOD = 0x06,
    SMOD = 0x07,
    ADDMOD = 0x08,
    MULMOD = 0x09,
    EXP = 0x0A,
    SIGNEXTEND = 0x0B,

    // Comparison & Bitwise Logic
    LT = 0x10,
    GT = 0x11,
    SLT = 0x12,
    SGT = 0x13,
    EQ = 0x14,
    ISZERO = 0x15,
    AND = 0x16,
    OR = 0x17,
    XOR = 0x18,
    NOT = 0x19,
    BYTE = 0x1A,
    SHL = 0x1B,
    SHR = 0x1C,
    SAR = 0x1D,

    // SHA3
    KECCAK256 = 0x20,

    // Environmental Information
    ADDRESS = 0x30,
    BALANCE = 0x31,
    ORIGIN = 0x32,
    CALLER = 0x33,
    CALLVALUE = 0x34,
    CALLDATALOAD = 0x35,
    CALLDATASIZE = 0x36,
    CALLDATACOPY = 0x37,
    CODESIZE = 0x38,
    CODECOPY = 0x39,
    GASPRICE = 0x3A,
    EXTCODESIZE = 0x3B,
    EXTCODECOPY = 0x3C,
    RETURNDATASIZE = 0x3D,
    RETURNDATACOPY = 0x3E,
    EXTCODEHASH = 0x3F,

    // Block Information
    BLOCKHASH = 0x40,
    COINBASE = 0x41,
    TIMESTAMP = 0x42,
    NUMBER = 0x43,
    PREVRANDAO = 0x44, // Formerly DIFFICULTY
    GASLIMIT = 0x45,
    CHAINID = 0x46,
    SELFBALANCE = 0x47,
    BASEFEE = 0x48,

    // Stack, Memory, Storage and Flow Operations
    POP = 0x50,
    MLOAD = 0x51,
    MSTORE = 0x52,
    MSTORE8 = 0x53,
    SLOAD = 0x54,
    SSTORE = 0x55,
    JUMP = 0x56,
    JUMPI = 0x57,
    PC = 0x58,
    MSIZE = 0x59,
    GAS = 0x5A,
    JUMPDEST = 0x5B,
    TLOAD = 0x5C,
    TSTORE = 0x5D,
    MCOPY = 0x5E,
    PUSH0 = 0x5F,

    // Push Operations
    PUSH1 = 0x60,
    PUSH2 = 0x61,
    PUSH3 = 0x62,
    PUSH4 = 0x63,
    PUSH5 = 0x64,
    PUSH6 = 0x65,
    PUSH7 = 0x66,
    PUSH8 = 0x67,
    PUSH9 = 0x68,
    PUSH10 = 0x69,
    PUSH11 = 0x6A,
    PUSH12 = 0x6B,
    PUSH13 = 0x6C,
    PUSH14 = 0x6D,
    PUSH15 = 0x6E,
    PUSH16 = 0x6F,
    PUSH17 = 0x70,
    PUSH18 = 0x71,
    PUSH19 = 0x72,
    PUSH20 = 0x73,
    PUSH21 = 0x74,
    PUSH22 = 0x75,
    PUSH23 = 0x76,
    PUSH24 = 0x77,
    PUSH25 = 0x78,
    PUSH26 = 0x79,
    PUSH27 = 0x7A,
    PUSH28 = 0x7B,
    PUSH29 = 0x7C,
    PUSH30 = 0x7D,
    PUSH31 = 0x7E,
    PUSH32 = 0x7F,

    // Dup Operations
    DUP1 = 0x80,
    DUP2 = 0x81,
    DUP3 = 0x82,
    DUP4 = 0x83,
    DUP5 = 0x84,
    DUP6 = 0x85,
    DUP7 = 0x86,
    DUP8 = 0x87,
    DUP9 = 0x88,
    DUP10 = 0x89,
    DUP11 = 0x8A,
    DUP12 = 0x8B,
    DUP13 = 0x8C,
    DUP14 = 0x8D,
    DUP15 = 0x8E,
    DUP16 = 0x8F,

    // Swap Operations
    SWAP1 = 0x90,
    SWAP2 = 0x91,
    SWAP3 = 0x92,
    SWAP4 = 0x93,
    SWAP5 = 0x94,
    SWAP6 = 0x95,
    SWAP7 = 0x96,
    SWAP8 = 0x97,
    SWAP9 = 0x98,
    SWAP10 = 0x99,
    SWAP11 = 0x9A,
    SWAP12 = 0x9B,
    SWAP13 = 0x9C,
    SWAP14 = 0x9D,
    SWAP15 = 0x9E,
    SWAP16 = 0x9F,

    // Logging
    LOG0 = 0xA0,
    LOG1 = 0xA1,
    LOG2 = 0xA2,
    LOG3 = 0xA3,
    LOG4 = 0xA4,

    // System Operations
    CREATE = 0xF0,
    CALL = 0xF1,
    CALLCODE = 0xF2,
    RETURN = 0xF3,
    DELEGATECALL = 0xF4,
    CREATE2 = 0xF5,
    STATICCALL = 0xFA,
    REVERT = 0xFD,
    INVALID = 0xFE,
    SELFDESTRUCT = 0xFF,
}

impl Opcode {
    /// Try to convert from byte
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::STOP),
            0x01 => Some(Self::ADD),
            0x02 => Some(Self::MUL),
            0x03 => Some(Self::SUB),
            0x04 => Some(Self::DIV),
            0x05 => Some(Self::SDIV),
            0x06 => Some(Self::MOD),
            0x07 => Some(Self::SMOD),
            0x08 => Some(Self::ADDMOD),
            0x09 => Some(Self::MULMOD),
            0x0A => Some(Self::EXP),
            0x0B => Some(Self::SIGNEXTEND),
            0x10 => Some(Self::LT),
            0x11 => Some(Self::GT),
            0x12 => Some(Self::SLT),
            0x13 => Some(Self::SGT),
            0x14 => Some(Self::EQ),
            0x15 => Some(Self::ISZERO),
            0x16 => Some(Self::AND),
            0x17 => Some(Self::OR),
            0x18 => Some(Self::XOR),
            0x19 => Some(Self::NOT),
            0x1A => Some(Self::BYTE),
            0x1B => Some(Self::SHL),
            0x1C => Some(Self::SHR),
            0x1D => Some(Self::SAR),
            0x20 => Some(Self::KECCAK256),
            0x30 => Some(Self::ADDRESS),
            0x31 => Some(Self::BALANCE),
            0x32 => Some(Self::ORIGIN),
            0x33 => Some(Self::CALLER),
            0x34 => Some(Self::CALLVALUE),
            0x35 => Some(Self::CALLDATALOAD),
            0x36 => Some(Self::CALLDATASIZE),
            0x37 => Some(Self::CALLDATACOPY),
            0x38 => Some(Self::CODESIZE),
            0x39 => Some(Self::CODECOPY),
            0x3A => Some(Self::GASPRICE),
            0x3B => Some(Self::EXTCODESIZE),
            0x3C => Some(Self::EXTCODECOPY),
            0x3D => Some(Self::RETURNDATASIZE),
            0x3E => Some(Self::RETURNDATACOPY),
            0x3F => Some(Self::EXTCODEHASH),
            0x40 => Some(Self::BLOCKHASH),
            0x41 => Some(Self::COINBASE),
            0x42 => Some(Self::TIMESTAMP),
            0x43 => Some(Self::NUMBER),
            0x44 => Some(Self::PREVRANDAO),
            0x45 => Some(Self::GASLIMIT),
            0x46 => Some(Self::CHAINID),
            0x47 => Some(Self::SELFBALANCE),
            0x48 => Some(Self::BASEFEE),
            0x50 => Some(Self::POP),
            0x51 => Some(Self::MLOAD),
            0x52 => Some(Self::MSTORE),
            0x53 => Some(Self::MSTORE8),
            0x54 => Some(Self::SLOAD),
            0x55 => Some(Self::SSTORE),
            0x56 => Some(Self::JUMP),
            0x57 => Some(Self::JUMPI),
            0x58 => Some(Self::PC),
            0x59 => Some(Self::MSIZE),
            0x5A => Some(Self::GAS),
            0x5B => Some(Self::JUMPDEST),
            0x5C => Some(Self::TLOAD),
            0x5D => Some(Self::TSTORE),
            0x5E => Some(Self::MCOPY),
            0x5F => Some(Self::PUSH0),
            0x60..=0x7F => Some(unsafe { std::mem::transmute::<u8, Opcode>(byte) }),
            0x80..=0x8F => Some(unsafe { std::mem::transmute::<u8, Opcode>(byte) }),
            0x90..=0x9F => Some(unsafe { std::mem::transmute::<u8, Opcode>(byte) }),
            0xA0 => Some(Self::LOG0),
            0xA1 => Some(Self::LOG1),
            0xA2 => Some(Self::LOG2),
            0xA3 => Some(Self::LOG3),
            0xA4 => Some(Self::LOG4),
            0xF0 => Some(Self::CREATE),
            0xF1 => Some(Self::CALL),
            0xF2 => Some(Self::CALLCODE),
            0xF3 => Some(Self::RETURN),
            0xF4 => Some(Self::DELEGATECALL),
            0xF5 => Some(Self::CREATE2),
            0xFA => Some(Self::STATICCALL),
            0xFD => Some(Self::REVERT),
            0xFE => Some(Self::INVALID),
            0xFF => Some(Self::SELFDESTRUCT),
            _ => None,
        }
    }

    /// Get PUSH operand size (1-32 for PUSH1-PUSH32, 0 otherwise)
    pub fn push_size(self) -> usize {
        let byte = self as u8;
        if (0x60..=0x7F).contains(&byte) {
            (byte - 0x5F) as usize
        } else {
            0
        }
    }

    /// Check if this is a PUSH opcode
    pub fn is_push(self) -> bool {
        let byte = self as u8;
        (0x5F..=0x7F).contains(&byte)
    }

    /// Get DUP depth (1-16 for DUP1-DUP16, 0 otherwise)
    pub fn dup_depth(self) -> usize {
        let byte = self as u8;
        if (0x80..=0x8F).contains(&byte) {
            (byte - 0x7F) as usize
        } else {
            0
        }
    }

    /// Get SWAP depth (1-16 for SWAP1-SWAP16, 0 otherwise)
    pub fn swap_depth(self) -> usize {
        let byte = self as u8;
        if (0x90..=0x9F).contains(&byte) {
            (byte - 0x8F) as usize
        } else {
            0
        }
    }

    /// Get LOG topic count (0-4 for LOG0-LOG4, 0 otherwise)
    pub fn log_topics(self) -> usize {
        let byte = self as u8;
        if (0xA0..=0xA4).contains(&byte) {
            (byte - 0xA0) as usize
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_size() {
        assert_eq!(Opcode::PUSH1.push_size(), 1);
        assert_eq!(Opcode::PUSH16.push_size(), 16);
        assert_eq!(Opcode::PUSH32.push_size(), 32);
        assert_eq!(Opcode::ADD.push_size(), 0);
    }

    #[test]
    fn test_dup_depth() {
        assert_eq!(Opcode::DUP1.dup_depth(), 1);
        assert_eq!(Opcode::DUP16.dup_depth(), 16);
        assert_eq!(Opcode::ADD.dup_depth(), 0);
    }

    #[test]
    fn test_swap_depth() {
        assert_eq!(Opcode::SWAP1.swap_depth(), 1);
        assert_eq!(Opcode::SWAP16.swap_depth(), 16);
        assert_eq!(Opcode::ADD.swap_depth(), 0);
    }

    #[test]
    fn test_from_byte() {
        assert_eq!(Opcode::from_byte(0x00), Some(Opcode::STOP));
        assert_eq!(Opcode::from_byte(0x01), Some(Opcode::ADD));
        assert_eq!(Opcode::from_byte(0x60), Some(Opcode::PUSH1));
        assert_eq!(Opcode::from_byte(0xFF), Some(Opcode::SELFDESTRUCT));
    }

    #[test]
    fn test_block_info_opcodes() {
        // EIP-1884: SELFBALANCE is 0x47
        assert_eq!(Opcode::SELFBALANCE as u8, 0x47);
        assert_eq!(Opcode::from_byte(0x47), Some(Opcode::SELFBALANCE));

        // EIP-3198: BASEFEE is 0x48
        assert_eq!(Opcode::BASEFEE as u8, 0x48);
        assert_eq!(Opcode::from_byte(0x48), Some(Opcode::BASEFEE));

        // Other block info opcodes
        assert_eq!(Opcode::CHAINID as u8, 0x46);
        assert_eq!(Opcode::GASLIMIT as u8, 0x45);
        assert_eq!(Opcode::PREVRANDAO as u8, 0x44);
    }

    #[test]
    fn test_log_topics() {
        assert_eq!(Opcode::LOG0.log_topics(), 0);
        assert_eq!(Opcode::LOG1.log_topics(), 1);
        assert_eq!(Opcode::LOG2.log_topics(), 2);
        assert_eq!(Opcode::LOG3.log_topics(), 3);
        assert_eq!(Opcode::LOG4.log_topics(), 4);
        assert_eq!(Opcode::ADD.log_topics(), 0);
    }

    #[test]
    fn test_is_push() {
        assert!(Opcode::PUSH0.is_push());
        assert!(Opcode::PUSH1.is_push());
        assert!(Opcode::PUSH32.is_push());
        assert!(!Opcode::ADD.is_push());
        assert!(!Opcode::POP.is_push());
    }

    // ==================== Extended Opcode Tests ====================

    #[test]
    fn test_all_push_sizes() {
        // Test all PUSH sizes from PUSH1 to PUSH32
        for i in 1..=32u8 {
            let opcode_byte = 0x5F + i; // PUSH1=0x60, PUSH2=0x61, etc.
            let opcode = Opcode::from_byte(opcode_byte).unwrap();
            assert_eq!(opcode.push_size(), i as usize);
            assert!(opcode.is_push());
        }
    }

    #[test]
    fn test_push0_special() {
        // PUSH0 is 0x5F, has push_size of 0 but is_push true
        assert_eq!(Opcode::PUSH0 as u8, 0x5F);
        assert_eq!(Opcode::PUSH0.push_size(), 0);
        assert!(Opcode::PUSH0.is_push());
    }

    #[test]
    fn test_all_dup_depths() {
        // Test all DUP depths from DUP1 to DUP16
        for i in 1..=16u8 {
            let opcode_byte = 0x7F + i; // DUP1=0x80, DUP2=0x81, etc.
            let opcode = Opcode::from_byte(opcode_byte).unwrap();
            assert_eq!(opcode.dup_depth(), i as usize);
        }
    }

    #[test]
    fn test_all_swap_depths() {
        // Test all SWAP depths from SWAP1 to SWAP16
        for i in 1..=16u8 {
            let opcode_byte = 0x8F + i; // SWAP1=0x90, SWAP2=0x91, etc.
            let opcode = Opcode::from_byte(opcode_byte).unwrap();
            assert_eq!(opcode.swap_depth(), i as usize);
        }
    }

    #[test]
    fn test_arithmetic_opcodes() {
        assert_eq!(Opcode::ADD as u8, 0x01);
        assert_eq!(Opcode::MUL as u8, 0x02);
        assert_eq!(Opcode::SUB as u8, 0x03);
        assert_eq!(Opcode::DIV as u8, 0x04);
        assert_eq!(Opcode::SDIV as u8, 0x05);
        assert_eq!(Opcode::MOD as u8, 0x06);
        assert_eq!(Opcode::SMOD as u8, 0x07);
        assert_eq!(Opcode::ADDMOD as u8, 0x08);
        assert_eq!(Opcode::MULMOD as u8, 0x09);
        assert_eq!(Opcode::EXP as u8, 0x0A);
        assert_eq!(Opcode::SIGNEXTEND as u8, 0x0B);
    }

    #[test]
    fn test_comparison_opcodes() {
        assert_eq!(Opcode::LT as u8, 0x10);
        assert_eq!(Opcode::GT as u8, 0x11);
        assert_eq!(Opcode::SLT as u8, 0x12);
        assert_eq!(Opcode::SGT as u8, 0x13);
        assert_eq!(Opcode::EQ as u8, 0x14);
        assert_eq!(Opcode::ISZERO as u8, 0x15);
    }

    #[test]
    fn test_bitwise_opcodes() {
        assert_eq!(Opcode::AND as u8, 0x16);
        assert_eq!(Opcode::OR as u8, 0x17);
        assert_eq!(Opcode::XOR as u8, 0x18);
        assert_eq!(Opcode::NOT as u8, 0x19);
        assert_eq!(Opcode::BYTE as u8, 0x1A);
        assert_eq!(Opcode::SHL as u8, 0x1B);
        assert_eq!(Opcode::SHR as u8, 0x1C);
        assert_eq!(Opcode::SAR as u8, 0x1D);
    }

    #[test]
    fn test_environmental_opcodes() {
        assert_eq!(Opcode::ADDRESS as u8, 0x30);
        assert_eq!(Opcode::BALANCE as u8, 0x31);
        assert_eq!(Opcode::ORIGIN as u8, 0x32);
        assert_eq!(Opcode::CALLER as u8, 0x33);
        assert_eq!(Opcode::CALLVALUE as u8, 0x34);
        assert_eq!(Opcode::CALLDATALOAD as u8, 0x35);
        assert_eq!(Opcode::CALLDATASIZE as u8, 0x36);
        assert_eq!(Opcode::CALLDATACOPY as u8, 0x37);
        assert_eq!(Opcode::CODESIZE as u8, 0x38);
        assert_eq!(Opcode::CODECOPY as u8, 0x39);
        assert_eq!(Opcode::GASPRICE as u8, 0x3A);
        assert_eq!(Opcode::EXTCODESIZE as u8, 0x3B);
        assert_eq!(Opcode::EXTCODECOPY as u8, 0x3C);
        assert_eq!(Opcode::RETURNDATASIZE as u8, 0x3D);
        assert_eq!(Opcode::RETURNDATACOPY as u8, 0x3E);
        assert_eq!(Opcode::EXTCODEHASH as u8, 0x3F);
    }

    #[test]
    fn test_memory_storage_opcodes() {
        assert_eq!(Opcode::POP as u8, 0x50);
        assert_eq!(Opcode::MLOAD as u8, 0x51);
        assert_eq!(Opcode::MSTORE as u8, 0x52);
        assert_eq!(Opcode::MSTORE8 as u8, 0x53);
        assert_eq!(Opcode::SLOAD as u8, 0x54);
        assert_eq!(Opcode::SSTORE as u8, 0x55);
        assert_eq!(Opcode::JUMP as u8, 0x56);
        assert_eq!(Opcode::JUMPI as u8, 0x57);
        assert_eq!(Opcode::PC as u8, 0x58);
        assert_eq!(Opcode::MSIZE as u8, 0x59);
        assert_eq!(Opcode::GAS as u8, 0x5A);
        assert_eq!(Opcode::JUMPDEST as u8, 0x5B);
    }

    #[test]
    fn test_transient_storage_opcodes() {
        // EIP-1153: Transient storage
        assert_eq!(Opcode::TLOAD as u8, 0x5C);
        assert_eq!(Opcode::TSTORE as u8, 0x5D);
    }

    #[test]
    fn test_mcopy_opcode() {
        // EIP-5656: MCOPY
        assert_eq!(Opcode::MCOPY as u8, 0x5E);
    }

    #[test]
    fn test_system_opcodes() {
        assert_eq!(Opcode::CREATE as u8, 0xF0);
        assert_eq!(Opcode::CALL as u8, 0xF1);
        assert_eq!(Opcode::CALLCODE as u8, 0xF2);
        assert_eq!(Opcode::RETURN as u8, 0xF3);
        assert_eq!(Opcode::DELEGATECALL as u8, 0xF4);
        assert_eq!(Opcode::CREATE2 as u8, 0xF5);
        assert_eq!(Opcode::STATICCALL as u8, 0xFA);
        assert_eq!(Opcode::REVERT as u8, 0xFD);
        assert_eq!(Opcode::INVALID as u8, 0xFE);
        assert_eq!(Opcode::SELFDESTRUCT as u8, 0xFF);
    }

    #[test]
    fn test_from_byte_invalid() {
        // Test some invalid opcodes (gaps in opcode table)
        assert_eq!(Opcode::from_byte(0x0C), None); // Gap after SIGNEXTEND
        assert_eq!(Opcode::from_byte(0x0D), None);
        assert_eq!(Opcode::from_byte(0x0E), None);
        assert_eq!(Opcode::from_byte(0x0F), None);
        assert_eq!(Opcode::from_byte(0x21), None); // Gap after KECCAK256
        assert_eq!(Opcode::from_byte(0x49), None); // Gap after BASEFEE
        assert_eq!(Opcode::from_byte(0xA5), None); // Gap after LOG4
        assert_eq!(Opcode::from_byte(0xF6), None); // Gap between CREATE2 and STATICCALL
    }

    #[test]
    fn test_non_push_push_size() {
        // Non-PUSH opcodes should have push_size of 0
        assert_eq!(Opcode::STOP.push_size(), 0);
        assert_eq!(Opcode::ADD.push_size(), 0);
        assert_eq!(Opcode::CALL.push_size(), 0);
        assert_eq!(Opcode::DUP1.push_size(), 0);
        assert_eq!(Opcode::SWAP1.push_size(), 0);
    }

    #[test]
    fn test_non_dup_dup_depth() {
        // Non-DUP opcodes should have dup_depth of 0
        assert_eq!(Opcode::STOP.dup_depth(), 0);
        assert_eq!(Opcode::PUSH1.dup_depth(), 0);
        assert_eq!(Opcode::SWAP1.dup_depth(), 0);
    }

    #[test]
    fn test_non_swap_swap_depth() {
        // Non-SWAP opcodes should have swap_depth of 0
        assert_eq!(Opcode::STOP.swap_depth(), 0);
        assert_eq!(Opcode::PUSH1.swap_depth(), 0);
        assert_eq!(Opcode::DUP1.swap_depth(), 0);
    }

    #[test]
    fn test_non_log_log_topics() {
        // Non-LOG opcodes should have log_topics of 0
        assert_eq!(Opcode::STOP.log_topics(), 0);
        assert_eq!(Opcode::PUSH1.log_topics(), 0);
        assert_eq!(Opcode::CALL.log_topics(), 0);
    }

    #[test]
    fn test_opcode_debug() {
        // Test Debug trait
        let debug_str = format!("{:?}", Opcode::ADD);
        assert!(debug_str.contains("ADD"));
    }

    #[test]
    fn test_opcode_clone() {
        let op = Opcode::MUL;
        let cloned = op.clone();
        assert_eq!(op, cloned);
    }

    #[test]
    fn test_opcode_copy() {
        let op = Opcode::DIV;
        let copied: Opcode = op; // Copy trait
        assert_eq!(op, copied);
    }

    #[test]
    fn test_all_valid_opcodes_roundtrip() {
        // Test that all defined opcodes can roundtrip through from_byte
        let opcodes = [
            Opcode::STOP, Opcode::ADD, Opcode::MUL, Opcode::SUB, Opcode::DIV,
            Opcode::SDIV, Opcode::MOD, Opcode::SMOD, Opcode::ADDMOD, Opcode::MULMOD,
            Opcode::EXP, Opcode::SIGNEXTEND, Opcode::LT, Opcode::GT, Opcode::SLT,
            Opcode::SGT, Opcode::EQ, Opcode::ISZERO, Opcode::AND, Opcode::OR,
            Opcode::XOR, Opcode::NOT, Opcode::BYTE, Opcode::SHL, Opcode::SHR,
            Opcode::SAR, Opcode::KECCAK256, Opcode::ADDRESS, Opcode::BALANCE,
            Opcode::ORIGIN, Opcode::CALLER, Opcode::CALLVALUE, Opcode::CALLDATALOAD,
            Opcode::CALLDATASIZE, Opcode::CALLDATACOPY, Opcode::CODESIZE,
            Opcode::CODECOPY, Opcode::GASPRICE, Opcode::EXTCODESIZE, Opcode::EXTCODECOPY,
            Opcode::RETURNDATASIZE, Opcode::RETURNDATACOPY, Opcode::EXTCODEHASH,
            Opcode::BLOCKHASH, Opcode::COINBASE, Opcode::TIMESTAMP, Opcode::NUMBER,
            Opcode::PREVRANDAO, Opcode::GASLIMIT, Opcode::CHAINID, Opcode::SELFBALANCE,
            Opcode::BASEFEE, Opcode::POP, Opcode::MLOAD, Opcode::MSTORE, Opcode::MSTORE8,
            Opcode::SLOAD, Opcode::SSTORE, Opcode::JUMP, Opcode::JUMPI, Opcode::PC,
            Opcode::MSIZE, Opcode::GAS, Opcode::JUMPDEST, Opcode::TLOAD, Opcode::TSTORE,
            Opcode::MCOPY, Opcode::PUSH0, Opcode::LOG0, Opcode::LOG1, Opcode::LOG2,
            Opcode::LOG3, Opcode::LOG4, Opcode::CREATE, Opcode::CALL, Opcode::CALLCODE,
            Opcode::RETURN, Opcode::DELEGATECALL, Opcode::CREATE2, Opcode::STATICCALL,
            Opcode::REVERT, Opcode::INVALID, Opcode::SELFDESTRUCT,
        ];

        for op in opcodes {
            let byte = op as u8;
            let recovered = Opcode::from_byte(byte);
            assert_eq!(recovered, Some(op), "Failed roundtrip for {:?} (0x{:02x})", op, byte);
        }
    }
}
