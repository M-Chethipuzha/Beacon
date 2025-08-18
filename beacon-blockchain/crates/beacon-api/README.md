# BEACON Blockchain REST API Documentation

## Overview

The BEACON Blockchain REST API provides comprehensive access to blockchain operations, state management, and chaincode execution. The API features JWT-based authentication, role-based access control, and rate limiting for production use.

## Base URL

```
http://localhost:8080
```

## Authentication

### JWT Token Authentication

Include the JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

### API Key Authentication

Alternatively, use an API key:

```
Authorization: ApiKey <api_key>
```

## User Roles and Permissions

- **admin**: Full access to all operations
- **operator**: Read blockchain, write transactions, invoke chaincode, read state
- **gateway**: Similar to operator with gateway-specific permissions
- **viewer**: Read-only access to blockchain and state

## Rate Limits

- **Admin**: 1000 requests/minute (100 burst)
- **Operator**: 500 requests/minute (50 burst)
- **Gateway**: 300 requests/minute (30 burst)
- **Viewer**: 100 requests/minute (20 burst)
- **Unauthenticated**: 30 requests/minute (5 burst)

## API Endpoints

### Health & Info

#### GET /health

Health check endpoint.

**Response:**

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "service": "beacon-api",
  "version": "1.0.0"
}
```

#### GET /info

Server information and statistics.

**Response:**

```json
{
  "service": "BEACON Blockchain API",
  "version": "1.0.0",
  "network": "beacon-mainnet",
  "api_version": "v1",
  "features": [...],
  "statistics": {
    "latest_block_number": 1000,
    "total_transactions": 5000,
    "uptime": "2024-01-01T00:00:00Z"
  }
}
```

### Authentication

#### POST /auth/login

User login to obtain JWT token.

**Request:**

```json
{
  "username": "admin",
  "password": "admin123",
  "node_id": "optional_node_id"
}
```

**Response:**

```json
{
  "token": "jwt_token_here",
  "expires_at": "2024-01-02T00:00:00Z",
  "user": {
    "username": "admin",
    "role": "admin",
    "node_id": null,
    "last_login": "2024-01-01T00:00:00Z"
  },
  "permissions": ["read:blockchain", "write:transactions", ...]
}
```

#### POST /auth/logout

Logout (token blacklisting).

**Headers:** `Authorization: Bearer <token>`

#### POST /api/v1/auth/refresh

Refresh JWT token.

**Headers:** `Authorization: Bearer <token>`

#### GET /api/v1/auth/user

Get current user information.

**Headers:** `Authorization: Bearer <token>`

### Blockchain Operations

#### GET /api/v1/blockchain/info

Get blockchain information and statistics.

**Query Parameters:**

- `include_validators` (boolean): Include validator information

#### GET /api/v1/blocks/latest

Get latest blocks.

**Query Parameters:**

- `limit` (integer, max 100): Number of blocks to return
- `include_transactions` (boolean): Include transaction details

#### GET /api/v1/blocks/{block_number}

Get block by number.

**Path Parameters:**

- `block_number` (integer): Block number

**Query Parameters:**

- `include_transactions` (boolean): Include transaction details

#### GET /api/v1/blocks/hash/{block_hash}

Get block by hash.

**Path Parameters:**

- `block_hash` (string): Block hash

### Transaction Operations

#### GET /api/v1/transactions

Get transactions with filtering.

**Query Parameters:**

- `limit` (integer, max 1000): Number of transactions
- `offset` (integer): Pagination offset
- `status` (string): Filter by status (pending, confirmed, failed)
- `from_block` (integer): Start block number
- `to_block` (integer): End block number
- `chaincode_id` (string): Filter by chaincode

#### GET /api/v1/transactions/{tx_hash}

Get transaction by hash.

**Path Parameters:**

- `tx_hash` (string): Transaction hash

#### POST /api/v1/transactions/submit

Submit a new transaction. **Requires authentication.**

**Headers:** `Authorization: Bearer <token>`

**Request:**

```json
{
  "chaincode_id": "example_chaincode",
  "function": "transfer",
  "args": ["alice", "bob", "100"],
  "metadata": {
    "description": "Transfer description",
    "priority": "normal"
  }
}
```

#### POST /api/v1/chaincode/invoke

Invoke chaincode function. **Requires authentication.**

**Headers:** `Authorization: Bearer <token>`

**Request:**

```json
{
  "chaincode_id": "balance_checker",
  "function": "get_balance",
  "args": ["alice"],
  "read_only": true
}
```

### State Management

#### GET /api/v1/state/{key}

Get state value by key. **Requires authentication.**

**Headers:** `Authorization: Bearer <token>`

**Path Parameters:**

- `key` (string): State key

**Query Parameters:**

- `version` (integer): Specific version (optional)

#### POST /api/v1/state/query

Query state with filters. **Requires authentication.**

**Headers:** `Authorization: Bearer <token>`

**Request:**

```json
{
  "query_type": "prefix",
  "prefix": "balance_",
  "limit": 100,
  "include_history": false
}
```

**Query Types:**

- `prefix`: Query by key prefix
- `range`: Query by key range
- `composite`: Query composite keys
- `all`: Get all state (admin only)

#### GET /api/v1/state/{key}/history

Get state history for a key. **Requires authentication.**

**Headers:** `Authorization: Bearer <token>`

**Query Parameters:**

- `limit` (integer, max 1000): Number of history entries
- `from_version` (integer): Start version
- `to_version` (integer): End version

## Error Responses

All endpoints return standardized error responses:

```json
{
  "error": "Error description",
  "code": "ERROR_CODE",
  "timestamp": "2024-01-01T00:00:00Z",
  "details": {}
}
```

### HTTP Status Codes

- `200` - Success
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `409` - Conflict
- `429` - Too Many Requests
- `500` - Internal Server Error

## Default User Accounts

For testing purposes, the following default accounts are available:

| Username | Password    | Role     | Permissions                                                                          |
| -------- | ----------- | -------- | ------------------------------------------------------------------------------------ |
| admin    | admin123    | admin    | All permissions                                                                      |
| operator | operator123 | operator | Read blockchain, write transactions, invoke chaincode, read state                    |
| viewer   | viewer123   | viewer   | Read blockchain, read state                                                          |
| gateway  | gateway123  | gateway  | Read blockchain, write transactions, invoke chaincode, read state, gateway heartbeat |

## API Keys

For service-to-service communication:

| API Key                  | Role    | Usage                      |
| ------------------------ | ------- | -------------------------- |
| beacon_admin_key_12345   | admin   | Full administrative access |
| beacon_gateway_key_67890 | gateway | Gateway node access        |

## Example Usage

### Using cURL

```bash
# Login
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}' | jq -r '.token')

