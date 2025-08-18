# BEACON Blockchain Documentation

## Overview

BEACON is a permissioned blockchain platform designed for enterprise applications with support for Go chaincodes, REST API integration, and comprehensive state management.

## Documentation Structure

### 📚 Core Documentation

- **[Architecture](architecture/)** - System architecture and design principles
- **[API Reference](api/)** - Complete REST API documentation
- **[Development Guide](development/)** - Developer setup and contribution guidelines
- **[Testing](testing/)** - Testing strategies and test documentation
- **[Access & Deployment](access/)** - Network access, administration, and deployment guides
- **[Containerization](containerization.md)** - Docker deployment and container management

### 🚀 Quick Start

1. **[Installation Guide](development/installation.md)** - Set up your development environment
2. **[API Quick Start](api/quickstart.md)** - Get started with the REST API
3. **[Chaincode Development](development/chaincode.md)** - Write your first chaincode
4. **[Testing Guide](testing/test-guide.md)** - Run and write tests
5. **[Network Access](access/README.md)** - Connect to and interact with BEACON networks
6. **[Docker Deployment](containerization.md)** - Deploy with Docker containers

### 🏗️ Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   REST API      │    │   Go SDK        │    │   Rust Core     │
│   (Axum)        │◄──►│   (Chaincode)   │◄──►│   (Blockchain)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web UI        │    │   gRPC Shim     │    │   RocksDB       │
│   (Future)      │    │   Service       │    │   Storage       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 🔧 Core Components

| Component              | Language   | Purpose                                   | Status      |
| ---------------------- | ---------- | ----------------------------------------- | ----------- |
| **Blockchain Core**    | Rust       | Core blockchain logic, consensus, storage | ✅ Complete |
| **Networking**         | Rust       | P2P communication, node discovery         | ✅ Complete |
| **Storage**            | Rust       | RocksDB integration, state management     | ✅ Complete |
| **Chaincode Executor** | Rust       | Go process management, gRPC server        | ✅ Complete |
| **Go SDK**             | Go         | Chaincode development framework           | ✅ Complete |
| **REST API**           | Rust       | HTTP API server with authentication       | ✅ Complete |
| **Documentation**      | Markdown   | Comprehensive guides and references       | ✅ Complete |
| **Testing Framework**  | Rust       | Multi-layered testing infrastructure      | ✅ Complete |
| **Containerization**   | Docker     | Production-ready container deployment     | ✅ Complete |
| **Web UI**             | JavaScript | Admin console and explorer                | 🔄 Planned  |

### 📊 Example Use Cases

- **[Gateway Management](../sdk/go/examples/gateway-management/)** - Network infrastructure control
- **[Supply Chain](../sdk/go/examples/supply-chain/)** - Product tracking and provenance
- **[Identity Verification](../sdk/go/examples/identity-verification/)** - Digital credentials

### 🧪 Testing

- **[Unit Tests](testing/unit-tests.md)** - Component-level testing
- **[Integration Tests](testing/integration-tests.md)** - End-to-end testing
- **[API Tests](testing/api-tests.md)** - REST API testing
- **[Performance Tests](testing/performance-tests.md)** - Load and stress testing

### 🚀 Deployment

- **[Network Access](access/README.md)** - Comprehensive network access guide
- **[Docker Deployment](containerization.md)** - Complete containerization guide
- **[Network Administration](access/network-admin.md)** - Network management and administration
- **[Production Deployment](access/deployment.md)** - Production deployment strategies

### 📝 Contributing

- **[Development Workflow](development/workflow.md)** - Git workflow and standards
- **[Code Style](development/style-guide.md)** - Coding standards and conventions
- **[Release Process](development/releases.md)** - Version management and releases

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/beacon-blockchain/beacon/issues)
- **Discussions**: [GitHub Discussions](https://github.com/beacon-blockchain/beacon/discussions)
- **Documentation**: This documentation site

## License

BEACON is licensed under the Apache 2.0 License. See [LICENSE](../LICENSE) for details.
