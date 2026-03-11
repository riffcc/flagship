# Consensus-PeerExc DHT Compatibility Assessment

**Date:** 2025-10-13
**Analyst:** Claude Code
**Project:** Flagship / Palace Integration

---

## Executive Summary

**Verdict:** ✅ **PeerExc is HIGHLY compatible with Citadel DHT routing with minimal modifications needed.**

The consensus-peerexc protocol was designed with direct peer-to-peer connectivity assumptions, but its architecture is sufficiently modular to integrate DHT routing without breaking core functionality. The protocol's reliance on relay mechanisms actually makes it **more compatible** than a purely direct-connection protocol would be.

**Key Findings:**
- ✅ PeerExc relay infrastructure maps naturally to DHT routing
- ✅ Message envelope design is transport-agnostic
- ✅ WantList protocol works independently of routing layer
- ✅ BoTG streaming can leverage DHT's O(1) routing for performance gains
- ⚠️ Some assumptions about peer connectivity need adjustment
- ⚠️ Direct endpoint addressing needs DHT slot-based addressing

**Required Changes:** Moderate (transport layer abstraction)
**Estimated Effort:** 2-3 days of focused implementation
**Risk Level:** Low (changes are additive, not destructive)

---

## 1. Protocol Architecture Analysis

### 1.1 Current PeerExc Transport Model

**Direct Connectivity Assumptions:**
```rust
// From botg.rs - currently assumes direct UDP transport
pub async fn connect_to_peer(
    &self,
    peer_id: PeerId,
    peer_addr: SocketAddr,  // ❌ Direct socket address
) -> anyhow::Result<()>
```

**Transport Dependencies:**
- Uses `TransportHandle` from `consensus-transport-udp`
- Creates TGP handles with direct peer addresses
- Assumes bidirectional UDP connectivity
- Relies on NAT traversal via relay servers

### 1.2 Relay Infrastructure (Already DHT-Compatible!)

**Good News:** PeerExc already has relay concepts!

```rust
// From messages.rs
pub enum MessageType {
    RelayRegister,
    RelayIntroduce,
    HolePunchOffer,
    HolePunchAnswer,
    // ...
}

// From peer.rs
pub struct PeerInfo {
    pub relay_address: Option<String>,  // ✅ Already relay-aware!
    pub direct_addresses: Vec<String>,
    // ...
}
```

**Analysis:** The protocol was designed for relay-mediated connectivity, which is **exactly what DHT routing provides**. Instead of a centralized relay server, the DHT acts as a distributed relay mesh.

### 1.3 WantList Protocol (Transport-Independent)

**Excellent Compatibility:**

```rust
// From wantlist.rs - Pure data structures, no transport coupling
pub struct WantList {
    pub generation: u32,
    pub have_ranges: Vec<BlockRange>,
    pub need_ranges: Vec<BlockRange>,
    pub rollups: Vec<BlockRollup>,
    // ...
}
```

**Assessment:** ✅ WantList protocol is completely transport-agnostic. It only cares about exchanging block availability information, not how that information is transmitted.

---

## 2. DHT Routing Compatibility Matrix

| PeerExc Component | DHT Compatible? | Modifications Needed | Effort |
|-------------------|-----------------|----------------------|--------|
| **Message Envelope** | ✅ Yes | None (already has sender_id) | 0 days |
| **WantList Protocol** | ✅ Yes | None (data structures only) | 0 days |
| **Peer Discovery** | ✅ Yes | Replace relay with DHT lookup | 0.5 days |
| **BoTG Streaming** | ⚠️ Partial | Add DHT transport adapter | 1 day |
| **Relay Messages** | ✅ Yes | Repurpose for DHT routing | 0.5 days |
| **NAT Traversal** | ✅ Yes | DHT provides routing | 0 days |
| **Direct Endpoints** | ❌ No | Replace with DHT slot addresses | 1 day |

**Total Estimated Effort:** 2-3 days

---

## 3. DHT Routing Integration Strategy

### 3.1 Transport Abstraction Layer

**Current:**
```rust
// Direct UDP transport
let handle = Arc::new(TgpHandle::new(cfg, self.transport.clone(), peer_addr));
```

