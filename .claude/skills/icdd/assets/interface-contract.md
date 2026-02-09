# Interface Contract: BachLedger

## Contract Information

| Field | Value |
|-------|-------|
| Module Name | BachLedger |
| Contract Version | 1.0.0 |
| Author | ICDD Architect |
| Date | 2026-02-09 |
| Status | **LOCKED** |

---

## Version Locking Mechanism

### Contract Version Rules

- **Major Version (X.0.0)**: Breaking changes to interface signatures or behavior
- **Minor Version (0.X.0)**: Backward-compatible additions
- **Patch Version (0.0.X)**: Bug fixes with no interface changes

### Breaking Change Policy

1. This contract is LOCKED - no changes without formal review
2. Any interface modification requires new contract version
3. Tester and Coder agents MUST NOT modify these signatures

---

## 1. Module Overview

### 1.1 Module Dependency Graph

```
bach-primitives (no dependencies)
      │
      ▼
bach-crypto
      │
      ▼
bach-types ◄─────────── bach-state
      │                       │
      └───────────┬───────────┘
                  ▼
           bach-scheduler
```

### 1.2 Module Summary

| Module | Purpose | Dependencies |
|--------|---------|--------------|
| bach-primitives | Basic types: Address, H256, U256 | None |
| bach-crypto | Keccak256, ECDSA signatures | bach-primitives |
| bach-types | Transaction, Block, PriorityCode, RWSet | bach-primitives, bach-crypto |
| bach-state | StateDB, OwnershipTable, Snapshot | bach-primitives, bach-types |
| bach-scheduler | SeamlessScheduler algorithm | All above |

---

## 2. Module: bach-primitives

### 2.1 Constants

```rust
/// Length of an Ethereum-style address in bytes
pub const ADDRESS_LENGTH: usize = 20;

/// Length of a 256-bit hash in bytes
pub const HASH_LENGTH: usize = 32;
```

### 2.2 Error Types

```rust
/// Errors from primitive operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveError {
    /// Slice length does not match expected size
    InvalidLength { expected: usize, actual: usize },
    /// Invalid hexadecimal character in string
    InvalidHex(String),
}
```

### 2.3 Type: Address

```rust
/// A 20-byte Ethereum-compatible address.
///
/// # Invariants
/// - Always exactly 20 bytes
/// - Zero address is valid (all zeros)
///
/// # Thread Safety
/// Send + Sync (pure data)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Address([u8; ADDRESS_LENGTH]);

impl Address {
    /// Creates an Address from a byte slice.
    ///
    /// # Errors
    /// Returns `InvalidLength` if slice is not exactly 20 bytes.
    pub fn from_slice(slice: &[u8]) -> Result<Self, PrimitiveError>;

    /// Parses an Address from a hex string.
    ///
    /// # Arguments
    /// * `s` - Hex string, with or without "0x" prefix
    ///
    /// # Errors
    /// - `InvalidHex` if string contains non-hex characters
    /// - `InvalidLength` if decoded bytes != 20
    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError>;

    /// Returns the zero address (all zeros).
    pub fn zero() -> Self;

    /// Returns a reference to the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; ADDRESS_LENGTH];

    /// Checks if this is the zero address.
    pub fn is_zero(&self) -> bool;
}

impl AsRef<[u8]> for Address;
impl From<[u8; ADDRESS_LENGTH]> for Address;
impl std::fmt::Display for Address; // Outputs "0x..." lowercase hex
impl std::fmt::LowerHex for Address;
```

### 2.4 Type: H256

