use crate::config::NodeConfig;
use beacon_core::{BeaconResult};
use beacon_storage::{Database, DatabaseConfig, BlockchainStorage, StateStorage, TransactionStorage};
use beacon_api::ApiServer;
use beacon_consensus::{ProofOfAuthority, Consensus};
use beacon_crypto::KeyStore;
use beacon_chaincode::{ChaincodeExecutor, ChaincodeExecutorConfig, ChaincodeShimService};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, error, debug};

/// Main BEACON blockchain node
pub struct BeaconNode {
    config: NodeConfig,
    database: Arc<Database>,
    blockchain_storage: Arc<BlockchainStorage>,
    state_storage: Arc<StateStorage>,
    transaction_storage: Arc<TransactionStorage>,
    consensus: Arc<dyn Consensus>,
    key_store: KeyStore,
    chaincode_executor: Arc<ChaincodeExecutor>,
    shutdown_sender: broadcast::Sender<()>,
}

impl BeaconNode {
    /// Create a new BEACON node
    pub async fn new(config: NodeConfig) -> BeaconResult<Self> {
        info!("Initializing BEACON node: {}", config.node.id);

        // Create necessary directories
        config.create_directories().await?;

        // Initialize database
        let db_config = DatabaseConfig {
            path: config.database_path(),
            cache_size: config.storage.cache_size,
            write_buffer_size: config.storage.write_buffer_size,
            max_open_files: config.storage.max_open_files,
            ..Default::default()
        };
        let database = Arc::new(Database::open(db_config)?);

        // Initialize storage layers
        let blockchain_storage = Arc::new(BlockchainStorage::new(database.clone()));
        let state_storage = Arc::new(StateStorage::new(database.clone()));
        let transaction_storage = Arc::new(TransactionStorage::new(database.clone()));

        // Initialize blockchain with genesis block if needed
        blockchain_storage.initialize(&config.network.network_id).await?;

        // Initialize consensus
        let consensus: Arc<dyn Consensus> = Arc::new(ProofOfAuthority::new(
            config.consensus.validators.clone(),
            config.node.id.clone(),
        ));

        // Initialize key store
        let key_store = KeyStore::new(config.keys_path());

        // Initialize chaincode services
        let chaincode_shim_service = Arc::new(ChaincodeShimService::new(state_storage.clone()));
        let chaincode_config = ChaincodeExecutorConfig::default();
        let chaincode_executor = Arc::new(ChaincodeExecutor::new(chaincode_config, chaincode_shim_service));

        // Create shutdown channel
        let (shutdown_sender, _) = broadcast::channel(1);

        let node = Self {
            config,
            database,
            blockchain_storage,
            state_storage,
            transaction_storage,
            consensus,
            key_store,
            chaincode_executor,
            shutdown_sender,
        };

        info!("BEACON node initialized successfully");
        Ok(node)
    }

    /// Start the node and all its services
    pub async fn run(&mut self) -> BeaconResult<()> {
        info!("Starting BEACON node services");

        let mut shutdown_receiver = self.shutdown_sender.subscribe();

        // Start API server if enabled
        let api_handle = if self.config.api.enabled {
            let api_server = ApiServer::new(
                self.config.api.bind_addr,
                self.database.clone(),
                self.chaincode_executor.clone()
            );
            Some(tokio::spawn(async move {
                if let Err(e) = api_server.run().await {
                    error!("API server error: {}", e);
                }
            }))
        } else {
            None
        };

        // Start consensus engine
        let consensus_handle = {
            let consensus = self.consensus.clone();
            tokio::spawn(async move {
                debug!("Consensus engine started");
                // Consensus engine would run here
                // For now, just wait for shutdown
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            })
        };

        // Start networking layer
        let networking_handle = tokio::spawn(async move {
            debug!("Networking layer started");
            // P2P networking would run here
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        // Start background maintenance tasks
        let maintenance_handle = {
            let database = self.database.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1 hour
                loop {
                    interval.tick().await;
                    debug!("Running database maintenance");
                    if let Err(e) = database.maintenance().await {
                        error!("Database maintenance error: {}", e);
                    }
                }
            })
        };

        info!("All services started, node is running");

        // Wait for shutdown signal
        tokio::select! {
            _ = shutdown_receiver.recv() => {
                info!("Received shutdown signal");
            }
            result = consensus_handle => {
                if let Err(e) = result {
                    error!("Consensus engine error: {}", e);
                }
            }
            result = networking_handle => {
                if let Err(e) = result {
                    error!("Networking layer error: {}", e);
                }
            }
            result = maintenance_handle => {
                if let Err(e) = result {
                    error!("Maintenance task error: {}", e);
                }
            }
        }

        // Shutdown API server
        if let Some(handle) = api_handle {
            handle.abort();
        }

        info!("BEACON node stopped");
        Ok(())
    }

    /// Gracefully shutdown the node
    pub async fn shutdown(&mut self) -> BeaconResult<()> {
        info!("Shutting down BEACON node");

        // Send shutdown signal to all services
        if let Err(e) = self.shutdown_sender.send(()) {
            debug!("No services listening for shutdown signal: {}", e);
        }

        // Perform any cleanup tasks
        self.database.maintenance().await?;

        info!("BEACON node shutdown complete");
        Ok(())
    }

    /// Get node status information
    pub async fn get_status(&self) -> BeaconResult<NodeStatus> {
        let blockchain_stats = self.blockchain_storage.get_stats().await?;
        let consensus_state = self.consensus.get_state();
        
        Ok(NodeStatus {
            node_id: self.config.node.id.clone(),
            network_id: self.config.network.network_id.clone(),
            is_validator: self.config.consensus.is_validator,
            blockchain_stats,
            consensus_state,
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Submit a transaction to the node
    pub async fn submit_transaction(&self, transaction: beacon_core::Transaction) -> BeaconResult<()> {
        debug!("Submitting transaction: {}", transaction.id.as_str());

        // Validate transaction
        transaction.validate()?;

        // Store transaction (as pending)
        self.transaction_storage.store_transaction(&transaction).await?;

        // Forward to consensus layer for inclusion in next block
        // This would be handled by the consensus engine in a real implementation

        info!("Transaction {} submitted successfully", transaction.id.as_str());
        Ok(())
    }

    /// Get blockchain information
    pub async fn get_blockchain_info(&self) -> BeaconResult<beacon_storage::BlockchainStats> {
        self.blockchain_storage.get_stats().await
    }

    /// Get a block by index
    pub async fn get_block(&self, index: u64) -> BeaconResult<Option<beacon_core::Block>> {
        self.blockchain_storage.get_block_by_index(index).await
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, tx_id: &beacon_core::TransactionId) -> BeaconResult<Option<beacon_core::Transaction>> {
        self.transaction_storage.get_transaction(tx_id).await
    }

    /// Get state value
    pub async fn get_state(&self, key: &str) -> BeaconResult<Option<Vec<u8>>> {
        let key_string = key.to_string();
        self.state_storage.get_state(&key_string).await
    }
}

/// Node status information
#[derive(Debug, serde::Serialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub network_id: String,
    pub is_validator: bool,
    pub blockchain_stats: beacon_storage::BlockchainStats,
    pub consensus_state: beacon_consensus::ConsensusState,
    pub uptime: u64,
}
