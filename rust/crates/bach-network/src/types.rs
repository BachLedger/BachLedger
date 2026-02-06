//! Network types

use bach_primitives::H256;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Peer identifier (32 bytes)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId([u8; 32]);

impl PeerId {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Generate random peer ID
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut bytes);
        Self(bytes)
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0[..8]))
    }
}

/// Network message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Handshake message
    Handshake = 0,
    /// Ping message
    Ping = 1,
    /// Pong response
    Pong = 2,
    /// Block announcement
    BlockAnnounce = 10,
    /// Transaction broadcast
    TxBroadcast = 11,
    /// Consensus message
    Consensus = 20,
    /// Block request
    GetBlock = 30,
    /// Block response
    Block = 31,
    /// Disconnect
    Disconnect = 255,
}

impl TryFrom<u8> for MessageType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Handshake),
            1 => Ok(Self::Ping),
            2 => Ok(Self::Pong),
            10 => Ok(Self::BlockAnnounce),
            11 => Ok(Self::TxBroadcast),
            20 => Ok(Self::Consensus),
            30 => Ok(Self::GetBlock),
            31 => Ok(Self::Block),
            255 => Ok(Self::Disconnect),
            _ => Err(()),
        }
    }
}

/// Handshake data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    /// Protocol version
    pub version: u32,
    /// Chain ID
    pub chain_id: u64,
    /// Genesis block hash
    pub genesis_hash: H256,
    /// Current block height
    pub height: u64,
    /// Peer ID
    pub peer_id: PeerId,
}

impl Handshake {
    /// Create a new handshake
    pub fn new(version: u32, chain_id: u64, genesis_hash: H256, height: u64, peer_id: PeerId) -> Self {
        Self {
            version,
            chain_id,
            genesis_hash,
            height,
            peer_id,
        }
    }
}

/// Network message
#[derive(Debug, Clone)]
pub struct Message {
    /// Message type
    pub msg_type: MessageType,
    /// Payload
    pub payload: Bytes,
}

impl Message {
    /// Create a new message
    pub fn new(msg_type: MessageType, payload: Bytes) -> Self {
        Self { msg_type, payload }
    }

    /// Create empty message
    pub fn empty(msg_type: MessageType) -> Self {
        Self {
            msg_type,
            payload: Bytes::new(),
        }
    }

    /// Create ping message
    pub fn ping() -> Self {
        Self::empty(MessageType::Ping)
    }

    /// Create pong message
    pub fn pong() -> Self {
        Self::empty(MessageType::Pong)
    }

    /// Create disconnect message
    pub fn disconnect() -> Self {
        Self::empty(MessageType::Disconnect)
    }

    /// Encode message to bytes
    /// Format: [length: 4 bytes][type: 1 byte][payload: N bytes]
    pub fn encode(&self) -> Bytes {
        let len = 1 + self.payload.len();
        let mut buf = BytesMut::with_capacity(4 + len);
        buf.put_u32(len as u32);
        buf.put_u8(self.msg_type as u8);
        buf.put_slice(&self.payload);
        buf.freeze()
    }

    /// Decode message from bytes
    pub fn decode(mut data: Bytes) -> Option<Self> {
        if data.len() < 5 {
            return None;
        }
        let len = data.get_u32() as usize;
        if data.len() < len || len < 1 {
            return None;
        }
        let msg_type = MessageType::try_from(data.get_u8()).ok()?;
        let payload = data.split_to(len - 1);
        Some(Self { msg_type, payload })
    }
}

/// Block announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAnnounce {
    /// Block hash
    pub hash: H256,
    /// Block number
    pub number: u64,
    /// Parent hash
    pub parent_hash: H256,
}

impl BlockAnnounce {
    /// Create new block announcement
    pub fn new(hash: H256, number: u64, parent_hash: H256) -> Self {
        Self {
            hash,
            number,
            parent_hash,
        }
    }
}

/// Transaction broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxBroadcast {
    /// Transaction hash
    pub hash: H256,
    /// Encoded transaction
    pub data: Vec<u8>,
}

impl TxBroadcast {
    /// Create new transaction broadcast
    pub fn new(hash: H256, data: Vec<u8>) -> Self {
        Self { hash, data }
    }
}

/// Block request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlock {
    /// Block hash or number
    pub by_hash: Option<H256>,
    /// Block number (if not by hash)
    pub by_number: Option<u64>,
}

impl GetBlock {
    /// Request by hash
    pub fn by_hash(hash: H256) -> Self {
        Self {
            by_hash: Some(hash),
            by_number: None,
        }
    }

