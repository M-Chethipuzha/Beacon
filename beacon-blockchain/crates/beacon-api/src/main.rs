use beacon_api::ApiServer;
use beacon_storage::{Database, DatabaseConfig, StateStorage};
use beacon_chaincode::{ChaincodeExecutor, ChaincodeExecutorConfig, ChaincodeShimService};
use std::{net::SocketAddr, sync::Arc, time::Duration, path::PathBuf};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter("info,beacon_api=debug,beacon_storage=info,beacon_chaincode=info")
        .init();

    tracing::info!("🚀 Starting BEACON API Server...");

    // Initialize database with configuration
    let db_config = DatabaseConfig {
        path: "./beacon_data".to_string(),
        enable_compression: true,
        cache_size: 256, // 256 MB
        write_buffer_size: 64, // 64 MB
        max_open_files: 1000,
        enable_statistics: true,
    };

    tracing::info!("📁 Initializing database at: {}", db_config.path);
    let database = Arc::new(Database::open(db_config)?);
    tracing::info!("✅ Database initialized successfully");

    // Initialize state storage for chaincode operations
    let state_storage = Arc::new(StateStorage::new(database.clone()));
    tracing::info!("✅ State storage initialized");

    // Initialize chaincode executor configuration
    let executor_config = ChaincodeExecutorConfig {
        chaincode_dir: PathBuf::from("./chaincodes"),
        execution_timeout: Duration::from_secs(30),
        max_concurrent: 10,
        grpc_addr: "127.0.0.1:9999".to_string(),
        debug_logging: true,
    };

    // Create chaincode GRPC service and executor
    let grpc_service = Arc::new(ChaincodeShimService::new(state_storage));
    let chaincode_executor = Arc::new(ChaincodeExecutor::new(executor_config, grpc_service));
    tracing::info!("✅ Chaincode executor initialized");

    // Parse server address
    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    
    // Create and configure the API server
    let server = ApiServer::new(addr, database, chaincode_executor);
    
    // Print startup information
    println!("🌟 ========================================");
    println!("🚀 BEACON Blockchain API Server");
    println!("🌟 ========================================");
    println!("🌐 Server Address: http://{}", addr);
    println!("📋 API Documentation: http://{}/docs", addr);
    println!("🔍 Health Check: http://{}/health", addr);
    println!("ℹ️  Server Info: http://{}/info", addr);
    println!("🔐 Authentication: POST http://{}/auth/login", addr);
    println!("🌟 ========================================");
    println!();
    
    // Example usage information
    println!("📖 Quick Start:");
    println!("   1. Check health: curl http://{}/health", addr);
    println!("   2. Get info: curl http://{}/info", addr);
    println!("   3. Login: curl -X POST http://{}/auth/login \\", addr);
    println!("             -H 'Content-Type: application/json' \\");
    println!("             -d '{{\"username\": \"admin\", \"password\": \"admin123\"}}'");
    println!();

    tracing::info!("🌐 Starting API server on {}", addr);
    
    // Start the server
    if let Err(e) = server.run().await {
        tracing::error!("❌ Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
