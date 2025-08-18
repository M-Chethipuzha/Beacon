# API Testing Documentation

## Overview

This document provides comprehensive testing strategies for the BEACON REST API, including automated tests, manual testing procedures, and validation scripts.

## Test Categories

### 1. Authentication Tests

#### Login/Logout Flow

```bash
#!/bin/bash
# Test authentication workflow

API_BASE="http://localhost:3000"

echo "Testing Authentication Flow..."

# Test 1: Valid login
echo "1. Testing valid login..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}')

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')
if [ "$TOKEN" != "null" ] && [ "$TOKEN" != "" ]; then
  echo "✅ Valid login successful"
else
  echo "❌ Valid login failed"
  exit 1
fi

# Test 2: Invalid credentials
echo "2. Testing invalid login..."
INVALID_RESPONSE=$(curl -s -w "%{http_code}" -X POST "$API_BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "invalid", "password": "wrong"}')

if [[ "$INVALID_RESPONSE" == *"401"* ]]; then
  echo "✅ Invalid login correctly rejected"
else
  echo "❌ Invalid login not properly handled"
fi

# Test 3: Authenticated request
echo "3. Testing authenticated request..."
AUTH_RESPONSE=$(curl -s -w "%{http_code}" \
  -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/blockchain/info")

if [[ "$AUTH_RESPONSE" == *"200"* ]]; then
  echo "✅ Authenticated request successful"
else
  echo "❌ Authenticated request failed"
fi

# Test 4: Logout
echo "4. Testing logout..."
LOGOUT_RESPONSE=$(curl -s -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/auth/logout")

if [[ "$LOGOUT_RESPONSE" == *"200"* ]]; then
  echo "✅ Logout successful"
else
  echo "❌ Logout failed"
fi

echo "Authentication tests completed!"
```

#### Token Validation Tests

```bash
#!/bin/bash
# Test token validation and expiry

# Test expired token
EXPIRED_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.expired.token"
EXPIRED_RESPONSE=$(curl -s -w "%{http_code}" \
  -H "Authorization: Bearer $EXPIRED_TOKEN" \
  "$API_BASE/api/v1/blockchain/info")

if [[ "$EXPIRED_RESPONSE" == *"401"* ]]; then
  echo "✅ Expired token correctly rejected"
else
  echo "❌ Expired token validation failed"
fi

# Test malformed token
MALFORMED_TOKEN="invalid.token.format"
MALFORMED_RESPONSE=$(curl -s -w "%{http_code}" \
  -H "Authorization: Bearer $MALFORMED_TOKEN" \
  "$API_BASE/api/v1/blockchain/info")

if [[ "$MALFORMED_RESPONSE" == *"401"* ]]; then
  echo "✅ Malformed token correctly rejected"
else
  echo "❌ Malformed token validation failed"
fi
```

### 2. Endpoint Tests

#### Blockchain Operations

```bash
#!/bin/bash
# Test blockchain endpoints

echo "Testing Blockchain Endpoints..."

# Get valid token
TOKEN=$(get_auth_token)

# Test blockchain info
echo "1. Testing blockchain info..."
INFO_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/blockchain/info")

NETWORK_ID=$(echo "$INFO_RESPONSE" | jq -r '.network_id')
if [ "$NETWORK_ID" != "null" ] && [ "$NETWORK_ID" != "" ]; then
  echo "✅ Blockchain info retrieved successfully"
  echo "   Network ID: $NETWORK_ID"
else
  echo "❌ Blockchain info failed"
fi

# Test blocks listing
echo "2. Testing blocks listing..."
BLOCKS_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/blockchain/blocks?limit=5")

BLOCKS_COUNT=$(echo "$BLOCKS_RESPONSE" | jq '.blocks | length')
if [ "$BLOCKS_COUNT" -gt 0 ]; then
  echo "✅ Blocks listing successful ($BLOCKS_COUNT blocks)"
else
  echo "❌ Blocks listing failed"
fi

# Test latest blocks
echo "3. Testing latest blocks..."
LATEST_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/blockchain/latest?count=3")

LATEST_COUNT=$(echo "$LATEST_RESPONSE" | jq '.blocks | length')
if [ "$LATEST_COUNT" -gt 0 ]; then
  echo "✅ Latest blocks retrieved ($LATEST_COUNT blocks)"
else
  echo "❌ Latest blocks failed"
fi
```

#### Transaction Tests

