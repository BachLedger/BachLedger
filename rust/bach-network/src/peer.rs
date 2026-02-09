//! Peer management and discovery

use bach_crypto::{keccak256, PublicKey};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// A 32-byte peer identifier derived from the public key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    /// Creates a PeerId from a public key (hash of the public key bytes).
    pub fn from_public_key(pubkey: &PublicKey) -> Self {
        let hash = keccak256(&pubkey.to_bytes());
        Self(*hash.as_bytes())
    }

    /// Creates a PeerId from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns a short hex representation for logging.
    pub fn short_hex(&self) -> String {
        format!("{:02x}{:02x}..{:02x}{:02x}",
            self.0[0], self.0[1], self.0[30], self.0[31])
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Status of a peer connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerStatus {
    /// Connection attempt in progress
    Connecting,
    /// Connected but handshake pending
    Connected,
    /// Fully authenticated and ready
    Active,
    /// Disconnecting
    Disconnecting,
}

/// Information about a peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Unique peer identifier
    pub id: PeerId,
    /// Network address
    pub address: SocketAddr,
    /// Peer's public key (available after handshake)
    pub public_key: Option<PublicKey>,
    /// Connection status
    pub status: PeerStatus,
    /// Last activity timestamp
    pub last_seen: Instant,
    /// Protocol version reported by peer
    pub version: Option<u32>,
    /// Number of failed connection attempts
    pub failed_attempts: u32,
    /// Last connection failure time
    pub last_failure: Option<Instant>,
}

impl PeerInfo {
    /// Creates new peer info for an outgoing connection.
    pub fn new_outgoing(address: SocketAddr) -> Self {
        // Generate temporary ID from address until handshake completes
        let mut id_bytes = [0u8; 32];
        let addr_bytes = format!("{}", address);
        let hash = keccak256(addr_bytes.as_bytes());
        id_bytes.copy_from_slice(hash.as_bytes());

        Self {
            id: PeerId(id_bytes),
            address,
            public_key: None,
            status: PeerStatus::Connecting,
            last_seen: Instant::now(),
            version: None,
            failed_attempts: 0,
            last_failure: None,
        }
    }

    /// Creates new peer info for an incoming connection.
    pub fn new_incoming(address: SocketAddr) -> Self {
        let mut info = Self::new_outgoing(address);
        info.status = PeerStatus::Connected;
        info
    }

    /// Updates the peer with handshake information.
    pub fn complete_handshake(&mut self, id: PeerId, public_key: PublicKey, version: u32) {
        self.id = id;
        self.public_key = Some(public_key);
        self.version = Some(version);
        self.status = PeerStatus::Active;
        self.last_seen = Instant::now();
    }

    /// Records a failed connection attempt.
    pub fn record_failure(&mut self) {
        self.failed_attempts += 1;
        self.last_failure = Some(Instant::now());
    }

    /// Returns the backoff duration before next connection attempt.
    pub fn backoff_duration(&self) -> Duration {
        let base = Duration::from_secs(5);
        let factor = 2u32.saturating_pow(self.failed_attempts.min(6));
        base * factor
    }

    /// Returns true if enough time has passed since the last failure.
    pub fn can_retry(&self) -> bool {
        match self.last_failure {
            None => true,
            Some(t) => t.elapsed() >= self.backoff_duration(),
        }
    }
}

/// Serializable peer info for peer exchange.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializablePeerInfo {
    /// Peer ID bytes
    pub id: [u8; 32],
    /// Socket address as string
    pub address: String,
}

impl From<&PeerInfo> for SerializablePeerInfo {
    fn from(info: &PeerInfo) -> Self {
        Self {
            id: info.id.0,
            address: info.address.to_string(),
        }
    }
}

/// Manages peer connections and discovery.
pub struct PeerManager {
    /// Known peers by ID
    peers: RwLock<HashMap<PeerId, PeerInfo>>,
    /// Peers by address for deduplication
    peers_by_addr: RwLock<HashMap<SocketAddr, PeerId>>,
    /// Maximum number of peers
    max_peers: usize,
    /// Bootstrap nodes
    bootstrap_nodes: Vec<SocketAddr>,
    /// Our peer ID
    local_id: Option<PeerId>,
}

impl PeerManager {
    /// Creates a new peer manager.
    pub fn new(max_peers: usize, bootstrap_nodes: Vec<SocketAddr>) -> Self {
        Self {
            peers: RwLock::new(HashMap::new()),
            peers_by_addr: RwLock::new(HashMap::new()),
            max_peers,
            bootstrap_nodes,
            local_id: None,
        }
    }

    /// Sets the local peer ID.
    pub fn set_local_id(&mut self, id: PeerId) {
        self.local_id = Some(id);
    }

    /// Returns the local peer ID.
    pub fn local_id(&self) -> Option<PeerId> {
        self.local_id
    }

    /// Adds a new peer (before handshake completes).
    pub fn add_peer(&self, info: PeerInfo) -> Result<(), &'static str> {
        let mut peers = self.peers.write();
        let mut by_addr = self.peers_by_addr.write();

        // Check limits
        let active_count = peers.values()
            .filter(|p| p.status == PeerStatus::Active)
            .count();
        if active_count >= self.max_peers {
            return Err("max peers reached");
        }

        // Check for duplicate address
        if by_addr.contains_key(&info.address) {
            return Err("already connected to address");
        }

