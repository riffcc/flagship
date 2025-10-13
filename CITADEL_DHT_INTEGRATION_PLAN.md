# Citadel DHT Integration Plan for Flagship/Lens-Node

**Date:** 2025-10-13
**Status:** Planning
**Goal:** Align Flagship and Lens-Node with Citadel's full recursive DHT architecture

---

## Executive Summary

Lens-Node v0.7.3 currently uses **partial DHT integration** - it queries Citadel's DHT storage for neighbor discovery but still relies on the relay for peer referrals and direct connections. This achieves ~20% mesh connectivity (4.4x improvement over v0.7.2).

**This plan implements the full Citadel recursive DHT architecture** to achieve:
- ✅ **100% mesh connectivity** via lazy-loaded DHT topology
- ✅ **64-byte minimal state** per node (no neighbor caches, no routing tables)
- ✅ **O(1) routing cost** independent of network size
- ✅ **1-message join/leave** (vs 80 messages with broadcast)
- ✅ **DHT-routed peer messaging** (no direct TCP connections needed)

---

## Current State (v0.7.3)

### What Works
- ✅ Citadel DHT storage integrated (shared between relay and orchestrator)
- ✅ SlotOwnership announcements written to DHT by relay
- ✅ Sync orchestrator queries DHT for 8 hexagonal mesh neighbors
- ✅ ~20% mesh connectivity (10 peers/node via SPORE peer exchange)
- ✅ DHT finding 1/8 neighbors (propagation still ongoing)

### What's Missing (Full Recursive DHT)
- ❌ **Lazy neighbor discovery** - Still uses relay peer referrals
- ❌ **DHT-routed messaging** - Still uses direct TCP to peers
- ❌ **DHT-native join/leave** - Still broadcasts (80 messages)
- ❌ **Minimal state** - Still caches neighbor lists (>5 KB vs 64 bytes)
- ❌ **Pure DHT routing** - Still depends on relay for coordination

---

## Architecture Overview

### Citadel's Recursive DHT (from SPEC)

```
┌─────────────────────────────────────────────────────────┐
│                   Citadel Recursive DHT                  │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  DHT STORES ITS OWN TOPOLOGY:                            │
│                                                           │
│  slot_ownership_key(slot) → SlotOwnership {              │
│      peer_id, blind_identities, epoch, heartbeat         │
│  }                                                        │
│                                                           │
│  peer_location_key(peer_id) → SlotCoordinate             │
│                                                           │
│  peer_message_key(peer_id, nonce) → PeerMessage {        │
│      from, to, payload, response_key, signature          │
│  }                                                        │
│                                                           │
│  LAZY NEIGHBOR DISCOVERY:                                │
│    1. Need neighbor in direction D                       │
│    2. Calculate neighbor_slot = my_slot.neighbor(D)      │
│    3. DHT GET slot_ownership_key(neighbor_slot)          │
│    4. Returns peer_id (just-in-time!)                    │
│                                                           │
│  DHT-ROUTED MESSAGING:                                   │
│    1. Want to send message to peer P                     │
│    2. DHT GET peer_location_key(P) → slot               │
│    3. DHT PUT peer_message_key(P, nonce) → message      │
│    4. Message routes through mesh to P's slot            │
│    5. P reads message, writes response                   │
│    6. Response routes back through mesh                  │
│                                                           │
│  MINIMAL STATE:                                          │
│    my_slot: 12 bytes                                     │
│    my_peer_id: 32 bytes                                  │
│    mesh_config: 12 bytes                                 │
│    epoch: 8 bytes                                        │
│    ──────────────────                                    │
│    TOTAL: 64 bytes                                       │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

### Current Lens-Node Architecture (Hybrid)

```
┌─────────────────────────────────────────────────────────┐
│            Lens-Node v0.7.3 (Partial DHT)               │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  Relay (routes messages, sends peer referrals)           │
│    │                                                      │
│    ├─ Writes SlotOwnership to DHT ✅                     │
│    └─ Sends PeerReferral events (SPORE) ⚠️              │
│                                                           │
│  Sync Orchestrator (coordinates sync)                    │
│    │                                                      │
│    ├─ Queries DHT for 8 neighbors ✅                     │
│    ├─ Receives PeerReferral from relay ⚠️                │
│    ├─ Caches neighbor list (~5 KB) ⚠️                    │
│    └─ Direct TCP to known peers ⚠️                       │
│                                                           │
│  P2P Network (direct connections)                        │
│    └─ TCP connections to cached peers ⚠️                 │
│                                                           │
└─────────────────────────────────────────────────────────┘