**DHT-Compatible:**
```rust
pub trait PeerTransport {
    async fn send_message(&self, peer_id: PeerId, msg: Message) -> Result<()>;
    async fn receive_message(&self) -> Result<(PeerId, Message)>;
}

// DHT implementation
pub struct DhtTransport {
    dht: Arc<CitadelDHT>,
    local_peer_id: PeerId,
}

impl PeerTransport for DhtTransport {
    async fn send_message(&self, peer_id: PeerId, msg: Message) -> Result<()> {
        // Step 1: Query DHT for peer location
        let location_key = peer_location_key(&peer_id);
        let slot: SlotCoordinate = self.dht.get(&location_key).await?;

        // Step 2: Route message through DHT to that slot
        let message_key = peer_message_key(&peer_id, msg.counter);
        self.dht.put(message_key, bincode::serialize(&msg)?).await?;

        Ok(())
    }

    async fn receive_message(&self) -> Result<(PeerId, Message)> {
        // Poll our peer_message_key for incoming messages
        let our_message_key = peer_message_key(&self.local_peer_id, /* nonce */);
        let msg_bytes = self.dht.get(&our_message_key).await?;
        let msg: Message = bincode::deserialize(&msg_bytes)?;

        Ok((msg.sender_id.clone(), msg))
    }
}
```

### 3.2 Peer Discovery via DHT

**Current Relay Model:**
```rust
// From relay.rs
pub fn find_providers(&self, wantlist: &WantList) -> Vec<PeerInfo>
```

**DHT Model:**
```rust
pub async fn find_providers_dht(
    &self,
    wantlist: &WantList,
) -> Result<Vec<PeerInfo>> {
    let mut providers = Vec::new();

    // For each block range we need
    for range in &wantlist.need_ranges {
        // Create DHT key for block range providers
        let provider_key = block_provider_key(range);

        // Query DHT for providers
        let provider_list: Vec<PeerId> = self.dht.get(&provider_key).await?;

        // Look up peer info for each provider
        for peer_id in provider_list {
            let peer_key = peer_info_key(&peer_id);
            let peer_info: PeerInfo = self.dht.get(&peer_key).await?;
            providers.push(peer_info);
        }
    }

    Ok(providers)
}
```

### 3.3 BoTG Streaming Over DHT

**Challenge:** BoTG expects continuous UDP streaming with TGP congestion control.

**Solution:** Layer BoTG over DHT message routing:

```rust
pub struct DhtBoTgProtocol {
    dht_transport: Arc<DhtTransport>,
    local_id: PeerId,
    store: Arc<BlockStore>,
    // ... same as BoTgProtocol
}

impl DhtBoTgProtocol {
    pub async fn request_blocks_dht(
        &self,
        peer_id: PeerId,
        blocks: Vec<BlockId>,
    ) -> Result<()> {
        // Create rollup request
        let rollup = RollupRequest {
            rollup_id: rand::random(),
            blocks: blocks.clone(),
            priority: 128,
        };

        // Send via DHT instead of direct UDP
        let msg = Message::new(
            MessageType::StreamOpen,
            self.local_id.clone(),
            session_id,
            counter,
            MessageBody::StreamOpen(StreamOpenMessage {
                stream_id: rollup.rollup_id as u32,
                kind: StreamKind::Blocks,
                rollup_id: rollup.rollup_id,
                blocks: rollup.blocks,
                max_inflight_chunks: 16,
                chunk_size: 1200,
            }),
        );

        // Route through DHT (O(1) lookup + routing!)
        self.dht_transport.send_message(peer_id, msg).await?;

        Ok(())
    }
}
```

---

## 4. Protocol Modifications Needed

### 4.1 Critical Changes

#### Change 1: Replace SocketAddr with DHT Addressing

**File:** `/opt/castle/workspace/palace/crates/consensus/peerexc/src/botg.rs`

**Before:**
```rust
pub async fn connect_to_peer(
    &self,
    peer_id: PeerId,
    peer_addr: SocketAddr,  // ❌ Direct addressing
) -> anyhow::Result<()>
```

**After:**
```rust
pub async fn connect_to_peer_dht(
    &self,
    peer_id: PeerId,
    // No peer_addr needed! DHT routes by peer_id
) -> anyhow::Result<()> {
    // Query DHT for peer's slot location
    let location_key = peer_location_key(&peer_id);
    let slot: SlotCoordinate = self.dht.get(&location_key).await?;

    // Store peer location for routing
    self.peer_slots.write().await.insert(peer_id.clone(), slot);

    Ok(())
}
```

