FROM python:3.11-slim

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements and install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application source
COPY src/ ./src/
COPY config/ ./config/

# Create data and logs directories
RUN mkdir -p data logs certs

# Create non-root user for security
RUN groupadd -r beacon && useradd -r -g beacon -d /app -s /bin/bash beacon
RUN chown -R beacon:beacon /app

# Switch to non-root user
USER beacon

# Expose ports
EXPOSE 8081 5683/udp 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD curl -f http://localhost:8081/health || exit 1

# Set environment variables
ENV PYTHONPATH=/app/src
ENV BEACON_CONFIG_FILE=/app/config/gateway.toml

# Start the application
CMD ["python", "-m", "src.main"]
