//! Transaction pool implementation

use crate::error::{TxPoolError, TxPoolResult};
use bach_primitives::{Address, H256};
use bach_types::SignedTransaction;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Minimum gas price bump percentage for replacement (default 10%)
const MIN_GAS_PRICE_BUMP: u128 = 10;

/// Maximum nonce gap allowed for queued transactions
const MAX_NONCE_GAP: u64 = 64;

/// Minimum gas for any transaction
const MIN_GAS_LIMIT: u64 = 21000;

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of pending transactions
    pub max_pending: usize,
    /// Maximum number of queued transactions
    pub max_queued: usize,
    /// Maximum transactions per account
    pub max_per_account: usize,
    /// Block gas limit
    pub block_gas_limit: u64,
    /// Minimum gas price
    pub min_gas_price: u128,
    /// Base fee (for EIP-1559)
    pub base_fee: u128,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_pending: 4096,
            max_queued: 1024,
            max_per_account: 16,
            block_gas_limit: 30_000_000,
            min_gas_price: 1_000_000_000, // 1 gwei
            base_fee: 0,
        }
    }
}

/// Transaction with computed metadata
#[derive(Clone, Debug)]
pub struct PooledTransaction {
    /// The signed transaction
    pub tx: SignedTransaction,
    /// Recovered sender address
    pub sender: Address,
    /// Transaction hash
    pub hash: H256,
    /// Effective gas price at current base fee
    pub effective_gas_price: u128,
}

impl PooledTransaction {
    /// Get nonce
    pub fn nonce(&self) -> u64 {
        self.tx.nonce()
    }

    /// Get gas limit
    pub fn gas_limit(&self) -> u64 {
        self.tx.gas_limit()
    }
}

/// Account state in pool
#[derive(Default)]
struct AccountTxs {
    /// Pending transactions (nonce -> tx), ready for execution
    pending: BTreeMap<u64, PooledTransaction>,
    /// Queued transactions (nonce -> tx), waiting for nonce gap to fill
    queued: BTreeMap<u64, PooledTransaction>,
    /// Current nonce from state
    state_nonce: u64,
}

/// Transaction pool
pub struct TxPool {
    /// Configuration
    config: PoolConfig,
    /// All transactions by hash
    by_hash: DashMap<H256, PooledTransaction>,
    /// Transactions organized by sender
    by_sender: DashMap<Address, RwLock<AccountTxs>>,
    /// Pending count
    pending_count: AtomicUsize,
    /// Queued count
    queued_count: AtomicUsize,
}

