//! Network service for managing P2P connections

use bach_crypto::{PrivateKey, PublicKey};
use bach_primitives::H256;
use futures::stream::StreamExt;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error, info, warn};

use crate::codec::MessageCodec;
use crate::error::{NetworkError, NetworkResult};
use crate::message::{NetworkMessage, PROTOCOL_VERSION};
use crate::peer::{PeerId, PeerInfo, PeerManager};

/// Configuration for the network service.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Address to listen on
    pub listen_addr: SocketAddr,
    /// Maximum number of peers
    pub max_peers: usize,
    /// Bootstrap node addresses
    pub bootstrap_nodes: Vec<SocketAddr>,
    /// Genesis hash for chain identification
    pub genesis_hash: H256,
    /// Private key for signing (generates random if None)
    pub private_key: Option<[u8; 32]>,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Ping interval for liveness checks
    pub ping_interval: Duration,
    /// Peer timeout (disconnect if no activity)
    pub peer_timeout: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:30303".parse().unwrap(),
            max_peers: 25,
            bootstrap_nodes: Vec::new(),
            genesis_hash: H256::zero(),
            private_key: None,
            connection_timeout: Duration::from_secs(10),
            ping_interval: Duration::from_secs(30),
            peer_timeout: Duration::from_secs(90),
        }
    }
}

impl NetworkConfig {
    /// Sets the listen address.
    pub fn with_listen_addr(mut self, addr: SocketAddr) -> Self {
        self.listen_addr = addr;
        self
    }

    /// Sets the maximum peers.
    pub fn with_max_peers(mut self, max: usize) -> Self {
        self.max_peers = max;
        self
    }

    /// Adds bootstrap nodes.
    pub fn with_bootstrap_nodes(mut self, nodes: Vec<SocketAddr>) -> Self {
        self.bootstrap_nodes = nodes;
        self
    }

    /// Sets the genesis hash.
    pub fn with_genesis_hash(mut self, hash: H256) -> Self {
        self.genesis_hash = hash;
        self
    }

    /// Sets the private key.
    pub fn with_private_key(mut self, key: [u8; 32]) -> Self {
        self.private_key = Some(key);
        self
    }
}

/// Events emitted by the network service.
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// A new peer connected and completed handshake
    PeerConnected(PeerId),
    /// A peer disconnected
    PeerDisconnected(PeerId),
    /// Message received from a peer
    MessageReceived {
        from: PeerId,
        message: NetworkMessage,
    },
}

/// Commands to send to the network service.
#[derive(Debug)]
pub enum NetworkCommand {
    /// Send a message to a specific peer
    SendMessage {
        to: PeerId,
        message: NetworkMessage,
    },
    /// Broadcast a message to all connected peers
    Broadcast {
        message: NetworkMessage,
    },
    /// Connect to a new peer
    Connect(SocketAddr),
    /// Disconnect from a peer
    Disconnect(PeerId),
    /// Shutdown the service
    Shutdown,
}

/// Internal message for connection handling.
enum ConnectionEvent {
    NewConnection {
        stream: TcpStream,
        addr: SocketAddr,
        outgoing: bool,
    },
    MessageReceived {
        peer_id: PeerId,
        message: NetworkMessage,
    },
    ConnectionClosed {
        peer_id: PeerId,
        reason: String,
    },
    HandshakeComplete {
        temp_id: PeerId,
        real_id: PeerId,
        public_key: PublicKey,
        version: u32,
    },
}

/// Handle to send messages to specific peers.
struct PeerHandle {
    sender: mpsc::Sender<NetworkMessage>,
}

/// The main network service.
pub struct NetworkService {
    config: NetworkConfig,
    peer_manager: Arc<PeerManager>,
    #[allow(dead_code)] // Reserved for future message signing
    private_key: PrivateKey,
    public_key: PublicKey,
    local_id: PeerId,
    event_tx: mpsc::Sender<NetworkEvent>,
    event_rx: Option<mpsc::Receiver<NetworkEvent>>,
    command_tx: Option<mpsc::Sender<NetworkCommand>>,
    running: Arc<RwLock<bool>>,
}

