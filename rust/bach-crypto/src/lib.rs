//! BachLedger Crypto
//!
//! Cryptographic primitives for blockchain operations:
//! - `keccak256`: Keccak-256 hash function
//! - `PrivateKey`: secp256k1 private key
//! - `PublicKey`: secp256k1 public key
//! - `Signature`: ECDSA signature with recovery ID

use bach_primitives::{Address, H256, ADDRESS_LENGTH};
use k256::ecdsa::{
    RecoveryId, Signature as K256Signature, SigningKey, VerifyingKey,
    signature::DigestVerifier,
};
use sha3::{Digest, Keccak256};

/// Length of a signature in bytes (r=32 + s=32 + v=1)
pub const SIGNATURE_LENGTH: usize = 65;

/// Errors from cryptographic operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    /// Private key bytes are not a valid scalar
    InvalidPrivateKey,
    /// Signature bytes are malformed
    InvalidSignature,
    /// Public key recovery failed
    RecoveryFailed,
    /// Public key is invalid
    InvalidPublicKey,
}

/// Computes the Keccak-256 hash of the input.
pub fn keccak256(data: &[u8]) -> H256 {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    H256::from(bytes)
}

/// Computes the Keccak-256 hash of concatenated inputs.
pub fn keccak256_concat(data: &[&[u8]]) -> H256 {
    let mut hasher = Keccak256::new();
    for slice in data {
        hasher.update(slice);
    }
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    H256::from(bytes)
}

/// A secp256k1 private key (32 bytes).
pub struct PrivateKey {
    inner: SigningKey,
}

impl PrivateKey {
    /// Generates a random private key using OS entropy.
    pub fn random() -> Self {
        let inner = SigningKey::random(&mut rand_core::OsRng);
        Self { inner }
    }

    /// Creates a private key from raw bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        let inner = SigningKey::from_bytes(bytes.into())
            .map_err(|_| CryptoError::InvalidPrivateKey)?;
        Ok(Self { inner })
    }

    /// Returns the raw bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.inner.to_bytes().into()
    }

    /// Derives the corresponding public key.
    pub fn public_key(&self) -> PublicKey {
        let verifying_key = self.inner.verifying_key();
        PublicKey::from_verifying_key(verifying_key)
    }

    /// Signs a message hash.
    pub fn sign(&self, message: &H256) -> Signature {
        // Message is already a hash - use prehash signing (no double-hashing)
        let (sig, recovery_id) = self.inner.sign_prehash_recoverable(message.as_bytes())
            .expect("signing should not fail with valid key");

        Signature::from_k256_signature(&sig, recovery_id)
    }
}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivateKey")
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

/// A secp256k1 public key (uncompressed, 64 bytes without prefix).
#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey {
    bytes: [u8; 64],
}

impl PublicKey {
    /// Creates from the k256 VerifyingKey
    fn from_verifying_key(key: &VerifyingKey) -> Self {
        let encoded = key.to_encoded_point(false);
        let uncompressed = encoded.as_bytes();
        // Skip the 0x04 prefix
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(&uncompressed[1..65]);
        Self { bytes }
    }

    /// Creates from uncompressed bytes (64 bytes, no 0x04 prefix).
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self, CryptoError> {
        // Verify it's a valid point by trying to create a VerifyingKey
        let mut with_prefix = [0u8; 65];
        with_prefix[0] = 0x04;
        with_prefix[1..65].copy_from_slice(bytes);

        VerifyingKey::from_sec1_bytes(&with_prefix)
            .map_err(|_| CryptoError::InvalidPublicKey)?;

        Ok(Self { bytes: *bytes })
    }

    /// Returns the uncompressed bytes (64 bytes).
    pub fn to_bytes(&self) -> [u8; 64] {
        self.bytes
    }

    /// Derives the Ethereum-style address.
    /// Address = keccak256(public_key)[12..32]
    pub fn to_address(&self) -> Address {
        let hash = keccak256(&self.bytes);
        let hash_bytes = hash.as_bytes();
        let mut addr_bytes = [0u8; ADDRESS_LENGTH];
        addr_bytes.copy_from_slice(&hash_bytes[12..32]);
        Address::from(addr_bytes)
    }

    /// Verifies a signature against this public key.
    pub fn verify(&self, signature: &Signature, message: &H256) -> bool {
        signature.verify(self, message)
    }

    /// Get the k256 VerifyingKey
    fn to_verifying_key(&self) -> Option<VerifyingKey> {
        let mut with_prefix = [0u8; 65];
        with_prefix[0] = 0x04;
        with_prefix[1..65].copy_from_slice(&self.bytes);
        VerifyingKey::from_sec1_bytes(&with_prefix).ok()
    }
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublicKey")
            .field("bytes", &hex_encode(&self.bytes))
            .finish()
    }
}

