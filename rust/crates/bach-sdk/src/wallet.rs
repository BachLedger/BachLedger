//! Wallet and account management

use bach_crypto::{
    keccak256, public_key_to_address, sign, PrivateKey, PublicKey, Signature,
};
use bach_primitives::{Address, H256};
use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;
use zeroize::Zeroize;

use crate::SdkError;

/// Wallet for managing private keys and signing
///
/// Note: Clone is intentionally not implemented to prevent accidental key duplication.
/// Use `from_private_key` to create a new wallet with the same key if needed.
pub struct Wallet {
    private_key: PrivateKey,
    address: Address,
}

impl Wallet {
    /// Create a new random wallet
    pub fn new_random() -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();
        let address = public_key_to_address(public_key);

        Self {
            private_key,
            address,
        }
    }

    /// Create a wallet from a 32-byte private key
    pub fn from_private_key(key: &[u8; 32]) -> Result<Self, SdkError> {
        let private_key = SigningKey::from_slice(key)
            .map_err(|e| SdkError::InvalidPrivateKey(e.to_string()))?;
        let public_key = private_key.verifying_key();
        let address = public_key_to_address(public_key);

        Ok(Self {
            private_key,
            address,
        })
    }

    /// Create a wallet from a hex-encoded private key
    ///
    /// Accepts both with and without "0x" prefix.
    pub fn from_private_key_hex(hex: &str) -> Result<Self, SdkError> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        let mut bytes = hex::decode(hex)?;
        if bytes.len() != 32 {
            bytes.zeroize(); // Clear sensitive data before returning error
            return Err(SdkError::InvalidPrivateKey(format!(
                "Expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        bytes.zeroize(); // Clear the intermediate vector

        let result = Self::from_private_key(&key);
        key.zeroize(); // Clear the array after use
        result
    }

    /// Get the wallet's address
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Get the wallet's public key
    pub fn public_key(&self) -> &PublicKey {
        self.private_key.verifying_key()
    }

    /// Get the private key reference (for internal use)
    pub(crate) fn private_key(&self) -> &PrivateKey {
        &self.private_key
    }

    /// Sign a message hash (32 bytes)
    pub fn sign_hash(&self, hash: &H256) -> Result<Signature, SdkError> {
        sign(hash, &self.private_key).map_err(|e| SdkError::SigningFailed(e.to_string()))
    }

    /// Sign a message with Ethereum personal sign prefix
    ///
    /// Prefixes the message with "\x19Ethereum Signed Message:\n{len}"
    pub fn sign_message(&self, message: &[u8]) -> Result<Signature, SdkError> {
        let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
        let mut data = prefix.into_bytes();
        data.extend_from_slice(message);
        let hash = keccak256(&data);
        self.sign_hash(&hash)
    }
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_random() {
        let wallet = Wallet::new_random();
        assert_ne!(wallet.address(), &Address::ZERO);
    }

    #[test]
    fn test_wallet_from_private_key() {
        let key = [0x42u8; 32];
        let wallet = Wallet::from_private_key(&key).unwrap();
        assert_ne!(wallet.address(), &Address::ZERO);
    }

    #[test]
    fn test_wallet_from_hex() {
        // Known test key
        let wallet = Wallet::from_private_key_hex(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        )
        .unwrap();

        assert_eq!(
            wallet.address().to_hex(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    #[test]
    fn test_wallet_from_hex_no_prefix() {
        let wallet = Wallet::from_private_key_hex(
            "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        )
        .unwrap();

        assert_eq!(
            wallet.address().to_hex(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    #[test]
    fn test_wallet_invalid_hex_length() {
        let result = Wallet::from_private_key_hex("0x1234");
        assert!(result.is_err());
    }

    #[test]
    fn test_wallet_sign_hash() {
        let wallet = Wallet::new_random();
        let hash = H256::from_bytes([0x42; 32]);
        let signature = wallet.sign_hash(&hash).unwrap();

        // Verify signature components
        assert_ne!(signature.r, [0u8; 32]);
        assert_ne!(signature.s, [0u8; 32]);
        assert!(signature.v == 27 || signature.v == 28);
    }

    #[test]
    fn test_wallet_sign_message() {
        let wallet = Wallet::new_random();
        let message = b"Hello, BachLedger!";
        let signature = wallet.sign_message(message).unwrap();

        assert_ne!(signature.r, [0u8; 32]);
        assert_ne!(signature.s, [0u8; 32]);
    }

    #[test]
    fn test_wallet_determinism() {
        let key = [0x42u8; 32];
        let wallet1 = Wallet::from_private_key(&key).unwrap();
        let wallet2 = Wallet::from_private_key(&key).unwrap();

        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_wallet_debug_hides_key() {
        let wallet = Wallet::new_random();
        let debug = format!("{:?}", wallet);
        assert!(debug.contains("Wallet"));
        assert!(debug.contains("address"));
        // Should not expose the private key
        assert!(!debug.contains("private_key"));
    }
}
