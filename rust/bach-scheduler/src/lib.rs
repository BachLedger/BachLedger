//! BachLedger Scheduler
//!
//! Transaction scheduling for parallel execution:
//! - `SeamlessScheduler`: Implementation of Algorithm 2 (Seamless Scheduling)
//! - `TransactionExecutor`: Trait for executing transactions
//! - `Scheduler`: Trait for scheduling blocks

use bach_crypto::keccak256_concat;
use bach_primitives::H256;
use bach_state::{OwnershipTable, Snapshot, StateDB, StateError};
use bach_types::{Block, PriorityCode, ReadWriteSet, Transaction};
use rayon::prelude::*;
use std::sync::Arc;

/// Default number of worker threads
pub const DEFAULT_THREAD_COUNT: usize = 4;

/// Maximum re-execution attempts per transaction
pub const MAX_RETRIES: usize = 100;

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

impl From<StateError> for SchedulerError {
    fn from(err: StateError) -> Self {
        SchedulerError::StateError(err)
    }
}

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
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success { .. })
    }
}

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
    pub fn hash(&self) -> H256 {
        self.transaction.hash()
    }
}

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

/// Interface for executing transactions.
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

/// Interface for transaction scheduling.
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

/// Implementation of Seamless Scheduling algorithm (Algorithm 2 from the paper).
///
/// This scheduler implements the core BachLedger innovation:
/// 1. Optimistic parallel execution of transactions
/// 2. Conflict detection via OwnershipTable
/// 3. Re-execution of conflicting transactions
/// 4. Priority-based ordering ensures deterministic results
pub struct SeamlessScheduler {
    /// Number of parallel execution threads (reserved for future use)
    #[allow(dead_code)]
    thread_count: usize,
}

impl SeamlessScheduler {
    /// Creates a new scheduler with the specified thread count.
    ///
    /// # Arguments
    /// * `thread_count` - Number of parallel execution threads
    pub fn new(thread_count: usize) -> Self {
        // Configure rayon thread pool
        let thread_count = if thread_count == 0 { 1 } else { thread_count };
        Self { thread_count }
    }

    /// Creates a scheduler with default thread count.
    pub fn with_default_threads() -> Self {
        Self::new(DEFAULT_THREAD_COUNT)
    }

    /// Computes the priority code for a transaction in a block.
    fn compute_priority(tx: &Transaction, block: &Block) -> PriorityCode {
        let tx_hash = tx.hash();
        let block_txs_hash = block.transactions_hash();
        let combined_hash = keccak256_concat(&[tx_hash.as_bytes(), block_txs_hash.as_bytes()]);
        PriorityCode::new(block.height, combined_hash)
    }

    /// Optimistically executes all transactions in parallel (Phase 1).
    fn optimistic_execute(
        &self,
        block: &Block,
        snapshot: &Snapshot,
        ownership_table: &OwnershipTable,
        executor: &dyn TransactionExecutor,
    ) -> Vec<ExecutedTransaction> {
        block
            .transactions
            .par_iter()
            .map(|tx| {
                // Compute priority code
                let priority = Self::compute_priority(tx, block);

                // Execute transaction
                let (rwset, result) = executor.execute(tx, snapshot);

                // Try to claim ownership of write keys
                for (key, _) in rwset.writes() {
                    let entry = ownership_table.get_or_create(key);
                    entry.try_set_owner(&priority);
                }

                ExecutedTransaction {
                    transaction: tx.clone(),
                    priority,
                    rwset,
                    result,
                }
            })
            .collect()
    }

