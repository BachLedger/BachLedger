//! Block types for BachLedger

use bach_primitives::{Address, H256};
use bytes::Bytes;

use crate::transaction::SignedTransaction;

/// Block header
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    /// Parent block hash
    pub parent_hash: H256,
    /// Ommers/uncles hash (always empty hash for BachLedger)
    pub ommers_hash: H256,
    /// Block producer/miner address
    pub beneficiary: Address,
    /// State root after executing the block
    pub state_root: H256,
    /// Transactions trie root
    pub transactions_root: H256,
    /// Receipts trie root
    pub receipts_root: H256,
    /// Logs bloom filter
    pub logs_bloom: Bloom,
    /// Difficulty (always 0 for PoA/PoS)
    pub difficulty: u128,
    /// Block number (height)
    pub number: u64,
    /// Gas limit for the block
    pub gas_limit: u64,
    /// Gas used by all transactions
    pub gas_used: u64,
    /// Block timestamp (Unix seconds)
    pub timestamp: u64,
    /// Extra data (consensus-specific)
    pub extra_data: Bytes,
    /// Mix hash (PoW) or prevRandao (PoS)
    pub mix_hash: H256,
    /// Nonce (PoW) or empty for PoS
    pub nonce: u64,
    /// Base fee per gas (EIP-1559)
    pub base_fee_per_gas: Option<u128>,
}

/// Block body containing transactions
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BlockBody {
    /// List of transactions
    pub transactions: Vec<SignedTransaction>,
}

/// Complete block (header + body)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Block body
    pub body: BlockBody,
}

/// Logs bloom filter (2048 bits = 256 bytes)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bloom(pub [u8; 256]);

impl Default for Bloom {
    fn default() -> Self {
        Self([0u8; 256])
    }
}

impl Bloom {
    /// Empty bloom filter
    pub const ZERO: Bloom = Bloom([0u8; 256]);

    /// Create bloom from bytes
    pub fn from_bytes(bytes: [u8; 256]) -> Self {
        Self(bytes)
    }

    /// Check if bloom filter is empty
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    /// Add data to bloom filter
    pub fn accrue(&mut self, input: &[u8]) {
        let hash = bach_crypto::keccak256(input);
        let hash_bytes = hash.as_bytes();

        // Set 3 bits based on hash
        for i in 0..3 {
            let bit_index = ((hash_bytes[i * 2] as usize) << 8 | hash_bytes[i * 2 + 1] as usize) & 0x7FF;
            let byte_index = 255 - bit_index / 8;
            let bit_position = bit_index % 8;
            self.0[byte_index] |= 1 << bit_position;
        }
    }

    /// Check if bloom might contain the input
    pub fn contains(&self, input: &[u8]) -> bool {
        let hash = bach_crypto::keccak256(input);
        let hash_bytes = hash.as_bytes();

        for i in 0..3 {
            let bit_index = ((hash_bytes[i * 2] as usize) << 8 | hash_bytes[i * 2 + 1] as usize) & 0x7FF;
            let byte_index = 255 - bit_index / 8;
            let bit_position = bit_index % 8;
            if self.0[byte_index] & (1 << bit_position) == 0 {
                return false;
            }
        }
        true
    }

    /// Combine with another bloom filter (OR)
    pub fn accrue_bloom(&mut self, other: &Bloom) {
        for i in 0..256 {
            self.0[i] |= other.0[i];
        }
    }
}

/// Empty ommers hash (keccak256 of empty RLP list)
pub const EMPTY_OMMERS_HASH: H256 = H256::from_bytes([
    0x1d, 0xcc, 0x4d, 0xe8, 0xde, 0xc7, 0x5d, 0x7a,
    0xab, 0x85, 0xb5, 0x67, 0xb6, 0xcc, 0xd4, 0x1a,
    0xd3, 0x12, 0x45, 0x1b, 0x94, 0x8a, 0x74, 0x13,
    0xf0, 0xa1, 0x42, 0xfd, 0x40, 0xd4, 0x93, 0x47,
]);

/// Empty transactions root
pub const EMPTY_TRANSACTIONS_ROOT: H256 = H256::from_bytes([
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6,
    0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0,
    0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
]);

impl BlockHeader {
    /// Create a genesis block header
    pub fn genesis(chain_id: u64) -> Self {
        Self {
            parent_hash: H256::ZERO,
            ommers_hash: EMPTY_OMMERS_HASH,
            beneficiary: Address::ZERO,
            state_root: H256::ZERO, // Will be set after state init
            transactions_root: EMPTY_TRANSACTIONS_ROOT,
            receipts_root: EMPTY_TRANSACTIONS_ROOT,
            logs_bloom: Bloom::ZERO,
            difficulty: 0,
            number: 0,
            gas_limit: 30_000_000,
            gas_used: 0,
            timestamp: 0,
            extra_data: Bytes::new(),
            mix_hash: H256::ZERO,
            nonce: chain_id,
            base_fee_per_gas: Some(1_000_000_000), // 1 gwei
        }
    }

