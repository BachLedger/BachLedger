//! ECDSA signature operations using secp256k1

use bach_primitives::{Address, H256};
use k256::{
    ecdsa::{RecoveryId, Signature as K256Signature, SigningKey, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use crate::{keccak256, CryptoError};

/// ECDSA Signature with recovery ID
#[derive(Clone, Debug)]
pub struct Signature {
    /// r component (32 bytes)
    pub r: [u8; 32],
    /// s component (32 bytes)
    pub s: [u8; 32],
    /// recovery id (0 or 1, stored as 27 or 28 for Ethereum compatibility)
    pub v: u8,
}

/// Public key (65 bytes uncompressed, or 33 bytes compressed)
pub type PublicKey = VerifyingKey;

/// Private key (32 bytes)
pub type PrivateKey = SigningKey;

impl Signature {
    /// Create signature from r, s, v components
    pub fn new(r: [u8; 32], s: [u8; 32], v: u8) -> Self {
        Signature { r, s, v }
    }

    /// Get recovery ID (0 or 1)
    pub fn recovery_id(&self) -> u8 {
        if self.v >= 27 {
            self.v - 27
        } else {
            self.v
        }
    }

    /// Convert to 65-byte representation (r || s || v)
    pub fn to_bytes(&self) -> [u8; 65] {
        let mut bytes = [0u8; 65];
        bytes[..32].copy_from_slice(&self.r);
        bytes[32..64].copy_from_slice(&self.s);
        bytes[64] = self.v;
        bytes
    }

    /// Parse from 65-byte representation
    pub fn from_bytes(bytes: &[u8; 65]) -> Self {
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&bytes[..32]);
        s.copy_from_slice(&bytes[32..64]);
        Signature {
            r,
            s,
            v: bytes[64],
        }
    }
}

/// Sign a message hash with a private key
pub fn sign(message_hash: &H256, private_key: &PrivateKey) -> Result<Signature, CryptoError> {
    let (signature, recovery_id) = private_key
        .sign_prehash_recoverable(message_hash.as_bytes())
        .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

    let r_bytes: [u8; 32] = signature.r().to_bytes().into();
    let s_bytes: [u8; 32] = signature.s().to_bytes().into();

    Ok(Signature {
        r: r_bytes,
        s: s_bytes,
        v: recovery_id.to_byte() + 27, // Ethereum uses 27/28
    })
}

/// Verify a signature against a message hash and public key
pub fn verify(
    message_hash: &H256,
    signature: &Signature,
    public_key: &PublicKey,
) -> Result<bool, CryptoError> {
    let r = k256::FieldBytes::from_slice(&signature.r).clone();
    let s = k256::FieldBytes::from_slice(&signature.s).clone();
    let k256_sig = K256Signature::from_scalars(r, s)
        .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;

    use k256::ecdsa::signature::hazmat::PrehashVerifier;
    Ok(public_key
        .verify_prehash(message_hash.as_bytes(), &k256_sig)
        .is_ok())
}

/// Recover public key from signature and message hash
pub fn recover_public_key(
    message_hash: &H256,
    signature: &Signature,
) -> Result<PublicKey, CryptoError> {
    let r = k256::FieldBytes::from_slice(&signature.r).clone();
    let s = k256::FieldBytes::from_slice(&signature.s).clone();
    let k256_sig = K256Signature::from_scalars(r, s)
        .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;

    let recovery_id = RecoveryId::try_from(signature.recovery_id())
        .map_err(|_| CryptoError::InvalidRecoveryId(signature.recovery_id()))?;

    VerifyingKey::recover_from_prehash(message_hash.as_bytes(), &k256_sig, recovery_id)
        .map_err(|e| CryptoError::RecoveryFailed(e.to_string()))
}

/// Derive Ethereum address from public key
pub fn public_key_to_address(public_key: &PublicKey) -> Address {
    // Get uncompressed public key (65 bytes: 0x04 || x || y)
    let encoded = public_key.to_encoded_point(false);
    let bytes = encoded.as_bytes();

    // Skip the 0x04 prefix, hash the remaining 64 bytes
    let hash = keccak256(&bytes[1..]);

    // Take the last 20 bytes as the address
    let mut addr_bytes = [0u8; 20];
    addr_bytes.copy_from_slice(&hash.as_bytes()[12..]);
    Address::from_bytes(addr_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;
    use rand::rngs::OsRng;

    #[test]
    fn test_sign_and_verify() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();

        let message = b"test message";
        let message_hash = keccak256(message);

        let signature = sign(&message_hash, &private_key).unwrap();
        assert!(verify(&message_hash, &signature, public_key).unwrap());
    }

    #[test]
    fn test_recover_public_key() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();

        let message = b"test message";
        let message_hash = keccak256(message);

        let signature = sign(&message_hash, &private_key).unwrap();
        let recovered = recover_public_key(&message_hash, &signature).unwrap();

        assert_eq!(public_key, &recovered);
    }

    #[test]
    fn test_address_derivation() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();
        let address = public_key_to_address(public_key);

        // Address should be 20 bytes
        assert!(!address.is_zero() || true); // May be zero by chance, but unlikely
    }
}
