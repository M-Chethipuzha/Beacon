FROM eclipse-mosquitto:2.0

# Set working directory
WORKDIR /mosquitto

# Copy configuration files
COPY mosquitto.conf /mosquitto/config/mosquitto.conf
COPY acl.conf /mosquitto/config/acl
COPY passwd /mosquitto/config/passwd

# Create necessary directories
RUN mkdir -p /mosquitto/data /mosquitto/log /mosquitto/config/certs

# Set permissions
RUN chown -R mosquitto:mosquitto /mosquitto/data /mosquitto/log
RUN chmod 644 /mosquitto/config/mosquitto.conf
RUN chmod 644 /mosquitto/config/acl
RUN chmod 600 /mosquitto/config/passwd

# Expose ports
EXPOSE 1883 8883 9001 9002

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD mosquitto_pub -h localhost -p 1883 -t '$SYS/broker/uptime' -n -u beacon-gateway -P beacon_mqtt_password || exit 1

# Start mosquitto
CMD ["/usr/sbin/mosquitto", "-c", "/mosquitto/config/mosquitto.conf"]