```rust
/// A 32-byte hash value.
///
/// # Invariants
/// - Always exactly 32 bytes
///
/// # Thread Safety
/// Send + Sync (pure data)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct H256([u8; HASH_LENGTH]);

impl H256 {
    /// Creates an H256 from a byte slice.
    ///
    /// # Errors
    /// Returns `InvalidLength` if slice is not exactly 32 bytes.
    pub fn from_slice(slice: &[u8]) -> Result<Self, PrimitiveError>;

    /// Parses an H256 from a hex string.
    ///
    /// # Errors
    /// - `InvalidHex` if string contains non-hex characters
    /// - `InvalidLength` if decoded bytes != 32
    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError>;

    /// Returns the zero hash (all zeros).
    pub fn zero() -> Self;

    /// Returns a reference to the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; HASH_LENGTH];

    /// Checks if this is the zero hash.
    pub fn is_zero(&self) -> bool;
}

impl AsRef<[u8]> for H256;
impl From<[u8; HASH_LENGTH]> for H256;
impl std::fmt::Display for H256;
impl std::fmt::LowerHex for H256;
```

### 2.5 Type: H160

```rust
/// Alias for Address (20-byte hash).
pub type H160 = Address;
```

### 2.6 Type: U256

```rust
/// A 256-bit unsigned integer.
///
/// # Invariants
/// - Arithmetic operations check for overflow
///
/// # Thread Safety
/// Send + Sync (pure data)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct U256([u64; 4]); // Little-endian limbs

impl U256 {
    /// Zero value.
    pub const ZERO: Self;

    /// Maximum value (2^256 - 1).
    pub const MAX: Self;

    /// One value.
    pub const ONE: Self;

    /// Creates a U256 from big-endian bytes.
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self;

    /// Creates a U256 from little-endian bytes.
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self;

    /// Converts to big-endian bytes.
    pub fn to_be_bytes(&self) -> [u8; 32];

    /// Converts to little-endian bytes.
    pub fn to_le_bytes(&self) -> [u8; 32];

    /// Creates from a u64 value.
    pub fn from_u64(val: u64) -> Self;

    /// Checked addition. Returns None on overflow.
    pub fn checked_add(&self, other: &Self) -> Option<Self>;

    /// Checked subtraction. Returns None on underflow.
    pub fn checked_sub(&self, other: &Self) -> Option<Self>;

    /// Checked multiplication. Returns None on overflow.
    pub fn checked_mul(&self, other: &Self) -> Option<Self>;

    /// Checked division. Returns None if divisor is zero.
    pub fn checked_div(&self, other: &Self) -> Option<Self>;

    /// Returns true if value is zero.
    pub fn is_zero(&self) -> bool;
}

impl From<u64> for U256;
impl From<u128> for U256;
impl std::fmt::Display for U256;
impl std::fmt::LowerHex for U256;
```

---

## 3. Module: bach-crypto

### 3.1 Error Types

```rust
/// Errors from cryptographic operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// Private key bytes are not a valid scalar
    InvalidPrivateKey,
    /// Signature bytes are malformed
    InvalidSignature,
    /// Public key recovery failed
    RecoveryFailed,
    /// Public key is invalid
    InvalidPublicKey,
}
```

### 3.2 Hashing Functions

```rust
/// Computes the Keccak-256 hash of the input.
///
/// # Arguments
/// * `data` - Byte slice to hash
///
/// # Returns
/// 32-byte hash result
pub fn keccak256(data: &[u8]) -> H256;

/// Computes the Keccak-256 hash of concatenated inputs.
///
/// # Arguments
/// * `data` - Slice of byte slices to concatenate and hash
///
/// # Returns
/// 32-byte hash result
pub fn keccak256_concat(data: &[&[u8]]) -> H256;
```

### 3.3 Type: PrivateKey

