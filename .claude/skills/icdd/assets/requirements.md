# Requirements Document: BachLedger

## Module Information

| Field | Value |
|-------|-------|
| Module Name | BachLedger |
| Version | 2.0.0 |
| Author | ICDD Architect |
| Date | 2026-02-09 |
| Status | Updated - Full Implementation Scope |

---

## 1. 需求图谱 (Requirement Graph)

### 1.1 System Overview

BachLedger 是一个高性能联盟区块链系统，采用 OEV (Ordering-Execution-Validation) 架构，核心创新是 Seamless Scheduling 算法，通过消除块间空闲时间来最大化资源利用率。

### 1.2 Core Requirements Tree

```
BachLedger: High-Performance Consortium Blockchain with Seamless Scheduling
│
├── Layer 0: Core Primitives (bach-primitives)
│   ├── Address type (20 bytes, Ethereum-compatible)
│   ├── H256 hash type (32 bytes)
│   ├── H160 type (20 bytes, alias for Address)
│   ├── U256 unsigned integer (256-bit)
│   └── Bytes/BytesMut for arbitrary data
│
├── Layer 1: Cryptography (bach-crypto)
│   ├── Keccak256 hashing
│   ├── ECDSA signatures (secp256k1)
│   ├── Public key recovery
│   └── Address derivation from public key
│
├── Layer 2: Core Types (bach-types)
│   ├── Transaction structure (with RLP encoding)
│   ├── Block structure (header + body)
│   ├── ReadWriteSet for conflict tracking
│   ├── PriorityCode (semantic prefix + hash)
│   ├── Account state (nonce, balance, code_hash, storage_root)
│   └── Receipt (transaction result)
│
├── Layer 3: Storage (bach-storage)
│   ├── KVStore trait (abstract interface)
│   ├── RocksDB implementation
│   ├── Merkle Patricia Trie
│   └── State root computation
│
├── Layer 4: State Management (bach-state)
│   ├── StateDB (account state management)
│   ├── OwnershipTable (conflict tracking)
│   ├── OwnershipEntry (per-key ownership)
│   ├── Snapshot management (MVCC)
│   └── State transition function
│
├── Layer 5: Execution (bach-evm)
│   ├── EVM Interpreter
│   │   ├── Stack machine (1024 depth)
│   │   ├── Memory (expandable)
│   │   ├── Storage (persistent)
│   │   └── Call context
│   ├── Opcode implementations (full EVM)
│   │   ├── Arithmetic (ADD, SUB, MUL, DIV, MOD, etc.)
│   │   ├── Comparison (LT, GT, EQ, ISZERO)
│   │   ├── Bitwise (AND, OR, XOR, NOT, SHL, SHR)
│   │   ├── SHA3 (Keccak256)
│   │   ├── Environment (ADDRESS, BALANCE, ORIGIN, CALLER, etc.)
│   │   ├── Block info (BLOCKHASH, COINBASE, TIMESTAMP, etc.)
│   │   ├── Stack/Memory/Storage (POP, MLOAD, MSTORE, SLOAD, SSTORE)
│   │   ├── Flow control (JUMP, JUMPI, PC, JUMPDEST)
│   │   ├── Logging (LOG0-LOG4)
│   │   ├── System (CREATE, CALL, CALLCODE, RETURN, DELEGATECALL, etc.)
│   │   └── STATICCALL, REVERT, SELFDESTRUCT
│   ├── Gas metering
│   ├── Precompiled contracts (ecrecover, sha256, ripemd160, identity)
│   └── Contract deployment
│
├── Layer 6: Scheduler (bach-scheduler)
│   ├── SeamlessScheduler (core algorithm)
│   ├── TransactionQueue (cross-block queue)
│   ├── ConflictDetector (DAG extraction)
│   ├── ExecutionCoordinator (thread pool)
│   └── Block finalizer
│
├── Layer 7: Consensus (bach-consensus)
│   ├── TBFT (Tendermint BFT) protocol
│   │   ├── Proposer selection
│   │   ├── Pre-vote phase
│   │   ├── Pre-commit phase
│   │   ├── Commit phase
│   │   └── View change
│   ├── Validator set management
│   ├── Vote aggregation
│   └── Block finality
│
├── Layer 8: Network (bach-network)
│   ├── P2P transport (TCP/QUIC)
│   ├── Peer discovery
│   ├── Message routing
│   ├── Transaction pool (mempool)
│   ├── Block propagation
│   └── Consensus message handling
│
└── Layer 9: Node (bach-node)
    ├── Node configuration
    ├── Component orchestration
    ├── RPC server (JSON-RPC)
    └── CLI interface
```

