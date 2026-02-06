//! Block executor implementation

use crate::error::{ExecutionError, ExecutionResult};
use bach_crypto::{keccak256, public_key_to_address, recover_public_key, Signature};
use bach_evm::{BlockContext, CallContext, Environment, Interpreter, TxContext};
use bach_primitives::{Address, H256};
use bach_storage::{Account, StateCache, StateReader, StateWriter};
use bach_types::{Block, Log, Receipt, SignedTransaction, TxStatus};

/// Minimum gas for any transaction
const MIN_TX_GAS: u64 = 21000;

/// Block execution result
#[derive(Debug, Clone)]
pub struct BlockExecutionResult {
    /// Transaction receipts
    pub receipts: Vec<Receipt>,
    /// State root after execution
    pub state_root: H256,
    /// Total gas used in block
    pub gas_used: u64,
    /// Logs bloom filter
    pub logs_bloom: bach_types::Bloom,
}

/// Simple in-memory state for execution
#[derive(Default)]
pub struct ExecutionState {
    /// State cache
    cache: StateCache,
}

impl ExecutionState {
    /// Create new empty state
    pub fn new() -> Self {
        Self {
            cache: StateCache::new(),
        }
    }

    /// Get account
    pub fn get_account(&self, address: &Address) -> Option<Account> {
        StateReader::get_account(&self.cache, address).ok().flatten()
    }

    /// Set account
    pub fn set_account(&mut self, address: Address, account: Account) {
        let _ = StateWriter::set_account(&mut self.cache, address, account);
    }

    /// Get code
    pub fn get_code(&self, code_hash: &H256) -> Option<Vec<u8>> {
        StateReader::get_code(&self.cache, code_hash).ok().flatten()
    }

    /// Set code
    pub fn set_code(&mut self, code_hash: H256, code: Vec<u8>) {
        let _ = StateWriter::set_code(&mut self.cache, code_hash, code);
    }

    /// Get storage
    pub fn get_storage(&self, address: &Address, key: &H256) -> H256 {
        StateReader::get_storage(&self.cache, address, key).unwrap_or(H256::ZERO)
    }

    /// Set storage
    pub fn set_storage(&mut self, address: Address, key: H256, value: H256) {
        let _ = StateWriter::set_storage(&mut self.cache, address, key, value);
    }

    /// Get the underlying cache
    pub fn cache(&self) -> &StateCache {
        &self.cache
    }

    /// Take ownership of the cache
    pub fn into_cache(self) -> StateCache {
        self.cache
    }
}

/// Block executor
pub struct BlockExecutor {
    /// Execution state
    state: ExecutionState,
    /// Chain ID
    chain_id: u64,
}

impl BlockExecutor {
    /// Create new block executor
    pub fn new(chain_id: u64) -> Self {
        Self {
            state: ExecutionState::new(),
            chain_id,
        }
    }

    /// Create with pre-existing state
    pub fn with_state(state: ExecutionState, chain_id: u64) -> Self {
        Self { state, chain_id }
    }

    /// Execute a block
    pub fn execute_block(&mut self, block: &Block) -> ExecutionResult<BlockExecutionResult> {
        let mut receipts = Vec::new();
        let mut cumulative_gas = 0u64;

        // Build block context
        let block_ctx = BlockContext {
            number: block.header.number,
            timestamp: block.header.timestamp,
            gas_limit: block.header.gas_limit,
            coinbase: block.header.beneficiary,
            prevrandao: block.header.mix_hash,
            chain_id: self.chain_id,
            base_fee: block.header.base_fee_per_gas.unwrap_or(0),
        };

        // Execute each transaction
        for tx in &block.body.transactions {
            // Check block gas limit
            if cumulative_gas + tx.gas_limit() > block.header.gas_limit {
                return Err(ExecutionError::BlockGasLimitExceeded {
                    used: cumulative_gas + tx.gas_limit(),
                    limit: block.header.gas_limit,
                });
            }

            let receipt = self.execute_transaction(tx, &block_ctx, cumulative_gas)?;
            cumulative_gas = receipt.cumulative_gas_used;
            receipts.push(receipt);
        }

        // Calculate logs bloom from all receipts
        let logs_bloom = self.calculate_logs_bloom(&receipts);

        // Compute state root (placeholder - needs merkle trie)
        let state_root = H256::ZERO;

        Ok(BlockExecutionResult {
            receipts,
            state_root,
            gas_used: cumulative_gas,
            logs_bloom,
        })
    }

