use libp2p::{
    gossipsub, identify, kad, mdns, noise, ping, swarm::SwarmEvent, tcp, yamux, Multiaddr, PeerId,
    Swarm, SwarmBuilder,
};
use futures::StreamExt;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};
use beacon_core::{BeaconError, BeaconResult, Block, Transaction};

use crate::{NetworkMessage, PeerInfo, ProtocolHandler};

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addr: Multiaddr,
    pub bootstrap_peers: Vec<Multiaddr>,
    pub max_connections: usize,
    pub network_id: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/30303".parse().unwrap(),
            bootstrap_peers: Vec::new(),
            max_connections: 50,
            network_id: "beacon_devnet".to_string(),
        }
    }
}

/// Network events that can be emitted
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerId, PeerInfo),
    /// Peer disconnected
    PeerDisconnected(PeerId),
    /// New block received from network
    BlockReceived(Block, PeerId),
    /// New transaction received from network
    TransactionReceived(Transaction, PeerId),
    /// Peer discovery update
    PeerDiscovered(PeerId, Vec<Multiaddr>),
    /// Network error occurred
    Error(String),
}

/// Main networking component
pub struct NetworkManager {
    swarm: Swarm<BeaconBehaviour>,
    peers: HashMap<PeerId, PeerInfo>,
    event_sender: broadcast::Sender<NetworkEvent>,
    command_receiver: mpsc::Receiver<NetworkCommand>,
    protocol_handler: ProtocolHandler,
}

/// Commands that can be sent to the network manager
#[derive(Debug)]
pub enum NetworkCommand {
    /// Broadcast a block to all peers
    BroadcastBlock(Block),
    /// Broadcast a transaction to all peers
    BroadcastTransaction(Transaction),
    /// Connect to a specific peer
    ConnectPeer(Multiaddr),
    /// Disconnect from a peer
    DisconnectPeer(PeerId),
    /// Get list of connected peers
    GetPeers(tokio::sync::oneshot::Sender<Vec<PeerInfo>>),
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(
        config: NetworkConfig,
        keypair: libp2p::identity::Keypair,
    ) -> BeaconResult<(Self, broadcast::Receiver<NetworkEvent>, mpsc::Sender<NetworkCommand>)> {
        let local_peer_id = PeerId::from(keypair.public());
        info!("Local peer id: {}", local_peer_id);

        // Create the behavior first
        let behaviour = BeaconBehaviour::new(&keypair, &config.network_id)
            .map_err(|e| BeaconError::network(format!("Failed to create behavior: {}", e)))?;

        // Create the Swarm
        let swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| BeaconError::network(format!("Failed to configure transport: {}", e)))?
            .with_behaviour(|_key| behaviour)
            .map_err(|e| BeaconError::network(format!("Failed to configure behavior: {}", e)))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();

        let (event_sender, event_receiver) = broadcast::channel(1000);
        let (command_sender, command_receiver) = mpsc::channel(100);

        let manager = Self {
            swarm,
            peers: HashMap::new(),
            event_sender,
            command_receiver,
            protocol_handler: ProtocolHandler::new(),
        };

