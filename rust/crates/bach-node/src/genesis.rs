//! Genesis block handling for bach-node

use crate::config::{parse_address, GenesisConfig};
use bach_crypto::keccak256;
use bach_primitives::{Address, H256, U256};
use bach_storage::{Account, BlockDb, StateDb, StateWriter, EMPTY_CODE_HASH, EMPTY_STORAGE_ROOT};
use bach_types::codec;
use bach_types::{Block, BlockBody, BlockHeader, Bloom};
use bytes::Bytes;
use thiserror::Error;

/// Genesis error types
#[derive(Debug, Error)]
pub enum GenesisError {
    /// Genesis already initialized
    #[error("genesis already initialized")]
    AlreadyInitialized,
    /// Storage error
    #[error("storage error: {0}")]
    Storage(#[from] bach_storage::StorageError),
    /// Invalid genesis configuration
    #[error("invalid genesis config: {0}")]
    InvalidConfig(String),
}

/// Result type for genesis operations
pub type GenesisResult<T> = Result<T, GenesisError>;

/// Genesis block builder
pub struct GenesisBuilder {
    config: GenesisConfig,
    chain_id: u64,
}

impl GenesisBuilder {
    /// Create a new genesis builder
    pub fn new(config: GenesisConfig, chain_id: u64) -> Self {
        Self { config, chain_id }
    }

    /// Initialize genesis state in database
    pub fn init_genesis(
        &self,
        state_db: &mut StateDb,
        block_db: &BlockDb,
    ) -> GenesisResult<Block> {
        // Check if genesis already exists
        if block_db.get_latest_block()?.is_some() {
            return Err(GenesisError::AlreadyInitialized);
        }

        tracing::info!("Initializing genesis state...");

        // Apply initial allocations
        for (addr_str, account) in &self.config.alloc {
            let address = parse_address(addr_str).ok_or_else(|| {
                GenesisError::InvalidConfig(format!("invalid address: {}", addr_str))
            })?;

            let balance = account.parse_balance();
            let balance_u128 = if balance > U256::from(u128::MAX) {
                u128::MAX
            } else {
                balance.as_u128()
            };

            // Determine code hash
            let code_hash = if let Some(code) = account.parse_code() {
                if code.is_empty() {
                    EMPTY_CODE_HASH
                } else {
                    let hash = keccak256(&code);
                    state_db.set_code(hash, code.to_vec())?;
                    hash
                }
            } else {
                EMPTY_CODE_HASH
            };

            let state_account = Account {
                nonce: account.nonce,
                balance: balance_u128,
                code_hash,
                storage_root: EMPTY_STORAGE_ROOT,
            };

            state_db.set_account(address, state_account)?;

            // Set initial storage
            let storage = account.parse_storage();
            for (key, value) in storage {
                state_db.set_storage(address, key, value)?;
            }

            tracing::debug!(
                "Genesis allocation: {} balance={}, nonce={}",
                addr_str,
                balance,
                account.nonce
            );
        }

        // Build genesis block
        let extra_data = if self.config.extra_data.is_empty() {
            Bytes::new()
        } else {
            let data = self.config.extra_data.strip_prefix("0x").unwrap_or(&self.config.extra_data);
            hex::decode(data).map(Bytes::from).unwrap_or_default()
        };

        let genesis_block = Block {
            header: BlockHeader {
                parent_hash: H256::ZERO,
                ommers_hash: EMPTY_OMMERS_HASH,
                beneficiary: Address::ZERO,
                state_root: H256::ZERO, // Placeholder - needs merkle trie
                transactions_root: EMPTY_TX_ROOT,
                receipts_root: EMPTY_RECEIPTS_ROOT,
                logs_bloom: Bloom::default(),
                difficulty: self.config.difficulty as u128,
                number: 0,
                gas_limit: self.config.gas_limit,
                gas_used: 0,
                timestamp: self.config.timestamp,
                extra_data,
                mix_hash: H256::ZERO,
                nonce: 0,
                base_fee_per_gas: Some(1_000_000_000), // 1 gwei
            },
            body: BlockBody {
                transactions: vec![],
            },
        };

        // Compute block hash
        let hash = compute_block_hash(&genesis_block);

        // Store genesis block
        let header_bytes = encode_header(&genesis_block.header);
        let body_bytes = encode_body(&genesis_block.body);

        block_db.put_header(&hash, &header_bytes)?;
        block_db.put_body(&hash, &body_bytes)?;
        block_db.put_hash_by_number(0, &hash)?;
        block_db.set_latest_block(0)?;
        block_db.set_finalized_block(0)?;

        // Store chain_id in metadata for consistency validation on restart
        block_db.put_meta(b"chain_id", &self.chain_id.to_le_bytes())?;

        tracing::info!(
            "Genesis block initialized: hash={}, allocations={}",
            hash.to_hex(),
            self.config.alloc.len()
        );

        Ok(genesis_block)
    }
}

// Placeholder constants - these should match Ethereum standards
const EMPTY_OMMERS_HASH: H256 = H256::from_bytes([
    0x1d, 0xcc, 0x4d, 0xe8, 0xde, 0xc7, 0x5d, 0x7a, 0xab, 0x85, 0xb5, 0x67, 0xb6, 0xcc, 0xd4, 0x1a,
    0xd3, 0x12, 0x45, 0x1b, 0x94, 0x8a, 0x74, 0x13, 0xf0, 0xa1, 0x42, 0xfd, 0x40, 0xd4, 0x93, 0x47,
]);

const EMPTY_TX_ROOT: H256 = H256::from_bytes([
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
]);

const EMPTY_RECEIPTS_ROOT: H256 = H256::from_bytes([
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
]);

/// Compute block hash
pub fn compute_block_hash(block: &Block) -> H256 {
    keccak256(&codec::encode_header(&block.header))
}

// Re-export codec functions for use by node.rs
pub use codec::encode_header;
pub use codec::encode_body;
pub use codec::encode_receipts;
pub use codec::decode_header;
pub use codec::decode_body;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{default_genesis_config, GenesisAccount};
    use bach_storage::Database;
    use std::collections::HashMap;
    use std::fs;

