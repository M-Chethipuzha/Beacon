# BEACON Docker Image Creation Guide

This comprehensive guide covers building, managing, and deploying BEACON blockchain Docker images for both development and production environments.

## 📋 Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Image Architecture](#image-architecture)
- [Building Images](#building-images)
- [Image Variants](#image-variants)
- [Configuration](#configuration)
- [Running Containers](#running-containers)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)
- [Advanced Usage](#advanced-usage)

## 🔍 Overview

The BEACON blockchain uses a multi-stage Docker build approach that creates optimized, secure, and lightweight container images. The production image is approximately 150MB and includes both the API server and blockchain node components.

### Key Features

- **Multi-stage build**: Separate build and runtime stages for optimization
- **Security focused**: Non-root user execution with minimal attack surface
- **Health monitoring**: Built-in health checks and monitoring endpoints
- **Configurable**: Template-based configuration with environment variable support
- **Production ready**: Optimized for high-performance blockchain operations

## 🛠 Prerequisites

### System Requirements

- Docker Engine 20.10+ or Docker Desktop
- 8GB+ RAM available for build process
- 20GB+ free disk space
- Internet connection for dependency downloads

### Development Tools (Optional)

```bash
# For advanced image management
docker buildx install    # Multi-platform builds
dive                     # Image layer analysis
hadolint                 # Dockerfile linting
```

### Verify Prerequisites

```powershell
# Check Docker installation
docker --version
docker compose --version

# Verify system resources
docker system df
docker system info
```

## 🏗 Image Architecture

### Multi-Stage Build Structure

```
┌─────────────────┐    ┌─────────────────┐
│   Build Stage   │───▶│  Runtime Stage  │
│                 │    │                 │
│ • Rust compiler │    │ • Minimal OS    │
│ • Dependencies  │    │ • Runtime deps  │
│ • Source code   │    │ • Binary files  │
│ • Build tools   │    │ • Config files  │
└─────────────────┘    └─────────────────┘
    ~2.5GB                   ~150MB
```

### Directory Structure in Container

```
/usr/local/bin/
├── beacon-api          # API server binary
└── beacon-node         # Blockchain node binary

/etc/beacon/            # Configuration templates
├── api.toml
├── node.toml
└── network.toml

/data/beacon/           # Persistent data
├── storage/            # Blockchain data
├── logs/               # Application logs
└── config/             # Runtime configuration
```

## 🔨 Building Images

### Basic Production Build

```bash
# Build production image
docker build -t beacon-blockchain:latest .

# Build with specific tag
docker build -t beacon-blockchain:v1.0.0 .

# Build with build arguments
docker build \
  --build-arg RUST_VERSION=1.75 \
  --build-arg BUILD_MODE=release \
  -t beacon-blockchain:latest .
```

### Development Build

```bash
# Build development image with hot reload
docker build -f Dockerfile.dev -t beacon-blockchain:dev .

# Build with debug symbols
docker build \
  --build-arg BUILD_MODE=debug \
  --target development \
  -t beacon-blockchain:debug .
```

### Multi-Platform Build

```bash
# Enable buildx
docker buildx create --use

# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --tag beacon-blockchain:multi \
  --push .
```

### Build with Cache Optimization

```bash
# Use cache mount for faster builds
docker build \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/app/target \
  -t beacon-blockchain:cached .
```

## 🔄 Image Variants

### Production Image (Dockerfile)

- **Size**: ~150MB
- **Base**: debian:bullseye-slim
- **Use Case**: Production deployments
- **Features**: Optimized binaries, minimal dependencies

```bash
docker build -t beacon-blockchain:prod .
```

### Development Image (Dockerfile.dev)

- **Size**: ~800MB
- **Base**: rust:1.75-bullseye
- **Use Case**: Development and debugging
- **Features**: Development tools, source code, hot reload

```bash
docker build -f Dockerfile.dev -t beacon-blockchain:dev .
```

### Debug Image

- **Size**: ~300MB
- **Base**: debian:bullseye-slim
- **Use Case**: Production debugging
- **Features**: Debug symbols, debugging tools

```bash
docker build \
  --build-arg BUILD_MODE=debug \
  --target debug \
  -t beacon-blockchain:debug .
```

## ⚙ Configuration

### Build Arguments

| Argument       | Default   | Description                |
| -------------- | --------- | -------------------------- |
| `RUST_VERSION` | `1.75`    | Rust compiler version      |
| `BUILD_MODE`   | `release` | Build mode (release/debug) |
| `FEATURES`     | `default` | Cargo features to enable   |
| `TARGET_ARCH`  | `x86_64`  | Target architecture        |

```bash
# Example with custom build arguments
docker build \
  --build-arg RUST_VERSION=1.76 \
  --build-arg BUILD_MODE=release \
  --build-arg FEATURES="metrics,tracing" \
  -t beacon-blockchain:custom .
```

### Environment Variables

| Variable       | Default    | Description           |
| -------------- | ---------- | --------------------- |
| `RUST_LOG`     | `info`     | Logging level         |
| `DATABASE_URL` | `:memory:` | Database connection   |
| `API_PORT`     | `3000`     | API server port       |
| `NODE_PORT`    | `8080`     | P2P node port         |
| `METRICS_PORT` | `9000`     | Metrics endpoint port |

### Configuration Templates

The image includes configuration templates in `/etc/beacon/`:

```toml
# /etc/beacon/api.toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "${DATABASE_URL}"

[security]
jwt_secret = "${JWT_SECRET}"
```

## 🚀 Running Containers

### Basic Container Run

```bash
# Run API server
docker run -d \
  --name beacon-api \
  -p 3000:3000 \
  beacon-blockchain:latest

# Run blockchain node
docker run -d \
  --name beacon-node \
  -p 8080:8080 \
  beacon-blockchain:latest beacon-node
```

### Production Deployment

```bash
# Run with persistent storage
docker run -d \
  --name beacon-production \
  -p 3000:3000 \
  -p 8080:8080 \
  -p 9000:9000 \
  -v beacon-data:/data/beacon \
  -e RUST_LOG=info \
  -e DATABASE_URL="postgresql://user:pass@db:5432/beacon" \
  --restart unless-stopped \
  beacon-blockchain:latest
```

### Development Environment

```bash
# Run development container with source mount
docker run -it \
  --name beacon-dev \
  -p 3000:3000 \
  -v $(pwd):/app \
  -v cargo-cache:/usr/local/cargo/registry \
  beacon-blockchain:dev
```

### Health Monitoring

```bash
# Check container health
docker inspect beacon-api | grep Health

# View health check logs
docker logs beacon-api | grep health
```

## 📊 Best Practices

### Build Optimization

```bash
# Use .dockerignore to exclude unnecessary files
echo "target/
.git/
*.log
node_modules/" > .dockerignore

# Multi-stage builds for smaller images
# Layer caching for faster builds
# Use specific base image tags (not 'latest')
```

### Security Practices

```bash
# Scan for vulnerabilities
docker scout cves beacon-blockchain:latest

# Run security scan
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  anchore/grype beacon-blockchain:latest
```

### Resource Management

```bash
# Set resource limits
docker run -d \
  --name beacon-limited \
  --memory=2g \
  --cpus=1.5 \
  beacon-blockchain:latest

# Monitor resource usage
docker stats beacon-api
```

### Image Maintenance

```bash
# Clean up build cache
docker builder prune

# Remove unused images
docker image prune -a

# Regular security updates
docker build --no-cache --pull -t beacon-blockchain:latest .
```

## 🔧 Troubleshooting

### Common Build Issues

#### Issue: Missing System Dependencies (libclang, protobuf, etc.)

**Error**: `Unable to find libclang` or similar C library errors

```bash
# For Rust projects requiring system dependencies
# Ensure Dockerfile includes all required system packages:

# Debian/Ubuntu based images
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Alpine based images (alternative)
RUN apk add --no-cache \
    build-base \
    pkgconfig \
    openssl-dev \
    protobuf-dev \
    cmake \
    clang-dev \
    llvm-dev
```

#### Issue: Rust Compilation Errors

**Error**: Function signature mismatches or missing imports

Common compilation issues and solutions:

```rust
// Error: ApiServer::new() requires multiple arguments
// Solution: Check the function signature and provide all required parameters

// Before (incorrect):
let api_server = ApiServer::new(bind_addr);

// After (correct):
let api_server = ApiServer::new(
    bind_addr,
    database.clone(),
    chaincode_executor.clone()
);
```

**Error**: Missing imports for chaincode components

```rust
// Add required imports to beacon-node/src/node.rs:
use beacon_chaincode::{ChaincodeExecutor, ChaincodeExecutorConfig, ChaincodeShimService};

// Initialize chaincode services properly:
let chaincode_shim_service = Arc::new(ChaincodeShimService::new(state_storage.clone()));
let chaincode_config = ChaincodeExecutorConfig::default();
let chaincode_executor = Arc::new(ChaincodeExecutor::new(chaincode_config, chaincode_shim_service));
```

**Error**: Missing struct fields

```rust
// Add required fields to BeaconNode struct:
pub struct BeaconNode {
    // ... existing fields ...
    chaincode_executor: Arc<ChaincodeExecutor>,
}
```

#### Issue: Build Context/File Not Found Errors

**Error**: `"/Cargo.lock": not found` or similar file missing errors

```bash
# Check .dockerignore file - ensure required files aren't excluded
cat .dockerignore

# Common issue: Cargo.lock excluded but needed for Docker builds
# Remove Cargo.lock from .dockerignore for reproducible builds

# Verify build context includes necessary files
docker build --dry-run -t beacon-blockchain .

# For debugging, check what files are sent to Docker daemon
docker build --no-cache --progress=plain -t beacon-blockchain . 2>&1 | grep "transferring context"
```

#### Issue: Network Connectivity/DNS Resolution Failures

**Error**: `failed to resolve source metadata` or `no such host` for docker.io/registry-1.docker.io

```bash
# Check Docker daemon status
docker info

# Test basic connectivity
docker run --rm hello-world

# Solution 1: Restart Docker Desktop
# Windows/Mac: Right-click Docker icon → Restart Docker Desktop

# Solution 2: Configure DNS in Docker Desktop
# Windows/Mac: Docker Desktop → Settings → Resources → Network
# Set DNS to: 8.8.8.8, 8.8.4.4

# Solution 3: Use different registry or mirror
docker build --build-arg REGISTRY_MIRROR=mirror.gcr.io -t beacon-blockchain .

# Solution 4: Corporate network - configure proxy
docker build \
  --build-arg HTTP_PROXY=http://proxy:8080 \
  --build-arg HTTPS_PROXY=http://proxy:8080 \
  --build-arg NO_PROXY=localhost,127.0.0.1 \
  -t beacon-blockchain .

# Solution 5: Reset Docker to factory defaults (last resort)
# Docker Desktop → Settings → Reset to factory defaults
```

#### Issue: Out of Memory During Build

```bash
# Solution: Increase Docker memory limit
# Windows/Mac: Docker Desktop → Resources → Memory
# Linux: Configure Docker daemon

# Alternative: Use smaller build context
docker build --context . --file Dockerfile -t beacon .
```

#### Issue: Dependency Download Failures

```bash
# Solution: Use proxy or cache
docker build \
  --build-arg HTTP_PROXY=http://proxy:8080 \
  --build-arg HTTPS_PROXY=http://proxy:8080 \
  -t beacon-blockchain .
```

#### Issue: Platform Compatibility

```bash
# Solution: Specify target platform
docker build --platform linux/amd64 -t beacon-blockchain .
```

### Runtime Issues

#### Issue: Health Check Failures

```bash
# Debug health check
docker exec beacon-api curl -f http://localhost:3000/health

# Check logs
docker logs beacon-api --tail 50

# Inspect container
docker exec -it beacon-api sh
```

#### Issue: Port Binding Conflicts

```bash
# Check port usage
netstat -tulpn | grep :3000

# Use different ports
docker run -p 3001:3000 beacon-blockchain
```

#### Issue: Permission Denied

```bash
# Check file permissions
docker exec beacon-api ls -la /data/beacon

# Fix ownership
docker exec --user root beacon-api chown -R beacon:beacon /data/beacon
```

### Debugging Commands

```bash
# Enter container for debugging
docker exec -it beacon-api bash

# Check process status
docker exec beacon-api ps aux

# View real-time logs
docker logs -f beacon-api

# Inspect image layers
dive beacon-blockchain:latest

# Analyze image size
docker images beacon-blockchain --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
```

## 🚀 Advanced Usage

### CI/CD Integration

```yaml
# GitHub Actions example
name: Build Docker Image
on: [push, pull_request]

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: |
            beacon-blockchain:latest
            beacon-blockchain:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### Custom Build Scripts

```bash
#!/bin/bash
# build-beacon.sh - Custom build script

set -e

VERSION=${1:-latest}
BUILD_TYPE=${2:-production}

echo "Building BEACON Docker image v${VERSION} (${BUILD_TYPE})"

case $BUILD_TYPE in
  "production")
    DOCKERFILE="Dockerfile"
    ;;
  "development")
    DOCKERFILE="Dockerfile.dev"
    ;;
  "debug")
    DOCKERFILE="Dockerfile"
    BUILD_ARGS="--build-arg BUILD_MODE=debug"
    ;;
esac

docker build \
  ${BUILD_ARGS} \
  -f ${DOCKERFILE} \
  -t beacon-blockchain:${VERSION} \
  .

echo "Build completed: beacon-blockchain:${VERSION}"
```

### Multi-Environment Configuration

```bash
# Production build
docker build \
  --build-arg BUILD_MODE=release \
  --build-arg FEATURES="production,metrics" \
  -t beacon-blockchain:prod .

# Staging build
docker build \
  --build-arg BUILD_MODE=release \
  --build-arg FEATURES="staging,debug-api" \
  -t beacon-blockchain:staging .

# Development build
docker build \
  -f Dockerfile.dev \
  --build-arg FEATURES="development,hot-reload" \
  -t beacon-blockchain:dev .
```

### Performance Optimization

```bash
# Use buildkit for better performance
export DOCKER_BUILDKIT=1

# Parallel builds
docker build --jobs 4 -t beacon-blockchain .

# Build with cache mounts
docker build \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/app/target \
  -t beacon-blockchain .
```

## 📈 Monitoring and Metrics

### Image Analysis

```bash
# Analyze image efficiency
docker history beacon-blockchain:latest

# Detailed layer inspection
dive beacon-blockchain:latest

# Security scan
docker scout quickview beacon-blockchain:latest
```

### Runtime Monitoring

```bash
# Container metrics
docker stats beacon-api --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}"

# Health status
docker inspect beacon-api --format='{{.State.Health.Status}}'

# Application metrics
curl http://localhost:9000/metrics
```

## ✅ Build Success and Verification

### Successful Build Results

After resolving all compilation and dependency issues, the Docker build should complete successfully:

```
[+] Building 147.6s (20/20) FINISHED
 => exporting to image                                                                                               0.2s
 => => writing image sha256:6a7c6abeda6a7872428351c7b500d0574231870d71a206c95d043781aee803ba                         0.0s
 => => naming to docker.io/library/beacon-blockchain:latest                                                          0.0s
```

### Image Verification

Verify the created image:

```bash
# Check image details
docker images beacon-blockchain

# Expected output:
# REPOSITORY          TAG       IMAGE ID       CREATED              SIZE
# beacon-blockchain   latest    6a7c6abeda6a   About a minute ago   116MB

# Test binary functionality
docker run --rm beacon-blockchain:latest beacon-node --help
docker run --rm beacon-blockchain:latest beacon-api --help
```

### Running the Container

For proper operation, mount volumes for data persistence:

```bash
# Create data directories
mkdir -p ./beacon_data/{storage,logs,config}

# Run beacon-node with proper volume mounts
docker run -d \
  --name beacon-node \
  -p 8080:8080 \
  -p 30303:30303 \
  -v $(pwd)/beacon_data:/data/beacon \
  beacon-blockchain:latest \
  beacon-node --data-dir /data/beacon/storage

# Run beacon-api with volume mounts
docker run -d \
  --name beacon-api \
  -p 3000:3000 \
  -v $(pwd)/beacon_data:/data/beacon \
  beacon-blockchain:latest \
  beacon-api --data-dir /data/beacon/storage
```

### Production Deployment Notes

1. **Resource Requirements**: ~116MB final image size
2. **Build Time**: Approximately 2.5 minutes on modern hardware
3. **Security**: Runs as non-root `beacon` user
4. **Data Persistence**: Requires volume mounts for database storage
5. **Networking**: Default ports 8080 (API) and 30303 (P2P)

## 🔗 Related Documentation

- [Docker Compose Guide](./containerization.md)
- [Production Deployment](./deployment.md)
- [API Documentation](./api.md)
- [Configuration Reference](./configuration.md)
- [Monitoring Setup](./monitoring.md)

## 📞 Support

For issues or questions regarding Docker image creation:

1. Check the [troubleshooting section](#troubleshooting)
2. Review [container logs](#debugging-commands)
3. Consult the [GitHub Issues](https://github.com/beacon-blockchain/beacon/issues)
4. Join our [Discord community](https://discord.gg/beacon)

---

**Version**: 1.0.0  
**Last Updated**: July 2025  
**Maintained by**: BEACON Development Team
