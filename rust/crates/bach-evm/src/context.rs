//! Execution context for EVM

use bach_primitives::{Address, H256};

/// Call context information
#[derive(Clone, Debug)]
pub struct CallContext {
    /// Contract address being executed
    pub address: Address,
    /// Caller address
    pub caller: Address,
    /// Call value in wei
    pub value: u128,
    /// Call data
    pub data: Vec<u8>,
    /// Gas limit
    pub gas: u64,
    /// Whether this is a static call (no state modifications)
    pub is_static: bool,
    /// Call depth
    pub depth: usize,
}

impl CallContext {
    /// Create a new call context
    pub fn new(
        address: Address,
        caller: Address,
        value: u128,
        data: Vec<u8>,
        gas: u64,
    ) -> Self {
        Self {
            address,
            caller,
            value,
            data,
            gas,
            is_static: false,
            depth: 0,
        }
    }

    /// Create a static call context
    pub fn new_static(
        address: Address,
        caller: Address,
        data: Vec<u8>,
        gas: u64,
    ) -> Self {
        Self {
            address,
            caller,
            value: 0,
            data,
            gas,
            is_static: true,
            depth: 0,
        }
    }
}

impl Default for CallContext {
    fn default() -> Self {
        Self {
            address: Address::ZERO,
            caller: Address::ZERO,
            value: 0,
            data: Vec::new(),
            gas: 0,
            is_static: false,
            depth: 0,
        }
    }
}

/// Block environment information
#[derive(Clone, Debug)]
pub struct BlockContext {
    /// Block number
    pub number: u64,
    /// Block timestamp
    pub timestamp: u64,
    /// Block gas limit
    pub gas_limit: u64,
    /// Block coinbase (miner/validator)
    pub coinbase: Address,
    /// Block difficulty/prevrandao
    pub prevrandao: H256,
    /// Chain ID
    pub chain_id: u64,
    /// Base fee (EIP-1559)
    pub base_fee: u128,
}

impl Default for BlockContext {
    fn default() -> Self {
        Self {
            number: 0,
            timestamp: 0,
            gas_limit: 30_000_000,
            coinbase: Address::ZERO,
            prevrandao: H256::ZERO,
            chain_id: 1,
            base_fee: 0,
        }
    }
}

/// Transaction environment information
#[derive(Clone, Debug)]
pub struct TxContext {
    /// Transaction origin (original sender)
    pub origin: Address,
    /// Gas price
    pub gas_price: u128,
}

impl Default for TxContext {
    fn default() -> Self {
        Self {
            origin: Address::ZERO,
            gas_price: 0,
        }
    }
}

/// Complete execution environment
#[derive(Clone, Debug, Default)]
pub struct Environment {
    /// Call context
    pub call: CallContext,
    /// Block context
    pub block: BlockContext,
    /// Transaction context
    pub tx: TxContext,
}

