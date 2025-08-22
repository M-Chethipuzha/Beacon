# BEACON Edge Gateway SCS - README

## Overview

The BEACON Edge Gateway SCS (Smart Control System) provides secure, policy-enforced communication between IoT devices and the BEACON blockchain infrastructure. It acts as a local gateway that discovers I&O SCS nodes, caches policies, and manages IoT device communication through MQTT and CoAP protocols.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   IoT Devices   │    │  Edge Gateway   │    │  I&O SCS Nodes  │
│                 │    │                 │    │                 │
│ • MQTT Clients  │◄──►│ • Policy Cache  │◄──►│ • Blockchain    │
│ • CoAP Devices  │    │ • Access Control│    │ • Chaincode     │
│ • Sensors       │    │ • Device Mgmt   │    │ • Consensus     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Features

- **Automatic I&O SCS Discovery**: DNS-based service discovery with health monitoring
- **Policy Caching**: Local SQLite cache with privacy-preserving device ID hashing
- **MQTT Communication**: Full-featured MQTT broker integration with access control
- **CoAP Support**: Constrained Application Protocol for resource-limited devices
- **REST API**: Local management interface for configuration and monitoring
- **Docker Integration**: Complete containerized deployment with monitoring stack
- **Security**: TLS encryption, device authentication, and audit logging

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Python 3.11+ (for development)
- OpenSSL (for certificate generation)

### Deployment

#### Using PowerShell (Windows)

```powershell
# Deploy with certificate generation
.\deploy.ps1

# Build images first
.\deploy.ps1 -Build

# View logs
.\deploy.ps1 -Logs

# Stop services
.\deploy.ps1 -Stop
```

#### Using Bash (Linux/macOS)

```bash
# Make script executable
chmod +x deploy.sh

# Deploy
./deploy.sh

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

#### Manual Docker Compose

```bash
# Create directories and certificates first
mkdir -p data logs certs docker/mosquitto/{data,log,config}

# Start services
docker-compose up -d

# Check status
docker-compose ps
```

## Configuration

### Main Configuration File

Edit `config/gateway.toml` to customize the gateway behavior:

```toml
[gateway]
id = "gateway-edge-001"
name = "BEACON Edge Gateway"
location = "Building A, Floor 1"

[io_scs]
discovery_domains = ["_beacon-io._tcp.beacon.local"]
static_nodes = ["localhost:8080"]

[mqtt]
broker_host = "localhost"
broker_port = 1883
use_tls = false

[api]
host = "0.0.0.0"
port = 8081
```

### Environment Variables

- `BEACON_CONFIG_FILE`: Path to configuration file
- `BEACON_LOG_LEVEL`: Logging level (DEBUG, INFO, WARNING, ERROR)
- `BEACON_GATEWAY_ID`: Override gateway ID

## API Reference

### Health Check

```bash
curl http://localhost:8081/health
```

### Device Management

```bash
# List devices
curl http://localhost:8081/devices

# Get device info
curl http://localhost:8081/devices/{device_id}

# Authorize device
curl -X POST http://localhost:8081/devices/{device_id}/authorize \
  -H "Content-Type: application/json" \
  -d '{"device_type": "sensor", "capabilities": ["temperature"]}'
```

### Policy Management

```bash
# List policies
curl http://localhost:8081/policies

# Check policy
curl -X POST http://localhost:8081/policies/check \
  -H "Content-Type: application/json" \
  -d '{"device_id": "sensor001", "resource": "data", "action": "read"}'

# Sync policies
curl -X POST http://localhost:8081/policies/sync
```

## MQTT Integration

### Connection Parameters

- **Host**: `localhost`
- **Port**: `1883` (plain) / `8883` (TLS)
- **WebSocket**: `9001`
- **Username**: `beacon-gateway`
- **Password**: `beacon_mqtt_password`

### Topic Structure

```
beacon/{gateway_id}/device/{device_id}/data
beacon/{gateway_id}/device/{device_id}/status
beacon/{gateway_id}/device/{device_id}/command
beacon/{gateway_id}/management/{topic}
beacon/broadcast/{topic}
```

### Example MQTT Client

```python
import paho.mqtt.client as mqtt

