# BEACON Development Setup Guide

## Current Status Summary

Your team has successfully completed **Phase 1** of the BEACON project - the Identity & Onboarding SCS (I&O SCS) blockchain. This provides the foundational distributed ledger for gateway identity management and policy storage.

## Next Development Steps

### 1. Complete Edge Gateway SCS Structure

First, let's create the remaining modules for the Edge Gateway:

```bash
# Navigate to the project root
cd d:\Project\Project-BEACON

# Create the complete edge gateway structure
mkdir -p edge-gateway-scs\gateway-app\src\discovery
mkdir -p edge-gateway-scs\gateway-app\src\policy
mkdir -p edge-gateway-scs\gateway-app\src\iot
mkdir -p edge-gateway-scs\gateway-app\src\vpn
mkdir -p edge-gateway-scs\gateway-app\src\api
mkdir -p edge-gateway-scs\gateway-app\src\blockchain
mkdir -p edge-gateway-scs\gateway-app\config
mkdir -p edge-gateway-scs\iot-broker\mosquitto
mkdir -p edge-gateway-scs\iot-broker\coap-server
mkdir -p edge-gateway-scs\vpn-client\openvpn
mkdir -p edge-gateway-scs\vpn-client\wireguard
```

### 2. Development Environment Setup

#### Prerequisites

- Python 3.9+ (for Edge Gateway)
- Docker & Docker Compose (for containers)
- Git (for version control)
- Your existing Rust environment (for blockchain)

#### Python Environment Setup

```bash
cd edge-gateway-scs\gateway-app

# Create virtual environment
python -m venv venv

# Activate virtual environment (Windows)
venv\Scripts\activate

# Install required packages
pip install -r requirements.txt
```

### 3. Configuration Management

Create the main configuration file:

**File**: `edge-gateway-scs\gateway-app\config\gateway.example.toml`

```toml
[gateway]
id = "beacon_gateway_001"
data_dir = "./gateway_data"
log_level = "info"
salt = "your_unique_gateway_salt_here_change_this"

[discovery]
enabled = true
methods = ["mdns", "registry"]
registry_url = "http://localhost:8080/api/v1/discovery"
preferred_io_scs = []

[iot_hub]
mqtt_port = 1883
mqtt_tls_port = 8883
coap_port = 5683
max_devices = 100

[vpn]
provider = "openvpn"  # or "wireguard"
config_dir = "./vpn_configs"
auto_connect = true

[api]
bind_addr = "127.0.0.1:8081"
cors_origins = ["http://localhost:3000"]
auth_required = false  # Set to true in production

[privacy]
device_id_hashing = true
log_anonymization = true
local_data_encryption = true

[blockchain]
io_scs_endpoint = "http://localhost:8080/api/v1"
connection_timeout = 30
retry_attempts = 3
policy_sync_interval = 30
```

### 4. Required Dependencies

**File**: `edge-gateway-scs\gateway-app\requirements.txt`

```txt
# Async framework
asyncio
uvloop==0.19.0

# Configuration
toml==0.10.2

# HTTP client and server
aiohttp==3.9.1
fastapi==0.104.1
uvicorn==0.24.0

# IoT protocols
paho-mqtt==1.6.1
aiocoap==0.4.7

# Cryptography
cryptography==41.0.8
ed25519==1.5

# VPN management
python-openvpn==1.0.1

# Monitoring
prometheus-client==0.19.0

# Data processing
pydantic==2.5.0
pydantic-settings==2.1.0

# Logging
structlog==23.2.0

# Testing
pytest==7.4.3
pytest-asyncio==0.21.1
pytest-mock==3.12.0

# Development
black==23.11.0
flake8==6.1.0
mypy==1.7.1
```

### 5. Docker Compose Setup

**File**: `edge-gateway-scs\docker-compose.yml`

