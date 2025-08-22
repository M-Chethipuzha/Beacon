FROM python:3.11-slim

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    libc-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application source
COPY src/ ./src/
COPY config/ ./config/

# Create non-root user
RUN groupadd -r coap && useradd -r -g coap -d /app -s /bin/bash coap
RUN chown -R coap:coap /app

# Switch to non-root user
USER coap

# Expose CoAP port
EXPOSE 5683/udp

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD python -c "import socket; s=socket.socket(socket.AF_INET, socket.SOCK_DGRAM); s.settimeout(5); s.connect(('localhost', 5683)); s.close()" || exit 1

# Set environment variables
ENV PYTHONPATH=/app/src
ENV COAP_CONFIG_FILE=/app/config/coap.conf

# Start the CoAP server
CMD ["python", "-m", "src.coap_server"]