# Get blockchain info
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/v1/blockchain/info

# Submit transaction
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"chaincode_id": "example", "function": "transfer", "args": ["alice", "bob", "100"]}' \
  http://localhost:8080/api/v1/transactions/submit
```

### Using JavaScript/Node.js

```javascript
const axios = require("axios");

const API_BASE = "http://localhost:8080";

// Login
const login = async () => {
  const response = await axios.post(`${API_BASE}/auth/login`, {
    username: "admin",
    password: "admin123",
  });
  return response.data.token;
};

// Submit transaction
const submitTransaction = async (token) => {
  const response = await axios.post(
    `${API_BASE}/api/v1/transactions/submit`,
    {
      chaincode_id: "example_chaincode",
      function: "transfer",
      args: ["alice", "bob", "100"],
    },
    {
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
    }
  );
  return response.data;
};
```

## Security Considerations

1. **JWT Tokens**: Tokens expire after 24 hours and should be refreshed regularly
2. **Rate Limiting**: Implement client-side rate limiting to avoid 429 errors
3. **HTTPS**: Use HTTPS in production environments
4. **API Keys**: Rotate API keys regularly and store securely
5. **Input Validation**: All inputs are validated server-side
6. **CORS**: Configure CORS policies for web applications

## Monitoring

The API includes comprehensive logging and tracing:

- Request/response logging
- Performance metrics
- Security events
- Rate limit violations
- Authentication attempts

Use the `/health` endpoint for basic health monitoring and `/info` for detailed statistics.
