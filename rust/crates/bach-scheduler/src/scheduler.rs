//! Main scheduler implementation
//!
//! Orchestrates parallel transaction execution using the Seamless Scheduling algorithm.

use crate::dependency::DependencyGraph;
use crate::error::{SchedulerError, SchedulerResult};
use crate::ownership::OwnershipTable;
use crate::rw_set::RWSet;
use crate::state_key::TxId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// A batch of transactions that can execute in parallel
#[derive(Clone, Debug)]
pub struct ExecutionBatch {
    /// Batch index (0 = first batch)
    pub batch_index: usize,
    /// Transactions in this batch
    pub transactions: Vec<TxId>,
}

impl ExecutionBatch {
    /// Create a new execution batch
    pub fn new(batch_index: usize, transactions: Vec<TxId>) -> Self {
        Self {
            batch_index,
            transactions,
        }
    }

    /// Get number of transactions in batch
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}

/// Result of scheduling analysis
#[derive(Clone, Debug)]
pub struct ScheduleResult {
    /// Execution batches in order
    pub batches: Vec<ExecutionBatch>,
    /// Total number of transactions
    pub total_transactions: usize,
    /// Maximum parallelism achieved (max batch size)
    pub max_parallelism: usize,
    /// Number of dependency edges
    pub dependency_count: usize,
}

impl ScheduleResult {
    /// Calculate parallelism ratio (higher = more parallel)
    ///
    /// Returns the ratio of total transactions to number of batches.
    /// A ratio of 1.0 means fully serial, higher means more parallel.
    pub fn parallelism_ratio(&self) -> f64 {
        if self.batches.is_empty() {
            return 0.0;
        }
        self.total_transactions as f64 / self.batches.len() as f64
    }

    /// Calculate speedup estimate assuming perfect parallel execution
    pub fn estimated_speedup(&self) -> f64 {
        if self.batches.is_empty() {
            return 1.0;
        }
        self.total_transactions as f64 / self.batches.len() as f64
    }
}

/// Transaction info for scheduling
#[derive(Clone, Debug)]
pub struct TxInfo {
    /// Transaction ID
    pub tx_id: TxId,
    /// Read/write set (predicted or actual)
    pub rw_set: RWSet,
    /// Number of retries
    pub retry_count: u32,
}

impl TxInfo {
    /// Create new transaction info
    pub fn new(tx_id: TxId, rw_set: RWSet) -> Self {
        Self {
            tx_id,
            rw_set,
            retry_count: 0,
        }
    }
}

/// Seamless Scheduler
///
/// Implements the core scheduling algorithm that analyzes transactions
/// and generates parallel execution batches.
pub struct Scheduler {
    /// Ownership table for tracking state ownership
    ownership: Arc<OwnershipTable>,
    /// Transaction information
    transactions: RwLock<HashMap<TxId, TxInfo>>,
    /// Maximum retry count before aborting
    max_retries: u32,
}

impl Scheduler {
    /// Create a new scheduler with default settings
    pub fn new() -> Self {
        Self {
            ownership: Arc::new(OwnershipTable::new()),
            transactions: RwLock::new(HashMap::new()),
            max_retries: 3,
        }
    }

    /// Create scheduler with custom max retries
    pub fn with_max_retries(max_retries: u32) -> Self {
        Self {
            ownership: Arc::new(OwnershipTable::new()),
            transactions: RwLock::new(HashMap::new()),
            max_retries,
        }
    }

    /// Get reference to ownership table
    pub fn ownership_table(&self) -> Arc<OwnershipTable> {
        Arc::clone(&self.ownership)
    }

    /// Register a transaction with its predicted RW set
    pub fn register_transaction(&self, tx_id: TxId, rw_set: RWSet) {
        let info = TxInfo::new(tx_id, rw_set);
        self.transactions.write().insert(tx_id, info);
    }

    /// Register multiple transactions
    pub fn register_transactions(&self, txs: impl IntoIterator<Item = (TxId, RWSet)>) {
        let mut transactions = self.transactions.write();
        for (tx_id, rw_set) in txs {
            transactions.insert(tx_id, TxInfo::new(tx_id, rw_set));
        }
    }

