#!/bin/sh

# BEACON OpenVPN Client - Entrypoint Script
# =========================================

set -e

# Configuration
CONFIG_FILE=${1:-client.ovpn}
LOG_FILE="/var/log/openvpn/openvpn.log"
PID_FILE="/var/run/openvpn.pid"

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Cleanup function
cleanup() {
    log "Shutting down OpenVPN client..."
    if [ -f "$PID_FILE" ]; then
        kill $(cat "$PID_FILE") 2>/dev/null || true
        rm -f "$PID_FILE"
    fi
    exit 0
}

# Set up signal handlers
trap cleanup INT TERM

# Validate configuration
if [ ! -f "/etc/openvpn/client/$CONFIG_FILE" ]; then
    log "ERROR: Configuration file $CONFIG_FILE not found"
    exit 1
fi

# Check for required certificates
if [ ! -f "/etc/openvpn/client/ca.crt" ]; then
    log "WARNING: CA certificate not found"
fi

if [ ! -f "/etc/openvpn/client/client.crt" ]; then
    log "WARNING: Client certificate not found"
fi

if [ ! -f "/etc/openvpn/client/client.key" ]; then
    log "WARNING: Client private key not found"
fi

# Create log directory
mkdir -p /var/log/openvpn

# Start health check server in background
/etc/openvpn/scripts/health-server.sh &

log "Starting BEACON OpenVPN client with config: $CONFIG_FILE"

# Start OpenVPN
exec openvpn \
    --config "/etc/openvpn/client/$CONFIG_FILE" \
    --writepid "$PID_FILE" \
    --log "$LOG_FILE" \
    --verb 3 \
    --daemon
