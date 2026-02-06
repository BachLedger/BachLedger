//! Test harness for E2E testing
//!
//! Provides a simple, declarative API for blockchain integration tests.

use crate::{E2EError, E2EResult};
use bach_core::{BlockExecutionResult, BlockExecutor, ExecutionState};
use bach_crypto::{keccak256, public_key_to_address, sign};
use bach_primitives::{Address, H256};
use bach_types::{
    Block, BlockBody, BlockHeader, Bloom, DynamicFeeTx, LegacyTx, Receipt, SignedTransaction,
    TxSignature, TxStatus,
};
use bytes::Bytes;
use k256::ecdsa::SigningKey;

/// Default chain ID for tests
pub const TEST_CHAIN_ID: u64 = 1337;

/// Default gas limit for test transactions
pub const DEFAULT_GAS_LIMIT: u64 = 1_000_000;

/// Default gas price (10 gwei)
pub const DEFAULT_GAS_PRICE: u128 = 10_000_000_000;

/// Default block gas limit
pub const DEFAULT_BLOCK_GAS_LIMIT: u64 = 30_000_000;

/// Default base fee per gas (1 gwei)
pub const DEFAULT_BASE_FEE: u128 = 1_000_000_000;

/// Initial balance for funded test accounts (100 ETH)
pub const FUNDED_BALANCE: u128 = 100_000_000_000_000_000_000;

/// Test account with private key and address
#[derive(Clone)]
pub struct TestAccount {
    /// Private key for signing
    private_key: SigningKey,
    /// Derived address
    address: Address,
    /// Current nonce (tracked locally)
    nonce: u64,
}

impl TestAccount {
    /// Create a new random test account
    pub fn random() -> Self {
        let private_key = SigningKey::random(&mut rand::thread_rng());
        let address = public_key_to_address(private_key.verifying_key());
        Self {
            private_key,
            address,
            nonce: 0,
        }
    }

    /// Create from a known private key (hex string without 0x prefix)
    pub fn from_hex(hex: &str) -> E2EResult<Self> {
        let bytes = hex::decode(hex).map_err(|e| E2EError::Setup(e.to_string()))?;
        let private_key =
            SigningKey::from_slice(&bytes).map_err(|e| E2EError::Setup(e.to_string()))?;
        let address = public_key_to_address(private_key.verifying_key());
        Ok(Self {
            private_key,
            address,
            nonce: 0,
        })
    }

    /// Get the account address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get current nonce
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Increment and return nonce (for transaction building)
    pub fn next_nonce(&mut self) -> u64 {
        let n = self.nonce;
        self.nonce += 1;
        n
    }

    /// Sign a transaction hash
    pub fn sign(&self, message_hash: &H256) -> E2EResult<TxSignature> {
        let sig = sign(message_hash, &self.private_key)
            .map_err(|e| E2EError::Transaction(e.to_string()))?;
        Ok(TxSignature::new(
            sig.v as u64,
            H256::from_bytes(sig.r),
            H256::from_bytes(sig.s),
        ))
    }
}

impl std::fmt::Debug for TestAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestAccount")
            .field("address", &self.address.to_hex())
            .field("nonce", &self.nonce)
            .finish()
    }
}

/// Test harness for E2E blockchain testing
pub struct TestHarness {
    /// Block executor
    executor: BlockExecutor,
    /// Chain ID
    chain_id: u64,
    /// Current block number
    block_number: u64,
    /// Current timestamp
    timestamp: u64,
    /// Beneficiary address (block producer)
    beneficiary: Address,
    /// Pending transactions for next block
    pending_txs: Vec<SignedTransaction>,
}

impl TestHarness {
    /// Create a new test harness with default configuration
    pub fn new() -> Self {
        Self::with_chain_id(TEST_CHAIN_ID)
    }

