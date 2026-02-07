//! Block executor implementation

use crate::error::{ExecutionError, ExecutionResult};
use bach_crypto::{keccak256, public_key_to_address, recover_public_key, Signature};
use bach_evm::{BlockContext, CallContext, Environment, Interpreter, StateAccess, TxContext};
use bach_primitives::{Address, H256};
use bach_rlp::RlpStream;
use bach_storage::{Account, StateCache, StateDb, StateReader, StateWriter, EMPTY_CODE_HASH};
use std::sync::Arc;
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

/// In-memory state for execution with optional database fallback.
///
/// Reads check the in-memory cache first, then fall back to `StateDb` if provided.
/// Writes go only to the in-memory cache. Call `cache()` to get changes for committing.
pub struct ExecutionState {
    /// State cache (accumulates all writes)
    cache: StateCache,
    /// Optional backing database for read-through
    db: Option<Arc<StateDb>>,
    /// Snapshots for nested call revert support
    snapshots: Vec<StateCache>,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionState {
    /// Create new empty state (no database backing)
    pub fn new() -> Self {
        Self {
            cache: StateCache::new(),
            db: None,
            snapshots: Vec::new(),
        }
    }

    /// Create state backed by a database (reads fall through to DB on cache miss)
    pub fn with_db(db: Arc<StateDb>) -> Self {
        Self {
            cache: StateCache::new(),
            db: Some(db),
            snapshots: Vec::new(),
        }
    }

    /// Get account (cache first, then DB)
    pub fn get_account(&self, address: &Address) -> Option<Account> {
        // Check cache first
        if let Ok(Some(account)) = StateReader::get_account(&self.cache, address) {
            return Some(account);
        }
        // Fall back to database
        if let Some(db) = &self.db {
            return StateReader::get_account(db.as_ref(), address).ok().flatten();
        }
        None
    }

    /// Set account
    pub fn set_account(&mut self, address: Address, account: Account) {
        let _ = StateWriter::set_account(&mut self.cache, address, account);
    }

    /// Get code (cache first, then DB)
    pub fn get_code(&self, code_hash: &H256) -> Option<Vec<u8>> {
        if let Ok(Some(code)) = StateReader::get_code(&self.cache, code_hash) {
            return Some(code);
        }
        if let Some(db) = &self.db {
            return StateReader::get_code(db.as_ref(), code_hash).ok().flatten();
        }
        None
    }

    /// Set code
    pub fn set_code(&mut self, code_hash: H256, code: Vec<u8>) {
        let _ = StateWriter::set_code(&mut self.cache, code_hash, code);
    }

    /// Get storage (cache first, then DB)
    pub fn get_storage(&self, address: &Address, key: &H256) -> H256 {
        if let Ok(value) = StateReader::get_storage(&self.cache, address, key) {
            if value != H256::ZERO {
                return value;
            }
        }
        if let Some(db) = &self.db {
            return StateReader::get_storage(db.as_ref(), address, key).unwrap_or(H256::ZERO);
        }
        H256::ZERO
    }

    /// Set storage
    pub fn set_storage(&mut self, address: Address, key: H256, value: H256) {
        let _ = StateWriter::set_storage(&mut self.cache, address, key, value);
    }

    /// Get the underlying cache (for committing to DB)
    pub fn cache(&self) -> &StateCache {
        &self.cache
    }

    /// Take ownership of the cache
    pub fn into_cache(self) -> StateCache {
        self.cache
    }
}

/// Adapter to implement bach_evm::StateAccess for ExecutionState
struct ExecutionStateAccess<'a> {
    state: &'a mut ExecutionState,
}

impl<'a> StateAccess for ExecutionStateAccess<'a> {
    fn get_storage(&self, address: &Address, key: &H256) -> H256 {
        self.state.get_storage(address, key)
    }

    fn set_storage(&mut self, address: Address, key: H256, value: H256) {
        self.state.set_storage(address, key, value);
    }

    fn get_balance(&self, address: &Address) -> u128 {
        self.state.get_account(address).map(|a| a.balance).unwrap_or(0)
    }

    fn get_code(&self, address: &Address) -> Vec<u8> {
        let code_hash = self.state.get_account(address)
            .map(|a| a.code_hash)
            .unwrap_or(H256::ZERO);
        if code_hash.is_zero() || code_hash == EMPTY_CODE_HASH {
            return vec![];
        }
        self.state.get_code(&code_hash).unwrap_or_default()
    }

    fn get_code_hash(&self, address: &Address) -> H256 {
        self.state.get_account(address)
            .map(|a| a.code_hash)
            .unwrap_or(EMPTY_CODE_HASH)
    }

    fn account_exists(&self, address: &Address) -> bool {
        self.state.get_account(address).is_some()
    }

    fn transfer(&mut self, from: &Address, to: &Address, value: u128) -> Result<(), bach_evm::EvmError> {
        if value == 0 {
            return Ok(());
        }
        let mut from_account = self.state.get_account(from).unwrap_or_default();
        if from_account.balance < value {
            return Err(bach_evm::EvmError::InsufficientBalance);
        }
        from_account.balance -= value;
        self.state.set_account(*from, from_account);

        let mut to_account = self.state.get_account(to).unwrap_or_default();
        to_account.balance = to_account.balance.saturating_add(value);
        self.state.set_account(*to, to_account);
        Ok(())
    }