impl NetworkService {
    /// Creates a new network service.
    pub async fn new(config: NetworkConfig) -> Self {
        let private_key = match config.private_key {
            Some(bytes) => PrivateKey::from_bytes(&bytes).expect("invalid private key"),
            None => PrivateKey::random(),
        };
        let public_key = private_key.public_key();
        let local_id = PeerId::from_public_key(&public_key);

        let mut peer_manager = PeerManager::new(config.max_peers, config.bootstrap_nodes.clone());
        peer_manager.set_local_id(local_id);

        let (event_tx, event_rx) = mpsc::channel(1024);

        Self {
            config,
            peer_manager: Arc::new(peer_manager),
            private_key,
            public_key,
            local_id,
            event_tx,
            event_rx: Some(event_rx),
            command_tx: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Returns the local peer ID.
    pub fn local_id(&self) -> PeerId {
        self.local_id
    }

    /// Returns the public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns a reference to the peer manager.
    pub fn peer_manager(&self) -> &Arc<PeerManager> {
        &self.peer_manager
    }

    /// Takes the event receiver (can only be called once).
    pub fn subscribe(&mut self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.event_rx.take()
    }

    /// Returns a command sender for controlling the service.
    pub fn command_sender(&self) -> Option<mpsc::Sender<NetworkCommand>> {
        self.command_tx.clone()
    }

    /// Starts the network service.
    pub async fn start(&mut self) -> NetworkResult<()> {
        if *self.running.read() {
            return Err(NetworkError::ConnectionFailed("already running".into()));
        }

        let listener = TcpListener::bind(&self.config.listen_addr).await?;
        info!("Network service listening on {}", self.config.listen_addr);

        *self.running.write() = true;

        let (command_tx, command_rx) = mpsc::channel(256);
        self.command_tx = Some(command_tx.clone());

        let (conn_event_tx, conn_event_rx) = mpsc::channel(256);

        // Spawn connection acceptor
        let running = self.running.clone();
        let conn_tx = conn_event_tx.clone();
        tokio::spawn(async move {
            Self::accept_connections(listener, conn_tx, running).await;
        });

        // Spawn main event loop
        let peer_handles: Arc<tokio::sync::RwLock<HashMap<PeerId, PeerHandle>>> =
            Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        let running = self.running.clone();
        let peer_manager = self.peer_manager.clone();
        let event_tx = self.event_tx.clone();
        let config = self.config.clone();
        let local_id = self.local_id;
        let public_key_bytes = self.public_key.to_bytes();
        let handles = peer_handles.clone();
        let conn_tx_clone = conn_event_tx.clone();

        tokio::spawn(async move {
            Self::run_event_loop(
                command_rx,
                conn_event_rx,
                peer_manager,
                event_tx,
                handles,
                config,
                local_id,
                public_key_bytes,
                running,
                conn_tx_clone,
            )
            .await;
        });

        // Connect to bootstrap nodes
        for addr in &self.config.bootstrap_nodes {
            let _ = command_tx
                .send(NetworkCommand::Connect(*addr))
                .await;
        }

        Ok(())
    }

    /// Stops the network service.
    pub async fn stop(&mut self) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(NetworkCommand::Shutdown).await;
        }
        *self.running.write() = false;
    }

    /// Broadcasts a message to all connected peers.
    pub async fn broadcast(&self, msg: NetworkMessage) -> NetworkResult<()> {
        let tx = self.command_tx.as_ref().ok_or(NetworkError::NotRunning)?;
        tx.send(NetworkCommand::Broadcast { message: msg })
            .await
            .map_err(|_| NetworkError::ChannelSend)
    }

