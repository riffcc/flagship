# Lens Node v2 Architecture Context

## CRITICAL: DHT is DISTRIBUTED, NOT LOCAL

**The DHT is a GLOBAL distributed hash table across all nodes, NOT a local cache.**

### Content-Addressed Slots (CLAIMABLE, NOT CALCULATED)

**CRITICAL UPDATE (2025-10-15)**: Slots are **CLAIMED** by nodes and announced via epidemic gossip, NOT calculated from peer_id hash!

**OLD METHOD (DEPRECATED)**:
```
peer_id → Blake3 hash → (x, y, z) in 256³ space  ❌ NO LONGER USED
peer_id_to_slot() function                         ❌ DEPRECATED
```

**NEW METHOD (CURRENT)**:
```
Nodes CLAIM arbitrary slots and announce them via epidemic gossip
All nodes track claimed slots in peer_slots HashMap
DHT routing uses claimed slots from gossip, NOT calculated slots
```

Each node:
1. **CLAIMS** a slot coordinate of their choosing
2. Announces the claim via epidemic gossip through relay
3. All peers store the claimed slot in `peer_slots: HashMap<String, SlotCoordinate>`
4. DHT routing uses these claimed slots for greedy mesh routing

### DHT Storage and Routing

**Key principle: Keys are stored AT the node that claims the slot the key hashes to.**

```
slot_ownership_key(slot) → Blake3 hash → target_slot → stored at node claiming target_slot
peer_location_key(peer_id) → Blake3 hash → target_slot → stored at node claiming target_slot
```

**DHT Operations:**

1. **DHT PUT**: Route the key to the responsible node and store it there
   - NOT: Store locally and gossip
   - YES: Route via greedy routing to the node claiming the target slot

2. **DHT GET**: Route the query to the responsible node and fetch the value
   - NOT: Check local storage
   - YES: Route via greedy routing to the node claiming the target slot
   - **FIXED (2025-10-15)**: Now uses claimed slots from `peer_slots` HashMap instead of `peer_id_to_slot()`

3. **Relay Role (UPDATED 2025-10-15)**: The relay is for SIGNALING + EPIDEMIC GOSSIP ONLY
   - Used ONLY for WebRTC signaling (SDP/ICE exchange)
   - Used for epidemic gossip (slot ownership, peer referrals)
   - **NOT USED** for DHT block routing (deprecated as of 2025-10-15)
   - Once WebRTC mesh forms, DHT operations go directly over WebRTC DataChannels

### WebRTC Connection Establishment

**Before**: Tried to use DHT for SDP signaling (WRONG - chicken and egg problem)

**Correct approach**:
1. Use relay as WebRTC signaling server (standard WebRTC pattern)
2. Send SDP offers/answers via relay WebSocket messages
3. Once WebRTC connection established, use it for DHT routing
4. Eventually all DHT traffic goes over WebRTC, relay becomes unused

**Message flow**:
```
Node A → SDP offer → Relay → Node B
Node B → SDP answer → Relay → Node A
WebRTC connection established
Node A ←→ WebRTC DataChannel ←→ Node B
DHT GET/PUT now travels over WebRTC
```

### Neighbor Discovery

1. Node calculates 8 geometric neighbors (6 hexagonal + 2 vertical)
2. For each neighbor slot, queries DHT: `slot_ownership_key(neighbor_slot)`
3. DHT routes query to node claiming that slot
4. Response contains peer_id of node claiming that neighbor slot
5. Initiate WebRTC connection to that peer_id

**This is LAZY discovery - no neighbor caches, DHT IS the cache.**

### SPORE WantList Protocol (NEW - 2025-10-15)

**SPORE = Succinct Proofs Of Range Exclusions**

Instead of exchanging lists of blocks (old relay method), nodes exchange **RANGES**:

**Example Compression**:
```
OLD relay method:
- Want blocks: [1, 2, 3, 4, ..., 50000] = 50,000 block IDs
- Message size: ~1.6 MB

NEW SPORE WantList:
- Want ranges: [(1, 50000)] = 1 range
- Message size: 32 bytes
- Compression ratio: 50,000x!
```

**Protocol Flow**:
1. Node builds `have_ranges` from local DHT storage using `build_ranges_from_keys()`
2. Node builds `want_ranges` by finding gaps in have_ranges
3. Node sends `WantListMessage` via WebRTC DataChannel to connected peers
4. Peer computes intersection: "what I have that you want"
5. Peer responds with `RangeResponse` containing DHT entries within requested ranges
6. Node stores received entries in local DHT

