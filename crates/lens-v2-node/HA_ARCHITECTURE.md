# Lens V2 Node - High Availability Architecture

## Overview

Lens V2 follows the **"Defederation" model** - each node is **independent and self-contained**, with data synchronized via **P2P CRDT-style replication**.

This is fundamentally different from traditional HA clustering:

### Traditional HA (What We DON'T Do)
- Shared storage backend (PostgreSQL, etc.)
- Strong consistency requirements
- Leader election / failover
- Synchronized writes across cluster

### Defederation Model (What We DO)
- **Independent nodes** with local storage (RocksDB)
- **Eventual consistency** via P2P sync
- **No leader** - every node is equal
- **CRDT-style conflict resolution**
- **One-way subscriptions** - nodes pull from others they follow

## Architecture Components

### 1. Local Storage (RocksDB)
Each node has its own **independent** RocksDB database:
- Location: `.lens-node-data/rocksdb/`
- Stores: releases, featured content, admin keys
- Fully self-contained - no shared storage

### 2. P2P Sync Layer (`lens-v2-p2p`)
Handles distributed synchronization:
- **BFT Consensus**: Agreement on block metadata across peers
- **TGP (The Graph Protocol)**: Efficient block exchange
- **Sync Tracker**: Monitors sync status, tracks missing blocks
- **Peer Discovery**: WebRTC relay for peer connections

### 3. Sync Status Endpoint (`/ready`)
Health check for sync status:
- **HTTP 200 OK** - Node is synced and ready
- **HTTP 503 Service Unavailable** - Node is behind or syncing

Response format:
```json
{
  "is_synced": true|false,
  "network_height": 150,
  "local_height": 150,
  "blocks_behind": 0,
  "peer_count": 3,
  "downloading": []
}
```

## How It Works

### Single Node Deployment
```
┌─────────────────────────┐
│     Lens Node A         │
│                         │
│  ┌──────────────────┐   │
│  │   RocksDB (local)│   │
│  │  - Releases      │   │
│  │  - Featured      │   │
│  └──────────────────┘   │
│                         │
│  ┌──────────────────┐   │
│  │   P2P Manager    │   │
│  │  (standalone)    │   │
│  └──────────────────┘   │
└─────────────────────────┘

/ready → 503 (no peers, but still functional)
```

### Multi-Node Defederated Network
```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  Lens Node A     │     │  Lens Node B     │     │  Lens Node C     │
│                  │     │                  │     │                  │
│  ┌───────────┐   │     │  ┌───────────┐   │     │  ┌───────────┐   │
│  │ RocksDB   │   │     │  │ RocksDB   │   │     │  │ RocksDB   │   │
│  │ (local)   │   │     │  │ (local)   │   │     │  │ (local)   │   │
│  └───────────┘   │     │  └───────────┘   │     │  └───────────┘   │
│        ▲         │     │        ▲         │     │        ▲         │
│        │         │     │        │         │     │        │         │
│  ┌─────┴──────┐  │     │  ┌─────┴──────┐  │     │  ┌─────┴──────┐  │
│  │ P2P Sync   │◀─┼─────┼─▶│ P2P Sync   │◀─┼─────┼─▶│ P2P Sync   │  │
│  │ (BFT/TGP)  │  │     │  │ (BFT/TGP)  │  │     │  │ (BFT/TGP)  │  │
│  └────────────┘  │     │  └────────────┘  │     │  └────────────┘  │
└──────────────────┘     └──────────────────┘     └──────────────────┘
         │                         │                         │
         └─────────────────────────┴─────────────────────────┘
                    P2P Federation Network
                    (CRDT-style sync)

/ready → 200 (synced with peers)
```

## Federation Workflow

### 1. Bootstrap
Node A starts with empty database:
```
Node A → /ready → 503 Service Unavailable
{
  "is_synced": false,
  "peer_count": 0,
  "blocks_behind": 0
}
```

### 2. Peer Discovery
Node A discovers peers (via relay or bootstrappers):
```
Node A connects to Nodes B, C
→ peer_count: 3
→ BFT consensus establishes network_height
```

