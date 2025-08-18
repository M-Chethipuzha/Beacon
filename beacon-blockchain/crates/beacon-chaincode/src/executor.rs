use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use beacon_core::{BeaconError, BeaconResult, Transaction};
use crate::grpc_server::{ChaincodeContext, ChaincodeShimService};

/// Configuration for chaincode execution
#[derive(Debug, Clone)]
pub struct ChaincodeExecutorConfig {
    /// Directory where chaincode binaries are stored
    pub chaincode_dir: PathBuf,
    /// Timeout for chaincode execution
    pub execution_timeout: Duration,
    /// Maximum number of concurrent executions
    pub max_concurrent: usize,
    /// gRPC server address for chaincode communication
    pub grpc_addr: String,
    /// Whether to enable debug logging for chaincode processes
    pub debug_logging: bool,
}

impl Default for ChaincodeExecutorConfig {
    fn default() -> Self {
        Self {
            chaincode_dir: PathBuf::from("./chaincode"),
            execution_timeout: Duration::from_secs(30),
            max_concurrent: 10,
            grpc_addr: "127.0.0.1:9090".to_string(),
            debug_logging: false,
        }
    }
}

/// Information about a running chaincode process
#[derive(Debug)]
struct ChaincodeProcess {
    /// Process handle
    child: Child,
    /// When the process was started
    started_at: Instant,
    /// Transaction ID being executed
    transaction_id: String,
    /// Chaincode ID
    chaincode_id: String,
}

/// Result of chaincode execution
#[derive(Debug, Clone)]
pub struct ChaincodeExecutionResult {
    /// Execution status (0 = success)
    pub status: i32,
    /// Response payload
    pub payload: Vec<u8>,
    /// Response message
    pub message: String,
    /// Events emitted during execution
    pub events: Vec<ChaincodeEvent>,
    /// State changes made during execution
    pub state_changes: Vec<ChaincodeStateChange>,
}

#[derive(Debug, Clone)]
pub struct ChaincodeEvent {
    pub name: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ChaincodeStateChange {
    pub key: String,
    pub value: Vec<u8>,
    pub operation: String, // PUT, DELETE
}

/// Manages chaincode execution in Go subprocesses
pub struct ChaincodeExecutor {
    config: ChaincodeExecutorConfig,
    grpc_service: Arc<ChaincodeShimService>,
    running_processes: Arc<Mutex<HashMap<String, ChaincodeProcess>>>,
    active_executions: Arc<RwLock<usize>>,
}

impl ChaincodeExecutor {
    pub fn new(config: ChaincodeExecutorConfig, grpc_service: Arc<ChaincodeShimService>) -> Self {
        Self {
            config,
            grpc_service,
            running_processes: Arc::new(Mutex::new(HashMap::new())),
            active_executions: Arc::new(RwLock::new(0)),
        }
    }

    /// Execute a chaincode function
    pub async fn execute_chaincode(
        &self,
        transaction: &Transaction,
        creator: Vec<u8>,
    ) -> BeaconResult<ChaincodeExecutionResult> {
        // Check if we're at the concurrency limit
        {
            let active = self.active_executions.read().await;
            if *active >= self.config.max_concurrent {
                return Err(BeaconError::chaincode(
                    "Maximum concurrent chaincode executions reached".to_string(),
                ));
            }
        }

        // Increment active executions count
        {
            let mut active = self.active_executions.write().await;
            *active += 1;
        }

        let execution_id = Uuid::new_v4().to_string();
        
        let result = self.execute_chaincode_internal(transaction, creator, &execution_id).await;

        // Decrement active executions count
        {
            let mut active = self.active_executions.write().await;
            *active = active.saturating_sub(1);
        }

        result
    }

    async fn execute_chaincode_internal(
        &self,
        transaction: &Transaction,
        creator: Vec<u8>,
        execution_id: &str,
    ) -> BeaconResult<ChaincodeExecutionResult> {
        let chaincode_binary = self.find_chaincode_binary(&transaction.input.chaincode_id)?;
        
        info!(
            "Executing chaincode {} function {} for transaction {}",
            transaction.input.chaincode_id, transaction.input.function, transaction.id.as_str()
        );

        // Set up the execution context in the gRPC service
        let context = ChaincodeContext {
            transaction_id: transaction.id.as_str().to_string(),
            channel_id: "beacon".to_string(), // Default channel
            creator,
            timestamp: transaction.timestamp.0.timestamp(),
            chaincode_id: transaction.input.chaincode_id.clone(),
        };

        self.grpc_service.set_context(context).await;

        // Start the chaincode process
        let child = self.start_chaincode_process(&chaincode_binary, transaction, execution_id).await?;

        let process_info = ChaincodeProcess {
            child: child,
            started_at: Instant::now(),
            transaction_id: transaction.id.as_str().to_string(),
            chaincode_id: transaction.input.chaincode_id.clone(),
        };

        // Store the process info
        {
            let mut processes = self.running_processes.lock().await;
            processes.insert(execution_id.to_string(), process_info);
        }

        // Wait for the process to complete with timeout
        let result = timeout(
            self.config.execution_timeout,
            self.wait_for_completion(execution_id),
        ).await;

        // Clean up the process
        self.cleanup_process(execution_id).await;
        
        // Clear the context
        self.grpc_service.clear_context().await;

        match result {
            Ok(Ok(execution_result)) => {
                info!(
                    "Chaincode execution completed: {} status={}",
                    transaction.id.as_str(), execution_result.status
                );
                Ok(execution_result)
            }
            Ok(Err(e)) => {
                error!("Chaincode execution failed: {}", e);
                Err(e)
            }
            Err(_) => {
                error!("Chaincode execution timed out: {}", transaction.id.as_str());
                Err(BeaconError::chaincode("Chaincode execution timed out".to_string()))
            }
        }
    }

