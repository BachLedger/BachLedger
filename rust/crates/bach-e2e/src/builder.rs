//! Transaction builder for declarative test construction
//!
//! Provides a fluent API for building transactions in tests.

use bach_primitives::Address;

/// Builder for constructing transactions declaratively
#[derive(Clone, Debug)]
pub struct TxBuilder {
    /// Sender account name
    pub from: String,
    /// Recipient address (None for contract creation)
    pub(crate) to: Option<Address>,
    /// Value to transfer (in wei)
    pub(crate) value: u128,
    /// Transaction data
    pub(crate) data: Vec<u8>,
    /// Gas limit (optional, uses default if not set)
    pub(crate) gas_limit: Option<u64>,
    /// Gas price (optional, uses default if not set)
    pub(crate) gas_price: Option<u128>,
    /// Explicit nonce (optional, auto-incremented if not set)
    pub(crate) nonce: Option<u64>,
}

impl TxBuilder {
    /// Create a simple value transfer
    pub fn transfer(from: &str, to: Address, value: u128) -> Self {
        Self {
            from: from.to_string(),
            to: Some(to),
            value,
            data: vec![],
            gas_limit: None,
            gas_price: None,
            nonce: None,
        }
    }

    /// Create a contract deployment
    pub fn deploy(from: &str, bytecode: Vec<u8>) -> Self {
        Self {
            from: from.to_string(),
            to: None,
            value: 0,
            data: bytecode,
            gas_limit: None,
            gas_price: None,
            nonce: None,
        }
    }

    /// Create a contract call
    pub fn call(from: &str, to: Address) -> Self {
        Self {
            from: from.to_string(),
            to: Some(to),
            value: 0,
            data: vec![],
            gas_limit: None,
            gas_price: None,
            nonce: None,
        }
    }

    /// Set the call data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Set the value to send
    pub fn value(mut self, value: u128) -> Self {
        self.value = value;
        self
    }

    /// Set the gas limit
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    /// Set the gas price
    pub fn gas_price(mut self, price: u128) -> Self {
        self.gas_price = Some(price);
        self
    }

    /// Set explicit nonce
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Check if this is a contract deployment
    pub fn is_deploy(&self) -> bool {
        self.to.is_none()
    }
}

/// Ether denomination helper trait
///
/// Allows writing amounts like `10.ether()` instead of `10_000_000_000_000_000_000`
pub trait EtherDenom {
    /// Convert to wei (base unit)
    fn wei(self) -> u128;
    /// Convert to gwei (10^9 wei)
    fn gwei(self) -> u128;
    /// Convert to ether (10^18 wei)
    fn ether(self) -> u128;
}

impl EtherDenom for u64 {
    fn wei(self) -> u128 {
        self as u128
    }

    fn gwei(self) -> u128 {
        (self as u128) * 1_000_000_000
    }

    fn ether(self) -> u128 {
        (self as u128) * 1_000_000_000_000_000_000
    }
}

impl EtherDenom for u128 {
    fn wei(self) -> u128 {
        self
    }

    fn gwei(self) -> u128 {
        self * 1_000_000_000
    }

    fn ether(self) -> u128 {
        self * 1_000_000_000_000_000_000
    }
}

impl EtherDenom for i32 {
    fn wei(self) -> u128 {
        self as u128
    }

    fn gwei(self) -> u128 {
        (self as u128) * 1_000_000_000
    }

    fn ether(self) -> u128 {
        (self as u128) * 1_000_000_000_000_000_000
    }
}

/// Execution result from a transaction
#[derive(Clone, Debug)]
pub struct ExecutionResult {
    /// Whether the transaction succeeded
    pub success: bool,
    /// Gas used by the transaction
    pub gas_used: u64,
    /// Contract address if this was a deployment
    pub contract_address: Option<Address>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ExecutionResult {
    /// Create a success result
    pub fn success(gas_used: u64, contract_address: Option<Address>) -> Self {
        Self {
            success: true,
            gas_used,
            contract_address,
            error: None,
        }
    }

    /// Create a failure result
    pub fn failure(gas_used: u64, error: String) -> Self {
        Self {
            success: false,
            gas_used,
            contract_address: None,
            error: Some(error),
        }
    }

    /// Create an error result (transaction didn't execute)
    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            gas_used: 0,
            contract_address: None,
            error: Some(msg),
        }
    }

    /// Calculate gas cost in wei (gas_used * gas_price)
    pub fn gas_cost(&self, gas_price: u128) -> u128 {
        (self.gas_used as u128) * gas_price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ether_denom() {
        assert_eq!(1u64.wei(), 1);
        assert_eq!(1u64.gwei(), 1_000_000_000);
        assert_eq!(1u64.ether(), 1_000_000_000_000_000_000);
        assert_eq!(10u64.ether(), 10_000_000_000_000_000_000);
    }

    #[test]
    fn test_tx_builder_transfer() {
        let to = Address::from_bytes([0x42; 20]);
        let builder = TxBuilder::transfer("alice", to, 1u64.ether());

        assert_eq!(builder.from, "alice");
        assert_eq!(builder.to, Some(to));
        assert_eq!(builder.value, 1u64.ether());
        assert!(builder.data.is_empty());
        assert!(!builder.is_deploy());
    }

    #[test]
    fn test_tx_builder_deploy() {
        let bytecode = vec![0x60, 0x80, 0x60, 0x40];
        let builder = TxBuilder::deploy("deployer", bytecode.clone());

        assert_eq!(builder.from, "deployer");
        assert!(builder.to.is_none());
        assert_eq!(builder.value, 0);
        assert_eq!(builder.data, bytecode);
        assert!(builder.is_deploy());
    }

    #[test]
    fn test_tx_builder_call() {
        let to = Address::from_bytes([0x42; 20]);
        let data = vec![0xa9, 0x05, 0x9c, 0xbb]; // transfer selector

        let builder = TxBuilder::call("caller", to)
            .data(data.clone())
            .value(100)
            .gas_limit(50000);

        assert_eq!(builder.from, "caller");
        assert_eq!(builder.to, Some(to));
        assert_eq!(builder.data, data);
        assert_eq!(builder.value, 100);
        assert_eq!(builder.gas_limit, Some(50000));
    }

    #[test]
    fn test_execution_result() {
        let success = ExecutionResult::success(21000, None);
        assert!(success.success);
        assert_eq!(success.gas_used, 21000);
        assert!(success.error.is_none());

        let failure = ExecutionResult::failure(21000, "out of gas".to_string());
        assert!(!failure.success);
        assert_eq!(failure.error.as_deref(), Some("out of gas"));
    }
}
