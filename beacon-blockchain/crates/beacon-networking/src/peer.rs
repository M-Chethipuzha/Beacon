use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Information about a network peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer identifier
    pub peer_id: PeerId,
    /// Known addresses for this peer
    pub addresses: Vec<Multiaddr>,
    /// Last time we connected to this peer
    pub last_seen: u64,
    /// Connection status
    pub status: PeerStatus,
    /// Protocol version supported by this peer
    pub protocol_version: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Latency to this peer (in milliseconds)
    pub latency: Option<u64>,
    /// Reputation score (0-100)
    pub reputation: u8,
}

/// Peer connection status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeerStatus {
    /// Currently connected
    Connected,
    /// Disconnected
    Disconnected,
    /// Connection attempt in progress
    Connecting,
    /// Peer is banned
    Banned,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(peer_id: PeerId, addresses: Vec<Multiaddr>) -> Self {
        Self {
            peer_id,
            addresses,
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: PeerStatus::Connected,
            protocol_version: None,
            user_agent: None,
            latency: None,
            reputation: 50, // Start with neutral reputation
        }
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Set the connection status
    pub fn set_status(&mut self, status: PeerStatus) {
        self.status = status;
        if status == PeerStatus::Connected {
            self.update_last_seen();
        }
    }

    /// Add a new address if not already known
    pub fn add_address(&mut self, address: Multiaddr) {
        if !self.addresses.contains(&address) {
            self.addresses.push(address);
        }
    }

    /// Remove an address
    pub fn remove_address(&mut self, address: &Multiaddr) {
        self.addresses.retain(|addr| addr != address);
    }

    /// Check if this peer was seen recently
    pub fn is_recent(&self, threshold: Duration) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now - self.last_seen < threshold.as_secs()
    }

    /// Update peer reputation (clamp between 0 and 100)
    pub fn adjust_reputation(&mut self, delta: i16) {
        let new_reputation = (self.reputation as i16 + delta).clamp(0, 100) as u8;
        self.reputation = new_reputation;
    }

    /// Check if peer has good reputation
    pub fn has_good_reputation(&self) -> bool {
        self.reputation >= 60
    }

    /// Check if peer should be banned
    pub fn should_be_banned(&self) -> bool {
        self.reputation < 20
    }

    /// Set protocol information
    pub fn set_protocol_info(&mut self, version: String, user_agent: String) {
        self.protocol_version = Some(version);
        self.user_agent = Some(user_agent);
    }

    /// Update latency measurement
    pub fn update_latency(&mut self, latency_ms: u64) {
        self.latency = Some(latency_ms);
    }
}

/// Peer management configuration
#[derive(Debug, Clone)]
pub struct PeerManagerConfig {
    /// Maximum number of connected peers
    pub max_peers: usize,
    /// Maximum number of peers to store in memory
    pub max_stored_peers: usize,
    /// How long to remember disconnected peers
    pub peer_retention_time: Duration,
    /// Minimum reputation for connecting
    pub min_reputation: u8,
    /// Ban duration for misbehaving peers
    pub ban_duration: Duration,
}

impl Default for PeerManagerConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            max_stored_peers: 1000,
            peer_retention_time: Duration::from_secs(24 * 60 * 60), // 24 hours
            min_reputation: 30,
            ban_duration: Duration::from_secs(60 * 60), // 1 hour
        }
    }
}

/// Manages peer connections and reputation
pub struct PeerManager {
    peers: std::collections::HashMap<PeerId, PeerInfo>,
    banned_peers: std::collections::HashMap<PeerId, u64>, // PeerId -> ban expiry timestamp
    config: PeerManagerConfig,
}

impl PeerManager {
    /// Create a new peer manager
    pub fn new(config: PeerManagerConfig) -> Self {
        Self {
            peers: std::collections::HashMap::new(),
            banned_peers: std::collections::HashMap::new(),
            config,
        }
    }