```rust
/// A secp256k1 private key (32 bytes).
///
/// # Security
/// - Implements Zeroize on drop
/// - Debug impl does not reveal key bytes
///
/// # Thread Safety
/// Send + Sync
pub struct PrivateKey { /* private fields */ }

impl PrivateKey {
    /// Generates a random private key using OS entropy.
    pub fn random() -> Self;

    /// Creates a private key from raw bytes.
    ///
    /// # Errors
    /// Returns `InvalidPrivateKey` if bytes are not a valid secp256k1 scalar.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, CryptoError>;

    /// Returns the raw bytes.
    pub fn to_bytes(&self) -> [u8; 32];

    /// Derives the corresponding public key.
    pub fn public_key(&self) -> PublicKey;

    /// Signs a message hash.
    ///
    /// # Arguments
    /// * `message` - 32-byte message hash (NOT the raw message)
    pub fn sign(&self, message: &H256) -> Signature;
}
```

### 3.4 Type: PublicKey

```rust
/// A secp256k1 public key (uncompressed, 64 bytes without prefix).
///
/// # Thread Safety
/// Send + Sync
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey { /* private fields */ }

impl PublicKey {
    /// Creates from uncompressed bytes (64 bytes, no 0x04 prefix).
    ///
    /// # Errors
    /// Returns `InvalidPublicKey` if bytes are not a valid point.
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self, CryptoError>;

    /// Returns the uncompressed bytes (64 bytes).
    pub fn to_bytes(&self) -> [u8; 64];

    /// Derives the Ethereum-style address.
    ///
    /// Address = keccak256(public_key)[12..32]
    pub fn to_address(&self) -> Address;

    /// Verifies a signature against this public key.
    ///
    /// # Arguments
    /// * `signature` - The signature to verify
    /// * `message` - 32-byte message hash
    pub fn verify(&self, signature: &Signature, message: &H256) -> bool;
}
```

### 3.5 Type: Signature

```rust
/// An ECDSA signature with recovery ID (65 bytes: r + s + v).
///
/// # Thread Safety
/// Send + Sync
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature { /* private fields */ }

/// Length of a signature in bytes (r=32 + s=32 + v=1)
pub const SIGNATURE_LENGTH: usize = 65;

impl Signature {
    /// Creates a signature from raw bytes.
    ///
    /// # Arguments
    /// * `bytes` - 65 bytes: r (32) + s (32) + v (1)
    ///
    /// # Errors
    /// Returns `InvalidSignature` if bytes are malformed.
    pub fn from_bytes(bytes: &[u8; SIGNATURE_LENGTH]) -> Result<Self, CryptoError>;

    /// Returns the raw bytes (r + s + v).
    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH];

    /// Verifies this signature against a public key and message.
    pub fn verify(&self, pubkey: &PublicKey, message: &H256) -> bool;

    /// Recovers the public key from the signature and message.
    ///
    /// # Errors
    /// Returns `RecoveryFailed` if recovery is not possible.
    pub fn recover(&self, message: &H256) -> Result<PublicKey, CryptoError>;

    /// Returns the r component.
    pub fn r(&self) -> &[u8; 32];

    /// Returns the s component.
    pub fn s(&self) -> &[u8; 32];

    /// Returns the recovery ID (0 or 1, stored as 27 or 28 for Ethereum).
    pub fn v(&self) -> u8;
}
```

---

## 4. Module: bach-types

### 4.1 Constants

```rust
/// Ownership status: transaction owns the key
pub const PRIORITY_OWNED: u8 = 0;

/// Ownership status: ownership released
pub const PRIORITY_DISOWNED: u8 = 1;
```

### 4.2 Error Types

```rust
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
```

### 4.3 Type: PriorityCode

