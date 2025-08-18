pub mod proof_of_authority;
pub mod validator;
pub mod engine;

pub use proof_of_authority::*;
pub use validator::*;
pub use engine::*;

use beacon_core::{BeaconResult, Block};

/// Consensus trait that all consensus algorithms must implement
#[async_trait::async_trait]
pub trait Consensus: Send + Sync {
    /// Validate a block according to consensus rules
    async fn validate_block(&self, block: &Block) -> BeaconResult<bool>;
    
    /// Create a new block (for validators)
    async fn create_block(&self, transactions: Vec<beacon_core::Transaction>) -> BeaconResult<Block>;
    
    /// Check if this node can create blocks
    fn can_create_blocks(&self) -> bool;
    
    /// Get the current consensus state
    fn get_state(&self) -> ConsensusState;
}

/// Consensus state information
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsensusState {
    pub current_validator: Option<String>,
    pub next_validator: Option<String>,
    pub validator_count: usize,
    pub is_synced: bool,
}
