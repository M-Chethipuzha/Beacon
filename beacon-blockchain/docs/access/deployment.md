# BEACON Blockchain Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying BEACON blockchain nodes in various environments, from development setups to production networks.

> **📦 For complete Docker containerization, see the [Containerization Guide](../containerization.md)**

## Deployment Options

### 1. Single Node Development Setup

#### Quick Development Start

```bash
# Clone repository
git clone https://github.com/beacon-blockchain/beacon-blockchain.git
cd beacon-blockchain

# Build all components
cargo build --release

# Start development node
./target/release/beacon-node --dev --api-port 3000 --p2p-port 8080
```

#### Docker Development Setup

> **🚀 Quick Start**: For automated Docker deployment, use the [Containerization Guide](../containerization.md) with:
>
> ```bash
> ./scripts/build-deploy.sh latest development
> ```

```dockerfile
# Dockerfile.dev
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/beacon-node /usr/local/bin/

EXPOSE 3000 8080 9000
CMD ["beacon-node", "--dev"]
```

```bash
# Build and run development container
docker build -f Dockerfile.dev -t beacon-dev .
docker run -p 3000:3000 -p 8080:8080 beacon-dev
```

### 2. Multi-Node Local Network

#### Docker Compose Setup

```yaml
# docker-compose.yml
version: "3.8"

services:
  beacon-node-1:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: beacon-node-1
    ports:
      - "3001:3000"
      - "8081:8080"
      - "9001:9000"
    environment:
      - NODE_ID=beacon-node-1
      - P2P_PORT=8080
      - API_PORT=3000
      - BOOTSTRAP_PEERS=beacon-node-2:8080,beacon-node-3:8080
    volumes:
      - beacon-node-1-data:/var/lib/beacon
    networks:
      - beacon-network

  beacon-node-2:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: beacon-node-2
    ports:
      - "3002:3000"
      - "8082:8080"
      - "9002:9000"
    environment:
      - NODE_ID=beacon-node-2
      - P2P_PORT=8080
      - API_PORT=3000
      - BOOTSTRAP_PEERS=beacon-node-1:8080,beacon-node-3:8080
    volumes:
      - beacon-node-2-data:/var/lib/beacon
    networks:
      - beacon-network

  beacon-node-3:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: beacon-node-3
    ports:
      - "3003:3000"
      - "8083:8080"
      - "9003:9000"
    environment:
      - NODE_ID=beacon-node-3
      - P2P_PORT=8080
      - API_PORT=3000
      - BOOTSTRAP_PEERS=beacon-node-1:8080,beacon-node-2:8080
    volumes:
      - beacon-node-3-data:/var/lib/beacon
    networks:
      - beacon-network

volumes:
  beacon-node-1-data:
  beacon-node-2-data:
  beacon-node-3-data:

networks:
  beacon-network:
    driver: bridge
```

```bash
# Start local network
docker-compose up -d

# Monitor logs
docker-compose logs -f

# Stop network
docker-compose down
```

### 3. Cloud Deployment

#### AWS Deployment with Terraform