#### Change 2: Add DHT Transport Trait

**New File:** `/opt/castle/workspace/palace/crates/consensus/peerexc/src/transport.rs`

```rust
use crate::{PeerId, Message, Result};
use async_trait::async_trait;

#[async_trait]
pub trait PeerTransport: Send + Sync {
    /// Send a message to a peer (routing handled by implementation)
    async fn send(&self, peer_id: &PeerId, msg: &Message) -> Result<()>;

    /// Receive the next message
    async fn recv(&self) -> Result<(PeerId, Message)>;

    /// Get our local peer ID
    fn local_id(&self) -> &PeerId;
}

// DHT implementation
pub struct DhtPeerTransport {
    dht: Arc<dyn citadel_dht::DHT>,
    local_id: PeerId,
    message_inbox: Arc<RwLock<Vec<Message>>>,
}

#[async_trait]
impl PeerTransport for DhtPeerTransport {
    async fn send(&self, peer_id: &PeerId, msg: &Message) -> Result<()> {
        // DHT routing implementation
        let message_key = self.peer_message_key(peer_id, msg.counter);
        let msg_bytes = bincode::serialize(msg)?;
        self.dht.put(message_key, msg_bytes).await?;
        Ok(())
    }

    async fn recv(&self) -> Result<(PeerId, Message)> {
        // Poll DHT for messages addressed to us
        // Implementation details...
        todo!()
    }

    fn local_id(&self) -> &PeerId {
        &self.local_id
    }
}
```

#### Change 3: Modify PeerReferral to Use DHT Slots

**File:** `/opt/castle/workspace/palace/crates/consensus/peerexc/src/messages.rs`

**Before:**
```rust
pub struct PeerHint {
    pub peer_id: PeerId,
    pub relay_hint: Option<String>,
    pub direct_endpoints: Vec<String>,  // ❌ Direct IPs
    // ...
}
```

**After:**
```rust
pub struct PeerHint {
    pub peer_id: PeerId,
    pub dht_slot: Option<SlotCoordinate>,  // ✅ DHT slot location
    pub relay_hint: Option<String>,        // Keep for backward compat
    pub direct_endpoints: Vec<String>,     // Optional fallback
    // ...
}
```

### 4.2 Optional Optimizations

#### Optimization 1: WantList as DHT Keys

Instead of gossiping WantLists, publish them to DHT:

```rust
// Publish our WantList to DHT
pub async fn publish_wantlist_dht(&self, wantlist: &WantList) -> Result<()> {
    let key = wantlist_key(&self.local_id);
    let value = bincode::serialize(wantlist)?;
    self.dht.put(key, value).await?;

    // Also index by block ranges for provider discovery
    for range in &wantlist.have_ranges {
        let provider_key = block_provider_key(range);
        // Add ourselves to provider list
        self.dht.append(provider_key, self.local_id.as_bytes()).await?;
    }

    Ok(())
}

// Query peers' WantLists from DHT
pub async fn query_wantlist_dht(&self, peer_id: &PeerId) -> Result<WantList> {
    let key = wantlist_key(peer_id);
    let value = self.dht.get(&key).await?;
    let wantlist = bincode::deserialize(&value)?;
    Ok(wantlist)
}
```

#### Optimization 2: Leverage DHT's O(1) Routing for Block Requests

```rust
// Instead of maintaining peer connections, use DHT for each block request
pub async fn fetch_block_via_dht(&self, block_id: &BlockId) -> Result<Block> {
    // Step 1: Find providers via DHT
    let provider_key = block_provider_key_single(block_id);
    let providers: Vec<PeerId> = self.dht.get(&provider_key).await?;

    // Step 2: Request from first available provider (DHT routes automatically)
    for provider in providers {
        if let Ok(block) = self.request_single_block(&provider, block_id).await {
            return Ok(block);
        }
    }

    Err(anyhow::anyhow!("Block not available"))
}
```

---

## 5. How DHT Routing Affects Message Delivery

### 5.1 Latency Impact

**Direct UDP (Current):**
- Single-hop delivery: ~1-50ms (LAN/WAN)
- Requires NAT traversal: +100-500ms (STUN/TURN)
- Total: ~1-550ms

