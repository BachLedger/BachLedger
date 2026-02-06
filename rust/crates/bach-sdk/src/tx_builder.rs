//! Transaction builder

use bach_crypto::{keccak256, sign};
use bach_primitives::{Address, H256};
use bach_types::{DynamicFeeTx, LegacyTx, SignedTransaction, TxSignature};
use bytes::Bytes;

use crate::{SdkError, Wallet};

/// Transaction builder with fluent API
#[derive(Debug, Clone, Default)]
pub struct TxBuilder {
    chain_id: u64,
    nonce: Option<u64>,
    gas_limit: Option<u64>,
    gas_price: Option<u128>,
    max_fee_per_gas: Option<u128>,
    max_priority_fee_per_gas: Option<u128>,
    to: Option<Address>,
    value: u128,
    data: Bytes,
}

impl TxBuilder {
    /// Create a new transaction builder
    pub fn new(chain_id: u64) -> Self {
        Self {
            chain_id,
            ..Default::default()
        }
    }

    /// Set the nonce
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set the gas limit
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    /// Set the gas price (for legacy transactions)
    pub fn gas_price(mut self, price: u128) -> Self {
        self.gas_price = Some(price);
        self
    }

    /// Set max fee per gas (for EIP-1559 transactions)
    pub fn max_fee_per_gas(mut self, fee: u128) -> Self {
        self.max_fee_per_gas = Some(fee);
        self
    }

    /// Set max priority fee per gas (for EIP-1559 transactions)
    pub fn max_priority_fee_per_gas(mut self, fee: u128) -> Self {
        self.max_priority_fee_per_gas = Some(fee);
        self
    }

    /// Set the recipient address
    pub fn to(mut self, address: Address) -> Self {
        self.to = Some(address);
        self
    }

    /// Set the value to transfer (in wei)
    pub fn value(mut self, value: u128) -> Self {
        self.value = value;
        self
    }

    /// Set the input data
    pub fn data(mut self, data: impl Into<Bytes>) -> Self {
        self.data = data.into();
        self
    }

    /// Build a legacy transaction (unsigned)
    pub fn build_legacy(&self) -> Result<LegacyTx, SdkError> {
        let nonce = self.nonce.ok_or(SdkError::MissingField("nonce".to_string()))?;
        let gas_limit = self.gas_limit.ok_or(SdkError::MissingField("gas_limit".to_string()))?;
        let gas_price = self.gas_price.ok_or(SdkError::MissingField("gas_price".to_string()))?;

        Ok(LegacyTx {
            nonce,
            gas_price,
            gas_limit,
            to: self.to,
            value: self.value,
            data: self.data.clone(),
        })
    }

    /// Build an EIP-1559 transaction (unsigned)
    pub fn build_eip1559(&self) -> Result<DynamicFeeTx, SdkError> {
        let nonce = self.nonce.ok_or(SdkError::MissingField("nonce".to_string()))?;
        let gas_limit = self.gas_limit.ok_or(SdkError::MissingField("gas_limit".to_string()))?;
        let max_fee = self.max_fee_per_gas.ok_or(SdkError::MissingField("max_fee_per_gas".to_string()))?;
        let max_priority = self.max_priority_fee_per_gas.ok_or(SdkError::MissingField("max_priority_fee_per_gas".to_string()))?;

        Ok(DynamicFeeTx {
            chain_id: self.chain_id,
            nonce,
            max_priority_fee_per_gas: max_priority,
            max_fee_per_gas: max_fee,
            gas_limit,
            to: self.to,
            value: self.value,
            data: self.data.clone(),
            access_list: vec![],
        })
    }

    /// Sign and build a legacy transaction
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields are missing (nonce, gas_limit, gas_price)
    /// - Chain ID is 0 (replay protection requires valid chain ID)
    pub fn sign_legacy(&self, wallet: &Wallet) -> Result<SignedTransaction, SdkError> {
        // Validate chain ID for replay protection (EIP-155)
        if self.chain_id == 0 {
            return Err(SdkError::InvalidChainId(
                "Chain ID cannot be 0 - replay protection requires a valid chain ID".to_string(),
            ));
        }
        let tx = self.build_legacy()?;
        sign_legacy_tx(&tx, self.chain_id, wallet)
    }

