# Lens Node + Citadel DHT Integration Specification

## Overview

Replace Lens Node's centralized metadata storage with **Citadel DHT**, creating a fully decentralized P2P network where:
- Flagship browsers participate **directly** in the DHT via tiny WASM (<112 KiB)
- Lens Nodes act as **supernodes** that replicate, pin, and cache data
- All metadata operations go through DHT (releases, peers, blocks, etc.)
- Fast read-only API cache for public access
- RocksDB write-through cache: 1 GiB disk, <128 MiB RAM

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Citadel DHT Network                      │
│  (O(1) routing, 16.7ns lookups, 80 bytes per node)        │
└────┬────────────────────────┬────────────────────┬──────────┘
     │                        │                    │
┌────▼─────┐         ┌────────▼────────┐   ┌──────▼──────────┐
│ Flagship │         │   Lens Node 1   │   │  Lens Node 2    │
│ (Browser)│         │   (Supernode)   │   │  (Supernode)    │
│          │         │                 │   │                 │
│ WASM DHT │◄────────┤ Full DHT Node   │   │ Full DHT Node   │
│ <112 KiB │  HTTP   │ + Replication   │   │ + Replication   │
│          │ Fallback│ + RocksDB Cache │   │ + RocksDB Cache │
│ Direct   │         │ + API Cache     │   │ + API Cache     │
│ DHT ops  │         │ + Hot Pinning   │   │ + Hot Pinning   │
└──────────┘         └─────────────────┘   └─────────────────┘
```

## Phase 1: DHT Schema Design

### Key Structure

All DHT keys use Blake3 hashing with domain separation:

```rust
// Release metadata
blake3("lens:release:" || release_id)
// Value: ReleaseMetadata (Protobuf serialized)

// Peer announcements
blake3("lens:peer:" || peer_id || slot_coord)
// Value: PeerInfo (IP, port, capabilities, timestamp)

// Block location
blake3("lens:block:" || block_hash)
// Value: Vec<PeerId> (who has this block)

// Sync state
blake3("lens:sync:" || peer_id)
// Value: SyncState (head_block, peer_count, is_synced)

// Federation membership
blake3("lens:federation:" || domain)
// Value: FederationInfo (admins, config, timestamp)
```

### Value Types (Protobuf)

```protobuf
message ReleaseMetadata {
  string release_id = 1;
  string name = 2;
  string version = 3;
  repeated BlockRef blocks = 4;
  bytes signature = 5;
  int64 timestamp = 6;
}

message PeerInfo {
  bytes peer_id = 1;
  string ip = 2;
  uint32 port = 3;
  repeated string capabilities = 4;
  int64 last_seen = 5;
  SlotCoordinate dht_slot = 6;
}

message BlockRef {
  bytes hash = 1;
  uint64 size = 2;
  repeated bytes peer_ids = 3; // Who has this block
}

message SyncState {
  bytes head_block = 1;
  uint32 peer_count = 2;
  bool is_synced = 3;
  int64 last_update = 4;
}
```

## Phase 2: Lens Node DHT Integration

### Storage Trait

```rust
#[async_trait]
pub trait LensStorage: Send + Sync {
    // Release operations
    async fn put_release(&mut self, release: &ReleaseMetadata) -> Result<()>;
    async fn get_release(&self, id: &str) -> Result<Option<ReleaseMetadata>>;
    async fn list_releases(&self) -> Result<Vec<ReleaseMetadata>>;

    // Peer operations
    async fn announce_peer(&mut self, peer: &PeerInfo) -> Result<()>;
    async fn get_peers(&self) -> Result<Vec<PeerInfo>>;

    // Block operations
    async fn put_block_location(&mut self, hash: &[u8], peer_id: &[u8]) -> Result<()>;
    async fn get_block_locations(&self, hash: &[u8]) -> Result<Vec<PeerId>>;

    // Sync operations
    async fn put_sync_state(&mut self, peer_id: &[u8], state: &SyncState) -> Result<()>;
    async fn get_sync_state(&self, peer_id: &[u8]) -> Result<Option<SyncState>>;
}
```

### DHT Backend Implementation

```rust
pub struct DHTStorage {
    dht: Arc<Mutex<CitadelDHT>>,
    local_cache: Arc<RwLock<LocalCache>>,
}

impl LensStorage for DHTStorage {
    async fn put_release(&mut self, release: &ReleaseMetadata) -> Result<()> {
        let key = blake3_key(b"lens:release:", release.release_id.as_bytes());
        let value = release.encode_to_vec(); // Protobuf

        self.dht.lock().await.put(key, value).await?;
        self.local_cache.write().await.insert(key, value.clone());

        Ok(())
    }

