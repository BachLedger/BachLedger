//! BachLedger Types
//!
//! Core types for blockchain operations:
//! - `PriorityCode`: Transaction priority for Seamless Scheduling
//! - `ReadWriteSet`: Records storage accesses during execution
//! - `Transaction`: Blockchain transaction with signature
//! - `Block`: Block containing transactions

use bach_primitives::{Address, H256, U256};
use bach_crypto::Signature;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityCode {
    release_bit: u8,
    block_height: u64,
    hash: H256,
}

impl PriorityCode {
    /// Creates a new priority code with OWNED status.
    pub fn new(_block_height: u64, _hash: H256) -> Self {
        todo!("Implementation needed")
    }

    /// Sets the release bit to DISOWNED.
    pub fn release(&mut self) {
        todo!("Implementation needed")
    }

    /// Returns true if ownership has been released.
    pub fn is_released(&self) -> bool {
        todo!("Implementation needed")
    }

    /// Returns the block height.
    pub fn block_height(&self) -> u64 {
        todo!("Implementation needed")
    }

    /// Returns the hash component.
    pub fn hash(&self) -> &H256 {
        todo!("Implementation needed")
    }

    /// Serializes to bytes (41 bytes).
    pub fn to_bytes(&self) -> [u8; 41] {
        todo!("Implementation needed")
    }

    /// Deserializes from bytes.
    pub fn from_bytes(_bytes: &[u8; 41]) -> Self {
        todo!("Implementation needed")
    }
}

impl Ord for PriorityCode {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        todo!("Implementation needed")
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
        todo!("Implementation needed")
    }

    /// Records a read access to a key.
    pub fn record_read(&mut self, _key: H256) {
        todo!("Implementation needed")
    }

    /// Records a write access to a key with its new value.
    pub fn record_write(&mut self, _key: H256, _value: Vec<u8>) {
        todo!("Implementation needed")
    }

    /// Returns all read keys.
    pub fn reads(&self) -> &[H256] {
        todo!("Implementation needed")
    }

    /// Returns all write key-value pairs.
    pub fn writes(&self) -> &[(H256, Vec<u8>)] {
        todo!("Implementation needed")
    }

    /// Returns all unique keys (reads + writes).
    pub fn all_keys(&self) -> Vec<H256> {
        todo!("Implementation needed")
    }

    /// Clears all recorded accesses.
    pub fn clear(&mut self) {
        todo!("Implementation needed")
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
        _nonce: u64,
        _to: Option<Address>,
        _value: U256,
        _data: Vec<u8>,
        _signature: Signature,
    ) -> Self {
        todo!("Implementation needed")
    }

    /// Computes the transaction hash.
    pub fn hash(&self) -> H256 {
        todo!("Implementation needed")
    }

    /// Recovers the sender address from the signature.
    pub fn sender(&self) -> Result<Address, TypeError> {
        todo!("Implementation needed")
    }

    /// Returns the signing hash (hash used for signature).
    pub fn signing_hash(&self) -> H256 {
        todo!("Implementation needed")
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
        _height: u64,
        _parent_hash: H256,
        _transactions: Vec<Transaction>,
        _timestamp: u64,
    ) -> Self {
        todo!("Implementation needed")
    }

    /// Computes the block hash.
    pub fn hash(&self) -> H256 {
        todo!("Implementation needed")
    }

    /// Computes the hash of all transaction hashes.
    pub fn transactions_hash(&self) -> H256 {
        todo!("Implementation needed")
    }

    /// Returns the number of transactions.
    pub fn transaction_count(&self) -> usize {
        todo!("Implementation needed")
    }
}