    /// Sign and build an EIP-1559 transaction
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields are missing (nonce, gas_limit, max fees)
    /// - Chain ID is 0 (replay protection requires valid chain ID)
    pub fn sign_eip1559(&self, wallet: &Wallet) -> Result<SignedTransaction, SdkError> {
        // Validate chain ID for replay protection
        if self.chain_id == 0 {
            return Err(SdkError::InvalidChainId(
                "Chain ID cannot be 0 - replay protection requires a valid chain ID".to_string(),
            ));
        }
        let tx = self.build_eip1559()?;
        sign_eip1559_tx(&tx, wallet)
    }
}

/// Sign a legacy transaction
fn sign_legacy_tx(tx: &LegacyTx, chain_id: u64, wallet: &Wallet) -> Result<SignedTransaction, SdkError> {
    // Compute the signing hash (EIP-155)
    let hash = compute_legacy_signing_hash(tx, chain_id);

    // Sign the hash
    let signature = sign(&hash, wallet.private_key())?;

    // Compute EIP-155 v value
    let v = (signature.v as u64) + chain_id * 2 + 35 - 27;

    let tx_sig = TxSignature::new(
        v,
        H256::from_bytes(signature.r),
        H256::from_bytes(signature.s),
    );

    Ok(SignedTransaction::new_legacy(tx.clone(), tx_sig))
}

/// Sign an EIP-1559 transaction
fn sign_eip1559_tx(tx: &DynamicFeeTx, wallet: &Wallet) -> Result<SignedTransaction, SdkError> {
    // Compute the signing hash
    let hash = compute_eip1559_signing_hash(tx);

    // Sign the hash
    let signature = sign(&hash, wallet.private_key())?;

    // EIP-1559 uses v = 0 or 1
    let v = if signature.v >= 27 {
        (signature.v - 27) as u64
    } else {
        signature.v as u64
    };

    let tx_sig = TxSignature::new(
        v,
        H256::from_bytes(signature.r),
        H256::from_bytes(signature.s),
    );

    Ok(SignedTransaction::new_dynamic_fee(tx.clone(), tx_sig))
}

/// Compute signing hash for legacy transaction (EIP-155)
fn compute_legacy_signing_hash(tx: &LegacyTx, chain_id: u64) -> H256 {
    // Simplified: hash the transaction fields + chain_id
    // In production, this would use proper RLP encoding
    let mut data = Vec::new();

    // Nonce
    data.extend_from_slice(&tx.nonce.to_be_bytes());
    // Gas price
    data.extend_from_slice(&tx.gas_price.to_be_bytes());
    // Gas limit
    data.extend_from_slice(&tx.gas_limit.to_be_bytes());
    // To
    if let Some(to) = &tx.to {
        data.extend_from_slice(to.as_bytes());
    }
    // Value
    data.extend_from_slice(&tx.value.to_be_bytes());
    // Data
    data.extend_from_slice(&tx.data);
    // Chain ID (EIP-155)
    data.extend_from_slice(&chain_id.to_be_bytes());
    // 0, 0 for EIP-155
    data.push(0);
    data.push(0);

    keccak256(&data)
}

