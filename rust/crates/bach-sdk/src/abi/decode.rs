//! ABI decoding

use bach_primitives::{Address, U256};

use super::types::{I256, ParamType, Token};
use crate::SdkError;

/// Decode tokens from ABI-encoded data
pub fn decode(types: &[ParamType], data: &[u8]) -> Result<Vec<Token>, SdkError> {
    let mut offset = 0;
    let mut tokens = Vec::with_capacity(types.len());

    for param_type in types {
        let (token, consumed) = decode_token(param_type, data, offset)?;
        tokens.push(token);
        offset += consumed;
    }

    Ok(tokens)
}

/// Decode a single token
fn decode_token(
    param_type: &ParamType,
    data: &[u8],
    offset: usize,
) -> Result<(Token, usize), SdkError> {
    match param_type {
        ParamType::Address => {
            check_length(data, offset + 32)?;
            let mut addr_bytes = [0u8; 20];
            addr_bytes.copy_from_slice(&data[offset + 12..offset + 32]);
            Ok((Token::Address(Address::from_bytes(addr_bytes)), 32))
        }
        ParamType::Uint(_) => {
            check_length(data, offset + 32)?;
            let value = U256::from_big_endian(&data[offset..offset + 32]);
            Ok((Token::Uint(value), 32))
        }
        ParamType::Int(_) => {
            check_length(data, offset + 32)?;
            let bytes = &data[offset..offset + 32];

            // Check sign bit
            let negative = bytes[0] & 0x80 != 0;
            let abs = if negative {
                // Two's complement: flip bits and add 1
                let mut flipped = [0u8; 32];
                for i in 0..32 {
                    flipped[i] = !bytes[i];
                }
                let mut carry = 1u16;
                for i in (0..32).rev() {
                    let sum = (flipped[i] as u16) + carry;
                    flipped[i] = sum as u8;
                    carry = sum >> 8;
                }
                U256::from_big_endian(&flipped)
            } else {
                U256::from_big_endian(bytes)
            };

            Ok((Token::Int(I256::new(abs, negative)), 32))
        }
        ParamType::Bool => {
            check_length(data, offset + 32)?;
            let value = data[offset + 31] != 0;
            Ok((Token::Bool(value), 32))
        }
        ParamType::FixedBytes(size) => {
            check_length(data, offset + 32)?;
            let bytes = data[offset..offset + *size].to_vec();
            Ok((Token::FixedBytes(bytes), 32))
        }
        ParamType::Bytes => {
            check_length(data, offset + 32)?;
            let data_offset = U256::from_big_endian(&data[offset..offset + 32]).as_usize();
            let (bytes, _) = decode_bytes(data, data_offset)?;
            Ok((Token::Bytes(bytes), 32))
        }
        ParamType::String => {
            check_length(data, offset + 32)?;
            let data_offset = U256::from_big_endian(&data[offset..offset + 32]).as_usize();
            let (bytes, _) = decode_bytes(data, data_offset)?;
            let s = String::from_utf8(bytes)
                .map_err(|e| SdkError::AbiDecode(format!("Invalid UTF-8: {}", e)))?;
            Ok((Token::String(s), 32))
        }
        ParamType::Array(inner) => {
            check_length(data, offset + 32)?;
            let data_offset = U256::from_big_endian(&data[offset..offset + 32]).as_usize();
            check_length(data, data_offset + 32)?;
            let len = U256::from_big_endian(&data[data_offset..data_offset + 32]).as_usize();

            let mut tokens = Vec::with_capacity(len);
            let mut inner_offset = data_offset + 32;

            for _ in 0..len {
                let (token, consumed) = decode_token(inner, data, inner_offset)?;
                tokens.push(token);
                inner_offset += consumed;
            }

            Ok((Token::Array(tokens), 32))
        }
        ParamType::FixedArray(inner, size) => {
            let mut tokens = Vec::with_capacity(*size);
            let mut inner_offset = offset;

            for _ in 0..*size {
                let (token, consumed) = decode_token(inner, data, inner_offset)?;
                tokens.push(token);
                inner_offset += consumed;
            }

            Ok((Token::FixedArray(tokens), inner_offset - offset))
        }
        ParamType::Tuple(types) => {
            let mut tokens = Vec::with_capacity(types.len());
            let mut inner_offset = offset;

            for inner_type in types {
                let (token, consumed) = decode_token(inner_type, data, inner_offset)?;
                tokens.push(token);
                inner_offset += consumed;
            }

            Ok((Token::Tuple(tokens), inner_offset - offset))
        }
    }
}

