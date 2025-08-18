# BEACON Network Administration Guide

## Overview

This guide provides comprehensive information for network administrators managing BEACON blockchain nodes, network topology, and infrastructure operations.

## Network Architecture

### BEACON Network Components

```
                    ┌─────────────────────────────────┐
                    │         BEACON Network          │
                    │                                 │
    ┌───────────────┼─────────────────────────────────┼───────────────┐
    │               │                                 │               │
    │ ┌─────────────▼─────────────┐ ┌─────────────────▼─────────────┐ │
    │ │     Validator Node 1      │ │     Validator Node 2          │ │
    │ │                           │ │                               │ │
    │ │ - REST API (:3000)        │ │ - REST API (:3000)            │ │
    │ │ - P2P Network (:8080)     │ │ - P2P Network (:8080)         │ │
    │ │ - Chaincode gRPC (:9000)  │ │ - Chaincode gRPC (:9000)      │ │
    │ │ - Admin Interface (:8082) │ │ - Admin Interface (:8082)     │ │
    │ └───────────────────────────┘ └───────────────────────────────┘ │
    │                                                                 │
    │ ┌───────────────────────────┐ ┌───────────────────────────────┐ │
    │ │     Regular Node 1        │ │     Regular Node 2            │ │
    │ │                           │ │                               │ │
    │ │ - REST API (:3000)        │ │ - REST API (:3000)            │ │
    │ │ - P2P Network (:8080)     │ │ - P2P Network (:8080)         │ │
    │ │ - Chaincode gRPC (:9000)  │ │ - Chaincode gRPC (:9000)      │ │
    │ └───────────────────────────┘ └───────────────────────────────┘ │
    └─────────────────────────────────────────────────────────────────┘
```

### Port Configuration

| Service         | Port | Protocol   | Description                |
| --------------- | ---- | ---------- | -------------------------- |
| REST API        | 3000 | HTTP/HTTPS | External API access        |
| P2P Network     | 8080 | TCP        | Node-to-node communication |
| Discovery       | 8081 | UDP        | Network peer discovery     |
| Admin Interface | 8082 | HTTP       | Administrative operations  |
| Chaincode gRPC  | 9000 | gRPC       | Chaincode execution        |
| Metrics         | 9090 | HTTP       | Prometheus metrics         |

## Node Management

### Node Installation

#### System Requirements

```bash
# Minimum requirements
- CPU: 4 cores
- RAM: 8GB
- Storage: 100GB SSD
- Network: 1Gbps
- OS: Ubuntu 20.04+ / CentOS 8+ / RHEL 8+

# Recommended requirements
- CPU: 8 cores
- RAM: 16GB
- Storage: 500GB NVMe SSD
- Network: 10Gbps
- OS: Ubuntu 22.04 LTS
```

#### Installation Script

```bash
#!/bin/bash
# install-beacon-node.sh

# Download BEACON node binary
wget https://releases.beacon-blockchain.com/v0.1.0/beacon-node-linux-amd64.tar.gz
tar -xzf beacon-node-linux-amd64.tar.gz

# Install system dependencies
sudo apt update
sudo apt install -y build-essential libssl-dev pkg-config

# Create beacon user
sudo useradd -r -s /bin/false beacon
sudo mkdir -p /opt/beacon /var/lib/beacon /var/log/beacon
sudo chown beacon:beacon /opt/beacon /var/lib/beacon /var/log/beacon

# Install binary
sudo cp beacon-node /opt/beacon/
sudo chmod +x /opt/beacon/beacon-node

# Create systemd service
sudo tee /etc/systemd/system/beacon-node.service > /dev/null <<EOF
[Unit]
Description=BEACON Blockchain Node
After=network.target

[Service]
Type=simple
User=beacon
Group=beacon
WorkingDirectory=/opt/beacon
ExecStart=/opt/beacon/beacon-node --config /etc/beacon/config.yaml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable beacon-node
sudo systemctl start beacon-node
```

### Node Configuration

#### Main Configuration File