    /// Add or update a peer
    pub fn add_peer(&mut self, mut peer_info: PeerInfo) {
        // Check if peer is banned
        if self.is_peer_banned(&peer_info.peer_id) {
            peer_info.set_status(PeerStatus::Banned);
        }

        self.peers.insert(peer_info.peer_id, peer_info);
        self.cleanup_old_peers();
    }

    /// Get peer information
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }

    /// Get mutable peer information
    pub fn get_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerInfo> {
        self.peers.get_mut(peer_id)
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
    }

    /// Get all connected peers
    pub fn get_connected_peers(&self) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|peer| peer.status == PeerStatus::Connected)
            .collect()
    }

    /// Get all peers with good reputation
    pub fn get_good_peers(&self) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|peer| peer.has_good_reputation() && !self.is_peer_banned(&peer.peer_id))
            .collect()
    }

    /// Ban a peer
    pub fn ban_peer(&mut self, peer_id: &PeerId, reason: &str) {
        let ban_expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + self.config.ban_duration.as_secs();
        
        self.banned_peers.insert(*peer_id, ban_expiry);
        
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.set_status(PeerStatus::Banned);
            peer.reputation = 0;
        }
        
        tracing::info!("Banned peer {} for reason: {}", peer_id, reason);
    }

    /// Check if a peer is banned
    pub fn is_peer_banned(&self, peer_id: &PeerId) -> bool {
        if let Some(&ban_expiry) = self.banned_peers.get(peer_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            if now < ban_expiry {
                return true;
            } else {
                // Ban expired, we'll clean it up later
                return false;
            }
        }
        false
    }

    /// Adjust peer reputation
    pub fn adjust_peer_reputation(&mut self, peer_id: &PeerId, delta: i16, reason: &str) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            let old_reputation = peer.reputation;
            peer.adjust_reputation(delta);
            
            tracing::debug!(
                "Adjusted reputation for peer {} from {} to {} (delta: {}, reason: {})",
                peer_id,
                old_reputation,
                peer.reputation,
                delta,
                reason
            );
            
            // Ban peer if reputation is too low
            if peer.should_be_banned() && !self.is_peer_banned(peer_id) {
                self.ban_peer(peer_id, "Low reputation");
            }
        }
    }

    /// Get the number of connected peers
    pub fn connected_peer_count(&self) -> usize {
        self.peers
            .values()
            .filter(|peer| peer.status == PeerStatus::Connected)
            .count()
    }

    /// Check if we can accept more connections
    pub fn can_accept_more_peers(&self) -> bool {
        self.connected_peer_count() < self.config.max_peers
    }

    /// Clean up old and banned peers
    pub fn cleanup_old_peers(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Remove expired bans
        self.banned_peers.retain(|_, &mut ban_expiry| now < ban_expiry);
        
        // Remove old peers if we have too many stored
        if self.peers.len() > self.config.max_stored_peers {
            let retention_threshold = now - self.config.peer_retention_time.as_secs();
            
            let peers_to_remove: Vec<PeerId> = self
                .peers
                .iter()
                .filter(|(_, peer)| {
                    peer.status != PeerStatus::Connected && peer.last_seen < retention_threshold
                })
                .map(|(peer_id, _)| *peer_id)
                .collect();
            
            for peer_id in peers_to_remove {
                self.peers.remove(&peer_id);
            }
        }
    }

    /// Get statistics about peers
    pub fn get_stats(&self) -> PeerStats {
        let connected = self.connected_peer_count();
        let total = self.peers.len();
        let banned = self.banned_peers.len();
        let good_reputation = self
            .peers
            .values()
            .filter(|peer| peer.has_good_reputation())
            .count();
        
        PeerStats {
            connected,
            total,
            banned,
            good_reputation,
        }
    }
}

/// Peer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStats {
    pub connected: usize,
    pub total: usize,
    pub banned: usize,
    pub good_reputation: usize,
}