    fn find_chaincode_binary(&self, chaincode_id: &str) -> BeaconResult<PathBuf> {
        let binary_name = if cfg!(windows) {
            format!("{}.exe", chaincode_id)
        } else {
            chaincode_id.to_string()
        };

        let binary_path = self.config.chaincode_dir.join(&binary_name);

        if binary_path.exists() && binary_path.is_file() {
            Ok(binary_path)
        } else {
            Err(BeaconError::not_found(format!(
                "Chaincode binary not found: {}",
                binary_path.display()
            )))
        }
    }

    async fn start_chaincode_process(
        &self,
        binary_path: &Path,
        transaction: &Transaction,
        execution_id: &str,
    ) -> BeaconResult<Child> {
        debug!("Starting chaincode process: {}", binary_path.display());

        let mut cmd = Command::new(binary_path);
        
        // Set environment variables
        cmd.env("BEACON_GRPC_ADDRESS", &self.config.grpc_addr)
           .env("BEACON_TRANSACTION_ID", transaction.id.as_str())
           .env("BEACON_CHAINCODE_ID", &transaction.input.chaincode_id)
           .env("BEACON_FUNCTION", &transaction.input.function)
           .env("BEACON_EXECUTION_ID", execution_id);

        // Pass function arguments as command line arguments
        cmd.args(&transaction.input.args);

        // Configure stdio
        cmd.stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        // Set working directory to chaincode directory
        cmd.current_dir(&self.config.chaincode_dir);

        // Spawn the process
        cmd.spawn()
            .map_err(|e| BeaconError::chaincode(format!("Failed to start chaincode process: {}", e)))
    }

    async fn wait_for_completion(&self, execution_id: &str) -> BeaconResult<ChaincodeExecutionResult> {
        // Wait for the process to exit
        let exit_status = {
            let mut processes = self.running_processes.lock().await;
            if let Some(process) = processes.get_mut(execution_id) {
                let status = process.child.wait().await
                    .map_err(|e| BeaconError::chaincode(format!("Process wait failed: {}", e)))?;
                
                status.code().unwrap_or(-1)
            } else {
                return Err(BeaconError::chaincode("Process not found".to_string()));
            }
        };

        // Collect events and state changes from the gRPC service
        let (events, state_changes) = self.grpc_service.get_execution_results().await;

        // Convert to our result types
        let events: Vec<ChaincodeEvent> = events.into_iter()
            .map(|e| ChaincodeEvent {
                name: e.name,
                payload: e.payload,
            })
            .collect();

        let state_changes: Vec<ChaincodeStateChange> = state_changes.into_iter()
            .map(|sc| ChaincodeStateChange {
                key: sc.key,
                value: sc.value,
                operation: sc.operation,
            })
            .collect();

        Ok(ChaincodeExecutionResult {
            status: exit_status,
            payload: vec![], // For now, we don't capture stdout as payload
            message: if exit_status == 0 { "Success".to_string() } else { "Failed".to_string() },
            events,
            state_changes,
        })
    }

    async fn cleanup_process(&self, execution_id: &str) {
        let mut processes = self.running_processes.lock().await;
        
        if let Some(mut process) = processes.remove(execution_id) {
            // Try to kill the process if it's still running
            if let Ok(None) = process.child.try_wait() {
                warn!("Killing chaincode process: {}", execution_id);
                let _ = process.child.kill().await;
            }
        }
    }

    /// Get information about currently running processes
    pub async fn get_running_processes(&self) -> Vec<String> {
        let processes = self.running_processes.lock().await;
        processes.keys().cloned().collect()
    }

    /// Get the number of active executions
    pub async fn get_active_count(&self) -> usize {
        *self.active_executions.read().await
    }

    /// Cleanup expired processes (processes that have been running too long)
    pub async fn cleanup_expired_processes(&self) {
        let mut processes = self.running_processes.lock().await;
        let mut to_remove = Vec::new();

        for (execution_id, process) in processes.iter_mut() {
            if process.started_at.elapsed() > self.config.execution_timeout {
                warn!("Killing expired chaincode process: {}", execution_id);
                let _ = process.child.kill().await;
                to_remove.push(execution_id.clone());
            }
        }

        for execution_id in to_remove {
            processes.remove(&execution_id);
        }
    }
}
