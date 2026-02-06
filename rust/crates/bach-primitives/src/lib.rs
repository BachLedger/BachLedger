//! # bach-primitives
//!
//! Primitive types for BachLedger blockchain.
//!
//! This crate provides the fundamental data types used throughout the system.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod address;
mod hash;
mod error;

pub use address::Address;
pub use hash::{Hash, H256, H160};
pub use error::PrimitiveError;

// Re-export primitive-types for U256
pub use primitive_types::U256;

/// Block height type
pub type BlockHeight = u64;

/// Transaction nonce type
pub type Nonce = u64;

/// Gas type
pub type Gas = u64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u256_basic() {
        let a = U256::from(100u64);
        let b = U256::from(200u64);
        assert_eq!(a + b, U256::from(300u64));
    }
}
