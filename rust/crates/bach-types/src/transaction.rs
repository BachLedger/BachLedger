//! Transaction types for BachLedger

use bach_primitives::{Address, H256};
use bytes::Bytes;

/// Transaction type identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TxType {
    /// Legacy transaction (pre-EIP-2718)
    #[default]
    Legacy = 0,
    /// EIP-2930 access list transaction
    AccessList = 1,
    /// EIP-1559 dynamic fee transaction
    DynamicFee = 2,
}

/// Legacy transaction (Type 0)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyTx {
    /// Transaction nonce
    pub nonce: u64,
    /// Gas price in wei
    pub gas_price: u128,
    /// Gas limit
    pub gas_limit: u64,
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,
    /// Value to transfer in wei
    pub value: u128,
    /// Input data
    pub data: Bytes,
}

/// EIP-1559 dynamic fee transaction (Type 2)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicFeeTx {
    /// Chain ID
    pub chain_id: u64,
    /// Transaction nonce
    pub nonce: u64,
    /// Max priority fee per gas (tip)
    pub max_priority_fee_per_gas: u128,
    /// Max fee per gas
    pub max_fee_per_gas: u128,
    /// Gas limit
    pub gas_limit: u64,
    /// Recipient address (None for contract creation)
    pub to: Option<Address>,
    /// Value to transfer in wei
    pub value: u128,
    /// Input data
    pub data: Bytes,
    /// Access list
    pub access_list: Vec<AccessListItem>,
}

/// Access list item (address + storage keys)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessListItem {
    /// Account address
    pub address: Address,
    /// Storage keys
    pub storage_keys: Vec<H256>,
}

/// Signature components
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxSignature {
    /// Recovery ID (v value)
    pub v: u64,
    /// R component
    pub r: H256,
    /// S component
    pub s: H256,
}

impl TxSignature {
    /// Create a new signature
    pub fn new(v: u64, r: H256, s: H256) -> Self {
        Self { v, r, s }
    }

    /// Check if signature is valid (non-zero r and s)
    pub fn is_valid(&self) -> bool {
        !self.r.is_zero() && !self.s.is_zero()
    }
}

/// Signed transaction
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedTransaction {
    /// Transaction type
    pub tx_type: TxType,
    /// Transaction body
    pub tx: TransactionBody,
    /// Signature
    pub signature: TxSignature,
    /// Cached transaction hash
    hash: Option<H256>,
}

/// Transaction body (unsigned)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactionBody {
    /// Legacy transaction
    Legacy(LegacyTx),
    /// EIP-1559 transaction
    DynamicFee(DynamicFeeTx),
}

impl SignedTransaction {
    /// Create a new signed legacy transaction
    pub fn new_legacy(tx: LegacyTx, signature: TxSignature) -> Self {
        Self {
            tx_type: TxType::Legacy,
            tx: TransactionBody::Legacy(tx),
            signature,
            hash: None,
        }
    }

    /// Create a new signed EIP-1559 transaction
    pub fn new_dynamic_fee(tx: DynamicFeeTx, signature: TxSignature) -> Self {
        Self {
            tx_type: TxType::DynamicFee,
            tx: TransactionBody::DynamicFee(tx),
            signature,
            hash: None,
        }
    }

    /// Get transaction nonce
    pub fn nonce(&self) -> u64 {
        match &self.tx {
            TransactionBody::Legacy(tx) => tx.nonce,
            TransactionBody::DynamicFee(tx) => tx.nonce,
        }
    }

    /// Get gas limit
    pub fn gas_limit(&self) -> u64 {
        match &self.tx {
            TransactionBody::Legacy(tx) => tx.gas_limit,
            TransactionBody::DynamicFee(tx) => tx.gas_limit,
        }
    }

    /// Get recipient address
    pub fn to(&self) -> Option<&Address> {
        match &self.tx {
            TransactionBody::Legacy(tx) => tx.to.as_ref(),
            TransactionBody::DynamicFee(tx) => tx.to.as_ref(),
        }
    }

    /// Get transfer value
    pub fn value(&self) -> u128 {
        match &self.tx {
            TransactionBody::Legacy(tx) => tx.value,
            TransactionBody::DynamicFee(tx) => tx.value,
        }
    }

    /// Get input data
    pub fn data(&self) -> &Bytes {
        match &self.tx {
            TransactionBody::Legacy(tx) => &tx.data,
            TransactionBody::DynamicFee(tx) => &tx.data,
        }
    }

