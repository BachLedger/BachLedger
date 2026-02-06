//! Peer management

use crate::error::{NetworkError, NetworkResult};
use crate::types::{Message, PeerId};
use bytes::BytesMut;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    /// Connecting (handshake in progress)
    Connecting,
    /// Connected and handshake complete
    Connected,
    /// Disconnecting
    Disconnecting,
    /// Disconnected
    Disconnected,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer ID
    pub id: PeerId,
    /// Remote address
    pub addr: SocketAddr,
    /// Connection state
    pub state: PeerState,
    /// Peer's reported chain height
    pub height: u64,
    /// Connection time
    pub connected_at: Instant,
    /// Last message time
    pub last_message_at: Instant,
    /// Is inbound connection
    pub inbound: bool,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(id: PeerId, addr: SocketAddr, inbound: bool) -> Self {
        let now = Instant::now();
        Self {
            id,
            addr,
            state: PeerState::Connecting,
            height: 0,
            connected_at: now,
            last_message_at: now,
            inbound,
        }
    }

    /// Update height
    pub fn set_height(&mut self, height: u64) {
        self.height = height;
    }

    /// Update last message time
    pub fn touch(&mut self) {
        self.last_message_at = Instant::now();
    }
}

/// Peer connection handle
pub struct PeerConnection {
    /// Peer ID
    pub peer_id: PeerId,
    /// Remote address
    pub addr: SocketAddr,
    /// Outgoing message sender
    pub sender: mpsc::Sender<Message>,
}

impl PeerConnection {
    /// Send a message to this peer
    pub async fn send(&self, msg: Message) -> NetworkResult<()> {
        self.sender
            .send(msg)
            .await
            .map_err(|_| NetworkError::ChannelClosed)
    }
}

/// Read a message from the stream
pub async fn read_message(stream: &mut TcpStream) -> NetworkResult<Message> {
    // Read length prefix (4 bytes)
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len == 0 || len > 16 * 1024 * 1024 {
        return Err(NetworkError::InvalidMessage("invalid message length".into()));
    }

    // Read message
    let mut buf = BytesMut::with_capacity(4 + len);
    buf.extend_from_slice(&len_buf);
    buf.resize(4 + len, 0);
    stream.read_exact(&mut buf[4..]).await?;

    Message::decode(buf.freeze())
        .ok_or_else(|| NetworkError::InvalidMessage("failed to decode message".into()))
}

/// Write a message to the stream
pub async fn write_message(stream: &mut TcpStream, msg: &Message) -> NetworkResult<()> {
    let data = msg.encode();
    stream.write_all(&data).await?;
    stream.flush().await?;
    Ok(())
}

/// Peer manager
pub struct PeerManager {
    /// Connected peers info
    peers: RwLock<HashMap<PeerId, PeerInfo>>,
    /// Peer connections
    connections: RwLock<HashMap<PeerId, Arc<PeerConnection>>>,
    /// Max peers
    max_peers: usize,
}

impl PeerManager {
    /// Create new peer manager
    pub fn new(max_peers: usize) -> Self {
        Self {
            peers: RwLock::new(HashMap::new()),
            connections: RwLock::new(HashMap::new()),
            max_peers,
        }
    }

    /// Add a peer
    pub fn add_peer(&self, info: PeerInfo, conn: Arc<PeerConnection>) -> NetworkResult<()> {
        let mut peers = self.peers.write();
        let mut connections = self.connections.write();

        if peers.len() >= self.max_peers {
            return Err(NetworkError::ConnectionFailed("max peers reached".into()));
        }

        if peers.contains_key(&info.id) {
            return Err(NetworkError::AlreadyConnected(info.id.to_string()));
        }

        peers.insert(info.id, info);
        connections.insert(conn.peer_id, conn);
        Ok(())
    }

    /// Remove a peer
    pub fn remove_peer(&self, peer_id: &PeerId) {
        self.peers.write().remove(peer_id);
        self.connections.write().remove(peer_id);
    }

