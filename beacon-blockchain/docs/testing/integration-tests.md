# Integration Testing Documentation

## Overview

Integration testing for the BEACON platform focuses on testing the interactions between different components, modules, and external systems. This document outlines strategies and implementations for comprehensive integration testing.

## Integration Test Categories

### 1. Component Integration Tests

#### API ↔ Storage Integration

```rust
// tests/integration/api_storage_integration.rs
use beacon_api::handlers;
use beacon_storage::{StateStorage, SQLiteStorage};
use beacon_core::types::{Transaction, Block};
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_api_storage_integration() {
    // Setup storage
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());

    // Initialize test data
    let test_tx = Transaction {
        id: "tx_001".to_string(),
        chaincode_id: "test-contract".to_string(),
        function: "setValue".to_string(),
        args: vec!["key1".to_string(), "value1".to_string()],
        timestamp: chrono::Utc::now(),
        signature: "test_signature".to_string(),
    };

    // Test transaction storage through API
    let result = handlers::submit_transaction(storage.clone(), test_tx.clone()).await;
    assert!(result.is_ok());

    // Verify storage
    let stored_tx = storage.get_transaction(&test_tx.id).await.unwrap();
    assert_eq!(stored_tx.id, test_tx.id);
    assert_eq!(stored_tx.function, test_tx.function);
}

#[tokio::test]
async fn test_state_query_integration() {
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());

    // Store test state
    storage.put_state("test-contract", "key1", "value1", 1).await.unwrap();
    storage.put_state("test-contract", "key2", "value2", 1).await.unwrap();

    // Test API state query
    let result = handlers::query_state(
        storage.clone(),
        "test-contract".to_string(),
        "key1".to_string()
    ).await;

    assert!(result.is_ok());
    let state_value = result.unwrap();
    assert_eq!(state_value, "value1");
}

#[tokio::test]
async fn test_blockchain_info_integration() {
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());

    // Create test blocks
    for i in 0..5 {
        let block = Block {
            number: i,
            hash: format!("hash_{}", i),
            previous_hash: if i == 0 { "genesis".to_string() } else { format!("hash_{}", i-1) },
            timestamp: chrono::Utc::now(),
            transactions: vec![],
        };
        storage.store_block(&block).await.unwrap();
    }

    // Test API blockchain info
    let info = handlers::get_blockchain_info(storage.clone()).await.unwrap();
    assert_eq!(info.latest_block_number, 4);
    assert_eq!(info.total_blocks, 5);
}
```

#### Chaincode ↔ Storage Integration

```rust
// tests/integration/chaincode_storage_integration.rs
use beacon_chaincode::{ChainCodeExecutor, ExecutionContext};
use beacon_storage::{StateStorage, SQLiteStorage};
use std::sync::Arc;

#[tokio::test]
async fn test_chaincode_execution_with_storage() {
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    let executor = ChainCodeExecutor::new();

    // Create execution context
    let mut ctx = ExecutionContext::new(
        "test-contract".to_string(),
        storage.clone(),
        1, // block number
    );

    // Test chaincode execution
    let result = executor.execute(
        &mut ctx,
        "setValue",
        vec!["test_key".to_string(), "test_value".to_string()]
    ).await;

    assert!(result.is_ok());

    // Verify state was stored
    let stored_value = storage.get_state("test-contract", "test_key").await.unwrap();
    assert_eq!(stored_value, Some("test_value".to_string()));
}

#[tokio::test]
async fn test_complex_chaincode_workflow() {
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    let executor = ChainCodeExecutor::new();

    // Setup initial state
    let mut ctx = ExecutionContext::new("supply-chain".to_string(), storage.clone(), 1);

    // Create product
    let create_result = executor.execute(
        &mut ctx,
        "createProduct",
        vec!["PROD001".to_string(), "Laptop".to_string(), "Electronics".to_string()]
    ).await;
    assert!(create_result.is_ok());

    // Update product
    ctx.block_number = 2;
    let update_result = executor.execute(
        &mut ctx,
        "updateProduct",
        vec!["PROD001".to_string(), "location".to_string(), "Warehouse A".to_string()]
    ).await;
    assert!(update_result.is_ok());

    // Query product
    let query_result = executor.execute(
        &mut ctx,
        "getProduct",
        vec!["PROD001".to_string()]
    ).await;
    assert!(query_result.is_ok());

    // Verify state consistency
    let product_data = storage.get_state("supply-chain", "product_PROD001").await.unwrap();
    assert!(product_data.is_some());

    // Check history
    let history = storage.get_state_history("supply-chain", "product_PROD001", 10).await.unwrap();
    assert_eq!(history.len(), 2); // Create + Update
}
```