```hcl
# main.tf
provider "aws" {
  region = var.aws_region
}

# VPC Configuration
resource "aws_vpc" "beacon_vpc" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "beacon-blockchain-vpc"
  }
}

resource "aws_subnet" "beacon_subnet" {
  count                   = length(var.availability_zones)
  vpc_id                  = aws_vpc.beacon_vpc.id
  cidr_block              = "10.0.${count.index + 1}.0/24"
  availability_zone       = var.availability_zones[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name = "beacon-subnet-${count.index + 1}"
  }
}

# Security Groups
resource "aws_security_group" "beacon_node_sg" {
  name        = "beacon-node-security-group"
  description = "Security group for BEACON blockchain nodes"
  vpc_id      = aws_vpc.beacon_vpc.id

  # REST API
  ingress {
    from_port   = 3000
    to_port     = 3000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # P2P Network
  ingress {
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/16"]
  }

  # Discovery
  ingress {
    from_port   = 8081
    to_port     = 8081
    protocol    = "udp"
    cidr_blocks = ["10.0.0.0/16"]
  }

  # SSH
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "beacon-node-sg"
  }
}

# EC2 Instances
resource "aws_instance" "beacon_nodes" {
  count                  = var.node_count
  ami                    = var.ubuntu_ami
  instance_type          = var.instance_type
  subnet_id              = aws_subnet.beacon_subnet[count.index % length(aws_subnet.beacon_subnet)].id
  security_groups        = [aws_security_group.beacon_node_sg.id]
  key_name              = var.key_pair_name

  root_block_device {
    volume_type = "gp3"
    volume_size = 100
    encrypted   = true
  }

  user_data = templatefile("${path.module}/user_data.sh", {
    node_id = "beacon-node-${count.index + 1}"
    node_index = count.index
  })

  tags = {
    Name = "beacon-node-${count.index + 1}"
    Type = "blockchain-node"
  }
}

# Load Balancer
resource "aws_lb" "beacon_api_lb" {
  name               = "beacon-api-load-balancer"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.beacon_node_sg.id]
  subnets           = aws_subnet.beacon_subnet[*].id

  tags = {
    Name = "beacon-api-lb"
  }
}

resource "aws_lb_target_group" "beacon_api_tg" {
  name     = "beacon-api-targets"
  port     = 3000
  protocol = "HTTP"
  vpc_id   = aws_vpc.beacon_vpc.id

  health_check {
    enabled             = true
    healthy_threshold   = 2
    interval            = 30
    matcher             = "200"
    path                = "/health"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    unhealthy_threshold = 2
  }
}

resource "aws_lb_target_group_attachment" "beacon_api_tg_attachment" {
  count            = var.node_count
  target_group_arn = aws_lb_target_group.beacon_api_tg.arn
  target_id        = aws_instance.beacon_nodes[count.index].id
  port             = 3000
}

resource "aws_lb_listener" "beacon_api_listener" {
  load_balancer_arn = aws_lb.beacon_api_lb.arn
  port              = "80"
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.beacon_api_tg.arn
  }
}
```

```hcl
# variables.tf
variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-east-1"
}

variable "availability_zones" {
  description = "Availability zones for deployment"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

variable "node_count" {
  description = "Number of BEACON nodes to deploy"
  type        = number
  default     = 3
}

variable "instance_type" {
  description = "EC2 instance type for nodes"
  type        = string
  default     = "t3.large"
}

variable "ubuntu_ami" {
  description = "Ubuntu AMI ID"
  type        = string
  default     = "ami-0c7217cdde317cfec"  # Ubuntu 22.04 LTS
}

variable "key_pair_name" {
  description = "AWS key pair name for SSH access"
  type        = string
}
```

```bash
#!/bin/bash
# user_data.sh
apt-get update -y
apt-get install -y docker.io docker-compose curl jq

# Install BEACON node
cd /opt
wget https://releases.beacon-blockchain.com/v0.1.0/beacon-node-linux-amd64.tar.gz
tar -xzf beacon-node-linux-amd64.tar.gz
chmod +x beacon-node

# Create configuration
mkdir -p /etc/beacon
cat > /etc/beacon/config.yaml <<EOF
node:
  id: "${node_id}"
  data_dir: "/var/lib/beacon"
  log_level: "info"

network:
  p2p:
    port: 8080
    bind_address: "0.0.0.0"
    external_address: "$(curl -s http://169.254.169.254/latest/meta-data/public-ipv4):8080"

  api:
    port: 3000
    bind_address: "0.0.0.0"

  discovery:
    enabled: true
    port: 8081

database:
  type: "rocksdb"
  path: "/var/lib/beacon/data"
  cache_size: "2GB"

consensus:
  type: "PoA"
  validator: ${node_index < 3 ? "true" : "false"}
EOF

# Create systemd service
cat > /etc/systemd/system/beacon-node.service <<EOF
[Unit]
Description=BEACON Blockchain Node
After=network.target

[Service]
Type=simple
User=root
ExecStart=/opt/beacon-node --config /etc/beacon/config.yaml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl enable beacon-node
systemctl start beacon-node
```

#### Kubernetes Deployment

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: beacon-blockchain
```

```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: beacon-config
  namespace: beacon-blockchain
