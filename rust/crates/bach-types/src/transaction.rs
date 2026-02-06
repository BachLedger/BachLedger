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
    pub fn effective_gas_price(&self, base_fee: u128) -> u128 {
        match &self.tx {
            TransactionBody::Legacy(tx) => tx.gas_price,
            TransactionBody::DynamicFee(tx) => {
                let priority_fee = tx.max_priority_fee_per_gas.min(tx.max_fee_per_gas - base_fee);
                base_fee + priority_fee
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

    #[test]
    fn test_legacy_tx_default() {
        let tx = LegacyTx::default();
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.gas_limit, 21000);
        assert!(tx.to.is_none());
    }

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
    fn test_effective_gas_price_legacy() {
        let tx = LegacyTx {
            gas_price: 100,
            ..Default::default()
        };
        let sig = TxSignature::new(27, H256::from_bytes([1u8; 32]), H256::from_bytes([2u8; 32]));
        let signed = SignedTransaction::new_legacy(tx, sig);
        assert_eq!(signed.effective_gas_price(50), 100);
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
        assert_eq!(signed.effective_gas_price(50), 60);
    }
}