    /// Update RW set for a transaction (e.g., after execution reveals actual accesses)
    pub fn update_rw_set(&self, tx_id: TxId, rw_set: RWSet) -> SchedulerResult<()> {
        let mut transactions = self.transactions.write();
        if let Some(info) = transactions.get_mut(&tx_id) {
            info.rw_set = rw_set;
            Ok(())
        } else {
            Err(SchedulerError::TxNotFound(tx_id))
        }
    }

    /// Increment retry count for a transaction
    pub fn increment_retry(&self, tx_id: TxId) -> SchedulerResult<u32> {
        let mut transactions = self.transactions.write();
        if let Some(info) = transactions.get_mut(&tx_id) {
            info.retry_count += 1;
            if info.retry_count > self.max_retries {
                return Err(SchedulerError::MaxRetriesExceeded(tx_id));
            }
            Ok(info.retry_count)
        } else {
            Err(SchedulerError::TxNotFound(tx_id))
        }
    }

    /// Analyze transactions and generate schedule
    ///
    /// This is the main entry point for the scheduling algorithm.
    pub fn schedule(&self) -> SchedulerResult<ScheduleResult> {
        let transactions = self.transactions.read();
        let mut rw_sets: Vec<(TxId, RWSet)> = transactions
            .values()
            .map(|info| (info.tx_id, info.rw_set.clone()))
            .collect();
        drop(transactions);

        // Sort by TxId to ensure deterministic dependency analysis
        rw_sets.sort_by_key(|(tx_id, _)| *tx_id);

        self.schedule_from_rw_sets(&rw_sets)
    }

    /// Generate schedule from RW sets directly
    pub fn schedule_from_rw_sets(&self, rw_sets: &[(TxId, RWSet)]) -> SchedulerResult<ScheduleResult> {
        if rw_sets.is_empty() {
            return Ok(ScheduleResult {
                batches: Vec::new(),
                total_transactions: 0,
                max_parallelism: 0,
                dependency_count: 0,
            });
        }

        // Build dependency graph
        let graph = DependencyGraph::build(rw_sets);

        // Generate parallel batches
        let batch_txs = graph.generate_batches()?;

        let batches: Vec<ExecutionBatch> = batch_txs
            .into_iter()
            .enumerate()
            .map(|(idx, txs)| ExecutionBatch::new(idx, txs))
            .collect();

        let max_parallelism = batches.iter().map(|b| b.len()).max().unwrap_or(0);

        Ok(ScheduleResult {
            batches,
            total_transactions: rw_sets.len(),
            max_parallelism,
            dependency_count: graph.edge_count(),
        })
    }

    /// Try to acquire ownership for a transaction's write set
    pub fn acquire_ownership(&self, tx_id: TxId) -> SchedulerResult<()> {
        let transactions = self.transactions.read();
        let info = transactions
            .get(&tx_id)
            .ok_or(SchedulerError::TxNotFound(tx_id))?;

        for key in &info.rw_set.writes {
            if let Err(owner) = self.ownership.try_acquire(key, tx_id) {
                return Err(SchedulerError::OwnershipConflict {
                    owner,
                    requester: tx_id,
                });
            }
        }

        Ok(())
    }

    /// Release ownership for a transaction
    pub fn release_ownership(&self, tx_id: TxId) {
        self.ownership.release_all(tx_id);
    }

    /// Clear all state (for new block)
    pub fn clear(&self) {
        self.transactions.write().clear();
        self.ownership.clear();
    }

