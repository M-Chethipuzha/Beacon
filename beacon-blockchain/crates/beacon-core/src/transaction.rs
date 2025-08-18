use serde::{Deserialize, Serialize};
use ed25519_dalek::{Signer, Verifier};
use crate::{TransactionId, Address, Timestamp, Hash};

/// Transaction type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    /// Transfer of value between accounts
    Transfer,
    /// Chaincode deployment
    Deploy,
    /// Chaincode invocation
    Invoke,
    /// System configuration update
    Config,
}

/// Transaction input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    /// The chaincode to invoke
    pub chaincode_id: String,
    /// Function name to call
    pub function: String,
    /// Function arguments
    pub args: Vec<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: TransactionId,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Sender address
    pub from: Address,
    /// Receiver address (optional for some transaction types)
    pub to: Option<Address>,
    /// Transaction input data
    pub input: TransactionInput,
    /// Transaction nonce (to prevent replay attacks)
    pub nonce: u64,
    /// Gas limit for execution
    pub gas_limit: u64,
    /// Gas price
    pub gas_price: u64,
    /// Transaction timestamp
    pub timestamp: Timestamp,
    /// Digital signature
    pub signature: String,
    /// Hash of the transaction
    pub hash: Hash,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        tx_type: TransactionType,
        from: Address,
        to: Option<Address>,
        input: TransactionInput,
        nonce: u64,
    ) -> Self {
        let id = TransactionId::new();
        let timestamp = Timestamp::now();
        
        let mut tx = Self {
            id,
            tx_type,
            from,
            to,
            input,
            nonce,
            gas_limit: 1_000_000, // Default gas limit
            gas_price: 1,         // Default gas price
            timestamp,
            signature: String::new(),
            hash: String::new(),
        };
        
        // Calculate hash
        tx.hash = tx.calculate_hash();
        tx
    }
    
    /// Calculate the hash of the transaction
    pub fn calculate_hash(&self) -> Hash {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_str().as_bytes());
        hasher.update(&bincode::serialize(&self.tx_type).unwrap_or_default());
        hasher.update(self.from.as_str().as_bytes());
        if let Some(ref to) = self.to {
            hasher.update(to.as_str().as_bytes());
        }
        hasher.update(&bincode::serialize(&self.input).unwrap_or_default());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.gas_limit.to_le_bytes());
        hasher.update(&self.gas_price.to_le_bytes());
        hasher.update(&self.timestamp.to_millis().to_le_bytes());
        
        hex::encode(hasher.finalize())
    }
    
    /// Validate the transaction structure
    pub fn validate(&self) -> Result<(), crate::BeaconError> {
        // Check if transaction ID is valid
        if self.id.as_str().is_empty() {
            return Err(crate::BeaconError::InvalidTransaction("Empty transaction ID".to_string()));
        }
        
        // Check if from address is valid
        if self.from.as_str().is_empty() {
            return Err(crate::BeaconError::InvalidTransaction("Empty from address".to_string()));
        }
        
        // Check if chaincode ID is valid for invoke/deploy transactions
        if matches!(self.tx_type, TransactionType::Invoke | TransactionType::Deploy) {
            if self.input.chaincode_id.is_empty() {
                return Err(crate::BeaconError::InvalidTransaction("Empty chaincode ID".to_string()));
            }
        }
        
        // Check if signature is present
        if self.signature.is_empty() {
            return Err(crate::BeaconError::InvalidTransaction("Missing signature".to_string()));
        }
        
        // Verify hash
        if self.hash != self.calculate_hash() {
            return Err(crate::BeaconError::InvalidTransaction("Invalid transaction hash".to_string()));
        }
        
        Ok(())
    }
    
    /// Sign the transaction with a private key
    pub fn sign(&mut self, private_key: &ed25519_dalek::SigningKey) -> Result<(), crate::BeaconError> {
        let message = self.get_signing_data();
        let signature = private_key.sign(&message);
        self.signature = hex::encode(signature.to_bytes());
        Ok(())
    }
    
    /// Get the data that should be signed
    fn get_signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.hash.as_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data.extend_from_slice(&self.timestamp.to_millis().to_le_bytes());
        data
    }
    
    /// Verify the transaction signature
    pub fn verify_signature(&self, public_key: &ed25519_dalek::VerifyingKey) -> bool {
        if let Ok(signature_bytes) = hex::decode(&self.signature) {
            if let Ok(signature) = ed25519_dalek::Signature::try_from(signature_bytes.as_slice()) {
                let message = self.get_signing_data();
                return public_key.verify(&message, &signature).is_ok();
            }
        }
        false
    }
}

/// Transaction execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    /// Transaction that was executed
    pub transaction: Transaction,
    /// Execution status
    pub status: TransactionStatus,
    /// Gas used during execution
    pub gas_used: u64,
    /// Return value from chaincode execution
    pub return_value: Option<Vec<u8>>,
    /// Error message if execution failed
    pub error: Option<String>,
    /// State changes made by the transaction
    pub state_changes: std::collections::HashMap<String, Vec<u8>>,
    /// Events emitted during execution
    pub events: Vec<TransactionEvent>,
}

/// Transaction execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction executed successfully
    Success,
    /// Transaction failed due to execution error
    Failed,
    /// Transaction failed due to insufficient gas
    OutOfGas,
    /// Transaction failed due to validation error
    Invalid,
}

/// Event emitted during transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEvent {
    /// Event name/type
    pub event_type: String,
    /// Event data
    pub data: Vec<u8>,
    /// Topics for event filtering
    pub topics: Vec<String>,
}