Legend:
  ✅ = Aligned with Citadel
  ⚠️ = Needs replacement with DHT
```

### Target Architecture (Full Recursive DHT)

```
┌─────────────────────────────────────────────────────────┐
│           Lens-Node v0.8.0 (Full Recursive DHT)         │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  Minimal Node State (64 bytes):                          │
│    my_slot, my_peer_id, mesh_config, epoch               │
│                                                           │
│  DHT Layer (all operations):                             │
│    │                                                      │
│    ├─ Lazy Neighbor Discovery                            │
│    │   └─ DHT GET slot_ownership_key(neighbor_slot)      │
│    │                                                      │
│    ├─ DHT-Routed Messaging                               │
│    │   ├─ DHT GET peer_location_key(target)              │
│    │   ├─ DHT PUT peer_message_key(target, nonce)        │
│    │   └─ DHT GET response_key(my_peer_id, nonce)        │
│    │                                                      │
│    └─ DHT-Native Join/Leave (1 message!)                 │
│        ├─ DHT PUT join_announcement_key(my_slot)         │
│        └─ DHT PUT leave_announcement_key(my_slot)        │
│                                                           │
│  Ephemeral Cache (10s TTL, optional):                    │
│    └─ Recently queried neighbors (avoid re-queries)      │
│                                                           │
│  NO RELAY NEEDED:                                        │
│    - Bootstrap from ANY node                             │
│    - Discover topology through DHT                       │
│    - Route all messages through mesh                     │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

---

## Implementation Plan

### Phase 1: Lazy Neighbor Discovery (Week 1)

**Goal:** Replace relay peer referrals with DHT queries

#### 1.1 Implement `LazyNode` Pattern

```rust
pub struct LazyNode {
    my_slot: SlotCoordinate,
    my_peer_id: PeerID,
    mesh_config: MeshConfig,
    dht_storage: Arc<Mutex<LocalStorage>>,

    // Optional ephemeral cache (10s TTL)
    neighbor_cache: Arc<RwLock<HashMap<Direction, (PeerID, Instant)>>>,
    cache_ttl: Duration,
}

impl LazyNode {
    /// Get neighbor on-demand (lazy load from DHT)
    pub async fn get_neighbor(&mut self, direction: Direction) -> Result<PeerID> {
        // Check ephemeral cache first (10s TTL)
        if let Some((peer_id, cached_at)) = self.neighbor_cache.read().await.get(&direction) {
            if cached_at.elapsed() < self.cache_ttl {
                return Ok(peer_id.clone());
            }
        }

        // Cache miss - query DHT
        let neighbor_slot = self.my_slot.neighbor(direction, &self.mesh_config);
        let ownership_key = slot_ownership_key(neighbor_slot);

        let dht = self.dht_storage.lock().await;
        let ownership_bytes = dht.get(&ownership_key)
            .ok_or_else(|| anyhow!("Neighbor slot {} not found in DHT", neighbor_slot))?;

        let ownership: SlotOwnership = serde_json::from_slice(ownership_bytes)?;

        // Update ephemeral cache
        self.neighbor_cache.write().await.insert(direction, (ownership.peer_id.clone(), Instant::now()));

        Ok(ownership.peer_id)
    }

    /// Get all 8 neighbors (lazy load)
    pub async fn get_all_neighbors(&mut self) -> Result<Vec<PeerID>> {
        let directions = [
            Direction::PlusA, Direction::MinusA,
            Direction::PlusB, Direction::MinusB,
            Direction::PlusC, Direction::MinusC,
            Direction::Up, Direction::Down,
        ];

        let mut neighbors = Vec::new();
        for direction in &directions {
            if let Ok(peer_id) = self.get_neighbor(*direction).await {
                neighbors.push(peer_id);
            }
        }

        Ok(neighbors)
    }
}
```

#### 1.2 Modify Sync Orchestrator to Use LazyNode

**Remove:**
- ❌ Relay peer referral processing
- ❌ Cached neighbor lists in P2pManager
- ❌ SPORE peer exchange logic

**Replace with:**
- ✅ LazyNode instance for on-demand neighbor queries
- ✅ Ephemeral cache (10s TTL) to reduce DHT GETs
- ✅ Pure DHT-based neighbor discovery

