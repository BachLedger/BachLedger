//! Transaction receipt types for BachLedger

use bach_primitives::{Address, H256};
use bytes::Bytes;

use crate::block::Bloom;

/// Transaction execution status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxStatus {
    /// Transaction failed
    Failure = 0,
    /// Transaction succeeded
    Success = 1,
}

impl From<bool> for TxStatus {
    fn from(success: bool) -> Self {
        if success {
            TxStatus::Success
        } else {
            TxStatus::Failure
        }
    }
}

impl From<TxStatus> for bool {
    fn from(status: TxStatus) -> Self {
        match status {
            TxStatus::Success => true,
            TxStatus::Failure => false,
        }
    }
}

/// Log entry emitted during transaction execution
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Log {
    /// Contract address that emitted the log
    pub address: Address,
    /// Log topics (indexed parameters)
    pub topics: Vec<H256>,
    /// Log data (non-indexed parameters)
    pub data: Bytes,
}

impl Log {
    /// Create a new log entry
    pub fn new(address: Address, topics: Vec<H256>, data: Bytes) -> Self {
        Self {
            address,
            topics,
            data,
        }
    }

    /// Get the first topic (usually the event signature)
    pub fn topic0(&self) -> Option<&H256> {
        self.topics.first()
    }

    /// Create bloom filter for this log
    pub fn bloom(&self) -> Bloom {
        let mut bloom = Bloom::default();
        bloom.accrue(self.address.as_bytes());
        for topic in &self.topics {
            bloom.accrue(topic.as_bytes());
        }
        bloom
    }
}

/// Transaction receipt
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Transaction status (success/failure)
    pub status: TxStatus,
    /// Cumulative gas used in the block up to this transaction
    pub cumulative_gas_used: u64,
    /// Gas used by this transaction
    pub gas_used: u64,
    /// Logs emitted by this transaction
    pub logs: Vec<Log>,
    /// Bloom filter for the logs
    pub logs_bloom: Bloom,
    /// Contract address created (if contract creation tx)
    pub contract_address: Option<Address>,
}

impl Receipt {
    /// Create a new receipt
    pub fn new(
        status: TxStatus,
        cumulative_gas_used: u64,
        gas_used: u64,
        logs: Vec<Log>,
    ) -> Self {
        let mut logs_bloom = Bloom::default();
        for log in &logs {
            logs_bloom.accrue_bloom(&log.bloom());
        }

        Self {
            status,
            cumulative_gas_used,
            gas_used,
            logs,
            logs_bloom,
            contract_address: None,
        }
    }

    /// Create a receipt with contract address
    pub fn with_contract_address(mut self, address: Address) -> Self {
        self.contract_address = Some(address);
        self
    }

    /// Check if transaction succeeded
    pub fn is_success(&self) -> bool {
        self.status == TxStatus::Success
    }

    /// Get number of logs
    pub fn log_count(&self) -> usize {
        self.logs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_status_conversion() {
        assert_eq!(TxStatus::from(true), TxStatus::Success);
        assert_eq!(TxStatus::from(false), TxStatus::Failure);
        assert!(bool::from(TxStatus::Success));
        assert!(!bool::from(TxStatus::Failure));
    }

    #[test]
    fn test_log_creation() {
        let addr = Address::from_bytes([0x42; 20]);
        let topic = H256::from_bytes([0x01; 32]);
        let data = Bytes::from(vec![0x02, 0x03]);

        let log = Log::new(addr, vec![topic], data.clone());

        assert_eq!(log.address, addr);
        assert_eq!(log.topic0(), Some(&topic));
        assert_eq!(log.data, data);
    }

    #[test]
    fn test_log_bloom() {
        let addr = Address::from_bytes([0x42; 20]);
        let topic = H256::from_bytes([0x01; 32]);
        let log = Log::new(addr, vec![topic], Bytes::new());

        let bloom = log.bloom();
        assert!(!bloom.is_empty());
        assert!(bloom.contains(addr.as_bytes()));
        assert!(bloom.contains(topic.as_bytes()));
    }

    #[test]
    fn test_receipt_creation() {
        let receipt = Receipt::new(TxStatus::Success, 100000, 21000, vec![]);

        assert!(receipt.is_success());
        assert_eq!(receipt.gas_used, 21000);
        assert_eq!(receipt.cumulative_gas_used, 100000);
        assert_eq!(receipt.log_count(), 0);
    }

    #[test]
    fn test_receipt_with_logs() {
        let log = Log::new(
            Address::from_bytes([0x42; 20]),
            vec![H256::from_bytes([0x01; 32])],
            Bytes::new(),
        );

        let receipt = Receipt::new(TxStatus::Success, 100000, 50000, vec![log]);

        assert_eq!(receipt.log_count(), 1);
        assert!(!receipt.logs_bloom.is_empty());
    }

    #[test]
    fn test_receipt_with_contract() {
        let contract = Address::from_bytes([0x99; 20]);
        let receipt = Receipt::new(TxStatus::Success, 100000, 100000, vec![])
            .with_contract_address(contract);

        assert_eq!(receipt.contract_address, Some(contract));
    }
}
