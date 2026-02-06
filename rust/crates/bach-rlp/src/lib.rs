//! # bach-rlp
//!
//! RLP (Recursive Length Prefix) encoding/decoding for BachLedger.
//!
//! Provides Ethereum-compatible serialization using the `rlp` crate,
//! with custom implementations for bach-primitives types.
//!
//! ## RLP Encoding Rules
//!
//! - Single byte `[0x00, 0x7f]`: itself
//! - Short string (0-55 bytes): `0x80 + len` + data
//! - Long string (>55 bytes): `0xb7 + len_of_len` + len + data
//! - Short list (0-55 bytes payload): `0xc0 + len` + items
//! - Long list (>55 bytes payload): `0xf7 + len_of_len` + len + items

#![warn(missing_docs)]
#![warn(clippy::all)]

use bytes::{BufMut, BytesMut};

// Re-export rlp crate for direct use
pub use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

// Re-export primitives with RLP support
pub use bach_primitives::{Address, H160, H256};

/// Encode a value to RLP bytes
pub fn encode<T: Encodable>(value: &T) -> Vec<u8> {
    rlp::encode(value).to_vec()
}

/// Decode RLP bytes to a value
pub fn decode<T: Decodable>(data: &[u8]) -> Result<T, DecoderError> {
    rlp::decode(data)
}

/// RLP encoding utilities
pub mod utils {
    use super::*;

    /// Encode a u64 with minimal bytes (no leading zeros)
    pub fn encode_u64(value: u64) -> Vec<u8> {
        if value == 0 {
            return vec![0x80]; // Empty string for 0
        }
        if value < 128 {
            return vec![value as u8];
        }

        let mut buf = BytesMut::with_capacity(9);
        let bytes = value.to_be_bytes();
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(8);
        let len = 8 - start;
        buf.put_u8(0x80 + len as u8);
        buf.put_slice(&bytes[start..]);
        buf.to_vec()
    }

    /// Encode a u128 with minimal bytes
    pub fn encode_u128(value: u128) -> Vec<u8> {
        if value == 0 {
            return vec![0x80];
        }
        if value < 128 {
            return vec![value as u8];
        }

        let mut buf = BytesMut::with_capacity(17);
        let bytes = value.to_be_bytes();
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(16);
        let len = 16 - start;
        buf.put_u8(0x80 + len as u8);
        buf.put_slice(&bytes[start..]);
        buf.to_vec()
    }

    /// Decode RLP-encoded integer to u64
    pub fn decode_u64(data: &[u8]) -> Result<(u64, usize), DecoderError> {
        if data.is_empty() {
            return Err(DecoderError::RlpIsTooShort);
        }

        let first = data[0];
        if first < 0x80 {
            return Ok((first as u64, 1));
        }
        if first == 0x80 {
            return Ok((0, 1));
        }
        if first <= 0xb7 {
            let len = (first - 0x80) as usize;
            if data.len() < 1 + len {
                return Err(DecoderError::RlpIsTooShort);
            }
            if len > 8 {
                return Err(DecoderError::RlpIsTooBig);
            }
            let mut value: u64 = 0;
            for &byte in &data[1..1 + len] {
                value = (value << 8) | byte as u64;
            }
            return Ok((value, 1 + len));
        }

        Err(DecoderError::RlpExpectedToBeData)
    }

    /// Compute list payload length header
    pub fn list_header(payload_len: usize) -> Vec<u8> {
        if payload_len < 56 {
            vec![0xc0 + payload_len as u8]
        } else {
            let len_bytes = encode_length(payload_len);
            let mut header = vec![0xf7 + len_bytes.len() as u8];
            header.extend(len_bytes);
            header
        }
    }