### 1.3 Requirement Details

| ID | Requirement | Priority | Source | Dependencies |
|----|-------------|----------|--------|--------------|
| REQ-001 | Implement OEV architecture | Critical | Paper §3.2 | - |
| REQ-002 | Implement Seamless Scheduling algorithm | Critical | Paper §4 | REQ-001 |
| REQ-003 | Implement Ownership Table | Critical | Paper §4.1 | REQ-004 |
| REQ-004 | Implement Priority Code with semantic prefix | Critical | Paper §4.1 | - |
| REQ-005 | Support parallel transaction execution | Critical | Paper §4.2 | REQ-002, REQ-003 |
| REQ-006 | Implement conflict detection mechanism | Critical | Paper §4.2 | REQ-003 |
| REQ-007 | Support optimistic execution with re-execution | Critical | Paper §4.2 | REQ-006 |
| REQ-008 | Ensure deterministic serializability | Critical | Paper §2.3 | REQ-002 |
| REQ-009 | Implement full EVM interpreter | High | User requirement | REQ-001 |
| REQ-010 | Implement TBFT consensus protocol | High | Paper §3.2, User | REQ-008 |
| REQ-011 | Implement P2P network layer | High | User requirement | - |
| REQ-012 | Implement persistent storage with RocksDB | High | User requirement | - |
| REQ-013 | Support cluster-node architecture | Medium | Paper §1 | REQ-011 |
| REQ-014 | Implement transaction pool | Medium | Paper §3.2 | REQ-011 |
| REQ-015 | Implement JSON-RPC interface | Medium | Interoperability | REQ-009 |

---

## 2. Explicit Requirements (显式需求)

### 2.1 Functional Requirements - Core Scheduling

| ID | Description | Rationale | Acceptance Criteria |
|----|-------------|-----------|---------------------|
| FR-001 | Transaction with priority code | Core scheduling unit | GIVEN tx WHEN priority computed THEN unique within block |
| FR-002 | Block with ordered transactions | Container for txs | GIVEN block WHEN created THEN all txs have valid priorities |
| FR-003 | OwnershipTable with fine-grained locking | Conflict resolution | GIVEN concurrent access to different keys THEN no blocking |
| FR-004 | OwnershipEntry operations | Per-key ownership | GIVEN release THEN priority code's first bit becomes 1 |
| FR-005 | SeamlessScheduler algorithm | Core innovation | GIVEN txs WHEN scheduled THEN topological order maintained |
| FR-006 | Priority Code structure | Deterministic ordering | GIVEN same block THEN hash determines tx order |
| FR-007 | ReadWriteSet tracking | Conflict detection | GIVEN tx execution THEN rset/wset recorded |
| FR-008 | Snapshot management | State isolation | GIVEN snapshot THEN consistent reads guaranteed |

### 2.2 Functional Requirements - EVM

| ID | Description | Rationale | Acceptance Criteria |
|----|-------------|-----------|---------------------|
| FR-EVM-001 | Stack machine with 1024 depth | EVM spec | GIVEN 1025 items WHEN push THEN stack overflow error |
| FR-EVM-002 | Memory operations | EVM spec | GIVEN MLOAD/MSTORE THEN correct byte access |
| FR-EVM-003 | Storage operations | EVM spec | GIVEN SLOAD/SSTORE THEN persistent state changes |
| FR-EVM-004 | Gas metering | Resource control | GIVEN operation WHEN executed THEN correct gas charged |
| FR-EVM-005 | Contract creation | Smart contracts | GIVEN CREATE WHEN valid THEN contract deployed |
| FR-EVM-006 | Contract calls | Interoperability | GIVEN CALL WHEN valid THEN cross-contract execution |
| FR-EVM-007 | Precompiled contracts | Efficiency | GIVEN ecrecover WHEN called THEN signature recovered |
| FR-EVM-008 | Event logging | Observability | GIVEN LOG0-4 THEN events emitted correctly |

### 2.3 Functional Requirements - Consensus