        Ok((manager, event_receiver, command_sender))
    }

    /// Start the network manager
    pub async fn run(mut self) -> BeaconResult<()> {
        // Start listening
        self.swarm
            .listen_on("/ip4/0.0.0.0/tcp/30303".parse().unwrap())
            .map_err(|e| BeaconError::network(format!("Failed to listen: {}", e)))?;

        info!("Network manager started");

        loop {
            tokio::select! {
                event = self.swarm.next() => {
                    if let Some(event) = event {
                        if let Err(e) = self.handle_swarm_event(event).await {
                            error!("Error handling swarm event: {}", e);
                        }
                    }
                }
                command = self.command_receiver.recv() => {
                    match command {
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd).await {
                                error!("Error handling command: {}", e);
                            }
                        }
                        None => {
                            warn!("Command channel closed, shutting down network manager");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle swarm events
    async fn handle_swarm_event(&mut self, event: SwarmEvent<BeaconBehaviourEvent>) -> BeaconResult<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }
            SwarmEvent::Behaviour(event) => {
                self.handle_behaviour_event(event).await?;
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                info!("Connected to peer: {} at {}", peer_id, endpoint.get_remote_address());
                let peer_info = PeerInfo::new(peer_id, vec![endpoint.get_remote_address().clone()]);
                self.peers.insert(peer_id, peer_info.clone());
                let _ = self.event_sender.send(NetworkEvent::PeerConnected(peer_id, peer_info));
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                info!("Disconnected from peer: {} (cause: {:?})", peer_id, cause);
                self.peers.remove(&peer_id);
                let _ = self.event_sender.send(NetworkEvent::PeerDisconnected(peer_id));
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle behaviour events
    async fn handle_behaviour_event(&mut self, event: BeaconBehaviourEvent) -> BeaconResult<()> {
        match event {
            BeaconBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            }) => {
                self.handle_gossip_message(propagation_source, message).await?;
            }
            BeaconBehaviourEvent::Mdns(mdns::Event::Discovered(list)) => {
                for (peer_id, multiaddr) in list {
                    debug!("Discovered peer: {} at {}", peer_id, multiaddr);
                    if let Err(e) = self.swarm.dial(multiaddr.clone()) {
                        warn!("Failed to dial discovered peer {}: {}", peer_id, e);
                    }
                }
            }
            BeaconBehaviourEvent::Identify(identify::Event::Received { peer_id, info }) => {
                debug!("Received identify info from {}: {:?}", peer_id, info);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle gossip messages
    async fn handle_gossip_message(
        &mut self,
        source: PeerId,
        message: gossipsub::Message,
    ) -> BeaconResult<()> {
        match self.protocol_handler.decode_message(&message.data) {
            Ok(NetworkMessage::Block(block)) => {
                debug!("Received block {} from peer {}", block.header.index, source);
                let _ = self.event_sender.send(NetworkEvent::BlockReceived(block, source));
            }
            Ok(NetworkMessage::Transaction(transaction)) => {
                debug!("Received transaction {} from peer {}", transaction.id.as_str(), source);
                let _ = self
                    .event_sender
                    .send(NetworkEvent::TransactionReceived(transaction, source));
            }
            Ok(NetworkMessage::Ping) => {
                debug!("Received ping from peer {}", source);
                // Respond with pong
                if let Ok(pong_data) = self.protocol_handler.encode_message(&NetworkMessage::Pong) {
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(gossipsub::IdentTopic::new("beacon-general"), pong_data)
                    {
                        warn!("Failed to send pong: {}", e);
                    }
                }
            }
            Ok(NetworkMessage::Pong) => {
                debug!("Received pong from peer {}", source);
            }
            Ok(NetworkMessage::BlockRequest { start_index, count }) => {
                debug!("Received block request from peer {}: start={}, count={}", source, start_index, count);
                // TODO: Handle block request
            }
            Ok(NetworkMessage::BlockResponse { blocks, request_id }) => {
                debug!("Received block response from peer {}: {} blocks, request_id={}", source, blocks.len(), request_id);
                // TODO: Handle block response
            }
            Ok(NetworkMessage::TransactionRequest { tx_id }) => {
                debug!("Received transaction request from peer {}: tx_id={}", source, tx_id);
                // TODO: Handle transaction request
            }
            Ok(NetworkMessage::TransactionResponse { transaction, request_id }) => {
                debug!("Received transaction response from peer {}: request_id={}", source, request_id);
                // TODO: Handle transaction response
            }
            Ok(NetworkMessage::PeerInfo { version, network_id, best_block_index, peer_count }) => {
                debug!("Received peer info from peer {}: version={}, network_id={}, best_block_index={}, peer_count={}", 
                       source, version, network_id, best_block_index, peer_count);
                // TODO: Handle peer info
            }
            Ok(NetworkMessage::PeerListRequest) => {
                debug!("Received peer list request from peer {}", source);
                // TODO: Handle peer list request
            }
            Ok(NetworkMessage::PeerListResponse { peers }) => {
                debug!("Received peer list response from peer {}: {} peers", source, peers.len());
                // TODO: Handle peer list response
            }
            Err(e) => {
                warn!("Failed to decode message from peer {}: {}", source, e);
            }
        }
        Ok(())
    }

    /// Handle network commands
    async fn handle_command(&mut self, command: NetworkCommand) -> BeaconResult<()> {
        match command {
            NetworkCommand::BroadcastBlock(block) => {
                debug!("Broadcasting block {}", block.header.index);
                let message = NetworkMessage::Block(block);
                let data = self.protocol_handler.encode_message(&message)?;
                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(gossipsub::IdentTopic::new("beacon-blocks"), data)
                {
                    return Err(BeaconError::network(format!("Failed to broadcast block: {}", e)));
                }
            }
            NetworkCommand::BroadcastTransaction(transaction) => {
                debug!("Broadcasting transaction {}", transaction.id.as_str());
                let message = NetworkMessage::Transaction(transaction);
                let data = self.protocol_handler.encode_message(&message)?;
                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(gossipsub::IdentTopic::new("beacon-transactions"), data)
                {
                    return Err(BeaconError::network(format!(
                        "Failed to broadcast transaction: {}"
                        , e
                    )));
                }
            }
            NetworkCommand::ConnectPeer(addr) => {
                debug!("Connecting to peer at {}", addr);
                if let Err(e) = self.swarm.dial(addr.clone()) {
                    return Err(BeaconError::network(format!(
                        "Failed to connect to peer at {}: {}",
                        addr, e
                    )));
                }
            }
            NetworkCommand::DisconnectPeer(peer_id) => {
                debug!("Disconnecting from peer {}", peer_id);
                self.swarm.disconnect_peer_id(peer_id).ok();
                self.peers.remove(&peer_id);
            }
            NetworkCommand::GetPeers(sender) => {
                let peers: Vec<PeerInfo> = self.peers.values().cloned().collect();
                let _ = sender.send(peers);
            }
        }
        Ok(())
    }

    /// Get the number of connected peers
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Check if a peer is connected
    pub fn is_peer_connected(&self, peer_id: &PeerId) -> bool {
        self.peers.contains_key(peer_id)
    }
}

/// Network behaviour combining multiple protocols
#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct BeaconBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

impl BeaconBehaviour {
    /// Create a new beacon behaviour
    pub fn new(
        keypair: &libp2p::identity::Keypair,
        network_id: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let local_peer_id = PeerId::from(keypair.public());

        // Configure Gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid config");

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )?;

        // Subscribe to topics
        gossipsub.subscribe(&gossipsub::IdentTopic::new("beacon-blocks"))?;
        gossipsub.subscribe(&gossipsub::IdentTopic::new("beacon-transactions"))?;
        gossipsub.subscribe(&gossipsub::IdentTopic::new("beacon-general"))?;

        // Configure mDNS
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Configure Identify
        let identify_config = identify::Config::new(
            format!("beacon/{}", network_id),
            keypair.public(),
        );
        let identify = identify::Behaviour::new(identify_config);

        // Configure Ping
        let ping = ping::Behaviour::default();

        // Configure Kademlia
        let store = kad::store::MemoryStore::new(local_peer_id);
        let kademlia = kad::Behaviour::new(local_peer_id, store);

        Ok(Self {
            gossipsub,
            mdns,
            identify,
            ping,
            kademlia,
        })
    }
}