```rust
pub struct SyncOrchestrator {
    lazy_node: LazyNode,
    db: Database,
    sync_interval: Duration,
    // ... (removed: p2p_manager neighbor cache)
}

async fn build_wantlist(&self) -> Result<WantList> {
    let mut wantlist = WantList::new(1);

    // Add local blocks
    let local_blocks = self.get_local_blocks().await?;
    for block in local_blocks {
        wantlist.add_have_block(block.id);
    }

    // DHT-based lazy neighbor discovery (NO relay referrals!)
    let neighbors = self.lazy_node.get_all_neighbors().await?;
    for neighbor in neighbors {
        wantlist.add_known_peer(neighbor, 255);
    }

    Ok(wantlist)
}
```

---

### Phase 2: DHT-Routed Messaging (Week 2)

**Goal:** Replace direct TCP connections with DHT-routed messages

#### 2.1 Implement `send_to_peer` via DHT

```rust
impl LazyNode {
    /// Send message to peer through DHT routing
    pub async fn send_to_peer(&mut self, target: PeerID, msg: Vec<u8>) -> Result<Vec<u8>> {
        // Step 1: Find which slot hosts target peer (query DHT!)
        let location_key = peer_location_key(target);
        let dht = self.dht_storage.lock().await;

        let location_bytes = dht.get(&location_key)
            .ok_or_else(|| anyhow!("Peer {} location not found in DHT", target))?;
        let location: SlotOwnership = serde_json::from_slice(location_bytes)?;

        // Step 2: Create message with response address
        let nonce = rand::random();
        let response_key = peer_message_key(self.my_peer_id, nonce);

        let message = PeerMessage {
            from: self.my_peer_id,
            to: target,
            nonce,
            payload: msg,
            response_key: response_key.clone(),
            signature: self.sign(&msg)?,
        };

        // Step 3: PUT message at target's message key (routes through DHT!)
        let message_key = peer_message_key(target, nonce);
        dht.put(&message_key, &message.to_bytes()?)?;

        drop(dht); // Release lock while waiting for response

        // Step 4: Poll for response at our response key
        let timeout = Duration::from_secs(5);
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!("Timeout waiting for response from {}", target));
            }

            let dht = self.dht_storage.lock().await;
            if let Some(response_bytes) = dht.get(&response_key) {
                let response: PeerMessage = serde_json::from_slice(response_bytes)?;
                return Ok(response.payload);
            }
            drop(dht);

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
```

#### 2.2 Implement Message Polling Loop

```rust
impl LazyNode {
    /// Background task: poll for incoming messages
    pub async fn message_polling_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            interval.tick().await;

            // Check for messages addressed to us
            for nonce in 0..10 { // Check last 10 nonces
                let message_key = peer_message_key(self.my_peer_id, nonce);

                let dht = self.dht_storage.lock().await;
                if let Some(message_bytes) = dht.get(&message_key) {
                    let message: PeerMessage = serde_json::from_slice(message_bytes).unwrap();

                    // Process message
                    let response_payload = self.handle_message(message.payload).await;

                    // Write response to sender's response_key
                    let response = PeerMessage {
                        from: self.my_peer_id,
                        to: message.from,
                        nonce: message.nonce,
                        payload: response_payload,
                        response_key: Vec::new(), // No response to response
                        signature: self.sign(&response_payload).unwrap(),
                    };

                    dht.put(&message.response_key, &response.to_bytes().unwrap()).unwrap();

                    // Delete processed message
                    dht.delete(&message_key).unwrap();
                }
                drop(dht);
            }
        }
    }
}
```

---

### Phase 3: DHT-Native Join/Leave (Week 3)

**Goal:** Replace broadcast announcements with 1-message DHT operations

#### 3.1 DHT-Native Join

```rust
impl LazyNode {
    /// Announce join via single DHT PUT
    pub async fn announce_join(&mut self) -> Result<()> {
        let key = join_announcement_key(self.my_slot);

        let announcement = JoinAnnouncement {
            peer_id: self.my_peer_id,
            slot: self.my_slot,
            pow_nonce: self.pow_nonce,
            epoch: self.current_epoch,
            timestamp: now(),
            signature: self.sign(&self.my_slot.to_bytes())?,
        };

        // Single DHT PUT (routed through mesh to target slot)
        let dht = self.dht_storage.lock().await;
        dht.put(&key, &announcement.to_bytes()?)?;

        info!("📢 Announced join via DHT (1 message)");
        Ok(())
    }
}
```

#### 3.2 DHT-Native Leave