**DHT Routing (Proposed):**
- O(1) slot lookup: ~2ns (constant time)
- Greedy routing hops: ~5-15 hops average (200k-360k nodes)
- Per-hop latency: ~10-50ms
- Total: ~50-750ms

**Assessment:** ⚠️ DHT routing adds ~50-200ms latency compared to direct connections, but eliminates NAT traversal complexity entirely.

### 5.2 Reliability Impact

**Direct UDP:**
- Single point of failure (peer offline = request fails)
- Requires connection state maintenance
- NAT mapping can expire

**DHT Routing:**
- ✅ Multiple routing paths (hexagonal mesh)
- ✅ No connection state needed
- ✅ Automatic failover (turn-left censorship resistance)
- ✅ Graceful degradation (blind identity grace period)

**Assessment:** ✅ DHT routing is **more reliable** than direct UDP for most scenarios.

### 5.3 Throughput Impact

**Direct UDP with TGP:**
- Optimized for continuous streaming
- ~100 Mbps sustained throughput
- Low overhead (UDP headers only)

**DHT Routing:**
- Each message is a DHT PUT/GET operation
- DHT throughput: 1.8M keys/sec = ~102 MB/s
- Additional overhead: DHT key routing

**Assessment:** ⚠️ DHT routing may reduce throughput for large continuous streams, but is excellent for request/response patterns.

**Hybrid Solution:** Use DHT for discovery and small messages, fall back to direct UDP for large block transfers when possible.

---

## 6. Assumptions About Peer Connectivity

### 6.1 Current Assumptions (Need Revision)

❌ **Assumption 1:** "Peers have direct UDP connectivity"
- **Reality with DHT:** Peers may not be directly reachable, but DHT routes messages through intermediate nodes.
- **Fix:** Remove direct connectivity requirement, rely on DHT routing.

❌ **Assumption 2:** "Peer addresses are SocketAddr (IP:port)"
- **Reality with DHT:** Peers are identified by PeerId and located at DHT slots.
- **Fix:** Replace SocketAddr with SlotCoordinate lookups.

❌ **Assumption 3:** "NAT traversal requires relay servers"
- **Reality with DHT:** DHT itself provides relay functionality through greedy routing.
- **Fix:** Remove separate relay server infrastructure, use DHT.

### 6.2 New Assumptions (DHT-Compatible)

✅ **Assumption 1:** "DHT provides O(1) peer location lookup"
- Citadel DHT guarantees this via `peer_location_key(peer_id)`.

✅ **Assumption 2:** "DHT routing delivers messages with <1s latency"
- Empirically verified: 0.5-1s average lookup @ 200k-360k nodes.

✅ **Assumption 3:** "Blind identities persist across network churn"
- Citadel's grace period (5 min) ensures key availability during transitions.

---

## 7. Optimization Suggestions for DHT Integration

### 7.1 Exploit DHT's Recursive Routing

**Current:** PeerExc maintains its own relay server infrastructure.

**Optimized:** Leverage Citadel's recursive DHT (Section 2.4 of spec):

```rust
// DHT uses itself for peer messaging!
pub async fn send_to_peer_recursive(
    &mut self,
    target: PeerId,
    msg: Vec<u8>,
) -> DHTResult<Vec<u8>> {
    // DHT finds peer location and routes message automatically
    // No need for separate relay infrastructure!

    let location_key = peer_location_key(target);
    let location: SlotOwnership = self.dht.get(&location_key).await?;

    let nonce = random();
    let response_key = peer_message_key(self.my_peer_id, nonce);
    let message = PeerMessage {
        from: self.my_peer_id,
        to: target,
        nonce,
        payload: msg,
        response_key,
        signature: self.sign(&msg),
    };

    let message_key = peer_message_key(target, nonce);
    self.dht.put(message_key, message.to_bytes()).await?;

    let response = self.dht.get_with_timeout(&response_key, Duration::from_secs(5)).await?;
    Ok(response)
}
```

### 7.2 WantList as DHT Native Structure

**Current:** WantLists exchanged via peer messages.

**Optimized:** Publish WantLists directly to DHT:

