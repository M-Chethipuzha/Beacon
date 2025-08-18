use thiserror::Error;

/// Main error type for the BEACON blockchain system
#[derive(Error, Debug)]
pub enum BeaconError {
    /// Networking related errors
    #[error("Network error: {0}")]
    Network(String),
    
    /// Consensus related errors
    #[error("Consensus error: {0}")]
    Consensus(String),
    
    /// Storage related errors
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Cryptographic errors
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    /// Transaction validation errors
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    /// Block validation errors
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    
    /// Chaincode execution errors
    #[error("Chaincode error: {0}")]
    Chaincode(String),
    
    /// API related errors
    #[error("API error: {0}")]
    Api(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// I/O errors
    #[error("I/O error: {0}")]
    Io(String),
    
    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Not found errors
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Already exists errors
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    /// Permission denied errors
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl BeaconError {
    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }
    
    /// Create a consensus error
    pub fn consensus(msg: impl Into<String>) -> Self {
        Self::Consensus(msg.into())
    }
    
    /// Create a storage error
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }
    
    /// Create a crypto error
    pub fn crypto(msg: impl Into<String>) -> Self {
        Self::Crypto(msg.into())
    }
    
    /// Create a chaincode error
    pub fn chaincode(msg: impl Into<String>) -> Self {
        Self::Chaincode(msg.into())
    }
    
    /// Create an API error
    pub fn api(msg: impl Into<String>) -> Self {
        Self::Api(msg.into())
    }
    
    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
    
    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }
    
    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }
    
    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// Result type alias for BEACON operations
pub type BeaconResult<T> = Result<T, BeaconError>;

/// Convert from serde_json errors
impl From<serde_json::Error> for BeaconError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for BeaconError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

/// Convert from bincode errors
impl From<bincode::Error> for BeaconError {
    fn from(err: bincode::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}