```rust
impl LazyNode {
    /// Announce leave via single DHT PUT (tombstone)
    pub async fn announce_leave(&mut self) -> Result<()> {
        let key = leave_announcement_key(self.my_slot);

        let announcement = LeaveAnnouncement {
            peer_id: self.my_peer_id,
            slot: self.my_slot,
            epoch: self.current_epoch,
            hosted_blind_identities: self.blind_identities.clone(),
            timestamp: now(),
            signature: self.sign(&self.my_slot.to_bytes())?,
        };

        // Single DHT PUT (triggers 5-min grace period)
        let dht = self.dht_storage.lock().await;
        dht.put(&key, &announcement.to_bytes()?)?;

        info!("📢 Announced leave via DHT (1 message)");
        Ok(())
    }
}
```

#### 3.3 Neighbor Discovery via DHT Announcements

```rust
impl LazyNode {
    /// Discover new neighbors via periodic DHT checks
    pub async fn discover_new_neighbors(&mut self) -> Vec<PeerID> {
        let mut new_neighbors = Vec::new();

        for direction in &self.neighbor_directions {
            let neighbor_slot = self.my_slot.neighbor(*direction, &self.mesh_config);
            let key = join_announcement_key(neighbor_slot);

            let dht = self.dht_storage.lock().await;
            if let Some(announcement_bytes) = dht.get(&key) {
                let announcement: JoinAnnouncement = serde_json::from_slice(announcement_bytes).unwrap();

                // Verify PoW, epoch, signature
                if self.verify_join_announcement(&announcement) {
                    new_neighbors.push(announcement.peer_id);
                    info!("🆕 Discovered new neighbor {} at slot {}", announcement.peer_id, neighbor_slot);
                }
            }
            drop(dht);
        }

        new_neighbors
    }
}
```

---

### Phase 4: Minimal State (Week 4)

**Goal:** Reduce to 64-byte state per node

#### 4.1 Remove All Caches and Routing Tables

**Before (v0.7.3):**
```rust
pub struct SyncOrchestrator {
    network: Arc<P2pNetwork>,           // ~1 KB
    p2p_manager: Arc<P2pManager>,       // ~3 KB (routing tables)
    db: Database,                        // ~200 MB (RocksDB)
    my_peer_id: Arc<RwLock<Option<String>>>,  // 64 bytes
    // ... lots of state
}
```

**After (v0.8.0):**
```rust
pub struct MinimalNode {
    my_slot: SlotCoordinate,            // 12 bytes
    my_peer_id: PeerID,                 // 32 bytes
    mesh_config: MeshConfig,            // 12 bytes (width, height, depth)
    epoch: u64,                         // 8 bytes

    dht_storage: Arc<Mutex<LocalStorage>>,  // Shared (200 MB)
    db: Database,                       // Shared (RocksDB)

    // Optional ephemeral cache (10s TTL)
    neighbor_cache: Arc<RwLock<HashMap<Direction, (PeerID, Instant)>>>,  // <1 KB
}
```

**Total per-node state: ~64 bytes + optional 1 KB ephemeral cache**

#### 4.2 Bootstrap from ANY Node

```rust
impl MinimalNode {
    /// Bootstrap from ANY node in the network
    pub async fn bootstrap_from_dht(&mut self, any_node: PeerID) -> Result<()> {
        info!("🚀 Bootstrapping from {}", any_node);

        // Step 1: Send bootstrap request
        let my_slot = self.compute_my_slot()?;
        let bootstrap_msg = BootstrapRequest { requested_slot: my_slot };

        let response = self.send_to_peer(any_node, bootstrap_msg.to_bytes()?).await?;
        let bootstrap_info: BootstrapResponse = bincode::deserialize(&response)?;

        info!("✅ Bootstrapped from DHT: {} neighbors available", bootstrap_info.neighbor_count);

        // Step 2: Announce our join
        self.announce_join().await?;

        // Step 3: Start message polling loop
        tokio::spawn(async move {
            self.message_polling_loop().await;
        });

        Ok(())
    }
}
```

---

## Performance Metrics

### Expected Improvements

| Metric | v0.7.3 (Partial DHT) | v0.8.0 (Full Recursive) | Improvement |
|--------|---------------------|-------------------------|-------------|
| **Mesh Connectivity** | ~20% (10 peers/node) | **100% (8 neighbors/node)** | **5× improvement** |
| **State per Node** | ~5 KB (caches + routing) | **64 bytes** | **78× reduction** |
| **Join/Leave Cost** | 80 messages (broadcast) | **1 message (DHT PUT)** | **80× reduction** |
| **Join/Leave Bandwidth** | ~50 KB | **~1 KB** | **50× reduction** |
| **Neighbor Discovery** | Relay referrals (10 msgs) | **1 DHT GET** | **10× reduction** |
| **Message Routing** | Direct TCP (peer discovery) | **DHT routing (O(1))** | **Constant cost** |

