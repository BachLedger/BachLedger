# Seamless Scheduling Algorithm Implementation Guide

This document describes the implementation of Algorithm 2 (Seamless Scheduling) from the BachLedger paper.

## Overview

Seamless Scheduling enables deterministic parallel transaction execution through:

1. **Optimistic Execution** - Execute all transactions concurrently
2. **Conflict Detection** - Identify data conflicts using ownership tracking
3. **Re-execution** - Retry conflicting transactions until resolution
4. **Priority-based Ordering** - Deterministic conflict resolution via priority codes

## Algorithm Components

### Priority Code

The `PriorityCode` determines transaction ordering and conflict resolution.

**Structure** (41 bytes total):
```
[0]      release_bit  (0 = OWNED, 1 = DISOWNED)
[1..9]   block_height (8 bytes, big-endian)
[9..41]  hash         (32 bytes)
```

**Ordering Rules** (lower value = higher priority):
1. `release_bit`: OWNED (0) < DISOWNED (1)
2. `block_height`: Lower height has higher priority
3. `hash`: Lexicographic comparison (tiebreaker)

**Computation**:
```rust
fn compute_priority(tx: &Transaction, block: &Block) -> PriorityCode {
    let tx_hash = tx.hash();
    let block_txs_hash = block.transactions_hash();
    let combined_hash = keccak256_concat(&[
        tx_hash.as_bytes(),
        block_txs_hash.as_bytes()
    ]);
    PriorityCode::new(block.height, combined_hash)
}
```

The hash includes `block.transactions_hash()` to ensure:
- Same transaction in different blocks gets different priority
- Priority is deterministic across all nodes

### Ownership Table (Algorithm 1)

The `OwnershipTable` tracks which transaction "owns" each storage key.

**Operations**:

```rust
// Create DISOWNED entry (can be claimed by anyone)
fn new() -> OwnershipEntry {
    let mut pc = PriorityCode::new(u64::MAX, H256::zero());
    pc.release();  // Set DISOWNED
    OwnershipEntry { owner: RwLock::new(pc) }
}

// Release ownership (allow lower-priority to claim)
fn release_ownership(&self) {
    let mut owner = self.owner.write().unwrap();
    owner.release();  // Set release_bit = DISOWNED
}

// Check if transaction can claim/keep ownership
fn check_ownership(&self, who: &PriorityCode) -> bool {
    let owner = self.owner.read().unwrap();
    who <= &*owner  // who has equal or higher priority
}

// Attempt to claim ownership
fn try_set_owner(&self, who: &PriorityCode) -> bool {
    let mut owner = self.owner.write().unwrap();
    if who <= &*owner {
        *owner = who.clone();
        true  // Claimed successfully
    } else {
        false  // Higher-priority owner exists
    }
}
```

**Key Insight**: A new entry has `(DISOWNED, MAX, zero)` which has the lowest possible priority. Any transaction's priority will be `<=` this, so the first `try_set_owner` always succeeds.

### Read-Write Set

The `ReadWriteSet` records all storage accesses during transaction execution.

```rust
struct ReadWriteSet {
    reads: Vec<H256>,              // Keys read
    writes: Vec<(H256, Vec<u8>)>,  // Keys written with values
}
```

This enables conflict detection:
- **Write-Write Conflict**: Two transactions write the same key
- **Read-Write Conflict**: One transaction reads what another writes

---

## Algorithm 2: Three-Phase Execution

### Phase 1: Optimistic Parallel Execution

All transactions execute concurrently against a snapshot.

```rust
fn optimistic_execute(
    block: &Block,
    snapshot: &Snapshot,
    ownership_table: &OwnershipTable,
    executor: &dyn TransactionExecutor,
) -> Vec<ExecutedTransaction> {
    block.transactions.par_iter().map(|tx| {
        // 1. Compute priority code
        let priority = compute_priority(tx, block);

        // 2. Execute transaction
        let (rwset, result) = executor.execute(tx, snapshot);

        // 3. Claim ownership of write keys
        for (key, _) in rwset.writes() {
            let entry = ownership_table.get_or_create(key);
            entry.try_set_owner(&priority);
        }

        ExecutedTransaction { transaction: tx.clone(), priority, rwset, result }
    }).collect()
}
```

