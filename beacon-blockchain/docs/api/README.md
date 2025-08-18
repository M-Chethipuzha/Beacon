# BEACON REST API Documentation

## Overview

The BEACON REST API provides comprehensive access to blockchain operations, transaction management, and state queries. The API features JWT-based authentication, role-based access control, and rate limiting.

## Base URL

```
http://localhost:3000
```

## Authentication

The API uses JWT (JSON Web Token) based authentication. Obtain a token by calling the login endpoint.

### Login

```http
POST /auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "admin123"
}
```

**Response:**

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### Using Authentication

Include the token in the Authorization header:

```http
Authorization: Bearer <your_access_token>
```

## Endpoints

### Health & Info

#### Health Check

```http
GET /health
```

Returns server health status.

#### Server Information

```http
GET /info
```

Returns server version and configuration information.

### Authentication Endpoints

#### Login

```http
POST /auth/login
```

Authenticate and receive access tokens.

#### Logout

```http
POST /auth/logout
Authorization: Bearer <token>
```

Invalidate the current session.

#### Refresh Token

```http
POST /auth/refresh
Authorization: Bearer <refresh_token>
```

Get a new access token using refresh token.

#### User Information

```http
GET /auth/user
Authorization: Bearer <token>
```

Get current user information.

### Blockchain Operations

#### Blockchain Information

```http
GET /api/v1/blockchain/info
Authorization: Bearer <token>
```

Get blockchain statistics and information.

**Response:**

```json
{
  "network_id": "beacon-mainnet",
  "latest_block": 12345,
  "total_transactions": 98765,
  "chain_height": 12345,
  "consensus_algorithm": "PoA",
  "node_count": 5,
  "uptime": "72h 15m 30s"
}
```

#### List Blocks

```http
GET /api/v1/blockchain/blocks?limit=10&offset=0
Authorization: Bearer <token>
```

**Query Parameters:**

- `limit` (optional): Number of blocks to return (default: 10, max: 100)
- `offset` (optional): Number of blocks to skip (default: 0)

#### Latest Blocks

```http
GET /api/v1/blockchain/latest?count=5
Authorization: Bearer <token>
```

### Transaction Management

#### Submit Transaction

```http
POST /api/v1/transactions
Authorization: Bearer <token>
Content-Type: application/json

{
  "chaincode_id": "supply-chain",
  "function": "createProduct",
  "args": ["PROD001", "Laptop", "Electronics"],
  "endorsement_policy": "default"
}
```

#### List Transactions

```http
GET /api/v1/transactions?limit=10&status=confirmed
Authorization: Bearer <token>
```

**Query Parameters:**

- `limit` (optional): Number of transactions to return
- `offset` (optional): Number of transactions to skip
- `status` (optional): Filter by status (pending, confirmed, failed)
- `chaincode_id` (optional): Filter by chaincode

#### Get Transaction

```http
GET /api/v1/transactions/{transaction_id}
Authorization: Bearer <token>
```

#### Invoke Chaincode

```http
POST /api/v1/chaincode/invoke
Authorization: Bearer <token>
Content-Type: application/json

{
  "chaincode_id": "gateway-management",
  "function": "registerGateway",
  "args": ["gateway001", "192.168.1.100"],
  "channel_id": "default"
}
```

### State Management

#### Get State

```http
GET /api/v1/state?key=product_PROD001&chaincode_id=supply-chain
Authorization: Bearer <token>
```

**Query Parameters:**

- `key` (required): State key to retrieve
- `chaincode_id` (optional): Chaincode ID (default: "default")
- `channel_id` (optional): Channel ID (default: "default")

#### Query State Range

```http
GET /api/v1/state/range?start_key=product_&end_key=product_z&limit=50
Authorization: Bearer <token>
```

#### Get State History

```http
GET /api/v1/state/history?key=product_PROD001&limit=10
Authorization: Bearer <token>
```

## Response Formats

### Success Response

```json
{
  "status": "success",
  "data": { ... },
  "timestamp": "2025-07-30T12:00:00Z"
}
```

### Error Response

```json
{
  "status": "error",
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Invalid request parameters",
    "details": "The 'key' parameter is required"
  },
  "timestamp": "2025-07-30T12:00:00Z"
}
```

## Rate Limiting

The API implements rate limiting based on user roles:

- **Guest**: 10 requests per minute
- **User**: 100 requests per minute
- **Admin**: 1000 requests per minute

Rate limit headers are included in responses:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1627734000
```

## Error Codes

| Code                       | Description                         |
| -------------------------- | ----------------------------------- |
| `AUTHENTICATION_REQUIRED`  | Valid authentication token required |
| `INSUFFICIENT_PERMISSIONS` | User lacks required permissions     |
| `RATE_LIMIT_EXCEEDED`      | Too many requests                   |
| `INVALID_REQUEST`          | Request validation failed           |
| `RESOURCE_NOT_FOUND`       | Requested resource not found        |
| `INTERNAL_ERROR`           | Server internal error               |
| `CHAINCODE_ERROR`          | Chaincode execution failed          |
| `STATE_NOT_FOUND`          | Requested state key not found       |

## Examples

See the [API Examples](examples/) directory for complete usage examples in various programming languages.

## Testing

Use the provided test scripts to validate API functionality:

```bash
# Linux/Mac
./test_api.sh

# Windows PowerShell
./test_api.ps1
```