    /// Encode length as minimal big-endian bytes
    fn encode_length(len: usize) -> Vec<u8> {
        let bytes = (len as u64).to_be_bytes();
        let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
        bytes[start..].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_h256() {
        let hash = H256::from_bytes([0x42; 32]);
        let encoded = encode(&hash);
        let decoded: H256 = decode(&encoded).unwrap();
        assert_eq!(hash, decoded);
    }

    #[test]
    fn test_encode_decode_h160() {
        let hash = H160::from_bytes([0x42; 20]);
        let encoded = encode(&hash);
        let decoded: H160 = decode(&encoded).unwrap();
        assert_eq!(hash, decoded);
    }

    #[test]
    fn test_encode_decode_address() {
        let addr = Address::from_bytes([0x42; 20]);
        let encoded = encode(&addr);
        let decoded: Address = decode(&encoded).unwrap();
        assert_eq!(addr, decoded);
    }

    #[test]
    fn test_encode_u64() {
        // 0 -> 0x80 (empty string)
        assert_eq!(utils::encode_u64(0), vec![0x80]);

        // Single byte < 128
        assert_eq!(utils::encode_u64(127), vec![127]);

        // 128 -> 0x81, 0x80
        assert_eq!(utils::encode_u64(128), vec![0x81, 0x80]);

        // 1024 -> 0x82, 0x04, 0x00
        assert_eq!(utils::encode_u64(1024), vec![0x82, 0x04, 0x00]);
    }

    #[test]
    fn test_decode_u64() {
        assert_eq!(utils::decode_u64(&[0x80]).unwrap(), (0, 1));
        assert_eq!(utils::decode_u64(&[127]).unwrap(), (127, 1));
        assert_eq!(utils::decode_u64(&[0x81, 0x80]).unwrap(), (128, 2));
        assert_eq!(utils::decode_u64(&[0x82, 0x04, 0x00]).unwrap(), (1024, 3));
    }

    #[test]
    fn test_list_header() {
        // Short list (< 56 bytes payload)
        assert_eq!(utils::list_header(0), vec![0xc0]);
        assert_eq!(utils::list_header(55), vec![0xc0 + 55]);

        // Long list (>= 56 bytes payload)
        let header = utils::list_header(56);
        assert_eq!(header[0], 0xf8); // 0xf7 + 1
        assert_eq!(header[1], 56);
    }

    #[test]
    fn test_rlp_stream() {
        let mut stream = RlpStream::new_list(2);
        stream.append(&1u64);
        stream.append(&"hello");
        let encoded = stream.out();

        let rlp = Rlp::new(&encoded);
        assert_eq!(rlp.item_count().unwrap(), 2);
        assert_eq!(rlp.val_at::<u64>(0).unwrap(), 1);
        assert_eq!(rlp.val_at::<String>(1).unwrap(), "hello");
    }

    #[test]
    fn test_ethereum_rlp_examples() {
        // From Ethereum Yellow Paper
        // "dog" = [0x83, 'd', 'o', 'g']
        let encoded = rlp::encode(&"dog");
        assert_eq!(&encoded[..], &[0x83, b'd', b'o', b'g']);

        // ["cat", "dog"] = [0xc8, 0x83, 'c', 'a', 't', 0x83, 'd', 'o', 'g']
        let mut stream = RlpStream::new_list(2);
        stream.append(&"cat");
        stream.append(&"dog");
        let encoded = stream.out();
        assert_eq!(
            &encoded[..],
            &[0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']
        );

        // Empty string = [0x80]
        let encoded = rlp::encode(&"");
        assert_eq!(&encoded[..], &[0x80]);

        // Empty list = [0xc0]
        let stream = RlpStream::new_list(0);
        let encoded = stream.out();
        assert_eq!(&encoded[..], &[0xc0]);

        // Integer 0 = [0x80] (same as empty string)
        let encoded = rlp::encode(&0u64);
        assert_eq!(&encoded[..], &[0x80]);

        // Integer 15 = [0x0f]
        let encoded = rlp::encode(&15u64);
        assert_eq!(&encoded[..], &[0x0f]);

        // Integer 1024 = [0x82, 0x04, 0x00]
        let encoded = rlp::encode(&1024u64);
        assert_eq!(&encoded[..], &[0x82, 0x04, 0x00]);
    }

    #[test]
    fn test_h256_rlp_roundtrip() {
        // Test with zero hash
        let zero = H256::ZERO;
        let encoded = encode(&zero);
        let decoded: H256 = decode(&encoded).unwrap();
        assert_eq!(zero, decoded);

        // Test with non-zero hash
        let hash = H256::from_bytes([
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ]);
        let encoded = encode(&hash);
        let decoded: H256 = decode(&encoded).unwrap();
        assert_eq!(hash, decoded);
    }

    #[test]
    fn test_address_in_rlp_list() {
        let addr1 = Address::from_bytes([0x11; 20]);
        let addr2 = Address::from_bytes([0x22; 20]);

        let mut stream = RlpStream::new_list(2);
        stream.append(&addr1);
        stream.append(&addr2);
        let encoded = stream.out();

        let rlp = Rlp::new(&encoded);
        assert_eq!(rlp.item_count().unwrap(), 2);
        assert_eq!(rlp.val_at::<Address>(0).unwrap(), addr1);
        assert_eq!(rlp.val_at::<Address>(1).unwrap(), addr2);
    }
}
