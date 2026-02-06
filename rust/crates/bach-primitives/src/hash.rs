//! Hash types (H256, H160)

use std::fmt;
use thiserror::Error;

/// Hash parsing error
#[derive(Debug, Error)]
pub enum HashError {
    /// Invalid hex string
    #[error("invalid hex string: {0}")]
    InvalidHex(String),
    /// Invalid length
    #[error("invalid hash length: expected {expected} bytes, got {got}")]
    InvalidLength { expected: usize, got: usize },
}

/// 256-bit hash (32 bytes)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct H256([u8; 32]);

/// Alias for H256
pub type Hash = H256;

impl H256 {
    /// Size in bytes
    pub const LEN: usize = 32;

    /// Zero hash
    pub const ZERO: H256 = H256([0u8; 32]);

    /// Create from bytes
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        H256(bytes)
    }

    /// Create from slice
    pub fn from_slice(slice: &[u8]) -> Result<Self, HashError> {
        if slice.len() != 32 {
            return Err(HashError::InvalidLength {
                expected: 32,
                got: slice.len(),
            });
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(H256(bytes))
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> Result<Self, HashError> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(s).map_err(|e| HashError::InvalidHex(e.to_string()))?;
        Self::from_slice(&bytes)
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

impl fmt::Debug for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "H256({})", self.to_hex())
    }
}

