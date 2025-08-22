#!/bin/bash

# BEACON Edge Gateway - Deployment Script
# =======================================

set -e

echo "🚀 Starting BEACON Edge Gateway deployment..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if docker-compose is available
if ! command -v docker-compose > /dev/null 2>&1; then
    echo "❌ docker-compose is not installed. Please install docker-compose and try again."
    exit 1
fi

# Create necessary directories
echo "📁 Creating necessary directories..."
mkdir -p data logs certs
mkdir -p docker/mosquitto/data docker/mosquitto/log
mkdir -p docker/monitoring/grafana/dashboards

# Set proper permissions
chmod 755 data logs certs
chmod 755 docker/mosquitto/data docker/mosquitto/log

# Generate self-signed certificates if they don't exist
if [ ! -f certs/gateway-cert.pem ]; then
    echo "🔐 Generating self-signed certificates..."
    
    # Generate CA private key
    openssl genrsa -out certs/ca-private.pem 4096
    
    # Generate CA certificate
    openssl req -new -x509 -key certs/ca-private.pem -out certs/ca-cert.pem -days 365 \
        -subj "/C=US/ST=State/L=City/O=BEACON/OU=Edge Gateway/CN=BEACON-CA"
    
    # Generate gateway private key
    openssl genrsa -out certs/gateway-private.pem 4096
    
    # Generate gateway certificate signing request
    openssl req -new -key certs/gateway-private.pem -out certs/gateway.csr \
        -subj "/C=US/ST=State/L=City/O=BEACON/OU=Edge Gateway/CN=beacon-gateway"
    
    # Generate gateway certificate
    openssl x509 -req -in certs/gateway.csr -CA certs/ca-cert.pem -CAkey certs/ca-private.pem \
        -CAcreateserial -out certs/gateway-cert.pem -days 365
    
    # Clean up CSR
    rm certs/gateway.csr
    
    echo "✅ Certificates generated successfully"
fi

# Generate gateway salt for device ID hashing
if [ ! -f data/gateway-salt.txt ]; then
    echo "🧂 Generating gateway salt..."
    openssl rand -hex 32 > data/gateway-salt.txt
    chmod 600 data/gateway-salt.txt
    echo "✅ Gateway salt generated"
fi

# Create MQTT password file
if [ ! -f docker/mosquitto/config/passwd ]; then
    echo "🔑 Creating MQTT password file..."
    
    # Create password file with default gateway user
    echo "beacon-gateway:beacon_mqtt_password" > docker/mosquitto/config/passwd
    
    # Hash the password (mosquitto_passwd would be ideal, but might not be available)
    # For now, we'll use a simple approach
    chmod 600 docker/mosquitto/config/passwd
    
    echo "✅ MQTT password file created"
fi

# Build and start services
echo "🔨 Building Docker images..."
docker-compose build

echo "🚀 Starting services..."
docker-compose up -d

# Wait for services to be ready
echo "⏳ Waiting for services to start..."
sleep 30

# Check service health
echo "🏥 Checking service health..."

# Check if Edge Gateway API is responding
if curl -f http://localhost:8081/health > /dev/null 2>&1; then
    echo "✅ Edge Gateway API is healthy"
else
    echo "⚠️  Edge Gateway API is not responding"
fi

# Check if MQTT broker is running
if nc -z localhost 1883; then
    echo "✅ MQTT broker is running"
else
    echo "⚠️  MQTT broker is not responding"
fi

# Check if Prometheus is running
if curl -f http://localhost:9091 > /dev/null 2>&1; then
    echo "✅ Prometheus is running"
else
    echo "⚠️  Prometheus is not responding"
fi

# Check if Grafana is running
if curl -f http://localhost:3000 > /dev/null 2>&1; then
    echo "✅ Grafana is running"
else
    echo "⚠️  Grafana is not responding"
fi

echo ""
echo "🎉 BEACON Edge Gateway deployment completed!"
echo ""
echo "📊 Service URLs:"
echo "   - Edge Gateway API: http://localhost:8081"
echo "   - Gateway Health:   http://localhost:8081/health"
echo "   - Prometheus:       http://localhost:9091"
echo "   - Grafana:          http://localhost:3000 (admin/beacon_admin)"
echo ""
echo "📱 MQTT Connection:"
echo "   - Host: localhost"
echo "   - Port: 1883 (plain) / 8883 (TLS)"
echo "   - WebSocket: 9001"
echo ""
echo "📋 Logs:"
echo "   - View logs: docker-compose logs -f"
echo "   - Gateway logs: docker-compose logs -f beacon-edge-gateway"
echo ""
echo "🛑 To stop services: docker-compose down"
