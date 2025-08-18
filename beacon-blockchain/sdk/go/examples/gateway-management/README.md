# Gateway Management Chaincode

This example demonstrates a comprehensive gateway management system for the BEACON blockchain network using the Go SDK. The chaincode provides functionality for gateway registration, access policy management, and audit logging.

## Features

### Gateway Management

- **Gateway Registration**: Register new gateways with unique IDs, public keys, and organization associations
- **Gateway Updates**: Update gateway metadata and status
- **Heartbeat Monitoring**: Track gateway activity with periodic heartbeat updates
- **Status Management**: Activate/deactivate gateways as needed
- **Query Operations**: Retrieve individual gateways or list all gateways with optional filtering

### Access Policy Management

- **Policy Creation**: Define granular access control policies with rules
- **Policy Updates**: Modify existing policies with versioning support
- **Rule-based Access**: Support for resource-action-principal based rules with conditions
- **Policy Queries**: Retrieve and list access policies

### Audit and Compliance

- **Comprehensive Logging**: Automatic audit logging for all operations
- **Audit Queries**: Query audit logs with filtering by gateway ID and action type
- **Transaction Tracking**: Link audit entries to blockchain transactions
- **Event Emission**: Emit blockchain events for external monitoring

## Data Structures

### Gateway

```go
type Gateway struct {
    ID              string            `json:"id"`
    PublicKey       string            `json:"publicKey"`
    OrganizationID  string            `json:"organizationID"`
    Status          string            `json:"status"`
    RegistrationTime int64            `json:"registrationTime"`
    LastHeartbeat   int64            `json:"lastHeartbeat"`
    Metadata        map[string]string `json:"metadata"`
}
```

### Access Policy

```go
type AccessPolicy struct {
    ID          string   `json:"id"`
    Name        string   `json:"name"`
    Description string   `json:"description"`
    Rules       []Rule   `json:"rules"`
    CreatedAt   int64    `json:"createdAt"`
    UpdatedAt   int64    `json:"updatedAt"`
    Version     int      `json:"version"`
}
```

### Rule

```go
type Rule struct {
    Resource   string   `json:"resource"`
    Action     string   `json:"action"`
    Principals []string `json:"principals"`
    Conditions []string `json:"conditions"`
}
```

## API Functions

### Gateway Operations

- `registerGateway(gatewayID, publicKey, organizationID)` - Register a new gateway
- `updateGateway(gatewayID, key1, value1, key2, value2, ...)` - Update gateway metadata
- `getGateway(gatewayID)` - Retrieve gateway information
- `listGateways([statusFilter])` - List all gateways with optional status filter
- `deactivateGateway(gatewayID)` - Deactivate a gateway
- `heartbeat(gatewayID)` - Update gateway heartbeat timestamp

### Policy Operations

- `createPolicy(policyID, name, description, rulesJSON)` - Create a new access policy
- `updatePolicy(policyID, rulesJSON)` - Update an existing policy
- `getPolicy(policyID)` - Retrieve policy information
- `listPolicies()` - List all policies

### Audit Operations

- `auditLog(gatewayID, action, resource, success, [errorMessage])` - Create an audit log entry
- `queryAuditLogs([gatewayFilter], [actionFilter])` - Query audit logs with filters

## Usage Examples

### 1. Building and Testing

```bash
# Build the chaincode
cd sdk/go/examples/gateway-management
go mod tidy
go build

# Run tests (if test files exist)
go test ./...
```

### 2. Gateway Registration

```bash
# Register a new gateway
peer chaincode invoke -C mychannel -n gateway-mgmt -c '{"function":"registerGateway","Args":["gw001","MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A...","org1"]}'

# Update gateway metadata
peer chaincode invoke -C mychannel -n gateway-mgmt -c '{"function":"updateGateway","Args":["gw001","location","datacenter-1","capacity","1000"]}'

# Send heartbeat
peer chaincode invoke -C mychannel -n gateway-mgmt -c '{"function":"heartbeat","Args":["gw001"]}'
```

### 3. Policy Management

```bash
# Create access policy
peer chaincode invoke -C mychannel -n gateway-mgmt -c '{"function":"createPolicy","Args":["policy001","Data Access Policy","Controls access to sensitive data","[{\"resource\":\"data/*\",\"action\":\"read\",\"principals\":[\"gw001\",\"gw002\"],\"conditions\":[\"time > 09:00\",\"time < 17:00\"]}]"]}'

# Query policy
peer chaincode query -C mychannel -n gateway-mgmt -c '{"function":"getPolicy","Args":["policy001"]}'
```

### 4. Audit and Monitoring

```bash
# Query audit logs for specific gateway
peer chaincode query -C mychannel -n gateway-mgmt -c '{"function":"queryAuditLogs","Args":["gw001"]}'

# Query audit logs for specific action
peer chaincode query -C mychannel -n gateway-mgmt -c '{"function":"queryAuditLogs","Args":["","REGISTER_GATEWAY"]}'
```

## Security Features

1. **Input Validation**: All functions validate input parameters
2. **State Integrity**: Proper state management with error handling
3. **Audit Trail**: Complete audit logging for compliance
4. **Access Control**: Policy-based access control framework
5. **Event Emission**: Blockchain events for external monitoring

## Integration with BEACON Network

This chaincode is designed specifically for the BEACON blockchain network and integrates with:

- **Node Communication**: Uses gRPC for communication with BEACON nodes
- **Identity Management**: Integrates with BEACON's identity and organization system
- **Event System**: Emits events that can be consumed by BEACON network services
- **API Gateway**: Can be accessed through BEACON's REST API layer

## Error Handling

The chaincode implements comprehensive error handling:

- Input validation with meaningful error messages
- State consistency checks before operations
- Graceful handling of missing resources
- Audit logging of both successful and failed operations

## Deployment

To deploy this chaincode to a BEACON network:

1. Package the chaincode
2. Install on peer nodes
3. Approve and commit the chaincode definition
4. Initialize with default configuration

The chaincode will automatically create initial system configuration and admin policies during initialization.
