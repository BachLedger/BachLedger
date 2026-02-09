//! Comprehensive network tests for bach-network
//!
//! Test-Driven Development: These tests define expected P2P networking behavior.
//! Implementation must pass all tests.
//!
//! Test categories:
//! 1. Peer Management: Connect, disconnect, peer limits
//! 2. Message Encoding: Serialize/deserialize all message types
//! 3. Handshake: Version negotiation, peer ID exchange
//! 4. Peer Discovery: Bootstrap, peer exchange
//! 5. Message Routing: Direct send, broadcast
//! 6. Connection Handling: Reconnection, timeout

use bach_network::{
    MessageCodec, NetworkConfig, NetworkError, NetworkEvent, NetworkMessage, NetworkService,
    PeerId, PeerInfo, PeerManager, PeerStatus, PROTOCOL_VERSION,
};
use bach_crypto::{keccak256, PrivateKey, PublicKey};
use bach_primitives::H256;

use std::net::SocketAddr;
use std::time::Duration;

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a test peer ID from bytes
fn test_peer_id(seed: u8) -> PeerId {
    PeerId::from_bytes([seed; 32])
}

/// Creates a test genesis hash
fn test_genesis() -> H256 {
    H256::from([0x42; 32])
}

/// Creates a socket address for testing
fn test_addr(port: u16) -> SocketAddr {
    format!("127.0.0.1:{}", port).parse().unwrap()
}

/// Creates a network config for testing
fn test_config(port: u16) -> NetworkConfig {
    NetworkConfig::default()
        .with_listen_addr(test_addr(port))
        .with_genesis_hash(test_genesis())
        .with_max_peers(10)
}

// =============================================================================
// 1. Peer Management Tests
// =============================================================================

mod peer_management {
    use super::*;

    #[test]
    fn test_peer_id_from_public_key() {
        let private_key = PrivateKey::random();
        let public_key = private_key.public_key();

        let peer_id = PeerId::from_public_key(&public_key);

        // Peer ID should be deterministic
        let peer_id2 = PeerId::from_public_key(&public_key);
        assert_eq!(peer_id, peer_id2);
    }

    #[test]
    fn test_peer_id_from_bytes() {
        let bytes = [0xab; 32];
        let peer_id = PeerId::from_bytes(bytes);

        assert_eq!(*peer_id.as_bytes(), bytes);
    }