    fn temp_db_path() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cnt = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/bach_genesis_test_{}_{}", id, cnt)
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_genesis_builder_init() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state_db = StateDb::new(db.clone());
        let block_db = BlockDb::new(db.clone());

        let genesis_config = default_genesis_config();
        let builder = GenesisBuilder::new(genesis_config, 1337);

        let block = builder.init_genesis(&mut state_db, &block_db).unwrap();

        assert_eq!(block.header.number, 0);
        assert_eq!(block.header.parent_hash, H256::ZERO);
        assert_eq!(block.header.gas_limit, 30_000_000);
        assert!(block.body.transactions.is_empty());

        // Verify block was stored
        assert_eq!(block_db.get_latest_block().unwrap(), Some(0));

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_genesis_already_initialized() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state_db = StateDb::new(db.clone());
        let block_db = BlockDb::new(db.clone());

        let genesis_config = default_genesis_config();
        let builder = GenesisBuilder::new(genesis_config.clone(), 1337);

        // First init should succeed
        builder.init_genesis(&mut state_db, &block_db).unwrap();

        // Second init should fail
        let result = builder.init_genesis(&mut state_db, &block_db);
        assert!(matches!(result, Err(GenesisError::AlreadyInitialized)));

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_genesis_with_allocations() {
        let path = temp_db_path();
        let db = Database::new(&path);
        db.open().unwrap();

        let mut state_db = StateDb::new(db.clone());
        let block_db = BlockDb::new(db.clone());

        let mut alloc = HashMap::new();
        alloc.insert(
            "0x1111111111111111111111111111111111111111".to_string(),
            GenesisAccount {
                balance: "0xde0b6b3a7640000".to_string(), // 1 ETH
                nonce: 5,
                code: None,
                storage: HashMap::new(),
            },
        );

        let genesis_config = GenesisConfig {
            alloc,
            timestamp: 1000,
            extra_data: String::new(),
            difficulty: 1,
            gas_limit: 30_000_000,
            chain_id: 1337,
        };

        let builder = GenesisBuilder::new(genesis_config, 1337);
        builder.init_genesis(&mut state_db, &block_db).unwrap();

        // Verify allocation was applied
        use bach_storage::StateReader;
        let addr = parse_address("0x1111111111111111111111111111111111111111").unwrap();
        let account = state_db.get_account(&addr).unwrap().unwrap();
        assert_eq!(account.nonce, 5);
        assert!(account.balance > 0);

        db.close();
        cleanup(&path);
    }

    #[test]
    fn test_compute_block_hash() {
        let block = Block {
            header: BlockHeader {
                parent_hash: H256::ZERO,
                ommers_hash: EMPTY_OMMERS_HASH,
                beneficiary: Address::ZERO,
                state_root: H256::ZERO,
                transactions_root: EMPTY_TX_ROOT,
                receipts_root: EMPTY_RECEIPTS_ROOT,
                logs_bloom: Bloom::default(),
                difficulty: 1,
                number: 0,
                gas_limit: 30_000_000,
                gas_used: 0,
                timestamp: 0,
                extra_data: Bytes::new(),
                mix_hash: H256::ZERO,
                nonce: 0,
                base_fee_per_gas: Some(1_000_000_000),
            },
            body: BlockBody {
                transactions: vec![],
            },
        };

        let hash = compute_block_hash(&block);
        assert_ne!(hash, H256::ZERO);
    }

    #[test]
    fn test_encode_decode_header() {
        let header = BlockHeader {
            parent_hash: H256::ZERO,
            ommers_hash: EMPTY_OMMERS_HASH,
            beneficiary: Address::ZERO,
            state_root: H256::ZERO,
            transactions_root: EMPTY_TX_ROOT,
            receipts_root: EMPTY_RECEIPTS_ROOT,
            logs_bloom: Bloom::default(),
            difficulty: 1,
            number: 100,
            gas_limit: 30_000_000,
            gas_used: 21000,
            timestamp: 1234567890,
            extra_data: Bytes::new(),
            mix_hash: H256::ZERO,
            nonce: 0,
            base_fee_per_gas: Some(1_000_000_000),
        };

        let encoded = encode_header(&header);
        let decoded = decode_header(&encoded).unwrap();

        assert_eq!(decoded.number, header.number);
        assert_eq!(decoded.gas_limit, header.gas_limit);
        assert_eq!(decoded.timestamp, header.timestamp);
    }
}