    /// Sends a message to a specific peer.
    pub async fn send_to(&self, peer: PeerId, msg: NetworkMessage) -> NetworkResult<()> {
        let tx = self.command_tx.as_ref().ok_or(NetworkError::NotRunning)?;
        tx.send(NetworkCommand::SendMessage {
            to: peer,
            message: msg,
        })
        .await
        .map_err(|_| NetworkError::ChannelSend)
    }

    /// Accepts incoming connections.
    async fn accept_connections(
        listener: TcpListener,
        conn_tx: mpsc::Sender<ConnectionEvent>,
        running: Arc<RwLock<bool>>,
    ) {
        loop {
            if !*running.read() {
                break;
            }

            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("Accepted connection from {}", addr);
                    let _ = conn_tx
                        .send(ConnectionEvent::NewConnection {
                            stream,
                            addr,
                            outgoing: false,
                        })
                        .await;
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Main event loop.
    #[allow(clippy::too_many_arguments)]
    async fn run_event_loop(
        mut command_rx: mpsc::Receiver<NetworkCommand>,
        mut conn_event_rx: mpsc::Receiver<ConnectionEvent>,
        peer_manager: Arc<PeerManager>,
        event_tx: mpsc::Sender<NetworkEvent>,
        peer_handles: Arc<tokio::sync::RwLock<HashMap<PeerId, PeerHandle>>>,
        config: NetworkConfig,
        local_id: PeerId,
        public_key_bytes: [u8; 64],
        running: Arc<RwLock<bool>>,
        conn_tx: mpsc::Sender<ConnectionEvent>,
    ) {
        let mut ping_interval = tokio::time::interval(config.ping_interval);

        loop {
            tokio::select! {
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        NetworkCommand::SendMessage { to, message } => {
                            let sender = {
                                let handles = peer_handles.read().await;
                                handles.get(&to).map(|h| h.sender.clone())
                            };
                            if let Some(sender) = sender {
                                let _ = sender.send(message).await;
                            }
                        }
                        NetworkCommand::Broadcast { message } => {
                            let senders: Vec<_> = {
                                let handles = peer_handles.read().await;
                                handles.values().map(|h| h.sender.clone()).collect()
                            };
                            for sender in senders {
                                let _ = sender.send(message.clone()).await;
                            }
                        }
                        NetworkCommand::Connect(addr) => {
                            let conn_tx = conn_tx.clone();
                            let timeout = config.connection_timeout;
                            tokio::spawn(async move {
                                match tokio::time::timeout(timeout, TcpStream::connect(addr)).await {
                                    Ok(Ok(stream)) => {
                                        let _ = conn_tx.send(ConnectionEvent::NewConnection {
                                            stream,
                                            addr,
                                            outgoing: true,
                                        }).await;
                                    }
                                    Ok(Err(e)) => {
                                        warn!("Failed to connect to {}: {}", addr, e);
                                    }
                                    Err(_) => {
                                        warn!("Connection to {} timed out", addr);
                                    }
                                }
                            });
                        }
                        NetworkCommand::Disconnect(peer_id) => {
                            let sender = {
                                let mut handles = peer_handles.write().await;
                                handles.remove(&peer_id).map(|h| h.sender)
                            };
                            if let Some(sender) = sender {
                                let _ = sender.send(NetworkMessage::disconnect("requested")).await;
                            }
                            peer_manager.remove_peer(&peer_id);
                        }
                        NetworkCommand::Shutdown => {
                            *running.write() = false;
                            break;
                        }
                    }
                }
                Some(event) = conn_event_rx.recv() => {
                    match event {
                        ConnectionEvent::NewConnection { stream, addr, outgoing } => {
                            let info = if outgoing {
                                PeerInfo::new_outgoing(addr)
                            } else {
                                PeerInfo::new_incoming(addr)
                            };
                            let temp_id = info.id;

                            if peer_manager.add_peer(info).is_ok() {
                                // Spawn connection handler
                                let (msg_tx, msg_rx) = mpsc::channel(64);
                                {
                                    let mut handles = peer_handles.write().await;
                                    handles.insert(temp_id, PeerHandle { sender: msg_tx });
                                }

                                let conn_tx = conn_tx.clone();
                                let genesis = config.genesis_hash;
                                let pubkey = public_key_bytes;

                                tokio::spawn(async move {
                                    Self::handle_connection(
                                        stream,
                                        temp_id,
                                        local_id,
                                        genesis,
                                        pubkey,
                                        outgoing,
                                        msg_rx,
                                        conn_tx,
                                    ).await;
                                });
                            }
                        }
                        ConnectionEvent::MessageReceived { peer_id, message } => {
                            peer_manager.touch_peer(&peer_id);

                            // Handle protocol messages internally
                            match &message {
                                NetworkMessage::GetPeers => {
                                    let peers = peer_manager.get_peers_for_exchange();
                                    let sender = {
                                        let handles = peer_handles.read().await;
                                        handles.get(&peer_id).map(|h| h.sender.clone())
                                    };
                                    if let Some(sender) = sender {
                                        let _ = sender.send(NetworkMessage::Peers(peers)).await;
                                    }
                                }
                                NetworkMessage::Ping(nonce) => {
                                    let sender = {
                                        let handles = peer_handles.read().await;
                                        handles.get(&peer_id).map(|h| h.sender.clone())
                                    };
                                    if let Some(sender) = sender {
                                        let _ = sender.send(NetworkMessage::pong(*nonce)).await;
                                    }
                                }
                                _ => {
                                    // Forward to application
                                    let _ = event_tx.send(NetworkEvent::MessageReceived {
                                        from: peer_id,
                                        message,
                                    }).await;
                                }
                            }
                        }
                        ConnectionEvent::ConnectionClosed { peer_id, reason } => {
                            debug!("Connection closed for {}: {}", peer_id.short_hex(), reason);
                            {
                                let mut handles = peer_handles.write().await;
                                handles.remove(&peer_id);
                            }
                            peer_manager.remove_peer(&peer_id);
                            let _ = event_tx.send(NetworkEvent::PeerDisconnected(peer_id)).await;
                        }
                        ConnectionEvent::HandshakeComplete { temp_id, real_id, public_key, version } => {
                            // Update peer manager with real ID
                            peer_manager.update_peer_id(temp_id, real_id, public_key, version);

                            // Update handles map
                            {
                                let mut handles = peer_handles.write().await;
                                if let Some(handle) = handles.remove(&temp_id) {
                                    handles.insert(real_id, handle);
                                }
                            }

                            info!("Peer connected: {}", real_id.short_hex());
                            let _ = event_tx.send(NetworkEvent::PeerConnected(real_id)).await;
                        }
                    }
                }
                _ = ping_interval.tick() => {
                    // Send pings to all peers
                    let senders: Vec<_> = {
                        let handles = peer_handles.read().await;
                        handles.values().map(|h| h.sender.clone()).collect()
                    };
                    let ping = NetworkMessage::ping();
                    for sender in senders {
                        let _ = sender.send(ping.clone()).await;
                    }

                    // Check for stale peers
                    let stale = peer_manager.stale_peers(config.peer_timeout);
                    for peer_id in stale {
                        let sender = {
                            let mut handles = peer_handles.write().await;
                            handles.remove(&peer_id).map(|h| h.sender)
                        };
                        if let Some(sender) = sender {
                            let _ = sender.send(NetworkMessage::disconnect("timeout")).await;
                        }
                        peer_manager.remove_peer(&peer_id);
                    }
                }
                else => break,
            }
        }

        info!("Network event loop terminated");
    }