/// Hex encoding helper for Debug
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// An ECDSA signature with recovery ID (65 bytes: r + s + v).
#[derive(Clone, PartialEq, Eq)]
pub struct Signature {
    bytes: [u8; SIGNATURE_LENGTH],
}

impl Signature {
    /// Create from k256 signature and recovery ID
    fn from_k256_signature(sig: &K256Signature, recovery_id: RecoveryId) -> Self {
        let mut bytes = [0u8; SIGNATURE_LENGTH];
        let r_bytes = sig.r().to_bytes();
        let s_bytes = sig.s().to_bytes();
        bytes[0..32].copy_from_slice(&r_bytes);
        bytes[32..64].copy_from_slice(&s_bytes);
        // Ethereum-style v = recovery_id + 27
        bytes[64] = recovery_id.to_byte() + 27;
        Self { bytes }
    }

    /// Creates a signature from raw bytes.
    pub fn from_bytes(bytes: &[u8; SIGNATURE_LENGTH]) -> Result<Self, CryptoError> {
        // Validate r and s are not zero
        let r = &bytes[0..32];
        let s = &bytes[32..64];

        if r.iter().all(|&b| b == 0) || s.iter().all(|&b| b == 0) {
            return Err(CryptoError::InvalidSignature);
        }

        // Validate v is 27 or 28
        let v = bytes[64];
        if v != 27 && v != 28 {
            return Err(CryptoError::InvalidSignature);
        }

        // Try to parse as k256 signature to validate r and s
        let r_arr: [u8; 32] = r.try_into().unwrap();
        let s_arr: [u8; 32] = s.try_into().unwrap();
        K256Signature::from_scalars(
            k256::FieldBytes::from(r_arr),
            k256::FieldBytes::from(s_arr),
        ).map_err(|_| CryptoError::InvalidSignature)?;

        Ok(Self { bytes: *bytes })
    }

    /// Returns the raw bytes (r + s + v).
    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        self.bytes
    }

    /// Verifies this signature against a public key and message.
    pub fn verify(&self, pubkey: &PublicKey, message: &H256) -> bool {
        let verifying_key = match pubkey.to_verifying_key() {
            Some(vk) => vk,
            None => return false,
        };

        let r_arr: [u8; 32] = self.bytes[0..32].try_into().unwrap();
        let s_arr: [u8; 32] = self.bytes[32..64].try_into().unwrap();
        let k256_sig = match K256Signature::from_scalars(
            k256::FieldBytes::from(r_arr),
            k256::FieldBytes::from(s_arr),
        ) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        let digest = Keccak256::new_with_prefix(message.as_bytes());
        verifying_key.verify_digest(digest, &k256_sig).is_ok()
    }

    /// Recovers the public key from the signature and message.
    pub fn recover(&self, message: &H256) -> Result<PublicKey, CryptoError> {
        let r_arr: [u8; 32] = self.bytes[0..32].try_into().unwrap();
        let s_arr: [u8; 32] = self.bytes[32..64].try_into().unwrap();
        let k256_sig = K256Signature::from_scalars(
            k256::FieldBytes::from(r_arr),
            k256::FieldBytes::from(s_arr),
        ).map_err(|_| CryptoError::InvalidSignature)?;

        // Convert Ethereum v (27 or 28) to recovery ID (0 or 1)
        let v = self.bytes[64];
        let recovery_id = RecoveryId::try_from(v.saturating_sub(27))
            .map_err(|_| CryptoError::RecoveryFailed)?;

        let digest = Keccak256::new_with_prefix(message.as_bytes());
        let recovered_key = VerifyingKey::recover_from_digest(digest, &k256_sig, recovery_id)
            .map_err(|_| CryptoError::RecoveryFailed)?;

        Ok(PublicKey::from_verifying_key(&recovered_key))
    }

    /// Returns the r component.
    pub fn r(&self) -> &[u8; 32] {
        self.bytes[0..32].try_into().unwrap()
    }

    /// Returns the s component.
    pub fn s(&self) -> &[u8; 32] {
        self.bytes[32..64].try_into().unwrap()
    }

    /// Returns the recovery ID (0 or 1, stored as 27 or 28 for Ethereum).
    pub fn v(&self) -> u8 {
        self.bytes[64]
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signature")
            .field("r", &hex_encode(self.r()))
            .field("s", &hex_encode(self.s()))
            .field("v", &self.v())
            .finish()
    }
}
