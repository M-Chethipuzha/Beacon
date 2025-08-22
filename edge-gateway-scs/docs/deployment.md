# BEACON Edge Gateway SCS - Production Deployment Guide

# =====================================================

Complete guide for deploying BEACON Edge Gateway in production environments.

## Pre-Production Checklist

### Hardware Requirements

- **Minimum**: 4 CPU cores, 8GB RAM, 50GB SSD
- **Recommended**: 8 CPU cores, 16GB RAM, 100GB SSD
- **Network**: Gigabit Ethernet, WiFi 6 capability
- **Security**: TPM 2.0 chip (recommended)

### Software Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- Linux OS (Ubuntu 20.04+ LTS recommended)
- OpenSSL 1.1.1+
- systemd (for service management)

### Network Requirements

- Static IP address or DDNS
- Firewall configuration capability
- VPN connectivity to I&O SCS
- Internet access for updates

## Security Hardening

### Certificate Management

#### Generate Production Certificates

```bash
# Create certificate authority
cd gateway-app/certs
./generate-production-certs.sh
```

#### Certificate Authority Setup

```bash
#!/bin/bash
# gateway-app/certs/generate-production-certs.sh

# Root CA
openssl genrsa -out ca.key 4096
openssl req -new -x509 -days 3650 -key ca.key -out ca.crt \
  -subj "/C=US/ST=CA/L=San Francisco/O=BEACON/OU=Edge Gateway/CN=BEACON Edge CA"

# Gateway certificate
openssl genrsa -out gateway.key 4096
openssl req -new -key gateway.key -out gateway.csr \
  -subj "/C=US/ST=CA/L=San Francisco/O=BEACON/OU=Edge Gateway/CN=$(hostname)"

# Sign gateway certificate
openssl x509 -req -in gateway.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out gateway.crt -days 365 -extensions v3_req \
  -extfile <(echo "subjectAltName=DNS:$(hostname),IP:$(hostname -I | awk '{print $1}')")

# MQTT broker certificates
openssl genrsa -out mqtt-broker.key 4096
openssl req -new -key mqtt-broker.key -out mqtt-broker.csr \
  -subj "/C=US/ST=CA/L=San Francisco/O=BEACON/OU=MQTT Broker/CN=beacon-mqtt"
openssl x509 -req -in mqtt-broker.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out mqtt-broker.crt -days 365

# Clean up CSRs
rm *.csr

# Set permissions
chmod 600 *.key
chmod 644 *.crt
```

### Firewall Configuration

#### UFW (Ubuntu)

```bash
# Reset firewall
sudo ufw --force reset

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# SSH access (change port if needed)
sudo ufw allow ssh

# Gateway API (restrict to management networks)
sudo ufw allow from 192.168.1.0/24 to any port 8081

# MQTT (restrict to device networks)
sudo ufw allow from 10.0.0.0/8 to any port 1883
sudo ufw allow from 10.0.0.0/8 to any port 8883

# CoAP (restrict to device networks)
sudo ufw allow from 10.0.0.0/8 to any port 5683/udp

# VPN (if hosting VPN server)
sudo ufw allow 1194/udp
sudo ufw allow 51820/udp

# Monitoring (restrict to monitoring network)
sudo ufw allow from 192.168.100.0/24 to any port 9091

# Enable firewall
sudo ufw enable
```

#### iptables (Advanced)

```bash
# Backup current rules
iptables-save > /etc/iptables/rules.backup

# Apply production rules
iptables-restore < gateway-app/config/iptables.rules
```

### User Management

#### Service User

```bash
# Create dedicated user
sudo useradd -r -s /bin/false beacon-gateway
sudo usermod -aG docker beacon-gateway

# Set ownership
sudo chown -R beacon-gateway:beacon-gateway /opt/beacon-gateway/
sudo chmod 750 /opt/beacon-gateway/
```

### Secret Management

#### Environment Variables

```bash
# Production environment file
cat > gateway-app/.env.production << EOF
# Database encryption
DATABASE_ENCRYPTION_KEY=\$(openssl rand -hex 32)

# MQTT credentials
MQTT_USERNAME=beacon-gateway-prod
MQTT_PASSWORD=\$(openssl rand -base64 32)

# Blockchain client credentials
BLOCKCHAIN_CLIENT_CERT_PATH=/opt/beacon-gateway/certs/gateway.crt
BLOCKCHAIN_CLIENT_KEY_PATH=/opt/beacon-gateway/certs/gateway.key

# Monitoring credentials
PROMETHEUS_USERNAME=beacon-metrics
PROMETHEUS_PASSWORD=\$(openssl rand -base64 24)

# I&O SCS connection
IO_SCS_ENDPOINT=https://io-scs.beacon.local:8443
IO_SCS_AUTH_TOKEN=\$(echo "your-secure-token-here")

# VPN configuration
VPN_TYPE=wireguard
VPN_CONFIG_PATH=/opt/beacon-gateway/vpn/production.conf
EOF

# Secure environment file
chmod 600 gateway-app/.env.production
```

