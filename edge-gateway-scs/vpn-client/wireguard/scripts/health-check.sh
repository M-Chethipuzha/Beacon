#!/bin/bash

# BEACON WireGuard - Health Check Script
# ======================================

INTERFACE="wg0"

# Check if WireGuard interface exists
if ! ip link show "$INTERFACE" > /dev/null 2>&1; then
    echo "WireGuard interface $INTERFACE not found"
    exit 1
fi

# Check if interface is up
if ! ip link show "$INTERFACE" | grep -q "state UP"; then
    echo "WireGuard interface $INTERFACE is down"
    exit 1
fi

# Check WireGuard status
if ! wg show "$INTERFACE" > /dev/null 2>&1; then
    echo "WireGuard interface $INTERFACE is not configured"
    exit 1
fi

# Check if we have a peer configured
if ! wg show "$INTERFACE" | grep -q "peer:"; then
    echo "No WireGuard peers configured"
    exit 1
fi

# Check connectivity through tunnel
# Get the tunnel IP
TUNNEL_IP=$(ip addr show "$INTERFACE" | grep -oP 'inet \K[^/]+' | head -n1)

if [ -z "$TUNNEL_IP" ]; then
    echo "No IP address assigned to tunnel interface"
    exit 1
fi

# Test connectivity to gateway (using the gateway IP from config)
GATEWAY_IP="10.10.10.1"  # Adjust based on your configuration

if ! ping -c 1 -W 5 "$GATEWAY_IP" > /dev/null 2>&1; then
    echo "Cannot reach WireGuard gateway"
    exit 1
fi

# Test DNS resolution
if ! nslookup google.com > /dev/null 2>&1; then
    echo "DNS resolution failed"
    exit 1
fi

# Check peer handshake (should be recent)
LAST_HANDSHAKE=$(wg show "$INTERFACE" latest-handshakes | cut -f2)
CURRENT_TIME=$(date +%s)

if [ -n "$LAST_HANDSHAKE" ] && [ "$LAST_HANDSHAKE" != "0" ]; then
    TIME_DIFF=$((CURRENT_TIME - LAST_HANDSHAKE))
    if [ $TIME_DIFF -gt 300 ]; then  # 5 minutes
        echo "Last handshake was more than 5 minutes ago"
        exit 1
    fi
fi

echo "WireGuard health check passed"
exit 0