/// Compute signing hash for EIP-1559 transaction
fn compute_eip1559_signing_hash(tx: &DynamicFeeTx) -> H256 {
    // Simplified: hash the transaction fields
    // In production, this would use proper RLP encoding with type prefix
    let mut data = Vec::new();

    // Type prefix
    data.push(0x02);
    // Chain ID
    data.extend_from_slice(&tx.chain_id.to_be_bytes());
    // Nonce
    data.extend_from_slice(&tx.nonce.to_be_bytes());
    // Max priority fee
    data.extend_from_slice(&tx.max_priority_fee_per_gas.to_be_bytes());
    // Max fee
    data.extend_from_slice(&tx.max_fee_per_gas.to_be_bytes());
    // Gas limit
    data.extend_from_slice(&tx.gas_limit.to_be_bytes());
    // To
    if let Some(to) = &tx.to {
        data.extend_from_slice(to.as_bytes());
    }
    // Value
    data.extend_from_slice(&tx.value.to_be_bytes());
    // Data
    data.extend_from_slice(&tx.data);

    keccak256(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_wallet() -> Wallet {
        Wallet::from_private_key_hex(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        )
        .unwrap()
    }

    #[test]
    fn test_tx_builder_legacy() {
        let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

        let tx = TxBuilder::new(1)
            .nonce(0)
            .gas_limit(21000)
            .gas_price(1_000_000_000)
            .to(to)
            .value(1_000_000_000_000_000_000)
            .build_legacy()
            .unwrap();

        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.gas_limit, 21000);
        assert_eq!(tx.gas_price, 1_000_000_000);
        assert_eq!(tx.to, Some(to));
        assert_eq!(tx.value, 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_tx_builder_eip1559() {
        let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

        let tx = TxBuilder::new(1)
            .nonce(0)
            .gas_limit(21000)
            .max_fee_per_gas(100_000_000_000)
            .max_priority_fee_per_gas(2_000_000_000)
            .to(to)
            .value(1_000_000_000_000_000_000)
            .build_eip1559()
            .unwrap();

        assert_eq!(tx.chain_id, 1);
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.gas_limit, 21000);
        assert_eq!(tx.max_fee_per_gas, 100_000_000_000);
        assert_eq!(tx.max_priority_fee_per_gas, 2_000_000_000);
    }

    #[test]
    fn test_tx_builder_missing_nonce() {
        let result = TxBuilder::new(1)
            .gas_limit(21000)
            .gas_price(1_000_000_000)
            .build_legacy();

        assert!(result.is_err());
    }

    #[test]
    fn test_tx_builder_sign_legacy() {
        let wallet = test_wallet();
        let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

        let signed = TxBuilder::new(1)
            .nonce(0)
            .gas_limit(21000)
            .gas_price(1_000_000_000)
            .to(to)
            .value(1_000_000_000_000_000_000)
            .sign_legacy(&wallet)
            .unwrap();

        assert!(signed.signature.is_valid());
        assert_eq!(signed.nonce(), 0);
        assert_eq!(signed.gas_limit(), 21000);
    }

    #[test]
    fn test_tx_builder_sign_eip1559() {
        let wallet = test_wallet();
        let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

        let signed = TxBuilder::new(1)
            .nonce(0)
            .gas_limit(21000)
            .max_fee_per_gas(100_000_000_000)
            .max_priority_fee_per_gas(2_000_000_000)
            .to(to)
            .value(1_000_000_000_000_000_000)
            .sign_eip1559(&wallet)
            .unwrap();

        assert!(signed.signature.is_valid());
        assert_eq!(signed.nonce(), 0);
        assert_eq!(signed.gas_limit(), 21000);
    }

    #[test]
    fn test_tx_builder_contract_creation() {
        let wallet = test_wallet();
        let bytecode = Bytes::from(vec![0x60, 0x80, 0x60, 0x40]);

        let signed = TxBuilder::new(1)
            .nonce(0)
            .gas_limit(1_000_000)
            .gas_price(1_000_000_000)
            .data(bytecode)
            .sign_legacy(&wallet)
            .unwrap();

        assert!(signed.is_contract_creation());
    }

    #[test]
    fn test_tx_builder_with_data() {
        let wallet = test_wallet();
        let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let data = Bytes::from(vec![0xa9, 0x05, 0x9c, 0xbb]); // transfer selector

        let signed = TxBuilder::new(1)
            .nonce(0)
            .gas_limit(100_000)
            .gas_price(1_000_000_000)
            .to(to)
            .data(data.clone())
            .sign_legacy(&wallet)
            .unwrap();

        assert_eq!(signed.data(), &data);
    }

    #[test]
    fn test_tx_builder_rejects_zero_chain_id_legacy() {
        let wallet = test_wallet();
        let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

        let result = TxBuilder::new(0) // Invalid chain ID
            .nonce(0)
            .gas_limit(21000)
            .gas_price(1_000_000_000)
            .to(to)
            .sign_legacy(&wallet);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Chain ID"));
    }

    #[test]
    fn test_tx_builder_rejects_zero_chain_id_eip1559() {
        let wallet = test_wallet();
        let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

        let result = TxBuilder::new(0) // Invalid chain ID
            .nonce(0)
            .gas_limit(21000)
            .max_fee_per_gas(100_000_000_000)
            .max_priority_fee_per_gas(2_000_000_000)
            .to(to)
            .sign_eip1559(&wallet);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Chain ID"));
    }
}