```rust
/// Priority code for transaction ordering in Seamless Scheduling.
///
/// Structure: [release_bit (1 byte)] [block_height (8 bytes)] [hash (32 bytes)]
///
/// # Ordering
/// Lower value = Higher priority
/// - Released (1) > Owned (0)
/// - Lower block height > Higher block height
/// - Lower hash > Higher hash
///
/// # Invariants
/// - release_bit is always 0 or 1
/// - Total size is 41 bytes
///
/// # Thread Safety
/// Send + Sync
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityCode {
    release_bit: u8,
    block_height: u64,
    hash: H256,
}

impl PriorityCode {
    /// Creates a new priority code with OWNED status.
    ///
    /// # Arguments
    /// * `block_height` - The block number
    /// * `hash` - Hash of (transaction, block_transactions_hash)
    pub fn new(block_height: u64, hash: H256) -> Self;

    /// Sets the release bit to DISOWNED.
    /// After calling, this priority code will compare as lower priority.
    pub fn release(&mut self);

    /// Returns true if ownership has been released.
    pub fn is_released(&self) -> bool;

    /// Returns the block height.
    pub fn block_height(&self) -> u64;

    /// Returns the hash component.
    pub fn hash(&self) -> &H256;

    /// Serializes to bytes (41 bytes).
    pub fn to_bytes(&self) -> [u8; 41];

    /// Deserializes from bytes.
    pub fn from_bytes(bytes: &[u8; 41]) -> Self;
}

impl Ord for PriorityCode;
impl PartialOrd for PriorityCode;
```

### 4.4 Type: ReadWriteSet

```rust
/// Records the keys read and written during transaction execution.
///
/// # Thread Safety
/// NOT thread-safe. Use one instance per transaction execution.
#[derive(Debug, Clone, Default)]
pub struct ReadWriteSet {
    reads: Vec<H256>,
    writes: Vec<(H256, Vec<u8>)>,
}

impl ReadWriteSet {
    /// Creates an empty read-write set.
    pub fn new() -> Self;

    /// Records a read access to a key.
    pub fn record_read(&mut self, key: H256);

    /// Records a write access to a key with its new value.
    pub fn record_write(&mut self, key: H256, value: Vec<u8>);

    /// Returns all read keys.
    pub fn reads(&self) -> &[H256];

    /// Returns all write key-value pairs.
    pub fn writes(&self) -> &[(H256, Vec<u8>)];

    /// Returns all unique keys (reads + writes).
    pub fn all_keys(&self) -> Vec<H256>;

    /// Clears all recorded accesses.
    pub fn clear(&mut self);
}
```

### 4.5 Type: Transaction

```rust
/// A blockchain transaction.
///
/// # Thread Safety
/// Send + Sync (immutable after creation)
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
    ) -> Self;

    /// Computes the transaction hash.
    pub fn hash(&self) -> H256;

    /// Recovers the sender address from the signature.
    ///
    /// # Errors
    /// Returns `RecoveryFailed` if signature recovery fails.
    pub fn sender(&self) -> Result<Address, TypeError>;

    /// Returns the signing hash (hash used for signature).
    pub fn signing_hash(&self) -> H256;
}
```

### 4.6 Type: Block

```rust
/// A block containing transactions.
///
/// # Thread Safety
/// Send + Sync
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
    ) -> Self;

    /// Computes the block hash.
    pub fn hash(&self) -> H256;

    /// Computes the hash of all transaction hashes.
    pub fn transactions_hash(&self) -> H256;

    /// Returns the number of transactions.
    pub fn transaction_count(&self) -> usize;
}
```

---

## 5. Module: bach-state

### 5.1 Error Types

```rust
/// Errors from state operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// Key not found in state
    KeyNotFound(H256),
    /// Snapshot has expired or is invalid
    SnapshotExpired,
    /// Lock acquisition failed
    LockError(String),
}
```

### 5.2 Trait: StateDB

```rust
/// Interface for state storage.
///
/// # Thread Safety
/// Implementation must be thread-safe if used with scheduler.
pub trait StateDB: Send + Sync {
    /// Reads a value by key.
    ///
    /// # Returns
    /// - `Some(value)` if key exists
    /// - `None` if key does not exist
    fn get(&self, key: &H256) -> Option<Vec<u8>>;

    /// Writes a value.
    fn set(&mut self, key: H256, value: Vec<u8>);

    /// Deletes a key.
    fn delete(&mut self, key: &H256);

    /// Creates a read-only snapshot.
    fn snapshot(&self) -> Snapshot;

    /// Commits a batch of writes atomically.
    fn commit(&mut self, writes: &[(H256, Vec<u8>)]);

    /// Returns all keys (for testing/debugging).
    fn keys(&self) -> Vec<H256>;
}
```

