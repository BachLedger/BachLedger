//! EVM error types

use thiserror::Error;

/// EVM execution errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EvmError {
    /// Out of gas
    #[error("out of gas")]
    OutOfGas,

    /// Stack underflow
    #[error("stack underflow")]
    StackUnderflow,

    /// Stack overflow
    #[error("stack overflow (max 1024)")]
    StackOverflow,

    /// Invalid jump destination
    #[error("invalid jump destination: {0}")]
    InvalidJump(usize),

    /// Invalid opcode
    #[error("invalid opcode: 0x{0:02x}")]
    InvalidOpcode(u8),

    /// Invalid memory access
    #[error("invalid memory access")]
    InvalidMemoryAccess,

    /// Write in static context
    #[error("state modification in static context")]
    StaticCallViolation,

    /// Return data out of bounds
    #[error("return data out of bounds")]
    ReturnDataOutOfBounds,

    /// Contract creation collision
    #[error("contract address collision")]
    CreateCollision,

    /// Max code size exceeded (EIP-170)
    #[error("max code size exceeded (limit: 24576 bytes)")]
    MaxCodeSizeExceeded,

    /// Call depth exceeded
    #[error("call depth exceeded (max 1024)")]
    CallDepthExceeded,

    /// Insufficient balance for transfer
    #[error("insufficient balance")]
    InsufficientBalance,

    /// Invalid call input
    #[error("invalid call input")]
    InvalidInput,

    /// Precompile error
    #[error("precompile error: {0}")]
    PrecompileError(String),

    /// Revert with data
    #[error("execution reverted")]
    Revert(Vec<u8>),

    /// Storage error
    #[error("storage error: {0}")]
    Storage(String),

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result type for EVM operations
pub type EvmResult<T> = Result<T, EvmError>;

/// Execution result with gas and return data
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Gas used
    pub gas_used: u64,
    /// Return data (or revert data)
    pub output: Vec<u8>,
    /// Logs emitted
    pub logs: Vec<Log>,
}

/// Log entry emitted by LOG opcodes
#[derive(Debug, Clone, Default)]
pub struct Log {
    /// Contract address that emitted the log
    pub address: bach_primitives::Address,
    /// Log topics (0-4)
    pub topics: Vec<bach_primitives::H256>,
    /// Log data
    pub data: Vec<u8>,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(gas_used: u64, output: Vec<u8>, logs: Vec<Log>) -> Self {
        Self {
            success: true,
            gas_used,
            output,
            logs,
        }
    }

    /// Create a failed result
    pub fn failure(gas_used: u64, output: Vec<u8>) -> Self {
        Self {
            success: false,
            gas_used,
            output,
            logs: Vec::new(),
        }
    }

    /// Create a revert result
    pub fn revert(gas_used: u64, output: Vec<u8>) -> Self {
        Self {
            success: false,
            gas_used,
            output,
            logs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(format!("{}", EvmError::OutOfGas), "out of gas");
        assert_eq!(format!("{}", EvmError::StackUnderflow), "stack underflow");
        assert_eq!(format!("{}", EvmError::StackOverflow), "stack overflow (max 1024)");
        assert_eq!(format!("{}", EvmError::InvalidJump(100)), "invalid jump destination: 100");
        assert_eq!(format!("{}", EvmError::InvalidOpcode(0xFE)), "invalid opcode: 0xfe");
        assert_eq!(format!("{}", EvmError::InvalidMemoryAccess), "invalid memory access");
        assert_eq!(format!("{}", EvmError::StaticCallViolation), "state modification in static context");
        assert_eq!(format!("{}", EvmError::ReturnDataOutOfBounds), "return data out of bounds");
        assert_eq!(format!("{}", EvmError::CreateCollision), "contract address collision");
        assert_eq!(format!("{}", EvmError::MaxCodeSizeExceeded), "max code size exceeded (limit: 24576 bytes)");
        assert_eq!(format!("{}", EvmError::CallDepthExceeded), "call depth exceeded (max 1024)");
        assert_eq!(format!("{}", EvmError::InsufficientBalance), "insufficient balance");
        assert_eq!(format!("{}", EvmError::InvalidInput), "invalid call input");
    }

    #[test]
    fn test_error_precompile() {
        let err = EvmError::PrecompileError("test error".to_string());
        assert_eq!(format!("{}", err), "precompile error: test error");
    }

    #[test]
    fn test_error_revert() {
        let err = EvmError::Revert(vec![1, 2, 3]);
        assert_eq!(format!("{}", err), "execution reverted");
    }

    #[test]
    fn test_error_storage() {
        let err = EvmError::Storage("db error".to_string());
        assert_eq!(format!("{}", err), "storage error: db error");
    }

    #[test]
    fn test_error_internal() {
        let err = EvmError::Internal("internal error".to_string());
        assert_eq!(format!("{}", err), "internal error: internal error");
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(EvmError::OutOfGas, EvmError::OutOfGas);
        assert_ne!(EvmError::OutOfGas, EvmError::StackUnderflow);
        assert_eq!(EvmError::InvalidJump(10), EvmError::InvalidJump(10));
        assert_ne!(EvmError::InvalidJump(10), EvmError::InvalidJump(20));
    }

    #[test]
    fn test_error_clone() {
        let err = EvmError::Revert(vec![1, 2, 3]);
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_error_debug() {
        let err = EvmError::OutOfGas;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("OutOfGas"));
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::success(100, vec![1, 2, 3], vec![]);
        assert!(result.success);
        assert_eq!(result.gas_used, 100);
        assert_eq!(result.output, vec![1, 2, 3]);
        assert!(result.logs.is_empty());
    }

    #[test]
    fn test_execution_result_success_with_logs() {
        let log = Log {
            address: bach_primitives::Address::ZERO,
            topics: vec![],
            data: vec![1, 2, 3],
        };
        let result = ExecutionResult::success(200, vec![], vec![log]);
        assert!(result.success);
        assert_eq!(result.logs.len(), 1);
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult::failure(500, vec![4, 5, 6]);
        assert!(!result.success);
        assert_eq!(result.gas_used, 500);
        assert_eq!(result.output, vec![4, 5, 6]);
        assert!(result.logs.is_empty());
    }

    #[test]
    fn test_execution_result_revert() {
        let result = ExecutionResult::revert(300, vec![7, 8, 9]);
        assert!(!result.success);
        assert_eq!(result.gas_used, 300);
        assert_eq!(result.output, vec![7, 8, 9]);
        assert!(result.logs.is_empty());
    }

    #[test]
    fn test_log_default() {
        let log: Log = Default::default();
        assert_eq!(log.address, bach_primitives::Address::ZERO);
        assert!(log.topics.is_empty());
        assert!(log.data.is_empty());
    }

    #[test]
    fn test_log_with_topics() {
        let log = Log {
            address: bach_primitives::Address::ZERO,
            topics: vec![bach_primitives::H256::ZERO; 4],
            data: vec![1, 2, 3, 4],
        };
        assert_eq!(log.topics.len(), 4);
        assert_eq!(log.data.len(), 4);
    }

    #[test]
    fn test_log_clone() {
        let log = Log {
            address: bach_primitives::Address::ZERO,
            topics: vec![bach_primitives::H256::ZERO],
            data: vec![1, 2, 3],
        };
        let cloned = log.clone();
        assert_eq!(log.address, cloned.address);
        assert_eq!(log.topics, cloned.topics);
        assert_eq!(log.data, cloned.data);
    }
}