    /// Handles a single peer connection.
    #[allow(clippy::too_many_arguments)]
    async fn handle_connection(
        stream: TcpStream,
        temp_id: PeerId,
        local_id: PeerId,
        genesis_hash: H256,
        public_key_bytes: [u8; 64],
        outgoing: bool,
        mut msg_rx: mpsc::Receiver<NetworkMessage>,
        conn_tx: mpsc::Sender<ConnectionEvent>,
    ) {
        let (read_half, write_half) = stream.into_split();
        let mut reader = FramedRead::new(read_half, MessageCodec::new());
        let mut writer = FramedWrite::new(write_half, MessageCodec::new());

        // Perform handshake
        let handshake_result = Self::perform_handshake(
            &mut reader,
            &mut writer,
            local_id,
            genesis_hash,
            public_key_bytes,
            outgoing,
        )
        .await;

        let (real_id, peer_pubkey, peer_version) = match handshake_result {
            Ok(result) => result,
            Err(e) => {
                warn!("Handshake failed: {}", e);
                let _ = conn_tx
                    .send(ConnectionEvent::ConnectionClosed {
                        peer_id: temp_id,
                        reason: format!("handshake failed: {}", e),
                    })
                    .await;
                return;
            }
        };

        // Notify handshake complete
        let _ = conn_tx
            .send(ConnectionEvent::HandshakeComplete {
                temp_id,
                real_id,
                public_key: peer_pubkey,
                version: peer_version,
            })
            .await;

        // Main message loop
        loop {
            tokio::select! {
                msg_result = reader.next() => {
                    match msg_result {
                        Some(Ok(msg)) => {
                            if matches!(msg, NetworkMessage::Disconnect { .. }) {
                                let _ = conn_tx.send(ConnectionEvent::ConnectionClosed {
                                    peer_id: real_id,
                                    reason: "peer disconnected".into(),
                                }).await;
                                break;
                            }
                            let _ = conn_tx.send(ConnectionEvent::MessageReceived {
                                peer_id: real_id,
                                message: msg,
                            }).await;
                        }
                        Some(Err(e)) => {
                            let _ = conn_tx.send(ConnectionEvent::ConnectionClosed {
                                peer_id: real_id,
                                reason: format!("read error: {}", e),
                            }).await;
                            break;
                        }
                        None => {
                            let _ = conn_tx.send(ConnectionEvent::ConnectionClosed {
                                peer_id: real_id,
                                reason: "connection closed".into(),
                            }).await;
                            break;
                        }
                    }
                }
                Some(msg) = msg_rx.recv() => {
                    use futures::SinkExt;
                    if writer.send(msg).await.is_err() {
                        let _ = conn_tx.send(ConnectionEvent::ConnectionClosed {
                            peer_id: real_id,
                            reason: "write error".into(),
                        }).await;
                        break;
                    }
                }
            }
        }
    }

