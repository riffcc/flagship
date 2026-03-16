# Web build stage
FROM node:22-alpine AS web-builder

# Install build dependencies
RUN apk add --no-cache python3 py3-setuptools make g++ git

# Install pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

# Set working directory
WORKDIR /app

# Copy package files
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile --ignore-scripts --unsafe-perm

# Copy source code
COPY . .

# Generate Electron vendors file
RUN node scripts/update-electron-vendors.mjs

# Build the web version
RUN pnpm build

# Rust runtime builder
FROM debian:13 AS rust-builder

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal --default-toolchain 1.94.0

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY packages/epub-wasm/Cargo.toml packages/epub-wasm/Cargo.toml
COPY packages/epub-wasm/src packages/epub-wasm/src
COPY packages/flagship-serve/Cargo.toml packages/flagship-serve/Cargo.toml
COPY packages/flagship-serve/src packages/flagship-serve/src
RUN cargo build --locked --release -p flagship-cli --bin flagship

# Production stage
FROM debian:13

ENV FLAGSHIP_STATIC_DIR=/srv/flagship
ENV LENS_NODE=http://127.0.0.1:8080/api/v1

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=web-builder /app/packages/renderer/dist/web /srv/flagship
COPY --from=rust-builder /app/target/release/flagship /usr/local/bin/flagship

# Expose port 80
EXPOSE 80

ENTRYPOINT ["/usr/local/bin/flagship", "serve"]