### 2. Full Stack Integration Tests

#### End-to-End API Workflow

```rust
// tests/integration/e2e_api_tests.rs
use axum::http::StatusCode;
use beacon_api::app::create_app;
use beacon_storage::SQLiteStorage;
use tower::ServiceExt;
use serde_json::json;

#[tokio::test]
async fn test_complete_transaction_workflow() {
    // Setup
    let storage = SQLiteStorage::new(":memory:").await.unwrap();
    let app = create_app(storage).await;

    // 1. Login and get token
    let login_request = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(
            json!({
                "username": "admin",
                "password": "admin123"
            }).to_string()
        )
        .unwrap();

    let login_response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = axum::body::to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
    let login_data: serde_json::Value = serde_json::from_slice(&login_body).unwrap();
    let token = login_data["access_token"].as_str().unwrap();

    // 2. Submit transaction
    let tx_request = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/transactions")
        .header("authorization", format!("Bearer {}", token))
        .header("content-type", "application/json")
        .body(
            json!({
                "chaincode_id": "supply-chain",
                "function": "createProduct",
                "args": ["PROD001", "Test Product", "Electronics"]
            }).to_string()
        )
        .unwrap();

    let tx_response = app.clone().oneshot(tx_request).await.unwrap();
    assert_eq!(tx_response.status(), StatusCode::OK);

    let tx_body = axum::body::to_bytes(tx_response.into_body(), usize::MAX).await.unwrap();
    let tx_data: serde_json::Value = serde_json::from_slice(&tx_body).unwrap();
    let tx_id = tx_data["transaction_id"].as_str().unwrap();

    // 3. Query transaction
    let query_request = axum::http::Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/transactions/{}", tx_id))
        .header("authorization", format!("Bearer {}", token))
        .body("".to_string())
        .unwrap();

    let query_response = app.clone().oneshot(query_request).await.unwrap();
    assert_eq!(query_response.status(), StatusCode::OK);

    // 4. Query state
    let state_request = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/state?key=product_PROD001&chaincode_id=supply-chain")
        .header("authorization", format!("Bearer {}", token))
        .body("".to_string())
        .unwrap();

    let state_response = app.clone().oneshot(state_request).await.unwrap();
    assert_eq!(state_response.status(), StatusCode::OK);

    let state_body = axum::body::to_bytes(state_response.into_body(), usize::MAX).await.unwrap();
    let state_data: serde_json::Value = serde_json::from_slice(&state_body).unwrap();
    assert!(state_data["value"].is_string());
}

#[tokio::test]
async fn test_authentication_and_authorization_flow() {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();
    let app = create_app(storage).await;

    // Test unauthorized access
    let unauth_request = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/blockchain/info")
        .body("".to_string())
        .unwrap();

    let unauth_response = app.clone().oneshot(unauth_request).await.unwrap();
    assert_eq!(unauth_response.status(), StatusCode::UNAUTHORIZED);

    // Test invalid token
    let invalid_request = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/blockchain/info")
        .header("authorization", "Bearer invalid_token")
        .body("".to_string())
        .unwrap();

    let invalid_response = app.clone().oneshot(invalid_request).await.unwrap();
    assert_eq!(invalid_response.status(), StatusCode::UNAUTHORIZED);

    // Test valid authentication (login first)
    // ... (login code from previous test)

    // Test valid authorized request
    // ... (authorized request code)
}
```

### 3. Database Integration Tests

#### Multi-Storage Backend Tests

```rust
// tests/integration/storage_backends.rs
use beacon_storage::{StateStorage, SQLiteStorage, InMemoryStorage};
use beacon_core::types::{Transaction, Block};

async fn test_storage_backend<S: StateStorage>(storage: S) {
    // Test transaction storage
    let tx = Transaction {
        id: "tx_001".to_string(),
        chaincode_id: "test".to_string(),
        function: "test".to_string(),
        args: vec![],
        timestamp: chrono::Utc::now(),
        signature: "sig".to_string(),
    };

    storage.store_transaction(&tx).await.unwrap();
    let retrieved = storage.get_transaction(&tx.id).await.unwrap();
    assert_eq!(retrieved.id, tx.id);

    // Test state operations
    storage.put_state("test", "key1", "value1", 1).await.unwrap();
    let value = storage.get_state("test", "key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    // Test range queries
    storage.put_state("test", "key2", "value2", 1).await.unwrap();
    storage.put_state("test", "key3", "value3", 1).await.unwrap();

    let range_results = storage.get_state_range("test", "key1", "key3", 10).await.unwrap();
    assert_eq!(range_results.len(), 2);

    // Test history
    storage.put_state("test", "key1", "value1_updated", 2).await.unwrap();
    let history = storage.get_state_history("test", "key1", 10).await.unwrap();
    assert_eq!(history.len(), 2);
}

#[tokio::test]
async fn test_sqlite_storage() {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();
    test_storage_backend(storage).await;
}

#[tokio::test]
async fn test_inmemory_storage() {
    let storage = InMemoryStorage::new();
    test_storage_backend(storage).await;
}
```