```yaml
version: "3.8"

services:
  gateway-app:
    build:
      context: ./gateway-app
      dockerfile: Dockerfile
    container_name: beacon-gateway
    ports:
      - "8081:8081" # Gateway API
    volumes:
      - ./gateway-app/config:/app/config
      - ./gateway_data:/app/data
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - GATEWAY_CONFIG=/app/config/gateway.toml
    depends_on:
      - mqtt-broker
      - coap-server
    networks:
      - beacon-network
    restart: unless-stopped

  mqtt-broker:
    image: eclipse-mosquitto:2.0
    container_name: beacon-mqtt
    ports:
      - "1883:1883" # MQTT
      - "8883:8883" # MQTT over TLS
      - "9001:9001" # WebSocket
    volumes:
      - ./iot-broker/mosquitto/config:/mosquitto/config
      - ./iot-broker/mosquitto/data:/mosquitto/data
      - ./iot-broker/mosquitto/log:/mosquitto/log
    networks:
      - beacon-network
    restart: unless-stopped

  coap-server:
    build:
      context: ./iot-broker/coap-server
      dockerfile: Dockerfile
    container_name: beacon-coap
    ports:
      - "5683:5683/udp" # CoAP
    networks:
      - beacon-network
    restart: unless-stopped

  vpn-client:
    build:
      context: ./vpn-client
      dockerfile: Dockerfile
    container_name: beacon-vpn
    cap_add:
      - NET_ADMIN
    devices:
      - /dev/net/tun
    volumes:
      - ./vpn-client/config:/vpn/config
    networks:
      - beacon-network
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    container_name: beacon-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus/config:/etc/prometheus
    command:
      - "--config.file=/etc/prometheus/prometheus.yml"
      - "--storage.tsdb.path=/prometheus"
      - "--web.console.libraries=/etc/prometheus/console_libraries"
      - "--web.console.templates=/etc/prometheus/consoles"
    networks:
      - beacon-network
    restart: unless-stopped

networks:
  beacon-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

### 6. Integration with Existing Blockchain

Your Edge Gateway needs to communicate with the existing BEACON blockchain. Here's how to integrate:

#### Blockchain Client Module

**File**: `edge-gateway-scs\gateway-app\src\blockchain\client.py`

```python
"""
BEACON Blockchain Client for Edge Gateway
Handles communication with the I&O SCS blockchain
"""

import asyncio
import aiohttp
import hashlib
import json
import logging
from typing import Dict, List, Optional
from dataclasses import dataclass

@dataclass
class GatewayPolicy:
    policy_id: str
    gateway_id: str
    hashed_device_id: str
    resource: str
    action: str
    effect: str  # "allow" or "deny"
    created_at: str

class BlockchainClient:
    """Client for communicating with BEACON I&O SCS blockchain"""

    def __init__(self, endpoint: str, gateway_id: str, gateway_salt: str):
        self.endpoint = endpoint.rstrip('/')
        self.gateway_id = gateway_id
        self.gateway_salt = gateway_salt
        self.session: Optional[aiohttp.ClientSession] = None
        self.logger = logging.getLogger('beacon.gateway.blockchain')

    async def connect(self):
        """Initialize connection to blockchain"""
        self.session = aiohttp.ClientSession(
            timeout=aiohttp.ClientTimeout(total=30)
        )

        # Test connection
        try:
            async with self.session.get(f"{self.endpoint}/health") as response:
                if response.status == 200:
                    self.logger.info("Connected to BEACON blockchain")
                else:
                    raise ConnectionError(f"Blockchain health check failed: {response.status}")
        except Exception as e:
            self.logger.error(f"Failed to connect to blockchain: {e}")
            raise

    async def disconnect(self):
        """Close connection to blockchain"""
        if self.session:
            await self.session.close()

    def hash_device_id(self, device_id: str) -> str:
        """Hash device ID for privacy preservation"""
        combined = f"{device_id}:{self.gateway_salt}"
        return hashlib.sha256(combined.encode()).hexdigest()

    async def register_gateway(self) -> bool:
        """Register this gateway with the I&O SCS"""
        try:
            payload = {
                "chaincode_id": "gateway_management",
                "function": "registerGateway",
                "args": [
                    self.gateway_id,
                    "beacon_org",  # Organization ID
                    "gateway_public_key_placeholder"  # TODO: Implement proper key management
                ]
            }

            async with self.session.post(
                f"{self.endpoint}/transactions",
                json=payload,
                headers={"Content-Type": "application/json"}
            ) as response:
                if response.status == 200:
                    result = await response.json()
                    self.logger.info(f"Gateway registered: {result.get('transaction_id')}")
                    return True
                else:
                    error = await response.text()
                    self.logger.error(f"Gateway registration failed: {error}")
                    return False

        except Exception as e:
            self.logger.error(f"Gateway registration error: {e}")
            return False

    async def get_gateway_policies(self) -> List[GatewayPolicy]:
        """Retrieve policies for this gateway"""
        try:
            # Query chaincode for gateway policies
            params = {
                "chaincode_id": "access_control",
                "function": "queryGatewayPolicies",
                "args": [self.gateway_id]
            }

            async with self.session.get(
                f"{self.endpoint}/query",
                params=params
            ) as response:
                if response.status == 200:
                    result = await response.json()
                    policies = []

                    for policy_data in result.get('result', []):
                        policies.append(GatewayPolicy(
                            policy_id=policy_data['policy_id'],
                            gateway_id=policy_data['gateway_id'],
                            hashed_device_id=policy_data['hashed_device_id'],
                            resource=policy_data['resource'],
                            action=policy_data['action'],
                            effect=policy_data['effect'],
                            created_at=policy_data['created_at']
                        ))

                    self.logger.debug(f"Retrieved {len(policies)} policies")
                    return policies
                else:
                    self.logger.error(f"Failed to retrieve policies: {response.status}")
                    return []

        except Exception as e:
            self.logger.error(f"Policy retrieval error: {e}")
            return []

    async def log_event(self, event_type: str, device_id: str, description: str, status: str):
        """Log security event to blockchain (with privacy)"""
        try:
            hashed_device = self.hash_device_id(device_id)

            payload = {
                "chaincode_id": "audit_logging",
                "function": "logEvent",
                "args": [
                    event_type,
                    self.gateway_id,
                    hashed_device,
                    description,
                    status
                ]
            }

            async with self.session.post(
                f"{self.endpoint}/transactions",
                json=payload,
                headers={"Content-Type": "application/json"}
            ) as response:
                if response.status == 200:
                    self.logger.debug(f"Event logged: {event_type}")
                else:
                    self.logger.warning(f"Event logging failed: {response.status}")

        except Exception as e:
            self.logger.error(f"Event logging error: {e}")
