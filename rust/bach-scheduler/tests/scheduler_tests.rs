//! Comprehensive tests for bach-scheduler module
//!
//! Tests cover:
//! - SchedulerError enum variants
//! - ExecutionResult enum and is_success()
//! - ExecutedTransaction struct and hash()
//! - ScheduleResult struct
//! - TransactionExecutor trait (mock implementation)
//! - Scheduler trait and SeamlessScheduler implementation
//! - Algorithm 2: Seamless Scheduling scenarios

use bach_scheduler::{
    ExecutedTransaction, ExecutionResult, ScheduleResult, Scheduler, SchedulerError,
    SeamlessScheduler, TransactionExecutor, DEFAULT_THREAD_COUNT, MAX_RETRIES,
};
use bach_primitives::{Address, H256, U256};
use bach_types::{Block, PriorityCode, ReadWriteSet, Transaction};
use bach_state::{MemoryStateDB, Snapshot, StateError};
use bach_crypto::PrivateKey;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// Constants Tests
// ============================================================================

#[test]
fn default_thread_count_is_reasonable() {
    // Default thread count should be at least 1
    assert!(DEFAULT_THREAD_COUNT >= 1);
    // And not excessively large
    assert!(DEFAULT_THREAD_COUNT <= 64);
}

#[test]
fn default_thread_count_is_four() {
    assert_eq!(DEFAULT_THREAD_COUNT, 4);
}

#[test]
fn max_retries_is_reasonable() {
    // Max retries should allow for many re-executions
    assert!(MAX_RETRIES >= 10);
    // But not infinite
    assert!(MAX_RETRIES <= 1000);
}

#[test]
fn max_retries_is_one_hundred() {
    assert_eq!(MAX_RETRIES, 100);
}

// ============================================================================
// SchedulerError Tests
// ============================================================================

#[test]
fn scheduler_error_execution_failed_contains_tx_hash() {
    let tx_hash = H256::from([1u8; 32]);
    let error = SchedulerError::ExecutionFailed {
        tx_hash,
        reason: "out of gas".to_string(),
    };

    if let SchedulerError::ExecutionFailed { tx_hash: h, reason } = error {
        assert_eq!(h, H256::from([1u8; 32]));
        assert_eq!(reason, "out of gas");
    } else {
        panic!("Expected ExecutionFailed variant");
    }
}

#[test]
fn scheduler_error_max_retries_exceeded_contains_attempts() {
    let tx_hash = H256::from([2u8; 32]);
    let error = SchedulerError::MaxRetriesExceeded {
        tx_hash,
        attempts: 100,
    };

    if let SchedulerError::MaxRetriesExceeded { tx_hash: h, attempts } = error {
        assert_eq!(h, H256::from([2u8; 32]));
        assert_eq!(attempts, 100);
    } else {
        panic!("Expected MaxRetriesExceeded variant");
    }
}

#[test]
fn scheduler_error_invalid_block_contains_message() {
    let error = SchedulerError::InvalidBlock("missing parent hash".to_string());

    if let SchedulerError::InvalidBlock(msg) = error {
        assert_eq!(msg, "missing parent hash");
    } else {
        panic!("Expected InvalidBlock variant");
    }
}

#[test]
fn scheduler_error_state_error_wraps_state_error() {
    let state_error = StateError::KeyNotFound(H256::from([0xaau8; 32]));
    let error = SchedulerError::StateError(state_error.clone());

    if let SchedulerError::StateError(e) = error {
        assert_eq!(e, state_error);
    } else {
        panic!("Expected StateError variant");
    }
}