| ID | Description | Rationale | Acceptance Criteria |
|----|-------------|-----------|---------------------|
| FR-CON-001 | Proposer selection | Leader election | GIVEN round WHEN started THEN deterministic proposer |
| FR-CON-002 | Pre-vote phase | BFT safety | GIVEN proposal WHEN 2f+1 prevotes THEN proceed |
| FR-CON-003 | Pre-commit phase | BFT safety | GIVEN prevotes WHEN 2f+1 precommits THEN proceed |
| FR-CON-004 | Commit phase | Finality | GIVEN precommits WHEN committed THEN block final |
| FR-CON-005 | View change | Liveness | GIVEN timeout WHEN no progress THEN view change |
| FR-CON-006 | Byzantine tolerance | Security | GIVEN f Byzantine WHEN n>3f THEN safety maintained |

### 2.4 Functional Requirements - Network

| ID | Description | Rationale | Acceptance Criteria |
|----|-------------|-----------|---------------------|
| FR-NET-001 | Peer discovery | Decentralization | GIVEN bootstrap WHEN started THEN peers discovered |
| FR-NET-002 | Message routing | Communication | GIVEN message WHEN sent THEN delivered to target |
| FR-NET-003 | Transaction broadcast | Mempool | GIVEN new tx WHEN valid THEN broadcast to peers |
| FR-NET-004 | Block propagation | Consensus | GIVEN new block WHEN finalized THEN propagated |
| FR-NET-005 | Connection management | Reliability | GIVEN peer failure WHEN detected THEN reconnect |

### 2.5 Functional Requirements - Storage

| ID | Description | Rationale | Acceptance Criteria |
|----|-------------|-----------|---------------------|
| FR-STO-001 | Key-value storage | Persistence | GIVEN put WHEN get THEN same value returned |
| FR-STO-002 | Batch operations | Atomicity | GIVEN batch WHEN committed THEN all or nothing |
| FR-STO-003 | Merkle Patricia Trie | State proof | GIVEN state WHEN computed THEN correct root hash |
| FR-STO-004 | State snapshots | Recovery | GIVEN snapshot WHEN restored THEN consistent state |

### 2.6 Non-Functional Requirements

| ID | Category | Description | Target Metric |
|----|----------|-------------|---------------|
| NFR-001 | Performance | Minimize thread idle time | < 20% idle with 64 threads |
| NFR-002 | Throughput | High transaction throughput | > 10,000 TPS with 64 threads |
| NFR-003 | Latency | Low per-tx latency | < 10ms average |
| NFR-004 | Concurrency | Linear scaling | Up to 64 threads |
| NFR-005 | Determinism | Reproducible results | 100% deterministic |
| NFR-006 | Security | BFT safety | Tolerates f < n/3 Byzantine |
| NFR-007 | Reliability | No data corruption | Zero corruption in stress tests |
| NFR-008 | Storage | Efficient disk usage | < 2x raw data size |
| NFR-009 | Network | Low message latency | < 100ms consensus round |

---

## 3. Implicit Requirements (隐式需求)

### 3.1 Thread Safety

| ID | Implicit Requirement | Derived From | Verification Method |
|----|---------------------|--------------|---------------------|
| IR-001 | Thread-safe OwnershipTable | Parallel execution | Race condition tests |
| IR-002 | Lock-free where possible | Performance | Contention benchmarks |
| IR-003 | Deadlock-free locking | Fine-grained locks | Deadlock detection |
| IR-004 | Ordered mutex for blocks | Cross-block scheduling | Ordering tests |

### 3.2 EVM Compliance

| ID | Implicit Requirement | Derived From | Verification Method |
|----|---------------------|--------------|---------------------|
| IR-EVM-001 | Ethereum Yellow Paper compliance | EVM | Official test vectors |
| IR-EVM-002 | Gas schedule accuracy | EVM | Gas comparison tests |
| IR-EVM-003 | Precompile correctness | EVM | Known test vectors |
| IR-EVM-004 | Revert handling | EVM | State rollback tests |

### 3.3 Industry Standards

| ID | Standard/Practice | Applicability | Implementation Notes |
|----|-------------------|---------------|---------------------|
| IS-001 | Ethereum Address (20 bytes) | Interoperability | keccak256 derivation |
| IS-002 | ECDSA secp256k1 | Signing | k256 crate |
| IS-003 | Keccak-256 | Hashing | sha3 crate |
| IS-004 | RLP encoding | Serialization | Custom implementation |
| IS-005 | JSON-RPC 2.0 | API | eth_* compatible subset |

