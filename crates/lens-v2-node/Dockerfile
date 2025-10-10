# Lens V2 Node - Docker Image
# Anycast-ready P2P lens node

FROM rust:1.80-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Copy palace consensus crates (symlinked dependencies)
COPY ../../../palace/crates/consensus ./palace/crates/consensus

# Build lens-v2-node
RUN cargo build --release -p lens-v2-node

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create lens user
RUN useradd -m -u 1000 lens

# Copy binary
COPY --from=builder /build/target/release/lens-node /usr/local/bin/

# Create data directory
RUN mkdir -p /data/rocksdb && chown -R lens:lens /data

# Switch to lens user
USER lens
WORKDIR /data

# Expose ports
EXPOSE 5002

# Environment variables (can be overridden)
ENV RUST_LOG=info
ENV PORT=5002
ENV DB_PATH=/data/rocksdb
ENV LENS_CLUSTER_ID=lens-cluster-prod
ENV LENS_ANYCAST_IP=10.7.1.200
ENV LENS_RELAY_URL=ws://${LENS_ANYCAST_IP}:5002/api/v1/relay/ws

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5002/api/v1/health || exit 1

# Run lens-node
CMD ["lens-node"]