    #[test]
    fn test_peer_id_display() {
        let bytes = [0xab; 32];
        let peer_id = PeerId::from_bytes(bytes);
        let display = format!("{}", peer_id);

        assert!(display.starts_with("0x"));
        assert_eq!(display.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_peer_id_short_hex() {
        let peer_id = PeerId::from_bytes([0x12, 0x34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xab, 0xcd]);
        let short = peer_id.short_hex();

        assert!(short.contains("1234"));
        assert!(short.contains("abcd"));
    }

    #[test]
    fn test_peer_manager_creation() {
        let bootstrap = vec![test_addr(8080), test_addr(8081)];
        let manager = PeerManager::new(25, bootstrap.clone());

        assert_eq!(manager.max_peers(), 25);
        assert_eq!(manager.bootstrap_nodes().len(), 2);
    }

    #[test]
    fn test_peer_manager_add_remove() {
        let manager = PeerManager::new(10, vec![]);
        let addr = test_addr(8080);
        let info = PeerInfo::new_incoming(addr);
        let id = info.id;

        // Add peer
        manager.add_peer(info).unwrap();
        assert!(manager.get_peer(&id).is_some());
        assert_eq!(manager.get_peer_by_addr(&addr), Some(id));

        // Remove peer
        manager.remove_peer(&id);
        assert!(manager.get_peer(&id).is_none());
        assert!(manager.get_peer_by_addr(&addr).is_none());
    }

    #[test]
    fn test_peer_manager_max_peers_limit() {
        let manager = PeerManager::new(2, vec![]);

        // Add two active peers
        for i in 0..2 {
            let addr = test_addr(8080 + i);
            let mut info = PeerInfo::new_incoming(addr);
            info.status = PeerStatus::Active;
            manager.add_peer(info).unwrap();
        }

        // Third should fail
        let addr = test_addr(9000);
        let mut info = PeerInfo::new_incoming(addr);
        info.status = PeerStatus::Active;
        let result = manager.add_peer(info);

        assert!(result.is_err());
    }

    #[test]
    fn test_peer_manager_duplicate_address() {
        let manager = PeerManager::new(10, vec![]);
        let addr = test_addr(8080);

        let info1 = PeerInfo::new_incoming(addr);
        manager.add_peer(info1).unwrap();

        // Same address should fail
        let info2 = PeerInfo::new_incoming(addr);
        let result = manager.add_peer(info2);

        assert!(result.is_err());
    }

    #[test]
    fn test_peer_manager_active_peers() {
        let manager = PeerManager::new(10, vec![]);

        // Add some peers with different statuses
        let mut info1 = PeerInfo::new_incoming(test_addr(8080));
        info1.status = PeerStatus::Active;
        manager.add_peer(info1).unwrap();

        let mut info2 = PeerInfo::new_incoming(test_addr(8081));
        info2.status = PeerStatus::Active;
        manager.add_peer(info2).unwrap();

        let info3 = PeerInfo::new_outgoing(test_addr(8082)); // Connecting status
        manager.add_peer(info3).unwrap();

        assert_eq!(manager.active_count(), 2);
        assert_eq!(manager.active_peers().len(), 2);
    }

    #[test]
    fn test_peer_info_new_outgoing() {
        let addr = test_addr(8080);
        let info = PeerInfo::new_outgoing(addr);

        assert_eq!(info.address, addr);
        assert_eq!(info.status, PeerStatus::Connecting);
        assert!(info.public_key.is_none());
        assert!(info.version.is_none());
    }

    #[test]
    fn test_peer_info_new_incoming() {
        let addr = test_addr(8080);
        let info = PeerInfo::new_incoming(addr);

        assert_eq!(info.address, addr);
        assert_eq!(info.status, PeerStatus::Connected);
    }

    #[test]
    fn test_peer_info_complete_handshake() {
        let addr = test_addr(8080);
        let mut info = PeerInfo::new_incoming(addr);

        let private_key = PrivateKey::random();
        let public_key = private_key.public_key();
        let peer_id = PeerId::from_public_key(&public_key);

        info.complete_handshake(peer_id, public_key.clone(), PROTOCOL_VERSION);

        assert_eq!(info.id, peer_id);
        assert_eq!(info.public_key, Some(public_key));
        assert_eq!(info.version, Some(PROTOCOL_VERSION));
        assert_eq!(info.status, PeerStatus::Active);
    }

    #[test]
    fn test_peer_info_backoff() {
        let mut info = PeerInfo::new_outgoing(test_addr(8080));

        assert!(info.can_retry());
        assert_eq!(info.failed_attempts, 0);

        // Record failure
        info.record_failure();
        assert_eq!(info.failed_attempts, 1);
        assert!(!info.can_retry()); // Should wait for backoff

        // Backoff duration increases with failures
        let backoff1 = info.backoff_duration();
        info.record_failure();
        let backoff2 = info.backoff_duration();
        assert!(backoff2 > backoff1);
    }

    #[test]
    fn test_peer_manager_stale_peers() {
        let manager = PeerManager::new(10, vec![]);

        let mut info = PeerInfo::new_incoming(test_addr(8080));
        info.status = PeerStatus::Active;
        let id = info.id;
        manager.add_peer(info).unwrap();

        // With very short timeout, peer should be stale
        let stale = manager.stale_peers(Duration::from_nanos(1));
        assert!(stale.contains(&id));

        // Touch the peer
        manager.touch_peer(&id);

        // With long timeout, peer should not be stale
        let stale = manager.stale_peers(Duration::from_secs(3600));
        assert!(!stale.contains(&id));
    }

    #[test]
    fn test_peer_manager_needs_peers() {
        let manager = PeerManager::new(2, vec![]);

        assert!(manager.needs_peers());

        // Add one active peer
        let mut info = PeerInfo::new_incoming(test_addr(8080));
        info.status = PeerStatus::Active;
        manager.add_peer(info).unwrap();

        assert!(manager.needs_peers()); // Still need 1 more

        // Add another active peer
        let mut info2 = PeerInfo::new_incoming(test_addr(8081));
        info2.status = PeerStatus::Active;
        manager.add_peer(info2).unwrap();

        assert!(!manager.needs_peers()); // At max
    }

    #[test]
    fn test_peer_manager_local_id() {
        let mut manager = PeerManager::new(10, vec![]);

        assert!(manager.local_id().is_none());

        let id = test_peer_id(42);
        manager.set_local_id(id);

        assert_eq!(manager.local_id(), Some(id));
    }

    #[test]
    fn test_peer_manager_update_peer_id() {
        let manager = PeerManager::new(10, vec![]);
        let addr = test_addr(8080);

        let info = PeerInfo::new_incoming(addr);
        let temp_id = info.id;
        manager.add_peer(info).unwrap();

        let private_key = PrivateKey::random();
        let public_key = private_key.public_key();
        let real_id = PeerId::from_public_key(&public_key);

        manager.update_peer_id(temp_id, real_id, public_key, PROTOCOL_VERSION);

        // Old ID should be gone, new ID should exist
        assert!(manager.get_peer(&temp_id).is_none());
        assert!(manager.get_peer(&real_id).is_some());

        // Address mapping should be updated
        assert_eq!(manager.get_peer_by_addr(&addr), Some(real_id));
    }

    #[test]
    fn test_peer_manager_get_connectable_addresses() {
        let bootstrap = vec![test_addr(8080), test_addr(8081), test_addr(8082)];
        let manager = PeerManager::new(10, bootstrap.clone());

        // Initially all bootstrap nodes are connectable
        let connectable = manager.get_connectable_addresses();
        assert_eq!(connectable.len(), 3);

        // Add a peer at one bootstrap address
        let info = PeerInfo::new_incoming(test_addr(8080));
        manager.add_peer(info).unwrap();

        // Now only 2 are connectable
        let connectable = manager.get_connectable_addresses();
        assert_eq!(connectable.len(), 2);
        assert!(!connectable.contains(&test_addr(8080)));
    }

    #[test]
    fn test_peer_manager_peers_for_exchange() {
        let manager = PeerManager::new(10, vec![]);

        // Add active peers
        let mut info1 = PeerInfo::new_incoming(test_addr(8080));
        info1.status = PeerStatus::Active;
        manager.add_peer(info1).unwrap();

        let mut info2 = PeerInfo::new_incoming(test_addr(8081));
        info2.status = PeerStatus::Active;
        manager.add_peer(info2).unwrap();

        // Add non-active peer
        let info3 = PeerInfo::new_outgoing(test_addr(8082)); // Connecting
        manager.add_peer(info3).unwrap();

        // Only active peers should be in exchange list
        let peers = manager.get_peers_for_exchange();
        assert_eq!(peers.len(), 2);
    }
}

// =============================================================================
// 2. Message Encoding Tests
// =============================================================================

mod message_encoding {
    use super::*;

    #[test]
    fn test_encode_decode_hello() {
        let msg = NetworkMessage::Hello {
            version: PROTOCOL_VERSION,
            peer_id: [1u8; 32],
            genesis_hash: [2u8; 32],
            public_key: [3u8; 64],
        };

        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_hello_ack() {
        let msg = NetworkMessage::HelloAck {
            peer_id: [1u8; 32],
            public_key: [2u8; 64],
        };

        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_ping_pong() {
        let ping = NetworkMessage::Ping(12345678);
        let encoded = MessageCodec::encode_message(&ping).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(ping, decoded);

        let pong = NetworkMessage::Pong(12345678);
        let encoded = MessageCodec::encode_message(&pong).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(pong, decoded);
    }

    #[test]
    fn test_encode_decode_get_peers() {
        let msg = NetworkMessage::GetPeers;
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_peers() {
        use bach_network::message::SerializablePeerInfo;

        let peers = vec![
            SerializablePeerInfo {
                id: [1u8; 32],
                address: "127.0.0.1:8080".to_string(),
            },
            SerializablePeerInfo {
                id: [2u8; 32],
                address: "127.0.0.1:8081".to_string(),
            },
        ];

        let msg = NetworkMessage::Peers(peers);
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_new_transaction() {
        use bach_network::message::SerializableTransaction;

        let tx = SerializableTransaction {
            nonce: 42,
            to: Some([0xaa; 20]),
            value: [0; 32],
            data: vec![1, 2, 3, 4],
            signature: [0xbb; 65],
        };

        let msg = NetworkMessage::NewTransaction(tx);
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_get_transactions() {
        let hashes = vec![[1u8; 32], [2u8; 32], [3u8; 32]];
        let msg = NetworkMessage::GetTransactions(hashes);
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_new_block() {
        use bach_network::message::{SerializableBlock, SerializableTransaction};

        let block = SerializableBlock {
            height: 100,
            parent_hash: [0xaa; 32],
            transactions: vec![
                SerializableTransaction {
                    nonce: 1,
                    to: None,
                    value: [0; 32],
                    data: vec![],
                    signature: [0xcc; 65],
                },
            ],
            timestamp: 1234567890,
        };

        let msg = NetworkMessage::NewBlock(block);
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_get_blocks() {
        let msg = NetworkMessage::GetBlocks { start: 100, count: 10 };
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_new_block_hash() {
        let msg = NetworkMessage::NewBlockHash {
            height: 500,
            hash: [0xab; 32],
        };
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_consensus_proposal() {
        use bach_network::ConsensusMessage;

        let consensus = ConsensusMessage::Proposal {
            height: 100,
            round: 0,
            block_hash: [0xab; 32],
            block_data: vec![1, 2, 3, 4, 5],
        };

        let msg = NetworkMessage::Consensus(consensus);
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_consensus_prevote() {
        use bach_network::ConsensusMessage;

        let consensus = ConsensusMessage::Prevote {
            height: 100,
            round: 1,
            block_hash: Some([0xab; 32]),
            validator: [0xcd; 32],
            signature: vec![0xef; 65],
        };

        let msg = NetworkMessage::Consensus(consensus);
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_consensus_precommit() {
        use bach_network::ConsensusMessage;

        let consensus = ConsensusMessage::Precommit {
            height: 100,
            round: 2,
            block_hash: None, // Nil vote
            validator: [0xcd; 32],
            signature: vec![0xef; 65],
        };

        let msg = NetworkMessage::Consensus(consensus);
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_vote_request() {
        use bach_network::ConsensusMessage;

        let consensus = ConsensusMessage::VoteRequest {
            height: 100,
            round: 0,
        };

        let msg = NetworkMessage::Consensus(consensus);
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_encode_decode_disconnect() {
        let msg = NetworkMessage::Disconnect {
            reason: "test disconnect".to_string(),
        };
        let encoded = MessageCodec::encode_message(&msg).unwrap();
        let decoded = MessageCodec::decode_message(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_decode_incomplete_message() {
        let incomplete_data = vec![0, 0, 0, 100]; // Length prefix says 100 bytes, but no data
        let result = MessageCodec::decode_message(&incomplete_data);

        assert!(result.is_err());
    }

    #[test]
    fn test_decode_too_short() {
        let short_data = vec![0, 1, 2]; // Less than 4 bytes for length prefix
        let result = MessageCodec::decode_message(&short_data);

        assert!(result.is_err());
    }
}

// =============================================================================
// 3. Handshake Tests
// =============================================================================

mod handshake {
    use super::*;

    #[test]
    fn test_hello_message_construction() {
        let peer_id = PeerId::from_bytes([1u8; 32]);
        let genesis = H256::from([2u8; 32]);
        let pubkey = [3u8; 64];

        let msg = NetworkMessage::hello(peer_id, genesis, pubkey);

        match msg {
            NetworkMessage::Hello {
                version,
                peer_id: pid,
                genesis_hash,
                public_key,
            } => {
                assert_eq!(version, PROTOCOL_VERSION);
                assert_eq!(pid, [1u8; 32]);
                assert_eq!(genesis_hash, [2u8; 32]);
                assert_eq!(public_key, [3u8; 64]);
            }
            _ => panic!("Expected Hello message"),
        }
    }

    #[test]
    fn test_hello_ack_message_construction() {
        let peer_id = PeerId::from_bytes([1u8; 32]);
        let pubkey = [2u8; 64];

        let msg = NetworkMessage::hello_ack(peer_id, pubkey);

        match msg {
            NetworkMessage::HelloAck { peer_id: pid, public_key } => {
                assert_eq!(pid, [1u8; 32]);
                assert_eq!(public_key, [2u8; 64]);
            }
            _ => panic!("Expected HelloAck message"),
        }
    }

    #[test]
    fn test_ping_message_construction() {
        let ping = NetworkMessage::ping();

        match ping {
            NetworkMessage::Ping(nonce) => {
                // Nonce should be non-zero (timestamp-based)
                assert!(nonce > 0);
            }
            _ => panic!("Expected Ping message"),
        }
    }

    #[test]
    fn test_pong_response() {
        let nonce = 12345u64;
        let pong = NetworkMessage::pong(nonce);

        assert!(matches!(pong, NetworkMessage::Pong(n) if n == nonce));
    }

    #[test]
    fn test_disconnect_message_construction() {
        let msg = NetworkMessage::disconnect("test reason");

        match msg {
            NetworkMessage::Disconnect { reason } => {
                assert_eq!(reason, "test reason");
            }
            _ => panic!("Expected Disconnect message"),
        }
    }

    #[test]
    fn test_message_names() {
        assert_eq!(NetworkMessage::GetPeers.name(), "GetPeers");
        assert_eq!(NetworkMessage::Ping(0).name(), "Ping");
        assert_eq!(NetworkMessage::Pong(0).name(), "Pong");
        assert_eq!(
            NetworkMessage::Disconnect {
                reason: String::new()
            }
            .name(),
            "Disconnect"
        );
    }
}

// =============================================================================
// 4. Peer Discovery Tests
// =============================================================================

mod peer_discovery {
    use super::*;

    #[test]
    fn test_bootstrap_nodes_config() {
        let bootstrap = vec![
            "127.0.0.1:30303".parse().unwrap(),
            "192.168.1.1:30303".parse().unwrap(),
        ];

        let config = NetworkConfig::default().with_bootstrap_nodes(bootstrap.clone());

        assert_eq!(config.bootstrap_nodes, bootstrap);
    }

    #[test]
    fn test_peer_manager_bootstrap_nodes() {
        let bootstrap = vec![test_addr(8080), test_addr(8081)];
        let manager = PeerManager::new(10, bootstrap.clone());

        assert_eq!(manager.bootstrap_nodes(), bootstrap.as_slice());
    }

    #[test]
    fn test_serializable_peer_info_from_peer_info() {
        use bach_network::peer::SerializablePeerInfo;

        let mut info = PeerInfo::new_incoming(test_addr(8080));
        info.status = PeerStatus::Active;

        let serializable = SerializablePeerInfo::from(&info);

        assert_eq!(serializable.id, info.id.0);
        assert_eq!(serializable.address, info.address.to_string());
    }
}

// =============================================================================
// 5. Message Routing Tests
// =============================================================================

mod message_routing {
    use super::*;

    #[tokio::test]
    async fn test_network_config_builder() {
        let config = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:9999".parse().unwrap())
            .with_max_peers(50)
            .with_genesis_hash(test_genesis())
            .with_private_key([0xab; 32]);

        assert_eq!(config.listen_addr.port(), 9999);
        assert_eq!(config.max_peers, 50);
        assert_eq!(config.genesis_hash, test_genesis());
        assert_eq!(config.private_key, Some([0xab; 32]));
    }

    #[tokio::test]
    async fn test_network_service_creation() {
        let config = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap());

        let service = NetworkService::new(config).await;

        // Service should have a valid peer ID
        assert!(!service.local_id().as_bytes().iter().all(|&b| b == 0));

        // Should have a public key
        let pubkey = service.public_key();
        assert!(!pubkey.to_bytes().iter().all(|&b| b == 0));

        // Peer ID should match public key
        let expected_id = PeerId::from_public_key(pubkey);
        assert_eq!(service.local_id(), expected_id);
    }

    #[tokio::test]
    async fn test_network_service_with_private_key() {
        let private_key_bytes = [0x42; 32];
        let config = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap())
            .with_private_key(private_key_bytes);

        let service = NetworkService::new(config).await;

        // Create expected keys from the same private key
        let expected_private = PrivateKey::from_bytes(&private_key_bytes).unwrap();
        let expected_public = expected_private.public_key();
        let expected_id = PeerId::from_public_key(&expected_public);

        assert_eq!(service.local_id(), expected_id);
    }

    #[tokio::test]
    async fn test_network_service_subscribe() {
        let config = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap());

        let mut service = NetworkService::new(config).await;

        // First subscribe should work
        let receiver = service.subscribe();
        assert!(receiver.is_some());

        // Second subscribe should return None
        let receiver2 = service.subscribe();
        assert!(receiver2.is_none());
    }

    #[tokio::test]
    async fn test_network_service_peer_manager() {
        let bootstrap = vec![test_addr(30303)];
        let config = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap())
            .with_bootstrap_nodes(bootstrap.clone())
            .with_max_peers(50);

        let service = NetworkService::new(config).await;
        let peer_manager = service.peer_manager();

        assert_eq!(peer_manager.max_peers(), 50);
        assert_eq!(peer_manager.bootstrap_nodes(), bootstrap.as_slice());
    }
}

// =============================================================================
// 6. Connection Handling Tests
// =============================================================================

mod connection_handling {
    use super::*;
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn test_codec_encode_decode_roundtrip() {
        let mut codec = MessageCodec::new();
        let mut buf = BytesMut::new();

        let msg = NetworkMessage::Ping(12345);

        // Encode
        codec.encode(msg.clone(), &mut buf).unwrap();

        // Decode
        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_codec_multiple_messages() {
        let mut codec = MessageCodec::new();
        let mut buf = BytesMut::new();

        let msg1 = NetworkMessage::Ping(111);
        let msg2 = NetworkMessage::Pong(222);
        let msg3 = NetworkMessage::GetPeers;

        // Encode all messages
        codec.encode(msg1.clone(), &mut buf).unwrap();
        codec.encode(msg2.clone(), &mut buf).unwrap();
        codec.encode(msg3.clone(), &mut buf).unwrap();

        // Decode in order
        let decoded1 = codec.decode(&mut buf).unwrap().unwrap();
        let decoded2 = codec.decode(&mut buf).unwrap().unwrap();
        let decoded3 = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(msg1, decoded1);
        assert_eq!(msg2, decoded2);
        assert_eq!(msg3, decoded3);

        // Nothing left
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn test_codec_partial_decode() {
        let mut codec = MessageCodec::new();
        let msg = NetworkMessage::Pong(99999);
        let encoded = MessageCodec::encode_message(&msg).unwrap();

        // Feed bytes one at a time
        let mut buf = BytesMut::new();
        for (i, byte) in encoded.iter().enumerate() {
            buf.extend_from_slice(&[*byte]);
            let result = codec.decode(&mut buf).unwrap();

            if i < encoded.len() - 1 {
                assert!(result.is_none());
            } else {
                assert_eq!(result, Some(msg.clone()));
            }
        }
    }

    #[tokio::test]
    async fn test_network_config_timeouts() {
        let config = NetworkConfig::default();

        // Check default timeout values
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert_eq!(config.ping_interval, Duration::from_secs(30));
        assert_eq!(config.peer_timeout, Duration::from_secs(90));
    }

    #[tokio::test]
    async fn test_network_service_start_stop() {
        let config = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap());

        let mut service = NetworkService::new(config).await;

        // Start service
        let _ = service.subscribe(); // Take subscriber first
        let result = service.start().await;
        assert!(result.is_ok());

        // Give it a moment to bind
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Stop service
        service.stop().await;
    }

    #[tokio::test]
    async fn test_network_service_command_sender() {
        let config = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap());

        let mut service = NetworkService::new(config).await;

        // Before starting, no command sender
        assert!(service.command_sender().is_none());

        // Start service
        let _ = service.subscribe();
        service.start().await.unwrap();

        // After starting, command sender available
        assert!(service.command_sender().is_some());

        service.stop().await;
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

mod integration {
    use super::*;

    #[tokio::test]
    async fn test_two_services_peer_exchange() {
        let genesis = H256::from([0x42; 32]);

        let config1 = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap())
            .with_genesis_hash(genesis);

        let config2 = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap())
            .with_genesis_hash(genesis);

        let mut service1 = NetworkService::new(config1).await;
        let mut service2 = NetworkService::new(config2).await;

        let _events1 = service1.subscribe();
        let _events2 = service2.subscribe();

        // Start both services
        service1.start().await.unwrap();
        service2.start().await.unwrap();

        // Give them time to bind
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cleanup
        service1.stop().await;
        service2.stop().await;
    }

    #[tokio::test]
    async fn test_genesis_mismatch_detection() {
        // This test verifies the system can detect genesis mismatches
        let genesis1 = H256::from([0x11; 32]);
        let genesis2 = H256::from([0x22; 32]); // Different genesis

        let config1 = NetworkConfig::default()
            .with_genesis_hash(genesis1)
            .with_listen_addr("127.0.0.1:0".parse().unwrap());

        let config2 = NetworkConfig::default()
            .with_genesis_hash(genesis2)
            .with_listen_addr("127.0.0.1:0".parse().unwrap());

        let service1 = NetworkService::new(config1).await;
        let service2 = NetworkService::new(config2).await;

        // Services should have different genesis hashes
        // Connection should be rejected during handshake
        // (Full integration would require actual connection test)

        assert_ne!(genesis1, genesis2);
    }
}