## Production Configuration

### Gateway Configuration

```toml
# gateway-app/config/gateway.production.toml
[gateway]
id = "gateway-prod-001"
name = "BEACON Production Edge Gateway"
environment = "production"
log_level = "INFO"
data_dir = "/opt/beacon-gateway/data"

[api]
host = "0.0.0.0"
port = 8081
tls_enabled = true
cert_file = "/opt/beacon-gateway/certs/gateway.crt"
key_file = "/opt/beacon-gateway/certs/gateway.key"
cors_enabled = false
request_timeout = 30
max_request_size = "10MB"

[database]
path = "/opt/beacon-gateway/data/gateway.db"
encryption_enabled = true
backup_enabled = true
backup_interval = "24h"
retention_days = 30

[mqtt]
broker_host = "beacon-mosquitto"
broker_port = 8883
username = "${MQTT_USERNAME}"
password = "${MQTT_PASSWORD}"
tls_enabled = true
ca_file = "/opt/beacon-gateway/certs/ca.crt"
cert_file = "/opt/beacon-gateway/certs/gateway.crt"
key_file = "/opt/beacon-gateway/certs/gateway.key"
keepalive = 60
qos = 1

[coap]
host = "0.0.0.0"
port = 5684  # Use secure CoAP port
tls_enabled = true
cert_file = "/opt/beacon-gateway/certs/gateway.crt"
key_file = "/opt/beacon-gateway/certs/gateway.key"

[blockchain]
client_type = "grpc"
endpoint = "${IO_SCS_ENDPOINT}"
auth_token = "${IO_SCS_AUTH_TOKEN}"
tls_enabled = true
cert_file = "/opt/beacon-gateway/certs/gateway.crt"
key_file = "/opt/beacon-gateway/certs/gateway.key"
timeout = 30
retry_attempts = 3

[vpn]
enabled = true
client_type = "${VPN_TYPE}"
config_path = "${VPN_CONFIG_PATH}"
auto_connect = true
health_check_interval = 300

[monitoring]
prometheus_enabled = true
prometheus_port = 9100
metrics_interval = 30
health_check_enabled = true
log_metrics = true

[logging]
level = "INFO"
file_enabled = true
file_path = "/opt/beacon-gateway/logs/gateway.log"
file_max_size = "100MB"
file_max_backups = 10
file_max_age = 30
json_format = true
include_caller = false

[security]
rate_limiting_enabled = true
rate_limit_requests = 1000
rate_limit_window = "1h"
device_auth_required = true
admin_auth_required = true
audit_logging = true
```

### Docker Compose Production

