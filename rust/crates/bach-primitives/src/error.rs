//! Common error types for primitives

use thiserror::Error;
use crate::address::AddressError;
use crate::hash::HashError;

/// Primitive operation error
#[derive(Debug, Error)]
pub enum PrimitiveError {
    /// Address error
    #[error("address error: {0}")]
    Address(#[from] AddressError),

    /// Hash error
    #[error("hash error: {0}")]
    Hash(#[from] HashError),
}
