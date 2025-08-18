use serde::{Deserialize, Serialize};
use ed25519_dalek::{Signer, Verifier};
use beacon_core::{BeaconError, BeaconResult, Block, Transaction};

/// Network protocol version
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Maximum message size (1MB)
pub const MAX_MESSAGE_SIZE: usize = 1_048_576;

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Ping message for connectivity testing
    Ping,
    /// Pong response to ping
    Pong,
    /// New block announcement
    Block(Block),
    /// New transaction announcement
    Transaction(Transaction),
    /// Request for blocks starting from a specific index
    BlockRequest {
        start_index: u64,
        count: u32,
    },
    /// Response containing requested blocks
    BlockResponse {
        blocks: Vec<Block>,
        request_id: String,
    },
    /// Request for specific transaction
    TransactionRequest {
        tx_id: String,
    },
    /// Response containing requested transaction
    TransactionResponse {
        transaction: Option<Transaction>,
        request_id: String,
    },
    /// Peer information exchange
    PeerInfo {
        version: String,
        network_id: String,
        best_block_index: u64,
        peer_count: u32,
    },
    /// Request for peer list
    PeerListRequest,
    /// Response with peer list
    PeerListResponse {
        peers: Vec<String>, // Multiaddr strings
    },
}

/// Protocol message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// Protocol version
    pub version: String,
    /// Message timestamp
    pub timestamp: u64,
    /// Message payload
    pub payload: NetworkMessage,
    /// Message signature (optional)
    pub signature: Option<String>,
}

impl ProtocolMessage {
    /// Create a new protocol message
    pub fn new(payload: NetworkMessage) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            payload,
            signature: None,
        }
    }
    
    /// Sign the message with a private key
    pub fn sign(&mut self, private_key: &ed25519_dalek::SigningKey) -> BeaconResult<()> {
        let message_data = self.get_signing_data()?;
        let signature = private_key.sign(&message_data);
        self.signature = Some(hex::encode(signature.to_bytes()));
        Ok(())
    }
    
    /// Verify the message signature
    pub fn verify_signature(&self, public_key: &ed25519_dalek::VerifyingKey) -> bool {
        if let Some(ref signature_hex) = self.signature {
            if let Ok(signature_bytes) = hex::decode(signature_hex) {
                if let Ok(signature) = ed25519_dalek::Signature::try_from(signature_bytes.as_slice()) {
                    if let Ok(message_data) = self.get_signing_data() {
                        return public_key.verify(&message_data, &signature).is_ok();
                    }
                }
            }
        }
        false
    }
    
    /// Get the data that should be signed
    fn get_signing_data(&self) -> BeaconResult<Vec<u8>> {
        let mut data = Vec::new();
        data.extend_from_slice(self.version.as_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        
        let payload_bytes = bincode::serialize(&self.payload)
            .map_err(|e| BeaconError::serialization(format!("Failed to serialize payload: {}", e)))?;
        data.extend_from_slice(&payload_bytes);
        
        Ok(data)
    }
}

/// Protocol handler for encoding/decoding messages
pub struct ProtocolHandler {
    version: String,
}

impl ProtocolHandler {
    /// Create a new protocol handler
    pub fn new() -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
        }
    }
    
    /// Encode a network message into bytes
    pub fn encode_message(&self, message: &NetworkMessage) -> BeaconResult<Vec<u8>> {
        let protocol_message = ProtocolMessage::new(message.clone());
        let encoded = bincode::serialize(&protocol_message)
            .map_err(|e| BeaconError::serialization(format!("Failed to encode message: {}", e)))?;
        
        if encoded.len() > MAX_MESSAGE_SIZE {
            return Err(BeaconError::network(format!(
                "Message too large: {} bytes (max: {} bytes)",
                encoded.len(),
                MAX_MESSAGE_SIZE
            )));
        }
        
        Ok(encoded)
    }
    
    /// Decode bytes into a network message
    pub fn decode_message(&self, data: &[u8]) -> BeaconResult<NetworkMessage> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(BeaconError::network(format!(
                "Message too large: {} bytes (max: {} bytes)",
                data.len(),
                MAX_MESSAGE_SIZE
            )));
        }
        
        let protocol_message: ProtocolMessage = bincode::deserialize(data)
            .map_err(|e| BeaconError::serialization(format!("Failed to decode message: {}", e)))?;
        
        // Verify protocol version compatibility
        if !self.is_version_compatible(&protocol_message.version) {
            return Err(BeaconError::network(format!(
                "Incompatible protocol version: {} (expected: {})",
                protocol_message.version, self.version
            )));
        }
        
        Ok(protocol_message.payload)
    }
    
    /// Check if a protocol version is compatible
    fn is_version_compatible(&self, version: &str) -> bool {
        // For now, only exact version match
        version == self.version
    }
    
    /// Create a ping message
    pub fn create_ping(&self) -> BeaconResult<Vec<u8>> {
        self.encode_message(&NetworkMessage::Ping)
    }
    
    /// Create a pong message
    pub fn create_pong(&self) -> BeaconResult<Vec<u8>> {
        self.encode_message(&NetworkMessage::Pong)
    }
    
    /// Create a block announcement
    pub fn create_block_announcement(&self, block: Block) -> BeaconResult<Vec<u8>> {
        self.encode_message(&NetworkMessage::Block(block))
    }
    
    /// Create a transaction announcement
    pub fn create_transaction_announcement(&self, transaction: Transaction) -> BeaconResult<Vec<u8>> {
        self.encode_message(&NetworkMessage::Transaction(transaction))
    }
    
    /// Create a block request
    pub fn create_block_request(&self, start_index: u64, count: u32) -> BeaconResult<Vec<u8>> {
        self.encode_message(&NetworkMessage::BlockRequest { start_index, count })
    }
    
    /// Create a block response
    pub fn create_block_response(&self, blocks: Vec<Block>, request_id: String) -> BeaconResult<Vec<u8>> {
        self.encode_message(&NetworkMessage::BlockResponse { blocks, request_id })
    }
    
    /// Create a peer info message
    pub fn create_peer_info(
        &self,
        network_id: String,
        best_block_index: u64,
        peer_count: u32,
    ) -> BeaconResult<Vec<u8>> {
        self.encode_message(&NetworkMessage::PeerInfo {
            version: self.version.clone(),
            network_id,
            best_block_index,
            peer_count,
        })
    }
}

