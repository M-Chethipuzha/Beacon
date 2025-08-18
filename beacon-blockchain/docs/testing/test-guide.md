# BEACON Testing Guide

## Overview

This guide covers the comprehensive testing strategy for the BEACON blockchain platform, including unit tests, integration tests, API tests, and performance testing.

## Testing Philosophy

BEACON employs a multi-layered testing approach:

1. **Unit Tests** - Test individual components in isolation
2. **Integration Tests** - Test component interactions and workflows
3. **API Tests** - Test REST API endpoints and authentication
4. **Performance Tests** - Test system performance and scalability
5. **Chaincode Tests** - Test Go chaincode functionality

## Test Structure

```
tests/
├── integration/           # Integration test suites
│   ├── api_integration.rs
│   ├── chaincode_integration.rs
│   └── storage_integration.rs
├── performance/           # Performance and load tests
│   ├── api_load_test.rs
│   └── chaincode_perf_test.rs
├── fixtures/              # Test data and fixtures
│   ├── test_data.json
│   └── mock_chaincodes/
└── utils/                 # Test utilities and helpers
    ├── test_helpers.rs
    └── mock_services.rs
```

## Running Tests

### Prerequisites

```bash
# Install test dependencies
cargo install cargo-nextest
cargo install cargo-tarpaulin  # For coverage

# Start test database (if needed)
docker run -d --name test-rocks -p 5432:5432 rocksdb-test
```

### Unit Tests

```bash
# Run all unit tests
cargo test

# Run tests for specific crate
cargo test -p beacon-core
cargo test -p beacon-api
cargo test -p beacon-storage

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_block_validation
```

### Integration Tests

```bash
# Run all integration tests
cargo test --test integration

# Run specific integration test suite
cargo test --test api_integration
cargo test --test chaincode_integration

# Run with logging
RUST_LOG=debug cargo test --test integration
```

### API Tests

```bash
# Start the API server first
cargo run --bin beacon-api &

# Run API test scripts
./test_api.sh        # Linux/Mac
./test_api.ps1       # Windows

# Run automated API tests
cargo test --test api_integration
```

### Performance Tests

```bash
# Run performance test suite
cargo test --test performance --release

# Run load tests with specific parameters
LOAD_TEST_DURATION=60 CONCURRENT_USERS=100 cargo test --test api_load_test
```

### Test Coverage

```bash
# Generate coverage report
cargo tarpaulin --out html --output-dir coverage/

# View coverage
open coverage/tarpaulin-report.html
```

## Test Categories

### 1. Unit Tests

#### Core Components

- **Blocks**: Block creation, validation, serialization
- **Transactions**: Transaction validation, signing, processing
- **Hashing**: Cryptographic functions and merkle trees
- **Storage**: Database operations and state management

