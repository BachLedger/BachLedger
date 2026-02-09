# BachLedger API Reference

This document provides the API reference for all 5 core BachLedger modules.

## Table of Contents

1. [bach-primitives](#bach-primitives)
2. [bach-crypto](#bach-crypto)
3. [bach-types](#bach-types)
4. [bach-state](#bach-state)
5. [bach-scheduler](#bach-scheduler)

---

## bach-primitives

Basic types for blockchain operations.

### Constants

```rust
pub const ADDRESS_LENGTH: usize = 20;
pub const HASH_LENGTH: usize = 32;
```

### PrimitiveError

```rust
pub enum PrimitiveError {
    InvalidLength { expected: usize, actual: usize },
    InvalidHex(String),
}
```

### Address

20-byte Ethereum-compatible address.

```rust
impl Address {
    /// Creates from byte slice (must be exactly 20 bytes)
    pub fn from_slice(slice: &[u8]) -> Result<Self, PrimitiveError>;

    /// Parses from hex string (with or without "0x" prefix)
    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError>;

    /// Returns the zero address (all zeros)
    pub fn zero() -> Self;

    /// Returns reference to underlying bytes
    pub fn as_bytes(&self) -> &[u8; 20];

    /// Returns true if all bytes are zero
    pub fn is_zero(&self) -> bool;
}

// Traits: Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default
// Traits: Display, LowerHex, AsRef<[u8]>, From<[u8; 20]>
```

**Example**:
```rust
let addr = Address::from_hex("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")?;
println!("{}", addr);  // 0xd8da6bf26964af9d7eed9e03e53415d37aa96045
```

### H256

32-byte hash value.

```rust
impl H256 {
    pub fn from_slice(slice: &[u8]) -> Result<Self, PrimitiveError>;
    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError>;
    pub fn zero() -> Self;
    pub fn as_bytes(&self) -> &[u8; 32];
    pub fn is_zero(&self) -> bool;
}

// Same traits as Address
```

### H160

Type alias for Address.

```rust
pub type H160 = Address;
```

### U256

256-bit unsigned integer with little-endian limb storage.

```rust
impl U256 {
    pub const ZERO: Self;
    pub const ONE: Self;
    pub const MAX: Self;  // 2^256 - 1

    /// Creates from big-endian bytes
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self;

    /// Creates from little-endian bytes
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self;

    /// Converts to big-endian bytes
    pub fn to_be_bytes(&self) -> [u8; 32];

    /// Converts to little-endian bytes
    pub fn to_le_bytes(&self) -> [u8; 32];

    /// Creates from u64
    pub fn from_u64(val: u64) -> Self;

    /// Checked addition (None on overflow)
    pub fn checked_add(&self, other: &Self) -> Option<Self>;

    /// Checked subtraction (None on underflow)
    pub fn checked_sub(&self, other: &Self) -> Option<Self>;

    /// Checked multiplication (None on overflow)
    pub fn checked_mul(&self, other: &Self) -> Option<Self>;

    /// Checked division (None if divisor is zero)
    pub fn checked_div(&self, other: &Self) -> Option<Self>;

    /// Returns true if value is zero
    pub fn is_zero(&self) -> bool;
}

// Traits: Debug, Clone, Copy, PartialEq, Eq, Hash, Default
// Traits: PartialOrd, Ord, Display, LowerHex, From<u64>, From<u128>
```

**Example**:
```rust
let a = U256::from_u64(1000);
let b = U256::from_u64(500);
let sum = a.checked_add(&b).unwrap();  // 1500
let product = a.checked_mul(&b).unwrap();  // 500000
```

---

## bach-crypto

Cryptographic primitives for blockchain operations.

### Constants

```rust
pub const SIGNATURE_LENGTH: usize = 65;  // r(32) + s(32) + v(1)
```

### CryptoError

```rust
pub enum CryptoError {
    InvalidPrivateKey,
    InvalidSignature,
    RecoveryFailed,
    InvalidPublicKey,
}
```

### Hash Functions

```rust
/// Computes Keccak-256 hash
pub fn keccak256(data: &[u8]) -> H256;

/// Computes Keccak-256 hash of concatenated inputs
pub fn keccak256_concat(data: &[&[u8]]) -> H256;
```

**Example**:
```rust
let hash = keccak256(b"hello world");
// 0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad

let combined = keccak256_concat(&[b"hello", b" ", b"world"]);
assert_eq!(hash, combined);
```

### PrivateKey

secp256k1 private key (32 bytes).

```rust
impl PrivateKey {
    /// Generates random key using OS entropy
    pub fn random() -> Self;

    /// Creates from raw bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, CryptoError>;

    /// Returns raw bytes
    pub fn to_bytes(&self) -> [u8; 32];

    /// Derives corresponding public key
    pub fn public_key(&self) -> PublicKey;

    /// Signs a message hash (H256)
    pub fn sign(&self, message: &H256) -> Signature;
}

// Debug: Redacts key bytes for security
```

**Example**:
```rust
let key = PrivateKey::random();
let message = keccak256(b"transaction data");
let sig = key.sign(&message);
```

### PublicKey

secp256k1 public key (64 bytes, uncompressed, without 0x04 prefix).

```rust
impl PublicKey {
    /// Creates from uncompressed bytes (validates point is on curve)
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self, CryptoError>;

    /// Returns uncompressed bytes
    pub fn to_bytes(&self) -> [u8; 64];

    /// Derives Ethereum-style address: keccak256(pubkey)[12..32]
    pub fn to_address(&self) -> Address;

    /// Verifies signature against this public key
    pub fn verify(&self, signature: &Signature, message: &H256) -> bool;
}

// Traits: Debug, Clone, PartialEq, Eq
```

**Example**:
```rust
let pubkey = private_key.public_key();
let address = pubkey.to_address();
let valid = pubkey.verify(&signature, &message);
```

### Signature

ECDSA signature with recovery ID (65 bytes: r + s + v).

```rust
impl Signature {
    /// Creates from raw bytes (validates r, s, v)
    pub fn from_bytes(bytes: &[u8; 65]) -> Result<Self, CryptoError>;

    /// Returns raw bytes
    pub fn to_bytes(&self) -> [u8; 65];

    /// Verifies against public key and message
    pub fn verify(&self, pubkey: &PublicKey, message: &H256) -> bool;

    /// Recovers public key from signature and message
    pub fn recover(&self, message: &H256) -> Result<PublicKey, CryptoError>;

    /// Returns r component (32 bytes)
    pub fn r(&self) -> &[u8; 32];

    /// Returns s component (32 bytes)
    pub fn s(&self) -> &[u8; 32];

    /// Returns v value (27 or 28 for Ethereum)
    pub fn v(&self) -> u8;
}

// Traits: Debug, Clone, PartialEq, Eq
```

**Example**:
```rust
let recovered_pubkey = signature.recover(&message)?;
assert_eq!(recovered_pubkey, original_pubkey);
```

---

## bach-types

Core blockchain types for transactions and scheduling.

### Constants

```rust
pub const PRIORITY_OWNED: u8 = 0;     // Transaction owns the key
pub const PRIORITY_DISOWNED: u8 = 1;  // Ownership released
```

### TypeError

```rust
pub enum TypeError {
    InvalidSignature,
    RecoveryFailed,
    InvalidTransaction(String),
}
```

### PriorityCode

Transaction priority for Seamless Scheduling.

```rust
impl PriorityCode {
    /// Creates with OWNED status
    pub fn new(block_height: u64, hash: H256) -> Self;

    /// Sets release bit to DISOWNED
    pub fn release(&mut self);

    /// Returns true if ownership released
    pub fn is_released(&self) -> bool;

    /// Returns block height
    pub fn block_height(&self) -> u64;

    /// Returns hash component
    pub fn hash(&self) -> &H256;

    /// Serializes to 41 bytes
    pub fn to_bytes(&self) -> [u8; 41];

    /// Deserializes from 41 bytes
    pub fn from_bytes(bytes: &[u8; 41]) -> Self;
}

// Traits: Debug, Clone, PartialEq, Eq, Ord, PartialOrd
// Ordering: Lower value = Higher priority
```

**Byte Layout**:
```
[0]      release_bit (0=OWNED, 1=DISOWNED)
[1..9]   block_height (big-endian u64)
[9..41]  hash (32 bytes)
```

### ReadWriteSet

Records storage accesses during transaction execution.

```rust
impl ReadWriteSet {
    pub fn new() -> Self;

    /// Records a read access
    pub fn record_read(&mut self, key: H256);

    /// Records a write access with value
    pub fn record_write(&mut self, key: H256, value: Vec<u8>);

    /// Returns all read keys
    pub fn reads(&self) -> &[H256];

    /// Returns all write key-value pairs
    pub fn writes(&self) -> &[(H256, Vec<u8>)];

    /// Returns unique keys (reads + writes, deduplicated)
    pub fn all_keys(&self) -> Vec<H256>;

    /// Clears all recorded accesses
    pub fn clear(&mut self);
}

// Traits: Debug, Clone, Default
// Note: NOT thread-safe by design
```

### Transaction

Blockchain transaction with signature.

```rust
pub struct Transaction {
    pub nonce: u64,           // Sender's transaction count
    pub to: Option<Address>,  // None for contract creation
    pub value: U256,          // Transfer value
    pub data: Vec<u8>,        // Call data
    pub signature: Signature, // ECDSA signature
}

impl Transaction {
    pub fn new(nonce: u64, to: Option<Address>, value: U256,
               data: Vec<u8>, signature: Signature) -> Self;

    /// Hash includes all fields (including signature)
    pub fn hash(&self) -> H256;

    /// Recovers sender address from signature
    pub fn sender(&self) -> Result<Address, TypeError>;

    /// Hash used for signing (excludes signature)
    pub fn signing_hash(&self) -> H256;
}

// Traits: Debug, Clone, PartialEq, Eq
```

### Block

Block containing transactions.

```rust
pub struct Block {
    pub height: u64,
    pub parent_hash: H256,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,  // Unix seconds
}

impl Block {
    pub fn new(height: u64, parent_hash: H256,
               transactions: Vec<Transaction>, timestamp: u64) -> Self;

    /// Block hash (includes height, parent, txs_hash, timestamp)
    pub fn hash(&self) -> H256;

    /// Hash of all transaction hashes
    pub fn transactions_hash(&self) -> H256;

    /// Number of transactions
    pub fn transaction_count(&self) -> usize;
}

// Traits: Debug, Clone, PartialEq, Eq
```

---

## bach-state

State management and ownership tracking.

### StateError

```rust
pub enum StateError {
    KeyNotFound(H256),
    SnapshotExpired,
    LockError(String),
}
```

### StateDB Trait

Interface for state storage.

```rust
pub trait StateDB: Send + Sync {
    fn get(&self, key: &H256) -> Option<Vec<u8>>;
    fn set(&mut self, key: H256, value: Vec<u8>);
    fn delete(&mut self, key: &H256);
    fn snapshot(&self) -> Snapshot;
    fn commit(&mut self, writes: &[(H256, Vec<u8>)]);
    fn keys(&self) -> Vec<H256>;
}
```

### MemoryStateDB

In-memory implementation of StateDB.

```rust
impl MemoryStateDB {
    pub fn new() -> Self;
}

impl StateDB for MemoryStateDB { ... }

// Traits: Debug, Default
```

### Snapshot

Read-only state snapshot providing point-in-time isolation.

```rust
impl Snapshot {
    pub fn get(&self, key: &H256) -> Option<Vec<u8>>;
}

// Traits: Debug, Clone
// Thread-safe (immutable after creation)
```

### OwnershipEntry

Per-key ownership tracking (implements Algorithm 1).

```rust
impl OwnershipEntry {
    /// Creates with DISOWNED status
    pub fn new() -> Self;

    /// Releases ownership (sets DISOWNED)
    pub fn release_ownership(&self);

    /// Returns true if who <= current_owner (can claim)
    pub fn check_ownership(&self, who: &PriorityCode) -> bool;

    /// Attempts to claim ownership, returns true on success
    pub fn try_set_owner(&self, who: &PriorityCode) -> bool;

    /// Returns clone of current owner's priority code
    pub fn current_owner(&self) -> PriorityCode;
}

// Traits: Default, Clone
// Thread-safe (uses RwLock internally)
```

### OwnershipTable

Concurrent table mapping keys to ownership entries.

```rust
impl OwnershipTable {
    pub fn new() -> Self;

    /// Gets or creates entry for key
    pub fn get_or_create(&self, key: &H256) -> Arc<OwnershipEntry>;

    /// Releases ownership of all specified keys
    pub fn release_all(&self, keys: &[H256]);

    /// Clears all entries
    pub fn clear(&self);

    /// Number of entries
    pub fn len(&self) -> usize;

    /// Returns true if empty
    pub fn is_empty(&self) -> bool;
}

// Traits: Default
// Thread-safe (uses RwLock<HashMap>)
```

---

## bach-scheduler

Transaction scheduling with Seamless Scheduling algorithm.

### Constants

```rust
pub const DEFAULT_THREAD_COUNT: usize = 4;
pub const MAX_RETRIES: usize = 100;
```

### SchedulerError

```rust
pub enum SchedulerError {
    ExecutionFailed { tx_hash: H256, reason: String },
    MaxRetriesExceeded { tx_hash: H256, attempts: usize },
    InvalidBlock(String),
    StateError(StateError),
}
```

### ExecutionResult

```rust
pub enum ExecutionResult {
    Success { output: Vec<u8> },
    Failed { reason: String },
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool;
}
```

### ExecutedTransaction

```rust
pub struct ExecutedTransaction {
    pub transaction: Transaction,
    pub priority: PriorityCode,
    pub rwset: ReadWriteSet,
    pub result: ExecutionResult,
}

impl ExecutedTransaction {
    pub fn hash(&self) -> H256;
}
```

### ScheduleResult

```rust
pub struct ScheduleResult {
    pub confirmed: Vec<ExecutedTransaction>,
    pub block_hash: H256,
    pub state_root: H256,
    pub reexecution_count: usize,
}
```

### TransactionExecutor Trait

```rust
pub trait TransactionExecutor: Send + Sync {
    /// Executes transaction against snapshot, returns (rwset, result)
    fn execute(&self, tx: &Transaction, snapshot: &Snapshot)
        -> (ReadWriteSet, ExecutionResult);
}
```

### Scheduler Trait

```rust
pub trait Scheduler: Send + Sync {
    fn schedule(&self, block: Block, state: &mut dyn StateDB,
                executor: &dyn TransactionExecutor)
        -> Result<ScheduleResult, SchedulerError>;
}
```

### SeamlessScheduler

Implementation of Algorithm 2.

```rust
impl SeamlessScheduler {
    pub fn new(thread_count: usize) -> Self;
    pub fn with_default_threads() -> Self;
}

impl Default for SeamlessScheduler { ... }
impl Scheduler for SeamlessScheduler { ... }
```

**Example**:
```rust
let scheduler = SeamlessScheduler::with_default_threads();
let mut state = MemoryStateDB::new();
let executor = MyExecutor::new();

let result = scheduler.schedule(block, &mut state, &executor)?;
println!("Confirmed {} transactions", result.confirmed.len());
println!("Re-executions: {}", result.reexecution_count);
```

---

## Thread Safety Summary

| Type | Send | Sync | Notes |
|------|------|------|-------|
| Address, H256, U256 | Yes | Yes | Copy types |
| PrivateKey | Yes | Yes | Internal SigningKey |
| PublicKey, Signature | Yes | Yes | Byte arrays |
| PriorityCode | Yes | Yes | Plain data |
| ReadWriteSet | No | No | Use one per tx |
| Transaction, Block | Yes | Yes | Immutable after creation |
| MemoryStateDB | Yes | Yes | Requires &mut for writes |
| Snapshot | Yes | Yes | Immutable |
| OwnershipEntry | Yes | Yes | RwLock protected |
| OwnershipTable | Yes | Yes | RwLock protected |
| SeamlessScheduler | Yes | Yes | Stateless |
