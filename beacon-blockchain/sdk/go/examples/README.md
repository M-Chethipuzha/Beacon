# BEACON Go SDK Examples

This directory contains example chaincodes demonstrating various use cases and features of the BEACON blockchain platform using the Go SDK.

## Examples Overview

### 1. Gateway Management (`gateway-management/`)

**Purpose**: Demonstrates comprehensive gateway management for the BEACON network
**Use Case**: Network infrastructure management and access control

**Key Features**:

- Gateway registration and lifecycle management
- Access policy creation and enforcement
- Audit logging for compliance
- Heartbeat monitoring for gateway health
- Event emission for external monitoring

**Functions**:

- `registerGateway` - Register new gateways
- `updateGateway` - Update gateway metadata
- `deactivateGateway` - Deactivate gateways
- `heartbeat` - Update gateway health status
- `createPolicy` - Create access control policies
- `updatePolicy` - Update existing policies
- `auditLog` - Create audit log entries
- `queryAuditLogs` - Query audit logs with filters

### 2. Supply Chain Management (`supply-chain/`)

**Purpose**: Demonstrates end-to-end supply chain tracking and provenance
**Use Case**: Product lifecycle management and traceability

**Key Features**:

- Product creation and metadata management
- Shipment tracking with carrier integration
- Transaction recording for financial flows
- Complete provenance tracking
- Verification and compliance checking

**Functions**:

- `createProduct` - Register new products
- `updateProduct` - Update product metadata
- `createShipment` - Create shipping records
- `updateShipmentStatus` - Track shipment progress
- `deliverShipment` - Mark deliveries complete
- `recordTransaction` - Record financial transactions
- `recordProvenance` - Create provenance records
- `traceProduct` - Complete product traceability

### 3. Identity Verification (`identity-verification/`)

**Purpose**: Demonstrates digital identity and credential management
**Use Case**: Verifiable credentials and identity verification systems

**Key Features**:

- Digital identity creation and management
- Verifiable credential issuance
- Credential verification and validation
- Revocation management
- Verification request workflows

**Functions**:

- `createIdentity` - Create digital identities
- `updateIdentity` - Update identity attributes
- `issueCredential` - Issue verifiable credentials
- `verifyCredential` - Verify credential authenticity
- `revokeCredential` - Revoke credentials
- `requestVerification` - Create verification requests
- `checkRevocationStatus` - Check credential revocation

## Building and Testing

### Prerequisites

- Go 1.21 or later
- Protocol Buffers compiler (protoc)
- BEACON blockchain development environment

### Build All Examples

```bash
# From the examples directory
cd sdk/go/examples

# Build gateway management
cd gateway-management
go mod tidy
go build -o gateway-management.exe .

# Build supply chain
cd ../supply-chain
go mod tidy
go build -o supply-chain.exe .

# Build identity verification
cd ../identity-verification
go mod tidy
go build -o identity-verification.exe .
```

### Run Tests

```bash
# Run tests for each example (if test files exist)
cd gateway-management && go test ./...
cd ../supply-chain && go test ./...
cd ../identity-verification && go test ./...
```

## Deployment Guide

### 1. Package Chaincode

```bash
# Package for deployment
peer lifecycle chaincode package gateway-mgmt.tar.gz --path ./gateway-management --lang golang --label gateway-mgmt_1.0

peer lifecycle chaincode package supply-chain.tar.gz --path ./supply-chain --lang golang --label supply-chain_1.0

peer lifecycle chaincode package identity-verification.tar.gz --path ./identity-verification --lang golang --label identity-verification_1.0
```

### 2. Install on Peers

```bash
# Install on peer nodes
peer lifecycle chaincode install gateway-mgmt.tar.gz
peer lifecycle chaincode install supply-chain.tar.gz
peer lifecycle chaincode install identity-verification.tar.gz
```

### 3. Approve and Commit

```bash
# Approve chaincode definitions
peer lifecycle chaincode approveformyorg -C mychannel -n gateway-mgmt --version 1.0 --package-id <package-id> --sequence 1

# Commit chaincode definitions
peer lifecycle chaincode commit -C mychannel -n gateway-mgmt --version 1.0 --sequence 1
```

## Usage Examples

### Gateway Management