    /// Execute a single transaction
    pub fn execute_transaction(
        &mut self,
        tx: &SignedTransaction,
        block_ctx: &BlockContext,
        cumulative_gas: u64,
    ) -> ExecutionResult<Receipt> {
        // Recover sender
        let sender = self.recover_sender(tx)?;

        // Get current account state
        let account = self.state.get_account(&sender).unwrap_or_default();

        // Validate nonce
        if tx.nonce() != account.nonce {
            return Err(ExecutionError::NonceMismatch {
                expected: account.nonce,
                got: tx.nonce(),
            });
        }

        // Calculate effective gas price
        let base_fee = block_ctx.base_fee;
        let effective_gas_price = tx.effective_gas_price(base_fee).ok_or_else(|| {
            ExecutionError::InvalidTransaction {
                tx_hash: H256::ZERO,
                reason: "max fee per gas less than base fee".into(),
            }
        })?;

        // Calculate up-front cost (gas * gas_price + value)
        let gas_cost = (tx.gas_limit() as u128) * effective_gas_price;
        let total_cost = gas_cost.saturating_add(tx.value());

        // Check balance
        if account.balance < total_cost {
            return Err(ExecutionError::InsufficientBalance {
                required: total_cost,
                available: account.balance,
            });
        }

        // Deduct gas cost from sender and increment nonce
        let mut sender_account = account;
        sender_account.balance = sender_account.balance.saturating_sub(gas_cost);
        sender_account.nonce = sender_account.nonce
            .checked_add(1)
            .ok_or_else(|| ExecutionError::Internal("nonce overflow".into()))?;
        self.state.set_account(sender, sender_account);

        // Build execution environment
        let to_address = tx.to().cloned().unwrap_or(Address::ZERO);
        let call_ctx = CallContext {
            caller: sender,
            address: to_address,
            value: tx.value(),
            data: tx.data().to_vec(),
            gas: tx.gas_limit().saturating_sub(MIN_TX_GAS),
            is_static: false,
            depth: 0,
        };

        let tx_ctx = TxContext {
            origin: sender,
            gas_price: effective_gas_price,
        };

        let env = Environment::new(call_ctx, block_ctx.clone(), tx_ctx);

        // Execute EVM
        let (gas_used, logs, status, contract_address) = if tx.is_contract_creation() {
            self.execute_create(tx, &env, &sender)?
        } else {
            let (gas, logs, status) = self.execute_call(tx, &env)?;
            (gas, logs, status, None)
        };

        // Calculate actual gas used
        let actual_gas_used = MIN_TX_GAS + gas_used;

        // Refund unused gas
        let gas_refund = (tx.gas_limit() - actual_gas_used) as u128 * effective_gas_price;
        if gas_refund > 0 {
            let mut sender_account = self.state.get_account(&sender).unwrap_or_default();
            sender_account.balance = sender_account.balance.saturating_add(gas_refund);
            self.state.set_account(sender, sender_account);
        }

        // Transfer value to recipient
        if status == TxStatus::Success && tx.value() > 0 && !tx.is_contract_creation() {
            if let Some(recipient) = tx.to() {
                let mut recipient_account =
                    self.state.get_account(recipient).unwrap_or_default();
                recipient_account.balance = recipient_account.balance.saturating_add(tx.value());
                self.state.set_account(*recipient, recipient_account);

                // Deduct value from sender
                let mut sender_account = self.state.get_account(&sender).unwrap_or_default();
                sender_account.balance = sender_account.balance.saturating_sub(tx.value());
                self.state.set_account(sender, sender_account);
            }
        }

        // Pay coinbase
        let miner_reward = actual_gas_used as u128 * effective_gas_price;
        let mut coinbase_account = self
            .state
            .get_account(&block_ctx.coinbase)
            .unwrap_or_default();
        coinbase_account.balance = coinbase_account.balance.saturating_add(miner_reward);
        self.state.set_account(block_ctx.coinbase, coinbase_account);

        // Build receipt
        let mut receipt = Receipt::new(status, cumulative_gas + actual_gas_used, actual_gas_used, logs);
        if let Some(addr) = contract_address {
            receipt = receipt.with_contract_address(addr);
        }
        Ok(receipt)
    }