### 3. Historical Sync
Node A fetches missing content from peers:
```
Node A sees network_height: 100
Node A local_height: 0
→ blocks_behind: 100
→ Starts downloading via TGP
→ /ready → 503 (syncing)
```

### 4. Sync Complete
Node A catches up:
```
local_height: 100
network_height: 100
blocks_behind: 0
peer_count: 3
→ /ready → 200 OK
```

### 5. Live Updates
New content published on Node B:
```
Node B creates release
→ BFT proposes new block (height 101)
→ Consensus reached
→ Node A, Node C receive update via P2P
→ All nodes update local RocksDB
→ Eventual consistency maintained
```

## Key Principles

### 1. **Independence**
- Each node can run standalone
- No dependency on other nodes for basic operation
- Local storage is authoritative

### 2. **Eventual Consistency**
- Writes happen locally first
- Sync happens asynchronously
- Conflicts resolved via CRDT semantics
- No blocking on remote nodes

### 3. **Peer-to-Peer**
- No central coordinator
- No single point of failure
- Nodes can join/leave freely
- Dynamic topology

### 4. **Subscription Model**
- Nodes "follow" other nodes
- One-way content replication
- Subscriber pulls from publisher
- Publisher broadcasts updates

## Benefits

### Resilience
- **No single point of failure** - any node can go down
- **Partition tolerance** - network splits don't block writes
- **Self-healing** - nodes catch up when reconnected

### Scalability
- **Horizontal scaling** - add more nodes easily
- **No write coordination** - each node writes locally
- **Efficient sync** - only missing blocks transferred

### Operational Simplicity
- **No shared infrastructure** - each node is self-contained
- **No complex failover** - all nodes are equal
- **Easy deployment** - just start more nodes

## Monitoring

### Health Checks
```bash
# Check if node is synced and ready
curl http://localhost:5002/api/v1/ready

# 200 OK → Node is ready to serve
# 503 Service Unavailable → Node is syncing
```

### Sync Status Details
```bash
# Get detailed sync status
curl http://localhost:5002/api/v1/ready | jq .

{
  "is_synced": true,          # Ready to serve?
  "network_height": 150,      # Latest block in network
  "local_height": 150,        # Latest block we have
  "blocks_behind": 0,         # How far behind we are
  "peer_count": 3,            # Number of connected peers
  "downloading": []           # Blocks currently being fetched
}
```

### Expected Behavior

**Normal Operation:**
- `is_synced: true`
- `blocks_behind: 0`
- `peer_count > 0`
- HTTP 200 OK

**Syncing:**
- `is_synced: false`
- `blocks_behind > 0`
- `peer_count > 0`
- `downloading` contains block IDs
- HTTP 503 Service Unavailable

**Isolated:**
- `is_synced: false`
- `peer_count: 0`
- Still functional, but no sync
- HTTP 503 Service Unavailable

## Future Enhancements

### Planned Features
- [ ] Active federation subscription management
- [ ] CRDT write conflict resolution
- [ ] Historical sync implementation
- [ ] Live update broadcasts
- [ ] Peer discovery via DHT
- [ ] Bandwidth-efficient delta sync

### Current Status
- [x] RocksDB local persistence
- [x] P2P manager infrastructure
- [x] Sync status tracking
- [x] `/ready` endpoint
- [x] P2P network layer with WebSocket relay
- [x] Citadel DHT integration for peer discovery
- [x] Sync orchestrator coordinating network + consensus + persistence
- [ ] Block serialization/deserialization (Release ↔ BlockData)
- [ ] TGP-based block exchange
- [ ] Full integration testing

## Comparison to Legacy Lens-Node

### Old (Peerbit-based)
- Uses Peerbit for distributed database
- CRDT via Peerbit's SharedLog
- OrbitDB for storage
- JavaScript/TypeScript

### New (Lens V2)
- Uses RocksDB for local storage
- Custom BFT + TGP consensus
- Pure Rust implementation
- WASM-compatible design
- Same defederation principles

## References

- [Defederation Concept](https://riff.cc/docs/concepts/defederation/)
- [Lenses Concept](https://riff.cc/docs/concepts/lenses/)
- [Legacy lens-node](https://github.com/riffcc/lens-node)
- [Legacy lens-sdk](https://github.com/riffcc/lens-sdk)
