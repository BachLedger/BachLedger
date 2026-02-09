# BachLedger Medical Blockchain Node
# Multi-stage build for optimized production image

# ==============================================================================
# Stage 1: Build
# ==============================================================================
FROM rust:1.93-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY rust/Cargo.toml rust/Cargo.lock ./
COPY rust/bach-primitives ./bach-primitives
COPY rust/bach-crypto ./bach-crypto
COPY rust/bach-types ./bach-types
COPY rust/bach-evm ./bach-evm
COPY rust/bach-storage ./bach-storage
COPY rust/bach-network ./bach-network
COPY rust/bach-consensus ./bach-consensus
COPY rust/bach-rpc ./bach-rpc
COPY rust/bach-scheduler ./bach-scheduler
COPY rust/bach-state ./bach-state
COPY rust/bach-contracts ./bach-contracts
COPY rust/bach-node ./bach-node

# Build release binary
RUN cargo build --release --package bach-node

# ==============================================================================
# Stage 2: Runtime
# ==============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r bachledger && useradd -r -g bachledger bachledger

# Create directories
RUN mkdir -p /app /data /keys /config && \
    chown -R bachledger:bachledger /app /data

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/bach-node /app/bach-node

# Copy entrypoint script
COPY deployment/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Switch to non-root user
USER bachledger

# Expose ports
# 8545 - JSON-RPC HTTP
# 8546 - JSON-RPC WebSocket
# 30303 - P2P TCP/UDP
EXPOSE 8545 8546 30303 30303/udp

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8545/health || exit 1

# Data volume
VOLUME ["/data"]

# Default command
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["--config", "/config/node.toml"]