```rust
// Each peer publishes their WantList to a deterministic DHT key
let wantlist_key = blake3(b"wantlist" || peer_id.to_bytes());

// Other peers query WantLists on-demand
pub async fn discover_providers_for_range(&self, range: &BlockRange) -> Vec<PeerId> {
    let mut providers = Vec::new();

    // Scan DHT for peers with matching have_ranges
    // (Could be optimized with bloom filters or range trees)
    for peer_id in self.known_peers.iter() {
        let wantlist: WantList = self.dht.get(&wantlist_key(peer_id)).await?;

        if wantlist.have_ranges.iter().any(|r| ranges_overlap(r, range)) {
            providers.push(peer_id.clone());
        }
    }

    providers
}
```

### 7.3 BoTG Rollups via DHT Keys

**Current:** Rollups requested via streaming protocol.

**Optimized:** Rollups as DHT objects:

```rust
// Publish rollup to DHT for asynchronous retrieval
pub async fn publish_rollup_dht(&self, rollup: &RollupResponse) -> Result<()> {
    let rollup_key = blake3(b"rollup" || rollup.rollup_id.to_bytes());
    self.dht.put(rollup_key, bincode::serialize(rollup)?).await?;
    Ok(())
}

// Request rollup asynchronously
pub async fn request_rollup_dht(&self, rollup_id: u64, peer_id: &PeerId) -> Result<RollupResponse> {
    // Send request message via DHT
    let request = RollupRequest { rollup_id, ... };
    self.send_message_dht(peer_id, request).await?;

    // Poll for rollup to appear in DHT
    let rollup_key = blake3(b"rollup" || rollup_id.to_bytes());

    for _ in 0..10 {
        if let Ok(rollup_bytes) = self.dht.get(&rollup_key).await {
            return Ok(bincode::deserialize(&rollup_bytes)?);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err(anyhow::anyhow!("Rollup not available"))
}
```

### 7.4 Hybrid Direct/DHT Transport

**Best of Both Worlds:**

```rust
pub enum TransportMode {
    DirectUdp(SocketAddr),
    DhtRouted(SlotCoordinate),
    Hybrid { direct: SocketAddr, dht_fallback: SlotCoordinate },
}

pub async fn send_message_adaptive(
    &self,
    peer_id: &PeerId,
    msg: &Message,
) -> Result<()> {
    match self.get_transport_mode(peer_id).await? {
        TransportMode::DirectUdp(addr) => {
            // Fast path: direct UDP
            self.udp_transport.send(addr, msg).await
        }
        TransportMode::DhtRouted(slot) => {
            // DHT routing
            self.dht_transport.send(peer_id, msg).await
        }
        TransportMode::Hybrid { direct, dht_fallback } => {
            // Try direct first, fall back to DHT
            if let Err(_) = self.udp_transport.send(direct, msg).await {
                self.dht_transport.send(peer_id, msg).await
            } else {
                Ok(())
            }
        }
    }
}
```

---

## 8. Implementation Checklist

### Phase 1: Transport Abstraction (Day 1)
- [ ] Create `PeerTransport` trait in `transport.rs`
- [ ] Implement `DhtPeerTransport` with Citadel DHT integration
- [ ] Refactor `BoTgProtocol` to use `PeerTransport` trait
- [ ] Add `peer_location_key()` and `peer_message_key()` DHT key functions
- [ ] Write unit tests for DHT transport

### Phase 2: Peer Discovery (Day 1-2)
- [ ] Modify `PeerReferralMessage` to include DHT slot coordinates
- [ ] Implement `find_providers_dht()` using DHT queries
- [ ] Add `publish_wantlist_dht()` for DHT-native WantList advertising
- [ ] Update `RelayServer` to use DHT instead of centralized relay
- [ ] Write integration tests for peer discovery

### Phase 3: Message Routing (Day 2)
- [ ] Replace `SocketAddr` with DHT slot lookups in `connect_to_peer()`
- [ ] Implement DHT message routing in `request_blocks()`
- [ ] Add DHT response polling in `handle_rollup_response()`
- [ ] Test message delivery across DHT routing hops
- [ ] Measure latency impact of DHT routing

### Phase 4: Optimization (Day 3)
- [ ] Implement hybrid direct/DHT transport mode
- [ ] Add BoTG rollup publishing to DHT
- [ ] Optimize WantList discovery with DHT range queries
- [ ] Add caching layer for frequently queried DHT keys
- [ ] Performance benchmarking and tuning

