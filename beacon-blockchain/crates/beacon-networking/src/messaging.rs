use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use futures::FutureExt;
use tokio::sync::{broadcast, mpsc};
use libp2p::PeerId;
use uuid::Uuid;
use tracing::{debug, info, warn, error};
use beacon_core::{BeaconError, BeaconResult, Block, Transaction};

/// Message types that can be sent through the network
#[derive(Debug, Clone)]
pub enum OutgoingMessage {
    /// Broadcast a block to all peers
    BroadcastBlock(Block),
    /// Broadcast a transaction to all peers
    BroadcastTransaction(Transaction),
    /// Send a direct message to a specific peer
    DirectMessage(PeerId, DirectMessageType),
    /// Request blocks from peers
    RequestBlocks(u64, u32), // start_index, count
    /// Request a specific transaction
    RequestTransaction(String), // transaction_id
}

/// Direct message types for peer-to-peer communication
#[derive(Debug, Clone)]
pub enum DirectMessageType {
    /// Ping a specific peer
    Ping,
    /// Request peer information
    RequestPeerInfo,
    /// Request peer list
    RequestPeerList,
}

/// Incoming message events
#[derive(Debug, Clone)]
pub enum IncomingMessage {
    /// Block received from a peer
    BlockReceived(Block, PeerId),
    /// Transaction received from a peer
    TransactionReceived(Transaction, PeerId),
    /// Ping received from a peer
    PingReceived(PeerId),
    /// Pong received from a peer
    PongReceived(PeerId),
    /// Block response received
    BlockResponseReceived(Vec<Block>, String, PeerId), // blocks, request_id, peer
    /// Transaction response received
    TransactionResponseReceived(Option<Transaction>, String, PeerId), // transaction, request_id, peer
    /// Peer info received
    PeerInfoReceived(PeerInfoData, PeerId),
    /// Peer list received
    PeerListReceived(Vec<String>, PeerId), // multiaddr strings, peer
}

/// Peer information data
#[derive(Debug, Clone)]
pub struct PeerInfoData {
    pub version: String,
    pub network_id: String,
    pub best_block_index: u64,
    pub peer_count: u32,
}

/// Message routing and delivery service
pub struct MessagingService {
    /// Outgoing message queue
    outgoing_queue: VecDeque<(OutgoingMessage, Instant)>,
    /// Pending requests waiting for responses
    pending_requests: HashMap<String, PendingRequest>,
    /// Message delivery statistics
    delivery_stats: DeliveryStats,
    /// Configuration
    config: MessagingConfig,
    /// Event sender for incoming messages
    incoming_sender: broadcast::Sender<IncomingMessage>,
    /// Command receiver for outgoing messages
    command_receiver: mpsc::Receiver<OutgoingMessage>,
}

/// Configuration for messaging service
#[derive(Debug, Clone)]
pub struct MessagingConfig {
    /// Maximum size of outgoing message queue
    pub max_queue_size: usize,
    /// Request timeout duration
    pub request_timeout: Duration,
    /// Maximum number of pending requests
    pub max_pending_requests: usize,
    /// Retry attempts for failed deliveries
    pub max_retry_attempts: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
}

impl Default for MessagingConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            request_timeout: Duration::from_secs(30),
            max_pending_requests: 1000,
            max_retry_attempts: 3,
            retry_delay: Duration::from_secs(5),
        }
    }
}

/// Pending request information
#[derive(Debug)]
struct PendingRequest {
    request_id: String,
    request_type: RequestType,
    peer_id: PeerId,
    created_at: Instant,
    retry_count: u32,
}

/// Types of requests that can be pending
#[derive(Debug)]
enum RequestType {
    BlockRequest { start_index: u64, count: u32 },
    TransactionRequest { tx_id: String },
    PeerInfoRequest,
    PeerListRequest,
}

/// Message delivery statistics
#[derive(Debug, Default)]
pub struct DeliveryStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub requests_sent: u64,
    pub requests_completed: u64,
    pub requests_timed_out: u64,
    pub delivery_failures: u64,
}

