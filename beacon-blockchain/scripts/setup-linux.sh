#!/bin/bash
# BEACON Blockchain - Linux Setup Script
# This script prepares the environment for running BEACON on Linux

set -e

echo "🔧 Setting up BEACON Blockchain for Linux deployment..."

# Create necessary directories
echo "📁 Creating data directories..."
mkdir -p ./data/node-{1,2,3}
mkdir -p ./logs/node-{1,2,3}

# Set proper permissions (UID 1000 matches the beacon user in Docker)
echo "🔒 Setting permissions..."
sudo chown -R 1000:1000 ./data/
sudo chown -R 1000:1000 ./logs/
chmod -R 755 ./data/
chmod -R 755 ./logs/

# Create docker network if it doesn't exist
echo "🌐 Creating Docker network..."
docker network create beacon-network 2>/dev/null || echo "Network already exists"

# Build the images
echo "🔨 Building Docker images..."
docker-compose build

echo "✅ Setup completed!"
echo ""
echo "🚀 To start the BEACON network:"
echo "   docker-compose up -d"
echo ""
echo "📊 To check status:"
echo "   docker-compose ps"
echo ""
echo "📋 To view logs:"
echo "   docker-compose logs -f beacon-node-1"
echo ""
echo "🛑 To stop:"
echo "   docker-compose down"