### 3.4 Security Considerations

| ID | Security Aspect | Threat Model | Mitigation |
|----|-----------------|--------------|------------|
| SC-001 | Priority manipulation | Adversary crafts tx | Hash includes block randomness |
| SC-002 | Ownership poisoning | Malicious claims | Strict priority comparison |
| SC-003 | Race conditions | Timing attacks | Fine-grained RWLock |
| SC-004 | DoS via conflicts | Resource exhaustion | Re-execution limits |
| SC-005 | Reentrancy attacks | Contract exploits | Check-effects-interactions |
| SC-006 | Integer overflow | EVM bugs | Checked arithmetic |
| SC-007 | Byzantine validators | Consensus attack | BFT 2f+1 threshold |
| SC-008 | Network partitions | Liveness attack | View change protocol |

---

## 4. 风险登记表 (Risk Register)

| ID | Risk Description | Probability | Impact | Severity | Mitigation Strategy | Owner | Status |
|----|------------------|-------------|--------|----------|---------------------|-------|--------|
| RISK-001 | Deadlock in OwnershipTable | Medium | High | High | Consistent lock ordering, timeout | Coder | Open |
| RISK-002 | Priority code collision | Low | High | Medium | 256-bit hash | Architect | Mitigated |
| RISK-003 | Memory exhaustion | Medium | Medium | Medium | Bounded queues | Coder | Open |
| RISK-004 | Re-execution loop | Low | High | Medium | Max retry count | Coder | Open |
| RISK-005 | Snapshot inconsistency | Low | Critical | High | MVCC implementation | Coder | Open |
| RISK-006 | EVM gas miscalculation | Medium | High | High | Extensive test vectors | Tester | Open |
| RISK-007 | Consensus liveness failure | Low | Critical | High | View change timeout | Coder | Open |
| RISK-008 | Network partition | Medium | High | High | Peer reconnection | Coder | Open |
| RISK-009 | Storage corruption | Low | Critical | High | Write-ahead logging | Coder | Open |
| RISK-010 | External dependency vuln | Medium | High | High | Minimal dependencies | Architect | Open |

---

## 5. 验收矩阵 (Acceptance Matrix)

### 5.1 Test Coverage by Module

| Module | Unit Tests | Integration | Concurrency | Property-based | Fuzz |
|--------|------------|-------------|-------------|----------------|------|
| bach-primitives | [ ] | [ ] | N/A | [ ] | [ ] |
| bach-crypto | [ ] | [ ] | N/A | [ ] | [ ] |
| bach-types | [ ] | [ ] | N/A | [ ] | [ ] |
| bach-storage | [ ] | [ ] | [ ] | [ ] | [ ] |
| bach-state | [ ] | [ ] | [ ] | [ ] | [ ] |
| bach-evm | [ ] | [ ] | [ ] | [ ] | [ ] |
| bach-scheduler | [ ] | [ ] | [ ] | [ ] | [ ] |
| bach-consensus | [ ] | [ ] | [ ] | [ ] | [ ] |
| bach-network | [ ] | [ ] | [ ] | [ ] | [ ] |

### 5.2 EVM Test Vectors

| Category | Test Suite | Status |
|----------|-----------|--------|
| Arithmetic | ethereum/tests/VMTests | [ ] Pending |
| Bitwise | ethereum/tests/VMTests | [ ] Pending |
| SHA3 | ethereum/tests/VMTests | [ ] Pending |
| Memory | ethereum/tests/VMTests | [ ] Pending |
| Storage | ethereum/tests/VMTests | [ ] Pending |
| Control Flow | ethereum/tests/VMTests | [ ] Pending |
| Calls | ethereum/tests/VMTests | [ ] Pending |
| Precompiles | ethereum/tests/VMTests | [ ] Pending |

### 5.3 Consensus Test Scenarios

| Scenario | Description | Status |
|----------|-------------|--------|
| Happy path | All honest, no delays | [ ] Pending |
| Leader failure | Proposer crashes | [ ] Pending |
| Network delay | Messages delayed | [ ] Pending |
| Byzantine proposer | Malicious leader | [ ] Pending |
| View change | Timeout triggered | [ ] Pending |