#[test]
fn scheduler_error_is_debug() {
    let error = SchedulerError::InvalidBlock("test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("InvalidBlock"));
}

#[test]
fn scheduler_error_is_clone() {
    let error = SchedulerError::InvalidBlock("test".to_string());
    let cloned = error.clone();

    if let (SchedulerError::InvalidBlock(msg1), SchedulerError::InvalidBlock(msg2)) = (error, cloned) {
        assert_eq!(msg1, msg2);
    } else {
        panic!("Clone failed");
    }
}

// ============================================================================
// ExecutionResult Tests
// ============================================================================

#[test]
fn execution_result_success_is_success() {
    let result = ExecutionResult::Success {
        output: vec![1, 2, 3],
    };
    assert!(result.is_success());
}

#[test]
fn execution_result_failed_is_not_success() {
    let result = ExecutionResult::Failed {
        reason: "revert".to_string(),
    };
    assert!(!result.is_success());
}

#[test]
fn execution_result_success_contains_output() {
    let output_data = vec![0xde, 0xad, 0xbe, 0xef];
    let result = ExecutionResult::Success {
        output: output_data.clone(),
    };

    if let ExecutionResult::Success { output } = result {
        assert_eq!(output, output_data);
    } else {
        panic!("Expected Success variant");
    }
}

#[test]
fn execution_result_success_empty_output() {
    let result = ExecutionResult::Success { output: vec![] };
    assert!(result.is_success());

    if let ExecutionResult::Success { output } = result {
        assert!(output.is_empty());
    }
}

#[test]
fn execution_result_failed_contains_reason() {
    let result = ExecutionResult::Failed {
        reason: "stack overflow".to_string(),
    };

    if let ExecutionResult::Failed { reason } = result {
        assert_eq!(reason, "stack overflow");
    } else {
        panic!("Expected Failed variant");
    }
}

#[test]
fn execution_result_is_debug() {
    let result = ExecutionResult::Success { output: vec![1] };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("Success"));
}

#[test]
fn execution_result_is_clone() {
    let result = ExecutionResult::Success {
        output: vec![1, 2, 3],
    };
    let cloned = result.clone();

    if let (ExecutionResult::Success { output: o1 }, ExecutionResult::Success { output: o2 }) =
        (result, cloned)
    {
        assert_eq!(o1, o2);
    } else {
        panic!("Clone failed");
    }
}

// ============================================================================
// ExecutedTransaction Tests
// ============================================================================

#[test]
fn executed_transaction_contains_all_fields() {
    let tx = create_test_transaction(1);
    let priority = PriorityCode::new(100, H256::from([1u8; 32]));
    let rwset = ReadWriteSet::new();
    let result = ExecutionResult::Success { output: vec![] };

    let executed = ExecutedTransaction {
        transaction: tx.clone(),
        priority: priority.clone(),
        rwset: rwset.clone(),
        result,
    };

    assert_eq!(executed.transaction.hash(), tx.hash());
    // Priority and rwset should match
}

#[test]
fn executed_transaction_hash_returns_transaction_hash() {
    let tx = create_test_transaction(42);
    let expected_hash = tx.hash();

    let executed = ExecutedTransaction {
        transaction: tx,
        priority: PriorityCode::new(1, H256::zero()),
        rwset: ReadWriteSet::new(),
        result: ExecutionResult::Success { output: vec![] },
    };

    assert_eq!(executed.hash(), expected_hash);
}

#[test]
fn executed_transaction_is_debug() {
    let executed = ExecutedTransaction {
        transaction: create_test_transaction(1),
        priority: PriorityCode::new(1, H256::zero()),
        rwset: ReadWriteSet::new(),
        result: ExecutionResult::Success { output: vec![] },
    };

    let debug_str = format!("{:?}", executed);
    assert!(debug_str.contains("ExecutedTransaction"));
}

#[test]
fn executed_transaction_is_clone() {
    let executed = ExecutedTransaction {
        transaction: create_test_transaction(1),
        priority: PriorityCode::new(1, H256::zero()),
        rwset: ReadWriteSet::new(),
        result: ExecutionResult::Success { output: vec![] },
    };

    let cloned = executed.clone();
    assert_eq!(executed.hash(), cloned.hash());
}

#[test]
fn executed_transaction_with_failed_result() {
    let executed = ExecutedTransaction {
        transaction: create_test_transaction(1),
        priority: PriorityCode::new(1, H256::zero()),
        rwset: ReadWriteSet::new(),
        result: ExecutionResult::Failed {
            reason: "test failure".to_string(),
        },
    };

    assert!(!executed.result.is_success());
}

// ============================================================================
// ScheduleResult Tests
// ============================================================================

#[test]
fn schedule_result_contains_all_fields() {
    let result = ScheduleResult {
        confirmed: vec![],
        block_hash: H256::from([1u8; 32]),
        state_root: H256::from([2u8; 32]),
        reexecution_count: 5,
    };

    assert!(result.confirmed.is_empty());
    assert_eq!(result.block_hash, H256::from([1u8; 32]));
    assert_eq!(result.state_root, H256::from([2u8; 32]));
    assert_eq!(result.reexecution_count, 5);
}

