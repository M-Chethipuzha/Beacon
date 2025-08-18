# BEACON Blockchain Complete API Reference

## Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [Base Configuration](#base-configuration)
4. [Health & System Endpoints](#health--system-endpoints)
5. [Authentication Endpoints](#authentication-endpoints)
6. [Blockchain Operation Endpoints](#blockchain-operation-endpoints)
7. [Transaction Management Endpoints](#transaction-management-endpoints)
8. [State Management Endpoints](#state-management-endpoints)
9. [Error Codes](#error-codes)
10. [Rate Limiting](#rate-limiting)
11. [Response Formats](#response-formats)
12. [Security](#security)

---

## Overview

The BEACON Blockchain REST API provides comprehensive access to blockchain operations, transaction management, state queries, and administrative functions. The API features JWT-based authentication, role-based access control, and rate limiting for production use.

### Key Features

- JWT-based authentication with role-based permissions
- RESTful API design with consistent response formats
- Comprehensive blockchain and state query capabilities
- Transaction submission and tracking
- Rate limiting based on user roles
- Detailed error handling and reporting

---

## Authentication

### Authentication Methods

1. **JWT Token Authentication** (Primary)
2. **API Key Authentication** (Service-to-service)

### User Roles and Permissions

| Role     | Permissions                                                                | Rate Limit   |
| -------- | -------------------------------------------------------------------------- | ------------ |
| admin    | All operations (read/write blockchain, invoke chaincode, state management) | 1000 req/min |
| operator | Read blockchain, write transactions, invoke chaincode, read state          | 500 req/min  |
| gateway  | Similar to operator with gateway-specific permissions                      | 300 req/min  |
| viewer   | Read-only access to blockchain and state                                   | 100 req/min  |
| guest    | Limited read access                                                        | 10 req/min   |

### Default Test Accounts

| Username | Password    | Role     |
| -------- | ----------- | -------- |
| admin    | admin123    | admin    |
| operator | operator123 | operator |
| gateway  | gateway123  | gateway  |
| viewer   | viewer123   | viewer   |

---

## Base Configuration

- **Base URL**: `http://localhost:3000` (Development)
- **Base URL**: `http://localhost:8080` (Alternative)
- **API Version**: v1
- **Content-Type**: `application/json`
- **Authentication Header**: `Authorization: Bearer <token>`

---

## Health & System Endpoints

### 1. Health Check

**Endpoint**: `GET /health`  
**Authentication**: None required  
**Description**: Check API server health status

#### Response

```json
{
  "status": "healthy",
  "timestamp": "2025-07-30T12:00:00Z",
  "service": "beacon-api",
  "version": "1.0.0"
}
```

### 2. Server Information

**Endpoint**: `GET /info`  
**Authentication**: None required  
**Description**: Get server version and configuration information

#### Response

```json
{
  "service": "BEACON Blockchain API",
  "version": "1.0.0",
  "network": "beacon-mainnet",
  "api_version": "v1",
  "features": ["jwt_auth", "rate_limiting", "state_queries"],
  "statistics": {
    "latest_block_number": 12345,
    "total_transactions": 98765,
    "uptime": "72h 15m 30s"
  }
}
```

---

## Authentication Endpoints

### 1. User Login

**Endpoint**: `POST /auth/login`  
**Authentication**: None required  
**Description**: Authenticate user and receive JWT token

#### Request Body

```json
{
  "username": "admin",
  "password": "admin123",
  "node_id": "optional_node_identifier"
}
```

#### Response

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2025-07-31T12:00:00Z",
  "user": {
    "username": "admin",
    "role": "admin",
    "node_id": null,
    "last_login": "2025-07-30T12:00:00Z"
  },
  "permissions": [
    "read:blockchain",
    "write:transactions",
    "admin:node",
    "invoke:chaincode",
    "read:state",
    "write:state"
  ]
}
```

### 2. User Logout

**Endpoint**: `POST /auth/logout`  
**Authentication**: Bearer token required  
**Description**: Invalidate current session (token blacklisting)

#### Response

```json
{
  "message": "Logged out successfully",
  "timestamp": "2025-07-30T12:00:00Z"
}
```

### 3. Refresh Token

**Endpoint**: `POST /api/v1/auth/refresh`  
**Authentication**: Bearer token required  
**Description**: Get a new access token with extended expiration

#### Response

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2025-07-31T12:00:00Z",
  "user": {
    "username": "admin",
    "role": "admin",
    "node_id": null,
    "last_login": "2025-07-30T12:00:00Z"
  },
  "permissions": ["read:blockchain", "write:transactions", ...]
}
```

### 4. Get User Information

**Endpoint**: `GET /api/v1/auth/user`  
**Authentication**: Bearer token required  
**Description**: Get current authenticated user information

#### Response

```json
{
  "username": "admin",
  "role": "admin",
  "node_id": null,
  "last_login": "2025-07-30T12:00:00Z"
}
```

---

## Blockchain Operation Endpoints

### 1. Get Blockchain Information

**Endpoint**: `GET /api/v1/blockchain/info`  
**Authentication**: Optional (enhanced features with auth)  
**Description**: Get blockchain statistics and network information

#### Query Parameters

- `include_validators` (boolean): Include validator information

#### Response

```json
{
  "network_id": "beacon-mainnet",
  "latest_block": 12345,
  "total_transactions": 98765,
  "chain_height": 12345,
  "consensus_algorithm": "PoA",
  "node_count": 5,
  "uptime": "72h 15m 30s",
  "validators": [
    {
      "id": "validator_1",
      "address": "0x1234567890abcdef",
      "stake": "1000000",
      "active": true
    }
  ]
}
```

### 2. Get Latest Blocks

**Endpoint**: `GET /api/v1/blockchain/latest`  
**Authentication**: Optional  
**Description**: Get the most recent blocks

#### Query Parameters

- `count` (integer, max 100): Number of blocks to return (default: 5)
- `include_transactions` (boolean): Include transaction details

#### Response

```json
{
  "blocks": [
    {
      "number": 12345,
      "hash": "0x1234567890abcdef...",
      "parent_hash": "0x0987654321fedcba...",
      "timestamp": "2025-07-30T12:00:00Z",
      "validator": "validator_1",
      "transaction_count": 15,
      "size": 2048,
      "gas_used": 5000000,
      "gas_limit": 10000000
    }
  ],
  "total": 12345
}
```

### 3. Get Block by Number

**Endpoint**: `GET /api/v1/blockchain/blocks/{block_number}`  
**Authentication**: Optional  
**Description**: Get specific block by block number

#### Path Parameters

- `block_number` (integer): Block number to retrieve

#### Query Parameters

- `include_transactions` (boolean): Include full transaction details
- `include_validators` (boolean): Include validator information

#### Response

```json
{
  "number": 12345,
  "hash": "0x1234567890abcdef...",
  "parent_hash": "0x0987654321fedcba...",
  "timestamp": "2025-07-30T12:00:00Z",
  "validator": "validator_1",
  "size": 2048,
  "gas_limit": 10000000,
  "gas_used": 5000000,
  "transaction_count": 15,
  "difficulty": 1000000,
  "extra_data": "0x424541434f4e",
  "transactions": [
    {
      "hash": "0xabcdef1234567890...",
      "from": "0x1111111111111111...",
      "to": "0x2222222222222222...",
      "value": "1000000000000000000",
      "gas": 21000,
      "gas_price": "20000000000"
    }
  ]
}
```

### 4. Get Block by Hash

**Endpoint**: `GET /api/v1/blockchain/blocks/hash/{block_hash}`  
**Authentication**: Optional  
**Description**: Get specific block by hash

#### Path Parameters

- `block_hash` (string): Block hash to retrieve

#### Query Parameters

- `include_transactions` (boolean): Include full transaction details

#### Response

Same format as "Get Block by Number"

### 5. Get Block Range

**Endpoint**: `GET /api/v1/blockchain/blocks`  
**Authentication**: Optional  
**Description**: Get multiple blocks within a range

#### Query Parameters

- `limit` (integer, max 100): Number of blocks to return (default: 10)
- `offset` (integer): Number of blocks to skip (default: 0)
- `start` (integer): Start block number
- `end` (integer): End block number
- `include_transactions` (boolean): Include transaction details

#### Response

```json
{
  "blocks": [
    // Array of block objects
  ],
  "pagination": {
    "limit": 10,
    "offset": 0,
    "total": 12345,
    "has_more": true
  }
}
```

---

## Transaction Management Endpoints

### 1. Submit Transaction

**Endpoint**: `POST /api/v1/transactions`  
**Authentication**: Bearer token required  
**Description**: Submit a new transaction to the blockchain

#### Request Body

```json
{
  "chaincode_id": "supply-chain",
  "function": "createProduct",
  "args": ["PROD001", "Laptop", "Electronics"],
  "endorsement_policy": "default",
  "metadata": {
    "description": "Create new product",
    "priority": "normal"
  }
}
```

#### Response

```json
{
  "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "submitted",
  "chaincode_id": "supply-chain",
  "function": "createProduct",
  "args": ["PROD001", "Laptop", "Electronics"],
  "timestamp": "2025-07-30T12:00:00Z",
  "estimated_confirmation_time": "30s",
  "gas_estimate": 21000
}
```

### 2. Get Transaction by ID

**Endpoint**: `GET /api/v1/transactions/{transaction_id}`  
**Authentication**: Optional  
**Description**: Get transaction details by transaction ID

#### Path Parameters

- `transaction_id` (string): Transaction ID to retrieve

#### Response

```json
{
  "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "confirmed",
  "block_number": 12345,
  "block_hash": "0x1234567890abcdef...",
  "chaincode_id": "supply-chain",
  "function": "createProduct",
  "args": ["PROD001", "Laptop", "Electronics"],
  "timestamp": "2025-07-30T12:00:00Z",
  "gas_used": 21000,
  "events": [
    {
      "event_name": "ProductCreated",
      "payload": {
        "product_id": "PROD001",
        "name": "Laptop",
        "category": "Electronics"
      }
    }
  ]
}
```

### 3. List Transactions

**Endpoint**: `GET /api/v1/transactions`  
**Authentication**: Optional  
**Description**: Get list of transactions with filtering options

#### Query Parameters

- `limit` (integer, max 100): Number of transactions to return (default: 10)
- `offset` (integer): Number of transactions to skip (default: 0)
- `status` (string): Filter by status (pending, confirmed, failed)
- `chaincode_id` (string): Filter by chaincode ID

#### Response

```json
{
  "transactions": [
    {
      "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "confirmed",
      "block_number": 12345,
      "chaincode_id": "supply-chain",
      "function": "createProduct",
      "timestamp": "2025-07-30T12:00:00Z",
      "gas_used": 21000
    }
  ],
  "pagination": {
    "limit": 10,
    "offset": 0,
    "total": 5000,
    "has_more": true
  }
}
```

### 4. Invoke Chaincode

**Endpoint**: `POST /api/v1/chaincode/invoke`  
**Authentication**: Bearer token required  
**Description**: Directly invoke chaincode function

#### Request Body

```json
{
  "chaincode_id": "gateway-management",
  "function": "registerGateway",
  "args": ["gateway001", "192.168.1.100"],
  "channel_id": "default",
  "read_only": false
}
```

#### Response

```json
{
  "transaction_id": "550e8400-e29b-41d4-a716-446655440001",
  "result": {
    "status": "success",
    "payload": "Gateway registered successfully",
    "events": [
      {
        "event_name": "GatewayRegistered",
        "payload": {
          "gateway_id": "gateway001",
          "ip_address": "192.168.1.100"
        }
      }
    ]
  },
  "timestamp": "2025-07-30T12:00:00Z"
}
```

---

## State Management Endpoints

### 1. Get State by Key

**Endpoint**: `GET /api/v1/state`  
**Authentication**: Bearer token required  
**Description**: Get state value by key

#### Query Parameters

- `key` (string, required): State key to retrieve
- `chaincode_id` (string): Chaincode ID (default: "default")
- `channel_id` (string): Channel ID (default: "default")
- `version` (integer): Specific version (optional)

#### Response

```json
{
  "key": "product_PROD001",
  "value": "{\"id\":\"PROD001\",\"name\":\"Laptop\",\"category\":\"Electronics\"}",
  "chaincode_id": "supply-chain",
  "channel_id": "default",
  "block_number": 12345,
  "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-07-30T12:00:00Z",
  "version": {
    "block_num": 12345,
    "tx_num": 1
  }
}
```

### 2. Query State Range

**Endpoint**: `GET /api/v1/state/range`  
**Authentication**: Bearer token required  
**Description**: Query state by key range

#### Query Parameters

- `start_key` (string, required): Start key for range
- `end_key` (string, required): End key for range
- `limit` (integer, max 100): Number of results to return (default: 10)
- `chaincode_id` (string): Chaincode ID (default: "default")

#### Response

```json
{
  "results": [
    {
      "key": "product_PROD001",
      "value": "{\"id\":\"PROD001\",\"name\":\"Laptop\"}",
      "chaincode_id": "supply-chain",
      "block_number": 12345,
      "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
      "timestamp": "2025-07-30T12:00:00Z"
    }
  ],
  "range": {
    "start_key": "product_",
    "end_key": "product_z",
    "limit": 50
  },
  "has_more": false
}
```

### 3. Get State History

**Endpoint**: `GET /api/v1/state/history`  
**Authentication**: Bearer token required  
**Description**: Get historical values for a state key

#### Query Parameters

- `key` (string, required): State key to get history for
- `limit` (integer, max 100): Number of history entries (default: 10)
- `chaincode_id` (string): Chaincode ID (default: "default")
- `from_block` (integer): Start block number
- `to_block` (integer): End block number

#### Response

```json
{
  "key": "product_PROD001",
  "history": [
    {
      "value": "{\"id\":\"PROD001\",\"name\":\"Gaming Laptop\"}",
      "block_number": 12346,
      "transaction_id": "550e8400-e29b-41d4-a716-446655440002",
      "timestamp": "2025-07-30T12:30:00Z",
      "is_delete": false,
      "version": {
        "block_num": 12346,
        "tx_num": 1
      }
    },
    {
      "value": "{\"id\":\"PROD001\",\"name\":\"Laptop\"}",
      "block_number": 12345,
      "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
      "timestamp": "2025-07-30T12:00:00Z",
      "is_delete": false,
      "version": {
        "block_num": 12345,
        "tx_num": 1
      }
    }
  ],
  "pagination": {
    "limit": 10,
    "total": 2,
    "has_more": false
  }
}
```

### 4. Advanced State Query

**Endpoint**: `POST /api/v1/state/query`  
**Authentication**: Bearer token required  
**Description**: Advanced state queries with filtering

#### Request Body

```json
{
  "query_type": "prefix",
  "prefix": "balance_",
  "limit": 100,
  "include_history": false,
  "chaincode_id": "token-ledger",
  "filters": {
    "min_value": 1000,
    "category": "active"
  }
}
```

#### Query Types

- `prefix`: Query by key prefix
- `range`: Query by key range
- `composite`: Query composite keys
- `all`: Get all state (admin only)

#### Response

```json
{
  "query": {
    "type": "prefix",
    "prefix": "balance_",
    "limit": 100
  },
  "results": [
    {
      "key": "balance_alice",
      "value": "5000",
      "chaincode_id": "token-ledger",
      "block_number": 12340,
      "timestamp": "2025-07-30T11:30:00Z"
    }
  ],
  "pagination": {
    "limit": 100,
    "total": 1,
    "has_more": false
  }
}
```

---

## Error Codes

### Standard HTTP Status Codes

- `200` - Success
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `409` - Conflict
- `429` - Too Many Requests
- `500` - Internal Server Error

### Custom Error Codes

| Code                       | Description                         | HTTP Status |
| -------------------------- | ----------------------------------- | ----------- |
| `AUTHENTICATION_REQUIRED`  | Valid authentication token required | 401         |
| `INSUFFICIENT_PERMISSIONS` | User lacks required permissions     | 403         |
| `RATE_LIMIT_EXCEEDED`      | Too many requests                   | 429         |
| `INVALID_REQUEST`          | Request validation failed           | 400         |
| `RESOURCE_NOT_FOUND`       | Requested resource not found        | 404         |
| `INTERNAL_ERROR`           | Server internal error               | 500         |
| `CHAINCODE_ERROR`          | Chaincode execution failed          | 500         |
| `STATE_NOT_FOUND`          | Requested state key not found       | 404         |
| `TRANSACTION_FAILED`       | Transaction execution failed        | 400         |
| `INVALID_BLOCK_NUMBER`     | Invalid block number specified      | 400         |
| `TOKEN_EXPIRED`            | JWT token has expired               | 401         |
| `INVALID_CREDENTIALS`      | Invalid username or password        | 401         |

---

## Rate Limiting

### Rate Limits by Role

| Role            | Requests/Minute | Burst Limit |
| --------------- | --------------- | ----------- |
| admin           | 1000            | 100         |
| operator        | 500             | 50          |
| gateway         | 300             | 30          |
| viewer          | 100             | 20          |
| unauthenticated | 30              | 5           |

### Rate Limit Headers

The API includes rate limit information in response headers:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1627734000
X-RateLimit-Window: 60
```

### Rate Limit Exceeded Response

```json
{
  "status": "error",
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Try again later.",
    "details": {
      "limit": 100,
      "window": 60,
      "retry_after": 30
    }
  },
  "timestamp": "2025-07-30T12:00:00Z"
}
```

---

## Response Formats

### Success Response Format

```json
{
  "status": "success",
  "data": {
    // Response data here
  },
  "timestamp": "2025-07-30T12:00:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Error Response Format

```json
{
  "status": "error",
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {
      // Additional error details
    }
  },
  "timestamp": "2025-07-30T12:00:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Pagination Format

```json
{
  "pagination": {
    "limit": 10,
    "offset": 0,
    "total": 1000,
    "has_more": true,
    "next_offset": 10,
    "previous_offset": null
  }
}
```

---

## Security

### Security Features

1. **JWT Authentication**: Tokens expire after 24 hours
2. **Role-Based Access Control**: Fine-grained permissions
3. **Rate Limiting**: Prevents abuse and DoS attacks
4. **Input Validation**: All inputs validated server-side
5. **CORS Support**: Configurable cross-origin policies
6. **Security Headers**: Standard security headers included
7. **Request Size Limits**: Prevents large payload attacks
8. **Timeout Protection**: Request timeout middleware

### Security Headers

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security: max-age=31536000`

### Best Practices

1. Use HTTPS in production environments
2. Rotate JWT secrets regularly
3. Implement proper token refresh logic
4. Store API keys securely
5. Monitor for suspicious activity
6. Implement proper CORS policies
7. Use strong passwords for default accounts
8. Enable comprehensive logging

---

## Example Usage

### cURL Examples

#### Login and Get Token

```bash
# Login
TOKEN=$(curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}' | jq -r '.token')

# Use token for authenticated request
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/v1/blockchain/info
```

#### Submit Transaction

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"chaincode_id": "supply-chain", "function": "createProduct", "args": ["PROD001", "Laptop", "Electronics"]}' \
  http://localhost:3000/api/v1/transactions
```

### JavaScript Examples

#### Using Fetch API

```javascript
// Login
const login = async () => {
  const response = await fetch("http://localhost:3000/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      username: "admin",
      password: "admin123",
    }),
  });
  const data = await response.json();
  return data.token;
};

// Submit transaction
const submitTransaction = async (token) => {
  const response = await fetch("http://localhost:3000/api/v1/transactions", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      chaincode_id: "supply-chain",
      function: "createProduct",
      args: ["PROD001", "Laptop", "Electronics"],
    }),
  });
  return await response.json();
};
```

---

This complete API reference provides comprehensive documentation for all BEACON blockchain API endpoints, including authentication, blockchain operations, transaction management, and state queries. Use this reference for integration, testing, and development purposes.