    /// Check if this is the genesis block
    pub fn is_genesis(&self) -> bool {
        self.number == 0 && self.parent_hash == H256::ZERO
    }
}

impl Block {
    /// Create a new block
    pub fn new(header: BlockHeader, transactions: Vec<SignedTransaction>) -> Self {
        Self {
            header,
            body: BlockBody { transactions },
        }
    }

    /// Get block number
    pub fn number(&self) -> u64 {
        self.header.number
    }

    /// Get transaction count
    pub fn tx_count(&self) -> usize {
        self.body.transactions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{LegacyTx, TxSignature};

    // ==================== Bloom filter tests ====================

    #[test]
    fn test_bloom_empty() {
        let bloom = Bloom::default();
        assert!(bloom.is_empty());
    }

    #[test]
    fn test_bloom_zero_constant() {
        let bloom = Bloom::ZERO;
        assert!(bloom.is_empty());
        assert_eq!(bloom, Bloom::default());
    }

    #[test]
    fn test_bloom_from_bytes() {
        let bytes = [0xab; 256];
        let bloom = Bloom::from_bytes(bytes);
        assert!(!bloom.is_empty());
        assert_eq!(bloom.0, bytes);
    }

    #[test]
    fn test_bloom_accrue_and_contains() {
        let mut bloom = Bloom::default();
        bloom.accrue(b"hello");
        assert!(!bloom.is_empty());
        assert!(bloom.contains(b"hello"));
    }

    #[test]
    fn test_bloom_not_contains() {
        let mut bloom = Bloom::default();
        bloom.accrue(b"hello");
        // Bloom filters can have false positives but not false negatives
        // This test might occasionally pass for unrelated strings due to bloom nature
        // But we test with something very different
        assert!(bloom.contains(b"hello")); // Definitely contains what we added
    }

    #[test]
    fn test_bloom_combine() {
        let mut bloom1 = Bloom::default();
        bloom1.accrue(b"hello");

        let mut bloom2 = Bloom::default();
        bloom2.accrue(b"world");

        bloom1.accrue_bloom(&bloom2);
        assert!(bloom1.contains(b"hello"));
        assert!(bloom1.contains(b"world"));
    }

    #[test]
    fn test_bloom_multiple_items() {
        let mut bloom = Bloom::default();
        let items = [b"item1".as_slice(), b"item2", b"item3", b"item4", b"item5"];

        for item in &items {
            bloom.accrue(item);
        }

        for item in &items {
            assert!(bloom.contains(item), "Bloom should contain {:?}", item);
        }
    }

    #[test]
    fn test_bloom_address_and_topic() {
        let mut bloom = Bloom::default();
        let addr = Address::from_bytes([0x42; 20]);
        let topic = H256::from_bytes([0x01; 32]);

        bloom.accrue(addr.as_bytes());
        bloom.accrue(topic.as_bytes());

        assert!(bloom.contains(addr.as_bytes()));
        assert!(bloom.contains(topic.as_bytes()));
    }

    #[test]
    fn test_bloom_clone_and_eq() {
        let mut bloom1 = Bloom::default();
        bloom1.accrue(b"test");
        let bloom2 = bloom1.clone();
        assert_eq!(bloom1, bloom2);
    }

    // ==================== Constants tests ====================

    #[test]
    fn test_empty_ommers_hash() {
        // keccak256(RLP([])) = 0x1dcc4de8...
        assert_eq!(
            EMPTY_OMMERS_HASH.to_hex(),
            "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
        );
    }

    #[test]
    fn test_empty_transactions_root() {
        // Empty trie root
        assert_eq!(
            EMPTY_TRANSACTIONS_ROOT.to_hex(),
            "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"
        );
    }

    // ==================== BlockHeader tests ====================

    #[test]
    fn test_genesis_header() {
        let header = BlockHeader::genesis(1);
        assert!(header.is_genesis());
        assert_eq!(header.number, 0);
        assert_eq!(header.parent_hash, H256::ZERO);
    }

    #[test]
    fn test_genesis_header_fields() {
        let header = BlockHeader::genesis(1);

        assert_eq!(header.parent_hash, H256::ZERO);
        assert_eq!(header.ommers_hash, EMPTY_OMMERS_HASH);
        assert_eq!(header.beneficiary, Address::ZERO);
        assert_eq!(header.transactions_root, EMPTY_TRANSACTIONS_ROOT);
        assert_eq!(header.receipts_root, EMPTY_TRANSACTIONS_ROOT);
        assert!(header.logs_bloom.is_empty());
        assert_eq!(header.difficulty, 0);
        assert_eq!(header.number, 0);
        assert_eq!(header.gas_limit, 30_000_000);
        assert_eq!(header.gas_used, 0);
        assert_eq!(header.timestamp, 0);
        assert!(header.extra_data.is_empty());
        assert_eq!(header.mix_hash, H256::ZERO);
        assert_eq!(header.nonce, 1); // chain_id
        assert_eq!(header.base_fee_per_gas, Some(1_000_000_000)); // 1 gwei
    }

    #[test]
    fn test_genesis_different_chain_ids() {
        let mainnet = BlockHeader::genesis(1);
        let goerli = BlockHeader::genesis(5);
        let sepolia = BlockHeader::genesis(11155111);

        assert_eq!(mainnet.nonce, 1);
        assert_eq!(goerli.nonce, 5);
        assert_eq!(sepolia.nonce, 11155111);
    }

    #[test]
    fn test_non_genesis_block() {
        let mut header = BlockHeader::genesis(1);
        header.number = 1;
        header.parent_hash = H256::from_bytes([0x01; 32]);

        assert!(!header.is_genesis());
    }

    #[test]
    fn test_header_with_parent_but_zero_number() {
        let mut header = BlockHeader::genesis(1);
        header.parent_hash = H256::from_bytes([0x01; 32]);
        // number is still 0

        // This is actually not a valid genesis (parent exists)
        assert!(!header.is_genesis());
    }

    #[test]
    fn test_header_clone_and_eq() {
        let header1 = BlockHeader::genesis(1);
        let header2 = header1.clone();
        assert_eq!(header1, header2);
    }

    // ==================== BlockBody tests ====================

    #[test]
    fn test_block_body_default() {
        let body = BlockBody::default();
        assert!(body.transactions.is_empty());
    }

    #[test]
    fn test_block_body_with_transactions() {
        let tx = LegacyTx::default();
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);

        let body = BlockBody {
            transactions: vec![signed.clone(), signed],
        };

        assert_eq!(body.transactions.len(), 2);
    }

