# Flagship Docker

## Base Images

Use **Debian 13 Trixie** (stable as of 2025) for all container images:

- Rust: `rust:1.92-slim-trixie` or later
- Node: `node:22-alpine` with custom Dockerfile.dev for build deps
- Debian: `debian:trixie-slim`

Trixie has GLIBC 2.39+ which is required for modern binaries like `watchexec`.

## Services

- `flagship-dev` - Vite dev server on port 5175 (mapped to 9999)
- `citadel-builder` - Rust build environment with watchexec for hot reload
- `citadel-{1..N}` - Citadel lens nodes
- `citadel-lb` - HAProxy load balancer for citadel nodes

## Airgapped Operation

The flagship-dev container uses `pnpm install || true` so it can start even without network access, as long as node_modules is cached in the volume.