### 4. Concurrency Integration Tests

#### Concurrent API Access

```rust
// tests/integration/concurrency_tests.rs
use std::sync::Arc;
use tokio::task::JoinSet;
use beacon_api::app::create_app;
use beacon_storage::SQLiteStorage;

#[tokio::test]
async fn test_concurrent_transactions() {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();
    let app = Arc::new(create_app(storage).await);

    let mut join_set = JoinSet::new();

    // Spawn multiple concurrent transaction submissions
    for i in 0..10 {
        let app_clone = app.clone();
        join_set.spawn(async move {
            // Login
            let login_request = create_login_request("admin", "admin123");
            let login_response = app_clone.clone().oneshot(login_request).await.unwrap();
            let token = extract_token_from_response(login_response).await;

            // Submit transaction
            let tx_request = create_transaction_request(
                &token,
                "supply-chain",
                "createProduct",
                vec![format!("PROD{:03}", i), "Product".to_string(), "Category".to_string()]
            );

            let tx_response = app_clone.clone().oneshot(tx_request).await.unwrap();
            assert_eq!(tx_response.status(), StatusCode::OK);

            i
        });
    }

    // Wait for all tasks to complete
    let mut completed = 0;
    while let Some(result) = join_set.join_next().await {
        assert!(result.is_ok());
        completed += 1;
    }

    assert_eq!(completed, 10);
}

#[tokio::test]
async fn test_concurrent_state_access() {
    let storage = Arc::new(SQLiteStorage::new(":memory:").await.unwrap());
    let mut join_set = JoinSet::new();

    // Setup initial state
    storage.put_state("test", "counter", "0", 1).await.unwrap();

    // Spawn concurrent state updates
    for i in 0..5 {
        let storage_clone = storage.clone();
        join_set.spawn(async move {
            // Read current value
            let current = storage_clone.get_state("test", "counter").await.unwrap()
                .unwrap_or("0".to_string());
            let current_num: i32 = current.parse().unwrap_or(0);

            // Update value
            let new_value = current_num + 1;
            storage_clone.put_state("test", "counter", &new_value.to_string(), i + 2).await.unwrap();

            new_value
        });
    }

    // Collect results
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        results.push(result.unwrap());
    }

    // Verify final state
    let final_value = storage.get_state("test", "counter").await.unwrap().unwrap();
    let final_num: i32 = final_value.parse().unwrap();
    assert!(final_num > 0);
}
```

### 5. External System Integration

#### Gateway Integration Tests

```rust
// tests/integration/gateway_integration.rs
use beacon_api::handlers;
use beacon_core::types::Gateway;
use std::collections::HashMap;

#[tokio::test]
async fn test_gateway_registration_flow() {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();

    // Register gateway through API
    let gateway_data = json!({
        "gateway_id": "GW001",
        "ip_address": "192.168.1.100",
        "port": 8080,
        "status": "active"
    });

    let result = handlers::register_gateway(storage.clone(), gateway_data).await;
    assert!(result.is_ok());

    // Verify gateway in storage
    let stored_gateway = storage.get_state("gateway-management", "gateway_GW001").await.unwrap();
    assert!(stored_gateway.is_some());

    // Test gateway status update
    let update_data = json!({
        "gateway_id": "GW001",
        "status": "maintenance"
    });

    let update_result = handlers::update_gateway_status(storage.clone(), update_data).await;
    assert!(update_result.is_ok());
}

#[tokio::test]
async fn test_cross_gateway_communication() {
    // Mock multiple gateways
    let gateway1 = MockGateway::new("GW001", "192.168.1.100");
    let gateway2 = MockGateway::new("GW002", "192.168.1.101");

    // Test message passing
    let message = json!({
        "type": "transaction_sync",
        "data": {
            "transaction_id": "tx_001",
            "source_gateway": "GW001"
        }
    });

    let result = gateway1.send_message_to(&gateway2, message).await;
    assert!(result.is_ok());

    // Verify message received
    let received_messages = gateway2.get_received_messages().await;
    assert_eq!(received_messages.len(), 1);
}
```

### 6. Performance Integration Tests

#### Load Testing

