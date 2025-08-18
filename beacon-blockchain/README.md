# BEACON Blockchain

A high-performance blockchain platform built in Rust with Proof of Authority consensus, designed for enterprise applications with chaincode execution capabilities.

## Features

- **Proof of Authority (PoA) Consensus**: Fast, deterministic consensus suitable for enterprise networks
- **P2P Networking**: Built on libp2p for robust peer-to-peer communication
- **Chaincode Execution**: Support for Go-based smart contracts via gRPC
- **REST API**: Complete HTTP API for blockchain interaction
- **High Performance**: Designed for 1000+ TPS throughput
- **RocksDB Storage**: Efficient persistent storage with state management
- **Real-time Events**: WebSocket-based event notifications
- **Monitoring**: Built-in metrics and observability

## Architecture

The BEACON blockchain is structured as a modular Rust workspace with the following crates:

- **beacon-core**: Fundamental types, transactions, blocks, and cryptography
- **beacon-networking**: P2P networking layer using libp2p
- **beacon-storage**: Database abstraction and blockchain storage
- **beacon-consensus**: Proof of Authority consensus implementation
- **beacon-api**: REST API server and HTTP endpoints
- **beacon-chaincode**: Chaincode execution engine and Go SDK
- **beacon-crypto**: Cryptographic utilities and key management
- **beacon-node**: Main node binary that ties everything together

## Quick Start

### Prerequisites

- Rust 1.70+ (stable toolchain)
- Go 1.21+ (for chaincode development)
- Git

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd beacon-blockchain

# Build all components
cargo build --release

# Run tests
cargo test
```

### Running a Node

```bash
# Start a single node (development mode)
cargo run --bin beacon-node

# Start with custom configuration
cargo run --bin beacon-node -- --config node.toml

# Start as a validator
cargo run --bin beacon-node -- --validator --node-id validator1
```

### Configuration

Create a configuration file `node.toml`:

```toml
[node]
id = "beacon_node_001"
data_dir = "./beacon_data"
log_level = "info"

[network]
listen_addr = "/ip4/0.0.0.0/tcp/30303"
bootstrap_peers = []
max_connections = 50
network_id = "beacon_mainnet"

[consensus]
consensus_type = "proof_of_authority"
is_validator = false
validators = [
    "ed25519_pubkey_hex_1",
    "ed25519_pubkey_hex_2",
    "ed25519_pubkey_hex_3"
]

[consensus.params]
block_time = 2000
block_size_limit = 1048576
transaction_timeout = 300
validator_rotation_period = 86400

[storage]
engine = "rocksdb"
cache_size = 256
write_buffer_size = 64
max_open_files = 1000

[api]
enabled = true
bind_addr = "0.0.0.0:8080"
cors_origins = ["*"]
rate_limit = 1000

[chaincode]
grpc_addr = "127.0.0.1:9090"
execution_timeout = 30
max_concurrent = 10

[monitoring]
metrics_enabled = true
metrics_addr = "0.0.0.0:9091"
```

### Docker Deployment

```bash
# Build Docker image
docker build -t beacon-node .

# Run with Docker Compose
docker-compose up -d

# Scale to multiple nodes
docker-compose up --scale validator=3 --scale peer=2
```

## API Usage

### Submit a Transaction

```bash
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "chaincode_id": "access_control",
    "function": "registerGateway",
    "args": ["gateway_001", "ed25519_pubkey_hex"]
  }'
```

### Query Blockchain

```bash
# Get latest block
curl http://localhost:8080/api/v1/blocks/latest

# Get block by index
curl http://localhost:8080/api/v1/blocks/123

# Get transaction
curl http://localhost:8080/api/v1/transactions/tx_id_here

# Query state
curl http://localhost:8080/api/v1/state/gateway:gateway_001
```

### WebSocket Events

```javascript
const ws = new WebSocket("ws://localhost:8080/api/v1/events");

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log("Blockchain event:", data);
};
```

## Chaincode Development

### Go SDK Example

```go
package main

import (
    "github.com/beacon/chaincode-go/shim"
    "github.com/beacon/chaincode-go/peer"
)