#### Example Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let transactions = vec![
            Transaction::new("test_tx".to_string()),
        ];

        let block = Block::new(0, "genesis".to_string(), transactions);

        assert_eq!(block.index, 0);
        assert_eq!(block.previous_hash, "genesis");
        assert_eq!(block.transactions.len(), 1);
    }

    #[test]
    fn test_block_validation() {
        let block = create_test_block();
        let result = block.validate();

        assert!(result.is_ok());
    }
}
```

### 2. Integration Tests

#### API Integration Tests

```rust
#[tokio::test]
async fn test_full_api_workflow() {
    // Start test server
    let server = start_test_server().await;

    // Test authentication
    let token = authenticate(&server).await.unwrap();

    // Test blockchain operations
    let blockchain_info = get_blockchain_info(&server, &token).await.unwrap();
    assert!(blockchain_info.latest_block > 0);

    // Test transaction submission
    let tx_response = submit_transaction(&server, &token, test_transaction()).await.unwrap();
    assert_eq!(tx_response.status, "submitted");

    // Cleanup
    server.shutdown().await;
}
```

#### Chaincode Integration Tests

```rust
#[tokio::test]
async fn test_chaincode_execution() {
    let executor = setup_test_executor().await;

    // Deploy test chaincode
    let deployment = deploy_chaincode(&executor, "test-chaincode").await.unwrap();

    // Execute chaincode function
    let result = executor.invoke_chaincode(
        "test-chaincode",
        "testFunction",
        vec!["arg1".to_string(), "arg2".to_string()]
    ).await.unwrap();

    assert!(result.success);
    assert_eq!(result.payload, "expected_result");
}
```

### 3. API Tests

#### Authentication Tests

```rust
#[tokio::test]
async fn test_jwt_authentication() {
    let client = create_test_client();

    // Test login
    let login_response = client
        .post("/auth/login")
        .json(&json!({
            "username": "admin",
            "password": "admin123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(login_response.status(), 200);

    let auth_data: AuthResponse = login_response.json().await.unwrap();
    assert!(!auth_data.access_token.is_empty());

    // Test authenticated endpoint
    let protected_response = client
        .get("/api/v1/blockchain/info")
        .bearer_auth(&auth_data.access_token)
        .send()
        .await
        .unwrap();

    assert_eq!(protected_response.status(), 200);
}
```

#### Rate Limiting Tests

```rust
#[tokio::test]
async fn test_rate_limiting() {
    let client = create_test_client();
    let token = authenticate_test_user(&client).await;

    // Make requests up to the limit
    for _ in 0..100 {
        let response = client
            .get("/api/v1/blockchain/info")
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    // Next request should be rate limited
    let response = client
        .get("/api/v1/blockchain/info")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 429);
}
```

### 4. Performance Tests

#### Load Testing

```rust
#[tokio::test]
async fn test_api_load() {
    let server = start_test_server().await;
    let concurrent_users = 50;
    let requests_per_user = 100;

    let mut handles = vec![];

    for _ in 0..concurrent_users {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            let client = create_test_client();
            let token = authenticate_test_user(&client).await;

            for _ in 0..requests_per_user {
                let response = client
                    .get("/api/v1/blockchain/info")
                    .bearer_auth(&token)
                    .send()
                    .await
                    .unwrap();

                assert_eq!(response.status(), 200);
            }
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }

    server.shutdown().await;
}
```

## Test Data and Fixtures

### Test Data Setup

```rust
pub fn create_test_blockchain() -> Blockchain {
    let mut blockchain = Blockchain::new();

    // Add test blocks
    for i in 1..=10 {
        let transactions = vec![
            Transaction::new(format!("test_tx_{}", i)),
        ];
        blockchain.add_block(transactions).unwrap();
    }

    blockchain
}

pub fn create_test_transaction() -> Transaction {
    Transaction {
        id: "test_tx_001".to_string(),
        chaincode_id: "test-chaincode".to_string(),
        function: "testFunction".to_string(),
        args: vec!["arg1".to_string(), "arg2".to_string()],
        timestamp: Utc::now(),
    }
}
```

### Mock Services

```rust
pub struct MockChaincodeExecutor {
    responses: HashMap<String, ChaincodeExecutionResult>,
}

impl MockChaincodeExecutor {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    pub fn add_mock_response(&mut self, function: &str, result: ChaincodeExecutionResult) {
        self.responses.insert(function.to_string(), result);
    }
}

#[async_trait]
impl ChaincodeExecutorInterface for MockChaincodeExecutor {
    async fn execute_chaincode(&self, tx: &Transaction) -> BeaconResult<ChaincodeExecutionResult> {
        if let Some(response) = self.responses.get(&tx.function) {
            Ok(response.clone())
        } else {
            Ok(ChaincodeExecutionResult::success("mock_result".to_string()))
        }
    }
}
```

## Continuous Integration

### GitHub Actions Configuration

```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: |
          cargo test --all
          cargo test --test integration

      - name: Generate coverage
        run: |
          cargo tarpaulin --out xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Test Reporting

### Coverage Reports

- Minimum coverage threshold: 80%
- Generate HTML reports for detailed analysis
- Track coverage trends over time

### Test Metrics

- Test execution time
- Flaky test detection
- Performance regression detection

## Best Practices

1. **Test Isolation**: Each test should be independent
2. **Clean State**: Reset database/state between tests
3. **Deterministic**: Tests should produce consistent results
4. **Fast Feedback**: Unit tests should run quickly
5. **Meaningful Assertions**: Test the right things with clear assertions
6. **Error Cases**: Test both success and failure scenarios

## Debugging Tests

```bash
# Run tests with debug output
RUST_LOG=debug cargo test -- --nocapture

# Run specific test with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run tests in single thread for debugging
cargo test -- --test-threads=1
```

## Next Steps

- [API Testing Details](api-tests.md)
- [Integration Testing Guide](integration-tests.md)
- [Performance Testing](performance-tests.md)
- [Chaincode Testing](chaincode-tests.md)
