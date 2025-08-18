use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use crate::{BeaconError, BeaconResult};

/// Key pair for digital signatures
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        
        Self {
            signing_key,
            verifying_key,
        }
    }
    
    /// Create key pair from signing key bytes
    pub fn from_bytes(secret_bytes: &[u8]) -> BeaconResult<Self> {
        if secret_bytes.len() != 32 {
            return Err(BeaconError::crypto("Invalid key length, expected 32 bytes"));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(secret_bytes);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }
    
    /// Get the signing key bytes
    pub fn signing_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
    
    /// Get the verifying key bytes
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
    
    /// Get the verifying key as hex string
    pub fn verifying_key_hex(&self) -> String {
        hex::encode(self.verifying_key.to_bytes())
    }
    
    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> String {
        let signature = self.signing_key.sign(message);
        hex::encode(signature.to_bytes())
    }
    
    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature_hex: &str) -> bool {
        verify_signature(&self.verifying_key, message, signature_hex)
    }
}

/// Create a verifying key from hex string
pub fn verifying_key_from_hex(hex_str: &str) -> BeaconResult<VerifyingKey> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| BeaconError::crypto(format!("Invalid hex: {}", e)))?;
    
    if bytes.len() != 32 {
        return Err(BeaconError::crypto("Invalid key length, expected 32 bytes"));
    }
    
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&bytes);
    
    Ok(VerifyingKey::from_bytes(&key_bytes)
        .map_err(|e| BeaconError::crypto(format!("Invalid key: {}", e)))?)
}

/// Verify a signature given a public key, message, and signature
pub fn verify_signature(public_key: &VerifyingKey, message: &[u8], signature_hex: &str) -> bool {
    if let Ok(signature_bytes) = hex::decode(signature_hex) {
        if let Ok(signature) = Signature::try_from(signature_bytes.as_slice()) {
            return public_key.verify(message, &signature).is_ok();
        }
    }
    false
}

/// Hash a message using SHA-256
pub fn hash_message(message: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(message);
    hex::encode(hasher.finalize())
}

/// Hash multiple messages together
pub fn hash_messages(messages: &[&[u8]]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    for message in messages {
        hasher.update(message);
    }
    hex::encode(hasher.finalize())
}

/// Generate a random nonce
pub fn generate_nonce() -> u64 {
    use rand::Rng;
    let mut rng = OsRng;
    rng.r#gen()
}

/// Constant-time comparison for sensitive data
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_pair_generation() {
        let keypair = KeyPair::generate();
        let message = b"test message";
        
        let signature = keypair.sign(message);
        assert!(keypair.verify(message, &signature));
    }

    #[test]
    fn test_key_pair_from_bytes() {
        let original = KeyPair::generate();
        let bytes = original.signing_key_bytes();
        
        let restored = KeyPair::from_bytes(&bytes).unwrap();
        assert_eq!(original.verifying_key_bytes(), restored.verifying_key_bytes());
    }

    #[test]
    fn test_signature_verification() {
        let keypair = KeyPair::generate();
        let message = b"test message";
        let wrong_message = b"wrong message";
        
        let signature = keypair.sign(message);
        
        assert!(keypair.verify(message, &signature));
        assert!(!keypair.verify(wrong_message, &signature));
    }

    #[test]
    fn test_hash_message() {
        let message = b"test message";
        let hash1 = hash_message(message);
        let hash2 = hash_message(message);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
    }
}
