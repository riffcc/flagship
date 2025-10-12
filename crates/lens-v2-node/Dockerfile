# Lens V2 Node - Docker Image
# Build from /opt/castle/workspace: docker build -f flagship/crates/lens-v2-node/Dockerfile -t lens-v2-node .

FROM rust:1.90.0-trixie as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create build directory
WORKDIR /build

# Copy workspace Cargo files
COPY flagship/Cargo.toml flagship/Cargo.lock ./

# Copy flagship crates and packages
COPY flagship/crates ./crates
COPY flagship/packages ./packages

# Copy citadel workspace (needs all for workspace build)
COPY citadel /citadel

# Copy palace workspace (needs all for workspace build)
COPY palace /palace

# Build lens-v2-node
RUN cargo build --release -p lens-v2-node

# Runtime stage
FROM debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create lens user
RUN useradd -m -u 1000 lens

# Copy binary
COPY --from=builder /build/target/release/lens-node /usr/local/bin/

# Create data directory
RUN mkdir -p /data/.lens-node-data && chown -R lens:lens /data

# Switch to lens user
USER lens
WORKDIR /data

# Expose ports
EXPOSE 5000

# Environment variables (can be overridden)
ENV RUST_LOG=info
ENV PORT=5000
ENV DB_PATH=/data/.lens-node-data
ENV RELAY_URL=ws://localhost:5000/api/v1/relay/ws

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT}/api/v1/ready || exit 1

# Run lens-node
CMD ["lens-node"]
