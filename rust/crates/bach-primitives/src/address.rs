//! Ethereum-compatible address type (20 bytes)

use std::fmt;
use thiserror::Error;

/// Address parsing error
#[derive(Debug, Error)]
pub enum AddressError {
    /// Invalid hex string
    #[error("invalid hex string: {0}")]
    InvalidHex(String),
    /// Invalid length
    #[error("invalid address length: expected 20 bytes, got {0}")]
    InvalidLength(usize),
}

/// Ethereum-compatible 20-byte address
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Address([u8; 20]);

impl Address {
    /// Size of address in bytes
    pub const LEN: usize = 20;

    /// Zero address (0x0000...0000)
    pub const ZERO: Address = Address([0u8; 20]);

    /// Create address from bytes
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Address(bytes)
    }

    /// Create address from slice
    pub fn from_slice(slice: &[u8]) -> Result<Self, AddressError> {
        if slice.len() != 20 {
            return Err(AddressError::InvalidLength(slice.len()));
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(slice);
        Ok(Address(bytes))
    }

    /// Parse address from hex string (with or without 0x prefix)
    pub fn from_hex(s: &str) -> Result<Self, AddressError> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(s).map_err(|e| AddressError::InvalidHex(e.to_string()))?;
        Self::from_slice(&bytes)
    }

    /// Get as byte slice
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Get as mutable byte slice
    pub fn as_bytes_mut(&mut self) -> &mut [u8; 20] {
        &mut self.0
    }

    /// Check if this is the zero address
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 20]
    }

    /// Convert to hex string with 0x prefix
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address({})", self.to_hex())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 20]> for Address {
    fn from(bytes: [u8; 20]) -> Self {
        Address(bytes)
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// RLP implementation (behind feature flag)
#[cfg(feature = "rlp")]
mod rlp_impl {
    use super::*;
    use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

    impl Encodable for Address {
        fn rlp_append(&self, s: &mut RlpStream) {
            s.encoder().encode_value(&self.0);
        }
    }

    impl Decodable for Address {
        fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
            let bytes: Vec<u8> = rlp.as_val()?;
            if bytes.len() != 20 {
                return Err(DecoderError::RlpInvalidLength);
            }
            let mut arr = [0u8; 20];
            arr.copy_from_slice(&bytes);
            Ok(Address(arr))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic functionality tests ====================

    #[test]
    fn test_address_from_hex() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        assert!(!addr.is_zero());

        let addr2 = Address::from_hex("742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        assert_eq!(addr, addr2);
    }

    #[test]
    fn test_zero_address() {
        let zero = Address::ZERO;
        assert!(zero.is_zero());
        assert_eq!(zero.to_hex(), "0x0000000000000000000000000000000000000000");
    }

    #[test]
    fn test_address_display() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        assert_eq!(
            format!("{}", addr),
            "0x742d35cc6634c0532925a3b844bc9e7595f0ab3d"
        );
    }

    // ==================== Hex parsing edge cases ====================

    #[test]
    fn test_address_from_hex_lowercase() {
        let addr = Address::from_hex("0x742d35cc6634c0532925a3b844bc9e7595f0ab3d").unwrap();
        assert_eq!(
            addr.to_hex(),
            "0x742d35cc6634c0532925a3b844bc9e7595f0ab3d"
        );
    }

    #[test]
    fn test_address_from_hex_uppercase() {
        let addr = Address::from_hex("0x742D35CC6634C0532925A3B844BC9E7595F0AB3D").unwrap();
        assert_eq!(
            addr.to_hex(),
            "0x742d35cc6634c0532925a3b844bc9e7595f0ab3d"
        );
    }

    #[test]
    fn test_address_from_hex_mixed_case() {
        let lower = Address::from_hex("0x742d35cc6634c0532925a3b844bc9e7595f0ab3d").unwrap();
        let upper = Address::from_hex("0x742D35CC6634C0532925A3B844BC9E7595F0AB3D").unwrap();
        let mixed = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        assert_eq!(lower, upper);
        assert_eq!(lower, mixed);
    }

    #[test]
    fn test_address_from_hex_invalid_chars() {
        let result = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aGGG");
        assert!(result.is_err());
        match result {
            Err(AddressError::InvalidHex(_)) => {}
            _ => panic!("Expected InvalidHex error"),
        }
    }

    #[test]
    fn test_address_from_hex_non_hex_chars() {
        assert!(Address::from_hex("0xghijklmnopqrstuvwxyz0123456789ab").is_err());
        assert!(Address::from_hex("0x!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!").is_err());
    }

    #[test]
    fn test_address_from_hex_empty() {
        let result = Address::from_hex("");
        assert!(result.is_err());
        match result {
            Err(AddressError::InvalidLength(0)) => {}
            _ => panic!("Expected InvalidLength(0) error"),
        }
    }

    #[test]
    fn test_address_from_hex_only_prefix() {
        let result = Address::from_hex("0x");
        assert!(result.is_err());
        match result {
            Err(AddressError::InvalidLength(0)) => {}
            _ => panic!("Expected InvalidLength(0) error"),
        }
    }

    // ==================== Length boundary tests ====================

    #[test]
    fn test_address_from_hex_too_short() {
        // 19 bytes (38 hex chars)
        let result = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB");
        assert!(result.is_err());
        match result {
            Err(AddressError::InvalidLength(19)) => {}
            _ => panic!("Expected InvalidLength(19) error"),
        }
    }

    #[test]
    fn test_address_from_hex_too_long() {
        // 21 bytes (42 hex chars)
        let result = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d00");
        assert!(result.is_err());
        match result {
            Err(AddressError::InvalidLength(21)) => {}
            _ => panic!("Expected InvalidLength(21) error"),
        }
    }

    #[test]
    fn test_address_from_hex_odd_length() {
        // Odd number of hex chars
        let result = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3");
        assert!(result.is_err());
    }

    #[test]
    fn test_address_from_slice_too_short() {
        let short = vec![0u8; 19];
        let result = Address::from_slice(&short);
        assert!(result.is_err());
        match result {
            Err(AddressError::InvalidLength(19)) => {}
            _ => panic!("Expected InvalidLength(19) error"),
        }
    }

    #[test]
    fn test_address_from_slice_too_long() {
        let long = vec![0u8; 21];
        let result = Address::from_slice(&long);
        assert!(result.is_err());
        match result {
            Err(AddressError::InvalidLength(21)) => {}
            _ => panic!("Expected InvalidLength(21) error"),
        }
    }

    #[test]
    fn test_address_from_slice_empty() {
        let result = Address::from_slice(&[]);
        assert!(result.is_err());
        match result {
            Err(AddressError::InvalidLength(0)) => {}
            _ => panic!("Expected InvalidLength(0) error"),
        }
    }

    #[test]
    fn test_address_from_slice_exact() {
        let bytes = [0xab; 20];
        let addr = Address::from_slice(&bytes).unwrap();
        assert_eq!(addr.as_bytes(), &bytes);
    }

    // ==================== Conversion tests ====================

    #[test]
    fn test_address_from_bytes() {
        let bytes = [0x12; 20];
        let addr = Address::from_bytes(bytes);
        assert_eq!(addr.as_bytes(), &bytes);
    }

    #[test]
    fn test_address_from_array() {
        let bytes: [u8; 20] = [0x34; 20];
        let addr: Address = bytes.into();
        assert_eq!(addr.as_bytes(), &bytes);
    }

    #[test]
    fn test_address_as_ref() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let slice: &[u8] = addr.as_ref();
        assert_eq!(slice.len(), 20);
    }

    #[test]
    fn test_address_as_bytes_mut() {
        let mut addr = Address::ZERO;
        let bytes = addr.as_bytes_mut();
        bytes[0] = 0xff;
        assert_eq!(addr.as_bytes()[0], 0xff);
        assert!(!addr.is_zero());
    }

    // ==================== Roundtrip tests ====================

    #[test]
    fn test_address_hex_roundtrip() {
        let original = "0x742d35cc6634c0532925a3b844bc9e7595f0ab3d";
        let addr = Address::from_hex(original).unwrap();
        assert_eq!(addr.to_hex(), original);
    }

    #[test]
    fn test_address_bytes_roundtrip() {
        let bytes = [
            0x74, 0x2d, 0x35, 0xcc, 0x66, 0x34, 0xc0, 0x53, 0x29, 0x25,
            0xa3, 0xb8, 0x44, 0xbc, 0x9e, 0x75, 0x95, 0xf0, 0xab, 0x3d,
        ];
        let addr = Address::from_bytes(bytes);
        assert_eq!(addr.as_bytes(), &bytes);
    }

    // ==================== Equality and hash tests ====================

    #[test]
    fn test_address_equality() {
        let addr1 = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let addr2 = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let addr3 = Address::from_hex("0x0000000000000000000000000000000000000001").unwrap();

        assert_eq!(addr1, addr2);
        assert_ne!(addr1, addr3);
    }

    #[test]
    fn test_address_hash_consistency() {
        use std::collections::HashSet;

        let addr1 = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let addr2 = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

        let mut set = HashSet::new();
        set.insert(addr1);
        assert!(set.contains(&addr2));
    }

    #[test]
    fn test_address_default() {
        let default_addr = Address::default();
        assert!(default_addr.is_zero());
        assert_eq!(default_addr, Address::ZERO);
    }

    // ==================== Debug format test ====================

    #[test]
    fn test_address_debug() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let debug = format!("{:?}", addr);
        assert!(debug.contains("Address(0x742d35cc6634c0532925a3b844bc9e7595f0ab3d)"));
    }

    // ==================== Ethereum compatibility tests ====================

    #[test]
    fn test_ethereum_known_addresses() {
        // Zero address
        let zero = Address::from_hex("0x0000000000000000000000000000000000000000").unwrap();
        assert!(zero.is_zero());

        // Ethereum burn address
        let dead = Address::from_hex("0x000000000000000000000000000000000000dEaD").unwrap();
        assert!(!dead.is_zero());
        assert_eq!(dead.as_bytes()[19], 0xad);

        // WETH contract address (mainnet)
        let weth = Address::from_hex("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
        assert!(!weth.is_zero());
    }

    #[test]
    fn test_address_len_constant() {
        assert_eq!(Address::LEN, 20);
    }

    // ==================== Clone and Copy tests ====================

    #[test]
    fn test_address_clone() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let cloned = addr.clone();
        assert_eq!(addr, cloned);
    }

    #[test]
    fn test_address_copy() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let copied = addr; // Copy
        assert_eq!(addr, copied); // Original still valid because Address is Copy
    }
}