    /// Detects conflicts and partitions transactions (Phase 2).
    fn detect_conflicts(
        executed: Vec<ExecutedTransaction>,
        ownership_table: &OwnershipTable,
    ) -> (Vec<ExecutedTransaction>, Vec<ExecutedTransaction>) {
        let mut passed = Vec::new();
        let mut aborted = Vec::new();

        for etx in executed {
            let mut conflict = false;

            // Check write set ownership
            for (key, _) in etx.rwset.writes() {
                let entry = ownership_table.get_or_create(key);
                if !entry.check_ownership(&etx.priority) {
                    conflict = true;
                    break;
                }
            }

            // Check read set: abort if another transaction has written to a key we read
            // A reader conflicts if any writer (other than itself) owns the key
            if !conflict {
                for key in etx.rwset.reads() {
                    let entry = ownership_table.get_or_create(key);
                    let current_owner = entry.current_owner();
                    // Conflict if someone else owns this key (they wrote to it)
                    // We check: is the owner NOT released AND NOT us?
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

    /// Re-executes aborted transactions (Phase 2 continued).
    fn re_execute(
        aborted: Vec<ExecutedTransaction>,
        snapshot: &Snapshot,
        ownership_table: &OwnershipTable,
        executor: &dyn TransactionExecutor,
    ) -> Vec<ExecutedTransaction> {
        aborted
            .into_par_iter()
            .map(|etx| {
                // Re-execute with same priority
                let (rwset, result) = executor.execute(&etx.transaction, snapshot);

                // Try to claim ownership of new write keys
                for (key, _) in rwset.writes() {
                    let entry = ownership_table.get_or_create(key);
                    entry.try_set_owner(&etx.priority);
                }

                ExecutedTransaction {
                    transaction: etx.transaction,
                    priority: etx.priority,
                    rwset,
                    result,
                }
            })
            .collect()
    }
}

impl Default for SeamlessScheduler {
    fn default() -> Self {
        Self::with_default_threads()
    }
}

impl Scheduler for SeamlessScheduler {
    fn schedule(
        &self,
        block: Block,
        state: &mut dyn StateDB,
        executor: &dyn TransactionExecutor,
    ) -> Result<ScheduleResult, SchedulerError> {
        // Create ownership table for conflict tracking
        let ownership_table = Arc::new(OwnershipTable::new());

        // Create snapshot for consistent reads
        let snapshot = state.snapshot();

        // Track confirmed transactions and re-execution count
        let mut confirmed: Vec<ExecutedTransaction> = Vec::new();
        let mut reexecution_count: usize = 0;

        // Phase 1: Optimistic parallel execution
        let mut pending = self.optimistic_execute(&block, &snapshot, &ownership_table, executor);

        // Phase 2: Conflict detection and resolution loop
        let mut iteration = 0;
        while !pending.is_empty() {
            iteration += 1;
            if iteration > MAX_RETRIES {
                // Find a transaction that's stuck
                if let Some(etx) = pending.first() {
                    return Err(SchedulerError::MaxRetriesExceeded {
                        tx_hash: etx.hash(),
                        attempts: MAX_RETRIES,
                    });
                }
            }

            // Detect conflicts
            let (passed, aborted) = Self::detect_conflicts(pending, &ownership_table);

            // Release ownership for confirmed transactions and add to confirmed list
            for etx in passed {
                // Release ownership of write keys
                let write_keys: Vec<H256> = etx.rwset.writes().iter().map(|(k, _)| *k).collect();
                ownership_table.release_all(&write_keys);

                confirmed.push(etx);
            }

            // Re-execute aborted transactions
            if !aborted.is_empty() {
                reexecution_count += aborted.len();
                pending = Self::re_execute(aborted, &snapshot, &ownership_table, executor);
            } else {
                pending = Vec::new();
            }
        }

        // Phase 3: Commit all writes to state
        let mut all_writes: Vec<(H256, Vec<u8>)> = Vec::new();
        for etx in &confirmed {
            for (key, value) in etx.rwset.writes() {
                all_writes.push((*key, value.clone()));
            }
        }
        state.commit(&all_writes);

        // Compute state root (simplified - use snapshot hash)
        let state_root = {
            let final_snapshot = state.snapshot();
            // Use keys hash as simple state root
            let keys = state.keys();
            if keys.is_empty() {
                H256::zero()
            } else {
                let mut data = Vec::new();
                for key in keys {
                    data.extend_from_slice(key.as_bytes());
                    if let Some(value) = final_snapshot.get(&key) {
                        data.extend_from_slice(&value);
                    }
                }
                bach_crypto::keccak256(&data)
            }
        };

        Ok(ScheduleResult {
            confirmed,
            block_hash: block.hash(),
            state_root,
            reexecution_count,
        })
    }
}

// Ensure thread safety
unsafe impl Send for SeamlessScheduler {}
unsafe impl Sync for SeamlessScheduler {}