```yaml
# /etc/beacon/config.yaml
node:
  id: "beacon-node-1"
  data_dir: "/var/lib/beacon"
  log_level: "info"
  log_file: "/var/log/beacon/node.log"

network:
  # P2P configuration
  p2p:
    port: 8080
    bind_address: "0.0.0.0"
    external_address: "192.168.1.10:8080"
    max_peers: 50
    connection_timeout: 30s

  # Discovery configuration
  discovery:
    enabled: true
    port: 8081
    bootstrap_peers:
      - "bootstrap-1.beacon-network.com:8080"
      - "bootstrap-2.beacon-network.com:8080"
    announce_interval: 60s

  # API configuration
  api:
    port: 3000
    bind_address: "0.0.0.0"
    tls:
      enabled: false
      cert_file: "/etc/beacon/tls/cert.pem"
      key_file: "/etc/beacon/tls/key.pem"
    cors:
      enabled: true
      allowed_origins: ["*"]
    rate_limiting:
      enabled: true
      requests_per_minute: 1000

# Database configuration
database:
  type: "rocksdb"
  path: "/var/lib/beacon/data"
  cache_size: "1GB"
  write_buffer_size: "256MB"

# Consensus configuration
consensus:
  type: "PoA"
  validator: false # Set to true for validator nodes
  validator_key: "/etc/beacon/keys/validator.key"
  block_time: "5s"

# Chaincode configuration
chaincode:
  grpc_port: 9000
  execution_timeout: 30s
  max_concurrent: 10

# Monitoring configuration
monitoring:
  metrics:
    enabled: true
    port: 9090
    path: "/metrics"
  health_checks:
    enabled: true
    interval: 30s
```

### Node Operations

#### Starting/Stopping Nodes

```bash
# Start node
sudo systemctl start beacon-node

# Stop node
sudo systemctl stop beacon-node

# Restart node
sudo systemctl restart beacon-node

# Check status
sudo systemctl status beacon-node

# View logs
sudo journalctl -u beacon-node -f
```

#### Node Health Monitoring

```bash
#!/bin/bash
# health-check.sh

NODE_URL="http://localhost:3000"

# Basic health check
HEALTH=$(curl -s "$NODE_URL/health" | jq -r '.status')
if [ "$HEALTH" != "healthy" ]; then
    echo "ERROR: Node health check failed"
    exit 1
fi

# Check API connectivity
API_RESPONSE=$(curl -s -w "%{http_code}" "$NODE_URL/info" -o /dev/null)
if [ "$API_RESPONSE" != "200" ]; then
    echo "ERROR: API not responding"
    exit 1
fi

# Check peer connectivity
PEER_COUNT=$(curl -s "$NODE_URL/api/v1/network/peers" | jq '.peers | length')
if [ "$PEER_COUNT" -lt 1 ]; then
    echo "WARNING: Low peer count: $PEER_COUNT"
fi

echo "Node health check passed"
```

## Network Management

### Network Topology Management

#### Adding New Nodes

```bash
# 1. Configure new node with bootstrap peers
# 2. Start node with discovery enabled
# 3. Verify peer connections

# Add node to network
curl -X POST "http://admin-node:8082/admin/network/add-node" \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "new-node-1",
    "address": "192.168.1.20:8080",
    "role": "regular",
    "metadata": {
      "location": "datacenter-2",
      "operator": "admin@company.com"
    }
  }'
```

#### Removing Nodes

```bash
# Graceful node removal
curl -X POST "http://admin-node:8082/admin/network/remove-node" \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "old-node-1",
    "drain_time": "300s",
    "notify_peers": true
  }'

# Force removal (emergency)
curl -X DELETE "http://admin-node:8082/admin/network/nodes/old-node-1?force=true"
```

### Validator Management

#### Adding Validators

```bash
# Generate validator key
./beacon-keygen --type validator --output /etc/beacon/keys/validator.key

# Register validator
curl -X POST "http://admin-node:8082/admin/consensus/add-validator" \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "validator-node-3",
    "public_key": "0x1234567890abcdef...",
    "stake_amount": "1000000",
    "metadata": {
      "operator": "validator@company.com",
      "commission": "5%"
    }
  }'
```

#### Validator Operations

```bash
# Check validator status
curl "http://validator-node:8082/admin/consensus/status"

# Temporarily disable validator
curl -X POST "http://validator-node:8082/admin/consensus/disable" \
  -d '{"reason": "maintenance", "duration": "1h"}'

# Re-enable validator
curl -X POST "http://validator-node:8082/admin/consensus/enable"
```