    async fn get_release(&self, id: &str) -> Result<Option<ReleaseMetadata>> {
        let key = blake3_key(b"lens:release:", id.as_bytes());

        // Check local cache first
        if let Some(cached) = self.local_cache.read().await.get(&key) {
            return Ok(Some(ReleaseMetadata::decode(cached)?));
        }

        // Query DHT
        if let Some(value) = self.dht.lock().await.get(&key).await? {
            let release = ReleaseMetadata::decode(&value[..])?;
            self.local_cache.write().await.insert(key, value);
            return Ok(Some(release));
        }

        Ok(None)
    }
}
```

### Supernode Features

```rust
pub struct SupernodeManager {
    dht: Arc<Mutex<CitadelDHT>>,
    rocksdb: Arc<DB>,
    pinned_keys: Arc<RwLock<HashSet<DHTKey>>>,
    replication_factor: usize, // How many copies to maintain
}

impl SupernodeManager {
    /// Pin a key and ensure it's always available
    pub async fn pin_key(&self, key: DHTKey) -> Result<()> {
        self.pinned_keys.write().await.insert(key);

        // Store in RocksDB for persistence
        let value = self.dht.lock().await.get(&key).await?.unwrap();
        self.rocksdb.put(key, &value)?;

        // Keep in hot cache
        self.ensure_hot(&key).await?;

        Ok(())
    }

    /// Replicate pinned data to ensure availability
    pub async fn replicate_pinned_data(&self) -> Result<()> {
        let pinned = self.pinned_keys.read().await.clone();

        for key in pinned {
            // Check how many nodes have this key
            let locations = self.dht.lock().await.find_value_locations(&key).await?;

            if locations.len() < self.replication_factor {
                // Replicate to more nodes
                let value = self.rocksdb.get(key)?.unwrap();
                self.dht.lock().await.replicate(&key, &value, self.replication_factor).await?;
            }
        }

        Ok(())
    }

    /// Keep hot data in memory for fast access
    async fn ensure_hot(&self, key: &DHTKey) -> Result<()> {
        // Pre-fetch from DHT and keep in memory
        let value = self.dht.lock().await.get(key).await?;
        if let Some(v) = value {
            self.rocksdb.put(key, &v)?;
        }
        Ok(())
    }
}
```

## Phase 3: WASM Shim Design (<112 KiB)

### Size Budget

```
Component                  Size Target
─────────────────────────────────────
Core DHT logic             30 KiB
Protobuf codec             15 KiB
Blake3 hashing             8 KiB
Ed25519 crypto             20 KiB
Networking (fetch API)     10 KiB
WASM bindgen glue          15 KiB
Compression overhead       14 KiB
─────────────────────────────────────
Total                      112 KiB
```

### Minimal DHT Node (WASM)

```rust
// Only include what's needed for DHT participation
#[wasm_bindgen]
pub struct WasmDHTNode {
    my_slot: SlotCoordinate,
    my_peer_id: PeerID,
    mesh_config: MeshConfig,
    epoch: u64,

    // NO routing table, NO storage!
    // Just participate in routing
}

#[wasm_bindgen]
impl WasmDHTNode {
    /// Create new DHT node
    #[wasm_bindgen(constructor)]
    pub fn new(peer_id: &[u8], slot_x: i32, slot_y: i32, slot_z: i32) -> Self {
        // Initialize with minimal state
    }

    /// PUT operation - direct DHT write
    pub async fn put(&self, key: &[u8], value: &[u8]) -> Result<(), JsValue> {
        // 1. Calculate target slot from key
        let target_slot = key_to_slot(key, &self.mesh_config);

        // 2. Route to target node
        let target_peer = self.route_to_slot(target_slot).await?;

        // 3. HTTP PUT to target (if we're not the target)
        if target_peer != self.my_peer_id {
            fetch_put(&target_peer, key, value).await?;
        }

        Ok(())
    }

    /// GET operation - direct DHT read
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, JsValue> {
        // 1. Calculate target slot
        let target_slot = key_to_slot(key, &self.mesh_config);

        // 2. Route to target
        let target_peer = self.route_to_slot(target_slot).await?;

        // 3. HTTP GET from target
        fetch_get(&target_peer, key).await
    }