/// Decode dynamic bytes from data at offset
fn decode_bytes(data: &[u8], offset: usize) -> Result<(Vec<u8>, usize), SdkError> {
    check_length(data, offset + 32)?;
    let len = U256::from_big_endian(&data[offset..offset + 32]).as_usize();
    check_length(data, offset + 32 + len)?;
    let bytes = data[offset + 32..offset + 32 + len].to_vec();

    // Calculate total consumed (including padding)
    let padded_len = len.div_ceil(32) * 32;
    Ok((bytes, 32 + padded_len))
}

/// Check that data has at least `required` bytes
fn check_length(data: &[u8], required: usize) -> Result<(), SdkError> {
    if data.len() < required {
        return Err(SdkError::AbiDecode(format!(
            "Insufficient data: need {} bytes, have {}",
            required,
            data.len()
        )));
    }
    Ok(())
}

/// Decode function return data
pub fn decode_output(types: &[ParamType], data: &[u8]) -> Result<Vec<Token>, SdkError> {
    decode(types, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_address() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let mut encoded = [0u8; 32];
        encoded[12..32].copy_from_slice(addr.as_bytes());

        let tokens = decode(&[ParamType::Address], &encoded).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Address(addr));
    }

    #[test]
    fn test_decode_uint() {
        let mut encoded = [0u8; 32];
        encoded[31] = 100;

        let tokens = decode(&[ParamType::Uint(256)], &encoded).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Uint(U256::from(100)));
    }

    #[test]
    fn test_decode_bool() {
        let mut encoded_true = [0u8; 32];
        encoded_true[31] = 1;
        let encoded_false = [0u8; 32];

        let tokens_true = decode(&[ParamType::Bool], &encoded_true).unwrap();
        let tokens_false = decode(&[ParamType::Bool], &encoded_false).unwrap();

        assert_eq!(tokens_true[0], Token::Bool(true));
        assert_eq!(tokens_false[0], Token::Bool(false));
    }

    #[test]
    fn test_decode_bytes32() {
        let data = [0x42u8; 32];
        let tokens = decode(&[ParamType::FixedBytes(32)], &data).unwrap();

        assert_eq!(tokens[0], Token::FixedBytes(data.to_vec()));
    }

    #[test]
    fn test_decode_dynamic_bytes() {
        let original = vec![0x01, 0x02, 0x03];

        // Build encoded data: offset (32) + length (32) + data (padded to 32)
        let mut encoded = vec![0u8; 96];
        // Offset = 32
        encoded[31] = 32;
        // Length = 3
        encoded[63] = 3;
        // Data
        encoded[64..67].copy_from_slice(&original);

        let tokens = decode(&[ParamType::Bytes], &encoded).unwrap();
        assert_eq!(tokens[0], Token::Bytes(original));
    }

    #[test]
    fn test_decode_string() {
        let s = "hello";

        // Build encoded data
        let mut encoded = vec![0u8; 96];
        // Offset = 32
        encoded[31] = 32;
        // Length = 5
        encoded[63] = 5;
        // Data
        encoded[64..69].copy_from_slice(s.as_bytes());

        let tokens = decode(&[ParamType::String], &encoded).unwrap();
        assert_eq!(tokens[0], Token::String(s.to_string()));
    }

    #[test]
    fn test_decode_multiple_params() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();

        let mut encoded = [0u8; 64];
        // Address
        encoded[12..32].copy_from_slice(addr.as_bytes());
        // Uint
        encoded[63] = 100;

        let tokens = decode(
            &[ParamType::Address, ParamType::Uint(256)],
            &encoded,
        ).unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::Address(addr));
        assert_eq!(tokens[1], Token::Uint(U256::from(100)));
    }

    #[test]
    fn test_decode_insufficient_data() {
        let data = [0u8; 16]; // Only 16 bytes, need 32
        let result = decode(&[ParamType::Uint(256)], &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_int_positive() {
        let mut encoded = [0u8; 32];
        encoded[31] = 100;

        let tokens = decode(&[ParamType::Int(256)], &encoded).unwrap();
        match &tokens[0] {
            Token::Int(i256) => {
                assert!(!i256.negative);
                assert_eq!(i256.abs, U256::from(100));
            }
            _ => panic!("Expected Int token"),
        }
    }

    #[test]
    fn test_decode_int_negative() {
        // -1 in two's complement is all 1s
        let encoded = [0xffu8; 32];

        let tokens = decode(&[ParamType::Int(256)], &encoded).unwrap();
        match &tokens[0] {
            Token::Int(i256) => {
                assert!(i256.negative);
                assert_eq!(i256.abs, U256::from(1));
            }
            _ => panic!("Expected Int token"),
        }
    }
}