    /// Create a new test harness with a specific chain ID
    pub fn with_chain_id(chain_id: u64) -> Self {
        Self {
            executor: BlockExecutor::new(chain_id),
            chain_id,
            block_number: 0,
            timestamp: 1_700_000_000, // Some reasonable starting timestamp
            beneficiary: Address::from_bytes([0x00; 20]),
            pending_txs: Vec::new(),
        }
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Get current block number
    pub fn block_number(&self) -> u64 {
        self.block_number
    }

    /// Create a new funded test account
    ///
    /// The account is created with a random private key and funded with FUNDED_BALANCE
    pub fn create_account(&mut self) -> TestAccount {
        let account = TestAccount::random();
        self.fund_account(&account.address, FUNDED_BALANCE);
        account
    }

    /// Create a test account from a known private key
    pub fn create_account_from_hex(&mut self, hex: &str) -> E2EResult<TestAccount> {
        let account = TestAccount::from_hex(hex)?;
        self.fund_account(&account.address, FUNDED_BALANCE);
        Ok(account)
    }

    /// Fund an address with a specific amount
    pub fn fund_account(&mut self, address: &Address, amount: u128) {
        let mut account = self
            .executor
            .state()
            .get_account(address)
            .unwrap_or_default();
        account.balance = account.balance.saturating_add(amount);
        self.executor.state_mut().set_account(*address, account);
    }

    /// Get account balance
    pub fn balance(&self, address: &Address) -> u128 {
        self.executor
            .state()
            .get_account(address)
            .map(|a| a.balance)
            .unwrap_or(0)
    }

    /// Get account nonce
    pub fn nonce(&self, address: &Address) -> u64 {
        self.executor
            .state()
            .get_account(address)
            .map(|a| a.nonce)
            .unwrap_or(0)
    }

    /// Get contract code
    pub fn code(&self, address: &Address) -> Option<Vec<u8>> {
        let account = self.executor.state().get_account(address)?;
        if account.code_hash.is_zero() {
            return None;
        }
        self.executor.state().get_code(&account.code_hash)
    }

    /// Get storage value
    pub fn storage(&self, address: &Address, key: &H256) -> H256 {
        self.executor.state().get_storage(address, key)
    }

    /// Build a legacy transaction
    pub fn build_legacy_tx(
        &self,
        from: &mut TestAccount,
        to: Option<Address>,
        value: u128,
        data: Vec<u8>,
    ) -> E2EResult<SignedTransaction> {
        let nonce = from.next_nonce();
        let tx = LegacyTx {
            nonce,
            gas_price: DEFAULT_GAS_PRICE,
            gas_limit: DEFAULT_GAS_LIMIT,
            to,
            value,
            data: Bytes::from(data),
        };

        let hash = self.compute_legacy_tx_hash(&tx);
        let signature = from.sign(&hash)?;

        Ok(SignedTransaction::new_legacy(tx, signature))
    }

    /// Build an EIP-1559 transaction
    pub fn build_eip1559_tx(
        &self,
        from: &mut TestAccount,
        to: Option<Address>,
        value: u128,
        data: Vec<u8>,
    ) -> E2EResult<SignedTransaction> {
        let nonce = from.next_nonce();
        let tx = DynamicFeeTx {
            chain_id: self.chain_id,
            nonce,
            max_priority_fee_per_gas: 2_000_000_000, // 2 gwei tip
            max_fee_per_gas: DEFAULT_GAS_PRICE,
            gas_limit: DEFAULT_GAS_LIMIT,
            to,
            value,
            data: Bytes::from(data),
            access_list: vec![],
        };

        let hash = self.compute_eip1559_tx_hash(&tx);
        let signature = from.sign(&hash)?;

        Ok(SignedTransaction::new_dynamic_fee(tx, signature))
    }

    /// Simple ETH transfer
    pub fn transfer(
        &mut self,
        from: &mut TestAccount,
        to: Address,
        value: u128,
    ) -> E2EResult<Receipt> {
        let tx = self.build_legacy_tx(from, Some(to), value, vec![])?;
        self.send_tx(tx)
    }

    /// Deploy a contract
    ///
    /// Returns (receipt, contract_address)
    pub fn deploy_contract(
        &mut self,
        from: &mut TestAccount,
        bytecode: Vec<u8>,
    ) -> E2EResult<(Receipt, Option<Address>)> {
        let tx = self.build_legacy_tx(from, None, 0, bytecode)?;
        let receipt = self.send_tx(tx)?;
        let contract_address = receipt.contract_address;
        Ok((receipt, contract_address))
    }

    /// Call a contract
    pub fn call(
        &mut self,
        from: &mut TestAccount,
        to: Address,
        data: Vec<u8>,
    ) -> E2EResult<Receipt> {
        let tx = self.build_legacy_tx(from, Some(to), 0, data)?;
        self.send_tx(tx)
    }

    /// Call a contract with value
    pub fn call_with_value(
        &mut self,
        from: &mut TestAccount,
        to: Address,
        value: u128,
        data: Vec<u8>,
    ) -> E2EResult<Receipt> {
        let tx = self.build_legacy_tx(from, Some(to), value, data)?;
        self.send_tx(tx)
    }

    /// Add transaction to pending pool
    pub fn add_pending_tx(&mut self, tx: SignedTransaction) {
        self.pending_txs.push(tx);
    }

    /// Send a transaction immediately (execute in its own block)
    pub fn send_tx(&mut self, tx: SignedTransaction) -> E2EResult<Receipt> {
        self.pending_txs.push(tx);
        let result = self.execute_block()?;
        result
            .receipts
            .into_iter()
            .last()
            .ok_or_else(|| E2EError::Transaction("no receipt generated".to_string()))
    }

    /// Execute a block with all pending transactions
    pub fn execute_block(&mut self) -> E2EResult<BlockExecutionResult> {
        let txs = std::mem::take(&mut self.pending_txs);
        let block = self.build_block(txs);

        let result = self
            .executor
            .execute_block(&block)
            .map_err(|e| E2EError::Transaction(e.to_string()))?;

        self.block_number += 1;
        self.timestamp += 12; // ~12 second block time

        Ok(result)
    }

    /// Execute an empty block
    pub fn mine_empty_block(&mut self) -> E2EResult<BlockExecutionResult> {
        self.execute_block()
    }

    /// Advance time by seconds
    pub fn advance_time(&mut self, seconds: u64) {
        self.timestamp += seconds;
    }

    /// Set the beneficiary (block producer) address
    pub fn set_beneficiary(&mut self, address: Address) {
        self.beneficiary = address;
    }

    /// Build a block header
    fn build_block_header(&self) -> BlockHeader {
        BlockHeader {
            parent_hash: H256::ZERO, // Simplified for testing
            ommers_hash: H256::ZERO,
            beneficiary: self.beneficiary,
            state_root: H256::ZERO,
            transactions_root: H256::ZERO,
            receipts_root: H256::ZERO,
            logs_bloom: Bloom::default(),
            difficulty: 0,
            number: self.block_number + 1,
            gas_limit: DEFAULT_BLOCK_GAS_LIMIT,
            gas_used: 0,
            timestamp: self.timestamp,
            extra_data: Bytes::new(),
            mix_hash: H256::ZERO,
            nonce: 0,
            base_fee_per_gas: Some(DEFAULT_BASE_FEE),
        }
    }

    /// Build a block with transactions
    fn build_block(&self, transactions: Vec<SignedTransaction>) -> Block {
        Block {
            header: self.build_block_header(),
            body: BlockBody { transactions },
        }
    }

    /// Compute legacy transaction hash for signing
    ///
    /// Must match the executor's compute_signing_hash exactly
    fn compute_legacy_tx_hash(&self, tx: &LegacyTx) -> H256 {
        // Must match bach_core::executor::compute_signing_hash
        // Fields: nonce, gas_limit, value, data, [to]
        let mut data = Vec::new();
        data.extend_from_slice(&tx.nonce.to_le_bytes());
        data.extend_from_slice(&tx.gas_limit.to_le_bytes());
        data.extend_from_slice(&tx.value.to_le_bytes());
        data.extend_from_slice(&tx.data);
        if let Some(to) = &tx.to {
            data.extend_from_slice(to.as_bytes());
        }
        keccak256(&data)
    }

    /// Compute EIP-1559 transaction hash for signing
    ///
    /// Must match the executor's compute_signing_hash exactly
    fn compute_eip1559_tx_hash(&self, tx: &DynamicFeeTx) -> H256 {
        // Must match bach_core::executor::compute_signing_hash
        // Fields: nonce, gas_limit, value, data, [to]
        let mut data = Vec::new();
        data.extend_from_slice(&tx.nonce.to_le_bytes());
        data.extend_from_slice(&tx.gas_limit.to_le_bytes());
        data.extend_from_slice(&tx.value.to_le_bytes());
        data.extend_from_slice(&tx.data);
        if let Some(to) = &tx.to {
            data.extend_from_slice(to.as_bytes());
        }
        keccak256(&data)
    }

    /// Get access to the underlying state (for advanced testing)
    pub fn state(&self) -> &ExecutionState {
        self.executor.state()
    }

    /// Get mutable access to the underlying state (for advanced testing)
    pub fn state_mut(&mut self) -> &mut ExecutionState {
        self.executor.state_mut()
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

// Helper trait for asserting on receipts
pub trait ReceiptAssertions {
    /// Assert transaction succeeded
    fn assert_success(&self) -> &Self;

    /// Assert transaction failed
    fn assert_failure(&self) -> &Self;

    /// Assert specific gas used
    fn assert_gas_used(&self, expected: u64) -> &Self;

    /// Assert contract was created
    fn assert_contract_created(&self) -> Address;
}

impl ReceiptAssertions for Receipt {
    fn assert_success(&self) -> &Self {
        assert_eq!(
            self.status,
            TxStatus::Success,
            "Expected transaction to succeed"
        );
        self
    }

    fn assert_failure(&self) -> &Self {
        assert_eq!(
            self.status,
            TxStatus::Failure,
            "Expected transaction to fail"
        );
        self
    }

    fn assert_gas_used(&self, expected: u64) -> &Self {
        assert_eq!(
            self.gas_used, expected,
            "Gas used mismatch: expected {}, got {}",
            expected, self.gas_used
        );
        self
    }

    fn assert_contract_created(&self) -> Address {
        self.contract_address
            .expect("Expected contract to be created")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = TestHarness::new();
        assert_eq!(harness.chain_id(), TEST_CHAIN_ID);
        assert_eq!(harness.block_number(), 0);
    }

    #[test]
    fn test_create_account() {
        let mut harness = TestHarness::new();
        let account = harness.create_account();

        assert_eq!(harness.balance(&account.address()), FUNDED_BALANCE);
        assert_eq!(account.nonce(), 0);
    }

    #[test]
    fn test_fund_account() {
        let mut harness = TestHarness::new();
        let account = TestAccount::random();

        assert_eq!(harness.balance(&account.address()), 0);

        harness.fund_account(&account.address(), 1000);
        assert_eq!(harness.balance(&account.address()), 1000);

        // Fund more
        harness.fund_account(&account.address(), 500);
        assert_eq!(harness.balance(&account.address()), 1500);
    }

    #[test]
    fn test_account_from_known_key() {
        let mut harness = TestHarness::new();

        // Well-known test key
        let account = harness
            .create_account_from_hex(
                "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            )
            .unwrap();

        assert_eq!(
            account.address().to_hex(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    #[test]
    fn test_mine_empty_block() {
        let mut harness = TestHarness::new();

        assert_eq!(harness.block_number(), 0);

        let result = harness.mine_empty_block().unwrap();
        assert_eq!(result.receipts.len(), 0);
        assert_eq!(result.gas_used, 0);

        assert_eq!(harness.block_number(), 1);
    }

    #[test]
    fn test_advance_time() {
        let mut harness = TestHarness::new();
        let initial_time = harness.timestamp;

        harness.advance_time(3600); // 1 hour

        assert_eq!(harness.timestamp, initial_time + 3600);
    }

    #[test]
    fn test_set_beneficiary() {
        let mut harness = TestHarness::new();
        let beneficiary = Address::from_bytes([0x42; 20]);

        harness.set_beneficiary(beneficiary);
        assert_eq!(harness.beneficiary, beneficiary);
    }
}