```rust
// tests/integration/performance_tests.rs
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_api_load_performance() {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();
    let app = create_app(storage).await;

    // Warm up
    for _ in 0..10 {
        let request = create_info_request();
        let _ = app.clone().oneshot(request).await;
    }

    // Measure performance under load
    let start_time = Instant::now();
    let mut join_set = JoinSet::new();

    for _ in 0..100 {
        let app_clone = app.clone();
        join_set.spawn(async move {
            let request = create_info_request();
            let response = app_clone.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        });
    }

    // Wait for completion
    while let Some(_) = join_set.join_next().await {}

    let duration = start_time.elapsed();
    let requests_per_second = 100.0 / duration.as_secs_f64();

    println!("Performance: {:.2} requests/second", requests_per_second);
    assert!(requests_per_second > 50.0); // Minimum performance threshold
}

#[tokio::test]
async fn test_storage_performance() {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();

    // Test write performance
    let start_time = Instant::now();
    for i in 0..1000 {
        storage.put_state("test", &format!("key_{}", i), &format!("value_{}", i), 1).await.unwrap();
    }
    let write_duration = start_time.elapsed();

    // Test read performance
    let start_time = Instant::now();
    for i in 0..1000 {
        let _ = storage.get_state("test", &format!("key_{}", i)).await.unwrap();
    }
    let read_duration = start_time.elapsed();

    println!("Write performance: {:.2} ops/second", 1000.0 / write_duration.as_secs_f64());
    println!("Read performance: {:.2} ops/second", 1000.0 / read_duration.as_secs_f64());

    // Performance thresholds
    assert!(write_duration < Duration::from_secs(5));
    assert!(read_duration < Duration::from_secs(2));
}
```

## Test Configuration

### Integration Test Setup

```toml
# tests/Cargo.toml
[package]
name = "beacon-integration-tests"
version = "0.1.0"
edition = "2021"

[dependencies]
beacon-api = { path = "../crates/beacon-api" }
beacon-storage = { path = "../crates/beacon-storage" }
beacon-chaincode = { path = "../crates/beacon-chaincode" }
beacon-core = { path = "../crates/beacon-core" }

tokio = { version = "1.0", features = ["full"] }
axum = { version = "0.7", features = ["headers"] }
tower = { version = "0.4", features = ["util"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
criterion = "0.5"
```

### Test Environment Configuration

```rust
// tests/common/mod.rs
use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup_test_environment() {
    INIT.call_once(|| {
        env_logger::init();

        // Set test environment variables
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("DATABASE_URL", ":memory:");
        std::env::set_var("JWT_SECRET", "test_secret_key");
    });
}

pub async fn create_test_storage() -> Arc<SQLiteStorage> {
    Arc::new(SQLiteStorage::new(":memory:").await.unwrap())
}

pub async fn create_test_app() -> Router {
    let storage = create_test_storage().await;
    create_app(storage).await
}
```

## Test Execution

### Running Integration Tests

```bash
# Run all integration tests
cargo test --test integration --features test-integration

# Run specific test file
cargo test --test api_storage_integration

# Run with output
cargo test --test integration -- --nocapture

# Run performance tests
cargo test --test performance_tests --release

# Run with specific thread count
cargo test --test concurrency_tests -- --test-threads=1
```

### Test Data Management

```rust
// tests/fixtures/mod.rs
pub struct TestFixtures {
    pub users: Vec<User>,
    pub transactions: Vec<Transaction>,
    pub blocks: Vec<Block>,
}

impl TestFixtures {
    pub fn new() -> Self {
        Self {
            users: vec![
                User {
                    username: "admin".to_string(),
                    password: "admin123".to_string(),
                    role: "admin".to_string(),
                },
                User {
                    username: "user".to_string(),
                    password: "user123".to_string(),
                    role: "user".to_string(),
                },
            ],
            transactions: vec![],
            blocks: vec![],
        }
    }

    pub async fn setup_database(&self, storage: Arc<dyn StateStorage>) {
        // Setup test users
        for user in &self.users {
            storage.store_user(user).await.unwrap();
        }

        // Setup test blockchain data
        for block in &self.blocks {
            storage.store_block(block).await.unwrap();
        }
    }
}
```

## Continuous Integration

### GitHub Actions Integration Tests

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Run integration tests
        run: |
          cargo test --test integration --features test-integration

      - name: Run performance tests
        run: |
          cargo test --test performance_tests --release

      - name: Generate test report
        run: |
          cargo test --test integration -- --format json > test-results.json

      - name: Upload test results
        uses: actions/upload-artifact@v3
        with:
          name: integration-test-results
          path: test-results.json
```

This comprehensive integration testing framework ensures all components of the BEACON platform work together correctly and maintains system reliability across different scenarios and loads.
