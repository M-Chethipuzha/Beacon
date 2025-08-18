// Extended key management functionality
use beacon_core::crypto::KeyPair;
use beacon_core::{BeaconResult, BeaconError};

/// Key store for managing node keys
pub struct KeyStore {
    key_dir: String,
}

impl KeyStore {
    pub fn new(key_dir: String) -> Self {
        Self { key_dir }
    }
    
    pub async fn load_or_generate_keypair(&self, name: &str) -> BeaconResult<KeyPair> {
        let key_path = format!("{}/{}.key", self.key_dir, name);
        
        if std::path::Path::new(&key_path).exists() {
            self.load_keypair(&key_path).await
        } else {
            let keypair = KeyPair::generate();
            self.save_keypair(&keypair, &key_path).await?;
            Ok(keypair)
        }
    }
    
    async fn load_keypair(&self, path: &str) -> BeaconResult<KeyPair> {
        let data = tokio::fs::read(path).await?;
        if data.len() != 32 {
            return Err(BeaconError::crypto("Invalid key file length"));
        }
        KeyPair::from_bytes(&data)
    }
    
    async fn save_keypair(&self, keypair: &KeyPair, path: &str) -> BeaconResult<()> {
        tokio::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).await?;
        tokio::fs::write(path, keypair.signing_key_bytes()).await?;
        Ok(())
    }
}
