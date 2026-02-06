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

    #[test]
    fn test_bloom_empty() {
        let bloom = Bloom::default();
        assert!(bloom.is_empty());
    }

    #[test]
    fn test_bloom_accrue_and_contains() {
        let mut bloom = Bloom::default();
        bloom.accrue(b"hello");
        assert!(!bloom.is_empty());
        assert!(bloom.contains(b"hello"));
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
    fn test_genesis_header() {
        let header = BlockHeader::genesis(1);
        assert!(header.is_genesis());
        assert_eq!(header.number, 0);
        assert_eq!(header.parent_hash, H256::ZERO);
    }

    #[test]
    fn test_block_creation() {
        let header = BlockHeader::genesis(1);
        let block = Block::new(header, vec![]);
        assert_eq!(block.number(), 0);
        assert_eq!(block.tx_count(), 0);
    }
}
