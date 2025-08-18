# BEACON Testing Infrastructure

This directory contains comprehensive test suites for the BEACON blockchain platform.

## Test Structure

```
tests/
├── integration/          # Integration tests
│   ├── api_storage_test.rs
│   ├── chaincode_integration_test.rs
│   └── full_stack_test.rs
├── api/                  # API-specific tests
│   ├── auth_test.rs
│   ├── blockchain_endpoints_test.rs
│   ├── transaction_endpoints_test.rs
│   └── state_endpoints_test.rs
├── unit/                 # Unit tests (in individual crate directories)
└── scripts/              # Test automation scripts
    ├── run_all_tests.sh
    ├── api_load_test.sh
    └── performance_monitor.sh
```

## Test Categories

### 1. Unit Tests

Located within each crate's `src/` directory:

- `crates/beacon-core/src/lib.rs` - Core types and utilities
- `crates/beacon-storage/src/lib.rs` - Storage implementations
- `crates/beacon-chaincode/src/lib.rs` - Chaincode execution
- `crates/beacon-api/src/lib.rs` - API handlers and middleware

### 2. Integration Tests

Test interactions between components:

- API ↔ Storage integration
- Chaincode ↔ Storage integration
- Full end-to-end workflows

### 3. API Tests

HTTP/REST API validation:

- Authentication and authorization
- Endpoint functionality
- Error handling
- Rate limiting

### 4. Performance Tests

System performance validation:

- Load testing
- Stress testing
- Resource utilization
- Response time benchmarks

## Running Tests

### All Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test --jobs 4
```

### Specific Test Categories

```bash
# Integration tests only
cargo test --test integration

# API tests only
cargo test --test api

# Unit tests only
cargo test --lib

# Performance tests
cargo test --release --test performance
```

### Individual Test Files

```bash
# Specific integration test
cargo test --test api_storage_test

# Specific API test
cargo test --test auth_test

# Run with specific features
cargo test --features "test-utils,mock-storage"
```

## Test Configuration

### Environment Variables

```bash
export RUST_LOG=debug
export DATABASE_URL=":memory:"
export JWT_SECRET="test_secret_key_for_testing_only"
export API_PORT=3001
export RATE_LIMIT_REQUESTS=1000
export RATE_LIMIT_WINDOW=60
```

### Test Database

Tests use in-memory SQLite databases by default. For persistent testing:

```bash
export TEST_DATABASE_URL="sqlite:test_beacon.db"
```

## Test Data and Fixtures

Test data is managed through fixtures:

- User accounts for authentication testing
- Sample transactions and blocks
- Chaincode test contracts
- State data for validation

## CI/CD Integration

### GitHub Actions

Tests run automatically on:

- Pull requests
- Pushes to main branch
- Scheduled performance regression checks

### Test Coverage

Generate coverage reports:

```bash
cargo tarpaulin --out Html --output-dir target/coverage
```

## Best Practices

1. **Test Isolation**: Each test uses fresh data and state
2. **Async Testing**: All async code uses proper test runtime
3. **Error Testing**: Both success and failure scenarios tested
4. **Performance Baselines**: Performance tests have defined thresholds
5. **Mock Services**: External dependencies mocked appropriately

## Troubleshooting

### Common Issues

1. **Port Conflicts**: Ensure test ports don't conflict with running services
2. **Database Locks**: Use separate test databases or in-memory stores
3. **Async Timeouts**: Increase timeouts for slow CI environments
4. **Resource Limits**: Monitor memory and CPU usage during tests

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo test -- --nocapture

# Run specific test with backtrace
RUST_BACKTRACE=1 cargo test specific_test_name

# Run with verbose output
cargo test --verbose
```

## Contributing

When adding new tests:

1. Follow existing naming conventions
2. Include both positive and negative test cases
3. Add performance tests for new features
4. Update documentation for new test categories
5. Ensure tests pass in CI environment

## Performance Baselines

Current performance targets:

- API response time: < 100ms (95th percentile)
- Transaction throughput: > 1000 TPS
- Memory usage: < 512MB under load
- Database operations: < 10ms per operation

These baselines are validated in automated performance tests.
