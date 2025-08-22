#!/bin/sh

# BEACON OpenVPN - Health Check Script
# ====================================

# Check if OpenVPN process is running
if ! pgrep openvpn > /dev/null; then
    echo "OpenVPN process not running"
    exit 1
fi

# Check if tunnel interface exists
if ! ip link show tun0 > /dev/null 2>&1; then
    echo "Tunnel interface not found"
    exit 1
fi

# Check if we can resolve DNS
if ! nslookup google.com > /dev/null 2>&1; then
    echo "DNS resolution failed"
    exit 1
fi

# Check connectivity to a test endpoint
if ! ping -c 1 -W 5 8.8.8.8 > /dev/null 2>&1; then
    echo "Network connectivity test failed"
    exit 1
fi

# Check VPN status file
STATUS_FILE="/var/log/openvpn/status.log"
if [ -f "$STATUS_FILE" ]; then
    if ! grep -q "CONNECTED" "$STATUS_FILE"; then
        echo "VPN not in connected state"
        exit 1
    fi
fi

echo "OpenVPN health check passed"
exit 0
