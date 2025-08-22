# BEACON Edge Gateway SCS - Quick Start Guide

# ===========================================

Get your BEACON Edge Gateway up and running in minutes.

## Prerequisites

- Docker and Docker Compose installed
- At least 2GB RAM and 10GB disk space
- Network access for I&O SCS discovery
- OpenSSL (for certificate generation)

## Quick Deployment

### 1. Clone and Setup

```bash
git clone <repository-url>
cd edge-gateway-scs
```

### 2. Deploy with One Command

#### Windows (PowerShell)

```powershell
.\gateway-app\deploy.ps1
```

#### Linux/macOS

```bash
chmod +x gateway-app/deploy.sh
./gateway-app/deploy.sh
```

### 3. Verify Deployment

Check all services are running:

```bash
docker-compose ps
```

Verify gateway health:

```bash
curl http://localhost:8081/health
```

## Service URLs

After successful deployment:

- **Gateway API**: http://localhost:8081
- **Gateway Health**: http://localhost:8081/health
- **Prometheus**: http://localhost:9091
- **Grafana**: http://localhost:3000 (admin/beacon_admin)

## MQTT Connection

Connect devices to the MQTT broker:

- **Host**: localhost
- **Port**: 1883 (plain) / 8883 (TLS)
- **WebSocket**: 9001
- **Username**: beacon-gateway
- **Password**: beacon_mqtt_password

## CoAP Access

Test CoAP connectivity:

```bash
# Using aiocoap (Python)
pip install aiocoap
aiocoap-client coap://localhost:5683/.well-known/core

# Using libcoap (C)
coap-client -m get coap://localhost:5683/.well-known/core
```

## Basic Device Registration

### Via REST API

```bash
curl -X POST http://localhost:8081/devices/sensor001/authorize \
  -H "Content-Type: application/json" \
  -d '{"device_type": "temperature_sensor", "capabilities": ["temperature", "humidity"]}'
```

### Via MQTT

```bash
mosquitto_pub -h localhost -p 1883 \
  -t "beacon/gateway-edge-001/device/sensor001/data" \
  -m '{"temperature": 23.5, "humidity": 65.2}'
```

## Configuration

Basic configuration is in `gateway-app/config/gateway.toml`:

```toml
[gateway]
id = "gateway-edge-001"
name = "BEACON Edge Gateway"

[mqtt]
broker_host = "localhost"
broker_port = 1883

[api]
host = "0.0.0.0"
port = 8081
```

## Stopping Services

```bash
# Stop all services
docker-compose down

# Or using the script
.\gateway-app\deploy.ps1 -Stop
```

## Next Steps

1. **Configure I&O SCS Discovery**: Update `io_scs.discovery_domains` in config
2. **Setup VPN**: Configure VPN client for secure communication
3. **Add Devices**: Register your IoT devices
4. **Setup Monitoring**: Configure Grafana dashboards
5. **Production Setup**: See [Production Deployment](deployment.md)

## Troubleshooting

### Common Issues

#### Gateway not starting

```bash
# Check logs
docker-compose logs beacon-edge-gateway

# Check configuration
cat gateway-app/config/gateway.toml
```

#### MQTT connection refused

```bash
# Check broker status
docker-compose logs beacon-mosquitto

# Test connection
mosquitto_pub -h localhost -p 1883 -t test -m "hello"
```

#### Port conflicts

```bash
# Check port usage
netstat -tulpn | grep -E "(8081|1883|5683)"

# Use different ports in docker-compose.yml
```

## Support

- Documentation: [docs/README.md](README.md)
- Troubleshooting: [troubleshooting.md](troubleshooting.md)
- Issues: GitHub Issues
- Examples: [examples/](examples/)
