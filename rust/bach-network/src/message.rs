//! Network protocol messages

use bach_primitives::H256;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::peer::{PeerId, SerializablePeerInfo};

/// Protocol version for compatibility checking.
pub const PROTOCOL_VERSION: u32 = 1;

/// Consensus-related messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsensusMessage {
    /// Proposal for a new block
    Proposal {
        height: u64,
        round: u32,
        block_hash: [u8; 32],
        block_data: Vec<u8>,
    },
    /// Pre-vote for a proposal
    Prevote {
        height: u64,
        round: u32,
        block_hash: Option<[u8; 32]>,
        validator: [u8; 32],
        signature: Vec<u8>,
    },
    /// Pre-commit for a proposal
    Precommit {
        height: u64,
        round: u32,
        block_hash: Option<[u8; 32]>,
        validator: [u8; 32],
        signature: Vec<u8>,
    },
    /// Request missing votes
    VoteRequest {
        height: u64,
        round: u32,
    },
}

/// Serializable transaction for network transfer.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableTransaction {
    pub nonce: u64,
    pub to: Option<[u8; 20]>,
    pub value: [u8; 32],
    pub data: Vec<u8>,
    #[serde_as(as = "[_; 65]")]
    pub signature: [u8; 65],
}

/// Serializable block for network transfer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableBlock {
    pub height: u64,
    pub parent_hash: [u8; 32],
    pub transactions: Vec<SerializableTransaction>,
    pub timestamp: u64,
}

/// Network protocol messages.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkMessage {
    // ========== Handshake ==========
    /// Initial handshake message
    Hello {
        /// Protocol version
        version: u32,
        /// Sender's peer ID
        peer_id: [u8; 32],
        /// Genesis block hash for chain identification
        genesis_hash: [u8; 32],
        /// Sender's public key (64 bytes, uncompressed without prefix)
        #[serde_as(as = "[_; 64]")]
        public_key: [u8; 64],
    },

    /// Handshake acknowledgment
    HelloAck {
        /// Responder's peer ID
        peer_id: [u8; 32],
        /// Responder's public key
        #[serde_as(as = "[_; 64]")]
        public_key: [u8; 64],
    },

    // ========== Peer Discovery ==========
    /// Request peer list
    GetPeers,

    /// Response with peer list
    Peers(Vec<SerializablePeerInfo>),

    // ========== Transaction Propagation ==========
    /// Announce a new transaction
    NewTransaction(SerializableTransaction),

    /// Request transactions by hash
    GetTransactions(Vec<[u8; 32]>),

    /// Response with requested transactions
    Transactions(Vec<SerializableTransaction>),

    // ========== Block Propagation ==========
    /// Announce a new block
    NewBlock(SerializableBlock),

    /// Request blocks by height range
    GetBlocks {
        start: u64,
        count: u64,
    },

    /// Response with requested blocks
    Blocks(Vec<SerializableBlock>),

    /// Announce new block hash (lightweight notification)
    NewBlockHash {
        height: u64,
        hash: [u8; 32],
    },

    // ========== Consensus ==========
    /// Consensus protocol message
    Consensus(ConsensusMessage),

    // ========== Utilities ==========
    /// Ping for liveness check
    Ping(u64),

    /// Pong response
    Pong(u64),

    /// Graceful disconnect notification
    Disconnect {
        reason: String,
    },
}

impl NetworkMessage {
    /// Returns a short description of the message type for logging.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hello { .. } => "Hello",
            Self::HelloAck { .. } => "HelloAck",
            Self::GetPeers => "GetPeers",
            Self::Peers(_) => "Peers",
            Self::NewTransaction(_) => "NewTransaction",
            Self::GetTransactions(_) => "GetTransactions",
            Self::Transactions(_) => "Transactions",
            Self::NewBlock(_) => "NewBlock",
            Self::GetBlocks { .. } => "GetBlocks",
            Self::Blocks(_) => "Blocks",
            Self::NewBlockHash { .. } => "NewBlockHash",
            Self::Consensus(_) => "Consensus",
            Self::Ping(_) => "Ping",
            Self::Pong(_) => "Pong",
            Self::Disconnect { .. } => "Disconnect",
        }
    }

    /// Creates a Hello message.
    pub fn hello(peer_id: PeerId, genesis_hash: H256, public_key: [u8; 64]) -> Self {
        Self::Hello {
            version: PROTOCOL_VERSION,
            peer_id: peer_id.0,
            genesis_hash: *genesis_hash.as_bytes(),
            public_key,
        }
    }

    /// Creates a HelloAck message.
    pub fn hello_ack(peer_id: PeerId, public_key: [u8; 64]) -> Self {
        Self::HelloAck {
            peer_id: peer_id.0,
            public_key,
        }
    }

    /// Creates a Ping message with the current timestamp.
    pub fn ping() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        Self::Ping(nonce)
    }

    /// Creates a Pong response to a Ping.
    pub fn pong(nonce: u64) -> Self {
        Self::Pong(nonce)
    }

    /// Creates a Disconnect message.
    pub fn disconnect(reason: impl Into<String>) -> Self {
        Self::Disconnect {
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_name() {
        assert_eq!(NetworkMessage::GetPeers.name(), "GetPeers");
        assert_eq!(NetworkMessage::Ping(0).name(), "Ping");
    }

    #[test]
    fn test_hello_message() {
        let peer_id = PeerId::from_bytes([1u8; 32]);
        let genesis = H256::from([2u8; 32]);
        let pubkey = [3u8; 64];

        let msg = NetworkMessage::hello(peer_id, genesis, pubkey);
        match msg {
            NetworkMessage::Hello { version, peer_id: pid, genesis_hash, public_key } => {
                assert_eq!(version, PROTOCOL_VERSION);
                assert_eq!(pid, [1u8; 32]);
                assert_eq!(genesis_hash, [2u8; 32]);
                assert_eq!(public_key, [3u8; 64]);
            }
            _ => panic!("wrong message type"),
        }
    }

    #[test]
    fn test_ping_pong() {
        let ping = NetworkMessage::ping();
        if let NetworkMessage::Ping(nonce) = ping {
            let pong = NetworkMessage::pong(nonce);
            assert!(matches!(pong, NetworkMessage::Pong(n) if n == nonce));
        } else {
            panic!("expected Ping");
        }
    }
}