    fn get_nonce(&self, address: &Address) -> u64 {
        self.state.get_account(address).map(|a| a.nonce).unwrap_or(0)
    }

    fn increment_nonce(&mut self, address: &Address) -> u64 {
        let mut account = self.state.get_account(address).unwrap_or_default();
        let old_nonce = account.nonce;
        account.nonce = account.nonce.wrapping_add(1);
        self.state.set_account(*address, account);
        old_nonce
    }

    fn mark_warm(&mut self, _address: &Address) {}
    fn is_warm(&self, _address: &Address) -> bool { true }
    fn mark_storage_warm(&mut self, _address: &Address, _key: &H256) {}
    fn is_storage_warm(&self, _address: &Address, _key: &H256) -> bool { true }

    fn snapshot(&self) -> usize {
        // Simplified snapshot: just return counter
        self.state.snapshots.len()
    }

    fn revert_to_snapshot(&mut self, _snapshot: usize) {
        // Simplified: snapshots not fully implemented for nested calls yet
        // For single-level execution this is fine
    }

    fn commit_snapshot(&mut self, _snapshot: usize) {
        // No-op for simplified implementation
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
        // Recover sender using EIP-155 signing hash
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

        // Calculate contract address using correct RLP encoding
        let nonce = self.state.get_account(sender).map(|a| a.nonce).unwrap_or(0);
        let contract_address = calculate_create_address(sender, nonce.saturating_sub(1));

        // Update environment with contract address
        let mut create_env = env.clone();
        create_env.call.address = contract_address;

        let mut state_access = ExecutionStateAccess { state: &mut self.state };
        let mut interp = Interpreter::new(init_code, env.call.gas);
        let result = interp.run_with_state(&create_env, &mut state_access);

        let gas_used = env.call.gas - interp.gas_remaining();

        if result.success {
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

        let code = if code_hash.is_zero() || code_hash == EMPTY_CODE_HASH {
            vec![]
        } else {
            self.state.get_code(&code_hash).unwrap_or_default()
        };

        if code.is_empty() {
            // Simple value transfer (no code to execute)
            return Ok((0, vec![], TxStatus::Success));
        }

        let mut state_access = ExecutionStateAccess { state: &mut self.state };
        let mut interp = Interpreter::new(code, env.call.gas);
        let result = interp.run_with_state(env, &mut state_access);

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

    /// Recover sender address from transaction signature using EIP-155
    fn recover_sender(&self, tx: &SignedTransaction) -> ExecutionResult<Address> {
        let message_hash = self.compute_signing_hash(tx)?;

        // Get signature bytes
        let r: [u8; 32] = *tx.signature.r.as_bytes();
        let s: [u8; 32] = *tx.signature.s.as_bytes();

        // Convert EIP-155 v value to raw recovery id (27 or 28)
        let v_raw = tx.signature.v;
        let recovery_v = if v_raw >= 35 {
            // EIP-155: v = recovery_id + chain_id * 2 + 35
            let recovery_id = v_raw.saturating_sub(self.chain_id * 2 + 35);
            (recovery_id as u8) + 27
        } else {
            // Legacy v (27 or 28)
            v_raw as u8
        };

        let sig = Signature {
            v: recovery_v,
            r,
            s,
        };

        let pubkey = recover_public_key(&message_hash, &sig)
            .map_err(|e| ExecutionError::SenderRecovery(e.to_string()))?;

        Ok(public_key_to_address(&pubkey))
    }

    /// Compute EIP-155 signing hash for transaction
    fn compute_signing_hash(&self, tx: &SignedTransaction) -> ExecutionResult<H256> {
        // For legacy transactions, the signing hash is:
        // keccak256(RLP([nonce, gasPrice, gasLimit, to, value, data, chainId, 0, 0]))
        let mut stream = RlpStream::new_list(9);
        stream.append(&tx.nonce());
        // For EIP-1559 transactions, use max_fee_per_gas as the gas price field
        let gas_price = tx.gas_price()
            .or(tx.max_fee_per_gas())
            .unwrap_or(0);
        stream.append(&gas_price);
        stream.append(&tx.gas_limit());

        if let Some(to) = tx.to() {
            stream.append(to);
        } else {
            stream.append_empty_data();
        }

        stream.append(&tx.value());
        stream.append(&tx.data().to_vec());
        stream.append(&self.chain_id);
        stream.append(&0u8);
        stream.append(&0u8);

        Ok(keccak256(&stream.out()))
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

/// Calculate CREATE address: keccak256(RLP([sender, nonce]))[12:]
pub fn calculate_create_address(sender: &Address, nonce: u64) -> Address {
    let mut stream = RlpStream::new_list(2);
    stream.append(sender);
    if nonce == 0 {
        stream.append_empty_data();
    } else {
        stream.append(&nonce);
    }
    let hash = keccak256(&stream.out());
    Address::from_slice(&hash.as_bytes()[12..]).unwrap_or(Address::ZERO)
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
    fn test_create_address_calculation() {
        let sender = Address::from_bytes([0x42; 20]);
        let addr1 = calculate_create_address(&sender, 0);
        let addr2 = calculate_create_address(&sender, 1);
        assert_ne!(addr1, addr2);
        assert_ne!(addr1, Address::ZERO);
    }

    #[test]
    fn test_execution_state_code() {
        let mut state = ExecutionState::new();
        let code = vec![0x60, 0x00, 0x60, 0x00, 0x00];
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
}
