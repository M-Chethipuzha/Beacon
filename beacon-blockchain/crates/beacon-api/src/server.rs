use axum::{
    Router, 
    routing::{get, post},
    middleware,
    extract::State,
    http::StatusCode,
    response::Json,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use beacon_core::BeaconResult;
use beacon_storage::Database;
use beacon_chaincode::ChaincodeExecutor;
use serde_json::Value;

use crate::handlers::{
    health::health_check,
    blockchain::{get_block, get_block_by_hash, get_latest_blocks, get_blockchain_info},
    transactions::{submit_transaction, get_transaction, get_transactions, invoke_chaincode},
    state::{get_state, query_state, get_state_history},
    auth::{login, logout, refresh_token, get_user_info},
};
use crate::middleware::{
    auth_middleware,
    optional_auth_middleware,
    rate_limit_middleware,
    logging_middleware,
    security_headers_middleware,
    cors_middleware,
};

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Database>,
    pub chaincode_executor: Arc<ChaincodeExecutor>,
}

pub struct ApiServer {
    addr: SocketAddr,
    state: AppState,
}

impl ApiServer {
    pub fn new(addr: SocketAddr, storage: Arc<Database>, chaincode_executor: Arc<ChaincodeExecutor>) -> Self {
        let state = AppState {
            storage,
            chaincode_executor,
        };
        Self { addr, state }
    }
    
    pub async fn run(self) -> BeaconResult<()> {
        let addr = self.addr;
        let app = self.create_router();
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("🚀 BEACON API server listening on {}", addr);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
    
    fn create_router(self) -> Router {
        Router::new()
            // Health and info endpoints (no auth required)
            .route("/health", get(health_check))
            .route("/info", get(server_info))
            
            // Authentication endpoints (no auth required)
            .route("/auth/login", post(login))
            .route("/auth/logout", post(logout))
            
            // Public blockchain query endpoints (optional auth for enhanced features)
            .nest("/api/v1", Router::new()
                .route("/blocks/latest", get(get_latest_blocks))
                .route("/blocks/:block_number", get(get_block))
                .route("/blocks/hash/:block_hash", get(get_block_by_hash))
                .route("/blockchain/info", get(get_blockchain_info))
                .route("/transactions/:tx_hash", get(get_transaction))
                .route("/transactions", get(get_transactions))
                .layer(middleware::from_fn_with_state(
                    self.state.clone(), 
                    optional_auth_middleware
                ))
            )
            
            // Protected endpoints requiring authentication
            .nest("/api/v1", Router::new()
                .route("/auth/user", get(get_user_info))
                .route("/auth/refresh", post(refresh_token))
                .route("/transactions/submit", post(submit_transaction))
                .route("/chaincode/invoke", post(invoke_chaincode))
                .route("/state/:key", get(get_state))
                .route("/state/query", post(query_state))
                .route("/state/:key/history", get(get_state_history))
                .layer(middleware::from_fn_with_state(
                    self.state.clone(), 
                    auth_middleware
                ))
            )
            
            // Apply global middleware stack
            .layer(middleware::from_fn_with_state(
                self.state.clone(),
                rate_limit_middleware
            ))
            .layer(middleware::from_fn(logging_middleware))
            .layer(middleware::from_fn(security_headers_middleware))
            .layer(middleware::from_fn(cors_middleware))
            .layer(TraceLayer::new_for_http())
            .with_state(self.state)
    }
}

async fn server_info(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Simplified server info - can be enhanced later
    Ok(Json(serde_json::json!({
        "service": "BEACON Blockchain API",
        "version": "1.0.0",
        "network": "beacon-mainnet",
        "api_version": "v1",
        "features": [
            "transaction_submission",
            "blockchain_queries", 
            "state_queries",
            "chaincode_invocation",
            "authentication",
            "rate_limiting"
        ],
        "statistics": {
            "latest_block_number": 0,
            "total_transactions": 0,
            "uptime": chrono::Utc::now().to_rfc3339()
        },
        "endpoints": {
            "health": "/health",
            "blockchain": "/api/v1/blocks/*",
            "transactions": "/api/v1/transactions/*",
            "state": "/api/v1/state/*",
            "chaincode": "/api/v1/chaincode/*",
            "auth": "/auth/*"
        }
    })))
}