**Key Points**:
- Uses `par_iter()` from Rayon for parallelism
- All transactions see the same snapshot (isolation)
- Ownership claims may fail if higher-priority tx claims first

### Phase 2: Conflict Detection and Resolution

Repeatedly detect and resolve conflicts until all transactions confirm.

```rust
fn schedule(block: Block, state: &mut dyn StateDB, executor: &dyn TransactionExecutor)
    -> Result<ScheduleResult, SchedulerError>
{
    let ownership_table = OwnershipTable::new();
    let snapshot = state.snapshot();
    let mut confirmed = Vec::new();
    let mut reexecution_count = 0;

    // Phase 1
    let mut pending = optimistic_execute(&block, &snapshot, &ownership_table, executor);

    // Phase 2: Resolution loop
    let mut iteration = 0;
    while !pending.is_empty() {
        iteration += 1;
        if iteration > MAX_RETRIES {
            return Err(SchedulerError::MaxRetriesExceeded { ... });
        }

        // Detect conflicts
        let (passed, aborted) = detect_conflicts(pending, &ownership_table);

        // Confirm passed transactions
        for etx in passed {
            let write_keys: Vec<H256> = etx.rwset.writes()
                .iter().map(|(k, _)| *k).collect();
            ownership_table.release_all(&write_keys);
            confirmed.push(etx);
        }

        // Re-execute aborted transactions
        if !aborted.is_empty() {
            reexecution_count += aborted.len();
            pending = re_execute(aborted, &snapshot, &ownership_table, executor);
        } else {
            pending = Vec::new();
        }
    }

    // Phase 3: Commit
    // ...
}
```

#### Conflict Detection

```rust
fn detect_conflicts(
    executed: Vec<ExecutedTransaction>,
    ownership_table: &OwnershipTable,
) -> (Vec<ExecutedTransaction>, Vec<ExecutedTransaction>) {
    let mut passed = Vec::new();
    let mut aborted = Vec::new();

    for etx in executed {
        let mut conflict = false;

        // Check write conflicts: do we still own our write keys?
        for (key, _) in etx.rwset.writes() {
            let entry = ownership_table.get_or_create(key);
            if !entry.check_ownership(&etx.priority) {
                conflict = true;
                break;
            }
        }

        // Check read conflicts: did someone else write to our read keys?
        if !conflict {
            for key in etx.rwset.reads() {
                let entry = ownership_table.get_or_create(key);
                let current_owner = entry.current_owner();
                // Conflict if key is owned by someone else
                if !current_owner.is_released() && current_owner != etx.priority {
                    conflict = true;
                    break;
                }
            }
        }

        if conflict {
            aborted.push(etx);
        } else {
            passed.push(etx);
        }
    }

    (passed, aborted)
}
```

**Write Conflict**: `check_ownership` returns false if a higher-priority transaction claimed the key.

**Read Conflict**: Another transaction wrote to a key we read. This detects:
- **WAR (Write-After-Read)**: We read, they wrote
- **RAW (Read-After-Write)**: They wrote, we read

#### Re-execution

```rust
fn re_execute(
    aborted: Vec<ExecutedTransaction>,
    snapshot: &Snapshot,
    ownership_table: &OwnershipTable,
    executor: &dyn TransactionExecutor,
) -> Vec<ExecutedTransaction> {
    aborted.into_par_iter().map(|etx| {
        // Re-execute with SAME priority
        let (rwset, result) = executor.execute(&etx.transaction, snapshot);

        // Try to claim new write keys
        for (key, _) in rwset.writes() {
            let entry = ownership_table.get_or_create(key);
            entry.try_set_owner(&etx.priority);
        }

        ExecutedTransaction {
            transaction: etx.transaction,
            priority: etx.priority,  // Priority unchanged
            rwset,
            result,
        }
    }).collect()
}
```

**Key Point**: Priority remains the same across re-executions. This ensures eventual convergence.

### Phase 3: Commit Changes

Apply all confirmed writes atomically.

```rust
// Collect all writes from confirmed transactions
let mut all_writes: Vec<(H256, Vec<u8>)> = Vec::new();
for etx in &confirmed {
    for (key, value) in etx.rwset.writes() {
        all_writes.push((*key, value.clone()));
    }
}

// Atomic commit
state.commit(&all_writes);

// Compute state root
let state_root = compute_state_root(state);
```

---

## Termination Guarantees