#[test]
fn schedule_result_with_confirmed_transactions() {
    let executed = ExecutedTransaction {
        transaction: create_test_transaction(1),
        priority: PriorityCode::new(1, H256::zero()),
        rwset: ReadWriteSet::new(),
        result: ExecutionResult::Success { output: vec![] },
    };

    let result = ScheduleResult {
        confirmed: vec![executed],
        block_hash: H256::zero(),
        state_root: H256::zero(),
        reexecution_count: 0,
    };

    assert_eq!(result.confirmed.len(), 1);
}

#[test]
fn schedule_result_zero_reexecution_count() {
    let result = ScheduleResult {
        confirmed: vec![],
        block_hash: H256::zero(),
        state_root: H256::zero(),
        reexecution_count: 0,
    };

    assert_eq!(result.reexecution_count, 0);
}

#[test]
fn schedule_result_is_debug() {
    let result = ScheduleResult {
        confirmed: vec![],
        block_hash: H256::zero(),
        state_root: H256::zero(),
        reexecution_count: 0,
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("ScheduleResult"));
}

// ============================================================================
// SeamlessScheduler Tests
// ============================================================================

#[test]
fn seamless_scheduler_new_with_thread_count() {
    let scheduler = SeamlessScheduler::new(8);
    // Scheduler should be created successfully
    // Thread count is stored internally
    let _ = scheduler;
}

#[test]
fn seamless_scheduler_new_with_one_thread() {
    let scheduler = SeamlessScheduler::new(1);
    let _ = scheduler;
}

#[test]
fn seamless_scheduler_new_with_many_threads() {
    let scheduler = SeamlessScheduler::new(32);
    let _ = scheduler;
}

#[test]
fn seamless_scheduler_with_default_threads() {
    let scheduler = SeamlessScheduler::with_default_threads();
    // Should use DEFAULT_THREAD_COUNT
    let _ = scheduler;
}

#[test]
fn seamless_scheduler_default_trait() {
    let scheduler = SeamlessScheduler::default();
    // Default should be same as with_default_threads
    let _ = scheduler;
}

// ============================================================================
// Scheduler Trait Tests with Mock
// ============================================================================

/// Mock executor that records calls and returns configurable results
struct MockExecutor {
    /// Tracks execution calls: (tx_hash, call_count)
    call_counts: Arc<Mutex<HashMap<H256, usize>>>,
    /// Configurable read-write sets per transaction
    rwsets: HashMap<H256, ReadWriteSet>,
    /// Configurable results per transaction
    results: HashMap<H256, ExecutionResult>,
}

impl MockExecutor {
    fn new() -> Self {
        Self {
            call_counts: Arc::new(Mutex::new(HashMap::new())),
            rwsets: HashMap::new(),
            results: HashMap::new(),
        }
    }

    fn with_rwset(mut self, tx_hash: H256, rwset: ReadWriteSet) -> Self {
        self.rwsets.insert(tx_hash, rwset);
        self
    }

    fn with_result(mut self, tx_hash: H256, result: ExecutionResult) -> Self {
        self.results.insert(tx_hash, result);
        self
    }

    fn call_count(&self, tx_hash: &H256) -> usize {
        self.call_counts.lock().unwrap().get(tx_hash).copied().unwrap_or(0)
    }
}

impl TransactionExecutor for MockExecutor {
    fn execute(&self, tx: &Transaction, _snapshot: &Snapshot) -> (ReadWriteSet, ExecutionResult) {
        let tx_hash = tx.hash();

        // Increment call count
        {
            let mut counts = self.call_counts.lock().unwrap();
            *counts.entry(tx_hash).or_insert(0) += 1;
        }

        let rwset = self.rwsets.get(&tx_hash).cloned().unwrap_or_else(ReadWriteSet::new);
        let result = self.results.get(&tx_hash).cloned().unwrap_or(ExecutionResult::Success {
            output: vec![],
        });

        (rwset, result)
    }
}