```yaml
# docker-compose.production.yml
version: "3.8"

services:
  beacon-edge-gateway:
    image: beacon/edge-gateway:${GATEWAY_VERSION:-latest}
    container_name: beacon-edge-gateway-prod
    restart: unless-stopped
    environment:
      - GATEWAY_CONFIG=/app/config/gateway.production.toml
      - GATEWAY_ENV=production
    env_file:
      - .env.production
    volumes:
      - /opt/beacon-gateway/config:/app/config:ro
      - /opt/beacon-gateway/data:/app/data
      - /opt/beacon-gateway/logs:/app/logs
      - /opt/beacon-gateway/certs:/app/certs:ro
    ports:
      - "8081:8081"
    networks:
      - beacon-internal
      - beacon-external
    depends_on:
      - beacon-mosquitto
      - beacon-prometheus
    healthcheck:
      test: ["CMD", "curl", "-f", "https://localhost:8081/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "5"

  beacon-mosquitto:
    image: eclipse-mosquitto:2.0
    container_name: beacon-mosquitto-prod
    restart: unless-stopped
    volumes:
      - /opt/beacon-gateway/iot-broker/mosquitto:/mosquitto/config:ro
      - /opt/beacon-gateway/data/mosquitto:/mosquitto/data
      - /opt/beacon-gateway/logs/mosquitto:/mosquitto/log
      - /opt/beacon-gateway/certs:/mosquitto/certs:ro
    ports:
      - "8883:8883" # TLS only in production
      - "9001:9001" # WebSocket TLS
    networks:
      - beacon-internal
      - beacon-devices
    healthcheck:
      test:
        [
          "CMD",
          "mosquitto_pub",
          "-h",
          "localhost",
          "-p",
          "8883",
          "--cafile",
          "/mosquitto/certs/ca.crt",
          "-t",
          "health",
          "-m",
          "ok",
        ]
      interval: 30s
      timeout: 10s
      retries: 3

  beacon-coap-server:
    build:
      context: ./iot-broker/coap-server
      dockerfile: Dockerfile.production
    container_name: beacon-coap-server-prod
    restart: unless-stopped
    environment:
      - COAP_TLS_ENABLED=true
    volumes:
      - /opt/beacon-gateway/certs:/app/certs:ro
      - /opt/beacon-gateway/logs/coap:/app/logs
    ports:
      - "5684:5684/udp" # Secure CoAP
    networks:
      - beacon-internal
      - beacon-devices

  beacon-prometheus:
    image: prom/prometheus:latest
    container_name: beacon-prometheus-prod
    restart: unless-stopped
    command:
      - "--config.file=/etc/prometheus/prometheus.yml"
      - "--storage.tsdb.path=/prometheus"
      - "--web.console.libraries=/etc/prometheus/console_libraries"
      - "--web.console.templates=/etc/prometheus/consoles"
      - "--storage.tsdb.retention.time=30d"
      - "--web.enable-lifecycle"
      - "--web.enable-admin-api"
    volumes:
      - /opt/beacon-gateway/monitoring/prometheus:/etc/prometheus:ro
      - /opt/beacon-gateway/data/prometheus:/prometheus
    ports:
      - "127.0.0.1:9091:9090" # Bind to localhost only
    networks:
      - beacon-internal

  beacon-grafana:
    image: grafana/grafana:latest
    container_name: beacon-grafana-prod
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD}
      - GF_USERS_ALLOW_SIGN_UP=false
      - GF_SERVER_DOMAIN=${GRAFANA_DOMAIN:-localhost}
      - GF_SERVER_ROOT_URL=https://${GRAFANA_DOMAIN:-localhost}:3000
      - GF_SECURITY_COOKIE_SECURE=true
    volumes:
      - /opt/beacon-gateway/monitoring/grafana:/etc/grafana/provisioning:ro
      - /opt/beacon-gateway/data/grafana:/var/lib/grafana
    ports:
      - "127.0.0.1:3000:3000" # Bind to localhost only
    networks:
      - beacon-internal

networks:
  beacon-internal:
    driver: bridge
    internal: true
  beacon-external:
    driver: bridge
  beacon-devices:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16

volumes:
  beacon-gateway-data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /opt/beacon-gateway/data
  beacon-gateway-logs:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /opt/beacon-gateway/logs
```

## Service Management

### Systemd Service

```ini
# /etc/systemd/system/beacon-gateway.service
[Unit]
Description=BEACON Edge Gateway
Requires=docker.service
After=docker.service
StartLimitIntervalSec=0

[Service]
Type=oneshot
RemainAfterExit=yes
User=beacon-gateway
Group=beacon-gateway
WorkingDirectory=/opt/beacon-gateway
ExecStartPre=/usr/bin/docker-compose -f docker-compose.production.yml pull
ExecStart=/usr/bin/docker-compose -f docker-compose.production.yml up -d
ExecStop=/usr/bin/docker-compose -f docker-compose.production.yml down
ExecReload=/usr/bin/docker-compose -f docker-compose.production.yml restart
Restart=on-failure
RestartSec=30

[Install]
WantedBy=multi-user.target
```

### Service Commands

```bash
# Enable and start service
sudo systemctl enable beacon-gateway
sudo systemctl start beacon-gateway

# Check status
sudo systemctl status beacon-gateway

# View logs
sudo journalctl -u beacon-gateway -f

# Restart service
sudo systemctl restart beacon-gateway
```

## Monitoring and Alerting

### Prometheus Production Config

```yaml
# monitoring/prometheus/prometheus.production.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    environment: "production"
    gateway_id: "gateway-prod-001"

rule_files:
  - "rules/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - alertmanager:9093

scrape_configs:
  - job_name: "beacon-gateway"
    static_configs:
      - targets: ["beacon-edge-gateway:9100"]
    scrape_interval: 30s
    metrics_path: /metrics
    scheme: https
    tls_config:
      ca_file: /etc/prometheus/certs/ca.crt
      cert_file: /etc/prometheus/certs/prometheus.crt
      key_file: /etc/prometheus/certs/prometheus.key

  - job_name: "node-exporter"
    static_configs:
      - targets: ["localhost:9100"]

  - job_name: "docker"
    static_configs:
      - targets: ["localhost:9323"]
```