    /// Performs the handshake protocol.
    async fn perform_handshake<R, W>(
        reader: &mut FramedRead<R, MessageCodec>,
        writer: &mut FramedWrite<W, MessageCodec>,
        local_id: PeerId,
        genesis_hash: H256,
        public_key_bytes: [u8; 64],
        outgoing: bool,
    ) -> NetworkResult<(PeerId, PublicKey, u32)>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        use futures::SinkExt;

        if outgoing {
            // Send Hello first
            let hello = NetworkMessage::hello(local_id, genesis_hash, public_key_bytes);
            writer
                .send(hello)
                .await
                .map_err(|e| NetworkError::HandshakeFailed(format!("send hello: {}", e)))?;

            // Wait for HelloAck
            let response = tokio::time::timeout(Duration::from_secs(10), reader.next())
                .await
                .map_err(|_| NetworkError::HandshakeFailed("timeout waiting for HelloAck".into()))?
                .ok_or_else(|| NetworkError::HandshakeFailed("connection closed".into()))?
                .map_err(|e| NetworkError::HandshakeFailed(format!("read HelloAck: {}", e)))?;

            match response {
                NetworkMessage::HelloAck { peer_id, public_key } => {
                    let pubkey = PublicKey::from_bytes(&public_key)
                        .map_err(|_| NetworkError::HandshakeFailed("invalid public key".into()))?;
                    let expected_id = PeerId::from_public_key(&pubkey);
                    if expected_id.0 != peer_id {
                        return Err(NetworkError::HandshakeFailed(
                            "peer ID doesn't match public key".into(),
                        ));
                    }
                    Ok((expected_id, pubkey, PROTOCOL_VERSION))
                }
                _ => Err(NetworkError::HandshakeFailed("expected HelloAck".into())),
            }
        } else {
            // Wait for Hello first
            let hello = tokio::time::timeout(Duration::from_secs(10), reader.next())
                .await
                .map_err(|_| NetworkError::HandshakeFailed("timeout waiting for Hello".into()))?
                .ok_or_else(|| NetworkError::HandshakeFailed("connection closed".into()))?
                .map_err(|e| NetworkError::HandshakeFailed(format!("read Hello: {}", e)))?;

            match hello {
                NetworkMessage::Hello {
                    version,
                    peer_id,
                    genesis_hash: peer_genesis,
                    public_key,
                } => {
                    // Verify version
                    if version != PROTOCOL_VERSION {
                        return Err(NetworkError::VersionMismatch {
                            our_version: PROTOCOL_VERSION,
                            peer_version: version,
                        });
                    }

                    // Verify genesis
                    if peer_genesis != *genesis_hash.as_bytes() {
                        return Err(NetworkError::GenesisMismatch {
                            expected: format!("{}", genesis_hash),
                            actual: format!("0x{}", hex::encode(peer_genesis)),
                        });
                    }

                    // Verify public key matches peer ID
                    let pubkey = PublicKey::from_bytes(&public_key)
                        .map_err(|_| NetworkError::HandshakeFailed("invalid public key".into()))?;
                    let expected_id = PeerId::from_public_key(&pubkey);
                    if expected_id.0 != peer_id {
                        return Err(NetworkError::HandshakeFailed(
                            "peer ID doesn't match public key".into(),
                        ));
                    }

                    // Send HelloAck
                    let ack = NetworkMessage::hello_ack(local_id, public_key_bytes);
                    writer
                        .send(ack)
                        .await
                        .map_err(|e| NetworkError::HandshakeFailed(format!("send HelloAck: {}", e)))?;

                    Ok((expected_id, pubkey, version))
                }
                _ => Err(NetworkError::HandshakeFailed("expected Hello".into())),
            }
        }
    }
}