impl fmt::Display for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for H256 {
    fn from(bytes: [u8; 32]) -> Self {
        H256(bytes)
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// 160-bit hash (20 bytes) - same size as Address
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct H160([u8; 20]);

impl H160 {
    /// Size in bytes
    pub const LEN: usize = 20;

    /// Zero hash
    pub const ZERO: H160 = H160([0u8; 20]);

    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        H160(bytes)
    }

    /// Create from slice
    pub fn from_slice(slice: &[u8]) -> Result<Self, HashError> {
        if slice.len() != 20 {
            return Err(HashError::InvalidLength {
                expected: 20,
                got: slice.len(),
            });
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(slice);
        Ok(H160(bytes))
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl From<[u8; 20]> for H160 {
    fn from(bytes: [u8; 20]) -> Self {
        H160(bytes)
    }
}

// RLP implementations (behind feature flag)
#[cfg(feature = "rlp")]
mod rlp_impl {
    use super::*;
    use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

    impl Encodable for H256 {
        fn rlp_append(&self, s: &mut RlpStream) {
            s.encoder().encode_value(&self.0);
        }
    }

    impl Decodable for H256 {
        fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
            let bytes: Vec<u8> = rlp.as_val()?;
            if bytes.len() != 32 {
                return Err(DecoderError::RlpInvalidLength);
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Ok(H256(arr))
        }
    }

    impl Encodable for H160 {
        fn rlp_append(&self, s: &mut RlpStream) {
            s.encoder().encode_value(&self.0);
        }
    }

    impl Decodable for H160 {
        fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
            let bytes: Vec<u8> = rlp.as_val()?;
            if bytes.len() != 20 {
                return Err(DecoderError::RlpInvalidLength);
            }
            let mut arr = [0u8; 20];
            arr.copy_from_slice(&bytes);
            Ok(H160(arr))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== H256 Basic tests ====================

    #[test]
    fn test_h256_from_hex() {
        let hash = H256::from_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        assert!(!hash.is_zero());
        assert_eq!(hash.as_bytes()[31], 1);
    }

    #[test]
    fn test_h256_zero() {
        let zero = H256::ZERO;
        assert!(zero.is_zero());
    }

    #[test]
    fn test_h256_from_hex_without_prefix() {
        let hash = H256::from_hex(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        assert!(!hash.is_zero());
    }

    #[test]
    fn test_h256_from_hex_mixed_case() {
        let lower = H256::from_hex(
            "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ).unwrap();
        let upper = H256::from_hex(
            "0xABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789",
        ).unwrap();
        assert_eq!(lower, upper);
    }

    // ==================== H256 Hex parsing edge cases ====================

    #[test]
    fn test_h256_from_hex_invalid_chars() {
        let result = H256::from_hex(
            "0xgggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
        );
        assert!(result.is_err());
        match result {
            Err(HashError::InvalidHex(_)) => {}
            _ => panic!("Expected InvalidHex error"),
        }
    }

    #[test]
    fn test_h256_from_hex_empty() {
        let result = H256::from_hex("");
        assert!(result.is_err());
        match result {
            Err(HashError::InvalidLength { expected: 32, got: 0 }) => {}
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn test_h256_from_hex_only_prefix() {
        let result = H256::from_hex("0x");
        assert!(result.is_err());
        match result {
            Err(HashError::InvalidLength { expected: 32, got: 0 }) => {}
            _ => panic!("Expected InvalidLength error"),
        }
    }

    // ==================== H256 Length boundary tests ====================

    #[test]
    fn test_h256_from_hex_too_short() {
        // 31 bytes
        let result = H256::from_hex(
            "0x00000000000000000000000000000000000000000000000000000000000001",
        );
        assert!(result.is_err());
        match result {
            Err(HashError::InvalidLength { expected: 32, got: 31 }) => {}
            _ => panic!("Expected InvalidLength {{ expected: 32, got: 31 }}"),
        }
    }

    #[test]
    fn test_h256_from_hex_too_long() {
        // 33 bytes
        let result = H256::from_hex(
            "0x000000000000000000000000000000000000000000000000000000000000000100",
        );
        assert!(result.is_err());
        match result {
            Err(HashError::InvalidLength { expected: 32, got: 33 }) => {}
            _ => panic!("Expected InvalidLength {{ expected: 32, got: 33 }}"),
        }
    }

    #[test]
    fn test_h256_from_slice_too_short() {
        let short = vec![0u8; 31];
        let result = H256::from_slice(&short);
        assert!(result.is_err());
        match result {
            Err(HashError::InvalidLength { expected: 32, got: 31 }) => {}
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn test_h256_from_slice_too_long() {
        let long = vec![0u8; 33];
        let result = H256::from_slice(&long);
        assert!(result.is_err());
        match result {
            Err(HashError::InvalidLength { expected: 32, got: 33 }) => {}
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn test_h256_from_slice_empty() {
        let result = H256::from_slice(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_h256_from_slice_exact() {
        let bytes = [0xab; 32];
        let hash = H256::from_slice(&bytes).unwrap();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    // ==================== H256 Conversion tests ====================

    #[test]
    fn test_h256_from_bytes() {
        let bytes = [0x12; 32];
        let hash = H256::from_bytes(bytes);
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_h256_from_array() {
        let bytes: [u8; 32] = [0x34; 32];
        let hash: H256 = bytes.into();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_h256_as_ref() {
        let hash = H256::from_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();
        let slice: &[u8] = hash.as_ref();
        assert_eq!(slice.len(), 32);
    }

    // ==================== H256 Roundtrip tests ====================

    #[test]
    fn test_h256_hex_roundtrip() {
        let original = "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        let hash = H256::from_hex(original).unwrap();
        assert_eq!(hash.to_hex(), original);
    }

    #[test]
    fn test_h256_bytes_roundtrip() {
        let bytes = [
            0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
            0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
            0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
            0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
        ];
        let hash = H256::from_bytes(bytes);
        assert_eq!(hash.as_bytes(), &bytes);
    }

    // ==================== H256 Display and Debug ====================

    #[test]
    fn test_h256_display() {
        let hash = H256::from_hex(
            "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ).unwrap();
        assert_eq!(
            format!("{}", hash),
            "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
        );
    }

    #[test]
    fn test_h256_debug() {
        let hash = H256::from_hex(
            "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ).unwrap();
        let debug = format!("{:?}", hash);
        assert!(debug.starts_with("H256(0x"));
    }

    // ==================== H256 Equality and hash tests ====================

    #[test]
    fn test_h256_equality() {
        let h1 = H256::from_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();
        let h2 = H256::from_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();
        let h3 = H256::from_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000002",
        ).unwrap();
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_h256_hash_consistency() {
        use std::collections::HashSet;

        let h1 = H256::from_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();
        let h2 = H256::from_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ).unwrap();

        let mut set = HashSet::new();
        set.insert(h1);
        assert!(set.contains(&h2));
    }

    #[test]
    fn test_h256_default() {
        let default_hash = H256::default();
        assert!(default_hash.is_zero());
        assert_eq!(default_hash, H256::ZERO);
    }

    #[test]
    fn test_h256_len_constant() {
        assert_eq!(H256::LEN, 32);
    }

    // ==================== H256 Clone and Copy tests ====================

    #[test]
    fn test_h256_clone() {
        let hash = H256::from_hex(
            "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ).unwrap();
        let cloned = hash.clone();
        assert_eq!(hash, cloned);
    }

    #[test]
    fn test_h256_copy() {
        let hash = H256::from_hex(
            "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ).unwrap();
        let copied = hash;
        assert_eq!(hash, copied);
    }

    // ==================== H160 tests ====================

    #[test]
    fn test_h160_from_bytes() {
        let bytes = [0x12; 20];
        let hash = H160::from_bytes(bytes);
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_h160_from_slice() {
        let bytes = [0xab; 20];
        let hash = H160::from_slice(&bytes).unwrap();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_h160_from_slice_invalid_length() {
        let short = vec![0u8; 19];
        let result = H160::from_slice(&short);
        assert!(result.is_err());
        match result {
            Err(HashError::InvalidLength { expected: 20, got: 19 }) => {}
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn test_h160_zero() {
        let zero = H160::ZERO;
        assert_eq!(zero.as_bytes(), &[0u8; 20]);
    }

    #[test]
    fn test_h160_from_array() {
        let bytes: [u8; 20] = [0x34; 20];
        let hash: H160 = bytes.into();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_h160_len_constant() {
        assert_eq!(H160::LEN, 20);
    }

    #[test]
    fn test_h160_equality() {
        let h1 = H160::from_bytes([0x01; 20]);
        let h2 = H160::from_bytes([0x01; 20]);
        let h3 = H160::from_bytes([0x02; 20]);
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_h160_default() {
        let default_hash = H160::default();
        assert_eq!(default_hash, H160::ZERO);
    }

    // ==================== Ethereum specific hash values ====================

    #[test]
    fn test_ethereum_empty_hash() {
        // keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        let empty_hash = H256::from_hex(
            "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
        ).unwrap();
        assert!(!empty_hash.is_zero());
    }

    #[test]
    fn test_ethereum_null_rlp_hash() {
        // keccak256(RLP(null)) - empty trie root
        let null_hash = H256::from_hex(
            "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
        ).unwrap();
        assert!(!null_hash.is_zero());
    }
}
