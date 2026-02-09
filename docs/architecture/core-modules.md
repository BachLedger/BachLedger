# BachLedger Core Module Architecture

This document describes the architecture and dependency chain of the 5 core BachLedger modules implemented in Rust.

## Module Dependency Chain

```
bach-primitives (Layer 0)
    |
    v
bach-crypto (Layer 1) <-- uses primitives for H256, Address
    |
    v
bach-types (Layer 2) <-- uses crypto for signing, primitives for types
    |
    v
bach-state (Layer 3) <-- uses types for PriorityCode, primitives for H256
    |
    v
bach-scheduler (Layer 4) <-- uses all above, implements Algorithm 2
```

## Module Overview

### Layer 0: bach-primitives

**Purpose**: Foundation types for blockchain operations

**Key Types**:
- `Address` - 20-byte Ethereum-compatible address
- `H256` - 32-byte hash value
- `H160` - Type alias for Address
- `U256` - 256-bit unsigned integer

**Features**:
- No unsafe code (`#![forbid(unsafe_code)]`)
- Complete hex parsing with case-insensitive support
- Full 256-bit arithmetic with overflow detection
- All types implement `Send + Sync`

**Dependencies**: None (standalone)

---

### Layer 1: bach-crypto

**Purpose**: Cryptographic primitives for blockchain operations

**Key Types**:
- `PrivateKey` - secp256k1 private key (32 bytes)
- `PublicKey` - secp256k1 public key (64 bytes, uncompressed)
- `Signature` - ECDSA signature with recovery ID (65 bytes)

**Key Functions**:
- `keccak256(data) -> H256` - Keccak-256 hash
- `keccak256_concat(slices) -> H256` - Hash of concatenated inputs

**Security Features**:
- Uses OS entropy via `rand_core::OsRng`
- Private key bytes redacted in Debug output
- RFC6979 deterministic signatures
- Prehash signing/verification (no double-hashing)

**Dependencies**: `bach-primitives`, `k256`, `sha3`, `rand_core`

---

### Layer 2: bach-types

**Purpose**: Core blockchain types for transactions and scheduling

**Key Types**:
- `PriorityCode` - Transaction priority for Seamless Scheduling
- `ReadWriteSet` - Records storage accesses during execution
- `Transaction` - Blockchain transaction with signature
- `Block` - Block containing transactions

**PriorityCode Structure**:
```
[release_bit: 1 byte] [block_height: 8 bytes BE] [hash: 32 bytes]
```

**Ordering Semantics**:
- Lower value = Higher priority
- OWNED (0) > DISOWNED (1) in priority
- Lower block_height > Higher block_height in priority
- Lower hash > Higher hash in priority (tiebreaker)

**Dependencies**: `bach-primitives`, `bach-crypto`

---

### Layer 3: bach-state

**Purpose**: State management and ownership tracking

**Key Types**:
- `StateDB` trait - Interface for state storage
- `MemoryStateDB` - In-memory implementation
- `Snapshot` - Read-only state snapshot (point-in-time isolation)
- `OwnershipEntry` - Per-key ownership tracking (Algorithm 1)
- `OwnershipTable` - Concurrent ownership table with `RwLock`

**Thread Safety**:
- `OwnershipEntry` uses `RwLock<PriorityCode>`
- `OwnershipTable` uses `RwLock<HashMap<H256, Arc<OwnershipEntry>>>`
- Double-checked locking in `get_or_create`
- No deadlock potential (simple lock hierarchy)

**Dependencies**: `bach-primitives`, `bach-types`

---

### Layer 4: bach-scheduler

**Purpose**: Implementation of Seamless Scheduling (Algorithm 2)

**Key Types**:
- `SeamlessScheduler` - Main scheduler implementation
- `TransactionExecutor` trait - Interface for transaction execution
- `Scheduler` trait - Interface for block scheduling
- `ExecutedTransaction` - Transaction with execution results
- `ScheduleResult` - Final scheduling result

**Algorithm 2 Phases**:
1. **Optimistic Parallel Execution** - Execute all transactions concurrently
2. **Conflict Detection** - Identify write-write and read-write conflicts
3. **Re-execution Loop** - Re-execute conflicting transactions
4. **Commit** - Apply all writes atomically

**Features**:
- Uses `rayon` for parallel execution
- Bounded re-execution (MAX_RETRIES = 100)
- Deterministic priority codes ensure consistent results
- Priority-based conflict resolution