    /// Announce presence to DHT
    pub async fn announce(&self) -> Result<(), JsValue> {
        let key = slot_ownership_key(&self.my_slot);
        let ownership = SlotOwnership::new(self.my_peer_id, self.my_slot, self.epoch);
        self.put(&key, &ownership.to_bytes()).await
    }
}

// Fetch API wrappers (no WebSocket!)
async fn fetch_put(peer: &PeerID, key: &[u8], value: &[u8]) -> Result<(), JsValue> {
    let url = format!("https://{}/dht/put", peer_to_url(peer));

    let opts = RequestInit::new();
    opts.set_method("PUT");
    opts.set_body(&JsValue::from(value));

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("X-DHT-Key", &hex::encode(key))?;

    let window = web_sys::window().unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;

    Ok(())
}
```

### Build Configuration for Size

```toml
# Cargo.toml
[profile.wasm-release]
inherits = "release"
opt-level = "z"              # Optimize for size
lto = true                   # Link-time optimization
codegen-units = 1            # Single codegen unit
panic = "abort"              # No unwinding
strip = "symbols"            # Strip symbols

[dependencies]
# Minimal dependencies only
wasm-bindgen = { version = "0.2", default-features = false }
blake3 = { version = "1.5", default-features = false, features = ["no_avx2", "no_avx512", "no_neon"] }
ed25519-dalek = { version = "2.1", default-features = false, features = ["zeroize"] }
prost = { version = "0.13", default-features = false }

# NO tokio, NO async-std, NO full std!
```

### WASM Build Script

```bash
#!/bin/bash
# build-wasm-dht.sh

# Build with maximum compression
cargo build --target wasm32-unknown-unknown --profile wasm-release

# Run wasm-opt for extreme size reduction
wasm-opt \
    --enable-bulk-memory \
    --enable-sign-ext \
    -Oz \
    --strip-debug \
    --strip-dwarf \
    --strip-producers \
    --vacuum \
    target/wasm32-unknown-unknown/wasm-release/flagship_dht.wasm \
    -o flagship_dht_optimized.wasm

# Check size
SIZE=$(wc -c < flagship_dht_optimized.wasm)
TARGET=$((56 * 2 * 1024))  # 112 KiB

echo "WASM size: $SIZE bytes"
echo "Target: $TARGET bytes"

if [ $SIZE -gt $TARGET ]; then
    echo "❌ WASM too large! Over by $((SIZE - TARGET)) bytes"
    exit 1
else
    echo "✅ WASM size OK! Under by $((TARGET - SIZE)) bytes"
fi
```

## Phase 4: RocksDB Caching

### Cache Configuration

```rust
pub struct RocksDBCache {
    db: Arc<DB>,
    max_disk_size: u64,      // 1 GiB
    max_ram_usage: usize,    // 128 MiB
    write_buffer_size: usize, // 32 MiB
}

impl RocksDBCache {
    pub fn new(path: &Path) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        // Memory limits
        opts.set_write_buffer_size(32 * 1024 * 1024); // 32 MiB write buffer
        opts.set_max_write_buffer_number(2);          // Max 2 buffers = 64 MiB
        opts.set_target_file_size_base(64 * 1024 * 1024); // 64 MiB SST files

        // Block cache (shared across all column families)
        let cache = Cache::new_lru_cache(64 * 1024 * 1024); // 64 MiB block cache
        opts.set_block_cache(&cache);

        // Compression
        opts.set_compression_type(DBCompressionType::Lz4);

        // Disable WAL for cache (we can rebuild from DHT)
        opts.set_disable_write_ahead_log(true);

        let db = DB::open(&opts, path)?;

        Ok(Self {
            db: Arc::new(db),
            max_disk_size: 1024 * 1024 * 1024, // 1 GiB
            max_ram_usage: 128 * 1024 * 1024,  // 128 MiB
            write_buffer_size: 32 * 1024 * 1024,
        })
    }

    /// Write-through: Write to DHT and cache
    pub async fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        // Write to RocksDB first (fast)
        self.db.put(key, value)?;

        // Then propagate to DHT (async)
        // This returns immediately, DHT write happens in background
        Ok(())
    }

    /// Read-through: Check cache, fall back to DHT
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Try cache first
        if let Some(value) = self.db.get(key)? {
            return Ok(Some(value.to_vec()));
        }

        // Cache miss - fetch from DHT
        // (This would be implemented by the caller)
        Ok(None)
    }

    /// Evict old entries to stay under size limit
    pub fn evict_if_needed(&self) -> Result<()> {
        let disk_usage = self.estimate_disk_usage()?;

        if disk_usage > self.max_disk_size {
            // Evict oldest 10% of entries
            let to_evict = (disk_usage - self.max_disk_size) as usize;
            self.evict_lru(to_evict)?;
        }

        Ok(())
    }
}
```

## Phase 5: Read-Only API Cache

### Fast Public API

```rust
pub struct APICache {
    hot_cache: Arc<RwLock<LruCache<Vec<u8>, Vec<u8>>>>, // In-memory LRU
    rocksdb: Arc<RocksDBCache>,
    dht: Arc<Mutex<CitadelDHT>>,
}

