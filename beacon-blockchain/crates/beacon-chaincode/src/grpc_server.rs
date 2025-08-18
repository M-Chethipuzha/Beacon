use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};
use beacon_core::{BeaconResult, BeaconError};
use beacon_storage::StateStorage;

// Include the generated protobuf code
pub mod chaincode {
    tonic::include_proto!("chaincode");
}

use chaincode::{
    chaincode_shim_server::{ChaincodeShim, ChaincodeShimServer},
    *,
};

/// Chaincode execution context that maintains state during a transaction
#[derive(Debug, Clone)]
pub struct ChaincodeContext {
    pub transaction_id: String,
    pub channel_id: String,
    pub creator: Vec<u8>,
    pub timestamp: i64,
    pub chaincode_id: String,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StateChange {
    pub key: String,
    pub value: Vec<u8>,
    pub operation: String, // PUT, DELETE
}

/// The gRPC server that handles chaincode communication
#[derive(Clone)]
pub struct ChaincodeShimService {
    state_storage: Arc<StateStorage>,
    current_context: Arc<RwLock<Option<ChaincodeContext>>>,
    events: Arc<RwLock<Vec<Event>>>,
    state_changes: Arc<RwLock<Vec<StateChange>>>,
}

impl ChaincodeShimService {
    pub fn new(state_storage: Arc<StateStorage>) -> Self {
        Self {
            state_storage,
            current_context: Arc::new(RwLock::new(None)),
            events: Arc::new(RwLock::new(Vec::new())),
            state_changes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set the current execution context
    pub async fn set_context(&self, context: ChaincodeContext) {
        let mut ctx = self.current_context.write().await;
        *ctx = Some(context);
        
        // Clear previous execution results
        let mut events = self.events.write().await;
        events.clear();
        let mut state_changes = self.state_changes.write().await;
        state_changes.clear();
    }

    /// Clear the current execution context
    pub async fn clear_context(&self) {
        let mut ctx = self.current_context.write().await;
        *ctx = None;
    }

    /// Get events and state changes from the current execution
    pub async fn get_execution_results(&self) -> (Vec<Event>, Vec<StateChange>) {
        let events = self.events.read().await;
        let state_changes = self.state_changes.read().await;
        (events.clone(), state_changes.clone())
    }
}

#[tonic::async_trait]
impl ChaincodeShim for ChaincodeShimService {
    async fn get_state(
        &self,
        request: Request<GetStateRequest>,
    ) -> Result<Response<GetStateResponse>, Status> {
        let req = request.into_inner();
        debug!("Getting state for key: {}", req.key);

        match self.state_storage.get_state(&req.key).await {
            Ok(Some(value)) => {
                debug!("Found state for key {}: {} bytes", req.key, value.len());
                Ok(Response::new(GetStateResponse {
                    value,
                    found: true,
                }))
            }
            Ok(None) => {
                debug!("No state found for key: {}", req.key);
                Ok(Response::new(GetStateResponse {
                    value: vec![],
                    found: false,
                }))
            }
            Err(e) => {
                error!("Failed to get state for key {}: {}", req.key, e);
                Err(Status::internal(format!("Failed to get state: {}", e)))
            }
        }
    }

    async fn put_state(
        &self,
        request: Request<PutStateRequest>,
    ) -> Result<Response<PutStateResponse>, Status> {
        let req = request.into_inner();
        debug!("Putting state for key: {} ({} bytes)", req.key, req.value.len());

        // Record the state change
        {
            let mut changes = self.state_changes.write().await;
            changes.push(StateChange {
                key: req.key.clone(),
                value: req.value.clone(),
                operation: "PUT".to_string(),
            });
        }

        match self.state_storage.put_state(&req.key, req.value).await {
            Ok(_) => {
                debug!("Successfully put state for key: {}", req.key);
                Ok(Response::new(PutStateResponse {
                    success: true,
                    error: String::new(),
                }))
            }
            Err(e) => {
                error!("Failed to put state for key {}: {}", req.key, e);
                Ok(Response::new(PutStateResponse {
                    success: false,
                    error: format!("Failed to put state: {}", e),
                }))
            }
        }
    }

    async fn delete_state(
        &self,
        request: Request<DeleteStateRequest>,
    ) -> Result<Response<DeleteStateResponse>, Status> {
        let req = request.into_inner();
        debug!("Deleting state for key: {}", req.key);

        // Record the state change
        {
            let mut changes = self.state_changes.write().await;
            changes.push(StateChange {
                key: req.key.clone(),
                value: vec![],
                operation: "DELETE".to_string(),
            });
        }

        match self.state_storage.delete_state(&req.key).await {
            Ok(_) => {
                debug!("Successfully deleted state for key: {}", req.key);
                Ok(Response::new(DeleteStateResponse {
                    success: true,
                    error: String::new(),
                }))
            }
            Err(e) => {
                error!("Failed to delete state for key {}: {}", req.key, e);
                Ok(Response::new(DeleteStateResponse {
                    success: false,
                    error: format!("Failed to delete state: {}", e),
                }))
            }
        }
    }

    async fn get_state_by_range(
        &self,
        request: Request<GetStateByRangeRequest>,
    ) -> Result<Response<GetStateByRangeResponse>, Status> {
        let req = request.into_inner();
        debug!("Getting state by range: {} to {}", req.start_key, req.end_key);

        match self.state_storage.get_state_range(&req.start_key, &req.end_key).await {
            Ok(results) => {
                let key_values: Vec<KeyValue> = results
                    .into_iter()
                    .map(|(key, value)| KeyValue { key, value })
                    .collect();

                debug!("Found {} results for range query", key_values.len());
                Ok(Response::new(GetStateByRangeResponse {
                    results: key_values,
                    has_more: false, // For simplicity, assume no pagination for now
                    bookmark: String::new(),
                }))
            }
            Err(e) => {
                error!("Failed to get state by range: {}", e);
                Err(Status::internal(format!("Failed to get state by range: {}", e)))
            }
        }
    }

    async fn get_state_by_partial_composite_key(
        &self,
        request: Request<GetStateByPartialCompositeKeyRequest>,
    ) -> Result<Response<GetStateByPartialCompositeKeyResponse>, Status> {
        let req = request.into_inner();
        
        // Build composite key prefix
        let mut prefix = req.object_type;
        for key in req.keys {
            prefix.push('\u{0000}'); // Null separator
            prefix.push_str(&key);
        }
        
        debug!("Getting state by partial composite key: {}", prefix);

        match self.state_storage.get_state_with_prefix(&prefix).await {
            Ok(results) => {
                let key_values: Vec<KeyValue> = results
                    .into_iter()
                    .map(|(key, value)| KeyValue { key, value })
                    .collect();

                debug!("Found {} results for composite key query", key_values.len());
                Ok(Response::new(GetStateByPartialCompositeKeyResponse {
                    results: key_values,
                    has_more: false,
                    bookmark: String::new(),
                }))
            }
            Err(e) => {
                error!("Failed to get state by composite key: {}", e);
                Err(Status::internal(format!("Failed to get state by composite key: {}", e)))
            }
        }
    }

    async fn get_transaction_id(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GetTransactionIdResponse>, Status> {
        let context = self.current_context.read().await;
        match &*context {
            Some(ctx) => {
                debug!("Returning transaction ID: {}", ctx.transaction_id);
                Ok(Response::new(GetTransactionIdResponse {
                    transaction_id: ctx.transaction_id.clone(),
                }))
            }
            None => {
                warn!("No transaction context available");
                Err(Status::failed_precondition("No transaction context"))
            }
        }
    }

    async fn get_channel_id(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GetChannelIdResponse>, Status> {
        let context = self.current_context.read().await;
        match &*context {
            Some(ctx) => {
                debug!("Returning channel ID: {}", ctx.channel_id);
                Ok(Response::new(GetChannelIdResponse {
                    channel_id: ctx.channel_id.clone(),
                }))
            }
            None => {
                warn!("No transaction context available");
                Err(Status::failed_precondition("No transaction context"))
            }
        }
    }

    async fn get_creator(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GetCreatorResponse>, Status> {
        let context = self.current_context.read().await;
        match &*context {
            Some(ctx) => {
                debug!("Returning creator: {} bytes", ctx.creator.len());
                Ok(Response::new(GetCreatorResponse {
                    creator: ctx.creator.clone(),
                }))
            }
            None => {
                warn!("No transaction context available");
                Err(Status::failed_precondition("No transaction context"))
            }
        }
    }

    async fn get_transaction_timestamp(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<GetTransactionTimestampResponse>, Status> {
        let context = self.current_context.read().await;
        match &*context {
            Some(ctx) => {
                debug!("Returning transaction timestamp: {}", ctx.timestamp);
                Ok(Response::new(GetTransactionTimestampResponse {
                    timestamp: ctx.timestamp,
                }))
            }
            None => {
                warn!("No transaction context available");
                Err(Status::failed_precondition("No transaction context"))
            }
        }
    }

    async fn set_event(
        &self,
        request: Request<SetEventRequest>,
    ) -> Result<Response<SetEventResponse>, Status> {
        let req = request.into_inner();
        debug!("Setting event: {} ({} bytes)", req.name, req.payload.len());

        // Add event to collection
        {
            let mut events = self.events.write().await;
            events.push(Event {
                name: req.name.clone(),
                payload: req.payload,
            });
        }

        Ok(Response::new(SetEventResponse {
            success: true,
            error: String::new(),
        }))
    }

    async fn invoke_chaincode(
        &self,
        request: Request<InvokeChaincodeRequest>,
    ) -> Result<Response<InvokeChaincodeResponse>, Status> {
        let req = request.into_inner();
        debug!("Invoking chaincode: {} function: {}", req.chaincode_name, req.function);

        // For now, return a not implemented error
        // In a full implementation, this would invoke another chaincode
        warn!("Cross-chaincode invocation not yet implemented");
        Err(Status::unimplemented("Cross-chaincode invocation not yet implemented"))
    }

    async fn log_message(
        &self,
        request: Request<LogMessageRequest>,
    ) -> Result<Response<LogMessageResponse>, Status> {
        let req = request.into_inner();
        
        // Forward the log message to our logging system
        match req.level {
            0 => debug!("[Chaincode] {}", req.message), // DEBUG
            1 => info!("[Chaincode] {}", req.message),  // INFO
            2 => warn!("[Chaincode] {}", req.message),  // WARN
            3 => error!("[Chaincode] {}", req.message), // ERROR
            _ => info!("[Chaincode] {}", req.message),
        }

        Ok(Response::new(LogMessageResponse { success: true }))
    }
}

/// Chaincode gRPC server
pub struct ChaincodeGrpcServer {
    service: Arc<ChaincodeShimService>,
    server_addr: String,
}

impl ChaincodeGrpcServer {
    pub fn new(state_storage: Arc<StateStorage>, addr: String) -> Self {
        Self {
            service: Arc::new(ChaincodeShimService::new(state_storage)),
            server_addr: addr,
        }
    }

    pub fn get_service(&self) -> Arc<ChaincodeShimService> {
        self.service.clone()
    }

    pub async fn start(&self) -> BeaconResult<()> {
        let addr = self.server_addr.parse()
            .map_err(|e| BeaconError::config(format!("Invalid server address: {}", e)))?;

        info!("Starting chaincode gRPC server on {}", addr);

        let service = ChaincodeShimServer::new((*self.service).clone());

        tonic::transport::Server::builder()
            .add_service(service)
            .serve(addr)
            .await
            .map_err(|e| BeaconError::network(format!("gRPC server error: {}", e)))?;

        Ok(())
    }
}
