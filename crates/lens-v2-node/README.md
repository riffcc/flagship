# Lens Node v2

High-performance Rust backend for Riff.CC with P2P capabilities and distributed metadata storage.

## Overview

Lens Node v2 is the next-generation backend for the Riff.CC platform, providing:

- **Distributed Hash Table (DHT)**: Citadel DHT integration for decentralized metadata storage
- **P2P Networking**: WebRTC-based peer-to-peer connections for content distribution
- **HTTP API**: RESTful API for browser clients
- **WebSocket Support**: Real-time updates and streaming
- **Metadata Management**: Releases, categories, featured content, and search
- **Encryption**: Optional ChaCha20-Poly1305 encryption for private DHT data
- **Metrics & Monitoring**: Comprehensive health checks and performance tracking

## Features

### Citadel DHT Integration (v0.7.2)

Lens Node v0.7.2 integrates **Citadel DHT**, a revolutionary distributed hash table with O(1) key lookups:

- **O(1) Lookups**: Geometric routing provides constant-time key lookups
- **2.5D Hexagonal Toroidal Mesh**: Innovative topology with 8-neighbor discovery
- **Provably Optimal Routing**: Every path can be cryptographically verified
- **64-Byte Node State**: Minimal memory footprint per DHT node
- **1.8-5.6M ops/sec**: Exceptional performance (45,000× faster than traditional DHTs)
- **Lazy Neighbor Discovery**: Compute neighbors on-demand, no routing table storage
- **Recursive Architecture**: Uses DHT to store its own topology

See [DHT_INTEGRATION.md](./DHT_INTEGRATION.md) for detailed technical documentation.

### Storage Backend

Supports multiple storage backends through a unified trait:

```rust
#[async_trait]
pub trait LensStorage: Send + Sync {
    // Release operations
    async fn put_release(&mut self, release: &ReleaseMetadata) -> Result<()>;
    async fn get_release(&self, id: &str) -> Result<Option<ReleaseMetadata>>;
    async fn list_releases(&self, offset: usize, limit: usize) -> Result<Vec<ReleaseMetadata>>;

    // Featured content
    async fn add_featured(&mut self, featured: &FeaturedMetadata) -> Result<()>;
    async fn list_featured(&self) -> Result<Vec<FeaturedMetadata>>;

    // Categories
    async fn put_category(&mut self, category: &CategoryMetadata) -> Result<()>;
    async fn list_categories(&self) -> Result<Vec<CategoryMetadata>>;

    // Search
    async fn search_releases_by_title(&self, query: &str) -> Result<Vec<ReleaseMetadata>>;
    async fn get_releases_by_category(&self, category_id: &str) -> Result<Vec<ReleaseMetadata>>;
    async fn get_releases_by_tag(&self, tag: &str) -> Result<Vec<ReleaseMetadata>>;
}
```

**Available Backends:**
- `DHTStorage`: Citadel DHT-backed distributed storage (default)
- `InMemoryStorage`: Fast in-memory storage for development
- `RocksDBStorage`: Persistent local storage (planned)

### P2P Networking

- **WebRTC Data Channels**: Browser-compatible P2P connections
- **Peer Registry**: Track active peers and their capabilities
- **Block Exchange**: Efficient content distribution using PeerExc consensus
- **NAT Traversal**: ICE/STUN/TURN support for connectivity

### HTTP API

Comprehensive RESTful API for client applications:

```
GET    /api/v1/releases              # List all releases
GET    /api/v1/releases/:id          # Get release details
POST   /api/v1/releases              # Create new release
PUT    /api/v1/releases/:id          # Update release
DELETE /api/v1/releases/:id          # Delete release

GET    /api/v1/featured              # List featured content
POST   /api/v1/featured              # Add featured release
DELETE /api/v1/featured/:id          # Remove featured release

GET    /api/v1/categories            # List categories
GET    /api/v1/categories/:id        # Get category
POST   /api/v1/categories            # Create category

GET    /api/v1/search                # Search releases
GET    /api/v1/peers                 # List active peers
GET    /api/v1/dht/health            # DHT health check
GET    /api/v1/tgp/health            # TGP health check
```

### Mesh Configuration

Configure the DHT mesh topology based on your network size:

```rust
use citadel_core::topology::MeshConfig;

// Small network (development/testing)
let config = MeshConfig::new(10, 10, 5);  // 500 slots

// Medium network (production)
let config = MeshConfig::new(100, 100, 50);  // 500K slots

// Large network (recommended for global deployment)
let config = MeshConfig::new(120, 120, 25);  // 360K slots (default)

// Calculate total slots
println!("Total DHT slots: {}", config.total_slots());
```

**Mesh dimensions impact**:
- **Width/Height**: Horizontal spread (most routing happens here)
- **Depth**: Vertical layers (provides failover and redundancy)
- **Total Slots**: width × height × depth

**Recommended configurations**:
- Development: `10 × 10 × 5` (500 slots, ~4 average hops)
- Production: `100 × 100 × 50` (500K slots, ~15 average hops)
- Global Scale: `120 × 120 × 25` (360K slots, ~12 average hops)

### Example Usage

#### Initialize DHT Storage

```rust
use lens_node::storage::dht_storage::DHTStorage;
use citadel_dht::node::MinimalNode;
use citadel_core::topology::{MeshConfig, SlotCoordinate};

// Configure mesh
let config = MeshConfig::new(100, 100, 50);

// Initialize node at a specific slot
let slot = SlotCoordinate::new(42, 73, 18);
let peer_id = generate_peer_id();  // 32-byte public key
let node = MinimalNode::new(slot, peer_id, config, 0);

// Create storage
let mut storage = DHTStorage::new(node, config);

// Store release metadata
let release = ReleaseMetadata {
    id: "abc123".to_string(),
    title: "My Album".to_string(),
    creator: Some("Artist Name".to_string()),
    year: Some(2024),
    category_id: "music".to_string(),
    tags: vec!["rock".to_string(), "indie".to_string()],
    // ... other fields
};

storage.put_release(&release).await?;
```