    /// Execute a contract creation
    fn execute_create(
        &mut self,
        tx: &SignedTransaction,
        env: &Environment,
        sender: &Address,
    ) -> ExecutionResult<(u64, Vec<Log>, TxStatus, Option<Address>)> {
        let init_code = tx.data().to_vec();
        let mut interp = Interpreter::new(init_code, env.call.gas);
        let result = interp.run(env);

        let gas_used = env.call.gas - interp.gas_remaining();

        if result.success {
            // Calculate contract address (CREATE)
            let nonce = self.state.get_account(sender).map(|a| a.nonce).unwrap_or(0);
            let contract_address = self.calculate_create_address(sender, nonce.saturating_sub(1));

            // Store contract code
            if !result.output.is_empty() {
                let code_hash = keccak256(&result.output);
                self.state.set_code(code_hash, result.output.clone());

                let contract_account = Account {
                    code_hash,
                    ..Default::default()
                };
                self.state.set_account(contract_address, contract_account);
            }

            let logs = result
                .logs
                .into_iter()
                .map(|l| Log::new(l.address, l.topics, l.data.into()))
                .collect();
            Ok((gas_used, logs, TxStatus::Success, Some(contract_address)))
        } else {
            Ok((gas_used, vec![], TxStatus::Failure, None))
        }
    }

    /// Execute a contract call
    fn execute_call(
        &mut self,
        _tx: &SignedTransaction,
        env: &Environment,
    ) -> ExecutionResult<(u64, Vec<Log>, TxStatus)> {
        // Get contract code
        let code_hash = self
            .state
            .get_account(&env.call.address)
            .map(|a| a.code_hash)
            .unwrap_or(H256::ZERO);

        let code = if code_hash.is_zero() {
            vec![]
        } else {
            self.state.get_code(&code_hash).unwrap_or_default()
        };

        if code.is_empty() {
            // Simple value transfer (no code to execute)
            return Ok((0, vec![], TxStatus::Success));
        }

        let mut interp = Interpreter::new(code, env.call.gas);
        let result = interp.run(env);

        let gas_used = env.call.gas - interp.gas_remaining();

        if result.success {
            let logs = result
                .logs
                .into_iter()
                .map(|l| Log::new(l.address, l.topics, l.data.into()))
                .collect();
            Ok((gas_used, logs, TxStatus::Success))
        } else {
            Ok((gas_used, vec![], TxStatus::Failure))
        }
    }

    /// Calculate contract address for CREATE
    fn calculate_create_address(&self, sender: &Address, nonce: u64) -> Address {
        // address = keccak256(rlp([sender, nonce]))[12:]
        // Simplified: just hash sender + nonce
        let mut data = Vec::new();
        data.extend_from_slice(sender.as_bytes());
        data.extend_from_slice(&nonce.to_be_bytes());
        let hash = keccak256(&data);
        Address::from_slice(&hash.as_bytes()[12..]).unwrap_or(Address::ZERO)
    }

    /// Recover sender address from transaction signature
    fn recover_sender(&self, tx: &SignedTransaction) -> ExecutionResult<Address> {
        let message_hash = self.compute_signing_hash(tx)?;

        // Get signature bytes
        let r: [u8; 32] = *tx.signature.r.as_bytes();
        let s: [u8; 32] = *tx.signature.s.as_bytes();

        let sig = Signature {
            v: (tx.signature.v % 256) as u8,
            r,
            s,
        };

        let pubkey = recover_public_key(&message_hash, &sig)
            .map_err(|e| ExecutionError::SenderRecovery(e.to_string()))?;

        Ok(public_key_to_address(&pubkey))
    }