**Dependencies**: `bach-primitives`, `bach-crypto`, `bach-types`, `bach-state`, `rayon`

---

## Data Flow

### Transaction Processing

```
1. Block arrives with transactions
                |
                v
2. Snapshot created from current state
                |
                v
3. Parallel execution (Phase 1)
   - Each transaction executes against snapshot
   - ReadWriteSet recorded
   - Ownership claimed for writes
                |
                v
4. Conflict detection (Phase 2)
   - Check write ownership (who <= current_owner?)
   - Check read conflicts (key owned by other tx?)
                |
                v
5. Resolution loop
   - Passed transactions confirm and release ownership
   - Aborted transactions re-execute
   - Repeat until all resolved
                |
                v
6. Commit changes (Phase 3)
   - All writes applied atomically to state
   - State root computed
```

### Ownership Flow

```
DISOWNED state (initial)
    |
    | try_set_owner(priority)
    v
OWNED by transaction (priority stored)
    |
    | release_ownership()
    v
DISOWNED state (can be claimed again)
```

### Priority Comparison

```rust
// Lower value = higher priority
fn cmp(&self, other: &Self) -> Ordering {
    // 1. Compare release_bit (0 < 1)
    self.release_bit.cmp(&other.release_bit)
        // 2. Compare block_height
        .then(self.block_height.cmp(&other.block_height))
        // 3. Compare hash (tiebreaker)
        .then(self.hash.cmp(&other.hash))
}
```

---

## Design Decisions

### Why Prehash Signing?

The message passed to `sign()` is already a hash (H256). Using prehash methods avoids double-hashing:
- `sign_prehash_recoverable` instead of `sign_recoverable`
- `verify_prehash` instead of `verify`
- `recover_from_prehash` instead of `recover_from_msg`

This ensures Ethereum compatibility and interoperability.

### Why Priority Code Ordering?

The "lower value = higher priority" semantics enable:
1. Earlier transactions (lower block_height) get priority
2. OWNED transactions beat DISOWNED for same key
3. Deterministic tiebreaker via hash comparison

### Why Snapshot Isolation?

Full clone ensures:
1. Parallel reads see consistent state
2. Writes during execution don't affect other transactions
3. Re-execution sees same state as original execution

### Why RwLock?

Using `RwLock` instead of `Mutex` allows:
1. Multiple concurrent readers (check_ownership)
2. Exclusive writers (try_set_owner, release)
3. Better performance under read-heavy workloads

---

## Constants

| Module | Constant | Value | Description |
|--------|----------|-------|-------------|
| primitives | `ADDRESS_LENGTH` | 20 | Ethereum address size |
| primitives | `HASH_LENGTH` | 32 | H256 hash size |
| crypto | `SIGNATURE_LENGTH` | 65 | r(32) + s(32) + v(1) |
| types | `PRIORITY_OWNED` | 0 | Ownership claimed |
| types | `PRIORITY_DISOWNED` | 1 | Ownership released |
| scheduler | `DEFAULT_THREAD_COUNT` | 4 | Default parallelism |
| scheduler | `MAX_RETRIES` | 100 | Re-execution limit |

---

## Error Types

| Module | Error | Description |
|--------|-------|-------------|
| primitives | `InvalidLength` | Slice/hex wrong size |
| primitives | `InvalidHex` | Invalid hex character |
| crypto | `InvalidPrivateKey` | Not a valid scalar |
| crypto | `InvalidSignature` | Malformed signature |
| crypto | `InvalidPublicKey` | Not on curve |
| crypto | `RecoveryFailed` | Cannot recover pubkey |
| types | `InvalidSignature` | Verification failed |
| types | `RecoveryFailed` | Cannot recover sender |
| types | `InvalidTransaction` | Bad transaction format |
| state | `KeyNotFound` | Key doesn't exist |
| state | `SnapshotExpired` | Snapshot invalid |
| state | `LockError` | Lock acquisition failed |
| scheduler | `ExecutionFailed` | Transaction failed |
| scheduler | `MaxRetriesExceeded` | Too many retries |
| scheduler | `InvalidBlock` | Bad block format |
| scheduler | `StateError` | Wrapped state error |

---

## Version Information

- **Rust Edition**: 2021
- **Review Date**: 2026-02-09
- **Status**: All modules APPROVED