impl Default for ProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Message validation utilities
pub struct MessageValidator;

impl MessageValidator {
    /// Validate a block message
    pub fn validate_block(block: &Block) -> BeaconResult<()> {
        block.validate()?;
        
        // Additional network-specific validation
        if block.size() > MAX_MESSAGE_SIZE {
            return Err(BeaconError::network(format!(
                "Block too large for network transmission: {} bytes",
                block.size()
            )));
        }
        
        Ok(())
    }
    
    /// Validate a transaction message
    pub fn validate_transaction(transaction: &Transaction) -> BeaconResult<()> {
        transaction.validate()?;
        
        // Additional network-specific validation
        let tx_size = bincode::serialize(transaction)
            .map(|data| data.len())
            .unwrap_or(0);
        
        if tx_size > MAX_MESSAGE_SIZE / 10 {
            return Err(BeaconError::network(format!(
                "Transaction too large for network transmission: {} bytes",
                tx_size
            )));
        }
        
        Ok(())
    }
    
    /// Validate message rate limits
    pub fn validate_rate_limit(
        peer_id: &libp2p::PeerId,
        message_type: &str,
        rate_limiter: &mut RateLimiter,
    ) -> BeaconResult<()> {
        if !rate_limiter.allow(peer_id, message_type) {
            return Err(BeaconError::network(format!(
                "Rate limit exceeded for peer {} and message type {}",
                peer_id, message_type
            )));
        }
        Ok(())
    }
}

/// Simple rate limiter for network messages
pub struct RateLimiter {
    limits: std::collections::HashMap<String, (u32, std::time::Duration)>, // message_type -> (max_count, window)
    counters: std::collections::HashMap<(libp2p::PeerId, String), (u32, std::time::Instant)>, // (peer, msg_type) -> (count, window_start)
}

impl RateLimiter {
    /// Create a new rate limiter with default limits
    pub fn new() -> Self {
        let mut limits = std::collections::HashMap::new();
        
        // Default rate limits
        limits.insert("Ping".to_string(), (60, std::time::Duration::from_secs(60))); // 60 pings per minute
        limits.insert("Block".to_string(), (10, std::time::Duration::from_secs(60))); // 10 blocks per minute
        limits.insert("Transaction".to_string(), (100, std::time::Duration::from_secs(60))); // 100 transactions per minute
        
        Self {
            limits,
            counters: std::collections::HashMap::new(),
        }
    }
    
    /// Check if a message is allowed under rate limits
    pub fn allow(&mut self, peer_id: &libp2p::PeerId, message_type: &str) -> bool {
        let (max_count, window) = match self.limits.get(message_type) {
            Some(&limits) => limits,
            None => return true, // No limit for unknown message types
        };
        
        let key = (*peer_id, message_type.to_string());
        let now = std::time::Instant::now();
        
        match self.counters.get_mut(&key) {
            Some((count, window_start)) => {
                if now.duration_since(*window_start) > window {
                    // Reset window
                    *count = 1;
                    *window_start = now;
                    true
                } else if *count < max_count {
                    // Increment counter
                    *count += 1;
                    true
                } else {
                    // Rate limit exceeded
                    false
                }
            }
            None => {
                // First message from this peer for this type
                self.counters.insert(key, (1, now));
                true
            }
        }
    }
    
    /// Clean up old counter entries
    pub fn cleanup(&mut self) {
        let now = std::time::Instant::now();
        let cleanup_threshold = std::time::Duration::from_secs(300); // 5 minutes
        
        self.counters.retain(|(_, message_type), (_, window_start)| {
            if let Some((_, window)) = self.limits.get(message_type) {
                now.duration_since(*window_start) < *window + cleanup_threshold
            } else {
                false
            }
        });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
