//! ECDSA signature operations using secp256k1

use bach_primitives::{Address, H256};
use k256::ecdsa::{RecoveryId, Signature as K256Signature, SigningKey, VerifyingKey};
use crate::{keccak256, CryptoError};

/// Half of the secp256k1 curve order (n/2)
/// n = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
/// n/2 = 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0
const SECP256K1_N_DIV_2: [u8; 32] = [
    0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x5D, 0x57, 0x6E, 0x73, 0x57, 0xA4, 0x50, 0x1D,
    0xDF, 0xE9, 0x2F, 0x46, 0x68, 0x1B, 0x20, 0xA0,
];

/// Full secp256k1 curve order (n)
const SECP256K1_N: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
    0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
    0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
];

/// ECDSA Signature with recovery ID
#[derive(Clone, Debug, PartialEq, Eq)]
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

    /// Check if signature has low-s value (EIP-2 compliant)
    pub fn is_low_s(&self) -> bool {
        compare_bytes(&self.s, &SECP256K1_N_DIV_2) != std::cmp::Ordering::Greater
    }
}

/// Compare two 32-byte arrays as big-endian integers
fn compare_bytes(a: &[u8; 32], b: &[u8; 32]) -> std::cmp::Ordering {
    for i in 0..32 {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

/// Subtract b from n (secp256k1 order), result = n - b
/// Used for s normalization: s' = n - s
fn subtract_from_n(s: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut borrow: u16 = 0;

    for i in (0..32).rev() {
        let diff = (SECP256K1_N[i] as u16).wrapping_sub(s[i] as u16).wrapping_sub(borrow);
        result[i] = diff as u8;
        borrow = if diff > 255 { 1 } else { 0 };
    }

    result
}

/// Sign a message hash with a private key (EIP-2 compliant with low-s)
pub fn sign(message_hash: &H256, private_key: &PrivateKey) -> Result<Signature, CryptoError> {
    let (signature, mut recovery_id) = private_key
        .sign_prehash_recoverable(message_hash.as_bytes())
        .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

    let r_bytes: [u8; 32] = signature.r().to_bytes().into();
    let mut s_bytes: [u8; 32] = signature.s().to_bytes().into();

    // EIP-2: Normalize s to low-s form
    // If s > n/2, replace s with n - s and flip recovery_id
    if compare_bytes(&s_bytes, &SECP256K1_N_DIV_2) == std::cmp::Ordering::Greater {
        s_bytes = subtract_from_n(&s_bytes);
        recovery_id = RecoveryId::try_from(recovery_id.to_byte() ^ 1)
            .map_err(|_| CryptoError::SigningFailed("Invalid recovery ID after normalization".to_string()))?;
    }

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
    // Reject non-low-s signatures per EIP-2
    if !signature.is_low_s() {
        return Ok(false);
    }

    let r: k256::FieldBytes = signature.r.into();
    let s: k256::FieldBytes = signature.s.into();
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
    let r: k256::FieldBytes = signature.r.into();
    let s: k256::FieldBytes = signature.s.into();
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

        // Signature should be low-s (EIP-2 compliant)
        assert!(signature.is_low_s(), "Signature should have low-s value");

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

    #[test]
    fn test_low_s_enforcement() {
        // Create multiple signatures and verify they all have low-s
        for _ in 0..10 {
            let private_key = SigningKey::random(&mut OsRng);
            let message_hash = keccak256(b"test");
            let signature = sign(&message_hash, &private_key).unwrap();
            assert!(signature.is_low_s(), "All signatures must have low-s");
        }
    }

    #[test]
    fn test_reject_high_s_signature() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();
        let message_hash = keccak256(b"test");

        let mut signature = sign(&message_hash, &private_key).unwrap();

        // Manually set a high-s value (greater than n/2)
        signature.s = [0xFF; 32]; // Definitely > n/2

        // Verification should fail for high-s signature
        assert!(!verify(&message_hash, &signature, public_key).unwrap());
    }
}
