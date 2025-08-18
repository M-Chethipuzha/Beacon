use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{debug, info, warn};
use beacon_core::{BeaconError, BeaconResult};

/// Peer discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Bootstrap peers to connect to initially
    pub bootstrap_peers: Vec<Multiaddr>,
    /// Interval for active peer discovery
    pub discovery_interval: Duration,
    /// Maximum number of peers to discover per round
    pub max_discovery_peers: usize,
    /// Timeout for connection attempts
    pub connection_timeout: Duration,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            bootstrap_peers: Vec::new(),
            discovery_interval: Duration::from_secs(30),
            max_discovery_peers: 10,
            connection_timeout: Duration::from_secs(10),
        }
    }
}

/// Peer discovery service
pub struct PeerDiscovery {
    config: DiscoveryConfig,
    discovered_peers: HashMap<PeerId, DiscoveredPeer>,
    last_discovery: Instant,
}

/// Information about a discovered peer
#[derive(Debug, Clone)]
struct DiscoveredPeer {
    peer_id: PeerId,
    addresses: Vec<Multiaddr>,
    discovered_at: Instant,
    connection_attempts: u32,
    last_attempt: Option<Instant>,
}

impl PeerDiscovery {
    /// Create a new peer discovery service
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            discovered_peers: HashMap::new(),
            last_discovery: Instant::now(),
        }
    }

    /// Start peer discovery process
    pub async fn start_discovery(&mut self) -> BeaconResult<Vec<Multiaddr>> {
        let mut peers_to_connect = Vec::new();

        // Connect to bootstrap peers first
        if self.discovered_peers.is_empty() {
            info!("Connecting to {} bootstrap peers", self.config.bootstrap_peers.len());
            for addr in &self.config.bootstrap_peers {
                peers_to_connect.push(addr.clone());
            }
        }

        // Check if it's time for active discovery
        if self.last_discovery.elapsed() > self.config.discovery_interval {
            debug!("Starting active peer discovery");
            let discovered = self.discover_new_peers().await?;
            peers_to_connect.extend(discovered);
            self.last_discovery = Instant::now();
        }

        // Clean up old discovery entries
        self.cleanup_old_discoveries();

        Ok(peers_to_connect)
    }

    /// Discover new peers through various methods
    async fn discover_new_peers(&mut self) -> BeaconResult<Vec<Multiaddr>> {
        let mut new_peers = Vec::new();

        // For now, implement a simple discovery mechanism
        // In a real implementation, this would include:
        // - DHT (Kademlia) lookups
        // - mDNS discovery
        // - Peer exchange with connected peers
        // - DNS seed nodes

        // Example: Random walk through known peers
        let known_peers: Vec<_> = self.discovered_peers.values().collect();
        for peer in known_peers.iter().take(self.config.max_discovery_peers) {
            // Check if we should retry connection
            if self.should_retry_connection(peer) {
                new_peers.extend(peer.addresses.clone());
            }
        }

        Ok(new_peers)
    }

    /// Check if we should retry connecting to a peer
    fn should_retry_connection(&self, peer: &DiscoveredPeer) -> bool {
        // Don't retry if we've attempted too many times
        if peer.connection_attempts >= 3 {
            return false;
        }

        // Don't retry if we've attempted recently
        if let Some(last_attempt) = peer.last_attempt {
            if last_attempt.elapsed() < Duration::from_secs(60) {
                return false;
            }
        }

        true
    }

    /// Add a discovered peer
    pub fn add_discovered_peer(&mut self, peer_id: PeerId, addresses: Vec<Multiaddr>) {
        let discovered_peer = DiscoveredPeer {
            peer_id,
            addresses,
            discovered_at: Instant::now(),
            connection_attempts: 0,
            last_attempt: None,
        };

        self.discovered_peers.insert(peer_id, discovered_peer);
        debug!("Added discovered peer: {}", peer_id);
    }

    /// Mark a connection attempt for a peer
    pub fn mark_connection_attempt(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.discovered_peers.get_mut(peer_id) {
            peer.connection_attempts += 1;
            peer.last_attempt = Some(Instant::now());
        }
    }

    /// Remove a peer from discovery (e.g., when successfully connected)
    pub fn remove_discovered_peer(&mut self, peer_id: &PeerId) {
        self.discovered_peers.remove(peer_id);
    }

    /// Clean up old discovery entries
    fn cleanup_old_discoveries(&mut self) {
        let retention_time = Duration::from_secs(24 * 60 * 60); // 24 hours
        let now = Instant::now();

        self.discovered_peers.retain(|_, peer| {
            now.duration_since(peer.discovered_at) < retention_time
        });
    }

    /// Get statistics about peer discovery
    pub fn get_discovery_stats(&self) -> DiscoveryStats {
        DiscoveryStats {
            discovered_peers: self.discovered_peers.len(),
            bootstrap_peers: self.config.bootstrap_peers.len(),
            last_discovery_duration: self.last_discovery.elapsed(),
        }
    }
}