```bash
#!/bin/bash
# Test transaction endpoints

echo "Testing Transaction Endpoints..."

# Test transaction submission
echo "1. Testing transaction submission..."
TX_RESPONSE=$(curl -s -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "chaincode_id": "supply-chain",
    "function": "createProduct",
    "args": ["PROD001", "Test Product", "Electronics"]
  }' \
  "$API_BASE/api/v1/transactions")

TX_ID=$(echo "$TX_RESPONSE" | jq -r '.transaction_id')
if [ "$TX_ID" != "null" ] && [ "$TX_ID" != "" ]; then
  echo "✅ Transaction submitted successfully"
  echo "   Transaction ID: $TX_ID"
else
  echo "❌ Transaction submission failed"
fi

# Test transaction query
echo "2. Testing transaction query..."
if [ "$TX_ID" != "null" ]; then
  QUERY_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
    "$API_BASE/api/v1/transactions/$TX_ID")

  QUERIED_ID=$(echo "$QUERY_RESPONSE" | jq -r '.transaction_id')
  if [ "$QUERIED_ID" == "$TX_ID" ]; then
    echo "✅ Transaction query successful"
  else
    echo "❌ Transaction query failed"
  fi
fi

# Test transactions listing
echo "3. Testing transactions listing..."
LIST_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/transactions?limit=10")

TX_COUNT=$(echo "$LIST_RESPONSE" | jq '.transactions | length')
if [ "$TX_COUNT" -gt 0 ]; then
  echo "✅ Transactions listing successful ($TX_COUNT transactions)"
else
  echo "❌ Transactions listing failed"
fi
```

#### State Management Tests

```bash
#!/bin/bash
# Test state management endpoints

echo "Testing State Management Endpoints..."

# Test state query
echo "1. Testing state query..."
STATE_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/state?key=product_PROD001&chaincode_id=supply-chain")

STATE_VALUE=$(echo "$STATE_RESPONSE" | jq -r '.value')
if [ "$STATE_VALUE" != "null" ] && [ "$STATE_VALUE" != "" ]; then
  echo "✅ State query successful"
  echo "   Value: $STATE_VALUE"
else
  echo "❌ State query failed"
fi

# Test state range query
echo "2. Testing state range query..."
RANGE_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/state/range?start_key=product_&end_key=product_z&limit=10")

RANGE_COUNT=$(echo "$RANGE_RESPONSE" | jq '.results | length')
if [ "$RANGE_COUNT" -ge 0 ]; then
  echo "✅ State range query successful ($RANGE_COUNT results)"
else
  echo "❌ State range query failed"
fi

# Test state history
echo "3. Testing state history..."
HISTORY_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/state/history?key=product_PROD001&limit=5")

HISTORY_COUNT=$(echo "$HISTORY_RESPONSE" | jq '.history | length')
if [ "$HISTORY_COUNT" -ge 0 ]; then
  echo "✅ State history query successful ($HISTORY_COUNT entries)"
else
  echo "❌ State history query failed"
fi
```

### 3. Rate Limiting Tests

```bash
#!/bin/bash
# Test rate limiting

echo "Testing Rate Limiting..."

# Get token
TOKEN=$(get_auth_token)

# Make requests up to the limit
RATE_LIMIT_EXCEEDED=false
for i in {1..105}; do  # Assume limit is 100 for standard user
  RESPONSE=$(curl -s -w "%{http_code}" \
    -H "Authorization: Bearer $TOKEN" \
    "$API_BASE/api/v1/blockchain/info")

  if [[ "$RESPONSE" == *"429"* ]]; then
    echo "✅ Rate limit triggered at request $i"
    RATE_LIMIT_EXCEEDED=true
    break
  fi
done

if [ "$RATE_LIMIT_EXCEEDED" = true ]; then
  echo "✅ Rate limiting working correctly"
else
  echo "❌ Rate limiting not triggered"
fi
```

### 4. Error Handling Tests

```bash
#!/bin/bash
# Test error handling

echo "Testing Error Handling..."

# Test missing authentication
echo "1. Testing missing authentication..."
NO_AUTH_RESPONSE=$(curl -s -w "%{http_code}" \
  "$API_BASE/api/v1/blockchain/info")

if [[ "$NO_AUTH_RESPONSE" == *"401"* ]]; then
  echo "✅ Missing authentication correctly handled"
else
  echo "❌ Missing authentication not handled"
fi

# Test invalid endpoint
echo "2. Testing invalid endpoint..."
INVALID_RESPONSE=$(curl -s -w "%{http_code}" \
  -H "Authorization: Bearer $TOKEN" \
  "$API_BASE/api/v1/invalid/endpoint")

if [[ "$INVALID_RESPONSE" == *"404"* ]]; then
  echo "✅ Invalid endpoint correctly handled"
else
  echo "❌ Invalid endpoint not handled"
fi

# Test invalid JSON
echo "3. Testing invalid JSON..."
INVALID_JSON_RESPONSE=$(curl -s -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"invalid": json}' \
  "$API_BASE/api/v1/transactions")

if [[ "$INVALID_JSON_RESPONSE" == *"400"* ]]; then
  echo "✅ Invalid JSON correctly handled"
else
  echo "❌ Invalid JSON not handled"
fi
```

### 5. Performance Tests

