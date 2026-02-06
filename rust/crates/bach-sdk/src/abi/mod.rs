//! ABI encoding and decoding for Solidity contracts
//!
//! This module provides functionality for:
//! - Encoding function calls
//! - Decoding function return values
//! - Computing function selectors
//!
//! # Example
//!
//! ```rust
//! use bach_sdk::abi::{encode, decode, function_selector, Token, ParamType};
//! use bach_primitives::{Address, U256};
//!
//! // Encode a transfer call
//! let to = Address::ZERO;
//! let amount = U256::from(1000);
//! let selector = function_selector("transfer(address,uint256)");
//! let data = encode(&[Token::Address(to), Token::Uint(amount)]);
//!
//! // Decode a balance response
//! let return_data = [0u8; 32]; // From eth_call
//! let balance = decode(&[ParamType::Uint(256)], &return_data).unwrap();
//! ```

mod decode;
mod encode;
mod types;

pub use decode::{decode, decode_output};
pub use encode::{encode, encode_function_call, function_selector, parse_type};
pub use types::{I256, ParamType, Token};