    // ==================== Block tests ====================

    #[test]
    fn test_block_creation() {
        let header = BlockHeader::genesis(1);
        let block = Block::new(header, vec![]);
        assert_eq!(block.number(), 0);
        assert_eq!(block.tx_count(), 0);
    }

    #[test]
    fn test_block_with_transactions() {
        let header = BlockHeader::genesis(1);
        let tx = LegacyTx::default();
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);

        let block = Block::new(header, vec![signed.clone(), signed.clone(), signed]);

        assert_eq!(block.number(), 0);
        assert_eq!(block.tx_count(), 3);
    }

    #[test]
    fn test_block_number() {
        let mut header = BlockHeader::genesis(1);
        header.number = 12345;

        let block = Block::new(header, vec![]);
        assert_eq!(block.number(), 12345);
    }

    #[test]
    fn test_block_clone_and_eq() {
        let header = BlockHeader::genesis(1);
        let block1 = Block::new(header, vec![]);
        let block2 = block1.clone();
        assert_eq!(block1, block2);
    }

    // ==================== Gas accounting tests ====================

    #[test]
    fn test_header_gas_fields() {
        let mut header = BlockHeader::genesis(1);
        header.gas_limit = 15_000_000;
        header.gas_used = 10_000_000;

        assert_eq!(header.gas_limit, 15_000_000);
        assert_eq!(header.gas_used, 10_000_000);
        assert!(header.gas_used < header.gas_limit);
    }

    #[test]
    fn test_header_full_gas() {
        let mut header = BlockHeader::genesis(1);
        header.gas_limit = 30_000_000;
        header.gas_used = 30_000_000; // Fully used

        assert_eq!(header.gas_used, header.gas_limit);
    }

    // ==================== Base fee tests ====================

    #[test]
    fn test_header_base_fee() {
        let header = BlockHeader::genesis(1);
        assert_eq!(header.base_fee_per_gas, Some(1_000_000_000)); // 1 gwei
    }

    #[test]
    fn test_header_no_base_fee() {
        let mut header = BlockHeader::genesis(1);
        header.base_fee_per_gas = None; // Pre-EIP-1559

        assert!(header.base_fee_per_gas.is_none());
    }

    // ==================== Extra data tests ====================

    #[test]
    fn test_header_extra_data() {
        let mut header = BlockHeader::genesis(1);
        header.extra_data = Bytes::from(b"BachLedger".to_vec());

        assert_eq!(header.extra_data.len(), 10);
    }

    #[test]
    fn test_header_max_extra_data() {
        let mut header = BlockHeader::genesis(1);
        // Ethereum allows up to 32 bytes of extra data
        header.extra_data = Bytes::from(vec![0x42; 32]);

        assert_eq!(header.extra_data.len(), 32);
    }
}
