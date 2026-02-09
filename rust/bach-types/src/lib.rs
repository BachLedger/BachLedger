//! BachLedger Types
//!
//! Core types for blockchain operations:
//! - `PriorityCode`: Transaction priority for Seamless Scheduling
//! - `ReadWriteSet`: Records storage accesses during execution
//! - `Transaction`: Blockchain transaction with signature
//! - `Block`: Block containing transactions

use bach_primitives::{Address, H256, U256};
use bach_crypto::{keccak256, keccak256_concat, Signature};
use std::collections::HashSet;

/// Ownership status: transaction owns the key
pub const PRIORITY_OWNED: u8 = 0;

/// Ownership status: ownership released
pub const PRIORITY_DISOWNED: u8 = 1;

/// Errors from type operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    /// Signature verification failed
    InvalidSignature,
    /// Could not recover sender address
    RecoveryFailed,
    /// Invalid transaction format
    InvalidTransaction(String),
}

/// Priority code for transaction ordering in Seamless Scheduling.
///
/// Structure: [release_bit (1 byte)] [block_height (8 bytes)] [hash (32 bytes)]
///
/// # Ordering
/// Lower value = Higher priority
/// - Owned (0) > Disowned (1) in priority (but 0 < 1 numerically)
/// - Lower block height > Higher block height in priority
/// - Lower hash > Higher hash in priority
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityCode {
    release_bit: u8,
    block_height: u64,
    hash: H256,
}

impl PriorityCode {
    /// Creates a new priority code with OWNED status.
    pub fn new(block_height: u64, hash: H256) -> Self {
        Self {
            release_bit: PRIORITY_OWNED,
            block_height,
            hash,
        }
    }

    /// Sets the release bit to DISOWNED.
    pub fn release(&mut self) {
        self.release_bit = PRIORITY_DISOWNED;
    }

    /// Returns true if ownership has been released.
    pub fn is_released(&self) -> bool {
        self.release_bit == PRIORITY_DISOWNED
    }

    /// Returns the block height.
    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    /// Returns the hash component.
    pub fn hash(&self) -> &H256 {
        &self.hash
    }

    /// Serializes to bytes (41 bytes).
    /// Format: [release_bit (1)] [block_height BE (8)] [hash (32)]
    pub fn to_bytes(&self) -> [u8; 41] {
        let mut bytes = [0u8; 41];
        bytes[0] = self.release_bit;
        bytes[1..9].copy_from_slice(&self.block_height.to_be_bytes());
        bytes[9..41].copy_from_slice(self.hash.as_bytes());
        bytes
    }

    /// Deserializes from bytes.
    pub fn from_bytes(bytes: &[u8; 41]) -> Self {
        let release_bit = bytes[0];
        let block_height = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&bytes[9..41]);
        let hash = H256::from(hash_bytes);

        Self {
            release_bit,
            block_height,
            hash,
        }
    }
}

impl Ord for PriorityCode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Lower value = Higher priority
        // Compare release_bit first (0 < 1, so owned sorts before disowned)
        match self.release_bit.cmp(&other.release_bit) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Compare block_height (lower is higher priority)
        match self.block_height.cmp(&other.block_height) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Compare hash (lower is higher priority)
        self.hash.as_bytes().cmp(other.hash.as_bytes())
    }
}

impl PartialOrd for PriorityCode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Records the keys read and written during transaction execution.
#[derive(Debug, Clone, Default)]
pub struct ReadWriteSet {
    reads: Vec<H256>,
    writes: Vec<(H256, Vec<u8>)>,
}

impl ReadWriteSet {
    /// Creates an empty read-write set.
    pub fn new() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    /// Records a read access to a key.
    pub fn record_read(&mut self, key: H256) {
        self.reads.push(key);
    }

    /// Records a write access to a key with its new value.
    pub fn record_write(&mut self, key: H256, value: Vec<u8>) {
        self.writes.push((key, value));
    }

    /// Returns all read keys.
    pub fn reads(&self) -> &[H256] {
        &self.reads
    }

    /// Returns all write key-value pairs.
    pub fn writes(&self) -> &[(H256, Vec<u8>)] {
        &self.writes
    }

