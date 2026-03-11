# Flagship Docker

Development stack for Flagship frontend backed by Citadel mesh.

## Quick Start

```bash
# Build the Citadel binary first (cross-compile for Docker)
cd ~/projects/citadel
cross build --release -p citadel-lens --target aarch64-unknown-linux-gnu

# Start the full stack
cd ~/projects/flagship/docker
./riffstack.py up 5

# Open http://localhost:9999
```

## Commands

| Command | Description |
|---------|-------------|
| `./riffstack.py up [N]` | Start stack with N Citadel nodes (default: 5) |
| `./riffstack.py down` | Stop everything |
| `./riffstack.py logs [service]` | View logs |
| `./riffstack.py ps` | List containers |
| `./riffstack.py restart` | Restart frontend only |

## Services

| Service | Port | Description |
|---------|------|-------------|
| `flagship-dev` | 9999 | Vite dev server with hot-reload |
| `citadel-lb` | 8085 | HAProxy load balancer for API |
| `citadel-lb` | 8404 | HAProxy stats dashboard |
| `citadel-1..N` | - | Mesh nodes |

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│  Browser        │────▶│  flagship-dev   │
│  localhost:9999 │     │  (Vite)         │
└─────────────────┘     └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │  citadel-lb     │
                        │  (HAProxy)      │
                        │  localhost:8085 │
                        └────────┬────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              ▼                  ▼                  ▼
       ┌────────────┐     ┌────────────┐     ┌────────────┐
       │ citadel-1  │────▶│ citadel-2  │────▶│ citadel-3  │
       └────────────┘     └────────────┘     └────────────┘
```

## Hot Reload

Frontend hot-reloads automatically when you edit files.

Backend hot-reloads when you rebuild the binary:

```bash
# In another terminal
cd ~/projects/citadel
cross build --release -p citadel-lens --target aarch64-unknown-linux-gnu
# Nodes detect the change and restart automatically
```

## Legacy

`cluster.py` is the old monolithic script. Use `riffstack.py` instead - it delegates Citadel concerns to `~/projects/citadel/docker/citadel.py`.