    /// Compute signing hash for transaction
    fn compute_signing_hash(&self, tx: &SignedTransaction) -> ExecutionResult<H256> {
        // Simplified - just hash the tx fields
        // Real implementation needs proper RLP encoding
        let mut data = Vec::new();
        data.extend_from_slice(&tx.nonce().to_le_bytes());
        data.extend_from_slice(&tx.gas_limit().to_le_bytes());
        data.extend_from_slice(&tx.value().to_le_bytes());
        data.extend_from_slice(tx.data());

        if let Some(to) = tx.to() {
            data.extend_from_slice(to.as_bytes());
        }

        Ok(keccak256(&data))
    }

    /// Calculate logs bloom from receipts
    fn calculate_logs_bloom(&self, receipts: &[Receipt]) -> bach_types::Bloom {
        let mut bloom = bach_types::Bloom::default();
        for receipt in receipts {
            bloom.accrue_bloom(&receipt.logs_bloom);
        }
        bloom
    }

    /// Get current state reference
    pub fn state(&self) -> &ExecutionState {
        &self.state
    }

    /// Get mutable state reference
    pub fn state_mut(&mut self) -> &mut ExecutionState {
        &mut self.state
    }

    /// Take the state
    pub fn into_state(self) -> ExecutionState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bach_types::{BlockBody, BlockHeader};
    use bytes::Bytes;

    fn create_test_executor() -> BlockExecutor {
        BlockExecutor::new(1)
    }

    fn create_test_block() -> Block {
        Block {
            header: BlockHeader {
                parent_hash: H256::ZERO,
                ommers_hash: H256::ZERO,
                beneficiary: Address::from_bytes([0x01; 20]),
                state_root: H256::ZERO,
                transactions_root: H256::ZERO,
                receipts_root: H256::ZERO,
                logs_bloom: bach_types::Bloom::default(),
                difficulty: 0,
                number: 1,
                gas_limit: 30_000_000,
                gas_used: 0,
                timestamp: 1000,
                extra_data: Bytes::new(),
                mix_hash: H256::ZERO,
                nonce: 0,
                base_fee_per_gas: Some(1_000_000_000),
            },
            body: BlockBody {
                transactions: vec![],
            },
        }
    }

    #[test]
    fn test_executor_creation() {
        let executor = create_test_executor();
        assert_eq!(executor.chain_id, 1);
    }

    #[test]
    fn test_empty_block_execution() {
        let mut executor = create_test_executor();
        let block = create_test_block();

        let result = executor.execute_block(&block).unwrap();

        assert_eq!(result.receipts.len(), 0);
        assert_eq!(result.gas_used, 0);
    }

    #[test]
    fn test_state_access() {
        let executor = create_test_executor();
        let addr = Address::from_bytes([0x42; 20]);

        // Initial state should have no account
        let account = executor.state().get_account(&addr);
        assert!(account.is_none());
    }

    #[test]
    fn test_state_modification() {
        let mut executor = create_test_executor();
        let addr = Address::from_bytes([0x42; 20]);

        let mut account = Account::default();
        account.balance = 1000;
        executor.state_mut().set_account(addr, account);

        let retrieved = executor.state().get_account(&addr).unwrap();
        assert_eq!(retrieved.balance, 1000);
    }

    #[test]
    fn test_execution_state_code() {
        let mut state = ExecutionState::new();
        let code = vec![0x60, 0x00, 0x60, 0x00, 0x00]; // PUSH1 0 PUSH1 0 STOP
        let code_hash = keccak256(&code);

        state.set_code(code_hash, code.clone());

        let retrieved = state.get_code(&code_hash).unwrap();
        assert_eq!(retrieved, code);
    }

    #[test]
    fn test_execution_state_storage() {
        let mut state = ExecutionState::new();
        let addr = Address::from_bytes([0x42; 20]);
        let key = H256::from_bytes([0x01; 32]);
        let value = H256::from_bytes([0x02; 32]);

        state.set_storage(addr, key, value);

        let retrieved = state.get_storage(&addr, &key);
        assert_eq!(retrieved, value);
    }