## Security Management

### Access Control

#### User Management

```bash
# Create admin user
curl -X POST "http://admin-node:8082/admin/users" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "network-admin",
    "password": "secure-password",
    "role": "admin",
    "permissions": [
      "network:manage",
      "nodes:manage",
      "users:manage",
      "consensus:manage"
    ]
  }'

# Create read-only monitoring user
curl -X POST "http://admin-node:8082/admin/users" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "monitor",
    "password": "monitor-password",
    "role": "monitor",
    "permissions": [
      "network:read",
      "metrics:read",
      "logs:read"
    ]
  }'
```

#### Role-Based Permissions

```yaml
# roles.yaml
roles:
  admin:
    permissions:
      - "network:*"
      - "nodes:*"
      - "users:*"
      - "consensus:*"
      - "chaincode:*"

  operator:
    permissions:
      - "network:read"
      - "nodes:manage"
      - "metrics:read"
      - "logs:read"

  monitor:
    permissions:
      - "network:read"
      - "metrics:read"
      - "logs:read"

  developer:
    permissions:
      - "chaincode:deploy"
      - "chaincode:invoke"
      - "transactions:submit"
      - "state:query"
```

### Network Security

#### Firewall Configuration

```bash
# Configure iptables for BEACON node
sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT  # REST API
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT  # P2P
sudo iptables -A INPUT -p udp --dport 8081 -j ACCEPT  # Discovery
sudo iptables -A INPUT -p tcp --dport 8082 -s 10.0.0.0/8 -j ACCEPT  # Admin (internal only)
sudo iptables -A INPUT -p tcp --dport 9000 -s 127.0.0.1 -j ACCEPT  # gRPC (localhost only)

# Save rules
sudo iptables-save > /etc/iptables/rules.v4
```

#### TLS Configuration

```bash
# Generate TLS certificates
openssl req -x509 -newkey rsa:4096 -keyout /etc/beacon/tls/key.pem \
  -out /etc/beacon/tls/cert.pem -days 365 -nodes \
  -subj "/CN=beacon-node-1.example.com"

# Update config for TLS
cat >> /etc/beacon/config.yaml <<EOF
network:
  api:
    tls:
      enabled: true
      cert_file: "/etc/beacon/tls/cert.pem"
      key_file: "/etc/beacon/tls/key.pem"
EOF
```

## Monitoring and Alerting

### System Monitoring

#### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: "beacon-nodes"
    static_configs:
      - targets:
          - "beacon-node-1:9090"
          - "beacon-node-2:9090"
          - "beacon-node-3:9090"
    scrape_interval: 10s
    metrics_path: /metrics
```

#### Key Metrics to Monitor

```yaml
# Key performance indicators
metrics:
  - beacon_node_health_status
  - beacon_peer_count
  - beacon_block_height
  - beacon_transaction_pool_size
  - beacon_consensus_round_duration
  - beacon_chaincode_execution_time
  - beacon_api_request_duration
  - beacon_storage_size_bytes
  - beacon_network_bandwidth_bytes