    /// Get peer info
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.read().get(peer_id).cloned()
    }

    /// Get peer connection
    pub fn get_connection(&self, peer_id: &PeerId) -> Option<Arc<PeerConnection>> {
        self.connections.read().get(peer_id).cloned()
    }

    /// Update peer state
    pub fn update_state(&self, peer_id: &PeerId, state: PeerState) {
        if let Some(peer) = self.peers.write().get_mut(peer_id) {
            peer.state = state;
        }
    }

    /// Update peer height
    pub fn update_height(&self, peer_id: &PeerId, height: u64) {
        if let Some(peer) = self.peers.write().get_mut(peer_id) {
            peer.set_height(height);
        }
    }

    /// Touch peer (update last message time)
    pub fn touch_peer(&self, peer_id: &PeerId) {
        if let Some(peer) = self.peers.write().get_mut(peer_id) {
            peer.touch();
        }
    }

    /// Get all peer IDs
    pub fn peer_ids(&self) -> Vec<PeerId> {
        self.peers.read().keys().cloned().collect()
    }

    /// Get all connected peers
    pub fn connected_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .read()
            .values()
            .filter(|p| p.state == PeerState::Connected)
            .cloned()
            .collect()
    }

    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.peers.read().len()
    }

    /// Check if can accept more peers
    pub fn can_accept(&self) -> bool {
        self.peers.read().len() < self.max_peers
    }

    /// Check if peer exists
    pub fn has_peer(&self, peer_id: &PeerId) -> bool {
        self.peers.read().contains_key(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_addr() -> SocketAddr {
        "127.0.0.1:8000".parse().unwrap()
    }

    #[test]
    fn test_peer_info_creation() {
        let id = PeerId::random();
        let info = PeerInfo::new(id, test_addr(), false);

        assert_eq!(info.id, id);
        assert_eq!(info.state, PeerState::Connecting);
        assert_eq!(info.height, 0);
        assert!(!info.inbound);
    }

    #[test]
    fn test_peer_info_touch() {
        let id = PeerId::random();
        let mut info = PeerInfo::new(id, test_addr(), false);
        let before = info.last_message_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        info.touch();

        assert!(info.last_message_at > before);
    }

    #[test]
    fn test_peer_manager_add_remove() {
        let manager = PeerManager::new(10);
        let id = PeerId::random();
        let info = PeerInfo::new(id, test_addr(), false);

        let (tx, _rx) = mpsc::channel(1);
        let conn = Arc::new(PeerConnection {
            peer_id: id,
            addr: test_addr(),
            sender: tx,
        });

        manager.add_peer(info, conn).unwrap();
        assert!(manager.has_peer(&id));
        assert_eq!(manager.peer_count(), 1);

        manager.remove_peer(&id);
        assert!(!manager.has_peer(&id));
        assert_eq!(manager.peer_count(), 0);
    }

    #[test]
    fn test_peer_manager_max_peers() {
        let manager = PeerManager::new(2);

        for i in 0..2 {
            let id = PeerId::from_bytes([i as u8; 32]);
            let info = PeerInfo::new(id, test_addr(), false);
            let (tx, _rx) = mpsc::channel(1);
            let conn = Arc::new(PeerConnection {
                peer_id: id,
                addr: test_addr(),
                sender: tx,
            });
            manager.add_peer(info, conn).unwrap();
        }

        // Third peer should fail
        let id = PeerId::from_bytes([99; 32]);
        let info = PeerInfo::new(id, test_addr(), false);
        let (tx, _rx) = mpsc::channel(1);
        let conn = Arc::new(PeerConnection {
            peer_id: id,
            addr: test_addr(),
            sender: tx,
        });
        let result = manager.add_peer(info, conn);
        assert!(matches!(result, Err(NetworkError::ConnectionFailed(_))));
    }

    #[test]
    fn test_peer_manager_duplicate() {
        let manager = PeerManager::new(10);
        let id = PeerId::random();

        // First add should succeed
        let info1 = PeerInfo::new(id, test_addr(), false);
        let (tx1, _rx1) = mpsc::channel(1);
        let conn1 = Arc::new(PeerConnection {
            peer_id: id,
            addr: test_addr(),
            sender: tx1,
        });
        assert!(manager.add_peer(info1, conn1).is_ok());
        assert_eq!(manager.peer_count(), 1);

        // Second add with same ID should fail
        let info2 = PeerInfo::new(id, test_addr(), false);
        let (tx2, _rx2) = mpsc::channel(1);
        let conn2 = Arc::new(PeerConnection {
            peer_id: id,
            addr: test_addr(),
            sender: tx2,
        });
        let result = manager.add_peer(info2, conn2);
        assert!(matches!(result, Err(NetworkError::AlreadyConnected(_))));
        assert_eq!(manager.peer_count(), 1);
    }

    #[test]
    fn test_peer_manager_state_update() {
        let manager = PeerManager::new(10);
        let id = PeerId::random();
        let info = PeerInfo::new(id, test_addr(), false);
        let (tx, _rx) = mpsc::channel(1);
        let conn = Arc::new(PeerConnection {
            peer_id: id,
            addr: test_addr(),
            sender: tx,
        });
        manager.add_peer(info, conn).unwrap();

        manager.update_state(&id, PeerState::Connected);
        let peer = manager.get_peer(&id).unwrap();
        assert_eq!(peer.state, PeerState::Connected);
    }

    #[test]
    fn test_peer_manager_connected_peers() {
        let manager = PeerManager::new(10);

        // Add connected peer
        let id1 = PeerId::from_bytes([1; 32]);
        let mut info1 = PeerInfo::new(id1, test_addr(), false);
        info1.state = PeerState::Connected;
        let (tx1, _rx1) = mpsc::channel(1);
        let conn1 = Arc::new(PeerConnection {
            peer_id: id1,
            addr: test_addr(),
            sender: tx1,
        });
        manager.add_peer(info1, conn1).unwrap();

        // Add connecting peer
        let id2 = PeerId::from_bytes([2; 32]);
        let info2 = PeerInfo::new(id2, test_addr(), false);
        let (tx2, _rx2) = mpsc::channel(1);
        let conn2 = Arc::new(PeerConnection {
            peer_id: id2,
            addr: test_addr(),
            sender: tx2,
        });
        manager.add_peer(info2, conn2).unwrap();

        let connected = manager.connected_peers();
        assert_eq!(connected.len(), 1);
        assert_eq!(connected[0].id, id1);
    }
}