/// Peer discovery statistics
#[derive(Debug)]
pub struct DiscoveryStats {
    pub discovered_peers: usize,
    pub bootstrap_peers: usize,
    pub last_discovery_duration: Duration,
}

/// Bootstrap node information
#[derive(Debug, Clone)]
pub struct BootstrapNode {
    pub multiaddr: Multiaddr,
    pub peer_id: Option<PeerId>,
    pub description: String,
}

impl BootstrapNode {
    /// Create a new bootstrap node
    pub fn new(multiaddr: Multiaddr, description: String) -> Self {
        Self {
            multiaddr,
            peer_id: None,
            description,
        }
    }

    /// Create a bootstrap node with known peer ID
    pub fn with_peer_id(multiaddr: Multiaddr, peer_id: PeerId, description: String) -> Self {
        Self {
            multiaddr,
            peer_id: Some(peer_id),
            description,
        }
    }
}

/// Bootstrap configuration for different networks
pub struct BootstrapConfig;

impl BootstrapConfig {
    /// Get bootstrap nodes for mainnet
    pub fn mainnet_nodes() -> Vec<BootstrapNode> {
        // In a real implementation, these would be actual mainnet bootstrap nodes
        vec![
            // BootstrapNode::new(
            //     "/ip4/52.23.45.67/tcp/30303".parse().unwrap(),
            //     "Mainnet Bootstrap 1".to_string(),
            // ),
        ]
    }

    /// Get bootstrap nodes for testnet
    pub fn testnet_nodes() -> Vec<BootstrapNode> {
        vec![
            // BootstrapNode::new(
            //     "/ip4/testnet.beacon.com/tcp/30303".parse().unwrap(),
            //     "Testnet Bootstrap 1".to_string(),
            // ),
        ]
    }

    /// Get bootstrap nodes for local development
    pub fn devnet_nodes() -> Vec<BootstrapNode> {
        vec![
            BootstrapNode::new(
                "/ip4/127.0.0.1/tcp/30304".parse().unwrap(),
                "Local Development Node".to_string(),
            ),
        ]
    }
}

/// Peer exchange protocol for discovering peers from connected nodes
pub struct PeerExchange {
    max_peers_per_request: usize,
    exchange_interval: Duration,
    last_exchange: HashMap<PeerId, Instant>,
}

impl PeerExchange {
    /// Create a new peer exchange instance
    pub fn new() -> Self {
        Self {
            max_peers_per_request: 20,
            exchange_interval: Duration::from_secs(300), // 5 minutes
            last_exchange: HashMap::new(),
        }
    }

    /// Check if we should request peers from a given peer
    pub fn should_request_peers(&self, peer_id: &PeerId) -> bool {
        match self.last_exchange.get(peer_id) {
            Some(last_time) => last_time.elapsed() > self.exchange_interval,
            None => true,
        }
    }

    /// Mark that we've requested peers from a peer
    pub fn mark_peer_request(&mut self, peer_id: PeerId) {
        self.last_exchange.insert(peer_id, Instant::now());
    }

    /// Filter and validate received peer addresses
    pub fn validate_peer_addresses(&self, addresses: Vec<Multiaddr>) -> Vec<Multiaddr> {
        addresses
            .into_iter()
            .filter(|addr| self.is_valid_address(addr))
            .take(self.max_peers_per_request)
            .collect()
    }

    /// Check if an address is valid for connection
    fn is_valid_address(&self, addr: &Multiaddr) -> bool {
        // Basic validation - reject obviously invalid addresses
        for component in addr.iter() {
            match component {
                libp2p::multiaddr::Protocol::Ip4(ip) => {
                    if ip.is_loopback() || ip.is_multicast() || ip.is_broadcast() {
                        return false;
                    }
                }
                libp2p::multiaddr::Protocol::Ip6(ip) => {
                    if ip.is_loopback() || ip.is_multicast() {
                        return false;
                    }
                }
                libp2p::multiaddr::Protocol::Tcp(port) => {
                    if port == 0 || port > 65535 {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }

    /// Clean up old exchange records
    pub fn cleanup(&mut self) {
        let cleanup_threshold = self.exchange_interval * 2;
        self.last_exchange.retain(|_, last_time| {
            last_time.elapsed() < cleanup_threshold
        });
    }
}

impl Default for PeerExchange {
    fn default() -> Self {
        Self::new()
    }
}
