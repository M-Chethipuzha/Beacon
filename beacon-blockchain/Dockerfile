# BEACON Blockchain - Production Docker Image
# Multi-stage build for optimized production deployment

# Stage 1: Build Stage
FROM rust:1.75-bullseye AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
COPY proto/ proto/

# Build the project
RUN cargo build --release --bin beacon-api --bin beacon-node

# Stage 2: Runtime Stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    gosu \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create beacon user with specific UID/GID for consistent permissions
RUN groupadd -r beacon --gid=1000 && \
    useradd -r -g beacon --uid=1000 --home-dir=/data/beacon --shell=/bin/bash beacon

# Create data directories with proper permissions
RUN mkdir -p /data/beacon/storage /data/beacon/logs /data/beacon/config \
    && chown -R 1000:1000 /data/beacon \
    && chmod -R 755 /data/beacon

# Copy binaries from builder stage
COPY --from=builder /app/target/release/beacon-api /usr/local/bin/
COPY --from=builder /app/target/release/beacon-node /usr/local/bin/

# Copy configuration templates and entrypoint
COPY docker/config/ /etc/beacon/
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Switch to beacon user
USER beacon

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# Expose ports
EXPOSE 3000 8080 8081 9000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Default command (can be overridden)
CMD ["beacon-api"]

# Metadata
LABEL org.opencontainers.image.title="BEACON Blockchain"
LABEL org.opencontainers.image.description="Production-ready BEACON blockchain node"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.source="https://github.com/beacon-blockchain/beacon"