        by_addr.insert(info.address, info.id);
        peers.insert(info.id, info);
        Ok(())
    }

    /// Updates peer info after handshake.
    pub fn update_peer_id(&self, old_id: PeerId, new_id: PeerId, public_key: PublicKey, version: u32) {
        let mut peers = self.peers.write();
        let mut by_addr = self.peers_by_addr.write();

        if let Some(mut info) = peers.remove(&old_id) {
            by_addr.insert(info.address, new_id);
            info.complete_handshake(new_id, public_key, version);
            peers.insert(new_id, info);
        }
    }

    /// Removes a peer.
    pub fn remove_peer(&self, id: &PeerId) {
        let mut peers = self.peers.write();
        let mut by_addr = self.peers_by_addr.write();

        if let Some(info) = peers.remove(id) {
            by_addr.remove(&info.address);
        }
    }

    /// Gets peer info by ID.
    pub fn get_peer(&self, id: &PeerId) -> Option<PeerInfo> {
        self.peers.read().get(id).cloned()
    }

    /// Gets peer ID by address.
    pub fn get_peer_by_addr(&self, addr: &SocketAddr) -> Option<PeerId> {
        self.peers_by_addr.read().get(addr).copied()
    }

    /// Updates last seen time for a peer.
    pub fn touch_peer(&self, id: &PeerId) {
        if let Some(peer) = self.peers.write().get_mut(id) {
            peer.last_seen = Instant::now();
        }
    }

    /// Returns all active peer IDs.
    pub fn active_peers(&self) -> Vec<PeerId> {
        self.peers.read()
            .values()
            .filter(|p| p.status == PeerStatus::Active)
            .map(|p| p.id)
            .collect()
    }

    /// Returns the number of active peers.
    pub fn active_count(&self) -> usize {
        self.peers.read()
            .values()
            .filter(|p| p.status == PeerStatus::Active)
            .count()
    }

    /// Returns all peer info for peer exchange.
    pub fn get_peers_for_exchange(&self) -> Vec<SerializablePeerInfo> {
        self.peers.read()
            .values()
            .filter(|p| p.status == PeerStatus::Active)
            .map(SerializablePeerInfo::from)
            .collect()
    }

    /// Returns bootstrap nodes.
    pub fn bootstrap_nodes(&self) -> &[SocketAddr] {
        &self.bootstrap_nodes
    }

    /// Returns addresses to connect to (bootstrap nodes not yet connected).
    pub fn get_connectable_addresses(&self) -> Vec<SocketAddr> {
        let by_addr = self.peers_by_addr.read();
        self.bootstrap_nodes
            .iter()
            .filter(|addr| !by_addr.contains_key(addr))
            .copied()
            .collect()
    }

    /// Checks if we need more peers.
    pub fn needs_peers(&self) -> bool {
        self.active_count() < self.max_peers
    }

    /// Returns peers that have been inactive for too long.
    pub fn stale_peers(&self, timeout: Duration) -> Vec<PeerId> {
        self.peers.read()
            .values()
            .filter(|p| p.status == PeerStatus::Active && p.last_seen.elapsed() > timeout)
            .map(|p| p.id)
            .collect()
    }

    /// Returns the maximum peers setting.
    pub fn max_peers(&self) -> usize {
        self.max_peers
    }
}

impl std::fmt::Debug for PeerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let peers = self.peers.read();
        f.debug_struct("PeerManager")
            .field("peer_count", &peers.len())
            .field("max_peers", &self.max_peers)
            .field("bootstrap_nodes", &self.bootstrap_nodes.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id_from_bytes() {
        let bytes = [1u8; 32];
        let id = PeerId::from_bytes(bytes);
        assert_eq!(id.as_bytes(), &bytes);
    }

    #[test]
    fn test_peer_id_display() {
        let bytes = [0xab; 32];
        let id = PeerId::from_bytes(bytes);
        let display = format!("{}", id);
        assert!(display.starts_with("0x"));
        assert_eq!(display.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_peer_info_backoff() {
        let mut info = PeerInfo::new_outgoing("127.0.0.1:8080".parse().unwrap());
        assert!(info.can_retry());

        info.record_failure();
        assert_eq!(info.failed_attempts, 1);
        // Just after failure, should need to wait
        assert!(!info.can_retry());
    }

    #[test]
    fn test_peer_manager_add_remove() {
        let manager = PeerManager::new(10, vec![]);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let info = PeerInfo::new_incoming(addr);
        let id = info.id;

        manager.add_peer(info).unwrap();
        assert!(manager.get_peer(&id).is_some());
        assert_eq!(manager.get_peer_by_addr(&addr), Some(id));

        manager.remove_peer(&id);
        assert!(manager.get_peer(&id).is_none());
    }

    #[test]
    fn test_peer_manager_max_peers() {
        let manager = PeerManager::new(2, vec![]);

        // Add two peers successfully
        for i in 0..2 {
            let addr: SocketAddr = format!("127.0.0.1:808{}", i).parse().unwrap();
            let mut info = PeerInfo::new_incoming(addr);
            info.status = PeerStatus::Active;
            manager.add_peer(info).unwrap();
        }

        // Third should fail
        let addr: SocketAddr = "127.0.0.1:8082".parse().unwrap();
        let mut info = PeerInfo::new_incoming(addr);
        info.status = PeerStatus::Active;
        assert!(manager.add_peer(info).is_err());
    }
}
