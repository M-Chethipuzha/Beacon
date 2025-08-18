#!/bin/bash
set -e

# Function to wait for directory to be available and fix permissions
fix_permissions() {
    local dir="$1"
    
    # Create directory if it doesn't exist
    if [ ! -d "$dir" ]; then
        echo "Creating directory: $dir"
        mkdir -p "$dir"
    fi
    
    # Fix ownership if running as root (during initialization)
    if [ "$(id -u)" = "0" ]; then
        echo "Fixing permissions for: $dir"
        chown -R 1000:1000 "$dir"
        chmod -R 755 "$dir"
    fi
}

# Fix permissions for data directories
fix_permissions "/data/beacon/storage"
fix_permissions "/data/beacon/logs"
fix_permissions "/data/beacon/config"

# If running as root, switch to beacon user
if [ "$(id -u)" = "0" ]; then
    echo "Switching to beacon user..."
    exec gosu 1000:1000 "$@"
else
    echo "Running as beacon user..."
    exec "$@"
fi
