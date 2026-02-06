//! # bach-network
//!
//! P2P networking for BachLedger.
//!
//! This crate provides:
//! - Peer discovery and connection management
//! - Message broadcasting and routing
//! - Protocol handshake with chain validation
//! - Ping/pong keep-alive
//!
//! ## Architecture
//!
//! ```text
//! +-------------------+
//! |  NetworkService   |  <- Main service
//! +-------------------+
//!          |
//! +--------+--------+
//! | Listen | Connect|  <- TCP connections
//! +--------+--------+
//!          |
//! +-------------------+
//! |   PeerManager     |  <- Peer tracking
//! +-------------------+
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use bach_network::{NetworkService, NetworkConfig, NetworkEvent};
//!
//! // Configure network
//! let config = NetworkConfig {
//!     listen_addr: "0.0.0.0:30303".parse().unwrap(),
//!     ..Default::default()
//! };
//!
//! // Create service
//! let mut service = NetworkService::new(config);
//! let mut events = service.take_events().unwrap();
//!
//! // Start listening
//! service.start().await?;
//!
//! // Handle events
//! while let Some(event) = events.recv().await {
//!     match event {
//!         NetworkEvent::PeerConnected(peer_id) => { /* ... */ }
//!         NetworkEvent::Message { peer_id, message } => { /* ... */ }
//!         _ => {}
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod peer;
mod service;
mod types;

pub use error::{NetworkError, NetworkResult};
pub use peer::{PeerConnection, PeerInfo, PeerManager, PeerState};
pub use service::{NetworkConfig, NetworkEvent, NetworkService};
pub use types::{
    BlockAnnounce, GetBlock, Handshake, Message, MessageType, PeerId, TxBroadcast,
};
