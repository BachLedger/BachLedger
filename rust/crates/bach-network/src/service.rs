//! Network service

use crate::error::{NetworkError, NetworkResult};
use crate::peer::{
    read_message, write_message, PeerConnection, PeerInfo, PeerManager, PeerState,
};
use crate::types::{Handshake, Message, MessageType, PeerId};
use bach_primitives::H256;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Network service configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_addr: SocketAddr,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<SocketAddr>,
    /// Maximum peers
    pub max_peers: usize,
    /// Protocol version
    pub protocol_version: u32,
    /// Chain ID
    pub chain_id: u64,
    /// Genesis hash
    pub genesis_hash: H256,
    /// Our peer ID
    pub peer_id: PeerId,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:30303".parse().unwrap(),
            bootstrap_peers: Vec::new(),
            max_peers: 50,
            protocol_version: 1,
            chain_id: 1,
            genesis_hash: H256::ZERO,
            peer_id: PeerId::random(),
        }
    }
}

/// Event from the network
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Peer connected
    PeerConnected(PeerId),
    /// Peer disconnected
    PeerDisconnected(PeerId),
    /// Message received
    Message {
        /// Source peer
        peer_id: PeerId,
        /// Message
        message: Message,
    },
}

/// Network service handle
pub struct NetworkService {
    /// Configuration
    config: NetworkConfig,
    /// Peer manager
    peers: Arc<PeerManager>,
    /// Current block height
    height: Arc<RwLock<u64>>,
    /// Event sender
    event_tx: mpsc::Sender<NetworkEvent>,
    /// Event receiver
    event_rx: Option<mpsc::Receiver<NetworkEvent>>,
    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl NetworkService {
    /// Create a new network service
    pub fn new(config: NetworkConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1024);
        Self {
            config: config.clone(),
            peers: Arc::new(PeerManager::new(config.max_peers)),
            height: Arc::new(RwLock::new(0)),
            event_tx,
            event_rx: Some(event_rx),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Take event receiver
    pub fn take_events(&mut self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.event_rx.take()
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.peers.peer_count()
    }

    /// Get connected peer IDs
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.peers.peer_ids()
    }

    /// Get peer info
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.get_peer(peer_id)
    }

    /// Update our block height
    pub fn set_height(&self, height: u64) {
        *self.height.write() = height;
    }

    /// Get our block height
    pub fn height(&self) -> u64 {
        *self.height.read()
    }

    /// Get our peer ID
    pub fn peer_id(&self) -> PeerId {
        self.config.peer_id
    }

    /// Broadcast a message to all peers
    pub async fn broadcast(&self, msg: Message) {
        let peer_ids = self.peers.peer_ids();
        for peer_id in peer_ids {
            if let Some(conn) = self.peers.get_connection(&peer_id) {
                if let Err(e) = conn.send(msg.clone()).await {
                    warn!("Failed to send to peer {}: {}", peer_id, e);
                }
            }
        }
    }

    /// Send a message to a specific peer
    pub async fn send_to(&self, peer_id: &PeerId, msg: Message) -> NetworkResult<()> {
        let conn = self
            .peers
            .get_connection(peer_id)
            .ok_or_else(|| NetworkError::PeerNotFound(peer_id.to_string()))?;
        conn.send(msg).await
    }

    /// Connect to a peer
    pub async fn connect(&self, addr: SocketAddr) -> NetworkResult<PeerId> {
        if !self.peers.can_accept() {
            return Err(NetworkError::ConnectionFailed("max peers reached".into()));
        }

        info!("Connecting to {}", addr);

        let stream = TcpStream::connect(addr).await?;
        let peer_id = self
            .handle_connection(stream, addr, false)
            .await?;

        Ok(peer_id)
    }