    /// Check if this is a contract creation transaction
    pub fn is_contract_creation(&self) -> bool {
        self.to().is_none()
    }

    /// Get effective gas price for the given base fee
    ///
    /// Returns `None` if `base_fee > max_fee_per_gas` for EIP-1559 transactions
    /// (transaction cannot be included in block with this base fee).
    pub fn effective_gas_price(&self, base_fee: u128) -> Option<u128> {
        match &self.tx {
            TransactionBody::Legacy(tx) => Some(tx.gas_price),
            TransactionBody::DynamicFee(tx) => {
                if base_fee > tx.max_fee_per_gas {
                    return None;
                }
                let priority_fee = tx.max_priority_fee_per_gas.min(tx.max_fee_per_gas - base_fee);
                Some(base_fee + priority_fee)
            }
        }
    }
}

impl Default for LegacyTx {
    fn default() -> Self {
        Self {
            nonce: 0,
            gas_price: 0,
            gas_limit: 21000,
            to: None,
            value: 0,
            data: Bytes::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TxType tests ====================

    #[test]
    fn test_tx_type_default() {
        let tx_type = TxType::default();
        assert_eq!(tx_type, TxType::Legacy);
    }

    #[test]
    fn test_tx_type_values() {
        assert_eq!(TxType::Legacy as u8, 0);
        assert_eq!(TxType::AccessList as u8, 1);
        assert_eq!(TxType::DynamicFee as u8, 2);
    }

    #[test]
    fn test_tx_type_clone_and_eq() {
        let t1 = TxType::DynamicFee;
        let t2 = t1;
        assert_eq!(t1, t2);
    }

    // ==================== LegacyTx tests ====================

    #[test]
    fn test_legacy_tx_default() {
        let tx = LegacyTx::default();
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.gas_limit, 21000);
        assert!(tx.to.is_none());
        assert_eq!(tx.gas_price, 0);
        assert_eq!(tx.value, 0);
        assert!(tx.data.is_empty());
    }

    #[test]
    fn test_legacy_tx_with_values() {
        let to_addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let tx = LegacyTx {
            nonce: 42,
            gas_price: 20_000_000_000, // 20 gwei
            gas_limit: 100_000,
            to: Some(to_addr),
            value: 1_000_000_000_000_000_000, // 1 ETH
            data: Bytes::from(vec![0xa9, 0x05, 0x9c, 0xbb]), // transfer selector
        };

        assert_eq!(tx.nonce, 42);
        assert_eq!(tx.gas_price, 20_000_000_000);
        assert_eq!(tx.gas_limit, 100_000);
        assert_eq!(tx.to, Some(to_addr));
        assert_eq!(tx.value, 1_000_000_000_000_000_000);
        assert_eq!(tx.data.len(), 4);
    }

    #[test]
    fn test_legacy_tx_clone_and_eq() {
        let tx1 = LegacyTx {
            nonce: 1,
            gas_price: 100,
            ..Default::default()
        };
        let tx2 = tx1.clone();
        assert_eq!(tx1, tx2);
    }

    #[test]
    fn test_legacy_tx_max_values() {
        let tx = LegacyTx {
            nonce: u64::MAX,
            gas_price: u128::MAX,
            gas_limit: u64::MAX,
            to: None,
            value: u128::MAX,
            data: Bytes::new(),
        };
        assert_eq!(tx.nonce, u64::MAX);
        assert_eq!(tx.gas_price, u128::MAX);
    }

    // ==================== DynamicFeeTx tests ====================

    #[test]
    fn test_dynamic_fee_tx_creation() {
        let tx = DynamicFeeTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 2_000_000_000, // 2 gwei
            max_fee_per_gas: 100_000_000_000, // 100 gwei
            gas_limit: 21000,
            to: None,
            value: 0,
            data: Bytes::new(),
            access_list: vec![],
        };

        assert_eq!(tx.chain_id, 1);
        assert_eq!(tx.max_priority_fee_per_gas, 2_000_000_000);
        assert_eq!(tx.max_fee_per_gas, 100_000_000_000);
    }

    #[test]
    fn test_dynamic_fee_tx_with_access_list() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let storage_key = H256::from_bytes([0x01; 32]);

        let access_item = AccessListItem {
            address: addr,
            storage_keys: vec![storage_key],
        };

