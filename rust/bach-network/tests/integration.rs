//! Integration tests for bach-network

use bach_network::{
    MessageCodec, NetworkConfig, NetworkMessage, NetworkService, PeerId, PeerInfo,
    PeerManager, PROTOCOL_VERSION,
};
use bach_primitives::H256;
use std::net::SocketAddr;
use std::time::Duration;

/// Test message codec roundtrip with various message types.
#[test]
fn test_codec_all_message_types() {
    let messages = vec![
        NetworkMessage::GetPeers,
        NetworkMessage::Peers(vec![]),
        NetworkMessage::Ping(12345),
        NetworkMessage::Pong(12345),
        NetworkMessage::GetBlocks { start: 0, count: 10 },
        NetworkMessage::Blocks(vec![]),
        NetworkMessage::NewBlockHash {
            height: 100,
            hash: [0xab; 32],
        },
        NetworkMessage::disconnect("test reason"),
    ];

    for msg in messages {
        let encoded = MessageCodec::encode_message(&msg).expect("encode failed");
        let decoded = MessageCodec::decode_message(&encoded).expect("decode failed");
        assert_eq!(msg, decoded, "roundtrip failed for {:?}", msg.name());
    }
}

/// Test peer manager operations.
#[test]
fn test_peer_manager_operations() {
    let bootstrap: Vec<SocketAddr> = vec![
        "127.0.0.1:30303".parse().unwrap(),
        "127.0.0.1:30304".parse().unwrap(),
    ];
    let manager = PeerManager::new(10, bootstrap.clone());

    // Check bootstrap nodes
    assert_eq!(manager.bootstrap_nodes().len(), 2);

    // Add some peers
    let addr1: SocketAddr = "192.168.1.1:8080".parse().unwrap();
    let addr2: SocketAddr = "192.168.1.2:8080".parse().unwrap();

    let info1 = PeerInfo::new_incoming(addr1);
    let info2 = PeerInfo::new_incoming(addr2);

    let id1 = info1.id;
    let id2 = info2.id;

    manager.add_peer(info1).unwrap();
    manager.add_peer(info2).unwrap();

    // Check retrieval
    assert!(manager.get_peer(&id1).is_some());
    assert!(manager.get_peer(&id2).is_some());
    assert_eq!(manager.get_peer_by_addr(&addr1), Some(id1));

    // Remove peer
    manager.remove_peer(&id1);
    assert!(manager.get_peer(&id1).is_none());
    assert!(manager.get_peer_by_addr(&addr1).is_none());
}

/// Test peer info status transitions.
#[test]
fn test_peer_status_transitions() {
    use bach_network::PeerStatus;

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

    // Outgoing connection starts as Connecting
    let outgoing = PeerInfo::new_outgoing(addr);
    assert_eq!(outgoing.status, PeerStatus::Connecting);

    // Incoming connection starts as Connected
    let incoming = PeerInfo::new_incoming(addr);
    assert_eq!(incoming.status, PeerStatus::Connected);
}

/// Test peer backoff calculation.
#[test]
fn test_peer_backoff() {
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut info = PeerInfo::new_outgoing(addr);

    // Initial state - can retry
    assert!(info.can_retry());

    // After failure - exponential backoff
    info.record_failure();
    let backoff1 = info.backoff_duration();

    info.record_failure();
    let backoff2 = info.backoff_duration();

    info.record_failure();
    let backoff3 = info.backoff_duration();

    // Backoff should increase exponentially
    assert!(backoff2 > backoff1);
    assert!(backoff3 > backoff2);
}

/// Test Hello message construction.
#[test]
fn test_hello_message() {
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
            assert_eq!(public_key, pubkey);
        }
        _ => panic!("expected Hello message"),
    }
}

/// Test consensus message encoding.
#[test]
fn test_consensus_message() {
    use bach_network::ConsensusMessage;

    let proposal = ConsensusMessage::Proposal {
        height: 100,
        round: 0,
        block_hash: [0xaa; 32],
        block_data: vec![1, 2, 3, 4],
    };

    let msg = NetworkMessage::Consensus(proposal.clone());
    let encoded = MessageCodec::encode_message(&msg).unwrap();
    let decoded = MessageCodec::decode_message(&encoded).unwrap();

    match decoded {
        NetworkMessage::Consensus(ConsensusMessage::Proposal {
            height,
            round,
            block_hash,
            block_data,
        }) => {
            assert_eq!(height, 100);
            assert_eq!(round, 0);
            assert_eq!(block_hash, [0xaa; 32]);
            assert_eq!(block_data, vec![1, 2, 3, 4]);
        }
        _ => panic!("expected Consensus Proposal"),
    }
}

/// Test service can be created with custom config.
#[tokio::test]
async fn test_service_with_custom_config() {
    let config = NetworkConfig::default()
        .with_listen_addr("127.0.0.1:0".parse().unwrap())
        .with_max_peers(50)
        .with_genesis_hash(H256::from([0xab; 32]));

    let service = NetworkService::new(config).await;

    // Service should have valid local ID
    let id = service.local_id();
    assert!(!id.as_bytes().iter().all(|&b| b == 0));
}

/// Test that two services can start without conflict.
#[tokio::test]
async fn test_two_services_start() {
    let config1 = NetworkConfig::default()
        .with_listen_addr("127.0.0.1:0".parse().unwrap())
        .with_genesis_hash(H256::from([1u8; 32]));

    let config2 = NetworkConfig::default()
        .with_listen_addr("127.0.0.1:0".parse().unwrap())
        .with_genesis_hash(H256::from([1u8; 32]));

    let mut service1 = NetworkService::new(config1).await;
    let mut service2 = NetworkService::new(config2).await;

    // Both should start successfully
    service1.start().await.unwrap();
    service2.start().await.unwrap();

    // Give them time to initialize
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Clean shutdown
    service1.stop().await;
    service2.stop().await;
}

/// Test broadcast doesn't panic when no peers connected.
#[tokio::test]
async fn test_broadcast_no_peers() {
    let config = NetworkConfig::default()
        .with_listen_addr("127.0.0.1:0".parse().unwrap());

    let mut service = NetworkService::new(config).await;
    service.start().await.unwrap();

    // Broadcast should succeed even with no peers
    let result = service.broadcast(NetworkMessage::GetPeers).await;
    assert!(result.is_ok());

    service.stop().await;
}

/// Test peer ID derivation from public key is deterministic.
#[test]
fn test_peer_id_deterministic() {
    use bach_crypto::PrivateKey;

    let key = PrivateKey::random();
    let pubkey = key.public_key();

    let id1 = PeerId::from_public_key(&pubkey);
    let id2 = PeerId::from_public_key(&pubkey);

    assert_eq!(id1, id2);
}

/// Test peer ID short hex representation.
#[test]
fn test_peer_id_short_hex() {
    let bytes = [0xab, 0xcd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xef, 0x12];
    let id = PeerId::from_bytes(bytes);
    let short = id.short_hex();

    assert!(short.contains("abcd"));
    assert!(short.contains("ef12"));
}