    /// Returns all unique keys (reads + writes).
    pub fn all_keys(&self) -> Vec<H256> {
        let mut seen = HashSet::new();
        let mut keys = Vec::new();

        for key in &self.reads {
            if seen.insert(*key) {
                keys.push(*key);
            }
        }

        for (key, _) in &self.writes {
            if seen.insert(*key) {
                keys.push(*key);
            }
        }

        keys
    }

    /// Clears all recorded accesses.
    pub fn clear(&mut self) {
        self.reads.clear();
        self.writes.clear();
    }
}

/// A blockchain transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Sender's transaction count
    pub nonce: u64,
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,
    /// Transfer value
    pub value: U256,
    /// Call data
    pub data: Vec<u8>,
    /// ECDSA signature
    pub signature: Signature,
}

impl Transaction {
    /// Creates a new transaction.
    pub fn new(
        nonce: u64,
        to: Option<Address>,
        value: U256,
        data: Vec<u8>,
        signature: Signature,
    ) -> Self {
        Self {
            nonce,
            to,
            value,
            data,
            signature,
        }
    }

    /// Computes the transaction hash.
    /// Hash includes all fields including signature.
    pub fn hash(&self) -> H256 {
        let mut data = Vec::new();
        data.extend_from_slice(&self.nonce.to_be_bytes());
        if let Some(addr) = &self.to {
            data.push(1); // marker for Some
            data.extend_from_slice(addr.as_bytes());
        } else {
            data.push(0); // marker for None
        }
        data.extend_from_slice(&self.value.to_be_bytes());
        data.extend_from_slice(&self.data);
        data.extend_from_slice(&self.signature.to_bytes());
        keccak256(&data)
    }

    /// Recovers the sender address from the signature.
    pub fn sender(&self) -> Result<Address, TypeError> {
        let signing_hash = self.signing_hash();
        let pubkey = self.signature.recover(&signing_hash)
            .map_err(|_| TypeError::RecoveryFailed)?;
        Ok(pubkey.to_address())
    }

    /// Returns the signing hash (hash used for signature).
    /// This is the hash of the transaction data WITHOUT the signature.
    pub fn signing_hash(&self) -> H256 {
        let mut data = Vec::new();
        data.extend_from_slice(&self.nonce.to_be_bytes());
        if let Some(addr) = &self.to {
            data.extend_from_slice(addr.as_bytes());
        }
        data.extend_from_slice(&self.value.to_be_bytes());
        data.extend_from_slice(&self.data);
        keccak256(&data)
    }
}

/// A block containing transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Block number
    pub height: u64,
    /// Hash of the parent block
    pub parent_hash: H256,
    /// Ordered list of transactions
    pub transactions: Vec<Transaction>,
    /// Block timestamp (Unix seconds)
    pub timestamp: u64,
}

impl Block {
    /// Creates a new block.
    pub fn new(
        height: u64,
        parent_hash: H256,
        transactions: Vec<Transaction>,
        timestamp: u64,
    ) -> Self {
        Self {
            height,
            parent_hash,
            transactions,
            timestamp,
        }
    }

    /// Computes the block hash.
    /// Hash includes height, parent_hash, transactions_hash, and timestamp.
    pub fn hash(&self) -> H256 {
        let tx_hash = self.transactions_hash();
        keccak256_concat(&[
            &self.height.to_be_bytes(),
            self.parent_hash.as_bytes(),
            tx_hash.as_bytes(),
            &self.timestamp.to_be_bytes(),
        ])
    }

    /// Computes the hash of all transaction hashes.
    pub fn transactions_hash(&self) -> H256 {
        if self.transactions.is_empty() {
            // Hash of empty data
            return keccak256(&[]);
        }

        // Concatenate all transaction hashes and hash the result
        let mut tx_hashes = Vec::with_capacity(self.transactions.len() * 32);
        for tx in &self.transactions {
            tx_hashes.extend_from_slice(tx.hash().as_bytes());
        }
        keccak256(&tx_hashes)
    }

    /// Returns the number of transactions.
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }
}
