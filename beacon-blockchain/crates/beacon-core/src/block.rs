use serde::{Deserialize, Serialize};
use ed25519_dalek::{Signer, Verifier};
use crate::{BlockIndex, Hash, Timestamp, Transaction, TransactionResult};

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block number/height in the chain
    pub index: BlockIndex,
    /// Hash of the previous block
    pub previous_hash: Hash,
    /// Merkle root of all transactions in the block
    pub merkle_root: Hash,
    /// Block creation timestamp
    pub timestamp: Timestamp,
    /// Block creator (validator) identifier
    pub validator: String,
    /// Block difficulty (for future PoW support)
    pub difficulty: u64,
    /// Nonce for proof of work (unused in PoA)
    pub nonce: u64,
    /// Block version
    pub version: u32,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        index: BlockIndex,
        previous_hash: Hash,
        transactions: &[Transaction],
        validator: String,
    ) -> Self {
        Self {
            index,
            previous_hash,
            merkle_root: Self::calculate_merkle_root(transactions),
            timestamp: Timestamp::now(),
            validator,
            difficulty: 0, // PoA doesn't use difficulty
            nonce: 0,      // PoA doesn't use nonce
            version: 1,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Calculate merkle root of transactions
    fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
        use sha2::{Sha256, Digest};
        
        if transactions.is_empty() {
            return hex::encode(Sha256::digest(b""));
        }
        
        let mut hashes: Vec<String> = transactions.iter()
            .map(|tx| tx.hash.clone())
            .collect();
        
        // Build merkle tree bottom-up
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0].as_bytes());
                if chunk.len() > 1 {
                    hasher.update(chunk[1].as_bytes());
                } else {
                    // Duplicate last hash if odd number
                    hasher.update(chunk[0].as_bytes());
                }
                next_level.push(hex::encode(hasher.finalize()));
            }
            
            hashes = next_level;
        }
        
        hashes.into_iter().next().unwrap_or_default()
    }
    
    /// Calculate the hash of this block header
    pub fn calculate_hash(&self) -> Hash {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(&self.index.to_le_bytes());
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.merkle_root.as_bytes());
        hasher.update(&self.timestamp.to_millis().to_le_bytes());
        hasher.update(self.validator.as_bytes());
        hasher.update(&self.difficulty.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.version.to_le_bytes());
        
        hex::encode(hasher.finalize())
    }
}

/// Complete block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Transactions in this block
    pub transactions: Vec<Transaction>,
    /// Transaction execution results
    pub transaction_results: Vec<TransactionResult>,
    /// Block hash (calculated from header)
    pub hash: Hash,
}

impl Block {
    /// Create a new block
    pub fn new(
        index: BlockIndex,
        previous_hash: Hash,
        transactions: Vec<Transaction>,
        validator: String,
    ) -> Self {
        let header = BlockHeader::new(index, previous_hash, &transactions, validator);
        let hash = header.calculate_hash();
        
        Self {
            header,
            transactions,
            transaction_results: Vec::new(),
            hash,
        }
    }
    
    /// Create genesis block
    pub fn genesis(network_id: &str) -> Self {
        let mut genesis = Self::new(
            0,
            "0".repeat(64), // Genesis has no previous block
            Vec::new(),
            "genesis".to_string(),
        );
        
        // Add network ID to metadata
        genesis.header.metadata.insert(
            "network_id".to_string(),
            network_id.to_string(),
        );
        
        // Recalculate hash after adding metadata
        genesis.hash = genesis.header.calculate_hash();
        
        genesis
    }
    
    /// Validate the block structure
    pub fn validate(&self) -> Result<(), crate::BeaconError> {
        // Verify block hash
        if self.hash != self.header.calculate_hash() {
            return Err(crate::BeaconError::InvalidBlock("Block hash mismatch".to_string()));
        }
        
        // Verify merkle root
        let calculated_merkle = BlockHeader::calculate_merkle_root(&self.transactions);
        if self.header.merkle_root != calculated_merkle {
            return Err(crate::BeaconError::InvalidBlock("Merkle root mismatch".to_string()));
        }
        
        // Validate all transactions
        for transaction in &self.transactions {
            transaction.validate()?;
        }
        
        // Check if results match transactions
        if !self.transaction_results.is_empty() && 
           self.transaction_results.len() != self.transactions.len() {
            return Err(crate::BeaconError::InvalidBlock(
                "Transaction results count mismatch".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get block size in bytes
    pub fn size(&self) -> usize {
        bincode::serialize(self).map(|data| data.len()).unwrap_or(0)
    }
    
    /// Check if this block is the genesis block
    pub fn is_genesis(&self) -> bool {
        self.header.index == 0
    }
    
    /// Get the number of transactions in this block
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }
    
    /// Sign the block with validator's private key
    pub fn sign(&mut self, private_key: &ed25519_dalek::SigningKey) -> Result<(), crate::BeaconError> {
        let message = self.get_signing_data();
        let signature = private_key.sign(&message);
        self.header.metadata.insert(
            "signature".to_string(),
            hex::encode(signature.to_bytes()),
        );
        
        // Recalculate hash after adding signature
        self.hash = self.header.calculate_hash();
        Ok(())
    }
    
    /// Get the data that should be signed
    fn get_signing_data(&self) -> Vec<u8> {
        // Sign the block hash
        self.hash.as_bytes().to_vec()
    }
    
    /// Verify the block signature
    pub fn verify_signature(&self, public_key: &ed25519_dalek::VerifyingKey) -> bool {
        if let Some(signature_hex) = self.header.metadata.get("signature") {
            if let Ok(signature_bytes) = hex::decode(signature_hex) {
                if let Ok(signature) = ed25519_dalek::Signature::try_from(signature_bytes.as_slice()) {
                    let message = self.get_signing_data();
                    return public_key.verify(&message, &signature).is_ok();
                }
            }
        }
        false
    }
}

/// Block validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockValidationError {
    /// Block hash is invalid
    InvalidHash,
    /// Merkle root doesn't match transactions
    InvalidMerkleRoot,
    /// Block contains invalid transactions
    InvalidTransactions(Vec<String>),
    /// Block signature is invalid
    InvalidSignature,
    /// Block timestamp is invalid
    InvalidTimestamp,
    /// Block size exceeds limit
    BlockTooLarge(usize, usize), // actual, limit
}

impl std::fmt::Display for BlockValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockValidationError::InvalidHash => write!(f, "Invalid block hash"),
            BlockValidationError::InvalidMerkleRoot => write!(f, "Invalid merkle root"),
            BlockValidationError::InvalidTransactions(errors) => {
                write!(f, "Invalid transactions: {}", errors.join(", "))
            }
            BlockValidationError::InvalidSignature => write!(f, "Invalid block signature"),
            BlockValidationError::InvalidTimestamp => write!(f, "Invalid block timestamp"),
            BlockValidationError::BlockTooLarge(actual, limit) => {
                write!(f, "Block too large: {} bytes (limit: {} bytes)", actual, limit)
            }
        }
    }
}

impl std::error::Error for BlockValidationError {}
