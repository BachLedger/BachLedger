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

    // ==================== Basic sign/verify tests ====================

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
        assert_eq!(address.as_bytes().len(), 20);
    }

    // ==================== EIP-2 low-s signature malleability tests ====================

    #[test]
    fn test_low_s_enforcement() {
        // Create multiple signatures and verify they all have low-s
        for _ in 0..20 {
            let private_key = SigningKey::random(&mut OsRng);
            let message_hash = keccak256(b"EIP-2 low-s test");
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

    #[test]
    fn test_signature_s_boundary() {
        // Test that s is strictly <= n/2
        for _ in 0..10 {
            let private_key = SigningKey::random(&mut OsRng);
            let message_hash = keccak256(b"boundary test");
            let signature = sign(&message_hash, &private_key).unwrap();

            // s should be <= n/2
            let cmp = compare_bytes(&signature.s, &SECP256K1_N_DIV_2);
            assert!(
                cmp != std::cmp::Ordering::Greater,
                "s should be <= n/2"
            );
        }
    }

    #[test]
    fn test_signature_components_not_zero() {
        for _ in 0..10 {
            let private_key = SigningKey::random(&mut OsRng);
            let message_hash = keccak256(b"zero component test");
            let signature = sign(&message_hash, &private_key).unwrap();

            assert_ne!(signature.r, [0u8; 32], "r should not be zero");
            assert_ne!(signature.s, [0u8; 32], "s should not be zero");
        }
    }

    // ==================== Sign/verify/recover roundtrip tests ====================

    #[test]
    fn test_sign_verify_recover_roundtrip() {
        for _ in 0..10 {
            let private_key = SigningKey::random(&mut OsRng);
            let public_key = private_key.verifying_key();
            let expected_address = public_key_to_address(public_key);

            let message_hash = keccak256(b"roundtrip test");
            let signature = sign(&message_hash, &private_key).unwrap();

            // Verify
            assert!(verify(&message_hash, &signature, public_key).unwrap());

            // Recover
            let recovered_pubkey = recover_public_key(&message_hash, &signature).unwrap();
            let recovered_address = public_key_to_address(&recovered_pubkey);

            assert_eq!(expected_address, recovered_address);
        }
    }

    #[test]
    fn test_multiple_messages_same_key() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();

        let messages: &[&[u8]] = &[
            b"message 1",
            b"message 2",
            b"message 3",
            b"",
            &[0u8; 100],
        ];

        for msg in messages {
            let hash = keccak256(msg);
            let sig = sign(&hash, &private_key).unwrap();
            assert!(verify(&hash, &sig, public_key).unwrap());
        }
    }

    // ==================== Recovery ID tests ====================

    #[test]
    fn test_recovery_id_values() {
        let private_key = SigningKey::random(&mut OsRng);
        let message_hash = keccak256(b"recovery id test");
        let signature = sign(&message_hash, &private_key).unwrap();

        // v should be 27 or 28 (Ethereum format)
        assert!(
            signature.v == 27 || signature.v == 28,
            "v should be 27 or 28, got {}",
            signature.v
        );

        // recovery_id() should return 0 or 1
        let rec_id = signature.recovery_id();
        assert!(rec_id <= 1, "recovery_id should be 0 or 1, got {}", rec_id);
    }

    #[test]
    fn test_recovery_id_consistency() {
        for _ in 0..10 {
            let private_key = SigningKey::random(&mut OsRng);
            let message_hash = keccak256(b"consistency test");
            let signature = sign(&message_hash, &private_key).unwrap();

            // Test both v formats
            if signature.v >= 27 {
                assert_eq!(signature.recovery_id(), signature.v - 27);
            } else {
                assert_eq!(signature.recovery_id(), signature.v);
            }
        }
    }

    // ==================== Signature serialization tests ====================

    #[test]
    fn test_signature_to_bytes_roundtrip() {
        let private_key = SigningKey::random(&mut OsRng);
        let message_hash = keccak256(b"serialization test");
        let signature = sign(&message_hash, &private_key).unwrap();

        let bytes = signature.to_bytes();
        assert_eq!(bytes.len(), 65);

        let recovered = Signature::from_bytes(&bytes);
        assert_eq!(signature.r, recovered.r);
        assert_eq!(signature.s, recovered.s);
        assert_eq!(signature.v, recovered.v);
    }

    #[test]
    fn test_signature_new() {
        let r = [0x11; 32];
        let s = [0x22; 32];
        let v = 27u8;

        let sig = Signature::new(r, s, v);
        assert_eq!(sig.r, r);
        assert_eq!(sig.s, s);
        assert_eq!(sig.v, v);
    }

    #[test]
    fn test_signature_equality() {
        let r = [0x11; 32];
        let s = [0x22; 32];

        let sig1 = Signature::new(r, s, 27);
        let sig2 = Signature::new(r, s, 27);
        let sig3 = Signature::new(r, s, 28);

        assert_eq!(sig1, sig2);
        assert_ne!(sig1, sig3);
    }

    // ==================== Verification failure tests ====================

    #[test]
    fn test_verify_wrong_message() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();

        let message_hash = keccak256(b"original message");
        let signature = sign(&message_hash, &private_key).unwrap();

        let wrong_hash = keccak256(b"different message");
        assert!(!verify(&wrong_hash, &signature, public_key).unwrap());
    }

    #[test]
    fn test_verify_wrong_public_key() {
        let private_key1 = SigningKey::random(&mut OsRng);
        let private_key2 = SigningKey::random(&mut OsRng);
        let public_key2 = private_key2.verifying_key();

        let message_hash = keccak256(b"test message");
        let signature = sign(&message_hash, &private_key1).unwrap();

        // Verification with wrong key should fail
        assert!(!verify(&message_hash, &signature, public_key2).unwrap());
    }

    #[test]
    fn test_verify_tampered_signature_r() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();

        let message_hash = keccak256(b"tamper test");
        let mut signature = sign(&message_hash, &private_key).unwrap();

        // Tamper with r
        signature.r[0] ^= 0x01;

        // Verification should fail or return an error
        let result = verify(&message_hash, &signature, public_key);
        assert!(result.is_err() || !result.unwrap());
    }

    #[test]
    fn test_verify_tampered_signature_s() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();

        let message_hash = keccak256(b"tamper test");
        let mut signature = sign(&message_hash, &private_key).unwrap();

        // Tamper with s (but keep it low to pass the EIP-2 check)
        signature.s[31] ^= 0x01;

        // Verification should fail
        let result = verify(&message_hash, &signature, public_key);
        assert!(result.is_err() || !result.unwrap());
    }

    // ==================== Recovery failure tests ====================

    #[test]
    fn test_recover_invalid_recovery_id() {
        let private_key = SigningKey::random(&mut OsRng);
        let message_hash = keccak256(b"recovery test");
        let mut signature = sign(&message_hash, &private_key).unwrap();

        // Set invalid recovery ID
        signature.v = 30; // Invalid: should be 27 or 28

        let result = recover_public_key(&message_hash, &signature);
        assert!(result.is_err());
        // Accept either InvalidRecoveryId or RecoveryFailed error
        match result {
            Err(CryptoError::InvalidRecoveryId(_)) => {}
            Err(CryptoError::RecoveryFailed(_)) => {}
            Err(e) => panic!("Expected InvalidRecoveryId or RecoveryFailed, got {:?}", e),
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_recover_tampered_signature() {
        let private_key = SigningKey::random(&mut OsRng);
        let original_pubkey = private_key.verifying_key();
        let original_address = public_key_to_address(original_pubkey);

        let message_hash = keccak256(b"recovery tamper test");
        let mut signature = sign(&message_hash, &private_key).unwrap();

        // Tamper with r
        signature.r[15] ^= 0xff;

        // Recovery might succeed but return wrong public key
        if let Ok(recovered) = recover_public_key(&message_hash, &signature) {
            let recovered_address = public_key_to_address(&recovered);
            assert_ne!(original_address, recovered_address);
        }
        // Or it might fail, which is also acceptable
    }

    // ==================== Address derivation tests ====================

    #[test]
    fn test_address_from_known_private_key() {
        // Known test vector
        // Private key: 0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
        let pk_bytes = hex::decode(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        ).unwrap();

        let private_key = SigningKey::from_slice(&pk_bytes).unwrap();
        let public_key = private_key.verifying_key();
        let address = public_key_to_address(public_key);

        // The address derived from this key should be consistent
        let address_hex = address.to_hex();
        assert!(address_hex.starts_with("0x"));
        assert_eq!(address_hex.len(), 42); // "0x" + 40 hex chars
    }

    #[test]
    fn test_address_determinism() {
        let pk_bytes = hex::decode(
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        ).unwrap();

        let private_key = SigningKey::from_slice(&pk_bytes).unwrap();
        let public_key = private_key.verifying_key();

        let address1 = public_key_to_address(public_key);
        let address2 = public_key_to_address(public_key);

        assert_eq!(address1, address2);
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_sign_zero_hash() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();

        let zero_hash = H256::ZERO;
        let result = sign(&zero_hash, &private_key);

        // Should succeed
        assert!(result.is_ok());
        let signature = result.unwrap();

        // Should be verifiable
        assert!(verify(&zero_hash, &signature, public_key).unwrap());
    }

    #[test]
    fn test_sign_max_hash() {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key.verifying_key();

        let max_hash = H256::from_bytes([0xff; 32]);
        let result = sign(&max_hash, &private_key);

        // Should succeed
        assert!(result.is_ok());
        let signature = result.unwrap();

        // Should be verifiable
        assert!(verify(&max_hash, &signature, public_key).unwrap());
    }

    // ==================== Known Ethereum test vectors ====================

    #[test]
    fn test_ethereum_personal_sign_format() {
        // Ethereum "personal_sign" prefix
        let message = b"Hello, Ethereum!";
        let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
        let mut data = prefix.into_bytes();
        data.extend_from_slice(message);

        let hash = keccak256(&data);

        // The hash should be 32 bytes and deterministic
        assert_eq!(hash.as_bytes().len(), 32);

        // Hash should be consistent
        let hash2 = keccak256(&data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_known_private_key_address_derivation() {
        // Well-known test private key (DO NOT USE IN PRODUCTION)
        // This is the "test test test..." mnemonic first account
        let pk_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let pk_bytes = hex::decode(pk_hex).unwrap();

        let private_key = SigningKey::from_slice(&pk_bytes).unwrap();
        let public_key = private_key.verifying_key();
        let address = public_key_to_address(public_key);

        // Expected address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
        assert_eq!(
            address.to_hex(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    // ==================== Helper function tests ====================

    #[test]
    fn test_compare_bytes() {
        let a = [0u8; 32];
        let b = [0u8; 32];
        assert_eq!(compare_bytes(&a, &b), std::cmp::Ordering::Equal);

        let mut c = [0u8; 32];
        c[0] = 1;
        assert_eq!(compare_bytes(&c, &a), std::cmp::Ordering::Greater);
        assert_eq!(compare_bytes(&a, &c), std::cmp::Ordering::Less);

        let mut d = [0u8; 32];
        d[31] = 1;
        assert_eq!(compare_bytes(&d, &a), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_subtract_from_n() {
        // n - 0 = n
        let zero = [0u8; 32];
        let result = subtract_from_n(&zero);
        assert_eq!(result, SECP256K1_N);

        // n - 1 = n - 1
        let mut one = [0u8; 32];
        one[31] = 1;
        let result = subtract_from_n(&one);
        let mut expected = SECP256K1_N;
        expected[31] -= 1;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_is_low_s() {
        // s = 0 should be low-s
        let sig_zero_s = Signature::new([0; 32], [0; 32], 27);
        assert!(sig_zero_s.is_low_s());

        // s = n/2 should be low-s (boundary)
        let sig_half_n = Signature::new([0; 32], SECP256K1_N_DIV_2, 27);
        assert!(sig_half_n.is_low_s());

        // s > n/2 should not be low-s
        let mut high_s = SECP256K1_N_DIV_2;
        high_s[31] += 1;
        let sig_high_s = Signature::new([0; 32], high_s, 27);
        assert!(!sig_high_s.is_low_s());

        // s = max value should not be low-s
        let sig_max = Signature::new([0; 32], [0xff; 32], 27);
        assert!(!sig_max.is_low_s());
    }
}
