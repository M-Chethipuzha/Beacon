#!/bin/bash

# BEACON WireGuard - Health Server Script
# =======================================

PORT=8080

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Health Server: $1"
}

# Create HTTP response
create_response() {
    local status=$1
    local body=$2
    
    cat << EOF
HTTP/1.1 $status
Content-Type: application/json
Content-Length: ${#body}
Connection: close

$body
EOF
}

# Get WireGuard status
get_wg_status() {
    local interface="wg0"
    local status="{}"
    
    if wg show "$interface" > /dev/null 2>&1; then
        local peer_count=$(wg show "$interface" peers | wc -l)
        local endpoint=$(wg show "$interface" endpoints | head -n1 | cut -f2)
        local latest_handshake=$(wg show "$interface" latest-handshakes | cut -f2)
        local transfer=$(wg show "$interface" transfer | head -n1)
        
        status=$(cat << EOF
{
    "interface": "$interface",
    "status": "connected",
    "peer_count": $peer_count,
    "endpoint": "$endpoint",
    "latest_handshake": "$latest_handshake",
    "transfer": "$transfer"
}
EOF
)
    else
        status='{"interface": "wg0", "status": "disconnected"}'
    fi
    
    echo "$status"
}

# Handle HTTP request
handle_request() {
    if /etc/wireguard/scripts/health-check.sh > /dev/null 2>&1; then
        local status_json=$(get_wg_status)
        create_response "200 OK" "$status_json"
    else
        create_response "503 Service Unavailable" '{"status": "unhealthy", "error": "WireGuard health check failed"}'
    fi
}

log "Starting WireGuard health server on port $PORT"

# Simple HTTP server using nc
while true; do
    handle_request | nc -l -p $PORT -q 1 2>/dev/null || sleep 1
done
