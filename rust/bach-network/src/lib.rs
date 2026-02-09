//! BachLedger Network
//!
//! P2P networking layer for the medical blockchain.
//!
//! # Architecture
//!
//! - `PeerId`: 32-byte identifier derived from public key
//! - `PeerInfo`: Information about a connected peer
//! - `PeerManager`: Manages peer connections and discovery
//! - `NetworkMessage`: Protocol messages for peer communication
//! - `NetworkService`: Main service handling connections and messaging
//!
//! # Example
//!
//! ```ignore
//! use bach_network::{NetworkConfig, NetworkService};
//!
//! let config = NetworkConfig::default()
//!     .with_listen_addr("0.0.0.0:30303".parse().unwrap());
//!
//! let mut service = NetworkService::new(config).await?;
//! service.start().await?;
//! ```

#![forbid(unsafe_code)]

mod codec;
mod error;
mod message;
mod peer;
mod service;

pub use codec::MessageCodec;
pub use error::NetworkError;
pub use message::{ConsensusMessage, NetworkMessage, PROTOCOL_VERSION};
pub use peer::{PeerId, PeerInfo, PeerManager, PeerStatus};
pub use service::{NetworkCommand, NetworkConfig, NetworkEvent, NetworkService};
