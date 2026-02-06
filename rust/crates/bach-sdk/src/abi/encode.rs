//! ABI encoding

use bach_primitives::U256;

use super::types::{ParamType, Token};
use crate::SdkError;

/// Encode tokens according to Solidity ABI specification
pub fn encode(tokens: &[Token]) -> Vec<u8> {
    let types: Vec<ParamType> = tokens.iter().map(|t| t.type_of()).collect();
    encode_params(&types, tokens)
}

/// Encode function call (selector + params)
pub fn encode_function_call(selector: [u8; 4], tokens: &[Token]) -> Vec<u8> {
    let mut result = selector.to_vec();
    result.extend(encode(tokens));
    result
}

/// Encode parameters
fn encode_params(types: &[ParamType], tokens: &[Token]) -> Vec<u8> {
    // Calculate head size (fixed part)
    let head_size = types.iter().map(head_length).sum::<usize>();

    let mut head = Vec::new();
    let mut tail = Vec::new();

    for (param_type, token) in types.iter().zip(tokens.iter()) {
        if param_type.is_dynamic() {
            // Encode offset to tail
            let offset = head_size + tail.len();
            head.extend(encode_u256(&U256::from(offset)));
            // Encode actual data in tail
            tail.extend(encode_token(param_type, token));
        } else {
            // Encode directly in head
            head.extend(encode_token(param_type, token));
        }
    }

    head.extend(tail);
    head
}

/// Get the head length for a type
fn head_length(param_type: &ParamType) -> usize {
    match param_type {
        ParamType::FixedArray(inner, size) if !inner.is_dynamic() => {
            head_length(inner) * size
        }
        ParamType::Tuple(types) if !types.iter().any(|t| t.is_dynamic()) => {
            types.iter().map(head_length).sum()
        }
        _ => 32, // All other types are 32 bytes in head
    }
}

/// Encode a single token
fn encode_token(param_type: &ParamType, token: &Token) -> Vec<u8> {
    match (param_type, token) {
        (ParamType::Address, Token::Address(addr)) => {
            let mut buf = [0u8; 32];
            buf[12..32].copy_from_slice(addr.as_bytes());
            buf.to_vec()
        }
        (ParamType::Uint(_), Token::Uint(value)) => encode_u256(value),
        (ParamType::Int(_), Token::Int(value)) => {
            if value.negative {
                // Two's complement for negative numbers
                let abs_bytes = u256_to_bytes(&value.abs);
                let mut bytes = [0xffu8; 32];
                // Negate: flip bits and add 1
                for i in 0..32 {
                    bytes[i] = !abs_bytes[i];
                }
                // Add 1 (simple implementation for non-zero values)
                let mut carry = 1u16;
                for i in (0..32).rev() {
                    let sum = (bytes[i] as u16) + carry;
                    bytes[i] = sum as u8;
                    carry = sum >> 8;
                }
                bytes.to_vec()
            } else {
                encode_u256(&value.abs)
            }
        }
        (ParamType::Bool, Token::Bool(b)) => {
            let mut buf = [0u8; 32];
            buf[31] = if *b { 1 } else { 0 };
            buf.to_vec()
        }
        (ParamType::FixedBytes(size), Token::FixedBytes(data)) => {
            let mut buf = [0u8; 32];
            let len = data.len().min(*size);
            buf[..len].copy_from_slice(&data[..len]);
            buf.to_vec()
        }
        (ParamType::Bytes, Token::Bytes(data)) => {
            encode_bytes(data)
        }
        (ParamType::String, Token::String(s)) => {
            encode_bytes(s.as_bytes())
        }
        (ParamType::Array(inner), Token::Array(tokens)) => {
            let mut result = encode_u256(&U256::from(tokens.len()));
            let inner_types: Vec<ParamType> = tokens.iter().map(|_| (**inner).clone()).collect();
            result.extend(encode_params(&inner_types, tokens));
            result
        }
        (ParamType::FixedArray(inner, _), Token::FixedArray(tokens)) => {
            let inner_types: Vec<ParamType> = tokens.iter().map(|_| (**inner).clone()).collect();
            encode_params(&inner_types, tokens)
        }
        (ParamType::Tuple(types), Token::Tuple(tokens)) => {
            encode_params(types, tokens)
        }
        _ => vec![0u8; 32], // Fallback
    }
}

/// Convert U256 to 32-byte big-endian array
fn u256_to_bytes(value: &U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    value.to_big_endian(&mut bytes);
    bytes
}

/// Encode a U256 as 32 bytes
fn encode_u256(value: &U256) -> Vec<u8> {
    u256_to_bytes(value).to_vec()
}

/// Encode dynamic bytes
fn encode_bytes(data: &[u8]) -> Vec<u8> {
    let mut result = encode_u256(&U256::from(data.len()));

    // Pad to 32 bytes
    let padded_len = data.len().div_ceil(32) * 32;
    let mut padded = vec![0u8; padded_len];
    padded[..data.len()].copy_from_slice(data);
    result.extend(padded);

    result
}