client = mqtt.Client()
client.username_pw_set("beacon-gateway", "beacon_mqtt_password")
client.connect("localhost", 1883, 60)

# Publish device data
client.publish("beacon/gateway-001/device/sensor001/data",
               '{"temperature": 23.5, "humidity": 65}')
```

## CoAP Integration

### Endpoints

- `coap://localhost:5683/device/{device_id}` - Device status and data
- `coap://localhost:5683/policy` - Policy queries
- `coap://localhost:5683/gateway` - Gateway information

### Example CoAP Client

```python
import aiocoap
import asyncio

async def coap_client():
    context = await aiocoap.Context.create_client_context()

    # Get gateway status
    request = aiocoap.Message(code=aiocoap.GET, uri='coap://localhost/gateway')
    response = await context.request(request).response

    print(response.payload.decode())
```

## Development

### Local Development Setup

```bash
# Clone repository
git clone <repository-url>
cd edge-gateway-scs/gateway-app

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Run gateway
python -m src.main
```

### Testing

```bash
# Install test dependencies
pip install pytest pytest-asyncio pytest-cov

# Run tests
pytest tests/

# Run with coverage
pytest --cov=src tests/
```

### Code Quality

```bash
# Format code
black src/ tests/

# Lint code
flake8 src/ tests/

# Type checking
mypy src/
```

## Monitoring

### Prometheus Metrics

Available at `http://localhost:9091/metrics`:

- `beacon_gateway_info` - Gateway information
- `beacon_service_health` - Service health status
- `beacon_device_count` - Connected device count
- `beacon_policy_cache_size` - Cached policy count

### Grafana Dashboards

Access Grafana at `http://localhost:3000`:

- **Username**: `admin`
- **Password**: `beacon_admin`

### Logs

```bash
# View all logs
docker-compose logs -f

# View specific service logs
docker-compose logs -f beacon-edge-gateway
docker-compose logs -f mosquitto
docker-compose logs -f prometheus
```

## Troubleshooting

### Common Issues

#### Gateway Not Starting

1. Check Docker is running: `docker info`
2. Check port conflicts: `netstat -tlnp | grep 8081`
3. View logs: `docker-compose logs beacon-edge-gateway`

#### MQTT Connection Issues

1. Check broker is running: `nc -zv localhost 1883`
2. Verify credentials in `docker/mosquitto/config/passwd`
3. Check ACL permissions in `docker/mosquitto/config/acl`

#### Policy Sync Failures

1. Verify I&O SCS node connectivity
2. Check DNS resolution for discovery domains
3. Review blockchain client logs

#### Certificate Issues

1. Regenerate certificates: `rm -rf certs/ && ./deploy.sh`
2. Check certificate permissions
3. Verify certificate paths in configuration

### Debug Mode

Enable debug logging by setting:

```bash
export BEACON_LOG_LEVEL=DEBUG
```

Or in `config/gateway.toml`:

```toml
[logging]
level = "DEBUG"
```

## Security Considerations

### Device Authentication

- All devices must be registered before communication
- Device IDs are hashed for privacy
- Certificate-based authentication recommended

### Network Security

- Use TLS for MQTT connections in production
- Implement VPN for remote access
- Firewall rules for port access

### Data Privacy

- Device identifiers are hashed with gateway-specific salt
- Audit logs capture access patterns without sensitive data
- Policy cache encrypts sensitive policy details

## Integration with I&O SCS

The Edge Gateway integrates with the BEACON I&O SCS through:

1. **Service Discovery**: Automatic discovery of I&O SCS nodes
2. **Policy Synchronization**: Regular sync of access control policies
3. **Device Registration**: Register gateway and devices with blockchain
4. **Audit Logging**: Submit access logs to blockchain for compliance

## Support

### Documentation

- [API Reference](docs/api-reference.md)
- [Configuration Guide](docs/configuration.md)
- [Integration Guide](docs/integration.md)

### Community

- Issues: GitHub Issues
- Discussions: GitHub Discussions
- Wiki: Project Wiki

## License

This project is part of the BEACON blockchain platform and follows the project's licensing terms.