impl MessagingService {
    /// Create a new messaging service
    pub fn new(
        config: MessagingConfig,
    ) -> (Self, broadcast::Receiver<IncomingMessage>, mpsc::Sender<OutgoingMessage>) {
        let (incoming_sender, incoming_receiver) = broadcast::channel(10000);
        let (command_sender, command_receiver) = mpsc::channel(1000);

        let service = Self {
            outgoing_queue: VecDeque::new(),
            pending_requests: HashMap::new(),
            delivery_stats: DeliveryStats::default(),
            config,
            incoming_sender,
            command_receiver,
        };

        (service, incoming_receiver, command_sender)
    }

    /// Process the messaging service
    pub async fn run(mut self) -> BeaconResult<()> {
        let mut cleanup_interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            // Check for outgoing messages to process
            self.process_outgoing_queue().await;

            // Check for incoming commands (non-blocking)
            match self.command_receiver.try_recv() {
                Ok(msg) => {
                    if let Err(e) = self.enqueue_outgoing_message(msg).await {
                        error!("Failed to enqueue message: {}", e);
                    }
                },
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No messages available, continue
                },
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    warn!("Command channel closed, shutting down messaging service");
                    break;
                }
            }

            // Check if it's time for cleanup
            if cleanup_interval.tick().now_or_never().is_some() {
                self.cleanup_expired_requests();
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }

    /// Enqueue an outgoing message
    async fn enqueue_outgoing_message(&mut self, message: OutgoingMessage) -> BeaconResult<()> {
        if self.outgoing_queue.len() >= self.config.max_queue_size {
            warn!("Outgoing message queue is full, dropping oldest message");
            self.outgoing_queue.pop_front();
            self.delivery_stats.delivery_failures += 1;
        }

        self.outgoing_queue.push_back((message, Instant::now()));
        debug!("Enqueued outgoing message, queue size: {}", self.outgoing_queue.len());
        Ok(())
    }

    /// Process the outgoing message queue
    async fn process_outgoing_queue(&mut self) {
        while let Some((message, queued_at)) = self.outgoing_queue.pop_front() {
            // Check if message has been in queue too long
            if queued_at.elapsed() > self.config.request_timeout {
                warn!("Message expired in queue, dropping");
                self.delivery_stats.delivery_failures += 1;
                continue;
            }

            if let Err(e) = self.process_outgoing_message(message).await {
                error!("Failed to process outgoing message: {}", e);
                self.delivery_stats.delivery_failures += 1;
            } else {
                self.delivery_stats.messages_sent += 1;
            }

            // Yield control to allow other tasks to run
            tokio::task::yield_now().await;
        }
    }

    /// Process a single outgoing message
    async fn process_outgoing_message(&mut self, message: OutgoingMessage) -> BeaconResult<()> {
        match message {
            OutgoingMessage::BroadcastBlock(block) => {
                debug!("Broadcasting block {}", block.header.index);
                // In a real implementation, this would send to the network layer
                // For now, we'll just log it
                info!("Would broadcast block {} to all peers", block.header.index);
            }
            OutgoingMessage::BroadcastTransaction(transaction) => {
                debug!("Broadcasting transaction {}", transaction.id.as_str());
                info!("Would broadcast transaction {} to all peers", transaction.id.as_str());
            }
            OutgoingMessage::DirectMessage(peer_id, msg_type) => {
                debug!("Sending direct message to peer {}: {:?}", peer_id, msg_type);
                self.process_direct_message(peer_id, msg_type).await?;
            }
            OutgoingMessage::RequestBlocks(start_index, count) => {
                debug!("Requesting {} blocks starting from {}", count, start_index);
                self.send_block_request(start_index, count).await?;
            }
            OutgoingMessage::RequestTransaction(tx_id) => {
                debug!("Requesting transaction {}", tx_id);
                self.send_transaction_request(tx_id).await?;
            }
        }
        Ok(())
    }

    /// Process a direct message to a specific peer
    async fn process_direct_message(&mut self, peer_id: PeerId, msg_type: DirectMessageType) -> BeaconResult<()> {
        match msg_type {
            DirectMessageType::Ping => {
                info!("Would send ping to peer {}", peer_id);
            }
            DirectMessageType::RequestPeerInfo => {
                let request_id = self.create_request(peer_id, RequestType::PeerInfoRequest).await?;
                info!("Would request peer info from {} (request_id: {})", peer_id, request_id);
            }
            DirectMessageType::RequestPeerList => {
                let request_id = self.create_request(peer_id, RequestType::PeerListRequest).await?;
                info!("Would request peer list from {} (request_id: {})", peer_id, request_id);
            }
        }
        Ok(())
    }

    /// Send a block request
    async fn send_block_request(&mut self, start_index: u64, count: u32) -> BeaconResult<()> {
        // For now, just pick the first available peer
        // In a real implementation, we'd select the best peer for this request
        let peer_id = PeerId::random(); // Placeholder

        let request_id = self.create_request(
            peer_id,
            RequestType::BlockRequest { start_index, count }
        ).await?;

        info!("Would send block request to {} (request_id: {})", peer_id, request_id);
        Ok(())
    }

    /// Send a transaction request
    async fn send_transaction_request(&mut self, tx_id: String) -> BeaconResult<()> {
        let peer_id = PeerId::random(); // Placeholder

        let request_id = self.create_request(
            peer_id,
            RequestType::TransactionRequest { tx_id: tx_id.clone() }
        ).await?;

        info!("Would send transaction request for {} to {} (request_id: {})", tx_id, peer_id, request_id);
        Ok(())
    }

    /// Create a new request and track it
    async fn create_request(&mut self, peer_id: PeerId, request_type: RequestType) -> BeaconResult<String> {
        if self.pending_requests.len() >= self.config.max_pending_requests {
            return Err(BeaconError::network("Too many pending requests"));
        }

        let request_id = Uuid::new_v4().to_string();
        let pending_request = PendingRequest {
            request_id: request_id.clone(),
            request_type,
            peer_id,
            created_at: Instant::now(),
            retry_count: 0,
        };

        self.pending_requests.insert(request_id.clone(), pending_request);
        self.delivery_stats.requests_sent += 1;

        Ok(request_id)
    }

    /// Handle an incoming message
    pub async fn handle_incoming_message(&mut self, message: IncomingMessage) -> BeaconResult<()> {
        self.delivery_stats.messages_received += 1;

        match &message {
            IncomingMessage::BlockResponseReceived(_, request_id, _) => {
                if let Some(_) = self.pending_requests.remove(request_id) {
                    self.delivery_stats.requests_completed += 1;
                    debug!("Completed block request {}", request_id);
                }
            }
            IncomingMessage::TransactionResponseReceived(_, request_id, _) => {
                if let Some(_) = self.pending_requests.remove(request_id) {
                    self.delivery_stats.requests_completed += 1;
                    debug!("Completed transaction request {}", request_id);
                }
            }
            _ => {}
        }

        // Forward the message to subscribers
        if let Err(e) = self.incoming_sender.send(message) {
            debug!("No subscribers for incoming message: {}", e);
        }

        Ok(())
    }

    /// Clean up expired requests
    fn cleanup_expired_requests(&mut self) {
        let now = Instant::now();
        let mut expired_requests = Vec::new();

        for (request_id, request) in &self.pending_requests {
            if now.duration_since(request.created_at) > self.config.request_timeout {
                expired_requests.push(request_id.clone());
            }
        }

        for request_id in expired_requests {
            self.pending_requests.remove(&request_id);
            self.delivery_stats.requests_timed_out += 1;
            warn!("Request {} timed out", request_id);
        }
    }

    /// Get messaging statistics
    pub fn get_stats(&self) -> &DeliveryStats {
        &self.delivery_stats
    }

    /// Get the number of pending requests
    pub fn pending_request_count(&self) -> usize {
        self.pending_requests.len()
    }

    /// Get the size of the outgoing queue
    pub fn outgoing_queue_size(&self) -> usize {
        self.outgoing_queue.len()
    }
}

/// Message priority for queue management
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Message with priority information
#[derive(Debug, Clone)]
pub struct PriorityMessage {
    pub message: OutgoingMessage,
    pub priority: MessagePriority,
    pub created_at: Instant,
}

impl PriorityMessage {
    pub fn new(message: OutgoingMessage, priority: MessagePriority) -> Self {
        Self {
            message,
            priority,
            created_at: Instant::now(),
        }
    }
}