### 5.3 Type: MemoryStateDB

```rust
/// In-memory implementation of StateDB.
///
/// # Thread Safety
/// Send + Sync (uses internal locking)
#[derive(Debug, Default)]
pub struct MemoryStateDB { /* private fields */ }

impl MemoryStateDB {
    /// Creates a new empty state database.
    pub fn new() -> Self;
}

impl StateDB for MemoryStateDB;
```

### 5.4 Type: Snapshot

```rust
/// A read-only snapshot of state at a point in time.
///
/// # Thread Safety
/// Send + Sync (immutable)
#[derive(Debug, Clone)]
pub struct Snapshot { /* private fields */ }

impl Snapshot {
    /// Reads a value by key from the snapshot.
    pub fn get(&self, key: &H256) -> Option<Vec<u8>>;
}
```

### 5.5 Type: OwnershipEntry

```rust
/// An entry in the ownership table for a single key.
///
/// Implements Algorithm 1 from the paper.
///
/// # Thread Safety
/// Send + Sync (uses RwLock internally)
pub struct OwnershipEntry { /* private fields */ }

impl OwnershipEntry {
    /// Creates a new entry with DISOWNED status.
    pub fn new() -> Self;

    /// Releases ownership by setting status to DISOWNED.
    ///
    /// After calling, any priority code will be considered higher priority.
    pub fn release_ownership(&self);

    /// Checks if the given priority code can claim ownership.
    ///
    /// # Returns
    /// `true` if `who <= current_owner` (i.e., who has higher or equal priority)
    pub fn check_ownership(&self, who: &PriorityCode) -> bool;

    /// Attempts to claim ownership.
    ///
    /// # Returns
    /// `true` if ownership was successfully claimed (who has higher priority)
    /// `false` if a higher-priority transaction already owns this key
    pub fn try_set_owner(&self, who: &PriorityCode) -> bool;

    /// Returns a clone of the current owner's priority code.
    pub fn current_owner(&self) -> PriorityCode;
}

impl Default for OwnershipEntry;
impl Clone for OwnershipEntry;
```

### 5.6 Type: OwnershipTable

```rust
/// Table mapping storage keys to their ownership entries.
///
/// # Thread Safety
/// Send + Sync (uses concurrent hashmap internally)
pub struct OwnershipTable { /* private fields */ }

impl OwnershipTable {
    /// Creates a new empty ownership table.
    pub fn new() -> Self;

    /// Gets the ownership entry for a key, creating one if it doesn't exist.
    ///
    /// # Returns
    /// Arc to the entry (shared reference for concurrent access)
    pub fn get_or_create(&self, key: &H256) -> Arc<OwnershipEntry>;

    /// Releases ownership of all specified keys.
    pub fn release_all(&self, keys: &[H256]);

    /// Clears all entries from the table.
    pub fn clear(&self);

    /// Returns the number of entries.
    pub fn len(&self) -> usize;

    /// Returns true if the table is empty.
    pub fn is_empty(&self) -> bool;
}

impl Default for OwnershipTable;
```

---

## 6. Module: bach-scheduler

### 6.1 Error Types

```rust
/// Errors from scheduling operations
#[derive(Debug, Clone)]
pub enum SchedulerError {
    /// Transaction execution failed
    ExecutionFailed { tx_hash: H256, reason: String },
    /// Transaction exceeded maximum retry attempts
    MaxRetriesExceeded { tx_hash: H256, attempts: usize },
    /// Block validation failed
    InvalidBlock(String),
    /// State access error
    StateError(StateError),
}
```

### 6.2 Type: ExecutionResult

