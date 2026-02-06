//! # bach-sdk
//!
//! Rust SDK for BachLedger blockchain.
//!
//! ## Features
//!
//! - **BachClient**: RPC client for communicating with BachLedger nodes
//! - **Wallet**: Account management and transaction signing
//! - **TxBuilder**: Fluent API for building transactions
//! - **Contract**: Helpers for encoding/decoding contract calls
//! - **ABI**: Solidity ABI encoding and decoding
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use bach_sdk::{BachClient, Wallet, TxBuilder};
//! use bach_primitives::Address;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a mock client for testing
//!     let client = BachClient::new_mock();
//!
//!     // Create a wallet
//!     let wallet = Wallet::new_random();
//!     println!("Address: {}", wallet.address().to_hex());
//!
//!     // Get balance
//!     let balance = client.get_balance(wallet.address(), Default::default()).await?;
//!     println!("Balance: {:?}", balance);
//!
//!     // Build and sign a transaction
//!     let to = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d")?;
//!     let tx = TxBuilder::new(1)
//!         .nonce(0)
//!         .gas_limit(21000)
//!         .gas_price(1_000_000_000)
//!         .to(to)
//!         .value(1_000_000_000_000_000_000) // 1 ETH
//!         .sign_legacy(&wallet)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Contract Interaction
//!
//! ```rust,no_run
//! use bach_sdk::{BachClient, contract, abi::Token};
//! use bach_sdk::types::{BlockId, CallRequest};
//! use bach_primitives::{Address, U256};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = BachClient::new_mock();
//!
//!     // Create ERC20 contract helper
//!     let token = Address::from_hex("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
//!     let contract = contract::erc20(token);
//!
//!     // Encode balanceOf call
//!     let owner = Address::from_hex("0x742d35Cc6634C0532925a3b844Bc9e7595f0aB3d")?;
//!     let data = contract.encode_call("balanceOf", &[Token::Address(owner)])?;
//!
//!     // Execute call
//!     let result = client.call(&CallRequest {
//!         to: Some(token),
//!         data: Some(data),
//!         ..Default::default()
//!     }, BlockId::Latest).await?;
//!
//!     // Decode result
//!     let tokens = contract.decode_output("balanceOf", &result)?;
//!     println!("Balance: {:?}", tokens[0]);
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod abi;
mod client;
pub mod contract;
mod error;
mod transport;
mod tx_builder;
pub mod types;
mod wallet;

// Re-export main types
pub use client::BachClient;
pub use error::SdkError;
pub use transport::MockTransport;

/// Re-export Transport trait for custom implementations
pub use transport::Transport;
pub use tx_builder::TxBuilder;
pub use wallet::Wallet;

#[cfg(feature = "http")]
pub use transport::HttpTransport;

// Re-export primitives for convenience
pub use bach_primitives::{Address, BlockHeight, Gas, H256, Nonce, U256};
pub use bach_types::{Block, Receipt, SignedTransaction};