---

## 6. Module Decomposition

### 6.1 Updated Module Structure

```
bachledger/rust/
├── Cargo.toml                    # Workspace definition
├── bach-primitives/              # Layer 0: Basic types
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── address.rs            # Address (20 bytes)
│       ├── h256.rs               # H256 (32 bytes)
│       ├── h160.rs               # H160 (20 bytes)
│       ├── u256.rs               # U256 (256-bit integer)
│       └── bytes.rs              # Bytes/BytesMut
│
├── bach-crypto/                  # Layer 1: Cryptography
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── keccak.rs             # Keccak256
│       ├── signature.rs          # ECDSA
│       └── keys.rs               # Key derivation
│
├── bach-types/                   # Layer 2: Core types
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── transaction.rs        # Transaction
│       ├── block.rs              # Block
│       ├── priority.rs           # PriorityCode
│       ├── rwset.rs              # ReadWriteSet
│       ├── account.rs            # Account state
│       ├── receipt.rs            # Transaction receipt
│       └── rlp.rs                # RLP encoding
│
├── bach-storage/                 # Layer 3: Storage
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── traits.rs             # KVStore trait
│       ├── rocks.rs              # RocksDB implementation
│       ├── memory.rs             # In-memory (testing)
│       └── trie.rs               # Merkle Patricia Trie
│
├── bach-state/                   # Layer 4: State management
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── db.rs                 # StateDB
│       ├── ownership.rs          # OwnershipTable
│       ├── entry.rs              # OwnershipEntry
│       └── snapshot.rs           # Snapshot/MVCC
│
├── bach-evm/                     # Layer 5: EVM
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── interpreter.rs        # Main interpreter
│       ├── stack.rs              # EVM stack
│       ├── memory.rs             # EVM memory
│       ├── opcodes/              # Opcode implementations
│       │   ├── mod.rs
│       │   ├── arithmetic.rs
│       │   ├── comparison.rs
│       │   ├── bitwise.rs
│       │   ├── crypto.rs
│       │   ├── environment.rs
│       │   ├── block.rs
│       │   ├── stack_memory.rs
│       │   ├── storage.rs
│       │   ├── control.rs
│       │   ├── logging.rs
│       │   └── system.rs
│       ├── gas.rs                # Gas costs
│       ├── precompiles.rs        # Precompiled contracts
│       └── context.rs            # Execution context
│
├── bach-scheduler/               # Layer 6: Scheduling
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── scheduler.rs          # SeamlessScheduler
│       ├── queue.rs              # TransactionQueue
│       ├── conflict.rs           # ConflictDetector
│       ├── executor.rs           # ExecutionCoordinator
│       └── finalizer.rs          # Block finalizer
│
├── bach-consensus/               # Layer 7: Consensus
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── tbft.rs               # TBFT protocol
│       ├── proposer.rs           # Proposer selection
│       ├── vote.rs               # Vote handling
│       ├── validator.rs          # Validator set
│       └── state_machine.rs      # Consensus state
│
├── bach-network/                 # Layer 8: Network
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── transport.rs          # TCP/QUIC
│       ├── peer.rs               # Peer management
│       ├── discovery.rs          # Peer discovery
│       ├── mempool.rs            # Transaction pool
│       └── message.rs            # Message types
│
└── bach-node/                    # Layer 9: Node
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── config.rs             # Node configuration
        ├── node.rs               # Node orchestration
        ├── rpc.rs                # JSON-RPC server
        └── main.rs               # CLI entry point
```

### 6.2 Dependency Graph

```
                    bach-primitives
                          │
                          ▼
                    bach-crypto
                          │
                          ▼
                     bach-types
                    /    │    \
                   /     │     \
                  ▼      ▼      ▼
         bach-storage  bach-state  bach-network
                  \      │      /       │
                   \     │     /        │
                    ▼    ▼    ▼         │
                    bach-evm            │
                         │              │
                         ▼              │
                  bach-scheduler        │
                         │              │
                         ▼              ▼
                  bach-consensus ◄──────┘
                         │
                         ▼
                     bach-node
```

---

## 7. Algorithm Specifications

### 7.1 Ownership Entry Methods (Algorithm 1 from Paper)