impl Environment {
    /// Create new environment
    pub fn new(call: CallContext, block: BlockContext, tx: TxContext) -> Self {
        Self { call, block, tx }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_context_new() {
        let address = Address::from_bytes([0x11; 20]);
        let caller = Address::from_bytes([0x22; 20]);
        let ctx = CallContext::new(
            address,
            caller,
            1000,
            vec![1, 2, 3],
            100000,
        );

        assert_eq!(ctx.address, address);
        assert_eq!(ctx.caller, caller);
        assert_eq!(ctx.value, 1000);
        assert_eq!(ctx.data, vec![1, 2, 3]);
        assert_eq!(ctx.gas, 100000);
        assert!(!ctx.is_static);
        assert_eq!(ctx.depth, 0);
    }

    #[test]
    fn test_call_context_new_static() {
        let address = Address::from_bytes([0x11; 20]);
        let caller = Address::from_bytes([0x22; 20]);
        let ctx = CallContext::new_static(
            address,
            caller,
            vec![1, 2, 3],
            100000,
        );

        assert_eq!(ctx.address, address);
        assert_eq!(ctx.caller, caller);
        assert_eq!(ctx.value, 0); // Static calls have no value
        assert_eq!(ctx.data, vec![1, 2, 3]);
        assert_eq!(ctx.gas, 100000);
        assert!(ctx.is_static);
        assert_eq!(ctx.depth, 0);
    }

    #[test]
    fn test_call_context_default() {
        let ctx = CallContext::default();
        assert_eq!(ctx.address, Address::ZERO);
        assert_eq!(ctx.caller, Address::ZERO);
        assert_eq!(ctx.value, 0);
        assert!(ctx.data.is_empty());
        assert_eq!(ctx.gas, 0);
        assert!(!ctx.is_static);
        assert_eq!(ctx.depth, 0);
    }

    #[test]
    fn test_call_context_clone() {
        let ctx = CallContext::new(
            Address::from_bytes([0x11; 20]),
            Address::from_bytes([0x22; 20]),
            1000,
            vec![1, 2, 3],
            100000,
        );
        let cloned = ctx.clone();

        assert_eq!(ctx.address, cloned.address);
        assert_eq!(ctx.caller, cloned.caller);
        assert_eq!(ctx.value, cloned.value);
        assert_eq!(ctx.data, cloned.data);
    }

    #[test]
    fn test_block_context_default() {
        let ctx = BlockContext::default();
        assert_eq!(ctx.number, 0);
        assert_eq!(ctx.timestamp, 0);
        assert_eq!(ctx.gas_limit, 30_000_000);
        assert_eq!(ctx.coinbase, Address::ZERO);
        assert_eq!(ctx.prevrandao, H256::ZERO);
        assert_eq!(ctx.chain_id, 1);
        assert_eq!(ctx.base_fee, 0);
    }

    #[test]
    fn test_block_context_custom() {
        let coinbase = Address::from_bytes([0x42; 20]);
        let prevrandao = H256::from_bytes([0xAB; 32]);
        let ctx = BlockContext {
            number: 12345,
            timestamp: 1700000000,
            gas_limit: 15_000_000,
            coinbase,
            prevrandao,
            chain_id: 137,
            base_fee: 1_000_000_000,
        };

        assert_eq!(ctx.number, 12345);
        assert_eq!(ctx.timestamp, 1700000000);
        assert_eq!(ctx.gas_limit, 15_000_000);
        assert_eq!(ctx.coinbase, coinbase);
        assert_eq!(ctx.prevrandao, prevrandao);
        assert_eq!(ctx.chain_id, 137);
        assert_eq!(ctx.base_fee, 1_000_000_000);
    }

    #[test]
    fn test_block_context_clone() {
        let ctx = BlockContext {
            number: 100,
            timestamp: 1234567890,
            gas_limit: 1000000,
            coinbase: Address::from_bytes([0x11; 20]),
            prevrandao: H256::from_bytes([0x22; 32]),
            chain_id: 1,
            base_fee: 1000,
        };
        let cloned = ctx.clone();

        assert_eq!(ctx.number, cloned.number);
        assert_eq!(ctx.timestamp, cloned.timestamp);
        assert_eq!(ctx.coinbase, cloned.coinbase);
    }

    #[test]
    fn test_tx_context_default() {
        let ctx = TxContext::default();
        assert_eq!(ctx.origin, Address::ZERO);
        assert_eq!(ctx.gas_price, 0);
    }

    #[test]
    fn test_tx_context_custom() {
        let ctx = TxContext {
            origin: Address::from_bytes([0x33; 20]),
            gas_price: 20_000_000_000,
        };

        assert_eq!(ctx.origin, Address::from_bytes([0x33; 20]));
        assert_eq!(ctx.gas_price, 20_000_000_000);
    }

    #[test]
    fn test_tx_context_clone() {
        let ctx = TxContext {
            origin: Address::from_bytes([0x44; 20]),
            gas_price: 10000,
        };
        let cloned = ctx.clone();

        assert_eq!(ctx.origin, cloned.origin);
        assert_eq!(ctx.gas_price, cloned.gas_price);
    }

    #[test]
    fn test_environment_new() {
        let call = CallContext::default();
        let block = BlockContext::default();
        let tx = TxContext::default();

        let env = Environment::new(call.clone(), block.clone(), tx.clone());

        assert_eq!(env.call.address, Address::ZERO);
        assert_eq!(env.block.chain_id, 1);
        assert_eq!(env.tx.gas_price, 0);
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert_eq!(env.call.address, Address::ZERO);
        assert_eq!(env.block.gas_limit, 30_000_000);
        assert_eq!(env.tx.origin, Address::ZERO);
    }

    #[test]
    fn test_environment_clone() {
        let env = Environment {
            call: CallContext::new(
                Address::from_bytes([0x11; 20]),
                Address::from_bytes([0x22; 20]),
                1000,
                vec![1, 2],
                50000,
            ),
            block: BlockContext {
                number: 100,
                ..Default::default()
            },
            tx: TxContext {
                origin: Address::from_bytes([0x33; 20]),
                gas_price: 10000,
            },
        };
        let cloned = env.clone();

        assert_eq!(env.call.address, cloned.call.address);
        assert_eq!(env.block.number, cloned.block.number);
        assert_eq!(env.tx.origin, cloned.tx.origin);
    }

    #[test]
    fn test_environment_debug() {
        let env = Environment::default();
        let debug_str = format!("{:?}", env);
        assert!(debug_str.contains("Environment"));
    }

    #[test]
    fn test_call_context_debug() {
        let ctx = CallContext::default();
        let debug_str = format!("{:?}", ctx);
        assert!(debug_str.contains("CallContext"));
    }
}