### Benchmark Targets

**Network Sizes:**
- 50 nodes: 100% connectivity, <1s neighbor discovery
- 200 nodes: 100% connectivity, <2s neighbor discovery
- 1000 nodes: 100% connectivity, <5s neighbor discovery
- 10k nodes: 100% connectivity, <10s neighbor discovery

**Memory:**
- Physical node: <10 MB (minimal state + ephemeral cache)
- Virtual node: <5 MB (lazy-loaded DHT storage)
- 100 virtual nodes: <500 MB total

---

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_lazy_neighbor_discovery() {
    let lazy_node = LazyNode::new(/* ... */);

    // Query all 8 neighbors
    let neighbors = lazy_node.get_all_neighbors().await.unwrap();
    assert_eq!(neighbors.len(), 8);

    // Verify cache works (no re-query)
    let neighbors_cached = lazy_node.get_all_neighbors().await.unwrap();
    assert_eq!(neighbors, neighbors_cached);
}

#[tokio::test]
async fn test_dht_routed_messaging() {
    let node_a = LazyNode::new(/* ... */);
    let node_b = LazyNode::new(/* ... */);

    // Send message through DHT
    let response = node_a.send_to_peer(node_b.my_peer_id, b"Hello".to_vec()).await.unwrap();
    assert_eq!(response, b"World");
}

#[tokio::test]
async fn test_dht_native_join() {
    let node = LazyNode::new(/* ... */);

    // Announce join (1 message)
    node.announce_join().await.unwrap();

    // Verify announcement in DHT
    let key = join_announcement_key(node.my_slot);
    let dht = node.dht_storage.lock().await;
    assert!(dht.get(&key).is_some());
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_50_node_full_connectivity() {
    // Deploy 50 nodes with full recursive DHT
    let nodes = deploy_50_node_cluster().await;

    // Wait for DHT to stabilize (2-3 epochs = 20-30s)
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Verify 100% connectivity
    for node in &nodes {
        let neighbors = node.get_all_neighbors().await.unwrap();
        assert_eq!(neighbors.len(), 8, "Node {} should have 8 neighbors", node.my_peer_id);
    }

    // Verify minimal state
    for node in &nodes {
        assert!(node.state_size() <= 64 + 1024); // 64 bytes + 1 KB ephemeral cache
    }
}
```

---

## Migration Path

### v0.7.3 → v0.8.0 (Gradual Rollout)

**Phase 1: Hybrid Mode (v0.7.4)**
- Keep relay for fallback
- Add lazy neighbor discovery (DHT + relay)
- Measure performance
- **Goal:** Verify DHT lazy loading works

**Phase 2: DHT-Routed Messaging (v0.7.5)**
- Add DHT-routed peer messaging
- Keep relay for failover
- Measure latency and throughput
- **Goal:** Verify DHT messaging works

**Phase 3: Full Recursive (v0.8.0)**
- Remove relay dependency entirely
- Pure DHT lazy loading + routing
- Minimal 64-byte state
- **Goal:** Production-ready full recursive DHT

---

## Success Criteria

**v0.8.0 is complete when:**
- ✅ 100% mesh connectivity in 50-node cluster
- ✅ <64 bytes + 1 KB ephemeral cache per node
- ✅ 1-message join/leave (DHT-native announcements)
- ✅ DHT-routed messaging works end-to-end
- ✅ No relay dependency (bootstrap from ANY node)
- ✅ Lazy neighbor discovery with <1s latency
- ✅ All integration tests pass

---

## Related Documentation

- [Citadel DHT SPEC](/opt/castle/workspace/citadel/2025-10-12-Citadel-DHT-SPEC.md)
- [Flagship Roadmap](/opt/castle/workspace/flagship/ROADMAP_v0.6.0.md)
- [Citadel Internet Scale Architecture](/opt/castle/workspace/citadel/INTERNET_SCALE_ARCHITECTURE.md)

---

**Next Steps:**
1. Implement `LazyNode` pattern (Week 1)
2. Replace relay peer referrals with DHT queries
3. Add DHT-routed messaging
4. Test 50-node cluster for 100% connectivity
5. Push v0.8.0 with full recursive DHT

---

**Last Updated:** 2025-10-13
**Status:** Planning → Implementation