```bash
#!/bin/bash
# Basic performance testing

echo "Testing API Performance..."

# Test response times
echo "1. Testing response times..."
TOKEN=$(get_auth_token)

for endpoint in "health" "info" "api/v1/blockchain/info"; do
  echo "   Testing $endpoint..."

  # Run 10 requests and measure average time
  TOTAL_TIME=0
  for i in {1..10}; do
    START_TIME=$(date +%s%3N)

    if [[ "$endpoint" == "health" ]] || [[ "$endpoint" == "info" ]]; then
      curl -s "$API_BASE/$endpoint" > /dev/null
    else
      curl -s -H "Authorization: Bearer $TOKEN" "$API_BASE/$endpoint" > /dev/null
    fi

    END_TIME=$(date +%s%3N)
    DURATION=$((END_TIME - START_TIME))
    TOTAL_TIME=$((TOTAL_TIME + DURATION))
  done

  AVERAGE_TIME=$((TOTAL_TIME / 10))
  echo "     Average response time: ${AVERAGE_TIME}ms"

  if [ "$AVERAGE_TIME" -lt 1000 ]; then
    echo "     ✅ Performance acceptable"
  else
    echo "     ⚠️  Performance may need improvement"
  fi
done
```

## Automated Test Suite

### Test Runner Script

```bash
#!/bin/bash
# Complete API test suite

set -e  # Exit on any error

API_BASE="http://localhost:3000"

# Helper function to get auth token
get_auth_token() {
  local response=$(curl -s -X POST "$API_BASE/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username": "admin", "password": "admin123"}')
  echo "$response" | jq -r '.access_token'
}

# Check if server is running
check_server() {
  echo "Checking if API server is running..."
  if curl -s "$API_BASE/health" > /dev/null; then
    echo "✅ Server is running"
  else
    echo "❌ Server is not running. Please start the API server first."
    exit 1
  fi
}

# Run all test suites
run_all_tests() {
  echo "🧪 Starting BEACON API Test Suite..."
  echo "=================================="

  check_server

  # Run test suites
  ./tests/auth_tests.sh
  ./tests/blockchain_tests.sh
  ./tests/transaction_tests.sh
  ./tests/state_tests.sh
  ./tests/rate_limit_tests.sh
  ./tests/error_handling_tests.sh
  ./tests/performance_tests.sh

  echo "=================================="
  echo "🎉 All API tests completed!"
}

# Execute main function
run_all_tests
```

## Test Data Management

### Test Fixtures

```json
{
  "test_users": [
    {
      "username": "admin",
      "password": "admin123",
      "role": "admin"
    },
    {
      "username": "user",
      "password": "user123",
      "role": "user"
    }
  ],
  "test_transactions": [
    {
      "chaincode_id": "supply-chain",
      "function": "createProduct",
      "args": ["PROD001", "Test Laptop", "Electronics"]
    },
    {
      "chaincode_id": "gateway-management",
      "function": "registerGateway",
      "args": ["GW001", "192.168.1.100", "active"]
    }
  ],
  "test_state_keys": [
    {
      "key": "product_PROD001",
      "chaincode_id": "supply-chain"
    },
    {
      "key": "gateway_GW001",
      "chaincode_id": "gateway-management"
    }
  ]
}
```

## Continuous Integration

### GitHub Actions for API Testing

```yaml
name: API Tests
on: [push, pull_request]

jobs:
  api-tests:
    runs-on: ubuntu-latest
    services:
      beacon-node:
        image: beacon/node:latest
        ports:
          - 3000:3000

    steps:
      - uses: actions/checkout@v3

      - name: Wait for API server
        run: |
          timeout 60 bash -c 'until curl -f http://localhost:3000/health; do sleep 1; done'

      - name: Run API tests
        run: |
          chmod +x ./test_api.sh
          ./test_api.sh

      - name: Upload test results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: api-test-results
          path: test-results/
```

## Monitoring and Reporting

### Test Results Format

```json
{
  "test_suite": "BEACON API Tests",
  "timestamp": "2025-07-30T12:00:00Z",
  "total_tests": 45,
  "passed": 43,
  "failed": 2,
  "skipped": 0,
  "duration": "2.5s",
  "results": [
    {
      "test_name": "test_valid_login",
      "status": "passed",
      "duration": "150ms"
    },
    {
      "test_name": "test_rate_limiting",
      "status": "failed",
      "duration": "1.2s",
      "error": "Rate limit not triggered"
    }
  ]
}
```

## Best Practices

1. **Test Independence**: Each test should be isolated and not depend on others
2. **Data Cleanup**: Clean up test data after each test run
3. **Meaningful Assertions**: Test both positive and negative scenarios
4. **Performance Monitoring**: Track API response times
5. **Security Testing**: Validate authentication and authorization
6. **Error Scenarios**: Test error handling and edge cases

## Troubleshooting

### Common Issues

1. **Server Not Running**: Ensure API server is started before running tests
2. **Authentication Failures**: Check credentials and token expiration
3. **Network Issues**: Verify API server is accessible on the configured port
4. **Rate Limiting**: Wait between test runs if rate limits are triggered

### Debug Mode

```bash
# Run tests with debug output
DEBUG=1 ./test_api.sh

# Run specific test with verbose output
VERBOSE=1 ./tests/auth_tests.sh
```

This comprehensive API testing framework ensures the BEACON REST API is thoroughly validated and maintains high quality standards.