### Alert Rules

```yaml
# monitoring/prometheus/rules/beacon-gateway.yml
groups:
  - name: beacon-gateway
    rules:
      - alert: GatewayDown
        expr: up{job="beacon-gateway"} == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "BEACON Gateway is down"

      - alert: HighMemoryUsage
        expr: (container_memory_usage_bytes{name="beacon-edge-gateway"} / container_spec_memory_limit_bytes{name="beacon-edge-gateway"}) > 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage on BEACON Gateway"

      - alert: MQTTBrokerDown
        expr: up{job="mosquitto"} == 0
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "MQTT Broker is down"

      - alert: BlockchainConnectionFailed
        expr: beacon_blockchain_connection_status == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Blockchain connection failed"
```

## Backup and Recovery

### Automated Backup Script

```bash
#!/bin/bash
# /opt/beacon-gateway/scripts/backup.sh

BACKUP_DIR="/opt/beacon-gateway/backups"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="beacon-gateway-${DATE}"

# Create backup directory
mkdir -p "${BACKUP_DIR}/${BACKUP_NAME}"

# Stop services
systemctl stop beacon-gateway

# Backup data
tar -czf "${BACKUP_DIR}/${BACKUP_NAME}/data.tar.gz" -C /opt/beacon-gateway data/
tar -czf "${BACKUP_DIR}/${BACKUP_NAME}/config.tar.gz" -C /opt/beacon-gateway config/
tar -czf "${BACKUP_DIR}/${BACKUP_NAME}/certs.tar.gz" -C /opt/beacon-gateway certs/

# Backup logs
tar -czf "${BACKUP_DIR}/${BACKUP_NAME}/logs.tar.gz" -C /opt/beacon-gateway logs/

# Create backup manifest
cat > "${BACKUP_DIR}/${BACKUP_NAME}/manifest.txt" << EOF
Backup created: $(date)
Gateway version: $(docker image inspect beacon/edge-gateway:latest --format '{{.Config.Labels.version}}')
System: $(uname -a)
Docker version: $(docker version --format '{{.Server.Version}}')
EOF

# Start services
systemctl start beacon-gateway

# Cleanup old backups (keep 30 days)
find "${BACKUP_DIR}" -type d -name "beacon-gateway-*" -mtime +30 -exec rm -rf {} \;

echo "Backup completed: ${BACKUP_DIR}/${BACKUP_NAME}"
```

### Crontab for Automated Backups

```bash
# Add to /etc/crontab
0 2 * * * beacon-gateway /opt/beacon-gateway/scripts/backup.sh
```

## Updates and Maintenance

### Update Procedure

```bash
#!/bin/bash
# /opt/beacon-gateway/scripts/update.sh

# Pre-update backup
/opt/beacon-gateway/scripts/backup.sh

# Pull latest images
cd /opt/beacon-gateway
docker-compose -f docker-compose.production.yml pull

# Rolling update
docker-compose -f docker-compose.production.yml up -d --no-deps beacon-edge-gateway

# Verify update
sleep 30
curl -f https://localhost:8081/health || {
    echo "Health check failed, rolling back..."
    docker-compose -f docker-compose.production.yml restart beacon-edge-gateway
    exit 1
}

echo "Update completed successfully"
```

### Maintenance Window

```bash
# Schedule maintenance
echo "Starting maintenance window..."

# Stop non-critical services
docker-compose -f docker-compose.production.yml stop beacon-grafana

# Update configurations
# Apply security patches
# Rotate logs
# Clean temporary files

# Restart all services
docker-compose -f docker-compose.production.yml restart

echo "Maintenance completed"
```

## Compliance and Auditing

### Security Hardening Checklist

- [ ] All default passwords changed
- [ ] TLS enabled for all communications
- [ ] Firewall configured with minimal access
- [ ] Regular security updates applied
- [ ] Audit logging enabled
- [ ] Access controls implemented
- [ ] Certificate rotation scheduled
- [ ] Backup verification performed
- [ ] Monitoring and alerting configured
- [ ] Incident response plan documented

### Audit Configuration

```toml
# Add to gateway.production.toml
[audit]
enabled = true
log_file = "/opt/beacon-gateway/logs/audit.log"
log_format = "json"
events = [
    "authentication",
    "authorization",
    "configuration_changes",
    "device_registration",
    "blockchain_transactions",
    "vpn_connections",
    "api_access"
]
retention_days = 365
```

This production deployment guide provides comprehensive configuration for secure, scalable, and maintainable BEACON Edge Gateway deployments in production environments.