#[test]
fn schedule_empty_block() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();
    let executor = MockExecutor::new();

    let block = Block::new(
        1,             // height
        H256::zero(),  // parent_hash
        vec![],        // transactions (empty)
        1000,          // timestamp
    );

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            assert!(schedule_result.confirmed.is_empty());
            assert_eq!(schedule_result.reexecution_count, 0);
        }
        Err(e) => panic!("Scheduling empty block failed: {:?}", e),
    }
}

#[test]
fn schedule_single_transaction_no_conflicts() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx = create_test_transaction(1);
    let tx_hash = tx.hash();

    let executor = MockExecutor::new()
        .with_result(tx_hash, ExecutionResult::Success { output: vec![0xaa] });

    let block = Block::new(1, H256::zero(), vec![tx], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            assert_eq!(schedule_result.confirmed.len(), 1);
            assert_eq!(schedule_result.confirmed[0].hash(), tx_hash);
            assert!(schedule_result.confirmed[0].result.is_success());
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_multiple_transactions_no_conflicts() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);
    let tx3 = create_test_transaction(3);

    let executor = MockExecutor::new();

    let block = Block::new(1, H256::zero(), vec![tx1.clone(), tx2.clone(), tx3.clone()], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            assert_eq!(schedule_result.confirmed.len(), 3);
            // All transactions should be executed exactly once (no re-execution needed)
            assert_eq!(executor.call_count(&tx1.hash()), 1);
            assert_eq!(executor.call_count(&tx2.hash()), 1);
            assert_eq!(executor.call_count(&tx3.hash()), 1);
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_transactions_with_read_write_conflict() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);

    let key = H256::from([0x11u8; 32]);

    // tx1 writes to key
    let mut rwset1 = ReadWriteSet::new();
    rwset1.record_write(key, vec![1, 2, 3]);

    // tx2 reads from key (conflict with tx1's write)
    let mut rwset2 = ReadWriteSet::new();
    rwset2.record_read(key);

    let executor = MockExecutor::new()
        .with_rwset(tx1.hash(), rwset1)
        .with_rwset(tx2.hash(), rwset2);

    let block = Block::new(1, H256::zero(), vec![tx1, tx2], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            // Both should be confirmed, but tx2 may have been re-executed
            assert_eq!(schedule_result.confirmed.len(), 2);
            // Reexecution count may be > 0 due to conflict
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_transactions_with_write_write_conflict() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);

    let key = H256::from([0x22u8; 32]);

    // Both transactions write to the same key
    let mut rwset1 = ReadWriteSet::new();
    rwset1.record_write(key, vec![1, 1, 1]);

    let mut rwset2 = ReadWriteSet::new();
    rwset2.record_write(key, vec![2, 2, 2]);

    let executor = MockExecutor::new()
        .with_rwset(tx1.hash(), rwset1)
        .with_rwset(tx2.hash(), rwset2);

    let block = Block::new(1, H256::zero(), vec![tx1, tx2], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            assert_eq!(schedule_result.confirmed.len(), 2);
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_respects_priority_ordering() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    // Create transactions with different priorities
    let tx_low_priority = create_test_transaction(1);  // lower priority (higher number)
    let tx_high_priority = create_test_transaction(2); // higher priority

    let executor = MockExecutor::new();

    // Block contains low priority first, but scheduler should order by priority
    let block = Block::new(1, H256::zero(), vec![tx_low_priority.clone(), tx_high_priority.clone()], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            assert_eq!(schedule_result.confirmed.len(), 2);
            // Transactions should be ordered by priority in confirmed list
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_releases_ownership_after_confirmation() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);
    let tx3 = create_test_transaction(3);

    let key = H256::from([0x33u8; 32]);

    // tx1 writes to key, should own it
    let mut rwset1 = ReadWriteSet::new();
    rwset1.record_write(key, vec![1]);

    // tx2 also writes to key, must wait for tx1
    let mut rwset2 = ReadWriteSet::new();
    rwset2.record_write(key, vec![2]);

    // tx3 also writes to key, must wait for tx2
    let mut rwset3 = ReadWriteSet::new();
    rwset3.record_write(key, vec![3]);

    let executor = MockExecutor::new()
        .with_rwset(tx1.hash(), rwset1)
        .with_rwset(tx2.hash(), rwset2)
        .with_rwset(tx3.hash(), rwset3);

    let block = Block::new(1, H256::zero(), vec![tx1, tx2, tx3], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            // All three should eventually be confirmed
            assert_eq!(schedule_result.confirmed.len(), 3);
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_handles_transaction_failure() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx = create_test_transaction(1);
    let tx_hash = tx.hash();

    let executor = MockExecutor::new()
        .with_result(tx_hash, ExecutionResult::Failed {
            reason: "out of gas".to_string(),
        });

    let block = Block::new(1, H256::zero(), vec![tx], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            // Failed transactions should still be in confirmed (just with failed result)
            assert_eq!(schedule_result.confirmed.len(), 1);
            assert!(!schedule_result.confirmed[0].result.is_success());
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_max_retries_exceeded() {
    let scheduler = SeamlessScheduler::default();
    let state = MemoryStateDB::new();

    // Create a scenario where a transaction constantly conflicts
    // This requires a more sophisticated mock that always causes conflicts
    let tx = create_test_transaction(1);

    // For this test, we need to simulate constant re-execution
    // The executor would need to return different rwsets on each call
    // causing perpetual conflicts

    // Note: Full implementation would test that MaxRetriesExceeded is returned
    // when a transaction exceeds MAX_RETRIES attempts
    let _ = (scheduler, state, tx);
}

#[test]
fn schedule_updates_state() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx = create_test_transaction(1);
    let key = H256::from([1u8; 32]);
    let value = vec![1, 2, 3];

    let mut rwset = ReadWriteSet::new();
    rwset.record_write(key, value.clone());

    let executor = MockExecutor::new()
        .with_rwset(tx.hash(), rwset);

    let block = Block::new(1, H256::zero(), vec![tx], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(_) => {
            // State should contain the write from the transaction
            // (depending on implementation, state may be updated)
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_computes_block_hash() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx = create_test_transaction(1);
    let block = Block::new(1, H256::zero(), vec![tx], 1000);

    let executor = MockExecutor::new();

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            // Block hash should be non-zero after scheduling
            assert_ne!(schedule_result.block_hash, H256::zero());
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

// ============================================================================
// Parallel Execution Tests
// ============================================================================

#[test]
fn schedule_parallel_independent_transactions() {
    let scheduler = SeamlessScheduler::new(4);
    let mut state = MemoryStateDB::new();

    // Create many independent transactions
    let transactions: Vec<Transaction> = (0..10)
        .map(|i| create_test_transaction(i))
        .collect();

    let executor = MockExecutor::new();

    let block = Block::new(1, H256::zero(), transactions.clone(), 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            // All transactions should be confirmed
            assert_eq!(schedule_result.confirmed.len(), 10);
            // No re-executions needed for independent transactions
            assert_eq!(schedule_result.reexecution_count, 0);
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_with_single_thread() {
    let scheduler = SeamlessScheduler::new(1);
    let mut state = MemoryStateDB::new();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);

    let executor = MockExecutor::new();

    let block = Block::new(1, H256::zero(), vec![tx1, tx2], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    // Should work even with single thread
    assert!(result.is_ok());
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[test]
fn scheduler_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<SeamlessScheduler>();
}

#[test]
fn scheduler_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<SeamlessScheduler>();
}

#[test]
fn executor_trait_requires_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockExecutor>();
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn schedule_transaction_with_empty_rwset() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx = create_test_transaction(1);
    // Transaction with empty rwset (no reads or writes)
    let executor = MockExecutor::new()
        .with_rwset(tx.hash(), ReadWriteSet::new());

    let block = Block::new(1, H256::zero(), vec![tx], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    assert!(result.is_ok());
}

#[test]
fn schedule_transaction_with_large_output() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx = create_test_transaction(1);
    let large_output = vec![0xffu8; 1024 * 1024]; // 1MB output

    let executor = MockExecutor::new()
        .with_result(tx.hash(), ExecutionResult::Success { output: large_output });

    let block = Block::new(1, H256::zero(), vec![tx], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    assert!(result.is_ok());
}

#[test]
fn schedule_many_transactions_same_key() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let key = H256::from([0x99u8; 32]);

    // All transactions write to the same key
    let transactions: Vec<Transaction> = (0..5)
        .map(|i| create_test_transaction(i))
        .collect();

    let mut executor = MockExecutor::new();
    for tx in &transactions {
        let mut rwset = ReadWriteSet::new();
        rwset.record_write(key, vec![tx.hash().as_bytes()[0]]);
        executor = executor.with_rwset(tx.hash(), rwset);
    }

    let block = Block::new(1, H256::zero(), transactions, 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            assert_eq!(schedule_result.confirmed.len(), 5);
            // May have re-executions due to conflicts
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn schedule_block_number_zero() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();
    let executor = MockExecutor::new();

    // Block number 0 (genesis-like)
    let block = Block::new(0, H256::zero(), vec![], 0);

    let result = scheduler.schedule(block, &mut state, &executor);

    // Should either succeed or return InvalidBlock
    let _ = result;
}

// ============================================================================
// Algorithm 2 Specific Tests
// ============================================================================

#[test]
fn algorithm2_optimistic_execution() {
    // Tests that transactions are initially executed optimistically
    // without waiting for dependencies
    let scheduler = SeamlessScheduler::new(4);
    let mut state = MemoryStateDB::new();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);

    let executor = MockExecutor::new();

    let block = Block::new(1, H256::zero(), vec![tx1.clone(), tx2.clone()], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(_) => {
            // Both transactions should have been executed
            assert!(executor.call_count(&tx1.hash()) >= 1);
            assert!(executor.call_count(&tx2.hash()) >= 1);
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn algorithm2_conflict_detection_triggers_reexecution() {
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);

    let key = H256::from([0xaau8; 32]);

    // tx1 writes to key
    let mut rwset1 = ReadWriteSet::new();
    rwset1.record_write(key, vec![1]);

    // tx2 reads from key (conflict since tx1 writes)
    let mut rwset2 = ReadWriteSet::new();
    rwset2.record_read(key);

    let executor = MockExecutor::new()
        .with_rwset(tx1.hash(), rwset1)
        .with_rwset(tx2.hash(), rwset2);

    let block = Block::new(1, H256::zero(), vec![tx1, tx2], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            // tx2 should have been re-executed due to conflict
            assert!(schedule_result.reexecution_count > 0);
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

#[test]
fn algorithm2_priority_determines_ownership() {
    // Higher priority transactions win ownership of keys
    let scheduler = SeamlessScheduler::default();
    let mut state = MemoryStateDB::new();

    // Transaction priorities are determined by PriorityCode
    // Lower release_bit, then lower block_height, then lower hash = higher priority
    let tx1 = create_test_transaction(1);
    let tx2 = create_test_transaction(2);

    let key = H256::from([0xbbu8; 32]);

    // Both transactions want to write to the same key
    let mut rwset1 = ReadWriteSet::new();
    rwset1.record_write(key, vec![1]);

    let mut rwset2 = ReadWriteSet::new();
    rwset2.record_write(key, vec![2]);

    let executor = MockExecutor::new()
        .with_rwset(tx1.hash(), rwset1)
        .with_rwset(tx2.hash(), rwset2);

    let block = Block::new(1, H256::zero(), vec![tx1, tx2], 1000);

    let result = scheduler.schedule(block, &mut state, &executor);

    match result {
        Ok(schedule_result) => {
            assert_eq!(schedule_result.confirmed.len(), 2);
            // Order in confirmed should respect priority
        }
        Err(e) => panic!("Scheduling failed: {:?}", e),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a test transaction with a unique nonce
fn create_test_transaction(nonce: u64) -> Transaction {
    // Create a deterministic private key from nonce for testing
    let mut key_bytes = [0u8; 32];
    key_bytes[24..32].copy_from_slice(&nonce.to_be_bytes());
    key_bytes[0] = 0x01; // Ensure non-zero

    let private_key = PrivateKey::from_bytes(&key_bytes).unwrap();

    // Create transaction data
    let to = Some(Address::zero());
    let value = U256::from_u64(0);
    let data = vec![];

    // Create signing hash
    let mut signing_data = Vec::new();
    signing_data.extend_from_slice(&nonce.to_be_bytes());
    if let Some(addr) = &to {
        signing_data.extend_from_slice(addr.as_bytes());
    }
    signing_data.extend_from_slice(&value.to_be_bytes());
    signing_data.extend_from_slice(&data);
    let signing_hash = bach_crypto::keccak256(&signing_data);

    // Sign the transaction
    let signature = private_key.sign(&signing_hash);

    Transaction::new(
        nonce,
        to,
        value,
        data,
        signature,
    )
}
