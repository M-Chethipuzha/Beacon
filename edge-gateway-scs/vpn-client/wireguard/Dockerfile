FROM alpine:latest

# Install WireGuard and required tools
RUN apk add --no-cache wireguard-tools iptables curl bash

# Create WireGuard user
RUN adduser -D -s /bin/bash wg

# Create directories
RUN mkdir -p /etc/wireguard /var/log/wireguard /etc/wireguard/scripts

# Copy configuration files
COPY config/ /etc/wireguard/
COPY scripts/ /etc/wireguard/scripts/

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set permissions
RUN chown -R wg:wg /etc/wireguard /var/log/wireguard
RUN chmod 600 /etc/wireguard/*.conf
RUN chmod +x /etc/wireguard/scripts/*.sh

# Expose health check port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD /etc/wireguard/scripts/health-check.sh || exit 1

# Set working directory
WORKDIR /etc/wireguard

# Note: WireGuard requires privileged mode or specific capabilities
# Run as root for network configuration
USER root

# Start WireGuard
ENTRYPOINT ["/entrypoint.sh"]
CMD ["wg0"]