/// Helper for hex encoding (used in genesis mismatch error).
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_config_builder() {
        let config = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:9999".parse().unwrap())
            .with_max_peers(50);

        assert_eq!(config.listen_addr.port(), 9999);
        assert_eq!(config.max_peers, 50);
    }

    #[tokio::test]
    async fn test_service_creation() {
        let config = NetworkConfig::default();
        let service = NetworkService::new(config).await;

        assert!(!service.local_id().as_bytes().iter().all(|&b| b == 0));
    }

    #[tokio::test]
    async fn test_peer_to_peer_connection() {
        // Create two services
        let config1 = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap())
            .with_genesis_hash(H256::from([1u8; 32]));

        let config2 = NetworkConfig::default()
            .with_listen_addr("127.0.0.1:0".parse().unwrap())
            .with_genesis_hash(H256::from([1u8; 32]));

        let mut service1 = NetworkService::new(config1).await;
        let mut service2 = NetworkService::new(config2).await;

        let _events1 = service1.subscribe().unwrap();
        let _events2 = service2.subscribe().unwrap();

        // Start services
        service1.start().await.unwrap();
        service2.start().await.unwrap();

        // Give them a moment to bind
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get actual listen addresses
        // Note: In a real test we'd need to expose the bound address
        // For now, this test validates the service can start without errors

        // Cleanup
        service1.stop().await;
        service2.stop().await;
    }
}