    #[test]
    fn test_create_address_calculation() {
        let executor = create_test_executor();
        let sender = Address::from_bytes([0x42; 20]);

        let addr1 = executor.calculate_create_address(&sender, 0);
        let addr2 = executor.calculate_create_address(&sender, 1);

        // Different nonces should produce different addresses
        assert_ne!(addr1, addr2);
    }

    // ==================== ExecutionState Extended Tests ====================

    #[test]
    fn test_execution_state_default() {
        let state = ExecutionState::default();
        let addr = Address::from_bytes([0x42; 20]);
        assert!(state.get_account(&addr).is_none());
    }

    #[test]
    fn test_execution_state_get_nonexistent_code() {
        let state = ExecutionState::new();
        let code_hash = H256::from_bytes([0x99; 32]);
        assert!(state.get_code(&code_hash).is_none());
    }

    #[test]
    fn test_execution_state_storage_default() {
        let state = ExecutionState::new();
        let addr = Address::from_bytes([0x42; 20]);
        let key = H256::from_bytes([0x01; 32]);
        assert_eq!(state.get_storage(&addr, &key), H256::ZERO);
    }

    #[test]
    fn test_execution_state_cache_access() {
        let mut state = ExecutionState::new();
        let addr = Address::from_bytes([0x42; 20]);
        let mut account = Account::default();
        account.balance = 500;
        state.set_account(addr, account);
        // Verify cache is accessible (returns reference)
        let _cache = state.cache();
        // Account should still be retrievable
        assert_eq!(state.get_account(&addr).unwrap().balance, 500);
    }

    #[test]
    fn test_execution_state_into_cache() {
        let mut state = ExecutionState::new();
        let addr = Address::from_bytes([0x42; 20]);
        let mut account = Account::default();
        account.balance = 1000;
        state.set_account(addr, account);
        // Verify state can be consumed into cache
        let _cache = state.into_cache();
        // State is now moved, cache ownership transferred
    }

    #[test]
    fn test_execution_state_multiple_accounts() {
        let mut state = ExecutionState::new();
        let addr1 = Address::from_bytes([0x01; 20]);
        let addr2 = Address::from_bytes([0x02; 20]);
        let mut acc1 = Account::default();
        acc1.balance = 100;
        let mut acc2 = Account::default();
        acc2.balance = 200;
        state.set_account(addr1, acc1);
        state.set_account(addr2, acc2);
        assert_eq!(state.get_account(&addr1).unwrap().balance, 100);
        assert_eq!(state.get_account(&addr2).unwrap().balance, 200);
    }

    #[test]
    fn test_execution_state_overwrite_account() {
        let mut state = ExecutionState::new();
        let addr = Address::from_bytes([0x42; 20]);
        let mut acc1 = Account::default();
        acc1.balance = 100;
        state.set_account(addr, acc1);
        let mut acc2 = Account::default();
        acc2.balance = 999;
        state.set_account(addr, acc2);
        assert_eq!(state.get_account(&addr).unwrap().balance, 999);
    }

    #[test]
    fn test_execution_state_multiple_storage_slots() {
        let mut state = ExecutionState::new();
        let addr = Address::from_bytes([0x42; 20]);
        for i in 0..10u8 {
            let key = H256::from_bytes([i; 32]);
            let value = H256::from_bytes([i + 100; 32]);
            state.set_storage(addr, key, value);
        }
        for i in 0..10u8 {
            let key = H256::from_bytes([i; 32]);
            let expected = H256::from_bytes([i + 100; 32]);
            assert_eq!(state.get_storage(&addr, &key), expected);
        }
    }

    // ==================== BlockExecutor Extended Tests ====================

    #[test]
    fn test_executor_with_state() {
        let mut state = ExecutionState::new();
        let addr = Address::from_bytes([0x42; 20]);
        let mut account = Account::default();
        account.balance = 1_000_000;
        state.set_account(addr, account);
        let executor = BlockExecutor::with_state(state, 1);
        assert_eq!(executor.state().get_account(&addr).unwrap().balance, 1_000_000);
    }

    #[test]
    fn test_executor_chain_id() {
        let executor = BlockExecutor::new(137);
        assert_eq!(executor.chain_id, 137);
    }

