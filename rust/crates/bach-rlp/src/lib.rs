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

    // ==================== Basic encode/decode tests ====================

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

    // ==================== Single byte encoding ====================

    #[test]
    fn test_single_byte_encoding() {
        // Single byte [0x00, 0x7f] encodes as itself
        for i in 0u8..=0x7f {
            let encoded = rlp::encode(&vec![i]);
            // For single byte 0x00-0x7f, if it's the only content, it should be the byte itself
            if i == 0 {
                // Special case: single zero byte
                assert_eq!(&encoded[..], &[0x00]);
            } else {
                assert_eq!(encoded[0], i);
            }
        }
    }

    #[test]
    fn test_single_byte_values() {
        // 0x00 encodes as 0x00
        let encoded = rlp::encode(&vec![0x00u8]);
        assert_eq!(&encoded[..], &[0x00]);

        // 0x7f encodes as 0x7f
        let encoded = rlp::encode(&vec![0x7fu8]);
        assert_eq!(&encoded[..], &[0x7f]);

        // 0x80 needs length prefix
        let encoded = rlp::encode(&vec![0x80u8]);
        assert_eq!(&encoded[..], &[0x81, 0x80]);
    }

    // ==================== Short string encoding (0-55 bytes) ====================

    #[test]
    fn test_short_string_encoding() {
        // Empty string = 0x80
        let encoded = rlp::encode(&"");
        assert_eq!(&encoded[..], &[0x80]);

        // "a" (1 byte, < 0x80) = 0x61
        let encoded = rlp::encode(&"a");
        assert_eq!(&encoded[..], &[0x61]);

        // "abc" (3 bytes) = 0x83, 'a', 'b', 'c'
        let encoded = rlp::encode(&"abc");
        assert_eq!(&encoded[..], &[0x83, b'a', b'b', b'c']);
    }

    #[test]
    fn test_short_string_55_bytes() {
        // Maximum short string: 55 bytes
        let data: Vec<u8> = vec![0x42; 55];
        let encoded = rlp::encode(&data);
        assert_eq!(encoded[0], 0x80 + 55); // 0xb7
        assert_eq!(encoded.len(), 56); // 1 + 55
    }

    // ==================== Long string encoding (>55 bytes) ====================

    #[test]
    fn test_long_string_encoding() {
        // 56 bytes: starts long string format
        let data: Vec<u8> = vec![0x42; 56];
        let encoded = rlp::encode(&data);
        assert_eq!(encoded[0], 0xb8); // 0xb7 + 1 (1 byte for length)
        assert_eq!(encoded[1], 56);
        assert_eq!(encoded.len(), 58); // 1 + 1 + 56
    }

    #[test]
    fn test_long_string_256_bytes() {
        // 256 bytes: needs 2 bytes for length (but still 1 byte)
        let data: Vec<u8> = vec![0x42; 256];
        let encoded = rlp::encode(&data);
        assert_eq!(encoded[0], 0xb9); // 0xb7 + 2 (2 bytes for length)
        assert_eq!(encoded[1], 0x01);
        assert_eq!(encoded[2], 0x00); // 256 in big-endian
        assert_eq!(encoded.len(), 259); // 1 + 2 + 256
    }

    #[test]
    fn test_long_string_1024_bytes() {
        let data: Vec<u8> = vec![0x42; 1024];
        let encoded = rlp::encode(&data);
        assert_eq!(encoded[0], 0xb9); // 0xb7 + 2
        assert_eq!(encoded[1], 0x04);
        assert_eq!(encoded[2], 0x00); // 1024 = 0x0400
        assert_eq!(encoded.len(), 1027); // 1 + 2 + 1024
    }

    // ==================== Short list encoding (0-55 bytes payload) ====================

    #[test]
    fn test_empty_list() {
        let stream = RlpStream::new_list(0);
        let encoded = stream.out();
        assert_eq!(&encoded[..], &[0xc0]);
    }

    #[test]
    fn test_short_list() {
        let mut stream = RlpStream::new_list(2);
        stream.append(&1u64);
        stream.append(&2u64);
        let encoded = stream.out();
        // [1, 2] = 0xc2, 0x01, 0x02
        assert_eq!(&encoded[..], &[0xc2, 0x01, 0x02]);
    }

    #[test]
    fn test_short_list_max_payload() {
        // List with exactly 55 bytes payload
        let mut stream = RlpStream::new_list(55);
        for _ in 0..55 {
            stream.append(&1u8);
        }
        let encoded = stream.out();
        assert_eq!(encoded[0], 0xc0 + 55); // 0xf7
    }

    // ==================== Long list encoding (>55 bytes payload) ====================

    #[test]
    fn test_long_list() {
        // List with 56 bytes payload
        let mut stream = RlpStream::new_list(56);
        for _ in 0..56 {
            stream.append(&1u8);
        }
        let encoded = stream.out();
        assert_eq!(encoded[0], 0xf8); // 0xf7 + 1
        assert_eq!(encoded[1], 56);
    }

    // ==================== Nested list encoding ====================

    #[test]
    fn test_nested_list() {
        // [[]] = 0xc1, 0xc0
        let mut outer = RlpStream::new_list(1);
        outer.append_empty_data();
        // Actually need to encode a nested empty list
        let inner = RlpStream::new_list(0);
        let mut outer = RlpStream::new_list(1);
        outer.append_raw(&inner.out(), 1);
        let encoded = outer.out();
        assert_eq!(&encoded[..], &[0xc1, 0xc0]);
    }

    #[test]
    fn test_deeply_nested_list() {
        // [[["a"]]]
        let mut l1 = RlpStream::new_list(1);
        l1.append(&"a");

        let mut l2 = RlpStream::new_list(1);
        l2.append_raw(&l1.out(), 1);

        let mut l3 = RlpStream::new_list(1);
        l3.append_raw(&l2.out(), 1);

        let encoded = l3.out();
        // [0xc3, 0xc2, 0xc1, 0x61]
        assert_eq!(&encoded[..], &[0xc3, 0xc2, 0xc1, 0x61]);
    }

    // ==================== Integer encoding ====================

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
    fn test_encode_u64_max() {
        let max = u64::MAX;
        let encoded = utils::encode_u64(max);
        assert_eq!(encoded[0], 0x88); // 8 bytes
        assert_eq!(encoded.len(), 9);
    }

    #[test]
    fn test_encode_u128() {
        assert_eq!(utils::encode_u128(0), vec![0x80]);
        assert_eq!(utils::encode_u128(127), vec![127]);
        assert_eq!(utils::encode_u128(128), vec![0x81, 0x80]);
    }

    #[test]
    fn test_encode_u128_large() {
        let large = 0x0102030405060708u128;
        let encoded = utils::encode_u128(large);
        assert_eq!(encoded[0], 0x88); // 8 bytes
        assert_eq!(&encoded[1..], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    }

    #[test]
    fn test_decode_u64() {
        assert_eq!(utils::decode_u64(&[0x80]).unwrap(), (0, 1));
        assert_eq!(utils::decode_u64(&[127]).unwrap(), (127, 1));
        assert_eq!(utils::decode_u64(&[0x81, 0x80]).unwrap(), (128, 2));
        assert_eq!(utils::decode_u64(&[0x82, 0x04, 0x00]).unwrap(), (1024, 3));
    }

    #[test]
    fn test_decode_u64_errors() {
        // Empty input
        assert!(utils::decode_u64(&[]).is_err());

        // Too short for claimed length
        assert!(utils::decode_u64(&[0x82, 0x04]).is_err()); // Claims 2 bytes, only 1

        // Too big for u64 (> 8 bytes)
        assert!(utils::decode_u64(&[0x89]).is_err()); // Claims 9 bytes
    }

    // ==================== List header ====================

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
    fn test_list_header_large() {
        // 256 bytes payload
        let header = utils::list_header(256);
        assert_eq!(header[0], 0xf9); // 0xf7 + 2
        assert_eq!(header[1], 0x01);
        assert_eq!(header[2], 0x00);

        // 65536 bytes payload
        let header = utils::list_header(65536);
        assert_eq!(header[0], 0xfa); // 0xf7 + 3
        assert_eq!(header[1], 0x01);
        assert_eq!(header[2], 0x00);
        assert_eq!(header[3], 0x00);
    }

    // ==================== RLP Stream ====================

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
    fn test_rlp_stream_append_raw() {
        let inner_data = vec![0x82, 0x01, 0x02]; // RLP of 2-byte string 0x0102
        let mut stream = RlpStream::new_list(1);
        stream.append_raw(&inner_data, 1);
        let encoded = stream.out();

        assert_eq!(encoded[0], 0xc3); // list of 3 bytes (the raw data length)
    }

    // ==================== Ethereum Yellow Paper Test Vectors ====================

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
    fn test_ethereum_yellow_paper_set_theoretical() {
        // From Appendix B of Yellow Paper
        // The set T is defined as:
        // T ≡ L ∪ B
        // where L is the set of all tree-like structures
        // and B is the set of all byte arrays

        // Test: encoding of the empty byte array
        let empty: Vec<u8> = vec![];
        let encoded = rlp::encode(&empty);
        assert_eq!(&encoded[..], &[0x80]);

        // Test: encoding of a single zero byte
        let single_zero: Vec<u8> = vec![0x00];
        let encoded = rlp::encode(&single_zero);
        assert_eq!(&encoded[..], &[0x00]);
    }

    // ==================== bach-primitives RLP roundtrip ====================

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
    fn test_h256_encoding_format() {
        // H256 is 32 bytes, so short string format: 0x80 + 32 = 0xa0
        let hash = H256::from_bytes([0x42; 32]);
        let encoded = encode(&hash);
        assert_eq!(encoded[0], 0xa0); // 0x80 + 32
        assert_eq!(encoded.len(), 33); // 1 + 32
    }

    #[test]
    fn test_h160_rlp_roundtrip() {
        let hash = H160::from_bytes([0x42; 20]);
        let encoded = encode(&hash);
        let decoded: H160 = decode(&encoded).unwrap();
        assert_eq!(hash, decoded);
    }

    #[test]
    fn test_h160_encoding_format() {
        // H160 is 20 bytes: 0x80 + 20 = 0x94
        let hash = H160::from_bytes([0x42; 20]);
        let encoded = encode(&hash);
        assert_eq!(encoded[0], 0x94); // 0x80 + 20
        assert_eq!(encoded.len(), 21); // 1 + 20
    }

    #[test]
    fn test_address_rlp_roundtrip() {
        let addr = Address::from_bytes([0x42; 20]);
        let encoded = encode(&addr);
        let decoded: Address = decode(&encoded).unwrap();
        assert_eq!(addr, decoded);
    }

    #[test]
    fn test_address_encoding_format() {
        // Address is 20 bytes: 0x80 + 20 = 0x94
        let addr = Address::from_bytes([0x42; 20]);
        let encoded = encode(&addr);
        assert_eq!(encoded[0], 0x94);
        assert_eq!(encoded.len(), 21);
    }

    #[test]
    fn test_address_zero_roundtrip() {
        let zero = Address::ZERO;
        let encoded = encode(&zero);
        let decoded: Address = decode(&encoded).unwrap();
        assert_eq!(zero, decoded);
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

    #[test]
    fn test_h256_in_rlp_list() {
        let hash1 = H256::from_bytes([0x11; 32]);
        let hash2 = H256::from_bytes([0x22; 32]);

        let mut stream = RlpStream::new_list(2);
        stream.append(&hash1);
        stream.append(&hash2);
        let encoded = stream.out();

        let rlp = Rlp::new(&encoded);
        assert_eq!(rlp.item_count().unwrap(), 2);
        assert_eq!(rlp.val_at::<H256>(0).unwrap(), hash1);
        assert_eq!(rlp.val_at::<H256>(1).unwrap(), hash2);
    }

    // ==================== Boundary cases ====================

    #[test]
    fn test_decode_invalid_rlp() {
        // Invalid: claims to be a string of 55 bytes but only 10 bytes follow
        let invalid = vec![0xb7, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a];
        let result: Result<Vec<u8>, _> = decode(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_truncated_data() {
        // Truncated H256: claims 32 bytes but only has 10
        let truncated = vec![0xa0, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a];
        let result: Result<H256, _> = decode(&truncated);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_empty_input() {
        let empty: &[u8] = &[];
        let result: Result<u64, _> = decode(empty);
        assert!(result.is_err());
    }

    #[test]
    fn test_leading_zeros_in_length() {
        // RLP spec: length encoding should not have leading zeros
        // This tests that we correctly encode without leading zeros
        let value = 256u64;
        let encoded = utils::encode_u64(value);
        // 256 = 0x0100, needs 2 bytes
        assert_eq!(encoded, vec![0x82, 0x01, 0x00]);
    }

    // ==================== Mixed type lists ====================

    #[test]
    fn test_mixed_type_list() {
        let addr = Address::from_bytes([0x42; 20]);
        let hash = H256::from_bytes([0x01; 32]);
        let value = 1000u64;

        let mut stream = RlpStream::new_list(3);
        stream.append(&addr);
        stream.append(&hash);
        stream.append(&value);
        let encoded = stream.out();

        let rlp = Rlp::new(&encoded);
        assert_eq!(rlp.item_count().unwrap(), 3);
        assert_eq!(rlp.val_at::<Address>(0).unwrap(), addr);
        assert_eq!(rlp.val_at::<H256>(1).unwrap(), hash);
        assert_eq!(rlp.val_at::<u64>(2).unwrap(), value);
    }

    // ==================== Ethereum-specific values ====================

    #[test]
    fn test_ethereum_empty_trie_hash() {
        // keccak256(RLP([])) - empty trie root
        let empty_list = RlpStream::new_list(0);
        let encoded = empty_list.out();
        assert_eq!(&encoded[..], &[0xc0]);

        // The empty trie hash is keccak256([0xc0]) but we just test the encoding
    }

    #[test]
    fn test_ethereum_block_number_encoding() {
        // Block numbers are encoded as minimal big-endian integers
        let block_0 = 0u64;
        let block_1 = 1u64;
        let block_100 = 100u64;
        let block_large = 15_000_000u64;

        assert_eq!(rlp::encode(&block_0).to_vec(), vec![0x80]);
        assert_eq!(rlp::encode(&block_1).to_vec(), vec![0x01]);
        assert_eq!(rlp::encode(&block_100).to_vec(), vec![0x64]);

        let encoded_large = rlp::encode(&block_large);
        assert_eq!(encoded_large[0], 0x83); // 3 bytes
    }

    #[test]
    fn test_ethereum_gas_limit_encoding() {
        // Typical gas limits
        let gas_21000 = 21000u64;
        let gas_30m = 30_000_000u64; // 0x1C9C380 = 4 bytes

        let encoded = rlp::encode(&gas_21000);
        assert_eq!(encoded[0], 0x82); // 2 bytes
        assert_eq!(&encoded[1..], &[0x52, 0x08]); // 21000 = 0x5208

        let encoded = rlp::encode(&gas_30m);
        assert_eq!(encoded[0], 0x84); // 4 bytes (30M = 0x01C9C380)
    }
}