    /// Start listening for connections
    pub async fn start(&self) -> NetworkResult<()> {
        if *self.running.read() {
            return Err(NetworkError::AlreadyRunning);
        }
        *self.running.write() = true;

        let listener = TcpListener::bind(self.config.listen_addr).await?;
        info!("Listening on {}", self.config.listen_addr);

        // Connect to bootstrap peers
        for addr in &self.config.bootstrap_peers {
            let addr = *addr;
            let service = self.clone_handle();
            tokio::spawn(async move {
                if let Err(e) = service.connect(addr).await {
                    warn!("Failed to connect to bootstrap peer {}: {}", addr, e);
                }
            });
        }

        // Accept incoming connections
        let running = self.running.clone();
        let service = self.clone_handle();
        tokio::spawn(async move {
            while *running.read() {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("Incoming connection from {}", addr);
                        let service = service.clone_handle();
                        tokio::spawn(async move {
                            if let Err(e) = service.handle_connection(stream, addr, true).await {
                                warn!("Connection error from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Accept error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the service
    pub fn stop(&self) {
        *self.running.write() = false;
    }

    /// Handle a new connection
    async fn handle_connection(
        &self,
        mut stream: TcpStream,
        addr: SocketAddr,
        inbound: bool,
    ) -> NetworkResult<PeerId> {
        // Perform handshake
        let our_handshake = Handshake::new(
            self.config.protocol_version,
            self.config.chain_id,
            self.config.genesis_hash,
            self.height(),
            self.config.peer_id,
        );

        // Send our handshake
        let handshake_bytes = serde_json::to_vec(&our_handshake)
            .map_err(|e| NetworkError::Protocol(e.to_string()))?;
        let msg = Message::new(MessageType::Handshake, handshake_bytes.into());
        write_message(&mut stream, &msg).await?;

        // Receive their handshake
        let response = read_message(&mut stream).await?;
        if response.msg_type != MessageType::Handshake {
            return Err(NetworkError::Protocol("expected handshake".into()));
        }

        let their_handshake: Handshake = serde_json::from_slice(&response.payload)
            .map_err(|e| NetworkError::Protocol(e.to_string()))?;

        // Validate handshake
        if their_handshake.chain_id != self.config.chain_id {
            return Err(NetworkError::Protocol("chain ID mismatch".into()));
        }
        if their_handshake.genesis_hash != self.config.genesis_hash {
            return Err(NetworkError::Protocol("genesis hash mismatch".into()));
        }

        let peer_id = their_handshake.peer_id;

        // Set up message channels
        let (tx, mut rx) = mpsc::channel::<Message>(256);

        // Create peer info and connection
        let mut info = PeerInfo::new(peer_id, addr, inbound);
        info.state = PeerState::Connected;
        info.set_height(their_handshake.height);

        let conn = Arc::new(PeerConnection {
            peer_id,
            addr,
            sender: tx,
        });

        self.peers.add_peer(info, conn)?;

        // Send peer connected event
        let _ = self
            .event_tx
            .send(NetworkEvent::PeerConnected(peer_id))
            .await;

        info!("Connected to peer {} at {}", peer_id, addr);

        // Spawn writer task
        let (read_half, mut write_half) = stream.into_split();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let data = msg.encode();
                if tokio::io::AsyncWriteExt::write_all(&mut write_half, &data)
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        // Spawn reader task
        let peers = self.peers.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        tokio::spawn(async move {
            let mut read_half = tokio::io::BufReader::new(read_half);
            let mut len_buf = [0u8; 4];

            while *running.read() {
                // Read length prefix
                if tokio::io::AsyncReadExt::read_exact(&mut read_half, &mut len_buf)
                    .await
                    .is_err()
                {
                    break;
                }

                let len = u32::from_be_bytes(len_buf) as usize;
                if len == 0 || len > 16 * 1024 * 1024 {
                    break;
                }

                // Read message body
                let mut buf = bytes::BytesMut::with_capacity(4 + len);
                buf.extend_from_slice(&len_buf);
                buf.resize(4 + len, 0);

                if tokio::io::AsyncReadExt::read_exact(&mut read_half, &mut buf[4..])
                    .await
                    .is_err()
                {
                    break;
                }

                // Decode and dispatch
                if let Some(msg) = Message::decode(buf.freeze()) {
                    peers.touch_peer(&peer_id);

                    match msg.msg_type {
                        MessageType::Ping => {
                            // Respond with pong
                            if let Some(conn) = peers.get_connection(&peer_id) {
                                let _ = conn.send(Message::pong()).await;
                            }
                        }
                        MessageType::Pong => {
                            // Ignore
                        }
                        MessageType::Disconnect => {
                            break;
                        }
                        _ => {
                            // Forward to event handler
                            let _ = event_tx
                                .send(NetworkEvent::Message {
                                    peer_id,
                                    message: msg,
                                })
                                .await;
                        }
                    }
                }
            }

            // Peer disconnected
            peers.remove_peer(&peer_id);
            let _ = event_tx.send(NetworkEvent::PeerDisconnected(peer_id)).await;
            debug!("Peer {} disconnected", peer_id);
        });

        Ok(peer_id)
    }

    /// Clone handle for spawning
    fn clone_handle(&self) -> Self {
        Self {
            config: self.config.clone(),
            peers: self.peers.clone(),
            height: self.height.clone(),
            event_tx: self.event_tx.clone(),
            event_rx: None,
            running: self.running.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicU16, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Use a base port that depends on current time to avoid conflicts across test runs
    fn base_port() -> u16 {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Use a port in range 40000-60000
        40000 + ((secs % 20000) as u16)
    }

    static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

    fn next_port() -> u16 {
        base_port() + PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    fn test_config(port: u16) -> NetworkConfig {
        NetworkConfig {
            listen_addr: format!("127.0.0.1:{}", port).parse().unwrap(),
            ..Default::default()
        }
    }

    #[test]
    fn test_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.protocol_version, 1);
        assert_eq!(config.max_peers, 50);
        assert!(config.bootstrap_peers.is_empty());
    }

    #[test]
    fn test_service_creation() {
        let config = test_config(30000);
        let mut service = NetworkService::new(config);

        assert!(!service.is_running());
        assert_eq!(service.peer_count(), 0);
        assert!(service.take_events().is_some());
    }

    #[test]
    fn test_service_height() {
        let config = test_config(30001);
        let service = NetworkService::new(config);

        assert_eq!(service.height(), 0);
        service.set_height(100);
        assert_eq!(service.height(), 100);
    }

    #[test]
    fn test_service_peer_id() {
        let config = test_config(30002);
        let service = NetworkService::new(config.clone());
        assert_eq!(service.peer_id(), config.peer_id);
    }

    #[tokio::test]
    async fn test_service_start_stop() {
        let config = test_config(next_port());
        let service = NetworkService::new(config);

        service.start().await.unwrap();
        assert!(service.is_running());

        service.stop();
        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_service_double_start() {
        let config = test_config(next_port());
        let service = NetworkService::new(config);

        service.start().await.unwrap();
        let result = service.start().await;
        assert!(matches!(result, Err(NetworkError::AlreadyRunning)));

        service.stop();
    }

    #[tokio::test]
    async fn test_peer_to_peer_connection() {
        // This test creates two services and connects them
        let port1 = next_port();
        let port2 = next_port();
        let config1 = NetworkConfig {
            listen_addr: format!("127.0.0.1:{}", port1).parse().unwrap(),
            genesis_hash: H256::from_bytes([1; 32]),
            ..Default::default()
        };
        let config2 = NetworkConfig {
            listen_addr: format!("127.0.0.1:{}", port2).parse().unwrap(),
            genesis_hash: H256::from_bytes([1; 32]),
            ..Default::default()
        };

        let mut service1 = NetworkService::new(config1);
        let mut service2 = NetworkService::new(config2.clone());

        let mut events1 = service1.take_events().unwrap();
        let mut events2 = service2.take_events().unwrap();

        // Start both services
        service1.start().await.unwrap();
        service2.start().await.unwrap();

        // Give time to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Connect service1 to service2
        let _peer_id = service1.connect(config2.listen_addr).await.unwrap();

        // Wait for events
        let event1 = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            events1.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        assert!(matches!(event1, NetworkEvent::PeerConnected(_)));

        // Service2 should also get connected event
        let event2 = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            events2.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        assert!(matches!(event2, NetworkEvent::PeerConnected(_)));

        // Both should have 1 peer
        assert_eq!(service1.peer_count(), 1);
        assert_eq!(service2.peer_count(), 1);

        service1.stop();
        service2.stop();
    }
}
