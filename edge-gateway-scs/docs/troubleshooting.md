# BEACON Edge Gateway SCS - Troubleshooting Guide

# ================================================

Common issues and solutions for BEACON Edge Gateway deployment and operation.

## Deployment Issues

### Docker and Docker Compose

#### "docker-compose command not found"

```bash
# Install Docker Compose
curl -L "https://github.com/docker/compose/releases/download/1.29.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# Or use Docker Compose V2
docker compose version
```

#### "Permission denied" on Linux

```bash
# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Fix script permissions
chmod +x gateway-app/deploy.sh
chmod +x gateway-app/scripts/*.sh
```

#### "Port already in use"

```bash
# Find process using port
netstat -tulpn | grep :8081
lsof -i :8081

# Kill process or change port in docker-compose.yml
```

### Gateway Application

#### Gateway fails to start

```bash
# Check logs
docker-compose logs beacon-edge-gateway

# Common causes:
# 1. Configuration error
# 2. Database permission issues
# 3. Network connectivity
```

#### Configuration validation errors

```bash
# Validate TOML syntax
python -c "import toml; toml.load('gateway-app/config/gateway.toml')"

# Check required fields
grep -E "(gateway.id|gateway.name)" gateway-app/config/gateway.toml
```

#### Database errors

```bash
# Check database directory permissions
ls -la gateway-app/data/
chmod 755 gateway-app/data/

# Reset database
rm -rf gateway-app/data/gateway.db
docker-compose restart beacon-edge-gateway
```

## Network Connectivity

### I&O SCS Discovery

#### "Failed to discover I&O SCS"

```bash
# Check DNS resolution
nslookup beacon.discovery.local
dig beacon.discovery.local

# Test connectivity
curl -v http://io-scs.beacon.local:8080/health
ping io-scs.beacon.local
```

#### "Certificate verification failed"

```bash
# Check certificates
openssl x509 -in gateway-app/certs/gateway.crt -text -noout

# Regenerate certificates
cd gateway-app/certs
openssl req -x509 -newkey rsa:4096 -keyout gateway.key -out gateway.crt -days 365 -nodes
```

### VPN Connectivity

#### OpenVPN connection issues

```bash
# Check OpenVPN logs
docker-compose logs beacon-openvpn

# Test configuration
openvpn --config vpn-client/openvpn/client.ovpn --verb 6

# Common issues:
# 1. Incorrect server address
# 2. Certificate mismatch
# 3. Firewall blocking UDP 1194
```

#### WireGuard connection issues

```bash
# Check WireGuard status
docker exec beacon-wireguard wg show

# Test configuration
wg-quick up wg0

# Common issues:
# 1. Key mismatch
# 2. Endpoint unreachable
# 3. Port conflicts
```

## MQTT Broker Issues

### Mosquitto not starting

```bash
# Check Mosquitto logs
docker-compose logs beacon-mosquitto

# Validate configuration
mosquitto_passwd -c iot-broker/mosquitto/passwd beacon-gateway

# Test ACL permissions
mosquitto_pub -h localhost -p 1883 -u beacon-gateway -P beacon_mqtt_password -t test -m "hello"
```

### Client connection refused

```bash
# Check listener configuration
grep -A5 "listener" iot-broker/mosquitto/mosquitto.conf

# Test without authentication
mosquitto_pub -h localhost -p 1883 -t test -m "hello"

# Check firewall
sudo ufw allow 1883
sudo iptables -A INPUT -p tcp --dport 1883 -j ACCEPT
```

### TLS/SSL issues

```bash
# Check certificate validity
openssl x509 -in iot-broker/mosquitto/certs/ca.crt -text -noout

# Test TLS connection
mosquitto_pub -h localhost -p 8883 --cafile iot-broker/mosquitto/certs/ca.crt -t test -m "hello"

# Common issues:
# 1. Certificate expired
# 2. Hostname mismatch
# 3. CA certificate not trusted
```

## CoAP Server Issues

### CoAP server not responding

```bash
# Check CoAP server logs
docker-compose logs beacon-coap-server

# Test basic connectivity
aiocoap-client coap://localhost:5683/.well-known/core

# Check firewall for UDP
sudo ufw allow 5683/udp
```

### Resource access denied

```bash
# Check access control logs
docker-compose logs beacon-edge-gateway | grep -i coap

# Test without authentication
coap-client -m get coap://localhost:5683/test

# Common issues:
# 1. ACL configuration
# 2. Device not registered
# 3. Insufficient permissions
```