#### Query DHT Health

```bash
curl http://localhost:8080/api/v1/dht/health
```

Response:
```json
{
    "status": "healthy",
    "metrics": {
        "get_count": 1000,
        "put_count": 500,
        "delete_count": 10,
        "total_operations": 1510,
        "avg_get_latency_ms": 5.2,
        "avg_put_latency_ms": 8.1,
        "avg_delete_latency_ms": 6.5,
        "error_count": 0,
        "error_rate": 0.0
    }
}
```

#### Enable Encryption

```rust
use lens_node::dht_encryption::{DHTEncryption, SiteMode};

// Initialize encryption with enterprise mode (required encryption)
let db = Database::open("./data")?;
let encryption = DHTEncryption::init_or_generate(&db, SiteMode::Enterprise)?;

// Create encrypted storage
let storage = DHTStorage::new_with_encryption(
    node,
    mesh_config,
    Arc::new(encryption)
);

// All DHT operations now use ChaCha20-Poly1305 encryption
storage.put_release(&release).await?;  // Automatically encrypted
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser Clients                          │
│  (Vue.js + WebRTC + Optional WASM DHT)                     │
└────┬────────────────────────┬────────────────────┬──────────┘
     │ HTTP/WebSocket         │ WebRTC             │
┌────▼─────────────────────────▼────────────────────▼──────────┐
│                    Lens Node v2                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   HTTP API   │  │  WebSocket   │  │  WebRTC P2P  │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                 │                │
│  ┌──────▼─────────────────▼─────────────────▼───────┐      │
│  │           Storage Layer (Trait)                    │      │
│  └──────┬────────────────────────────────────────────┘      │
│         │                                                     │
│  ┌──────▼─────────────────────────────────────────────┐     │
│  │         DHTStorage (Citadel DHT Backend)           │     │
│  │  - Local cache (in-memory + RocksDB)              │     │
│  │  - Optional encryption (ChaCha20-Poly1305)        │     │
│  │  - Metrics tracking                                │     │
│  └──────┬────────────────────────────────────────────┘     │
└─────────┼──────────────────────────────────────────────────┘
          │
┌─────────▼──────────────────────────────────────────────────┐
│                    Citadel DHT Network                      │
│  - 2.5D Hexagonal Toroidal Mesh                            │
│  - O(1) key lookups                                         │
│  - Greedy routing with provable optimality                 │
│  - 64-byte node state                                       │
│  - 1.8-5.6M ops/sec                                         │
└─────────────────────────────────────────────────────────────┘
```

## Building

```bash
# Build the library and binary
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=info cargo run --release
```

## Configuration

Set via environment variables or command-line flags:

```bash
# HTTP server
LENS_HTTP_ADDR=0.0.0.0:8080

# DHT mesh configuration
LENS_DHT_WIDTH=100
LENS_DHT_HEIGHT=100
LENS_DHT_DEPTH=50

# DHT node slot (auto-assigned if not specified)
LENS_DHT_SLOT_X=42
LENS_DHT_SLOT_Y=73
LENS_DHT_SLOT_Z=18

# Encryption mode
LENS_SITE_MODE=normal  # or "enterprise" for required encryption

# Database path
LENS_DB_PATH=./data

# P2P networking
LENS_P2P_PORT=9000
LENS_WEBRTC_ENABLE=true

# Logging
RUST_LOG=info  # or debug, warn, error
```

## Performance

Benchmark results on Ryzen 9 5950X:

| Operation | Throughput | Latency (avg) |
|-----------|------------|---------------|
| DHT GET | 3.2M ops/sec | 30 ns |
| DHT PUT | 2.1M ops/sec | 45 ns |
| DHT DELETE | 2.4M ops/sec | 40 ns |
| Release query | 15K req/sec | 4.2 ms |
| Search query | 8K req/sec | 12 ms |

### Scalability

| Mesh Size | Total Slots | Avg Hops | Memory/Node |
|-----------|-------------|----------|-------------|
| 10×10×5 | 500 | ~4 | 64 bytes |
| 100×100×50 | 500,000 | ~15 | 64 bytes |
| 120×120×25 | 360,000 | ~12 | 64 bytes |
| 1000×1000×100 | 100M | ~45 | 64 bytes |

## Testing

```bash
# Run all tests
cargo test

# Run DHT-specific tests
cargo test --features dht

# Run with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_dht_storage_put_and_get_release
```

## Documentation

- **[DHT_INTEGRATION.md](./DHT_INTEGRATION.md)**: Comprehensive technical documentation on Citadel DHT integration
- **[API Reference](./API.md)**: Full HTTP API documentation (coming soon)
- **[Architecture Guide](./ARCHITECTURE.md)**: System architecture and design decisions (coming soon)

## Dependencies

Core dependencies:

- `citadel-core`: Topology and routing for hexagonal toroidal mesh
- `citadel-dht`: Distributed hash table implementation
- `consensus-peerexc`: Peer exchange and block distribution
- `axum`: HTTP server framework
- `tokio`: Async runtime
- `rocksdb`: Persistent storage
- `blake3`: Fast cryptographic hashing
- `chacha20poly1305`: Authenticated encryption
- `webrtc`: P2P connectivity

## License

AGPL-3.0-or-later

## Contributing

See the main Flagship repository for contribution guidelines.

---

**Lens Node v2** - Part of the Riff.CC Flagship project