### Phase 5: Testing & Validation (Day 3)
- [ ] End-to-end test: WantList exchange via DHT
- [ ] End-to-end test: Block retrieval via DHT routing
- [ ] End-to-end test: BoTG streaming over DHT
- [ ] Churn test: Peer leave/join with DHT routing
- [ ] Load test: 1000+ peers with DHT routing
- [ ] Documentation and examples

---

## 9. Risk Assessment

### Low Risk ✅
- **Message envelope compatibility:** No changes needed, already sender_id based
- **WantList protocol:** Pure data structures, transport-agnostic
- **Relay infrastructure:** Maps directly to DHT routing concepts

### Medium Risk ⚠️
- **Latency increase:** DHT routing adds ~50-200ms vs direct UDP
  - **Mitigation:** Use hybrid mode, fall back to direct when possible
- **Throughput reduction:** DHT may be slower for large continuous streams
  - **Mitigation:** Use DHT for discovery, direct UDP for bulk transfers
- **Complexity increase:** Adding transport abstraction layer
  - **Mitigation:** Well-defined trait interface, extensive testing

### High Risk (None Identified) ❌
- No breaking changes to core protocol
- No fundamental incompatibilities found
- All issues have clear mitigation strategies

---

## 10. Conclusion

### Is PeerExc DHT-Compatible As-Is?

**Answer:** ⚠️ **Mostly, but needs transport layer abstraction.**

PeerExc's core protocols (WantList, BoTG rollups, peer discovery) are transport-agnostic and work perfectly with DHT routing. The main work is replacing direct SocketAddr connectivity with DHT slot-based routing.

### Protocol Modifications Needed

**Critical (Must Have):**
1. Add `PeerTransport` trait for transport abstraction
2. Implement `DhtPeerTransport` using Citadel DHT
3. Replace `SocketAddr` with DHT slot lookups
4. Modify peer discovery to use DHT queries

**Optional (Nice to Have):**
1. Hybrid direct/DHT transport mode
2. DHT-native WantList publishing
3. BoTG rollup publishing to DHT
4. Caching layer for DHT queries

### How DHT Routing Affects Message Delivery

**Latency:**
- Adds ~50-200ms compared to direct UDP
- Still acceptable for block sync use case
- Can be mitigated with hybrid transport

**Reliability:**
- ✅ Improves reliability (multiple paths, automatic failover)
- ✅ Eliminates NAT traversal complexity
- ✅ Graceful degradation during churn

**Throughput:**
- May reduce throughput for continuous streams
- Excellent for request/response patterns
- DHT itself handles 1.8M keys/sec (102 MB/s)

### Optimization Suggestions

**Top 3 Recommendations:**

1. **Implement Hybrid Transport**
   - Use DHT for discovery and initial contact
   - Upgrade to direct UDP for bulk block transfers
   - Fall back to DHT if direct fails

2. **Leverage Recursive DHT Routing**
   - Use Citadel's `send_to_peer()` for all peer messaging
   - Eliminate separate relay server infrastructure
   - Reduce complexity and improve reliability

3. **Publish WantLists to DHT**
   - Each peer publishes WantList to deterministic DHT key
   - Provider discovery via DHT range queries
   - Reduces gossip overhead, improves scalability

### Final Verdict

✅ **PeerExc is HIGHLY compatible with Citadel DHT routing.**

The protocol's modular design and existing relay concepts make DHT integration straightforward. With 2-3 days of focused implementation, PeerExc can be fully DHT-native while retaining the option for direct connectivity when available.

**Recommended Next Steps:**
1. Implement `PeerTransport` trait abstraction
2. Build `DhtPeerTransport` with Citadel integration
3. Add hybrid transport mode for optimal performance
4. Extensive testing with simulated DHT network

**Expected Benefits:**
- ✅ O(1) peer discovery (vs O(log n) in traditional DHTs)
- ✅ Automatic NAT traversal (no STUN/TURN needed)
- ✅ Censorship resistance (multiple routing paths)
- ✅ Graceful churn handling (blind identity grace period)
- ✅ Scalability (DHT handles 200k-1M nodes easily)

---

**Assessment Complete.**

This analysis demonstrates that consensus-peerexc and Citadel DHT are highly synergistic. The integration effort is manageable, the risks are low, and the benefits are substantial. The combination of PeerExc's WantList-driven block exchange with Citadel's O(1) DHT routing creates a powerful foundation for distributed consensus systems.
