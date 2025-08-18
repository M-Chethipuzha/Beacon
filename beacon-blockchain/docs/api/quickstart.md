# BEACON API Quick Start Guide

## Prerequisites

- Rust 1.70+ installed
- BEACON blockchain node running
- API server compiled and ready

## 1. Start the API Server

```bash
# Navigate to the API server directory
cd crates/beacon-api

# Run the server
cargo run --bin beacon-api
```

The server will start on `http://localhost:3000`

## 2. Verify Server is Running

```bash
# Health check
curl http://localhost:3000/health

# Expected response:
{
  "status": "healthy",
  "timestamp": "2025-07-30T12:00:00Z",
  "version": "0.1.0"
}
```

## 3. Authenticate

```bash
# Login to get access token
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }'

# Save the access_token from the response
export TOKEN="your_access_token_here"
```

## 4. Make API Calls

### Get Blockchain Information

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/v1/blockchain/info
```

### Submit a Transaction

```bash
curl -X POST http://localhost:3000/api/v1/transactions \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "chaincode_id": "supply-chain",
    "function": "createProduct",
    "args": ["PROD001", "Laptop", "Electronics"]
  }'
```

### Query State

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/v1/state?key=product_PROD001"
```

## 5. Using the Test Scripts

For comprehensive testing, use the provided scripts:

### Linux/Mac

```bash
./test_api.sh
```

### Windows PowerShell

```powershell
./test_api.ps1
```

## Common Use Cases

### 1. Gateway Management

```bash
# Register a new gateway
curl -X POST http://localhost:3000/api/v1/chaincode/invoke \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "chaincode_id": "gateway-management",
    "function": "registerGateway",
    "args": ["gateway001", "192.168.1.100", "active"]
  }'
```

### 2. Supply Chain Tracking

```bash
# Create a product
curl -X POST http://localhost:3000/api/v1/chaincode/invoke \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "chaincode_id": "supply-chain",
    "function": "createProduct",
    "args": ["PROD001", "Laptop", "Electronics", "Manufacturer A"]
  }'

# Track product movement
curl -X POST http://localhost:3000/api/v1/chaincode/invoke \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "chaincode_id": "supply-chain",
    "function": "updateLocation",
    "args": ["PROD001", "Warehouse B", "In Transit"]
  }'
```

### 3. Identity Verification

```bash
# Create digital identity
curl -X POST http://localhost:3000/api/v1/chaincode/invoke \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "chaincode_id": "identity-verification",
    "function": "createIdentity",
    "args": ["user001", "John Doe", "john@example.com"]
  }'
```

## Error Handling

### Authentication Errors

```bash
# Missing token
HTTP 401 Unauthorized
{
  "status": "error",
  "error": {
    "code": "AUTHENTICATION_REQUIRED",
    "message": "Valid authentication token required"
  }
}
```

### Rate Limiting

```bash
# Too many requests
HTTP 429 Too Many Requests
{
  "status": "error",
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Try again later."
  }
}
```

## Next Steps

- Explore the [Complete API Reference](README.md)
- Check out [Example Implementations](examples/)
- Learn about [Testing Strategies](../testing/api-tests.md)
- Review [Development Guidelines](../development/)