    /// Request by number
    pub fn by_number(number: u64) -> Self {
        Self {
            by_hash: None,
            by_number: Some(number),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id() {
        let id = PeerId::random();
        assert_eq!(id.as_bytes().len(), 32);

        let id2 = PeerId::from_bytes(*id.as_bytes());
        assert_eq!(id, id2);
    }

    #[test]
    fn test_peer_id_from_bytes() {
        let bytes = [42u8; 32];
        let id = PeerId::from_bytes(bytes);
        assert_eq!(*id.as_bytes(), bytes);
    }

    #[test]
    fn test_peer_id_display() {
        let bytes = [0xAB; 32];
        let id = PeerId::from_bytes(bytes);
        let display = format!("{}", id);
        // Display shows first 8 bytes hex-encoded
        assert_eq!(display, "abababababababab");
    }

    #[test]
    fn test_peer_id_debug() {
        let bytes = [0xCD; 32];
        let id = PeerId::from_bytes(bytes);
        let debug = format!("{:?}", id);
        assert!(debug.contains("PeerId("));
        assert!(debug.contains("cdcdcdcdcdcdcdcd"));
    }

    #[test]
    fn test_peer_id_equality() {
        let bytes = [1u8; 32];
        let id1 = PeerId::from_bytes(bytes);
        let id2 = PeerId::from_bytes(bytes);
        let id3 = PeerId::from_bytes([2u8; 32]);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_peer_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let id1 = PeerId::from_bytes([1u8; 32]);
        let id2 = PeerId::from_bytes([2u8; 32]);

        set.insert(id1);
        set.insert(id2);
        set.insert(id1); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_peer_id_clone() {
        let id = PeerId::random();
        let cloned = id;
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_message_encode_decode() {
        let msg = Message::new(MessageType::BlockAnnounce, Bytes::from("test payload"));
        let encoded = msg.encode();
        let decoded = Message::decode(encoded).unwrap();

        assert_eq!(decoded.msg_type, MessageType::BlockAnnounce);
        assert_eq!(decoded.payload, Bytes::from("test payload"));
    }

    #[test]
    fn test_message_empty() {
        let msg = Message::ping();
        assert_eq!(msg.msg_type, MessageType::Ping);
        assert!(msg.payload.is_empty());

        let encoded = msg.encode();
        let decoded = Message::decode(encoded).unwrap();
        assert_eq!(decoded.msg_type, MessageType::Ping);
    }

    #[test]
    fn test_message_pong() {
        let msg = Message::pong();
        assert_eq!(msg.msg_type, MessageType::Pong);
        assert!(msg.payload.is_empty());
    }

    #[test]
    fn test_message_disconnect() {
        let msg = Message::disconnect();
        assert_eq!(msg.msg_type, MessageType::Disconnect);
        assert!(msg.payload.is_empty());
    }

    #[test]
    fn test_message_decode_too_short() {
        // Less than 5 bytes should fail
        let data = Bytes::from(vec![0, 0, 0, 1]);
        assert!(Message::decode(data).is_none());
    }

    #[test]
    fn test_message_decode_invalid_length() {
        // Length says 100 bytes but only 1 byte present
        let mut data = BytesMut::new();
        data.put_u32(100);
        data.put_u8(0); // type
        assert!(Message::decode(data.freeze()).is_none());
    }

    #[test]
    fn test_message_decode_zero_length() {
        let mut data = BytesMut::new();
        data.put_u32(0);
        data.put_u8(0);
        assert!(Message::decode(data.freeze()).is_none());
    }

    #[test]
    fn test_message_decode_invalid_type() {
        let mut data = BytesMut::new();
        data.put_u32(1);
        data.put_u8(100); // Invalid type
        assert!(Message::decode(data.freeze()).is_none());
    }

    #[test]
    fn test_message_large_payload() {
        let payload = vec![0xAB; 1000];
        let msg = Message::new(MessageType::Block, Bytes::from(payload.clone()));
        let encoded = msg.encode();
        let decoded = Message::decode(encoded).unwrap();

        assert_eq!(decoded.msg_type, MessageType::Block);
        assert_eq!(decoded.payload.len(), 1000);
    }

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(MessageType::try_from(0u8), Ok(MessageType::Handshake));
        assert_eq!(MessageType::try_from(1u8), Ok(MessageType::Ping));
        assert_eq!(MessageType::try_from(255u8), Ok(MessageType::Disconnect));
        assert!(MessageType::try_from(100u8).is_err());
    }

    #[test]
    fn test_message_type_all_variants() {
        assert_eq!(MessageType::try_from(0u8), Ok(MessageType::Handshake));
        assert_eq!(MessageType::try_from(1u8), Ok(MessageType::Ping));
        assert_eq!(MessageType::try_from(2u8), Ok(MessageType::Pong));
        assert_eq!(MessageType::try_from(10u8), Ok(MessageType::BlockAnnounce));
        assert_eq!(MessageType::try_from(11u8), Ok(MessageType::TxBroadcast));
        assert_eq!(MessageType::try_from(20u8), Ok(MessageType::Consensus));
        assert_eq!(MessageType::try_from(30u8), Ok(MessageType::GetBlock));
        assert_eq!(MessageType::try_from(31u8), Ok(MessageType::Block));
        assert_eq!(MessageType::try_from(255u8), Ok(MessageType::Disconnect));
    }

    #[test]
    fn test_message_type_invalid_values() {
        assert!(MessageType::try_from(3u8).is_err());
        assert!(MessageType::try_from(9u8).is_err());
        assert!(MessageType::try_from(12u8).is_err());
        assert!(MessageType::try_from(19u8).is_err());
        assert!(MessageType::try_from(21u8).is_err());
        assert!(MessageType::try_from(29u8).is_err());
        assert!(MessageType::try_from(32u8).is_err());
        assert!(MessageType::try_from(254u8).is_err());
    }

    #[test]
    fn test_handshake_new() {
        let peer_id = PeerId::random();
        let genesis = H256::from_bytes([1; 32]);
        let hs = Handshake::new(1, 42, genesis, 100, peer_id);

        assert_eq!(hs.version, 1);
        assert_eq!(hs.chain_id, 42);
        assert_eq!(hs.genesis_hash, genesis);
        assert_eq!(hs.height, 100);
        assert_eq!(hs.peer_id, peer_id);
    }

    #[test]
    fn test_handshake_serialize() {
        let peer_id = PeerId::from_bytes([0xAA; 32]);
        let genesis = H256::from_bytes([0xBB; 32]);
        let hs = Handshake::new(1, 1, genesis, 0, peer_id);

        let json = serde_json::to_string(&hs).unwrap();
        let decoded: Handshake = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.version, hs.version);
        assert_eq!(decoded.chain_id, hs.chain_id);
        assert_eq!(decoded.peer_id, hs.peer_id);
    }

    #[test]
    fn test_block_announce() {
        let ann = BlockAnnounce::new(
            H256::from_bytes([1; 32]),
            100,
            H256::from_bytes([2; 32]),
        );
        assert_eq!(ann.number, 100);
    }

    #[test]
    fn test_block_announce_fields() {
        let hash = H256::from_bytes([0x11; 32]);
        let parent = H256::from_bytes([0x22; 32]);
        let ann = BlockAnnounce::new(hash, 42, parent);

        assert_eq!(ann.hash, hash);
        assert_eq!(ann.number, 42);
        assert_eq!(ann.parent_hash, parent);
    }

    #[test]
    fn test_block_announce_serialize() {
        let ann = BlockAnnounce::new(
            H256::from_bytes([1; 32]),
            100,
            H256::from_bytes([2; 32]),
        );

        let json = serde_json::to_string(&ann).unwrap();
        let decoded: BlockAnnounce = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.number, 100);
    }

    #[test]
    fn test_tx_broadcast_new() {
        let hash = H256::from_bytes([0xAA; 32]);
        let data = vec![1, 2, 3, 4, 5];
        let tx = TxBroadcast::new(hash, data.clone());

        assert_eq!(tx.hash, hash);
        assert_eq!(tx.data, data);
    }

    #[test]
    fn test_tx_broadcast_serialize() {
        let tx = TxBroadcast::new(H256::from_bytes([1; 32]), vec![0xDE, 0xAD]);

        let json = serde_json::to_string(&tx).unwrap();
        let decoded: TxBroadcast = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.data, vec![0xDE, 0xAD]);
    }

    #[test]
    fn test_get_block() {
        let by_hash = GetBlock::by_hash(H256::from_bytes([1; 32]));
        assert!(by_hash.by_hash.is_some());
        assert!(by_hash.by_number.is_none());

        let by_num = GetBlock::by_number(100);
        assert!(by_num.by_hash.is_none());
        assert_eq!(by_num.by_number, Some(100));
    }

    #[test]
    fn test_get_block_serialize() {
        let req = GetBlock::by_number(42);

        let json = serde_json::to_string(&req).unwrap();
        let decoded: GetBlock = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.by_number, Some(42));
        assert!(decoded.by_hash.is_none());
    }

    #[test]
    fn test_message_clone() {
        let msg = Message::new(MessageType::Block, Bytes::from("data"));
        let cloned = msg.clone();

        assert_eq!(msg.msg_type, cloned.msg_type);
        assert_eq!(msg.payload, cloned.payload);
    }

    #[test]
    fn test_message_debug() {
        let msg = Message::ping();
        let debug = format!("{:?}", msg);
        assert!(debug.contains("Ping"));
    }
}
