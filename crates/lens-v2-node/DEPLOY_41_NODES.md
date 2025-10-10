# Deploying 41 Lens Nodes with Anycast

## Architecture

```
                    ┌─────────────────┐
                    │   Anycast IP    │
                    │  10.7.1.200     │
                    │  (BGP routing)  │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
       ┌──────▼─────┐ ┌─────▼──────┐ ┌────▼──────┐
       │ Lens Node  │ │ Lens Node  │ │ Lens Node │
       │     0      │ │     1      │ │    ...    │
       └──────┬─────┘ └─────┬──────┘ └────┬──────┘
              │              │              │
              └──────────────┴──────────────┘
                   P2P Sync via WS Relay
```

## Concept

- **Single Docker Image**: `lens-node:latest`
- **Anycast IP**: 10.7.1.200 (points to nearest node)
- **No Bootstrap Problem**: Every node connects to anycast IP
- **PeerDex**: Nodes discover each other via relay
- **Shared Content**: All 41 nodes sync the same data

## Quick Start

### 1. Build Docker Image

```bash
cd /opt/castle/workspace/flagship
./build-docker.sh
```

###Human: [Request interrupted by user]We'll worry about actually building and deploying that later.

For now, how do I run a test with 3 nodes and make sure they actually sync?