    /// Get number of registered transactions
    pub fn transaction_count(&self) -> usize {
        self.transactions.read().len()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_key::StateKey;
    use bach_primitives::{Address, H256};
    use std::sync::Arc;
    use std::thread;

    fn make_key(id: u8) -> StateKey {
        StateKey::new(
            Address::from_bytes([id; 20]),
            H256::from_bytes([id; 32]),
        )
    }

    #[test]
    fn test_empty_schedule() {
        let scheduler = Scheduler::new();
        let result = scheduler.schedule().unwrap();

        assert_eq!(result.total_transactions, 0);
        assert!(result.batches.is_empty());
    }

    #[test]
    fn test_single_transaction() {
        let scheduler = Scheduler::new();
        let mut rw = RWSet::new();
        rw.record_write(make_key(1));

        scheduler.register_transaction(TxId::new(0), rw);
        let result = scheduler.schedule().unwrap();

        assert_eq!(result.total_transactions, 1);
        assert_eq!(result.batches.len(), 1);
        assert_eq!(result.batches[0].len(), 1);
    }

    #[test]
    fn test_parallel_transactions() {
        let scheduler = Scheduler::new();

        // Three independent transactions
        for i in 0..3 {
            let mut rw = RWSet::new();
            rw.record_write(make_key(i));
            scheduler.register_transaction(TxId::new(i as u32), rw);
        }

        let result = scheduler.schedule().unwrap();

        assert_eq!(result.total_transactions, 3);
        assert_eq!(result.batches.len(), 1); // All in one batch
        assert_eq!(result.max_parallelism, 3);
    }

    #[test]
    fn test_serial_transactions() {
        let scheduler = Scheduler::new();
        let key = make_key(1);

        // Three transactions all writing same key
        for i in 0..3 {
            let mut rw = RWSet::new();
            rw.record_write(key.clone());
            scheduler.register_transaction(TxId::new(i), rw);
        }

        let result = scheduler.schedule().unwrap();

        assert_eq!(result.total_transactions, 3);
        assert_eq!(result.batches.len(), 3); // Each in separate batch
        assert_eq!(result.max_parallelism, 1);
    }

    #[test]
    fn test_mixed_dependencies() {
        let scheduler = Scheduler::new();
        let key1 = make_key(1);
        let key2 = make_key(2);

        // tx0: write key1
        let mut rw0 = RWSet::new();
        rw0.record_write(key1.clone());
        scheduler.register_transaction(TxId::new(0), rw0);

        // tx1: write key2
        let mut rw1 = RWSet::new();
        rw1.record_write(key2.clone());
        scheduler.register_transaction(TxId::new(1), rw1);

        // tx2: read key1 (depends on tx0)
        let mut rw2 = RWSet::new();
        rw2.record_read(key1.clone());
        scheduler.register_transaction(TxId::new(2), rw2);

        // tx3: read key2 (depends on tx1)
        let mut rw3 = RWSet::new();
        rw3.record_read(key2.clone());
        scheduler.register_transaction(TxId::new(3), rw3);

        let result = scheduler.schedule().unwrap();

        assert_eq!(result.total_transactions, 4);
        assert_eq!(result.batches.len(), 2);
        // Batch 0: tx0, tx1
        // Batch 1: tx2, tx3
        assert_eq!(result.max_parallelism, 2);
    }

    #[test]
    fn test_ownership_acquisition() {
        let scheduler = Scheduler::new();
        let key = make_key(1);

        let mut rw = RWSet::new();
        rw.record_write(key.clone());
        scheduler.register_transaction(TxId::new(0), rw.clone());

        // First acquisition should succeed
        assert!(scheduler.acquire_ownership(TxId::new(0)).is_ok());

        // Register second transaction with same key
        scheduler.register_transaction(TxId::new(1), rw);

        // Second acquisition should fail
        let err = scheduler.acquire_ownership(TxId::new(1)).unwrap_err();
        matches!(err, SchedulerError::OwnershipConflict { .. });

        // Release first, then second should succeed
        scheduler.release_ownership(TxId::new(0));
        assert!(scheduler.acquire_ownership(TxId::new(1)).is_ok());
    }

    #[test]
    fn test_retry_limit() {
        let scheduler = Scheduler::with_max_retries(2);

        let rw = RWSet::new();
        scheduler.register_transaction(TxId::new(0), rw);

        assert_eq!(scheduler.increment_retry(TxId::new(0)).unwrap(), 1);
        assert_eq!(scheduler.increment_retry(TxId::new(0)).unwrap(), 2);

        // Third retry should fail
        let err = scheduler.increment_retry(TxId::new(0)).unwrap_err();
        matches!(err, SchedulerError::MaxRetriesExceeded(_));
    }

    #[test]
    fn test_schedule_result_metrics() {
        let scheduler = Scheduler::new();

        // 4 transactions in 2 batches
        for i in 0..4 {
            let mut rw = RWSet::new();
            rw.record_write(make_key(i % 2)); // 0,1,0,1 - creates dependencies
            scheduler.register_transaction(TxId::new(i as u32), rw);
        }

        let result = scheduler.schedule().unwrap();

        assert!(result.parallelism_ratio() >= 1.0);
        assert!(result.estimated_speedup() >= 1.0);
    }

    #[test]
    fn test_clear_scheduler() {
        let scheduler = Scheduler::new();

        let mut rw = RWSet::new();
        rw.record_write(make_key(1));
        scheduler.register_transaction(TxId::new(0), rw);
        scheduler.acquire_ownership(TxId::new(0)).unwrap();

        assert_eq!(scheduler.transaction_count(), 1);
        assert!(!scheduler.ownership_table().is_empty());

        scheduler.clear();

        assert_eq!(scheduler.transaction_count(), 0);
        assert!(scheduler.ownership_table().is_empty());
    }

    // ==================== Additional Scheduler Tests ====================

    #[test]
    fn test_scheduler_default() {
        let scheduler = Scheduler::default();
        assert_eq!(scheduler.transaction_count(), 0);
    }

    #[test]
    fn test_register_transactions_batch() {
        let scheduler = Scheduler::new();

        let txs: Vec<(TxId, RWSet)> = (0..5)
            .map(|i| {
                let mut rw = RWSet::new();
                rw.record_write(make_key(i));
                (TxId::new(i as u32), rw)
            })
            .collect();

        scheduler.register_transactions(txs);

        assert_eq!(scheduler.transaction_count(), 5);
    }

    #[test]
    fn test_update_rw_set() {
        let scheduler = Scheduler::new();

        let mut rw = RWSet::new();
        rw.record_write(make_key(1));
        scheduler.register_transaction(TxId::new(0), rw);

        // Update with new RW set
        let mut new_rw = RWSet::new();
        new_rw.record_write(make_key(2));
        new_rw.record_read(make_key(3));

        assert!(scheduler.update_rw_set(TxId::new(0), new_rw).is_ok());
    }

    #[test]
    fn test_update_rw_set_nonexistent() {
        let scheduler = Scheduler::new();

        let rw = RWSet::new();
        let result = scheduler.update_rw_set(TxId::new(99), rw);

        assert!(result.is_err());
        matches!(result.unwrap_err(), SchedulerError::TxNotFound(_));
    }

    #[test]
    fn test_increment_retry_nonexistent() {
        let scheduler = Scheduler::new();

        let result = scheduler.increment_retry(TxId::new(99));
        assert!(result.is_err());
        matches!(result.unwrap_err(), SchedulerError::TxNotFound(_));
    }

    #[test]
    fn test_acquire_ownership_nonexistent() {
        let scheduler = Scheduler::new();

        let result = scheduler.acquire_ownership(TxId::new(99));
        assert!(result.is_err());
        matches!(result.unwrap_err(), SchedulerError::TxNotFound(_));
    }

    #[test]
    fn test_schedule_from_rw_sets_directly() {
        let scheduler = Scheduler::new();

        let mut rw_sets = Vec::new();

        // tx0 writes key1
        let mut rw0 = RWSet::new();
        rw0.record_write(make_key(1));
        rw_sets.push((TxId::new(0), rw0));

        // tx1 reads key1
        let mut rw1 = RWSet::new();
        rw1.record_read(make_key(1));
        rw_sets.push((TxId::new(1), rw1));

        let result = scheduler.schedule_from_rw_sets(&rw_sets).unwrap();

        assert_eq!(result.total_transactions, 2);
        assert_eq!(result.batches.len(), 2);
        assert_eq!(result.dependency_count, 1);
    }

    // ==================== ExecutionBatch Tests ====================

    #[test]
    fn test_execution_batch_new() {
        let txs = vec![TxId::new(0), TxId::new(1), TxId::new(2)];
        let batch = ExecutionBatch::new(0, txs);

        assert_eq!(batch.batch_index, 0);
        assert_eq!(batch.len(), 3);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_execution_batch_empty() {
        let batch = ExecutionBatch::new(0, Vec::new());

        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    // ==================== ScheduleResult Tests ====================

    #[test]
    fn test_schedule_result_parallelism_ratio_empty() {
        let result = ScheduleResult {
            batches: Vec::new(),
            total_transactions: 0,
            max_parallelism: 0,
            dependency_count: 0,
        };

        assert_eq!(result.parallelism_ratio(), 0.0);
    }

    #[test]
    fn test_schedule_result_parallelism_ratio_single_batch() {
        let result = ScheduleResult {
            batches: vec![ExecutionBatch::new(0, vec![TxId::new(0), TxId::new(1), TxId::new(2)])],
            total_transactions: 3,
            max_parallelism: 3,
            dependency_count: 0,
        };

        assert_eq!(result.parallelism_ratio(), 3.0);
    }

    #[test]
    fn test_schedule_result_parallelism_ratio_serial() {
        let batches: Vec<ExecutionBatch> = (0..3)
            .map(|i| ExecutionBatch::new(i, vec![TxId::new(i as u32)]))
            .collect();

        let result = ScheduleResult {
            batches,
            total_transactions: 3,
            max_parallelism: 1,
            dependency_count: 2,
        };

        assert_eq!(result.parallelism_ratio(), 1.0);
    }

    #[test]
    fn test_schedule_result_estimated_speedup() {
        let result = ScheduleResult {
            batches: vec![
                ExecutionBatch::new(0, vec![TxId::new(0), TxId::new(1)]),
                ExecutionBatch::new(1, vec![TxId::new(2), TxId::new(3)]),
            ],
            total_transactions: 4,
            max_parallelism: 2,
            dependency_count: 2,
        };

        assert_eq!(result.estimated_speedup(), 2.0);
    }

    #[test]
    fn test_schedule_result_estimated_speedup_empty() {
        let result = ScheduleResult {
            batches: Vec::new(),
            total_transactions: 0,
            max_parallelism: 0,
            dependency_count: 0,
        };

        assert_eq!(result.estimated_speedup(), 1.0);
    }

    // ==================== TxInfo Tests ====================

    #[test]
    fn test_tx_info_new() {
        let mut rw = RWSet::new();
        rw.record_write(make_key(1));

        let info = TxInfo::new(TxId::new(0), rw);

        assert_eq!(info.tx_id, TxId::new(0));
        assert_eq!(info.retry_count, 0);
        assert!(!info.rw_set.is_empty());
    }

    // ==================== Concurrent Scheduler Tests ====================

    #[test]
    fn test_concurrent_registration() {
        let scheduler = Arc::new(Scheduler::new());
        let mut handles = vec![];

        // 10 threads registering transactions
        for i in 0..10 {
            let scheduler = Arc::clone(&scheduler);
            let handle = thread::spawn(move || {
                let mut rw = RWSet::new();
                rw.record_write(make_key(i));
                scheduler.register_transaction(TxId::new(i as u32), rw);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(scheduler.transaction_count(), 10);
    }

    #[test]
    fn test_concurrent_ownership_different_keys() {
        let scheduler = Arc::new(Scheduler::new());

        // Register transactions first
        for i in 0..10 {
            let mut rw = RWSet::new();
            rw.record_write(make_key(i));
            scheduler.register_transaction(TxId::new(i as u32), rw);
        }

        let mut handles = vec![];

        // 10 threads acquiring ownership of different keys
        for i in 0..10 {
            let scheduler = Arc::clone(&scheduler);
            let handle = thread::spawn(move || {
                scheduler.acquire_ownership(TxId::new(i)).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(scheduler.ownership_table().len(), 10);
    }

    // ==================== Complex Scheduling Scenarios ====================

    #[test]
    fn test_erc20_batch_transfers() {
        // Simulate multiple ERC20 transfers
        // Each transfer: read sender balance, write sender balance, write recipient balance
        let scheduler = Scheduler::new();

        // 5 senders, each transferring to a unique recipient
        for i in 0..5 {
            let sender = Address::from_bytes([(i * 2) as u8; 20]);
            let recipient = Address::from_bytes([(i * 2 + 1) as u8; 20]);

            let mut rw = RWSet::new();
            rw.record_read(StateKey::balance(sender));
            rw.record_write(StateKey::balance(sender));
            rw.record_write(StateKey::balance(recipient));

            scheduler.register_transaction(TxId::new(i as u32), rw);
        }

        let result = scheduler.schedule().unwrap();

        // All 5 transfers are independent (different senders/recipients)
        assert_eq!(result.total_transactions, 5);
        assert_eq!(result.batches.len(), 1);
        assert_eq!(result.max_parallelism, 5);
    }

    #[test]
    fn test_contended_hot_key() {
        // Multiple transactions all accessing the same "hot" storage slot
        let scheduler = Scheduler::new();
        let hot_key = make_key(42);

        for i in 0..10 {
            let mut rw = RWSet::new();
            rw.record_read(hot_key.clone());
            rw.record_write(hot_key.clone());
            scheduler.register_transaction(TxId::new(i), rw);
        }

        let result = scheduler.schedule().unwrap();

        // All transactions must be serial
        assert_eq!(result.total_transactions, 10);
        assert_eq!(result.batches.len(), 10);
        assert_eq!(result.max_parallelism, 1);
    }

    #[test]
    fn test_mixed_hot_and_cold_keys() {
        let scheduler = Scheduler::new();
        let hot_key = make_key(0);

        // tx0, tx2, tx4: access hot key (serial)
        // tx1, tx3: access unique cold keys (can parallel with each other)
        for i in 0..5 {
            let mut rw = RWSet::new();
            if i % 2 == 0 {
                rw.record_write(hot_key.clone());
            } else {
                rw.record_write(make_key((i + 10) as u8));
            }
            scheduler.register_transaction(TxId::new(i as u32), rw);
        }

        let result = scheduler.schedule().unwrap();

        // tx0, tx2, tx4 must be serial
        // tx1 can run with tx0, tx3 can run with tx2
        assert!(result.batches.len() >= 3);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_many_transactions() {
        let scheduler = Scheduler::new();

        // 100 independent transactions
        for i in 0..100u8 {
            let mut rw = RWSet::new();
            rw.record_write(make_key(i));
            scheduler.register_transaction(TxId::new(i as u32), rw);
        }

        let result = scheduler.schedule().unwrap();

        assert_eq!(result.total_transactions, 100);
        assert_eq!(result.batches.len(), 1);
        assert_eq!(result.max_parallelism, 100);
    }

    #[test]
    fn test_long_dependency_chain() {
        // Use schedule_from_rw_sets with ordered input for deterministic results
        let scheduler = Scheduler::new();

        let mut rw_sets = Vec::new();
        // tx0 -> tx1 -> tx2 -> ... -> tx19
        for i in 0..20 {
            let mut rw = RWSet::new();
            if i > 0 {
                rw.record_read(make_key((i - 1) as u8));
            }
            rw.record_write(make_key(i as u8));
            rw_sets.push((TxId::new(i as u32), rw));
        }

        let result = scheduler.schedule_from_rw_sets(&rw_sets).unwrap();

        assert_eq!(result.total_transactions, 20);
        assert_eq!(result.batches.len(), 20);
        assert_eq!(result.max_parallelism, 1);
        assert_eq!(result.dependency_count, 19);
    }

    #[test]
    fn test_ownership_multiple_keys() {
        let scheduler = Scheduler::new();

        // Transaction writes multiple keys
        let mut rw = RWSet::new();
        for i in 0..5 {
            rw.record_write(make_key(i));
        }
        scheduler.register_transaction(TxId::new(0), rw);

        // Acquire ownership - should acquire all 5 keys
        scheduler.acquire_ownership(TxId::new(0)).unwrap();

        assert_eq!(scheduler.ownership_table().len(), 5);

        // Release - should release all 5 keys
        scheduler.release_ownership(TxId::new(0));

        assert!(scheduler.ownership_table().is_empty());
    }

    #[test]
    fn test_retry_count_tracking() {
        let scheduler = Scheduler::with_max_retries(5);

        let rw = RWSet::new();
        scheduler.register_transaction(TxId::new(0), rw);

        for expected in 1..=5 {
            let count = scheduler.increment_retry(TxId::new(0)).unwrap();
            assert_eq!(count, expected);
        }

        // 6th retry should fail
        assert!(scheduler.increment_retry(TxId::new(0)).is_err());
    }

    #[test]
    fn test_schedule_preserves_determinism() {
        // Same inputs should produce same outputs
        let build_scheduler = || {
            let scheduler = Scheduler::new();
            let key1 = make_key(1);
            let key2 = make_key(2);

            let mut rw0 = RWSet::new();
            rw0.record_write(key1.clone());
            scheduler.register_transaction(TxId::new(0), rw0);

            let mut rw1 = RWSet::new();
            rw1.record_read(key1.clone());
            rw1.record_write(key2.clone());
            scheduler.register_transaction(TxId::new(1), rw1);

            let mut rw2 = RWSet::new();
            rw2.record_read(key2.clone());
            scheduler.register_transaction(TxId::new(2), rw2);

            scheduler
        };

        let result1 = build_scheduler().schedule().unwrap();
        let result2 = build_scheduler().schedule().unwrap();

        assert_eq!(result1.total_transactions, result2.total_transactions);
        assert_eq!(result1.batches.len(), result2.batches.len());
        assert_eq!(result1.dependency_count, result2.dependency_count);
    }
}
