//! ABI type definitions

use bach_primitives::{Address, H256, U256};

/// Solidity ABI token types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Address (20 bytes)
    Address(Address),
    /// Unsigned integer (8-256 bits)
    Uint(U256),
    /// Signed integer (8-256 bits)
    Int(I256),
    /// Boolean
    Bool(bool),
    /// Dynamic bytes
    Bytes(Vec<u8>),
    /// Fixed-size bytes (1-32)
    FixedBytes(Vec<u8>),
    /// UTF-8 string
    String(String),
    /// Dynamic array
    Array(Vec<Token>),
    /// Fixed-size array
    FixedArray(Vec<Token>),
    /// Tuple (struct)
    Tuple(Vec<Token>),
}

/// Signed 256-bit integer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct I256 {
    /// Absolute value
    pub abs: U256,
    /// Sign (true if negative)
    pub negative: bool,
}

impl I256 {
    /// Create a new I256
    pub fn new(abs: U256, negative: bool) -> Self {
        Self { abs, negative }
    }

    /// Create from i128
    pub fn from_i128(value: i128) -> Self {
        if value < 0 {
            Self {
                abs: U256::from((-value) as u128),
                negative: true,
            }
        } else {
            Self {
                abs: U256::from(value as u128),
                negative: false,
            }
        }
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.abs.is_zero()
    }
}

/// Solidity parameter types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamType {
    /// Address
    Address,
    /// Unsigned integer with bit size (8, 16, ..., 256)
    Uint(usize),
    /// Signed integer with bit size
    Int(usize),
    /// Boolean
    Bool,
    /// Dynamic bytes
    Bytes,
    /// Fixed-size bytes (size 1-32)
    FixedBytes(usize),
    /// UTF-8 string
    String,
    /// Dynamic array
    Array(Box<ParamType>),
    /// Fixed-size array
    FixedArray(Box<ParamType>, usize),
    /// Tuple
    Tuple(Vec<ParamType>),
}

impl ParamType {
    /// Check if this type is dynamic (variable length)
    pub fn is_dynamic(&self) -> bool {
        match self {
            ParamType::Bytes | ParamType::String | ParamType::Array(_) => true,
            ParamType::FixedArray(inner, _) => inner.is_dynamic(),
            ParamType::Tuple(types) => types.iter().any(|t| t.is_dynamic()),
            _ => false,
        }
    }
}

impl Token {
    /// Create an address token
    pub fn address(addr: Address) -> Self {
        Token::Address(addr)
    }

    /// Create a uint256 token
    pub fn uint256(value: U256) -> Self {
        Token::Uint(value)
    }

    /// Create a uint256 from u128
    pub fn uint256_from_u128(value: u128) -> Self {
        Token::Uint(U256::from(value))
    }

    /// Create a bool token
    pub fn bool(value: bool) -> Self {
        Token::Bool(value)
    }

    /// Create a bytes token
    pub fn bytes(data: Vec<u8>) -> Self {
        Token::Bytes(data)
    }

    /// Create a string token
    pub fn string(s: impl Into<String>) -> Self {
        Token::String(s.into())
    }

    /// Create a bytes32 token
    pub fn bytes32(data: H256) -> Self {
        Token::FixedBytes(data.as_bytes().to_vec())
    }

    /// Get the type of this token
    pub fn type_of(&self) -> ParamType {
        match self {
            Token::Address(_) => ParamType::Address,
            Token::Uint(_) => ParamType::Uint(256),
            Token::Int(_) => ParamType::Int(256),
            Token::Bool(_) => ParamType::Bool,
            Token::Bytes(_) => ParamType::Bytes,
            Token::FixedBytes(b) => ParamType::FixedBytes(b.len()),
            Token::String(_) => ParamType::String,
            Token::Array(tokens) => {
                let inner = tokens.first().map(|t| t.type_of()).unwrap_or(ParamType::Uint(256));
                ParamType::Array(Box::new(inner))
            }
            Token::FixedArray(tokens) => {
                let inner = tokens.first().map(|t| t.type_of()).unwrap_or(ParamType::Uint(256));
                ParamType::FixedArray(Box::new(inner), tokens.len())
            }
            Token::Tuple(tokens) => {
                ParamType::Tuple(tokens.iter().map(|t| t.type_of()).collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_type_is_dynamic() {
        assert!(!ParamType::Address.is_dynamic());
        assert!(!ParamType::Uint(256).is_dynamic());
        assert!(!ParamType::Bool.is_dynamic());
        assert!(!ParamType::FixedBytes(32).is_dynamic());

        assert!(ParamType::Bytes.is_dynamic());
        assert!(ParamType::String.is_dynamic());
        assert!(ParamType::Array(Box::new(ParamType::Uint(256))).is_dynamic());
    }

    #[test]
    fn test_token_type_of() {
        assert_eq!(Token::Address(Address::ZERO).type_of(), ParamType::Address);
        assert_eq!(Token::Uint(U256::zero()).type_of(), ParamType::Uint(256));
        assert_eq!(Token::Bool(true).type_of(), ParamType::Bool);
    }

    #[test]
    fn test_i256_from_i128() {
        let positive = I256::from_i128(100);
        assert!(!positive.negative);
        assert_eq!(positive.abs, U256::from(100));

        let negative = I256::from_i128(-100);
        assert!(negative.negative);
        assert_eq!(negative.abs, U256::from(100));

        let zero = I256::from_i128(0);
        assert!(zero.is_zero());
    }
}
