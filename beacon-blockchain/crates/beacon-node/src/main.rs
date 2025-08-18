use clap::Parser;
use tracing::{info, error};
use std::path::PathBuf;

mod node;
mod config;

use node::BeaconNode;
use config::NodeConfig;

#[derive(Parser)]
#[command(name = "beacon-node")]
#[command(about = "BEACON Blockchain Node")]
#[command(version = "0.1.0")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Node identifier
    #[arg(short, long)]
    node_id: Option<String>,

    /// Data directory
    #[arg(short, long)]
    data_dir: Option<PathBuf>,

    /// Listen address for P2P networking
    #[arg(short, long)]
    listen: Option<String>,

    /// Bootstrap peers (can be specified multiple times)
    #[arg(short, long)]
    bootstrap: Vec<String>,

    /// Enable validator mode
    #[arg(long)]
    validator: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Enable metrics
    #[arg(long)]
    metrics: bool,

    /// API port
    #[arg(long, default_value = "8080")]
    api_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(&cli.log_level)?;

    info!("Starting BEACON Blockchain Node v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_config(&cli).await?;
    info!("Loaded configuration for node: {}", config.node.id);

    // Create and start the node
    let mut node = BeaconNode::new(config).await?;
    
    // Handle shutdown gracefully
    let shutdown_result = tokio::select! {
        result = node.run() => {
            match result {
                Ok(_) => {
                    info!("Node shut down normally");
                    Ok(())
                }
                Err(e) => {
                    error!("Node encountered an error: {}", e);
                    Err(e.into())
                }
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
            node.shutdown().await?;
            info!("Node shut down gracefully");
            Ok(())
        }
    };

    shutdown_result
}

fn init_logging(log_level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => return Err("Invalid log level".into()),
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    Ok(())
}

async fn load_config(cli: &Cli) -> Result<NodeConfig, Box<dyn std::error::Error>> {
    let mut config = if let Some(config_path) = &cli.config {
        NodeConfig::from_file(config_path).await?
    } else {
        NodeConfig::default()
    };

    // Override with CLI arguments
    if let Some(node_id) = &cli.node_id {
        config.node.id = node_id.clone();
    }

    if let Some(data_dir) = &cli.data_dir {
        config.node.data_dir = data_dir.to_string_lossy().to_string();
    }

    if let Some(listen_addr) = &cli.listen {
        config.network.listen_addr = listen_addr.parse()?;
    }

    if !cli.bootstrap.is_empty() {
        config.network.bootstrap_peers = cli.bootstrap
            .iter()
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?;
    }

    config.consensus.is_validator = cli.validator;
    config.api.bind_addr = ([0, 0, 0, 0], cli.api_port).into();
    config.monitoring.metrics_enabled = cli.metrics;

    Ok(config)
}
