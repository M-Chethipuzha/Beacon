# BEACON Blockchain Access Guide

## Overview

This comprehensive guide covers all methods to access, interact with, and monitor the BEACON blockchain network. Whether you're a developer integrating with BEACON, an administrator managing the network, or an end-user querying blockchain data, this guide provides the information you need.

## Table of Contents

1. [Network Overview](#network-overview)
2. [REST API Access](#rest-api-access)
3. [Go SDK Integration](#go-sdk-integration)
4. [Direct Node Communication](#direct-node-communication)
5. [Network Discovery](#network-discovery)
6. [Authentication & Authorization](#authentication--authorization)
7. [Monitoring & Health Checks](#monitoring--health-checks)
8. [Troubleshooting](#troubleshooting)

## Network Overview

### BEACON Network Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   BEACON Node   │    │   BEACON Node   │    │   BEACON Node   │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ REST API    │ │    │ │ REST API    │ │    │ │ REST API    │ │
│ │ :3000       │ │    │ │ :3000       │ │    │ │ :3000       │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ P2P Network │ │◄──►│ │ P2P Network │ │◄──►│ │ P2P Network │ │
│ │ :8080       │ │    │ │ :8080       │ │    │ │ :8080       │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Chaincode   │ │    │ │ Chaincode   │ │    │ │ Chaincode   │ │
│ │ gRPC :9000  │ │    │ │ gRPC :9000  │ │    │ │ gRPC :9000  │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Network Endpoints

- **REST API**: Port 3000 (HTTP/HTTPS)
- **P2P Network**: Port 8080 (TCP)
- **Chaincode gRPC**: Port 9000 (gRPC)
- **Node Discovery**: Port 8081 (UDP)

## REST API Access

### Base URL and Authentication

```bash
# Default local node
BASE_URL="http://localhost:3000"

# Production network (example)
BASE_URL="https://beacon-node-1.example.com"

# Authentication required for most endpoints
TOKEN=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}' | \
  jq -r '.access_token')
```

### Core API Endpoints

#### 1. Health and Status

```bash
# Check node health
curl "$BASE_URL/health"

# Get node information
curl "$BASE_URL/info"

# Get blockchain information (requires auth)
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/blockchain/info"
```

#### 2. Blockchain Queries

```bash
# Get latest blocks
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/blockchain/latest?count=10"

# Get specific block by number
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/blockchain/blocks?number=42"

# Get block range
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/blockchain/blocks?start=1&end=10"
```

#### 3. Transaction Operations

```bash
# Submit transaction
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "chaincode_id": "supply-chain",
    "function": "createProduct",
    "args": ["PROD001", "Laptop", "Electronics"]
  }' \
  "$BASE_URL/api/v1/transactions"

# Query transaction by ID
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/transactions/tx_12345"

# List recent transactions
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/transactions?limit=20"
```

#### 4. State Queries

```bash
# Get state by key
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/state?key=product_PROD001&chaincode_id=supply-chain"

# Range query
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/state/range?start_key=product_&end_key=product_z&limit=50"

# State history
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/state/history?key=product_PROD001&limit=10"
```

### API Response Formats

#### Successful Response

```json
{
  "success": true,
  "data": {
    "block_number": 123,
    "hash": "0x1234...",
    "timestamp": "2025-07-30T12:00:00Z"
  },
  "metadata": {
    "execution_time": "15ms",
    "node_id": "beacon-node-1"
  }
}
```

#### Error Response

```json
{
  "success": false,
  "error": {
    "code": "INVALID_TRANSACTION",
    "message": "Transaction validation failed",
    "details": "Invalid signature"
  },
  "timestamp": "2025-07-30T12:00:00Z"
}
```

## Go SDK Integration

### Setup and Installation

```go
// go.mod
module myapp

go 1.21

require (
    github.com/beacon-blockchain/sdk v0.1.0
    google.golang.org/grpc v1.74.2
)
```

### Basic Chaincode Development

```go
package main

import (
    "github.com/beacon-blockchain/sdk/shim"
    "github.com/beacon-blockchain/sdk/peer"
)

type MyChaincode struct{}

func (cc *MyChaincode) Init(stub shim.ChaincodeStubInterface) peer.Response {
    // Initialize chaincode state
    return shim.Success(nil)
}

func (cc *MyChaincode) Invoke(stub shim.ChaincodeStubInterface) peer.Response {
    function, args := stub.GetFunctionAndParameters()

    switch function {
    case "setValue":
        return cc.setValue(stub, args)
    case "getValue":
        return cc.getValue(stub, args)
    default:
        return shim.Error("Unknown function: " + function)
    }
}

func (cc *MyChaincode) setValue(stub shim.ChaincodeStubInterface, args []string) peer.Response {
    if len(args) != 2 {
        return shim.Error("Expected 2 arguments: key and value")
    }

    err := stub.PutState(args[0], []byte(args[1]))
    if err != nil {
        return shim.Error("Failed to set state: " + err.Error())
    }

    return shim.Success(nil)
}

func main() {
    chaincode := &MyChaincode{}
    err := shim.Start(chaincode)
    if err != nil {
        fmt.Printf("Error starting chaincode: %v\n", err)
    }
}
```

### Advanced SDK Features

#### State Management

```go
// Get state with validation
value, err := stub.GetStateWithValidation("product_001")
if err != nil {
    return shim.Error("State retrieval failed")
}

// Put state as JSON
product := Product{ID: "001", Name: "Laptop"}
err = stub.PutStateAsJSON("product_001", product)
if err != nil {
    return shim.Error("JSON state storage failed")
}

// Range queries
iterator, err := stub.GetStateByRange("product_", "product_z")
if err != nil {
    return shim.Error("Range query failed")
}
defer iterator.Close()

results, err := shim.IteratorToArray(iterator)
if err != nil {
    return shim.Error("Iterator processing failed")
}
```

#### Event Emission

```go
// Emit event for external monitoring
event := struct {
    Type      string `json:"type"`
    ProductID string `json:"product_id"`
    Action    string `json:"action"`
}{
    Type:      "PRODUCT_CREATED",
    ProductID: "PROD001",
    Action:    "CREATE",
}

err := stub.SetEvent("ProductEvent", event)
if err != nil {
    return shim.Error("Event emission failed")
}
```

## Direct Node Communication

### P2P Network Protocol

#### Network Discovery

```bash
# Discover network peers
curl "http://localhost:8081/discover"

# Join network
curl -X POST "http://localhost:8081/join" \
  -H "Content-Type: application/json" \
  -d '{"node_id": "new-node", "address": "192.168.1.100:8080"}'
```

#### Peer Communication

```bash
# Get peer information
curl "http://localhost:8080/peers"

# Send message to specific peer
curl -X POST "http://localhost:8080/message" \
  -H "Content-Type: application/json" \
  -d '{
    "target_peer": "peer-node-2",
    "message_type": "SYNC_REQUEST",
    "data": {}
  }'
```

### gRPC Chaincode Communication

#### Direct gRPC Calls

```go
// Connect to chaincode service
conn, err := grpc.Dial("localhost:9000", grpc.WithInsecure())
if err != nil {
    log.Fatal("Connection failed:", err)
}
defer conn.Close()

client := pb.NewChaincodeShimServiceClient(conn)

// Execute chaincode function
request := &pb.ChaincodeRequest{
    ChaincodeId: "supply-chain",
    Function:    "getProduct",
    Args:        []string{"PROD001"},
}

response, err := client.ExecuteChaincode(context.Background(), request)
if err != nil {
    log.Fatal("Execution failed:", err)
}
```

## Network Discovery

### Automatic Peer Discovery

```bash
# Start node with discovery enabled
./beacon-node --discovery-port 8081 --bootstrap-peers "192.168.1.10:8080,192.168.1.20:8080"

# Manual peer discovery
curl -X GET "http://localhost:8081/network/topology"
```

### Network Configuration

```yaml
# beacon-config.yaml
network:
  discovery:
    enabled: true
    port: 8081
    interval: 30s
    timeout: 5s

  p2p:
    port: 8080
    max_peers: 50
    bootstrap_peers:
      - "beacon-seed-1.example.com:8080"
      - "beacon-seed-2.example.com:8080"

  consensus:
    type: "PoA"
    validators:
      - "validator-1"
      - "validator-2"
      - "validator-3"
```

## Authentication & Authorization

### User Management

#### Create User Account

```bash
# Admin creates new user
curl -X POST -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "developer",
    "password": "secure_password",
    "role": "developer",
    "permissions": ["read", "write", "invoke"]
  }' \
  "$BASE_URL/admin/users"
```

#### Role-Based Access Control

```bash
# Check user permissions
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/auth/permissions"

# Response includes user roles and capabilities
{
  "user": "developer",
  "roles": ["developer"],
  "permissions": [
    "blockchain:read",
    "transactions:submit",
    "state:query",
    "chaincode:invoke"
  ]
}
```

### Token Management

#### Token Lifecycle

```bash
# Login and get token
TOKEN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "user", "password": "password"}')

ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
REFRESH_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.refresh_token')

# Refresh token before expiry
NEW_TOKEN=$(curl -s -X POST "$BASE_URL/auth/refresh" \
  -H "Authorization: Bearer $ACCESS_TOKEN" | \
  jq -r '.access_token')

# Logout and invalidate token
curl -X POST -H "Authorization: Bearer $ACCESS_TOKEN" \
  "$BASE_URL/auth/logout"
```

## Monitoring & Health Checks

### Node Health Monitoring

#### Health Check Endpoints

```bash
# Basic health check
curl "$BASE_URL/health"
# Response: {"status": "healthy", "timestamp": "2025-07-30T12:00:00Z"}

# Detailed health information
curl "$BASE_URL/health/detailed"
# Response includes database, network, and chaincode status

# Node metrics
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/metrics"
```

#### Network Status Monitoring

```bash
# Get network topology
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/network/topology"

# Peer connectivity status
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/network/peers"

# Consensus status
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/network/consensus"
```

### Performance Monitoring

#### Transaction Metrics

```bash
# Transaction throughput
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/metrics/transactions"

# Block generation rate
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/metrics/blocks"

# Chaincode execution statistics
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/metrics/chaincode"
```

#### System Resource Monitoring

```bash
# Node resource usage
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/system/resources"

# Database statistics
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/system/storage"

# Network bandwidth usage
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/system/network"
```

## Network Access Patterns

### Single Node Access

```bash
# Direct connection to specific node
NODE_1="http://beacon-node-1.example.com:3000"
curl -H "Authorization: Bearer $TOKEN" \
  "$NODE_1/api/v1/blockchain/info"
```

### Load Balanced Access

```bash
# Through load balancer
LOAD_BALANCER="https://api.beacon-network.com"
curl -H "Authorization: Bearer $TOKEN" \
  "$LOAD_BALANCER/api/v1/blockchain/info"
```

### Multi-Node Queries

```bash
# Query multiple nodes for consensus
NODES=("node1:3000" "node2:3000" "node3:3000")

for node in "${NODES[@]}"; do
  echo "Querying $node..."
  curl -H "Authorization: Bearer $TOKEN" \
    "http://$node/api/v1/blockchain/info"
done
```

## Troubleshooting

### Common Connection Issues

#### Network Connectivity

```bash
# Test node reachability
ping beacon-node-1.example.com

# Test port accessibility
telnet beacon-node-1.example.com 3000
nc -zv beacon-node-1.example.com 3000

# Test API availability
curl -v "http://beacon-node-1.example.com:3000/health"
```

#### Authentication Problems

```bash
# Verify credentials
curl -v -X POST "http://localhost:3000/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'

# Check token validity
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/auth/user"

# Debug authentication headers
curl -v -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/v1/blockchain/info"
```

### Performance Issues

#### Slow API Responses

```bash
# Measure response times
time curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/blockchain/info"

# Check node resource usage
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/system/resources"
```

#### Network Latency

```bash
# Test inter-node communication
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/network/latency"

# Check peer connection quality
curl -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/network/peers/status"
```

### Error Diagnostics

#### Common HTTP Status Codes

- `200 OK`: Request successful
- `400 Bad Request`: Invalid request format or parameters
- `401 Unauthorized`: Authentication required or invalid token
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server-side error
- `503 Service Unavailable`: Node temporarily unavailable

#### Error Response Analysis

```bash
# Get detailed error information
curl -v -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/v1/invalid/endpoint" 2>&1 | \
  grep -E "(HTTP|error|message)"
```

### Log Analysis

#### Node Logs

```bash
# View real-time logs
tail -f /var/log/beacon/node.log

# Search for specific errors
grep -i "error\|failed\|timeout" /var/log/beacon/node.log

# Filter by timestamp
grep "2025-07-30 12:" /var/log/beacon/node.log
```

#### API Access Logs

```bash
# Monitor API requests
tail -f /var/log/beacon/api-access.log

# Analyze request patterns
awk '{print $1}' /var/log/beacon/api-access.log | sort | uniq -c
```

## Security Considerations

### Network Security

- Use HTTPS in production environments
- Implement proper firewall rules for node ports
- Monitor for unauthorized access attempts
- Regular security audits and penetration testing

### Authentication Security

- Use strong passwords and regular rotation
- Implement token expiration and refresh policies
- Monitor for suspicious authentication patterns
- Secure token storage and transmission

### Network Access Control

- Implement IP whitelisting for administrative access
- Use VPN for sensitive network operations
- Regular review of user permissions and roles
- Audit logging for all network access

## Best Practices

### Development

1. Use the Go SDK for chaincode development
2. Implement proper error handling and validation
3. Test thoroughly in development environment
4. Follow BEACON coding standards and patterns

### Operations

1. Monitor node health and performance regularly
2. Implement backup and disaster recovery procedures
3. Keep software updated and patched
4. Document network configuration and procedures

### Security

1. Use strong authentication and authorization
2. Implement network security best practices
3. Regular security assessments and updates
4. Incident response planning and procedures

This comprehensive access guide provides all the information needed to successfully interact with the BEACON blockchain network through various interfaces and protocols.