        let tx = DynamicFeeTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 10,
            max_fee_per_gas: 100,
            gas_limit: 21000,
            to: None,
            value: 0,
            data: Bytes::new(),
            access_list: vec![access_item.clone()],
        };

        assert_eq!(tx.access_list.len(), 1);
        assert_eq!(tx.access_list[0].address, addr);
        assert_eq!(tx.access_list[0].storage_keys.len(), 1);
    }

    // ==================== AccessListItem tests ====================

    #[test]
    fn test_access_list_item() {
        let addr = Address::from_bytes([0x42; 20]);
        let key1 = H256::from_bytes([0x01; 32]);
        let key2 = H256::from_bytes([0x02; 32]);

        let item = AccessListItem {
            address: addr,
            storage_keys: vec![key1, key2],
        };

        assert_eq!(item.address, addr);
        assert_eq!(item.storage_keys.len(), 2);
    }

    #[test]
    fn test_access_list_item_empty_keys() {
        let item = AccessListItem {
            address: Address::ZERO,
            storage_keys: vec![],
        };
        assert!(item.storage_keys.is_empty());
    }

    // ==================== TxSignature tests ====================

    #[test]
    fn test_signature_validity() {
        let valid_sig = TxSignature::new(
            27,
            H256::from_bytes([1u8; 32]),
            H256::from_bytes([2u8; 32]),
        );
        assert!(valid_sig.is_valid());

        let invalid_sig = TxSignature::new(27, H256::ZERO, H256::from_bytes([2u8; 32]));
        assert!(!invalid_sig.is_valid());
    }

    #[test]
    fn test_signature_zero_r() {
        let sig = TxSignature::new(27, H256::ZERO, H256::from_bytes([1u8; 32]));
        assert!(!sig.is_valid());
    }

    #[test]
    fn test_signature_zero_s() {
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::ZERO);
        assert!(!sig.is_valid());
    }

    #[test]
    fn test_signature_both_zero() {
        let sig = TxSignature::new(27, H256::ZERO, H256::ZERO);
        assert!(!sig.is_valid());
    }

    #[test]
    fn test_signature_v_values() {
        // Ethereum uses v = 27 or 28 for legacy, or 0/1 for EIP-155
        let sig27 = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let sig28 = TxSignature::new(28, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));

        assert_eq!(sig27.v, 27);
        assert_eq!(sig28.v, 28);
    }

    #[test]
    fn test_signature_clone_and_eq() {
        let sig1 = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let sig2 = sig1.clone();
        assert_eq!(sig1, sig2);
    }

    // ==================== SignedTransaction tests ====================

    #[test]
    fn test_contract_creation() {
        let tx = LegacyTx {
            to: None,
            ..Default::default()
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);
        assert!(signed.is_contract_creation());
    }

    #[test]
    fn test_not_contract_creation() {
        let tx = LegacyTx {
            to: Some(Address::from_bytes([0x42; 20])),
            ..Default::default()
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);
        assert!(!signed.is_contract_creation());
    }

    #[test]
    fn test_signed_tx_accessors_legacy() {
        let to_addr = Address::from_bytes([0x42; 20]);
        let tx = LegacyTx {
            nonce: 5,
            gas_price: 100,
            gas_limit: 50000,
            to: Some(to_addr),
            value: 1000,
            data: Bytes::from(vec![0x01, 0x02]),
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);

        assert_eq!(signed.nonce(), 5);
        assert_eq!(signed.gas_limit(), 50000);
        assert_eq!(signed.to(), Some(&to_addr));
        assert_eq!(signed.value(), 1000);
        assert_eq!(signed.data().len(), 2);
        assert_eq!(signed.tx_type, TxType::Legacy);
    }

    #[test]
    fn test_signed_tx_accessors_dynamic_fee() {
        let to_addr = Address::from_bytes([0x42; 20]);
        let tx = DynamicFeeTx {
            chain_id: 1,
            nonce: 10,
            max_priority_fee_per_gas: 5,
            max_fee_per_gas: 100,
            gas_limit: 100000,
            to: Some(to_addr),
            value: 2000,
            data: Bytes::from(vec![0x03]),
            access_list: vec![],
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_dynamic_fee(tx, sig);

        assert_eq!(signed.nonce(), 10);
        assert_eq!(signed.gas_limit(), 100000);
        assert_eq!(signed.to(), Some(&to_addr));
        assert_eq!(signed.value(), 2000);
        assert_eq!(signed.data().len(), 1);
        assert_eq!(signed.tx_type, TxType::DynamicFee);
    }

    // ==================== Effective gas price tests ====================

    #[test]
    fn test_effective_gas_price_legacy() {
        let tx = LegacyTx {
            gas_price: 100,
            ..Default::default()
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);
        // Legacy tx ignores base fee, always uses gas_price
        assert_eq!(signed.effective_gas_price(50), Some(100));
        assert_eq!(signed.effective_gas_price(0), Some(100));
        assert_eq!(signed.effective_gas_price(200), Some(100));
    }

    #[test]
    fn test_effective_gas_price_dynamic() {
        let tx = DynamicFeeTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 10,
            max_fee_per_gas: 100,
            gas_limit: 21000,
            to: None,
            value: 0,
            data: Bytes::new(),
            access_list: vec![],
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_dynamic_fee(tx, sig);
        // base_fee=50, priority_fee=min(10, 100-50)=10, effective=60
        assert_eq!(signed.effective_gas_price(50), Some(60));
    }

    #[test]
    fn test_effective_gas_price_dynamic_capped() {
        // When base_fee is high, priority_fee gets capped
        let tx = DynamicFeeTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 50,
            max_fee_per_gas: 100,
            gas_limit: 21000,
            to: None,
            value: 0,
            data: Bytes::new(),
            access_list: vec![],
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_dynamic_fee(tx, sig);
        // base_fee=80, remaining=100-80=20, priority_fee=min(50, 20)=20, effective=100
        assert_eq!(signed.effective_gas_price(80), Some(100));
    }

    #[test]
    fn test_effective_gas_price_dynamic_zero_base() {
        let tx = DynamicFeeTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 10,
            max_fee_per_gas: 100,
            gas_limit: 21000,
            to: None,
            value: 0,
            data: Bytes::new(),
            access_list: vec![],
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_dynamic_fee(tx, sig);
        // base_fee=0, priority_fee=min(10, 100-0)=10, effective=10
        assert_eq!(signed.effective_gas_price(0), Some(10));
    }

    #[test]
    fn test_effective_gas_price_base_fee_too_high() {
        // Critical: base_fee > max_fee_per_gas should return None, not panic
        let tx = DynamicFeeTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 10,
            max_fee_per_gas: 100,
            gas_limit: 21000,
            to: None,
            value: 0,
            data: Bytes::new(),
            access_list: vec![],
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_dynamic_fee(tx, sig);

        // base_fee > max_fee_per_gas: transaction cannot be included
        assert_eq!(signed.effective_gas_price(150), None);
        assert_eq!(signed.effective_gas_price(101), None);

        // base_fee == max_fee_per_gas: still valid (edge case)
        assert_eq!(signed.effective_gas_price(100), Some(100));
    }

    // ==================== TransactionBody tests ====================

    #[test]
    fn test_transaction_body_legacy() {
        let tx = LegacyTx::default();
        let body = TransactionBody::Legacy(tx.clone());

        match body {
            TransactionBody::Legacy(inner) => assert_eq!(inner, tx),
            _ => panic!("Expected Legacy variant"),
        }
    }

    #[test]
    fn test_transaction_body_dynamic_fee() {
        let tx = DynamicFeeTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 10,
            max_fee_per_gas: 100,
            gas_limit: 21000,
            to: None,
            value: 0,
            data: Bytes::new(),
            access_list: vec![],
        };
        let body = TransactionBody::DynamicFee(tx.clone());

        match body {
            TransactionBody::DynamicFee(inner) => assert_eq!(inner, tx),
            _ => panic!("Expected DynamicFee variant"),
        }
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_empty_data_transaction() {
        let tx = LegacyTx {
            data: Bytes::new(),
            ..Default::default()
        };
        assert!(tx.data.is_empty());
    }

    #[test]
    fn test_large_data_transaction() {
        let large_data = vec![0xab; 32 * 1024]; // 32KB of data
        let tx = LegacyTx {
            data: Bytes::from(large_data.clone()),
            ..Default::default()
        };
        assert_eq!(tx.data.len(), 32 * 1024);
    }

    #[test]
    fn test_zero_value_transfer() {
        let tx = LegacyTx {
            to: Some(Address::from_bytes([0x42; 20])),
            value: 0,
            ..Default::default()
        };
        assert_eq!(tx.value, 0);
        assert!(!tx.to.is_none());
    }

    #[test]
    fn test_signed_tx_clone_and_eq() {
        let tx = LegacyTx::default();
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed1 = SignedTransaction::new_legacy(tx, sig);
        let signed2 = signed1.clone();
        assert_eq!(signed1, signed2);
    }
}
