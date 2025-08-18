use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Unique identifier for blockchain nodes
pub type NodeId = String;

/// Hash type used throughout the system
pub type Hash = String;

/// Block index (height) in the blockchain
pub type BlockIndex = u64;

/// Network identifier for different blockchain networks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkId(pub String);

impl NetworkId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
    
    pub fn mainnet() -> Self {
        Self("beacon_mainnet".to_string())
    }
    
    pub fn testnet() -> Self {
        Self("beacon_testnet".to_string())
    }
    
    pub fn devnet() -> Self {
        Self("beacon_devnet".to_string())
    }
}

/// Timestamp wrapper for consistent time handling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }
    
    pub fn from_millis(millis: i64) -> Self {
        Self(DateTime::from_timestamp_millis(millis).unwrap_or_else(|| Utc::now()))
    }
    
    pub fn to_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

/// Address type for accounts and contracts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Address(pub String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Self(addr.to_string())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Transaction ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransactionId(pub String);

impl TransactionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TransactionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Consensus configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusParams {
    pub block_time: u64,           // milliseconds
    pub block_size_limit: usize,   // bytes
    pub transaction_timeout: u64,  // seconds
    pub validator_rotation_period: u64, // seconds
}

impl Default for ConsensusParams {
    fn default() -> Self {
        Self {
            block_time: 2000,        // 2 seconds
            block_size_limit: 1_048_576, // 1 MB
            transaction_timeout: 300,     // 5 minutes
            validator_rotation_period: 86400, // 24 hours
        }
    }
}

/// State key-value pair
pub type StateKey = String;
pub type StateValue = Vec<u8>;
pub type StateMap = HashMap<StateKey, StateValue>;