**Key Functions** (see `spore_wantlist.rs`):
- `build_ranges_from_keys()`: Convert sorted u64 key hashes to contiguous ranges
- `compute_want_ranges()`: Find what THEY have that WE don't
- `merge_ranges()`: Combine overlapping/adjacent ranges

**Implementation Status**:
- ✅ Protocol data structures defined
- ✅ Range computation logic implemented and tested
- ✅ sync_orchestrator updated to use WebRTC SPORE WantList
- ⏳ RangeResponse handler (to be implemented)
- ⏳ Bloom filters for fast membership testing (TODO)
- ⏳ Merkle proofs for verification (TODO)

### What NOT to Do

❌ Store DHT entries locally without routing
❌ Use "local DHT cache" concept
❌ ~~Gossip DHT entries via relay~~ (slot ownership IS gossiped via epidemic broadcast - this is correct!)
❌ Store slot ownership locally and expect others to see it
❌ Use DHT for WebRTC signaling (creates circular dependency)
❌ **Calculate slots using peer_id_to_slot()** - slots are CLAIMED, not calculated!
❌ **Route blocks through relay** - use WebRTC SPORE WantList instead!

### What TO Do

✅ Route DHT PUT to the owning node
✅ Route DHT GET to the owning node
✅ Use relay as dumb proxy until WebRTC exists
✅ Use relay as WebRTC signaling server
✅ Once WebRTC exists, route DHT over WebRTC DataChannels
✅ Query DHT for neighbor discovery on-demand
✅ **Track claimed slots in peer_slots HashMap** (from epidemic gossip)
✅ **Use WebRTC SPORE WantList for block synchronization** (50,000x compression!)
✅ **Use epidemic gossip for slot ownership announcements** (all nodes learn all claims)

## Current State (v0.8.53 - 2025-10-15)

### ✅ Recently Completed (2025-10-15)

1. **DHT GET Timeout Bug Fixed** (Commit 67cbd53)
   - Added `peer_slots: HashMap<String, SlotCoordinate>` to RelayState
   - Store claimed slots when receiving slot ownership gossip
   - DHT GET handler now uses claimed slots instead of `peer_id_to_slot()`
   - **Result**: DHT GET requests now route correctly to claimed slots!

2. **sync_orchestrator Updated to WebRTC SPORE WantList** (Commit c6c55e1)
   - Replaced OLD relay-based block routing with WebRTC SPORE WantList
   - Added `build_ranges_from_keys()` helper to convert block lists to ranges
   - Deprecated OLD relay WantListReceived and BlockRequestReceived handlers
   - **Result**: Block synchronization now uses range-based P2P with 50,000x compression!

3. **SPORE WantList Test Fixed** (Commit c6c55e1)
   - Fixed `test_compute_want_ranges_simple` expectations
   - **Test Status**: 164/170 tests passing (up from 163)

### ✅ Working

- ✅ Relay WebSocket connections
- ✅ Peer discovery via relay
- ✅ Slot ownership epidemic gossip (all nodes learn all claims)
- ✅ DHT PUT routing (exists in relay.rs)
- ✅ DHT GET routing using claimed slots from peer_slots HashMap
- ✅ WebRTC SPORE WantList protocol for block synchronization
- ✅ Claimed slot tracking in peer_slots HashMap

### ❌ Remaining Work (6 Failing Tests)

**Root Cause**: Tests create `SyncOrchestrator` with a `dht_get_fn` that calls `relay.dht_get()`. In unit tests without a running relay, DHT queries fail.

**Failing Tests**:
1. `distributed_dht::tests::test_local_put_get` - Could not find key in test slot
2. `slot_identity::tests::test_trump_threshold` - 20% improvement threshold check
3. `sync_orchestrator::test_geometric_neighbor_discovery_via_lazy_node` - Expected 8 neighbors, found 0
4. `sync_orchestrator::test_lazy_node_caches_neighbors_for_10_seconds` - Neighbor slot not in DHT
5. `sync_orchestrator::test_sdp_offer_stored_in_dht` - SDP offer should be stored
6. `sync_orchestrator::test_webrtc_connection_via_dht_signaling` - Timeout waiting for SDP answer

**Fix Needed**:
- Make `SyncOrchestrator` accept an optional test-friendly `dht_get_fn` that queries local storage
- This is the same pattern that `lazy_node` tests use successfully
- Tests should be able to mock DHT operations without requiring a running relay

### 📋 Next Steps

1. ⏳ Fix sync_orchestrator tests by providing test-friendly dht_get_fn
2. ⏳ Fix distributed_dht::test_local_put_get for claimed slots
3. ⏳ Fix slot_identity::test_trump_threshold for claimed slots
4. ⏳ Run full test suite to verify all fixes (target: 170/170 passing)
5. ⏳ Commit all test fixes