/// Compute function selector (first 4 bytes of keccak256(signature))
pub fn function_selector(signature: &str) -> [u8; 4] {
    let hash = bach_crypto::keccak256(signature.as_bytes());
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&hash.as_bytes()[..4]);
    selector
}

/// Parse a simple type string (e.g., "uint256", "address")
pub fn parse_type(s: &str) -> Result<ParamType, SdkError> {
    let s = s.trim();

    if s == "address" {
        return Ok(ParamType::Address);
    }
    if s == "bool" {
        return Ok(ParamType::Bool);
    }
    if s == "string" {
        return Ok(ParamType::String);
    }
    if s == "bytes" {
        return Ok(ParamType::Bytes);
    }

    // uint<N>
    if let Some(rest) = s.strip_prefix("uint") {
        let bits: usize = if rest.is_empty() {
            256
        } else {
            rest.parse().map_err(|_| SdkError::AbiEncode(format!("Invalid uint size: {}", rest)))?
        };
        return Ok(ParamType::Uint(bits));
    }

    // int<N>
    if let Some(rest) = s.strip_prefix("int") {
        let bits: usize = if rest.is_empty() {
            256
        } else {
            rest.parse().map_err(|_| SdkError::AbiEncode(format!("Invalid int size: {}", rest)))?
        };
        return Ok(ParamType::Int(bits));
    }

    // bytes<N>
    if let Some(rest) = s.strip_prefix("bytes") {
        if !rest.is_empty() {
            let size: usize = rest.parse().map_err(|_| SdkError::AbiEncode(format!("Invalid bytes size: {}", rest)))?;
            return Ok(ParamType::FixedBytes(size));
        }
    }

    Err(SdkError::AbiEncode(format!("Unknown type: {}", s)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bach_primitives::Address;

    #[test]
    fn test_encode_address() {
        let addr = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let encoded = encode(&[Token::Address(addr)]);

        assert_eq!(encoded.len(), 32);
        // Address should be right-padded in 32 bytes
        assert_eq!(&encoded[12..32], addr.as_bytes());
    }

    #[test]
    fn test_encode_uint() {
        let encoded = encode(&[Token::Uint(U256::from(100))]);
        assert_eq!(encoded.len(), 32);
        assert_eq!(encoded[31], 100);
    }

    #[test]
    fn test_encode_bool() {
        let encoded_true = encode(&[Token::Bool(true)]);
        let encoded_false = encode(&[Token::Bool(false)]);

        assert_eq!(encoded_true[31], 1);
        assert_eq!(encoded_false[31], 0);
    }

    #[test]
    fn test_encode_bytes32() {
        let data = [0x42u8; 32];
        let encoded = encode(&[Token::FixedBytes(data.to_vec())]);

        assert_eq!(encoded.len(), 32);
        assert_eq!(&encoded[..], &data[..]);
    }

    #[test]
    fn test_encode_dynamic_bytes() {
        let data = vec![0x01, 0x02, 0x03];
        let encoded = encode(&[Token::Bytes(data.clone())]);

        // Should be: offset (32) + length (32) + padded data (32)
        assert_eq!(encoded.len(), 96);
        // Offset to data (32)
        assert_eq!(encoded[31], 32);
        // Length (3)
        assert_eq!(encoded[63], 3);
        // Data
        assert_eq!(&encoded[64..67], &data[..]);
    }

    #[test]
    fn test_encode_string() {
        let s = "hello";
        let encoded = encode(&[Token::String(s.to_string())]);

        // Offset + length + padded data
        assert_eq!(encoded.len(), 96);
    }

    #[test]
    fn test_function_selector() {
        // transfer(address,uint256)
        let selector = function_selector("transfer(address,uint256)");
        assert_eq!(selector, [0xa9, 0x05, 0x9c, 0xbb]);

        // balanceOf(address)
        let selector = function_selector("balanceOf(address)");
        assert_eq!(selector, [0x70, 0xa0, 0x82, 0x31]);
    }

    #[test]
    fn test_encode_function_call() {
        let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d").unwrap();
        let selector = function_selector("transfer(address,uint256)");
        let encoded = encode_function_call(
            selector,
            &[
                Token::Address(to),
                Token::Uint(U256::from(1000)),
            ],
        );

        // 4 bytes selector + 32 bytes address + 32 bytes uint
        assert_eq!(encoded.len(), 68);
        assert_eq!(&encoded[..4], &selector);
    }

    #[test]
    fn test_parse_type() {
        assert_eq!(parse_type("address").unwrap(), ParamType::Address);
        assert_eq!(parse_type("uint256").unwrap(), ParamType::Uint(256));
        assert_eq!(parse_type("uint").unwrap(), ParamType::Uint(256));
        assert_eq!(parse_type("uint8").unwrap(), ParamType::Uint(8));
        assert_eq!(parse_type("bool").unwrap(), ParamType::Bool);
        assert_eq!(parse_type("bytes").unwrap(), ParamType::Bytes);
        assert_eq!(parse_type("bytes32").unwrap(), ParamType::FixedBytes(32));
        assert_eq!(parse_type("string").unwrap(), ParamType::String);
    }
}