```rust
/// Result of executing a single transaction.
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Execution succeeded
    Success {
        /// Return data (if any)
        output: Vec<u8>,
    },
    /// Execution failed
    Failed {
        /// Error reason
        reason: String,
    },
}

impl ExecutionResult {
    /// Returns true if execution succeeded.
    pub fn is_success(&self) -> bool;
}
```

### 6.3 Type: ExecutedTransaction

```rust
/// A transaction that has been executed with its results.
#[derive(Debug, Clone)]
pub struct ExecutedTransaction {
    /// Original transaction
    pub transaction: Transaction,
    /// Assigned priority code
    pub priority: PriorityCode,
    /// Recorded read-write set
    pub rwset: ReadWriteSet,
    /// Execution result
    pub result: ExecutionResult,
}

impl ExecutedTransaction {
    /// Returns the transaction hash.
    pub fn hash(&self) -> H256;
}
```

### 6.4 Type: ScheduleResult

```rust
/// Result of scheduling a block.
#[derive(Debug)]
pub struct ScheduleResult {
    /// Successfully confirmed transactions in order
    pub confirmed: Vec<ExecutedTransaction>,
    /// Final block hash
    pub block_hash: H256,
    /// New state root after applying changes
    pub state_root: H256,
    /// Number of re-executions performed
    pub reexecution_count: usize,
}
```

### 6.5 Trait: TransactionExecutor

```rust
/// Interface for executing transactions.
///
/// # Thread Safety
/// Must be Send + Sync for parallel execution.
pub trait TransactionExecutor: Send + Sync {
    /// Executes a transaction against a state snapshot.
    ///
    /// # Arguments
    /// * `tx` - Transaction to execute
    /// * `snapshot` - State snapshot for reads
    ///
    /// # Returns
    /// Tuple of (read-write set, execution result)
    fn execute(&self, tx: &Transaction, snapshot: &Snapshot) -> (ReadWriteSet, ExecutionResult);
}
```

### 6.6 Trait: Scheduler

```rust
/// Interface for transaction scheduling.
///
/// # Thread Safety
/// Must be Send + Sync.
pub trait Scheduler: Send + Sync {
    /// Schedules and executes a block of transactions.
    ///
    /// # Arguments
    /// * `block` - Block containing transactions to execute
    /// * `state` - Mutable state database
    /// * `executor` - Transaction executor implementation
    ///
    /// # Returns
    /// Schedule result with confirmed transactions
    ///
    /// # Errors
    /// Returns error if block is invalid or execution fails fatally.
    fn schedule(
        &self,
        block: Block,
        state: &mut dyn StateDB,
        executor: &dyn TransactionExecutor,
    ) -> Result<ScheduleResult, SchedulerError>;
}
```

### 6.7 Type: SeamlessScheduler

```rust
/// Implementation of Seamless Scheduling algorithm (Algorithm 2).
///
/// # Thread Safety
/// Send + Sync (uses thread pool internally)
pub struct SeamlessScheduler { /* private fields */ }

/// Default number of worker threads
pub const DEFAULT_THREAD_COUNT: usize = 4;

/// Maximum re-execution attempts per transaction
pub const MAX_RETRIES: usize = 100;

impl SeamlessScheduler {
    /// Creates a new scheduler with the specified thread count.
    ///
    /// # Arguments
    /// * `thread_count` - Number of parallel execution threads
    pub fn new(thread_count: usize) -> Self;

    /// Creates a scheduler with default thread count.
    pub fn default() -> Self;
}

impl Scheduler for SeamlessScheduler;
```

---

## 7. Algorithm Specifications

### 7.1 Algorithm 1: OwnershipEntry Methods

