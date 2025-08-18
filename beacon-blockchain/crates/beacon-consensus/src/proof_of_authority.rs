use beacon_core::{BeaconResult, Block, Transaction};
use crate::{Consensus, ConsensusState};

/// Proof of Authority consensus implementation
pub struct ProofOfAuthority {
    validators: Vec<String>,
    current_validator_index: usize,
    is_validator: bool,
    node_id: String,
}

impl ProofOfAuthority {
    pub fn new(validators: Vec<String>, node_id: String) -> Self {
        let is_validator = validators.contains(&node_id);
        
        Self {
            validators,
            current_validator_index: 0,
            is_validator,
            node_id,
        }
    }
}

#[async_trait::async_trait]
impl Consensus for ProofOfAuthority {
    async fn validate_block(&self, block: &Block) -> BeaconResult<bool> {
        // Basic PoA validation
        // 1. Check if block was created by a valid validator
        // 2. Check if it's the validator's turn
        // 3. Validate signature
        
        // For now, return true (implement full validation later)
        Ok(true)
    }
    
    async fn create_block(&self, transactions: Vec<Transaction>) -> BeaconResult<Block> {
        if !self.can_create_blocks() {
            return Err(beacon_core::BeaconError::consensus("Node is not a validator"));
        }
        
        // Create a new block with the given transactions
        let block = Block::new(
            0, // This should be actual next block index
            "0".repeat(64), // This should be actual previous block hash
            transactions,
            self.node_id.clone(),
        );
        
        Ok(block)
    }
    
    fn can_create_blocks(&self) -> bool {
        self.is_validator
    }
    
    fn get_state(&self) -> ConsensusState {
        ConsensusState {
            current_validator: self.validators.get(self.current_validator_index).cloned(),
            next_validator: self.validators.get((self.current_validator_index + 1) % self.validators.len()).cloned(),
            validator_count: self.validators.len(),
            is_synced: true, // Simplified for now
        }
    }
}
