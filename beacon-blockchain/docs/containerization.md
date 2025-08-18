# BEACON Blockchain - Docker Containerization Guide

## Overview

This guide provides comprehensive instructions for building, deploying, and managing BEACON blockchain networks using Docker containers. We provide both development and production-ready containerization solutions with automated deployment scripts.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Docker Images](#docker-images)
3. [Development Environment](#development-environment)
4. [Production Deployment](#production-deployment)
5. [Configuration Management](#configuration-management)
6. [Monitoring & Logging](#monitoring--logging)
7. [Security Considerations](#security-considerations)
8. [Troubleshooting](#troubleshooting)

## Quick Start

### Prerequisites

- Docker 20.10+ and Docker Compose 2.0+
- Minimum 4GB RAM, 10GB disk space
- PowerShell 5.1+ (Windows) or Bash (Linux/macOS)

### Development Deployment (5 minutes)

```bash
# Linux/macOS
./scripts/build-deploy.sh latest development

# Windows PowerShell
.\scripts\build-deploy.ps1 -Version latest -BuildType development
```

### Production Deployment (10 minutes)

```bash
# Linux/macOS
./scripts/build-deploy.sh latest production

# Windows PowerShell
.\scripts\build-deploy.ps1 -Version latest -BuildType production
```

## Docker Images

### Production Image (`Dockerfile`)

**Features:**

- Multi-stage build for optimized size (~150MB final image)
- Debian slim base for security and performance
- Non-root user execution
- Health checks and monitoring endpoints
- Production-ready configuration

**Build manually:**

```bash
docker build -t beacon-blockchain/beacon-node:latest -f Dockerfile .
```

### Development Image (`Dockerfile.dev`)

**Features:**

- Full Rust development environment
- Hot reload with `cargo watch`
- Debugging tools and utilities
- Volume mounts for source code
- Development-specific configurations

**Build manually:**

```bash
docker build -t beacon-blockchain/beacon-node:dev -f Dockerfile.dev .
```

## Development Environment

### Quick Start Development

```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# View logs
docker-compose -f docker-compose.dev.yml logs -f beacon-dev

# Access development container
docker exec -it beacon-dev /bin/bash
```

### Development Services

| Service              | Port | Description                       |
| -------------------- | ---- | --------------------------------- |
| `beacon-dev`         | 3000 | BEACON API server with hot reload |
| `beacon-db-dev`      | 5432 | PostgreSQL development database   |
| `beacon-redis-dev`   | 6379 | Redis for caching and sessions    |
| `beacon-monitor-dev` | 9090 | Prometheus monitoring             |

### Development Features

- **Hot Reload**: Automatic restart on code changes
- **Debugging**: Full Rust debugging capabilities
- **Volume Mounts**: Source code mounted for editing
- **Database**: Isolated development database
- **Monitoring**: Real-time metrics and logs

### Development Configuration

```yaml
# docker-compose.dev.yml environment variables
environment:
  - BEACON_LOG_LEVEL=debug
  - BEACON_NETWORK_MODE=development
  - RUST_LOG=debug
  - RUST_BACKTRACE=1
```

## Production Deployment

### Multi-Node Production Network

The production deployment creates a highly available 3-node BEACON network with load balancing and monitoring.

```bash
# Deploy production network
docker-compose -f docker-compose.yml up -d

# Scale nodes (if needed)
docker-compose -f docker-compose.yml up -d --scale beacon-node-2=2
```

### Production Services

| Service         | External Port | Internal Port | Description           |
| --------------- | ------------- | ------------- | --------------------- |
| `beacon-lb`     | 80/443        | 80/443        | Nginx load balancer   |
| `beacon-node-1` | 3001          | 3000          | Primary BEACON node   |
| `beacon-node-2` | 3002          | 3000          | Secondary BEACON node |
| `beacon-node-3` | 3003          | 3000          | Tertiary BEACON node  |
| `prometheus`    | 9090          | 9090          | Metrics collection    |
| `grafana`       | 3000          | 3000          | Monitoring dashboards |

### Load Balancer Configuration

The Nginx load balancer provides:

- **High Availability**: Automatic failover between nodes
- **Rate Limiting**: Protection against API abuse
- **SSL Termination**: HTTPS support (when configured)
- **Health Checks**: Automatic node health monitoring

### Production Features

- **High Availability**: 3-node cluster with automatic failover
- **Load Balancing**: Nginx with least-connection routing
- **Monitoring**: Prometheus + Grafana stack
- **Security**: Non-root containers, security headers
- **Persistence**: Data volumes for blockchain storage
- **Backup**: Automated backup capabilities

## Configuration Management

### Environment Variables

#### Core Configuration

```bash
BEACON_NODE_ID=beacon-node-1           # Unique node identifier
BEACON_API_PORT=3000                   # REST API port
BEACON_P2P_PORT=8080                   # P2P network port
BEACON_GRPC_PORT=9000                  # Chaincode gRPC port
BEACON_DISCOVERY_PORT=8081             # Node discovery port
BEACON_DATA_DIR=/data/beacon/storage   # Data directory path
BEACON_LOG_LEVEL=info                  # Logging level
BEACON_NETWORK_MODE=production         # Network mode
```

#### Bootstrap Configuration

```bash
BEACON_BOOTSTRAP_PEERS=node2:8080,node3:8080  # Bootstrap peers
```

### Configuration Files

#### Main Configuration (`docker/config/beacon.toml`)

```toml
[node]
id = "${BEACON_NODE_ID}"
data_dir = "${BEACON_DATA_DIR}"
log_level = "${BEACON_LOG_LEVEL}"

[api]
host = "0.0.0.0"
port = ${BEACON_API_PORT}
enable_cors = true

[network]
p2p_port = ${BEACON_P2P_PORT}
discovery_port = ${BEACON_DISCOVERY_PORT}
bootstrap_peers = "${BEACON_BOOTSTRAP_PEERS}"

[storage]
backend = "rocksdb"
cache_size = "256MB"
compression = "snappy"
```

### Volume Mounts

#### Production Volumes

```yaml
volumes:
  - beacon-node-1-data:/data/beacon/storage # Blockchain data
  - beacon-node-1-logs:/data/beacon/logs # Application logs
  - ./docker/config:/etc/beacon:ro # Configuration files
```

#### Development Volumes

```yaml
volumes:
  - .:/app:cached # Source code (cached)
  - cargo-cache:/usr/local/cargo/registry # Cargo registry cache
  - target-cache:/app/target # Build cache
```

## Monitoring & Logging

### Prometheus Metrics

Available at `http://localhost:9090/metrics`:

- **Node Metrics**: CPU, memory, disk usage
- **API Metrics**: Request rates, response times, error rates
- **Blockchain Metrics**: Block height, transaction counts
- **Network Metrics**: P2P connections, message rates

### Grafana Dashboards

Access Grafana at `http://localhost:3000` (production):

- **System Overview**: Node health and resource usage
- **API Performance**: Request metrics and latency
- **Blockchain State**: Block production and transaction rates
- **Network Topology**: P2P network visualization

### Log Management

#### Log Locations

```bash
# Container logs
docker-compose logs -f beacon-node-1

# Application logs (inside container)
/data/beacon/logs/beacon.log

# System logs
/var/log/beacon/
```

#### Log Configuration

```toml
[logging]
level = "info"
format = "json"
output = "/data/beacon/logs/beacon.log"
max_size = "100MB"
max_files = 10
```

## Security Considerations

### Container Security

1. **Non-root User**: All containers run as non-root user `beacon`
2. **Read-only Filesystems**: Where possible
3. **Resource Limits**: CPU and memory constraints
4. **Network Isolation**: Custom Docker networks

### Network Security

1. **Port Exposure**: Only necessary ports exposed
2. **TLS/SSL**: Configure SSL certificates for HTTPS
3. **Firewall**: Configure host firewall rules
4. **Rate Limiting**: API rate limiting in load balancer

### Data Security

1. **Volume Encryption**: Encrypt Docker volumes at rest
2. **Backup Encryption**: Encrypt backup data
3. **Secret Management**: Use Docker secrets for sensitive data

### Example Security Configuration

```yaml
# docker-compose.yml security additions
services:
  beacon-node-1:
    security_opt:
      - no-new-privileges:true
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m
    ulimits:
      nofile:
        soft: 65536
        hard: 65536
```

## Backup & Recovery

### Backup Scripts

```bash
#!/bin/bash
# Backup BEACON blockchain data
docker run --rm \
  -v beacon-node-1-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/beacon-backup-$(date +%Y%m%d-%H%M%S).tar.gz /data
```

### Recovery Process

```bash
#!/bin/bash
# Restore from backup
docker run --rm \
  -v beacon-node-1-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar xzf /backup/beacon-backup-20241130-120000.tar.gz -C /
```

## Troubleshooting

### Common Issues

#### Container Won't Start

```bash
# Check logs
docker-compose logs beacon-node-1

# Check container status
docker ps -a

# Inspect container
docker inspect beacon-node-1
```

#### Network Connectivity Issues

```bash
# Check network
docker network ls
docker network inspect beacon-network

# Test connectivity
docker exec beacon-node-1 ping beacon-node-2
```

#### Performance Issues

```bash
# Check resource usage
docker stats

# Check container limits
docker exec beacon-node-1 cat /sys/fs/cgroup/memory/memory.limit_in_bytes
```

### Debug Mode

Enable debug mode in development:

```bash
# Set debug environment
export BEACON_LOG_LEVEL=debug
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Restart with debug
docker-compose -f docker-compose.dev.yml restart beacon-dev
```

### Health Checks

```bash
# Manual health check
curl http://localhost:3000/health

# All nodes health check
for port in 3001 3002 3003; do
  echo "Node on port $port:"
  curl -s http://localhost:$port/health | jq .
done
```

## Advanced Configuration

### Custom Networks

```yaml
# Custom network configuration
networks:
  beacon-custom:
    driver: bridge
    ipam:
      driver: default
      config:
        - subnet: 172.30.0.0/16
          gateway: 172.30.0.1
```

### Resource Limits

```yaml
# Resource constraints
services:
  beacon-node-1:
    deploy:
      resources:
        limits:
          cpus: "2.0"
          memory: 2G
        reservations:
          cpus: "1.0"
          memory: 1G
```

### SSL Configuration

```yaml
# SSL/TLS configuration
services:
  beacon-lb:
    volumes:
      - ./ssl/cert.pem:/etc/nginx/ssl/cert.pem:ro
      - ./ssl/key.pem:/etc/nginx/ssl/key.pem:ro
```

## Performance Tuning

### Production Optimizations

1. **CPU Affinity**: Pin containers to specific CPU cores
2. **Memory**: Optimize JVM heap sizes and caching
3. **Storage**: Use SSD storage for better I/O performance
4. **Network**: Tune network buffer sizes

### Example Performance Configuration

```yaml
# High-performance production setup
services:
  beacon-node-1:
    cpuset: "0,1" # Use specific CPU cores
    mem_limit: 4g
    memswap_limit: 4g
    volumes:
      - type: tmpfs
        target: /tmp
        tmpfs:
          size: 1g
```

## Deployment Scripts

### Build and Deploy Script Features

- **Automated**: One-command deployment
- **Validation**: Health checks and service validation
- **Cross-platform**: Both Bash and PowerShell versions
- **Configurable**: Version and environment selection
- **Monitoring**: Built-in health monitoring

### Usage Examples

```bash
# Development deployment
./scripts/build-deploy.sh v1.0.0 development

# Production deployment with specific version
./scripts/build-deploy.sh v1.0.0 production

# Windows PowerShell
.\scripts\build-deploy.ps1 -Version "v1.0.0" -BuildType "production"
```

---

## Next Steps

1. **Web Interface**: Deploy web-based admin console
2. **Kubernetes**: Migrate to Kubernetes for orchestration
3. **CI/CD**: Implement automated build and deployment pipelines
4. **Monitoring**: Advanced alerting and dashboard configuration
5. **Security**: Implement advanced security measures and auditing

For more information, see:

- [Network Access Guide](./access/README.md)
- [Deployment Guide](./access/deployment.md)
- [Network Administration](./access/network-admin.md)
