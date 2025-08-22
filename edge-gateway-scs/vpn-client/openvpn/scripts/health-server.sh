#!/bin/sh

# BEACON OpenVPN - Health Server Script
# =====================================

# Simple HTTP server for health checks
PORT=8080

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Health Server: $1"
}

# Create a simple HTTP response
create_response() {
    local status=$1
    local body=$2
    
    cat << EOF
HTTP/1.1 $status
Content-Type: text/plain
Content-Length: ${#body}
Connection: close

$body
EOF
}

# Handle HTTP request
handle_request() {
    # Run health check
    if /etc/openvpn/scripts/health-check.sh > /dev/null 2>&1; then
        create_response "200 OK" "OpenVPN healthy"
    else
        create_response "503 Service Unavailable" "OpenVPN unhealthy"
    fi
}

log "Starting health server on port $PORT"

# Simple netcat-based HTTP server
while true; do
    handle_request | nc -l -p $PORT -q 1
    sleep 1
done