data:
  config.yaml: |
    node:
      data_dir: "/var/lib/beacon"
      log_level: "info"

    network:
      p2p:
        port: 8080
        bind_address: "0.0.0.0"
      
      api:
        port: 3000
        bind_address: "0.0.0.0"
        
      discovery:
        enabled: true
        port: 8081

    database:
      type: "rocksdb"
      path: "/var/lib/beacon/data"
      cache_size: "2GB"

    consensus:
      type: "PoA"
```

```yaml
# k8s/statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: beacon-node
  namespace: beacon-blockchain
spec:
  serviceName: beacon-node-service
  replicas: 3
  selector:
    matchLabels:
      app: beacon-node
  template:
    metadata:
      labels:
        app: beacon-node
    spec:
      containers:
        - name: beacon-node
          image: beacon/node:latest
          ports:
            - containerPort: 3000
              name: api
            - containerPort: 8080
              name: p2p
            - containerPort: 8081
              name: discovery
            - containerPort: 9000
              name: grpc
          env:
            - name: NODE_ID
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
          volumeMounts:
            - name: beacon-data
              mountPath: /var/lib/beacon
            - name: beacon-config
              mountPath: /etc/beacon
          resources:
            requests:
              memory: "4Gi"
              cpu: "2000m"
            limits:
              memory: "8Gi"
              cpu: "4000m"
      volumes:
        - name: beacon-config
          configMap:
            name: beacon-config
  volumeClaimTemplates:
    - metadata:
        name: beacon-data
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 100Gi
        storageClassName: fast-ssd
```

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: beacon-node-service
  namespace: beacon-blockchain
spec:
  clusterIP: None
  ports:
    - port: 3000
      name: api
    - port: 8080
      name: p2p
    - port: 8081
      name: discovery
  selector:
    app: beacon-node
---
apiVersion: v1
kind: Service
metadata:
  name: beacon-api-lb
  namespace: beacon-blockchain
spec:
  type: LoadBalancer
  ports:
    - port: 80
      targetPort: 3000
      name: api
  selector:
    app: beacon-node
```

```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: beacon-api-ingress
  namespace: beacon-blockchain
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
    - hosts:
        - api.beacon-network.com
      secretName: beacon-api-tls
  rules:
    - host: api.beacon-network.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: beacon-api-lb
                port:
                  number: 80
```

```bash
# Deploy to Kubernetes
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/statefulset.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml

# Monitor deployment
kubectl get pods -n beacon-blockchain -w
kubectl logs -f beacon-node-0 -n beacon-blockchain
```

## Production Deployment Considerations

### High Availability Setup

#### Multi-Region Deployment

```yaml
# Global load balancer configuration
regions:
  us-east-1:
    nodes: 3
    validators: 2
  eu-west-1:
    nodes: 3
    validators: 1
  ap-southeast-1:
    nodes: 2
    validators: 0

# Cross-region replication
replication:
  enabled: true
  sync_interval: 10s
  conflict_resolution: "latest_timestamp"
```

#### Database Clustering

```yaml
# RocksDB cluster configuration
database:
  type: "rocksdb_cluster"
  nodes:
    - "db-node-1:6379"
    - "db-node-2:6379"
    - "db-node-3:6379"
  replication_factor: 3
  consistency_level: "quorum"
```

### Security Hardening

#### TLS Configuration

```yaml
# Production TLS setup
network:
  api:
    tls:
      enabled: true
      cert_file: "/etc/beacon/tls/cert.pem"
      key_file: "/etc/beacon/tls/key.pem"
      client_ca_file: "/etc/beacon/tls/ca.pem"
      min_version: "1.3"
      cipher_suites:
        - "TLS_AES_256_GCM_SHA384"
        - "TLS_CHACHA20_POLY1305_SHA256"
```

#### Network Security

```bash
# Firewall rules for production
ufw default deny incoming
ufw default allow outgoing
ufw allow from 10.0.0.0/8 to any port 8080  # P2P internal only
ufw allow from anywhere to any port 443       # HTTPS API
ufw allow from management_network to any port 22  # SSH from management
ufw enable
```

### Monitoring and Observability

#### Prometheus Monitoring