```

#### Alerting Rules

```yaml
# alerting.yml
groups:
  - name: beacon-alerts
    rules:
      - alert: BeaconNodeDown
        expr: up{job="beacon-nodes"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "BEACON node is down"

      - alert: LowPeerCount
        expr: beacon_peer_count < 3
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low peer count: {{ $value }}"

      - alert: HighTransactionPoolSize
        expr: beacon_transaction_pool_size > 1000
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High transaction pool size: {{ $value }}"
```

### Log Management

#### Centralized Logging

```yaml
# filebeat.yml
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /var/log/beacon/*.log
    fields:
      service: beacon-node
      environment: production

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "beacon-logs-%{+yyyy.MM.dd}"
```

#### Log Analysis Queries

```bash
# Search for errors in the last hour
grep -E "ERROR|FATAL" /var/log/beacon/node.log | tail -100

# Analyze API response times
awk '/API_REQUEST/ {print $5}' /var/log/beacon/api.log | sort -n | tail -10

# Monitor consensus operations
grep "CONSENSUS" /var/log/beacon/node.log | tail -20
```

## Backup and Recovery

### Data Backup

#### Automated Backup Script

```bash
#!/bin/bash
# backup-beacon-data.sh

BACKUP_DIR="/backup/beacon"
DATA_DIR="/var/lib/beacon"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="beacon_backup_$DATE.tar.gz"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Stop node for consistent backup
sudo systemctl stop beacon-node

# Create compressed backup
tar -czf "$BACKUP_DIR/$BACKUP_FILE" -C "$DATA_DIR" .

# Restart node
sudo systemctl start beacon-node

# Cleanup old backups (keep last 7 days)
find "$BACKUP_DIR" -name "beacon_backup_*.tar.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_DIR/$BACKUP_FILE"
```

#### Recovery Procedure

```bash
#!/bin/bash
# restore-beacon-data.sh

BACKUP_FILE="$1"
DATA_DIR="/var/lib/beacon"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

# Stop node
sudo systemctl stop beacon-node

# Backup current data (safety)
sudo mv "$DATA_DIR" "${DATA_DIR}.backup.$(date +%s)"

# Create new data directory
sudo mkdir -p "$DATA_DIR"
sudo chown beacon:beacon "$DATA_DIR"

# Restore from backup
sudo tar -xzf "$BACKUP_FILE" -C "$DATA_DIR"
sudo chown -R beacon:beacon "$DATA_DIR"

# Start node
sudo systemctl start beacon-node

echo "Recovery completed from: $BACKUP_FILE"
```

## Performance Optimization

### Node Performance Tuning

#### System Optimization

```bash
# Optimize kernel parameters
cat >> /etc/sysctl.conf <<EOF
# Network optimization
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216

# File descriptor limits
fs.file-max = 1000000
EOF

sysctl -p

# Optimize systemd limits
cat >> /etc/systemd/system/beacon-node.service.d/override.conf <<EOF
[Service]
LimitNOFILE=1000000
LimitNPROC=1000000
EOF
```

#### Database Optimization

```yaml
# Optimized database configuration
database:
  type: "rocksdb"
  path: "/var/lib/beacon/data"
  cache_size: "4GB"
  write_buffer_size: "512MB"
  max_write_buffer_number: 4
  max_background_jobs: 8
  compression: "snappy"
  block_size: "64KB"
  bloom_filter_bits_per_key: 10
```

### Network Optimization

#### Bandwidth Management

```bash
# QoS configuration for BEACON traffic
tc qdisc add dev eth0 root handle 1: htb default 30
tc class add dev eth0 parent 1: classid 1:1 htb rate 1gbit
tc class add dev eth0 parent 1:1 classid 1:10 htb rate 800mbit ceil 1gbit
tc class add dev eth0 parent 1:1 classid 1:20 htb rate 150mbit ceil 200mbit
tc class add dev eth0 parent 1:1 classid 1:30 htb rate 50mbit ceil 100mbit

# High priority for consensus traffic
tc filter add dev eth0 protocol ip parent 1:0 prio 1 u32 match ip dport 8080 0xffff flowid 1:10

# Medium priority for API traffic
tc filter add dev eth0 protocol ip parent 1:0 prio 2 u32 match ip dport 3000 0xffff flowid 1:20
```

## Troubleshooting

### Common Issues

#### Node Synchronization Problems

```bash
# Check sync status
curl "http://localhost:3000/api/v1/blockchain/info" | jq '.sync_status'

# Force resync
curl -X POST "http://localhost:8082/admin/sync/restart"

# Check peer connections
curl "http://localhost:3000/api/v1/network/peers" | jq '.peers[].status'
```

#### Performance Issues

```bash
# Check system resources
htop
iotop
nethogs

# Analyze database performance
curl "http://localhost:9090/metrics" | grep beacon_storage

# Check network latency
ping -c 10 peer-node-address
```

#### Consensus Issues

```bash
# Check validator status
curl "http://localhost:8082/admin/consensus/validators"

# View consensus logs
sudo journalctl -u beacon-node -f | grep CONSENSUS

# Check block production
curl "http://localhost:3000/api/v1/blockchain/latest?count=10"
```

This comprehensive network administration guide provides all the tools and procedures needed to effectively manage a BEACON blockchain network infrastructure.
