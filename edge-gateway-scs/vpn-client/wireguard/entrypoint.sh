#!/bin/bash

# BEACON WireGuard Client - Entrypoint Script
# ===========================================

set -e

# Configuration
INTERFACE=${1:-wg0}
CONFIG_FILE="/etc/wireguard/${INTERFACE}.conf"
LOG_FILE="/var/log/wireguard/wireguard.log"

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Cleanup function
cleanup() {
    log "Shutting down WireGuard interface: $INTERFACE"
    wg-quick down "$INTERFACE" 2>/dev/null || true
    exit 0
}

# Set up signal handlers
trap cleanup INT TERM

# Validate configuration
if [ ! -f "$CONFIG_FILE" ]; then
    log "ERROR: Configuration file $CONFIG_FILE not found"
    exit 1
fi

# Create log directory
mkdir -p /var/log/wireguard

# Start health check server in background
/etc/wireguard/scripts/health-server.sh &

log "Starting BEACON WireGuard client with interface: $INTERFACE"

# Bring up WireGuard interface
if wg-quick up "$CONFIG_FILE"; then
    log "WireGuard interface $INTERFACE is up"
else
    log "ERROR: Failed to bring up WireGuard interface"
    exit 1
fi

# Monitor the interface
while true; do
    if ! wg show "$INTERFACE" > /dev/null 2>&1; then
        log "ERROR: WireGuard interface $INTERFACE is down"
        exit 1
    fi
    sleep 30
done