```yaml
# prometheus-config.yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "beacon-alerts.yml"

scrape_configs:
  - job_name: "beacon-nodes"
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - beacon-blockchain
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: beacon-node
      - source_labels: [__meta_kubernetes_pod_ip]
        target_label: __address__
        replacement: ${1}:9090
```

#### Log Aggregation

```yaml
# fluentd-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
data:
  fluent.conf: |
    <source>
      @type tail
      path /var/log/beacon/*.log
      pos_file /var/log/fluentd/beacon.log.pos
      tag beacon.node
      format json
    </source>

    <match beacon.**>
      @type elasticsearch
      host elasticsearch.logging.svc.cluster.local
      port 9200
      index_name beacon-logs
      type_name beacon
    </match>
```

### Backup and Disaster Recovery

#### Automated Backup

```yaml
# backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: beacon-backup
  namespace: beacon-blockchain
spec:
  schedule: "0 2 * * *" # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: backup
              image: beacon/backup:latest
              env:
                - name: S3_BUCKET
                  value: "beacon-backups"
                - name: AWS_REGION
                  value: "us-east-1"
              volumeMounts:
                - name: beacon-data
                  mountPath: /var/lib/beacon
                  readOnly: true
              command:
                - /bin/bash
                - -c
                - |
                  DATE=$(date +%Y%m%d_%H%M%S)
                  tar -czf /tmp/beacon_backup_$DATE.tar.gz -C /var/lib/beacon .
                  aws s3 cp /tmp/beacon_backup_$DATE.tar.gz s3://$S3_BUCKET/
          restartPolicy: OnFailure
          volumes:
            - name: beacon-data
              persistentVolumeClaim:
                claimName: beacon-data-beacon-node-0
```

## Deployment Verification

### Health Checks

```bash
#!/bin/bash
# verify-deployment.sh

API_ENDPOINT="https://api.beacon-network.com"

echo "Verifying BEACON deployment..."

# Check API health
HEALTH=$(curl -s "$API_ENDPOINT/health" | jq -r '.status')
if [ "$HEALTH" = "healthy" ]; then
    echo "✅ API health check passed"
else
    echo "❌ API health check failed"
    exit 1
fi

# Check network connectivity
PEER_COUNT=$(curl -s "$API_ENDPOINT/api/v1/network/peers" | jq '.peers | length')
if [ "$PEER_COUNT" -ge 2 ]; then
    echo "✅ Network connectivity verified ($PEER_COUNT peers)"
else
    echo "❌ Insufficient peer connections"
    exit 1
fi

# Check consensus
LATEST_BLOCK=$(curl -s "$API_ENDPOINT/api/v1/blockchain/info" | jq '.latest_block_number')
sleep 10
CURRENT_BLOCK=$(curl -s "$API_ENDPOINT/api/v1/blockchain/info" | jq '.latest_block_number')

if [ "$CURRENT_BLOCK" -gt "$LATEST_BLOCK" ]; then
    echo "✅ Consensus is active (block progression verified)"
else
    echo "❌ Consensus appears stalled"
    exit 1
fi

echo "🎉 Deployment verification completed successfully!"
```

### Performance Testing

```bash
#!/bin/bash
# performance-test.sh

API_ENDPOINT="https://api.beacon-network.com"

echo "Running performance tests..."

# Load testing with multiple concurrent requests
echo "Testing API throughput..."
ab -n 1000 -c 10 "$API_ENDPOINT/health" > /tmp/load_test.txt

# Extract results
REQUESTS_PER_SECOND=$(grep "Requests per second" /tmp/load_test.txt | awk '{print $4}')
MEAN_TIME=$(grep "Time per request" /tmp/load_test.txt | head -1 | awk '{print $4}')

echo "Results:"
echo "  Requests per second: $REQUESTS_PER_SECOND"
echo "  Mean response time: ${MEAN_TIME}ms"

# Verify performance thresholds
if (( $(echo "$REQUESTS_PER_SECOND > 100" | bc -l) )); then
    echo "✅ Throughput test passed"
else
    echo "❌ Throughput below threshold"
fi
```

This comprehensive deployment guide covers all aspects of deploying BEACON blockchain networks from development to production environments.