type AccessControlChaincode struct{}

func (cc *AccessControlChaincode) RegisterGateway(stub shim.ChaincodeStubInterface, args []string) peer.Response {
    if len(args) != 2 {
        return shim.Error("Expecting 2 arguments")
    }

    gatewayId := args[0]
    publicKey := args[1]

    // Store gateway registration
    err := stub.PutState("gateway:"+gatewayId, []byte(publicKey))
    if err != nil {
        return shim.Error("Failed to register gateway")
    }

    return shim.Success(nil)
}

func main() {
    err := shim.Start(new(AccessControlChaincode))
    if err != nil {
        panic(err)
    }
}
```

### Deploy Chaincode

```bash
# Build chaincode
go build -o access_control chaincode/access_control

# Deploy via API
curl -X POST http://localhost:8080/api/v1/chaincode \
  -H "Content-Type: application/json" \
  -d '{
    "name": "access_control",
    "version": "1.0",
    "binary_path": "./access_control"
  }'
```

## Network Setup

### Single Node (Development)

```bash
cargo run --bin beacon-node -- \
  --node-id dev_node \
  --data-dir ./dev_data
```

### Multi-Node Network

#### Validator Nodes

```bash
# Validator 1
cargo run --bin beacon-node -- \
  --node-id validator1 \
  --validator \
  --listen /ip4/0.0.0.0/tcp/30303

# Validator 2
cargo run --bin beacon-node -- \
  --node-id validator2 \
  --validator \
  --listen /ip4/0.0.0.0/tcp/30304 \
  --bootstrap /ip4/127.0.0.1/tcp/30303

# Validator 3
cargo run --bin beacon-node -- \
  --node-id validator3 \
  --validator \
  --listen /ip4/0.0.0.0/tcp/30305 \
  --bootstrap /ip4/127.0.0.1/tcp/30303
```

#### Peer Nodes

```bash
# Regular peer
cargo run --bin beacon-node -- \
  --node-id peer1 \
  --listen /ip4/0.0.0.0/tcp/30306 \
  --bootstrap /ip4/127.0.0.1/tcp/30303
```

## Performance

The BEACON blockchain is designed for high performance:

- **Throughput**: 1000+ transactions per second
- **Latency**: <100ms transaction processing
- **Block Time**: 2 seconds (configurable)
- **Finality**: Immediate (PoA consensus)
- **Scalability**: 50+ nodes in production networks

## Security

- **Ed25519 Digital Signatures**: 128-bit security level
- **TLS 1.3 Encryption**: All network communication encrypted
- **Input Validation**: Comprehensive validation at all layers
- **Rate Limiting**: DDoS protection and resource management
- **Audit Trail**: Immutable transaction history

## Monitoring

### Metrics

The node exposes Prometheus metrics at `/metrics`:

```bash
curl http://localhost:9091/metrics
```

Key metrics include:

- Transaction throughput
- Block creation time
- Network connectivity
- Database performance
- Memory usage

### Logging

Structured logging with configurable levels:

```bash
# Debug logging
RUST_LOG=debug cargo run --bin beacon-node

# JSON logging for production
RUST_LOG_FORMAT=json cargo run --bin beacon-node
```

## Development

### Project Structure

```
beacon-blockchain/
├── crates/
│   ├── beacon-core/       # Core types and utilities
│   ├── beacon-networking/ # P2P networking
│   ├── beacon-storage/    # Database and storage
│   ├── beacon-consensus/  # Consensus algorithms
│   ├── beacon-api/        # REST API server
│   ├── beacon-chaincode/  # Chaincode execution
│   ├── beacon-crypto/     # Cryptographic utilities
│   └── beacon-node/       # Main node binary
├── chaincode/
│   └── sdk/               # Go SDK for chaincode
├── docker/                # Docker configurations
├── scripts/               # Utility scripts
└── docs/                  # Documentation
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration

# Run performance benchmarks
cargo bench

# Run with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- Documentation: [docs/](docs/)
- Issues: GitHub Issues
- Community: [Community Forum](https://community.beacon.com)
- Email: support@beacon.com