```bash
# Register a gateway
peer chaincode invoke -C mychannel -n gateway-mgmt -c '{"function":"registerGateway","Args":["gw001","MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A...","org1"]}'

# Create access policy
peer chaincode invoke -C mychannel -n gateway-mgmt -c '{"function":"createPolicy","Args":["policy001","Data Access","Controls data access","[{\"resource\":\"data/*\",\"action\":\"read\",\"principals\":[\"gw001\"]}]"]}'

# Query gateways
peer chaincode query -C mychannel -n gateway-mgmt -c '{"function":"listGateways","Args":["active"]}'
```

### Supply Chain

```bash
# Create product
peer chaincode invoke -C mychannel -n supply-chain -c '{"function":"createProduct","Args":["prod001","Laptop Computer","High-performance laptop","SKU123","TechCorp"]}'

# Create shipment
peer chaincode invoke -C mychannel -n supply-chain -c '{"function":"createShipment","Args":["ship001","prod001","Factory A","Warehouse B","FedEx","1234567890"]}'

# Track product
peer chaincode query -C mychannel -n supply-chain -c '{"function":"traceProduct","Args":["prod001"]}'
```

### Identity Verification

```bash
# Create identity
peer chaincode invoke -C mychannel -n identity-verification -c '{"function":"createIdentity","Args":["user001","MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A...","individual","University"]}'

# Issue credential
peer chaincode invoke -C mychannel -n identity-verification -c '{"function":"issueCredential","Args":["cred001","academic","university","user001","{\"degree\":\"Bachelor of Science\",\"major\":\"Computer Science\"}","signature_value"]}'

# Verify credential
peer chaincode query -C mychannel -n identity-verification -c '{"function":"verifyCredential","Args":["cred001"]}'
```

## Integration with BEACON Platform

### SDK Integration

These examples demonstrate how to integrate with the BEACON Go SDK:

- **gRPC Communication**: All examples use the SDK's gRPC client for node communication
- **State Management**: Proper state management patterns using the chaincode stub
- **Event Emission**: Examples show how to emit blockchain events for external systems
- **Error Handling**: Comprehensive error handling and validation patterns

### Network Features

The examples showcase BEACON-specific features:

- **Gateway Management**: Core BEACON network functionality
- **Identity Systems**: Integration with BEACON's identity management
- **Audit Logging**: Compliance and monitoring capabilities
- **Policy Management**: Access control and governance features

### API Integration

Examples can be integrated with BEACON's REST API:

- **REST Endpoints**: Chaincode functions can be called via REST API
- **Event Subscriptions**: Events can be consumed by external applications
- **Monitoring**: Integration with BEACON's monitoring and analytics systems

## Development Patterns

### Code Structure

- **Struct-based Design**: Each chaincode is implemented as a struct with methods
- **Data Models**: Clear data structures for business entities
- **Helper Functions**: Utility functions for common operations
- **Error Handling**: Consistent error handling patterns

### Best Practices Demonstrated

- **Input Validation**: All functions validate input parameters
- **State Consistency**: Proper state management and transaction handling
- **Event Emission**: Appropriate use of blockchain events
- **Audit Logging**: Comprehensive logging for compliance
- **Security**: Proper authorization and access control patterns

### Testing Strategies

- **Unit Testing**: Test individual functions in isolation
- **Integration Testing**: Test chaincode interactions with the blockchain
- **End-to-End Testing**: Test complete workflows and use cases
- **Performance Testing**: Test scalability and performance characteristics

## Contributing

### Adding New Examples

1. Create a new directory under `examples/`
2. Follow the existing structure with `go.mod`, `main.go`, and `README.md`
3. Implement the chaincode following BEACON patterns
4. Add comprehensive documentation and usage examples
5. Ensure the example builds and tests successfully

### Code Style

- Follow Go conventions and best practices
- Use meaningful variable and function names
- Include comprehensive comments and documentation
- Implement proper error handling and validation
- Follow the existing SDK patterns and interfaces

## Support and Documentation

- **BEACON Documentation**: [Link to BEACON docs]
- **Go SDK Reference**: [Link to SDK docs]
- **Community Forum**: [Link to community]
- **Issue Tracker**: [Link to issues]

For questions about these examples or the BEACON platform, please refer to the documentation or reach out to the community.
