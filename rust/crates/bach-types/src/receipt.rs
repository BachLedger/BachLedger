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

    // ==================== TxStatus tests ====================

    #[test]
    fn test_tx_status_conversion() {
        assert_eq!(TxStatus::from(true), TxStatus::Success);
        assert_eq!(TxStatus::from(false), TxStatus::Failure);
        assert!(bool::from(TxStatus::Success));
        assert!(!bool::from(TxStatus::Failure));
    }

    #[test]
    fn test_tx_status_values() {
        assert_eq!(TxStatus::Failure as u8, 0);
        assert_eq!(TxStatus::Success as u8, 1);
    }

    #[test]
    fn test_tx_status_clone_and_eq() {
        let status1 = TxStatus::Success;
        let status2 = status1;
        assert_eq!(status1, status2);
    }

    #[test]
    fn test_tx_status_debug() {
        let status = TxStatus::Success;
        let debug = format!("{:?}", status);
        assert!(debug.contains("Success"));
    }

    // ==================== Log tests ====================

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
    fn test_log_no_topics() {
        let log = Log::new(
            Address::from_bytes([0x42; 20]),
            vec![],
            Bytes::new(),
        );

        assert!(log.topics.is_empty());
        assert!(log.topic0().is_none());
    }

    #[test]
    fn test_log_multiple_topics() {
        let topic0 = H256::from_bytes([0x01; 32]); // Event signature
        let topic1 = H256::from_bytes([0x02; 32]); // Indexed param 1
        let topic2 = H256::from_bytes([0x03; 32]); // Indexed param 2

        let log = Log::new(
            Address::from_bytes([0x42; 20]),
            vec![topic0, topic1, topic2],
            Bytes::new(),
        );

        assert_eq!(log.topics.len(), 3);
        assert_eq!(log.topic0(), Some(&topic0));
        assert_eq!(log.topics[1], topic1);
        assert_eq!(log.topics[2], topic2);
    }

    #[test]
    fn test_log_max_topics() {
        // EVM allows up to 4 topics
        let topics: Vec<H256> = (0..4)
            .map(|i| H256::from_bytes([i as u8; 32]))
            .collect();

        let log = Log::new(
            Address::from_bytes([0x42; 20]),
            topics.clone(),
            Bytes::new(),
        );

        assert_eq!(log.topics.len(), 4);
    }

    #[test]
    fn test_log_with_data() {
        // Simulate ERC20 Transfer event data (value as uint256)
        let data = Bytes::from(vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // 100
        ]);

        let log = Log::new(
            Address::from_bytes([0x42; 20]),
            vec![H256::from_bytes([0x01; 32])],
            data.clone(),
        );

        assert_eq!(log.data.len(), 32);
        assert_eq!(log.data[31], 0x64); // 100
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
    fn test_log_bloom_multiple_topics() {
        let addr = Address::from_bytes([0x42; 20]);
        let topic1 = H256::from_bytes([0x01; 32]);
        let topic2 = H256::from_bytes([0x02; 32]);
        let log = Log::new(addr, vec![topic1, topic2], Bytes::new());

        let bloom = log.bloom();
        assert!(bloom.contains(addr.as_bytes()));
        assert!(bloom.contains(topic1.as_bytes()));
        assert!(bloom.contains(topic2.as_bytes()));
    }

    #[test]
    fn test_log_clone_and_eq() {
        let log1 = Log::new(
            Address::from_bytes([0x42; 20]),
            vec![H256::from_bytes([0x01; 32])],
            Bytes::from(vec![0x02]),
        );
        let log2 = log1.clone();
        assert_eq!(log1, log2);
    }

    // ==================== Receipt tests ====================

    #[test]
    fn test_receipt_creation() {
        let receipt = Receipt::new(TxStatus::Success, 100000, 21000, vec![]);

        assert!(receipt.is_success());
        assert_eq!(receipt.gas_used, 21000);
        assert_eq!(receipt.cumulative_gas_used, 100000);
        assert_eq!(receipt.log_count(), 0);
    }

    #[test]
    fn test_receipt_failure() {
        let receipt = Receipt::new(TxStatus::Failure, 100000, 21000, vec![]);

        assert!(!receipt.is_success());
        assert_eq!(receipt.status, TxStatus::Failure);
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
    fn test_receipt_bloom_aggregation() {
        let log1 = Log::new(
            Address::from_bytes([0x42; 20]),
            vec![H256::from_bytes([0x01; 32])],
            Bytes::new(),
        );
        let log2 = Log::new(
            Address::from_bytes([0x43; 20]),
            vec![H256::from_bytes([0x02; 32])],
            Bytes::new(),
        );

        let receipt = Receipt::new(TxStatus::Success, 100000, 50000, vec![log1.clone(), log2.clone()]);

        // Receipt bloom should contain all logs
        assert!(receipt.logs_bloom.contains(log1.address.as_bytes()));
        assert!(receipt.logs_bloom.contains(log2.address.as_bytes()));
    }

    #[test]
    fn test_receipt_with_contract() {
        let contract = Address::from_bytes([0x99; 20]);
        let receipt = Receipt::new(TxStatus::Success, 100000, 100000, vec![])
            .with_contract_address(contract);

        assert_eq!(receipt.contract_address, Some(contract));
    }

    #[test]
    fn test_receipt_no_contract() {
        let receipt = Receipt::new(TxStatus::Success, 100000, 21000, vec![]);
        assert!(receipt.contract_address.is_none());
    }

    #[test]
    fn test_receipt_gas_accounting() {
        // First tx in block: cumulative = gas_used
        let receipt1 = Receipt::new(TxStatus::Success, 21000, 21000, vec![]);
        assert_eq!(receipt1.cumulative_gas_used, receipt1.gas_used);

        // Second tx: cumulative > gas_used
        let receipt2 = Receipt::new(TxStatus::Success, 50000, 29000, vec![]);
        assert!(receipt2.cumulative_gas_used > receipt2.gas_used);
    }

    #[test]
    fn test_receipt_high_gas() {
        let receipt = Receipt::new(
            TxStatus::Success,
            30_000_000, // Block gas limit
            15_000_000, // Half the block
            vec![],
        );

        assert_eq!(receipt.gas_used, 15_000_000);
        assert_eq!(receipt.cumulative_gas_used, 30_000_000);
    }

    #[test]
    fn test_receipt_multiple_logs() {
        let logs: Vec<Log> = (0..10)
            .map(|i| {
                Log::new(
                    Address::from_bytes([i as u8; 20]),
                    vec![H256::from_bytes([i as u8; 32])],
                    Bytes::new(),
                )
            })
            .collect();

        let receipt = Receipt::new(TxStatus::Success, 100000, 50000, logs);

        assert_eq!(receipt.log_count(), 10);
        assert!(!receipt.logs_bloom.is_empty());
    }

    #[test]
    fn test_receipt_clone_and_eq() {
        let receipt1 = Receipt::new(TxStatus::Success, 100000, 21000, vec![]);
        let receipt2 = receipt1.clone();
        assert_eq!(receipt1, receipt2);
    }

    // ==================== ERC20 Transfer event simulation ====================

    #[test]
    fn test_erc20_transfer_log() {
        // ERC20 Transfer(address indexed from, address indexed to, uint256 value)
        // Topic0: keccak256("Transfer(address,address,uint256)")
        let transfer_topic = H256::from_hex(
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
        ).unwrap();

        // Indexed from address (padded to 32 bytes)
        let mut from_topic = [0u8; 32];
        from_topic[12..].copy_from_slice(&[0x11; 20]);

        // Indexed to address (padded to 32 bytes)
        let mut to_topic = [0u8; 32];
        to_topic[12..].copy_from_slice(&[0x22; 20]);

        // Value (non-indexed, in data field)
        let value_data = vec![0u8; 32]; // 0

        let log = Log::new(
            Address::from_bytes([0x42; 20]), // Token contract
            vec![transfer_topic, H256::from_bytes(from_topic), H256::from_bytes(to_topic)],
            Bytes::from(value_data),
        );

        assert_eq!(log.topics.len(), 3);
        assert_eq!(log.topic0(), Some(&transfer_topic));
    }
}