impl TxPool {
    /// Create new pool with config
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            by_hash: DashMap::new(),
            by_sender: DashMap::new(),
            pending_count: AtomicUsize::new(0),
            queued_count: AtomicUsize::new(0),
        }
    }

    /// Create pool with default config
    pub fn with_defaults() -> Self {
        Self::new(PoolConfig::default())
    }

    /// Add a transaction to the pool
    pub fn add(&self, tx: SignedTransaction, sender: Address, hash: H256) -> TxPoolResult<()> {
        // Check if already exists
        if self.by_hash.contains_key(&hash) {
            return Err(TxPoolError::AlreadyExists(hash));
        }

        // Validate gas limit
        let gas_limit = tx.gas_limit();
        if gas_limit < MIN_GAS_LIMIT {
            return Err(TxPoolError::GasLimitTooLow(gas_limit));
        }
        if gas_limit > self.config.block_gas_limit {
            return Err(TxPoolError::GasLimitExceedsBlock {
                gas_limit,
                block_limit: self.config.block_gas_limit,
            });
        }

        // Calculate effective gas price
        let effective_gas_price = tx
            .effective_gas_price(self.config.base_fee)
            .ok_or(TxPoolError::GasPriceTooLow(0))?;

        if effective_gas_price < self.config.min_gas_price {
            return Err(TxPoolError::GasPriceTooLow(effective_gas_price));
        }

        let pooled = PooledTransaction {
            tx,
            sender,
            hash,
            effective_gas_price,
        };

        // Get or create account entry
        let account_entry = self
            .by_sender
            .entry(sender)
            .or_insert_with(|| RwLock::new(AccountTxs::default()));
        let mut account = account_entry.write();

        let nonce = pooled.nonce();
        let state_nonce = account.state_nonce;

        // Check nonce
        if nonce < state_nonce {
            return Err(TxPoolError::NonceTooLow {
                expected: state_nonce,
                got: nonce,
            });
        }

        // Check nonce gap
        if nonce > state_nonce + MAX_NONCE_GAP {
            return Err(TxPoolError::NonceGapTooLarge {
                current: state_nonce,
                tx_nonce: nonce,
            });
        }

        // Check for replacement
        let mut replacing_pending = false;
        let mut replacing_queued = false;

        if let Some(existing) = account.pending.get(&nonce) {
            let min_price = existing.effective_gas_price
                + existing.effective_gas_price * MIN_GAS_PRICE_BUMP / 100;
            if effective_gas_price < min_price {
                return Err(TxPoolError::Underpriced {
                    old: existing.effective_gas_price,
                    new: effective_gas_price,
                });
            }
            // Remove old transaction from hash index
            self.by_hash.remove(&existing.hash);
            replacing_pending = true;
        }

        if let Some(existing) = account.queued.get(&nonce) {
            let min_price = existing.effective_gas_price
                + existing.effective_gas_price * MIN_GAS_PRICE_BUMP / 100;
            if effective_gas_price < min_price {
                return Err(TxPoolError::Underpriced {
                    old: existing.effective_gas_price,
                    new: effective_gas_price,
                });
            }
            // Remove old transaction from hash index
            self.by_hash.remove(&existing.hash);
            replacing_queued = true;
        }

        // Check pool limits
        let total = self.pending_count.load(Ordering::SeqCst) + self.queued_count.load(Ordering::SeqCst);
        if total >= self.config.max_pending + self.config.max_queued {
            return Err(TxPoolError::PoolFull(total));
        }

        // Check per-account limit
        if account.pending.len() + account.queued.len() >= self.config.max_per_account {
            return Err(TxPoolError::PoolFull(self.config.max_per_account));
        }

        // Insert transaction
        self.by_hash.insert(hash, pooled.clone());

        // Determine if pending or queued
        if replacing_pending || nonce == state_nonce + account.pending.len() as u64 {
            // Can go to pending (or is replacing a pending tx)
            account.pending.insert(nonce, pooled);
            if !replacing_pending {
                self.pending_count.fetch_add(1, Ordering::SeqCst);
            }
            // Try to promote queued transactions
            self.promote_queued(&mut account);
        } else if replacing_queued {
            // Replacing a queued tx
            account.queued.insert(nonce, pooled);
            // Count stays the same
        } else {
            // Goes to queued
            account.queued.insert(nonce, pooled);
            self.queued_count.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Promote queued transactions to pending if nonce gaps are filled
    fn promote_queued(&self, account: &mut AccountTxs) {
        let mut next_nonce = account.state_nonce + account.pending.len() as u64;

        while let Some(tx) = account.queued.remove(&next_nonce) {
            account.pending.insert(next_nonce, tx);
            self.queued_count.fetch_sub(1, Ordering::SeqCst);
            self.pending_count.fetch_add(1, Ordering::SeqCst);
            next_nonce += 1;
        }
    }

    /// Remove transaction by hash
    pub fn remove(&self, hash: &H256) -> Option<PooledTransaction> {
        let (_, pooled) = self.by_hash.remove(hash)?;

        if let Some(account_entry) = self.by_sender.get(&pooled.sender) {
            let mut account = account_entry.write();
            if account.pending.remove(&pooled.nonce()).is_some() {
                self.pending_count.fetch_sub(1, Ordering::SeqCst);
            } else if account.queued.remove(&pooled.nonce()).is_some() {
                self.queued_count.fetch_sub(1, Ordering::SeqCst);
            }
        }

        Some(pooled)
    }

    /// Get pending transactions for block production, ordered by gas price
    pub fn get_pending(&self, limit: usize) -> Vec<PooledTransaction> {
        let mut all_pending: Vec<PooledTransaction> = Vec::new();

        // Collect all pending transactions
        for entry in self.by_sender.iter() {
            let account = entry.value().read();
            for (_, tx) in account.pending.iter() {
                all_pending.push(tx.clone());
            }
        }

        // Sort by effective gas price (descending)
        all_pending.sort_by(|a, b| b.effective_gas_price.cmp(&a.effective_gas_price));

        // Limit and return
        all_pending.truncate(limit);
        all_pending
    }

    /// Get transaction by hash
    pub fn get_by_hash(&self, hash: &H256) -> Option<PooledTransaction> {
        self.by_hash.get(hash).map(|r| r.clone())
    }

    /// Get next nonce for account (considering pending transactions)
    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.by_sender
            .get(address)
            .map(|entry| {
                let account = entry.read();
                account.state_nonce + account.pending.len() as u64
            })
            .unwrap_or(0)
    }

    /// Update account state nonce (e.g., after block execution)
    pub fn set_nonce(&self, address: &Address, nonce: u64) {
        if let Some(entry) = self.by_sender.get(address) {
            let mut account = entry.write();
            let old_nonce = account.state_nonce;
            account.state_nonce = nonce;

            // Remove any pending transactions with nonce < new state nonce
            let to_remove: Vec<u64> = account
                .pending
                .range(..nonce)
                .map(|(n, _)| *n)
                .collect();

            for n in to_remove {
                if let Some(tx) = account.pending.remove(&n) {
                    self.by_hash.remove(&tx.hash);
                    self.pending_count.fetch_sub(1, Ordering::SeqCst);
                }
            }

            // Also remove queued transactions with nonce < new state nonce
            let to_remove: Vec<u64> = account
                .queued
                .range(..nonce)
                .map(|(n, _)| *n)
                .collect();

            for n in to_remove {
                if let Some(tx) = account.queued.remove(&n) {
                    self.by_hash.remove(&tx.hash);
                    self.queued_count.fetch_sub(1, Ordering::SeqCst);
                }
            }

            // Try to promote queued transactions if nonce increased
            if nonce > old_nonce {
                self.promote_queued(&mut account);
            }
        }
    }

    /// Update base fee (recalculates effective gas prices)
    pub fn set_base_fee(&self, base_fee: u128) {
        // Note: This is a simplified implementation
        // A production version would recalculate all effective gas prices
        // and potentially evict transactions that are no longer valid
        let _ = base_fee;
    }

    /// Get total number of transactions
    pub fn len(&self) -> usize {
        self.pending_count.load(Ordering::SeqCst) + self.queued_count.load(Ordering::SeqCst)
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get pending transaction count
    pub fn pending_len(&self) -> usize {
        self.pending_count.load(Ordering::SeqCst)
    }

    /// Get queued transaction count
    pub fn queued_len(&self) -> usize {
        self.queued_count.load(Ordering::SeqCst)
    }

    /// Get all pending hashes (for syncing)
    pub fn pending_hashes(&self) -> Vec<H256> {
        let mut hashes = Vec::new();
        for entry in self.by_sender.iter() {
            let account = entry.read();
            for (_, tx) in account.pending.iter() {
                hashes.push(tx.hash);
            }
        }
        hashes
    }

    /// Clear all transactions
    pub fn clear(&self) {
        self.by_hash.clear();
        self.by_sender.clear();
        self.pending_count.store(0, Ordering::SeqCst);
        self.queued_count.store(0, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bach_primitives::H256;
    use bach_types::{LegacyTx, TxSignature};
    use bytes::Bytes;

    fn create_test_tx(nonce: u64, gas_price: u128) -> SignedTransaction {
        let tx = LegacyTx {
            nonce,
            gas_price,
            gas_limit: 21000,
            to: Some(Address::from_bytes([0x42; 20])),
            value: 0,
            data: Bytes::new(),
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        SignedTransaction::new_legacy(tx, sig)
    }

    fn test_hash(n: u8) -> H256 {
        H256::from_bytes([n; 32])
    }

    fn test_sender() -> Address {
        Address::from_bytes([0x11; 20])
    }

    #[test]
    fn test_pool_add_and_get() {
        let pool = TxPool::with_defaults();
        let tx = create_test_tx(0, 10_000_000_000);
        let sender = test_sender();
        let hash = test_hash(1);

        pool.add(tx.clone(), sender, hash).unwrap();

        assert_eq!(pool.len(), 1);
        assert!(pool.get_by_hash(&hash).is_some());
    }

    #[test]
    fn test_pool_duplicate_rejection() {
        let pool = TxPool::with_defaults();
        let tx = create_test_tx(0, 10_000_000_000);
        let sender = test_sender();
        let hash = test_hash(1);

        pool.add(tx.clone(), sender, hash).unwrap();
        let result = pool.add(tx, sender, hash);

        assert!(matches!(result, Err(TxPoolError::AlreadyExists(_))));
    }

    #[test]
    fn test_pool_nonce_ordering() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add transactions out of order
        pool.add(create_test_tx(2, 10_000_000_000), sender, test_hash(3)).unwrap();
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();

        // Nonce 0 should be pending, others queued initially
        // After promotion, all should be pending
        assert_eq!(pool.pending_len(), 3);
        assert_eq!(pool.queued_len(), 0);
    }

    #[test]
    fn test_pool_nonce_gap() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add nonce 0
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();

        // Add nonce 2 (gap of 1)
        pool.add(create_test_tx(2, 10_000_000_000), sender, test_hash(3)).unwrap();

        assert_eq!(pool.pending_len(), 1); // Only nonce 0
        assert_eq!(pool.queued_len(), 1); // Nonce 2 is queued

        // Fill the gap
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();

        // Now all should be pending
        assert_eq!(pool.pending_len(), 3);
        assert_eq!(pool.queued_len(), 0);
    }

    #[test]
    fn test_pool_replacement() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();
        let hash1 = test_hash(1);
        let hash2 = test_hash(2);

        // Add initial transaction
        pool.add(create_test_tx(0, 10_000_000_000), sender, hash1).unwrap();

        // Try replacement with higher gas price (>10% bump)
        pool.add(create_test_tx(0, 12_000_000_000), sender, hash2).unwrap();

        assert_eq!(pool.len(), 1);
        assert!(pool.get_by_hash(&hash1).is_none());
        assert!(pool.get_by_hash(&hash2).is_some());
    }

    #[test]
    fn test_pool_replacement_underpriced() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();

        // Try replacement with insufficient bump (<10%)
        let result = pool.add(create_test_tx(0, 10_500_000_000), sender, test_hash(2));

        assert!(matches!(result, Err(TxPoolError::Underpriced { .. })));
    }

    #[test]
    fn test_pool_get_pending() {
        let pool = TxPool::with_defaults();
        let sender1 = Address::from_bytes([0x11; 20]);
        let sender2 = Address::from_bytes([0x22; 20]);

        // Add transactions with different gas prices
        pool.add(create_test_tx(0, 5_000_000_000), sender1, test_hash(1)).unwrap();
        pool.add(create_test_tx(0, 15_000_000_000), sender2, test_hash(2)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender1, test_hash(3)).unwrap();

        let pending = pool.get_pending(10);

        // Should be ordered by gas price (descending)
        assert_eq!(pending.len(), 3);
        assert!(pending[0].effective_gas_price >= pending[1].effective_gas_price);
        assert!(pending[1].effective_gas_price >= pending[2].effective_gas_price);
    }

    #[test]
    fn test_pool_remove() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();
        let hash = test_hash(1);

        pool.add(create_test_tx(0, 10_000_000_000), sender, hash).unwrap();
        assert_eq!(pool.len(), 1);

        let removed = pool.remove(&hash);
        assert!(removed.is_some());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_pool_clear() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();

        pool.clear();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_gas_limit_validation() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Gas too low
        let tx = LegacyTx {
            nonce: 0,
            gas_price: 10_000_000_000,
            gas_limit: 100, // Below minimum
            to: None,
            value: 0,
            data: Bytes::new(),
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);

        let result = pool.add(signed, sender, test_hash(1));
        assert!(matches!(result, Err(TxPoolError::GasLimitTooLow(_))));
    }

    #[test]
    fn test_pool_set_nonce() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();
        pool.add(create_test_tx(2, 10_000_000_000), sender, test_hash(3)).unwrap();

        assert_eq!(pool.pending_len(), 3);

        // Simulate nonce update after tx execution
        pool.set_nonce(&sender, 2);

        // Transactions with nonce < 2 should be removed
        assert_eq!(pool.pending_len(), 1);
        assert!(pool.get_by_hash(&test_hash(1)).is_none());
        assert!(pool.get_by_hash(&test_hash(2)).is_none());
        assert!(pool.get_by_hash(&test_hash(3)).is_some());
    }

    // ==================== Extended Pool Tests ====================

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_pending, 4096);
        assert_eq!(config.max_queued, 1024);
        assert_eq!(config.max_per_account, 16);
        assert_eq!(config.block_gas_limit, 30_000_000);
        assert_eq!(config.min_gas_price, 1_000_000_000);
        assert_eq!(config.base_fee, 0);
    }

    #[test]
    fn test_pool_config_custom() {
        let config = PoolConfig {
            max_pending: 100,
            max_queued: 50,
            max_per_account: 8,
            block_gas_limit: 15_000_000,
            min_gas_price: 5_000_000_000,
            base_fee: 1_000_000_000,
        };
        let pool = TxPool::new(config);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_is_empty() {
        let pool = TxPool::with_defaults();
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
        assert_eq!(pool.pending_len(), 0);
        assert_eq!(pool.queued_len(), 0);
    }

    #[test]
    fn test_pooled_transaction_accessors() {
        let tx = create_test_tx(5, 20_000_000_000);
        let pooled = PooledTransaction {
            tx: tx.clone(),
            sender: test_sender(),
            hash: test_hash(1),
            effective_gas_price: 20_000_000_000,
        };

        assert_eq!(pooled.nonce(), 5);
        assert_eq!(pooled.gas_limit(), 21000);
        assert_eq!(pooled.effective_gas_price, 20_000_000_000);
    }

    #[test]
    fn test_pool_nonce_too_low() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add nonce 0 and 1
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();

        // Update state nonce to 2
        pool.set_nonce(&sender, 2);

        // Try to add nonce 1 (now too low)
        let result = pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(3));
        assert!(matches!(result, Err(TxPoolError::NonceTooLow { expected: 2, got: 1 })));
    }

    #[test]
    fn test_pool_nonce_gap_too_large() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Try to add transaction with nonce > MAX_NONCE_GAP (64)
        let result = pool.add(create_test_tx(100, 10_000_000_000), sender, test_hash(1));
        assert!(matches!(result, Err(TxPoolError::NonceGapTooLarge { .. })));
    }

    #[test]
    fn test_pool_gas_price_too_low() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Gas price below minimum (1 gwei)
        let result = pool.add(create_test_tx(0, 100_000_000), sender, test_hash(1));
        assert!(matches!(result, Err(TxPoolError::GasPriceTooLow(_))));
    }

    #[test]
    fn test_pool_gas_limit_exceeds_block() {
        let config = PoolConfig {
            block_gas_limit: 100_000,
            ..Default::default()
        };
        let pool = TxPool::new(config);
        let sender = test_sender();

        let tx = LegacyTx {
            nonce: 0,
            gas_price: 10_000_000_000,
            gas_limit: 200_000, // Exceeds block limit
            to: None,
            value: 0,
            data: Bytes::new(),
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);

        let result = pool.add(signed, sender, test_hash(1));
        assert!(matches!(result, Err(TxPoolError::GasLimitExceedsBlock { .. })));
    }

    #[test]
    fn test_pool_per_account_limit() {
        let config = PoolConfig {
            max_per_account: 3,
            ..Default::default()
        };
        let pool = TxPool::new(config);
        let sender = test_sender();

        // Add 3 transactions (limit)
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();
        pool.add(create_test_tx(2, 10_000_000_000), sender, test_hash(3)).unwrap();

        // 4th should fail
        let result = pool.add(create_test_tx(3, 10_000_000_000), sender, test_hash(4));
        assert!(matches!(result, Err(TxPoolError::PoolFull(_))));
    }

    #[test]
    fn test_pool_total_limit() {
        let config = PoolConfig {
            max_pending: 2,
            max_queued: 1,
            max_per_account: 10,
            ..Default::default()
        };
        let pool = TxPool::new(config);
        let sender1 = Address::from_bytes([0x11; 20]);
        let sender2 = Address::from_bytes([0x22; 20]);

        // Fill pool
        pool.add(create_test_tx(0, 10_000_000_000), sender1, test_hash(1)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender1, test_hash(2)).unwrap();
        pool.add(create_test_tx(0, 10_000_000_000), sender2, test_hash(3)).unwrap();

        // Pool is full (2 pending + 1 = 3 total, but sender2 adds to pending too)
        // Actually: sender1 has 2 pending, sender2 has 1 pending = 3 total
        assert_eq!(pool.len(), 3);
    }

    #[test]
    fn test_pool_get_nonce_empty() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // No account entry yet
        assert_eq!(pool.get_nonce(&sender), 0);
    }

    #[test]
    fn test_pool_get_nonce_with_pending() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();

        // Next nonce should be 2 (state_nonce=0 + 2 pending)
        assert_eq!(pool.get_nonce(&sender), 2);
    }

    #[test]
    fn test_pool_pending_hashes() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        let hash1 = test_hash(1);
        let hash2 = test_hash(2);

        pool.add(create_test_tx(0, 10_000_000_000), sender, hash1).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender, hash2).unwrap();

        let hashes = pool.pending_hashes();
        assert_eq!(hashes.len(), 2);
        assert!(hashes.contains(&hash1));
        assert!(hashes.contains(&hash2));
    }

    #[test]
    fn test_pool_remove_nonexistent() {
        let pool = TxPool::with_defaults();
        let result = pool.remove(&test_hash(99));
        assert!(result.is_none());
    }

    #[test]
    fn test_pool_replacement_exact_threshold() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add with 10 gwei
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();

        // Replace with exactly 10% bump (11 gwei) - should succeed
        pool.add(create_test_tx(0, 11_000_000_000), sender, test_hash(2)).unwrap();

        assert_eq!(pool.len(), 1);
        let tx = pool.get_by_hash(&test_hash(2)).unwrap();
        assert_eq!(tx.effective_gas_price, 11_000_000_000);
    }

    #[test]
    fn test_pool_queued_promotion() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add nonce 0 (pending)
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();

        // Add nonces 2, 3, 4 (queued due to gap at 1)
        pool.add(create_test_tx(2, 10_000_000_000), sender, test_hash(3)).unwrap();
        pool.add(create_test_tx(3, 10_000_000_000), sender, test_hash(4)).unwrap();
        pool.add(create_test_tx(4, 10_000_000_000), sender, test_hash(5)).unwrap();

        assert_eq!(pool.pending_len(), 1);
        assert_eq!(pool.queued_len(), 3);

        // Fill the gap with nonce 1
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();

        // All should be promoted to pending
        assert_eq!(pool.pending_len(), 5);
        assert_eq!(pool.queued_len(), 0);
    }

    #[test]
    fn test_pool_multiple_accounts() {
        let pool = TxPool::with_defaults();
        let sender1 = Address::from_bytes([0x11; 20]);
        let sender2 = Address::from_bytes([0x22; 20]);
        let sender3 = Address::from_bytes([0x33; 20]);

        pool.add(create_test_tx(0, 10_000_000_000), sender1, test_hash(1)).unwrap();
        pool.add(create_test_tx(0, 20_000_000_000), sender2, test_hash(2)).unwrap();
        pool.add(create_test_tx(0, 15_000_000_000), sender3, test_hash(3)).unwrap();

        assert_eq!(pool.len(), 3);
        assert_eq!(pool.pending_len(), 3);

        let pending = pool.get_pending(10);
        assert_eq!(pending.len(), 3);

        // Verify ordering by gas price
        assert_eq!(pending[0].effective_gas_price, 20_000_000_000);
        assert_eq!(pending[1].effective_gas_price, 15_000_000_000);
        assert_eq!(pending[2].effective_gas_price, 10_000_000_000);
    }

    #[test]
    fn test_pool_get_pending_limit() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add 5 transactions
        for i in 0..5 {
            pool.add(
                create_test_tx(i, 10_000_000_000 + i as u128 * 1_000_000_000),
                sender,
                test_hash(i as u8 + 1),
            ).unwrap();
        }

        // Request only 3
        let pending = pool.get_pending(3);
        assert_eq!(pending.len(), 3);

        // Should be highest gas prices
        assert!(pending.iter().all(|tx| tx.effective_gas_price >= 12_000_000_000));
    }

    #[test]
    fn test_pool_set_nonce_promotes_queued() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add nonce 0 and 1 as pending
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(1, 10_000_000_000), sender, test_hash(2)).unwrap();

        // Add nonce 3 (queued, gap at 2)
        pool.add(create_test_tx(3, 10_000_000_000), sender, test_hash(4)).unwrap();

        assert_eq!(pool.pending_len(), 2);
        assert_eq!(pool.queued_len(), 1);

        // Execute nonces 0, 1, 2 (even though 2 wasn't in pool)
        pool.set_nonce(&sender, 3);

        // Nonce 3 should now be pending (gap is filled)
        assert_eq!(pool.pending_len(), 1);
        assert_eq!(pool.queued_len(), 0);
    }

    #[test]
    fn test_pool_set_nonce_removes_queued() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add some queued transactions (nonce gap)
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(5, 10_000_000_000), sender, test_hash(6)).unwrap();

        assert_eq!(pool.pending_len(), 1);
        assert_eq!(pool.queued_len(), 1);

        // Set nonce to 10 - removes both
        pool.set_nonce(&sender, 10);

        assert_eq!(pool.pending_len(), 0);
        assert_eq!(pool.queued_len(), 0);
    }

    #[test]
    fn test_pool_replacement_in_queued() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add nonce 0 (pending)
        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();

        // Add nonce 2 (queued due to gap)
        pool.add(create_test_tx(2, 10_000_000_000), sender, test_hash(3)).unwrap();

        // Replace the queued transaction
        pool.add(create_test_tx(2, 12_000_000_000), sender, test_hash(4)).unwrap();

        assert!(pool.get_by_hash(&test_hash(3)).is_none());
        assert!(pool.get_by_hash(&test_hash(4)).is_some());
    }

    #[test]
    fn test_pool_same_account_different_nonces() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        // Add many sequential nonces
        for i in 0..10 {
            pool.add(
                create_test_tx(i, 10_000_000_000),
                sender,
                test_hash(i as u8 + 1),
            ).unwrap();
        }

        assert_eq!(pool.len(), 10);
        assert_eq!(pool.pending_len(), 10);
        assert_eq!(pool.get_nonce(&sender), 10);
    }

    #[test]
    fn test_pool_clear_resets_counts() {
        let pool = TxPool::with_defaults();
        let sender = test_sender();

        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();
        pool.add(create_test_tx(2, 10_000_000_000), sender, test_hash(3)).unwrap();

        assert_eq!(pool.pending_len(), 1);
        assert_eq!(pool.queued_len(), 1);

        pool.clear();

        assert_eq!(pool.pending_len(), 0);
        assert_eq!(pool.queued_len(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_concurrent_reads() {
        use std::sync::Arc;
        use std::thread;

        let pool = Arc::new(TxPool::with_defaults());
        let sender = test_sender();

        pool.add(create_test_tx(0, 10_000_000_000), sender, test_hash(1)).unwrap();

        let mut handles = vec![];
        for _ in 0..10 {
            let pool = Arc::clone(&pool);
            let hash = test_hash(1);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let _ = pool.get_by_hash(&hash);
                    let _ = pool.len();
                    let _ = pool.pending_len();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_pool_set_base_fee() {
        let pool = TxPool::with_defaults();
        // This is currently a no-op but should not panic
        pool.set_base_fee(50_000_000_000);
    }

    // ==================== Error Tests ====================

    #[test]
    fn test_error_display() {
        use crate::error::TxPoolError;

        let err = TxPoolError::InvalidSignature;
        assert_eq!(format!("{}", err), "invalid signature");

        let err = TxPoolError::NonceTooLow { expected: 5, got: 3 };
        assert!(format!("{}", err).contains("5"));
        assert!(format!("{}", err).contains("3"));

        let err = TxPoolError::InsufficientBalance { required: 100, available: 50 };
        assert!(format!("{}", err).contains("100"));
        assert!(format!("{}", err).contains("50"));

        let err = TxPoolError::GasLimitTooLow(1000);
        assert!(format!("{}", err).contains("1000"));

        let err = TxPoolError::PoolFull(100);
        assert!(format!("{}", err).contains("100"));

        let err = TxPoolError::Underpriced { old: 10, new: 5 };
        assert!(format!("{}", err).contains("10"));
        assert!(format!("{}", err).contains("5"));
    }

    #[test]
    fn test_error_equality() {
        use crate::error::TxPoolError;

        assert_eq!(TxPoolError::InvalidSignature, TxPoolError::InvalidSignature);
        assert_ne!(TxPoolError::InvalidSignature, TxPoolError::GasLimitTooLow(100));

        let err1 = TxPoolError::NonceTooLow { expected: 5, got: 3 };
        let err2 = TxPoolError::NonceTooLow { expected: 5, got: 3 };
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_error_clone() {
        use crate::error::TxPoolError;

        let err = TxPoolError::RecoveryFailed("test".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }
}
