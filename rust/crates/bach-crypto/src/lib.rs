//! # bach-crypto
//!
//! Cryptographic primitives for BachLedger.
//!
//! - Keccak-256 hashing
//! - ECDSA signing/verification (secp256k1)
//! - Public key recovery
//! - Address derivation

#![warn(missing_docs)]
#![warn(clippy::all)]

mod hash;
mod signature;
mod error;

pub use hash::keccak256;
pub use signature::{
    sign, verify, recover_public_key, public_key_to_address,
    Signature, PublicKey, PrivateKey,
};
pub use error::CryptoError;