    #[test]
    fn test_executor_into_state() {
        let mut executor = BlockExecutor::new(1);
        let addr = Address::from_bytes([0x42; 20]);
        let mut account = Account::default();
        account.balance = 500;
        executor.state_mut().set_account(addr, account);
        let state = executor.into_state();
        assert_eq!(state.get_account(&addr).unwrap().balance, 500);
    }

    #[test]
    fn test_executor_state_mut() {
        let mut executor = BlockExecutor::new(1);
        let addr = Address::from_bytes([0x42; 20]);
        {
            let state = executor.state_mut();
            let mut account = Account::default();
            account.nonce = 5;
            state.set_account(addr, account);
        }
        assert_eq!(executor.state().get_account(&addr).unwrap().nonce, 5);
    }

    // ==================== Block Execution Tests ====================

    #[test]
    fn test_block_with_high_gas_limit() {
        let mut executor = create_test_executor();
        let mut block = create_test_block();
        block.header.gas_limit = 100_000_000;
        let result = executor.execute_block(&block).unwrap();
        assert_eq!(result.gas_used, 0);
    }

    #[test]
    fn test_block_execution_result_fields() {
        let mut executor = create_test_executor();
        let block = create_test_block();
        let result = executor.execute_block(&block).unwrap();
        assert!(result.receipts.is_empty());
        assert_eq!(result.gas_used, 0);
        assert_eq!(result.state_root, H256::ZERO);
    }

    #[test]
    fn test_block_with_zero_base_fee() {
        let mut executor = create_test_executor();
        let mut block = create_test_block();
        block.header.base_fee_per_gas = Some(0);
        let result = executor.execute_block(&block).unwrap();
        assert_eq!(result.gas_used, 0);
    }

    #[test]
    fn test_block_without_base_fee() {
        let mut executor = create_test_executor();
        let mut block = create_test_block();
        block.header.base_fee_per_gas = None;
        let result = executor.execute_block(&block).unwrap();
        assert_eq!(result.gas_used, 0);
    }

    // ==================== Create Address Tests ====================