```
Constants:
  DISOWNED = 1 (prefix byte)
  OWNED = 0 (prefix byte)

OwnershipEntry {
  owner: PriorityCode   // Current owner's priority code
  mutex: RWMutex        // Fine-grained lock
}

fn release_ownership(&self) {
  self.mutex.write_lock()
  self.owner.set_release_bit(DISOWNED)
  self.mutex.unlock()
}

fn check_ownership(&self, who: &PriorityCode) -> bool {
  self.mutex.read_lock()
  let result = who <= self.owner
  self.mutex.unlock()
  result
}

fn try_set_owner(&self, who: PriorityCode) -> bool {
  if !self.check_ownership(&who) {
    return false
  }
  self.mutex.write_lock()
  if who <= self.owner {
    self.owner = who
    self.mutex.unlock()
    return true
  }
  self.mutex.unlock()
  false
}
```

### 7.2 Seamless Scheduling (Algorithm 2 from Paper)

```
fn schedule(block: Block, state: &State) -> ProcessedBlock {
  let snapshot = state.create_snapshot()
  let mut approved: Vec<Transaction> = vec![]

  // Phase 1: Optimistic execution (parallel)
  let mut executed = optimistic_execute_txs(&block, &snapshot)

  // Critical section for conflict resolution
  lock!(global_mutex)
  state.update_snapshot()

  // Phase 2: Iterative conflict detection and re-execution
  while !executed.is_empty() {
    let (aborted, passed) = detect_conflict(&executed)
    approved.extend(passed)
    executed = re_execute_txs(&block, aborted, &snapshot)
  }

  unlock!(global_mutex)
  finalize_block(approved)
}

fn optimistic_execute_txs(block: &Block, snapshot: &Snapshot) -> Vec<ExecutedTx> {
  block.txs.par_iter().map(|tx| {
    let hash_field = keccak256(tx, keccak256(block.txs))
    tx.priority = PriorityCode::new(OWNED, block.height, hash_field)
    let (rset, wset) = execute(tx, snapshot)

    for w in &wset {
      ownership_table[w.key].try_set_owner(tx.priority)
    }

    ExecutedTx { tx, rset, wset }
  }).collect()
}

fn detect_conflict(executed: &[ExecutedTx]) -> (Vec<ExecutedTx>, Vec<ExecutedTx>) {
  executed.par_iter().partition(|tx| {
    // Check write set ownership
    for w in &tx.wset {
      if !ownership_table[w.key].check_ownership(&tx.priority) {
        return true // aborted
      }
    }
    // Check read set ownership
    for r in &tx.rset {
      if !ownership_table[r.key].check_ownership(&tx.priority) {
        return true // aborted
      }
    }
    false // passed
  })
}
```

### 7.3 Priority Code Structure

```
PriorityCode (41 bytes total):
┌──────────────┬─────────────────┬─────────────────────────┐
│ 1 byte       │ 8 bytes         │ 32 bytes                │
│ release_bit  │ block_height    │ hash(tx, block_hash)    │
└──────────────┴─────────────────┴─────────────────────────┘

Comparison: lexicographic (lower = higher priority)
- release_bit: 0x00 = OWNED (higher priority), 0x01 = DISOWNED
- block_height: big-endian u64, earlier blocks have priority
- hash: deterministic, unpredictable ordering within block
```

---

## 8. External Dependencies Policy

### 8.1 Allowed Dependencies (Minimal)

| Crate | Purpose | Justification |
|-------|---------|---------------|
| sha3 | Keccak256 | Standard crypto, well-audited |
| k256 | secp256k1 | Standard crypto, RustCrypto |
| rocksdb | Persistent storage | Industry standard |
| tokio | Async runtime | Networking requirement |
| parking_lot | Fast locks | Performance-critical |
| rayon | Parallel iterators | Scheduling parallelism |
| serde | Serialization | Config/RPC |

### 8.2 Forbidden Dependencies

- No full Ethereum client libraries (revm, ethers-rs)
- No external EVM implementations
- No complex framework dependencies
- Implement core functionality from scratch per paper

---

## 9. Sign-off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Architect | ICDD Architect | 2026-02-09 | [x] |
| Technical Lead | | | [ ] |
| User | | | [ ] Pending approval |

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-02-09 | ICDD Architect | Initial draft |
| 2.0 | 2026-02-09 | ICDD Architect | Added full EVM, TBFT, P2P, storage requirements |
