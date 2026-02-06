//! # bach-rpc
//!
//! JSON-RPC 2.0 server implementation for BachLedger.
//!
//! This crate provides an Ethereum-compatible RPC interface that allows
//! clients to interact with the BachLedger blockchain.
//!
//! ## Features
//!
//! - Full JSON-RPC 2.0 support
//! - Ethereum-compatible `eth_*` methods
//! - Network information via `net_*` methods
//! - Utility methods via `web3_*` methods
//! - HTTP server with CORS support
//!
//! ## Usage
//!
//! ```ignore
//! use bach_rpc::{RpcServer, RpcHandler, RpcContext, ServerConfig};
//! use std::sync::Arc;
//!
//! // Create context with database and txpool
//! let ctx = Arc::new(RpcContext::new(
//!     state_db,
//!     block_db,
//!     txpool,
//!     chain_id,
//! ));
//!
//! // Create handler and server
//! let handler = RpcHandler::new(ctx);
//! let server = RpcServer::new(ServerConfig::default(), handler);
//!
//! // Run the server
//! server.run().await?;
//! ```
//!
//! ## Supported Methods
//!
//! ### eth_* Methods
//!
//! | Method | Description |
//! |--------|-------------|
//! | `eth_chainId` | Returns the chain ID |
//! | `eth_blockNumber` | Returns the current block number |
//! | `eth_gasPrice` | Returns the current gas price |
//! | `eth_getBalance` | Returns the balance of an account |
//! | `eth_getTransactionCount` | Returns the nonce of an account |
//! | `eth_getCode` | Returns the code at an address |
//! | `eth_getStorageAt` | Returns storage value at a position |
//! | `eth_call` | Executes a call without creating a transaction |
//! | `eth_estimateGas` | Estimates gas for a transaction |
//! | `eth_sendRawTransaction` | Submits a raw transaction |
//! | `eth_getBlockByNumber` | Returns block by number |
//! | `eth_getBlockByHash` | Returns block by hash |
//! | `eth_getTransactionByHash` | Returns transaction by hash |
//! | `eth_getTransactionReceipt` | Returns transaction receipt |
//!
//! ### net_* Methods
//!
//! | Method | Description |
//! |--------|-------------|
//! | `net_version` | Returns the network ID |
//! | `net_listening` | Returns true if listening |
//! | `net_peerCount` | Returns the number of peers |
//!
//! ### web3_* Methods
//!
//! | Method | Description |
//! |--------|-------------|
//! | `web3_clientVersion` | Returns the client version |
//! | `web3_sha3` | Returns Keccak-256 hash of data |

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod handler;
pub mod methods;
pub mod server;
pub mod types;

// Re-export main types
pub use error::{JsonRpcError, RpcError, RpcResult};
pub use handler::{MethodRegistry, RpcContext, RpcHandler};
pub use server::{RpcServer, ServerConfig};
pub use types::{
    BlockId, CallRequest, CallRequestRaw, JsonRpcId, JsonRpcRequest, JsonRpcResponse, RpcBlock,
    RpcLog, RpcReceipt, RpcTransaction,
};