    #[test]
    fn test_create_address_same_sender_same_nonce() {
        let executor = create_test_executor();
        let sender = Address::from_bytes([0x42; 20]);
        let addr1 = executor.calculate_create_address(&sender, 5);
        let addr2 = executor.calculate_create_address(&sender, 5);
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_create_address_different_senders() {
        let executor = create_test_executor();
        let sender1 = Address::from_bytes([0x01; 20]);
        let sender2 = Address::from_bytes([0x02; 20]);
        let addr1 = executor.calculate_create_address(&sender1, 0);
        let addr2 = executor.calculate_create_address(&sender2, 0);
        assert_ne!(addr1, addr2);
    }

    #[test]
    fn test_create_address_zero_sender() {
        let executor = create_test_executor();
        let sender = Address::ZERO;
        let addr = executor.calculate_create_address(&sender, 0);
        assert_ne!(addr, Address::ZERO);
    }

    #[test]
    fn test_create_address_max_nonce() {
        let executor = create_test_executor();
        let sender = Address::from_bytes([0x42; 20]);
        let addr = executor.calculate_create_address(&sender, u64::MAX);
        assert_ne!(addr, Address::ZERO);
    }

    // ==================== Error Tests ====================

    #[test]
    fn test_error_display_invalid_block() {
        let err = ExecutionError::InvalidBlock("test reason".to_string());
        assert!(format!("{}", err).contains("test reason"));
    }

    #[test]
    fn test_error_display_invalid_transaction() {
        let err = ExecutionError::InvalidTransaction {
            tx_hash: H256::from_bytes([0x01; 32]),
            reason: "bad tx".to_string(),
        };
        assert!(format!("{}", err).contains("bad tx"));
    }

    #[test]
    fn test_error_display_insufficient_gas() {
        let err = ExecutionError::InsufficientGas {
            required: 100000,
            available: 21000,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("100000"));
        assert!(msg.contains("21000"));
    }

    #[test]
    fn test_error_display_insufficient_balance() {
        let err = ExecutionError::InsufficientBalance {
            required: 1_000_000,
            available: 500,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("1000000"));
        assert!(msg.contains("500"));
    }

    #[test]
    fn test_error_display_nonce_mismatch() {
        let err = ExecutionError::NonceMismatch {
            expected: 5,
            got: 3,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("5"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn test_error_display_sender_recovery() {
        let err = ExecutionError::SenderRecovery("invalid signature".to_string());
        assert!(format!("{}", err).contains("invalid signature"));
    }

    #[test]
    fn test_error_display_block_gas_limit_exceeded() {
        let err = ExecutionError::BlockGasLimitExceeded {
            used: 50_000_000,
            limit: 30_000_000,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("50000000"));
        assert!(msg.contains("30000000"));
    }

    #[test]
    fn test_error_display_internal() {
        let err = ExecutionError::Internal("something went wrong".to_string());
        assert!(format!("{}", err).contains("something went wrong"));
    }

    // ==================== BlockExecutionResult Tests ====================

    #[test]
    fn test_block_execution_result_clone() {
        let result = BlockExecutionResult {
            receipts: vec![],
            state_root: H256::from_bytes([0x01; 32]),
            gas_used: 21000,
            logs_bloom: bach_types::Bloom::default(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.state_root, result.state_root);
        assert_eq!(cloned.gas_used, result.gas_used);
    }

    #[test]
    fn test_block_execution_result_debug() {
        let result = BlockExecutionResult {
            receipts: vec![],
            state_root: H256::ZERO,
            gas_used: 0,
            logs_bloom: bach_types::Bloom::default(),
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("BlockExecutionResult"));
    }

    // ==================== Logs Bloom Tests ====================

    #[test]
    fn test_calculate_logs_bloom_empty() {
        let executor = create_test_executor();
        let receipts: Vec<Receipt> = vec![];
        let bloom = executor.calculate_logs_bloom(&receipts);
        assert_eq!(bloom, bach_types::Bloom::default());
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_state_workflow() {
        let mut executor = BlockExecutor::new(1);
        let alice = Address::from_bytes([0x01; 20]);
        let bob = Address::from_bytes([0x02; 20]);
        let mut alice_account = Account::default();
        alice_account.balance = 10_000_000_000_000_000_000u128;
        alice_account.nonce = 0;
        executor.state_mut().set_account(alice, alice_account);
        let retrieved = executor.state().get_account(&alice).unwrap();
        assert_eq!(retrieved.balance, 10_000_000_000_000_000_000u128);
        assert_eq!(retrieved.nonce, 0);
        assert!(executor.state().get_account(&bob).is_none());
    }

    #[test]
    fn test_contract_code_storage() {
        let mut executor = BlockExecutor::new(1);
        let code = vec![0x60, 0x42, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3];
        let code_hash = keccak256(&code);
        executor.state_mut().set_code(code_hash, code.clone());
        let contract_addr = Address::from_bytes([0xc0; 20]);
        let mut contract_account = Account::default();
        contract_account.code_hash = code_hash;
        executor.state_mut().set_account(contract_addr, contract_account);
        let account = executor.state().get_account(&contract_addr).unwrap();
        assert_eq!(account.code_hash, code_hash);
        let stored_code = executor.state().get_code(&code_hash).unwrap();
        assert_eq!(stored_code, code);
    }

    #[test]
    fn test_storage_slot_operations() {
        let mut executor = BlockExecutor::new(1);
        let contract_addr = Address::from_bytes([0xc0; 20]);
        let slot0 = H256::ZERO;
        let value0 = H256::from_bytes([0x42; 32]);
        executor.state_mut().set_storage(contract_addr, slot0, value0);
        let slot1 = H256::from_bytes([0x01; 32]);
        let value1 = H256::from_bytes([0x99; 32]);
        executor.state_mut().set_storage(contract_addr, slot1, value1);
        assert_eq!(executor.state().get_storage(&contract_addr, &slot0), value0);
        assert_eq!(executor.state().get_storage(&contract_addr, &slot1), value1);
        let new_value0 = H256::from_bytes([0xff; 32]);
        executor.state_mut().set_storage(contract_addr, slot0, new_value0);
        assert_eq!(executor.state().get_storage(&contract_addr, &slot0), new_value0);
    }
}