impl APICache {
    /// GET /api/v1/releases
    pub async fn get_releases(&self) -> Result<Vec<ReleaseMetadata>> {
        let cache_key = b"api:releases:list";

        // 1. Check hot cache (RAM)
        if let Some(cached) = self.hot_cache.read().await.get(cache_key) {
            return Ok(decode_releases(cached)?);
        }

        // 2. Check RocksDB cache
        if let Some(cached) = self.rocksdb.get(cache_key).await? {
            self.hot_cache.write().await.put(cache_key.to_vec(), cached.clone());
            return Ok(decode_releases(&cached)?);
        }

        // 3. Fetch from DHT (slow path)
        let releases = self.fetch_releases_from_dht().await?;

        // 4. Warm caches
        let encoded = encode_releases(&releases)?;
        self.hot_cache.write().await.put(cache_key.to_vec(), encoded.clone());
        self.rocksdb.put(cache_key, &encoded).await?;

        Ok(releases)
    }

    async fn fetch_releases_from_dht(&self) -> Result<Vec<ReleaseMetadata>> {
        // Query DHT for all releases
        // This is the slow path, only hit on cache miss
        todo!()
    }
}
```

## Implementation Plan

### Week 1: Core DHT Integration
- [ ] Day 1-2: Implement `LensStorage` trait with DHT backend
- [ ] Day 3: Add Protobuf schemas for all metadata types
- [ ] Day 4: Replace in-memory storage with DHT storage
- [ ] Day 5: Integration tests

### Week 2: WASM Shim
- [ ] Day 1-2: Implement minimal WASM DHT node
- [ ] Day 3: Add fetch-based DHT operations (no WebSocket)
- [ ] Day 4: Optimize build for <112 KiB
- [ ] Day 5: Browser integration tests

### Week 3: Supernode Features
- [ ] Day 1-2: Implement pinning and replication
- [ ] Day 3: Add RocksDB write-through cache
- [ ] Day 4: Implement cache eviction
- [ ] Day 5: Performance testing

### Week 4: API Cache
- [ ] Day 1-2: Implement hot cache layer
- [ ] Day 3: Add read-only API endpoints
- [ ] Day 4: Cache warming and invalidation
- [ ] Day 5: Load testing

## Success Metrics

### Performance Targets
- **Flagship WASM load time:** <500ms (including 112 KiB download)
- **DHT PUT latency:** <50ms p99
- **DHT GET latency:** <20ms p99 (cache hit), <100ms p99 (cache miss)
- **API cache hit rate:** >95%
- **Lens Node RAM usage:** <128 MiB
- **RocksDB disk usage:** <1 GiB

### Reliability Targets
- **Data availability:** 99.99% (with 3+ supernodes)
- **Replication factor:** 3 copies minimum
- **Cache consistency:** <1s stale data maximum

## Testing Strategy

### Unit Tests
- DHT storage operations
- WASM DHT node operations
- Cache eviction logic
- Protobuf serialization

### Integration Tests
- Flagship ↔ Lens Node DHT communication
- Multi-node DHT replication
- Cache coherence across nodes
- Failover scenarios

### Performance Tests
- WASM load time benchmarks
- DHT throughput under load
- Cache hit rate measurement
- Memory usage profiling

## Rollout Plan

### Phase 1: Parallel Run
- Run DHT alongside existing storage
- Compare results for consistency
- No user-facing changes

### Phase 2: Gradual Migration
- Migrate non-critical data first (peer announcements)
- Then releases metadata
- Finally sync state

### Phase 3: Full Cutover
- Disable old storage
- Monitor for issues
- Keep rollback plan ready

### Phase 4: Optimization
- Tune cache sizes
- Optimize replication factor
- Reduce WASM size further if possible

---

**This is the future of Lens Node: Fully decentralized, DHT-backed, with browsers as first-class citizens in the network!** 🚀