The algorithm is guaranteed to terminate due to:

1. **Fixed Priorities**: Each transaction's priority never changes
2. **Monotonic Progress**: Each iteration either:
   - Confirms at least one transaction (reducing pending count)
   - Or all transactions conflict (but priorities resolve eventually)
3. **MAX_RETRIES Bound**: Hard limit of 100 iterations prevents infinite loops

### Worst Case Analysis

Consider N transactions all conflicting on the same key:
- Iteration 1: Highest priority wins, others abort
- Iteration 2: Second highest wins, remaining abort
- ...
- Iteration N: Last transaction confirms

Maximum iterations = N (number of transactions)

The MAX_RETRIES = 100 is sufficient for reasonable block sizes.

---

## Determinism Guarantees

All nodes produce identical results because:

1. **Deterministic Priority**:
   - `PriorityCode = (block.height, keccak256(tx_hash || block_txs_hash))`
   - Same inputs always produce same priority

2. **Deterministic Conflict Resolution**:
   - Higher priority always wins
   - Same conflicts always resolve the same way

3. **Deterministic Execution Order**:
   - Within conflict groups, transactions process by priority
   - Final confirmed order is deterministic

4. **Snapshot Isolation**:
   - All transactions see identical snapshot
   - No visibility of concurrent writes

---

## Implementation Notes

### Thread Safety

- `OwnershipTable` uses `RwLock<HashMap>` for concurrent access
- `OwnershipEntry` uses `RwLock<PriorityCode>` per key
- Double-checked locking in `get_or_create` prevents races
- No deadlock: entries locked independently, simple hierarchy

### Performance Considerations

1. **Parallelism**: Rayon handles work stealing for load balancing
2. **Lock Contention**: RwLock allows concurrent readers
3. **Memory**: Snapshot clones state (trade-off: isolation vs. memory)
4. **Fast Path**: Most transactions don't conflict in practice

### Error Handling

```rust
// Execution failure doesn't abort scheduling
// Transaction is confirmed with Failed result
ExecutionResult::Failed { reason: String }

// Only fatal errors abort scheduling
SchedulerError::MaxRetriesExceeded { tx_hash, attempts }
SchedulerError::InvalidBlock(String)
SchedulerError::StateError(StateError)
```

---

## Diagram: Execution Flow

```
Block with Transactions [T1, T2, T3, T4]
           |
           v
+---------------------------+
| Phase 1: Parallel Execute |
| T1, T2, T3, T4 in parallel|
| Against snapshot S0       |
+---------------------------+
           |
           v
+---------------------------+
| Ownership Claims          |
| T1 claims K1              |
| T2 claims K2, K3          |
| T3 claims K1 (conflict!)  |
| T4 claims K4              |
+---------------------------+
           |
           v
+---------------------------+
| Phase 2: Detect Conflicts |
| T1: owns K1 -> pass       |
| T2: owns K2,K3 -> pass    |
| T3: lost K1 -> abort      |
| T4: owns K4 -> pass       |
+---------------------------+
           |
           v
+---------------------------+
| Confirm: T1, T2, T4       |
| Release: K1, K2, K3, K4   |
| Re-execute: T3            |
+---------------------------+
           |
           v
+---------------------------+
| T3 re-executes            |
| Claims K1 (now free)      |
| No conflict -> pass       |
+---------------------------+
           |
           v
+---------------------------+
| Confirm: T3               |
| All done!                 |
+---------------------------+
           |
           v
+---------------------------+
| Phase 3: Commit           |
| Apply all writes to state |
| Compute state root        |
+---------------------------+
           |
           v
    ScheduleResult {
        confirmed: [T1, T2, T4, T3],
        reexecution_count: 1,
        ...
    }
```

---

## Summary

The Seamless Scheduling algorithm achieves:

- **Parallelism**: Transactions execute concurrently
- **Determinism**: All nodes produce identical results
- **Correctness**: No data races or lost writes
- **Efficiency**: Most transactions confirm in first pass
- **Bounded**: Maximum iterations guaranteed by MAX_RETRIES

Key implementation files:
- `bach-types/src/lib.rs` - PriorityCode, ReadWriteSet
- `bach-state/src/lib.rs` - OwnershipEntry, OwnershipTable
- `bach-scheduler/src/lib.rs` - SeamlessScheduler
