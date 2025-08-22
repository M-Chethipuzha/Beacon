FROM alpine:latest

# Install OpenVPN and required tools
RUN apk add --no-cache openvpn openssl curl iptables

# Create OpenVPN user
RUN adduser -D -s /bin/sh openvpn

# Create directories
RUN mkdir -p /etc/openvpn/client /var/log/openvpn /etc/openvpn/scripts

# Copy configuration files
COPY config/ /etc/openvpn/client/
COPY scripts/ /etc/openvpn/scripts/

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set permissions
RUN chown -R openvpn:openvpn /etc/openvpn/client /var/log/openvpn
RUN chmod 600 /etc/openvpn/client/*.ovpn /etc/openvpn/client/*.key
RUN chmod +x /etc/openvpn/scripts/*.sh

# Create device for tun interface
RUN mkdir -p /dev/net && mknod /dev/net/tun c 10 200

# Expose health check
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD /etc/openvpn/scripts/health-check.sh || exit 1

# Set working directory
WORKDIR /etc/openvpn/client

# Run as openvpn user (note: may need privileged mode for VPN)
USER openvpn

# Start OpenVPN
ENTRYPOINT ["/entrypoint.sh"]
CMD ["client.ovpn"]