## Blockchain Integration

### Chaincode connection failures

```bash
# Check blockchain client logs
docker-compose logs beacon-edge-gateway | grep -i blockchain

# Test connectivity to I&O SCS
curl http://io-scs.beacon.local:8080/api/chaincode/ping

# Common issues:
# 1. Network unreachable
# 2. Authentication failure
# 3. Chaincode not deployed
```

### Transaction errors

```bash
# Check transaction logs
grep -i "transaction" gateway-app/logs/gateway.log

# Validate chaincode parameters
# Check for:
# 1. Invalid function names
# 2. Parameter type mismatches
# 3. Insufficient permissions
```

## Performance Issues

### High memory usage

```bash
# Check container memory usage
docker stats

# Monitor specific services
docker stats beacon-edge-gateway beacon-mosquitto

# Common causes:
# 1. Memory leaks in application
# 2. Large message queues
# 3. Excessive logging
```

### High CPU usage

```bash
# Check process CPU usage
top -p $(docker inspect -f '{{.State.Pid}}' beacon-edge-gateway)

# Profile application
# Add profiling to gateway configuration
# Monitor chaincode call frequency
```

### Disk space issues

```bash
# Check disk usage
df -h
du -sh gateway-app/data/ gateway-app/logs/

# Clean up logs
find gateway-app/logs/ -name "*.log" -mtime +7 -delete

# Rotate database
# Configure database cleanup in gateway.toml
```

## Monitoring and Diagnostics

### Prometheus not collecting metrics

```bash
# Check Prometheus configuration
curl http://localhost:9091/api/v1/label/__name__/values

# Validate targets
curl http://localhost:9091/api/v1/targets

# Check Gateway metrics endpoint
curl http://localhost:8081/metrics
```

### Grafana dashboard not showing data

```bash
# Check Grafana datasource
curl -u admin:beacon_admin http://localhost:3000/api/datasources

# Test Prometheus query
curl 'http://localhost:9091/api/v1/query?query=beacon_gateway_devices_total'

# Import dashboard manually
# Use dashboard ID or JSON from monitoring/grafana/dashboards/
```

## Log Analysis

### Enabling debug logging

```toml
# In gateway-app/config/gateway.toml
[logging]
level = "DEBUG"
file_level = "DEBUG"
```

### Key log patterns

```bash
# Error patterns
grep -i "error\|exception\|failed" gateway-app/logs/gateway.log

# Performance patterns
grep -i "slow\|timeout\|latency" gateway-app/logs/gateway.log

# Security patterns
grep -i "unauthorized\|denied\|authentication" gateway-app/logs/gateway.log
```

## Recovery Procedures

### Complete reset

```bash
# Stop all services
docker-compose down -v

# Remove data
rm -rf gateway-app/data/ gateway-app/logs/

# Regenerate certificates
cd gateway-app/certs
./generate-certs.sh

# Restart
docker-compose up -d
```

### Selective service restart

```bash
# Restart gateway only
docker-compose restart beacon-edge-gateway

# Restart all blockchain-related services
docker-compose restart beacon-edge-gateway beacon-prometheus

# Restart communication services
docker-compose restart beacon-mosquitto beacon-coap-server
```

### Configuration rollback

```bash
# Backup current config
cp gateway-app/config/gateway.toml gateway-app/config/gateway.toml.backup

# Restore default config
cp gateway-app/config/gateway.toml.default gateway-app/config/gateway.toml

# Restart with default config
docker-compose restart beacon-edge-gateway
```

## Getting Help

### Collecting diagnostic information

```bash
# System information
uname -a
docker version
docker-compose version

# Service status
docker-compose ps
docker-compose logs --tail=100 > debug.log

# Configuration files
tar -czf beacon-config.tar.gz gateway-app/config/ iot-broker/mosquitto/mosquitto.conf

# Send diagnostic bundle for support
```

### Support channels

- **Documentation**: [docs/README.md](README.md)
- **GitHub Issues**: Include diagnostic information
- **Community Forum**: BEACON Project discussions
- **Enterprise Support**: Contact system administrator

### Debug mode deployment

```bash
# Deploy with debug enabled
DEBUG=1 docker-compose -f docker-compose.yml -f docker-compose.debug.yml up -d

# Enable service debugging
docker-compose exec beacon-edge-gateway bash
```
