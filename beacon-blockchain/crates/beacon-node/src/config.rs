use serde::{Deserialize, Serialize};
use std::path::Path;
use std::net::SocketAddr;
use libp2p::Multiaddr;
use beacon_core::{BeaconError, BeaconResult, ConsensusParams};

/// Complete node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node: NodeSettings,
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub storage: StorageConfig,
    pub api: ApiConfig,
    pub chaincode: ChaincodeConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

/// Node-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSettings {
    /// Unique node identifier
    pub id: String,
    /// Data directory path
    pub data_dir: String,
    /// Log level
    pub log_level: String,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Address to listen on for P2P connections
    pub listen_addr: Multiaddr,
    /// Bootstrap peers to connect to initially
    pub bootstrap_peers: Vec<Multiaddr>,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Network identifier
    pub network_id: String,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus type (currently only PoA)
    pub consensus_type: String,
    /// Whether this node is a validator
    pub is_validator: bool,
    /// List of validator public keys
    pub validators: Vec<String>,
    /// Consensus parameters
    pub params: ConsensusParams,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage engine (currently only RocksDB)
    pub engine: String,
    /// Cache size in MB
    pub cache_size: usize,
    /// Write buffer size in MB
    pub write_buffer_size: usize,
    /// Maximum number of open files
    pub max_open_files: i32,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Whether the API is enabled
    pub enabled: bool,
    /// Address to bind the API server
    pub bind_addr: SocketAddr,
    /// CORS origins
    pub cors_origins: Vec<String>,
    /// Rate limit (requests per minute)
    pub rate_limit: u32,
}

/// Chaincode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaincodeConfig {
    /// gRPC server address for chaincode communication
    pub grpc_addr: SocketAddr,
    /// Execution timeout in seconds
    pub execution_timeout: u64,
    /// Maximum concurrent executions
    pub max_concurrent: usize,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// TLS certificate file path
    pub tls_cert: Option<String>,
    /// TLS key file path
    pub tls_key: Option<String>,
    /// Validator private key file path
    pub validator_key: Option<String>,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Whether metrics are enabled
    pub metrics_enabled: bool,
    /// Metrics server address
    pub metrics_addr: SocketAddr,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node: NodeSettings {
                id: "beacon_node_001".to_string(),
                data_dir: "./beacon_data".to_string(),
                log_level: "info".to_string(),
            },
            network: NetworkConfig {
                listen_addr: "/ip4/0.0.0.0/tcp/30303".parse().unwrap(),
                bootstrap_peers: Vec::new(),
                max_connections: 50,
                network_id: "beacon_devnet".to_string(),
            },
            consensus: ConsensusConfig {
                consensus_type: "proof_of_authority".to_string(),
                is_validator: false,
                validators: Vec::new(),
                params: ConsensusParams::default(),
            },
            storage: StorageConfig {
                engine: "rocksdb".to_string(),
                cache_size: 256,
                write_buffer_size: 64,
                max_open_files: 1000,
            },
            api: ApiConfig {
                enabled: true,
                bind_addr: ([0, 0, 0, 0], 8080).into(),
                cors_origins: vec!["*".to_string()],
                rate_limit: 1000,
            },
            chaincode: ChaincodeConfig {
                grpc_addr: ([127, 0, 0, 1], 9090).into(),
                execution_timeout: 30,
                max_concurrent: 10,
            },
            security: SecurityConfig {
                tls_cert: None,
                tls_key: None,
                validator_key: None,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: false,
                metrics_addr: ([0, 0, 0, 0], 9091).into(),
            },
        }
    }
}

impl NodeConfig {
    /// Load configuration from a TOML file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> BeaconResult<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| BeaconError::config(format!("Failed to read config file: {}", e)))?;

        let config: NodeConfig = toml::from_str(&content)
            .map_err(|e| BeaconError::config(format!("Failed to parse config file: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> BeaconResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| BeaconError::config(format!("Failed to serialize config: {}", e)))?;

        tokio::fs::write(path, content)
            .await
            .map_err(|e| BeaconError::config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> BeaconResult<()> {
        // Validate node ID
        if self.node.id.is_empty() {
            return Err(BeaconError::config("Node ID cannot be empty"));
        }

        // Validate data directory
        if self.node.data_dir.is_empty() {
            return Err(BeaconError::config("Data directory cannot be empty"));
        }

        // Validate network configuration
        if self.network.max_connections == 0 {
            return Err(BeaconError::config("Max connections must be greater than 0"));
        }

        // Validate consensus configuration
        if self.consensus.is_validator && self.consensus.validators.is_empty() {
            return Err(BeaconError::config("Validator node must have validator list"));
        }

        // Validate storage configuration
        if self.storage.cache_size == 0 {
            return Err(BeaconError::config("Cache size must be greater than 0"));
        }

        // Validate chaincode configuration
        if self.chaincode.execution_timeout == 0 {
            return Err(BeaconError::config("Chaincode execution timeout must be greater than 0"));
        }

        if self.chaincode.max_concurrent == 0 {
            return Err(BeaconError::config("Max concurrent executions must be greater than 0"));
        }

        Ok(())
    }

    /// Get the path to the database directory
    pub fn database_path(&self) -> String {
        format!("{}/db", self.node.data_dir)
    }

    /// Get the path to the keys directory
    pub fn keys_path(&self) -> String {
        format!("{}/keys", self.node.data_dir)
    }

    /// Get the path to the logs directory
    pub fn logs_path(&self) -> String {
        format!("{}/logs", self.node.data_dir)
    }

    /// Create all necessary directories
    pub async fn create_directories(&self) -> BeaconResult<()> {
        let database_path = self.database_path();
        let keys_path = self.keys_path();
        let logs_path = self.logs_path();
        
        let dirs = vec![
            &self.node.data_dir,
            &database_path,
            &keys_path,
            &logs_path,
        ];

        for dir in dirs {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|e| BeaconError::config(format!("Failed to create directory {}: {}", dir, e)))?;
        }

        Ok(())
    }

    /// Generate a sample configuration file
    pub fn generate_sample() -> String {
        let config = Self::default();
        toml::to_string_pretty(&config).unwrap_or_else(|_| "# Failed to generate sample config".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_config_serialization() {
        let config = NodeConfig::default();
        
        // Test TOML serialization
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: NodeConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.node.id, deserialized.node.id);
        assert_eq!(config.network.network_id, deserialized.network.network_id);
    }

    #[tokio::test]
    async fn test_config_file_operations() {
        let config = NodeConfig::default();
        
        // Create temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        
        // Save and load config
        config.to_file(temp_path).await.unwrap();
        let loaded_config = NodeConfig::from_file(temp_path).await.unwrap();
        
        assert_eq!(config.node.id, loaded_config.node.id);
    }

    #[test]
    fn test_config_validation() {
        let mut config = NodeConfig::default();
        
        // Valid config should pass
        assert!(config.validate().is_ok());
        
        // Invalid node ID should fail
        config.node.id = String::new();
        assert!(config.validate().is_err());
    }
}