```

### 7. Testing Integration

Create a simple test to verify blockchain connectivity:

**File**: `edge-gateway-scs\test_integration.py`

```python
"""
Integration test for Edge Gateway with BEACON blockchain
"""

import asyncio
import sys
import os

# Add the gateway app to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'gateway-app', 'src'))

from blockchain.client import BlockchainClient

async def test_blockchain_connection():
    """Test connection to BEACON blockchain"""
    print("Testing BEACON blockchain connection...")

    # Initialize client
    client = BlockchainClient(
        endpoint="http://localhost:8080/api/v1",
        gateway_id="test_gateway_001",
        gateway_salt="test_salt_123"
    )

    try:
        # Connect to blockchain
        await client.connect()
        print("✅ Successfully connected to blockchain")

        # Test gateway registration
        success = await client.register_gateway()
        if success:
            print("✅ Gateway registration successful")
        else:
            print("❌ Gateway registration failed")

        # Test policy retrieval
        policies = await client.get_gateway_policies()
        print(f"✅ Retrieved {len(policies)} policies")

        # Test event logging
        await client.log_event(
            event_type="test_event",
            device_id="test_device_001",
            description="Integration test event",
            status="success"
        )
        print("✅ Event logging successful")

    except Exception as e:
        print(f"❌ Integration test failed: {e}")
    finally:
        await client.disconnect()
        print("Connection closed")

if __name__ == "__main__":
    # Make sure your BEACON blockchain is running first!
    print("Make sure the BEACON blockchain is running on localhost:8080")
    print("You can start it with: cd beacon-blockchain && cargo run --bin beacon-node")
    print()

    asyncio.run(test_blockchain_connection())
```

### 8. Running the Integration Test

```bash
# 1. Start your BEACON blockchain (in one terminal)
cd beacon-blockchain
cargo run --bin beacon-node

# 2. Run the integration test (in another terminal)
cd edge-gateway-scs
python test_integration.py
```

## Next Development Priorities

1. **Complete Gateway Modules** (Week 1-2):

   - Implement discovery client
   - Implement policy cache and enforcer
   - Implement IoT communication hub
   - Implement VPN manager

2. **Container Integration** (Week 2-3):

   - Set up MQTT broker container
   - Set up CoAP server container
   - Set up VPN client container
   - Test multi-container deployment

3. **End-to-End Testing** (Week 3-4):
   - Create IoT device simulators
   - Test policy enforcement
   - Test privacy preservation
   - Performance testing

## Team Allocation Suggestions

- **Developer 1**: Complete blockchain client and policy modules
- **Developer 2**: Implement IoT communication hub (MQTT/CoAP)
- **Developer 3**: Set up container infrastructure and VPN
- **Developer 4**: Create testing framework and device simulators

This should give your team a clear path forward to complete the Edge Gateway SCS and integrate it with your existing blockchain foundation.