```
Constants:
    DISOWNED = 1
    OWNED = 0

OwnershipEntry::release_ownership(&self):
    lock(self.mutex)
    self.owner.release_bit = DISOWNED
    unlock(self.mutex)

OwnershipEntry::check_ownership(&self, who: &PriorityCode) -> bool:
    rlock(self.mutex)
    result = (who <= self.owner)  // Lower value = higher priority
    runlock(self.mutex)
    return result

OwnershipEntry::try_set_owner(&self, who: &PriorityCode) -> bool:
    // Fast path: check without write lock
    if not self.check_ownership(who):
        return false

    // Slow path: acquire write lock and verify
    lock(self.mutex)
    if who <= self.owner:
        self.owner = who.clone()
        unlock(self.mutex)
        return true
    unlock(self.mutex)
    return false
```

### 7.2 Algorithm 2: Seamless Scheduling

```
SeamlessScheduler::schedule(block, state, executor) -> ScheduleResult:
    ownership_table = OwnershipTable::new()
    snapshot = state.snapshot()
    confirmed = []
    reexecution_count = 0

    // Phase 1: Optimistic parallel execution
    executed = parallel_for tx in block.transactions:
        hash = keccak256_concat([tx.hash(), block.transactions_hash()])
        priority = PriorityCode::new(block.height, hash)
        (rwset, result) = executor.execute(tx, snapshot)

        // Try to claim ownership of write keys
        for (key, _) in rwset.writes:
            ownership_table.get_or_create(key).try_set_owner(priority)

        ExecutedTransaction { tx, priority, rwset, result }

    // Phase 2: Conflict detection and resolution loop
    pending = executed
    while not pending.is_empty():
        aborted = []

        // Check each transaction for conflicts
        for etx in pending:
            conflict = false

            // Check write set ownership
            for (key, _) in etx.rwset.writes:
                if not ownership_table.get_or_create(key).check_ownership(etx.priority):
                    conflict = true
                    break

            // Check read set ownership (if no write conflict)
            if not conflict:
                for key in etx.rwset.reads:
                    if not ownership_table.get_or_create(key).check_ownership(etx.priority):
                        conflict = true
                        break

            if conflict:
                aborted.push(etx)
            else:
                confirmed.push(etx)
                // Release ownership after confirmation
                for (key, _) in etx.rwset.writes:
                    ownership_table.get_or_create(key).release_ownership()

        // Re-execute aborted transactions
        pending = parallel_for etx in aborted:
            reexecution_count += 1
            (rwset, result) = executor.execute(etx.tx, snapshot)
            for (key, _) in rwset.writes:
                ownership_table.get_or_create(key).try_set_owner(etx.priority)
            ExecutedTransaction { tx: etx.tx, priority: etx.priority, rwset, result }

    // Phase 3: Commit changes
    all_writes = confirmed.flat_map(|etx| etx.rwset.writes)
    state.commit(all_writes)

    return ScheduleResult {
        confirmed,
        block_hash: block.hash(),
        state_root: compute_state_root(state),
        reexecution_count,
    }
```

---

## 8. Contract Verification

### 8.1 Automated Checks

| Check | Tool | Command | Required |
|-------|------|---------|----------|
| Type compatibility | `cargo check` | `cargo check --workspace` | Yes |
| Tests pass | `cargo test` | `cargo test --workspace` | Yes |
| No stubs | `check_stub_detection.sh` | See validator | Yes |
| Interface drift | `check_interface_drift.sh` | See validator | Yes |

### 8.2 Manual Review Checklist

- [x] All public APIs documented
- [x] Error conditions specified
- [x] Thread safety documented
- [x] Algorithms specified with pseudocode
- [x] Invariants listed for each type

---

## 9. Sign-off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Contract Author | ICDD Architect | 2026-02-09 | [x] |
| Technical Lead | | | [ ] |
| Security Review | | | [ ] |

---

## Revision History

| Version | Date | Author | Changes | Breaking |
|---------|------|--------|---------|----------|
| 1.0.0 | 2026-02-09 | ICDD Architect | Initial contract - LOCKED | - |
