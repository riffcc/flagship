# SPORE WantList Protocol with Range Exclusions

**Version:** 1.0.0
**Date:** October 15, 2025
**Status:** Specification

---

## Overview

SPORE (Streaming Protocol Over Relay Extensions) WantList protocol enables efficient DHT synchronization across the hexagonal toroidal mesh using **succinct proofs of range exclusions**.

Instead of requesting individual keys or transferring entire datasets, peers exchange compact range representations of what they have and need, then transfer only the gaps.

---

## Dual Mode Architecture

### Light Mode (Default)
**Purpose:** Sync slot ownership map only
**Size:** ~529 slots × 200 bytes = ~105 KB
**Use Case:** Public mesh, privacy-preserving, scalable routing

**What Syncs:**
- Slot ownership entries (who owns which slot)
- Compact representation: ~100 ranges × 16 bytes = 1.6 KB

**What Doesn't Sync:**
- DHT key/value pairs (except slot ownership)
- Content data (queries route to slot owner)

**Result:**
- Every peer has complete routing table
- Queries route through mesh to slot owner
- O(log n) routing distance
- Privacy-preserving (peers don't see all data)

### Heavy Mode (Enterprise)
**Purpose:** Full DHT replication across all peers
**Size:** Full DHT state (application-dependent)
**Use Case:** High availability, zero-latency reads, maximum redundancy

**What Syncs:**
- ALL DHT key/value pairs
- Compact representation: Roaring Bitmap (~1-2 bits per key)
- 1M keys = 125-250 KB compressed

**What Doesn't Sync:**
- Nothing (full replication)

**Result:**
- Every peer is a full replica
- All queries are LOCAL (O(1) reads)
- Perfect redundancy
- Load balancing via mesh routing

---

## Range Exclusion Protocol

### Data Structures

```rust
/// Range of keys (inclusive)
pub type KeyRange = (u64, u64);

/// WantList message with range exclusions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantListMessage {
    /// Protocol version
    pub version: u32,

    /// Ranges we DON'T have (requesting these)
    pub want_ranges: Vec<KeyRange>,

    /// Ranges we DO have (offering these)
    pub have_ranges: Vec<KeyRange>,

    /// Optional Bloom filter for fast membership testing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub have_filter: Option<BloomFilter>,

    /// Timestamp for freshness
    pub timestamp: u64,

    /// Peer ID of sender
    pub peer_id: String,
}

/// Response containing requested range data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeResponse {
    /// The range being satisfied
    pub range: KeyRange,

    /// DHT entries within this range
    pub entries: Vec<DhtEntry>,

    /// Optional: Merkle proof for verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merkle_proof: Option<MerkleProof>,
}

/// DHT entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtEntry {
    /// Blake3 hash of the key (u64 for range operations)
    pub key_hash: u64,

    /// Full key bytes
    pub key: Vec<u8>,

    /// Value bytes
    pub value: Vec<u8>,

    /// Timestamp for conflict resolution
    pub timestamp: u64,

    /// Slot owner (for verification)
    pub slot_owner: String,
}
```

### Protocol Flow

**1. Initial Handshake:**
```
Peer A → Peer B: WantList
{
    "want_ranges": [],  // Empty on first sync
    "have_ranges": [[0, 100], [200, 300]],
    "timestamp": 1760541769,
    "peer_id": "bafk..."
}
```

**2. Range Exchange:**
```
Peer B → Peer A: WantList
{
    "want_ranges": [[0, 49], [301, 1000]],  // Gaps in A's ranges
    "have_ranges": [[50, 250], [350, 500]],
    "timestamp": 1760541770,
    "peer_id": "bafk..."
}
```

**3. Data Transfer:**
```
Peer A → Peer B: RangeResponse
{
    "range": [0, 49],
    "entries": [
        {"key_hash": 15, "key": [...], "value": [...]},
        {"key_hash": 42, "key": [...], "value": [...]}
    ]
}

Peer B → Peer A: RangeResponse
{
    "range": [301, 1000],
    "entries": [...]
}
```

**4. Epidemic Gossip:**
- Peer A gossips new ranges to ITS 8 neighbors
- Peer B gossips new ranges to ITS 8 neighbors
- Eventually: All peers converge to same state

---

## Light Mode Implementation

### Slot Ownership Sync

**Key Space:** 0-528 (529 total slots)

**WantList Message:**
```json
{
    "version": 1,
    "want_ranges": [[0, 528]],
    "have_ranges": [],
    "timestamp": 1760541769,
    "peer_id": "bafk..."
}
```

**Response (from neighbor):**
```json
{
    "range": [0, 528],
    "entries": [
        {
            "key_hash": 42,
            "key": "slot-ownership-42",
            "value": {
                "slot": {"x": 1, "y": 2, "z": 0},
                "owner_peer_id": "bafk...",
                "timestamp": 1760541769
            },
            "timestamp": 1760541769,
            "slot_owner": "bafk..."
        }
    ]
}
```

**Bandwidth:**
- Initial sync: ~105 KB (529 entries × 200 bytes)
- Incremental sync: Only new/changed entries
- Typical update: <1 KB per new peer

**Convergence Time:**
- 8-neighbor mesh with epidemic gossip
- O(log n) hops to reach all peers
- Expected: <10 seconds for 529 nodes

---

## Heavy Mode Implementation

### Full DHT Sync

**Key Space:** 0 to u64::MAX (variable based on usage)

**WantList with Roaring Bitmap:**
```json
{
    "version": 1,
    "want_ranges": [[0, 1000000]],
    "have_ranges": [[0, 500000], [600000, 800000]],
    "have_filter": {
        "type": "roaring_bitmap",
        "compressed_data": "base64_encoded_bitmap"
    },
    "timestamp": 1760541769,
    "peer_id": "bafk..."
}
```

**Response (batched):**
```json
{
    "range": [500001, 599999],
    "entries": [
        // Batch of 1000 entries
        {"key_hash": 500123, "key": [...], "value": [...]},
        {"key_hash": 500456, "key": [...], "value": [...]}
    ]
}
```

**Bandwidth Optimization:**
- Roaring Bitmap: 1-2 bits per key
- 1M keys = 125-250 KB bitmap
- Transfer only gaps: Typically <10% of total
- **10x bandwidth savings!**

**Convergence Time:**
- Depends on DHT size and bandwidth
- 1M keys @ 1 Gbps = ~8 seconds initial sync
- Incremental updates: Real-time

---

## Range Set Operations

### Computing Want Ranges

```rust
/// Compute ranges we're missing based on what peer has
pub fn compute_want_ranges(
    their_ranges: &[KeyRange],
    my_ranges: &[KeyRange],
    total_range: KeyRange,
) -> Vec<KeyRange> {
    let mut wants = Vec::new();
    let (start, end) = total_range;

    // Start from beginning of total range
    let mut cursor = start;

    // For each range they have
    for &(their_start, their_end) in their_ranges {
        // If there's a gap before this range, we want it
        if cursor < their_start {
            wants.push((cursor, their_start - 1));
        }
        cursor = their_end + 1;
    }

    // Final gap to end of range
    if cursor <= end {
        wants.push((cursor, end));
    }

    wants
}
```

### Merging Ranges

```rust
/// Merge overlapping/adjacent ranges
pub fn merge_ranges(ranges: &[KeyRange]) -> Vec<KeyRange> {
    if ranges.is_empty() {
        return vec![];
    }

    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|r| r.0);

    let mut merged = vec![sorted[0]];

    for &(start, end) in &sorted[1..] {
        let last_idx = merged.len() - 1;
        let (last_start, last_end) = merged[last_idx];

        // Overlapping or adjacent?
        if start <= last_end + 1 {
            // Merge by extending last range
            merged[last_idx] = (last_start, end.max(last_end));
        } else {
            // Separate range
            merged.push((start, end));
        }
    }

    merged
}
```

---

## Epidemic Gossip Strategy

### Propagation Rules

1. **Initial Sync:**
   - New peer connects to 8 neighbors
   - Sends WantList for full range
   - Receives entries from neighbors
   - Stores in local DHT

2. **Continuous Sync:**
   - Every 30 seconds: Send WantList to neighbors
   - Include ranges acquired since last sync
   - Receive missing ranges from neighbors
   - Gossip new ranges to OTHER neighbors

3. **Deduplication:**
   - Track (key_hash, timestamp) pairs
   - Skip entries we already have
   - Use timestamp for conflict resolution (last-write-wins)

4. **Convergence:**
   - Eventually: All peers have same state
   - Time: O(log n) gossip rounds
   - Typical: 10-30 seconds for 529 nodes

---

## Bandwidth Analysis

### Light Mode

**Scenario:** 529 peers, each owns 1 slot

**Initial Sync (new peer):**
- Request: WantList with empty have_ranges (~200 bytes)
- Response: 529 slot ownership entries × 200 bytes = 105 KB
- **Total:** 105 KB per peer

**Incremental Sync (new peer joins):**
- Gossip: 1 new slot ownership entry = 200 bytes
- Propagates to all 529 peers via epidemic gossip
- **Total:** 200 bytes × 8 neighbors × log(529) = ~14 KB

**Steady State:**
- Periodic WantList exchange: ~200 bytes every 30s
- No data transfer if no changes
- **Bandwidth:** ~7 bytes/second per peer

### Heavy Mode

**Scenario:** 1M DHT keys across 529 peers

**Initial Sync (new peer):**
- Request: WantList with Roaring Bitmap (250 KB)
- Response: 1M entries, assume 10% gaps = 100K entries
- Entry size: ~500 bytes average (key + value + metadata)
- **Total:** 50 MB per peer

**Incremental Sync (new keys added):**
- Gossip: New keys only
- Assume 1% churn = 10K new keys/hour
- **Total:** 5 MB/hour per peer = 1.4 KB/second

**Steady State:**
- Periodic WantList: ~250 KB every 30s = 8 KB/second
- Plus incremental updates: 1.4 KB/second
- **Bandwidth:** ~10 KB/second per peer

**For 529 nodes @ 10 KB/s = 5.29 MB/s total mesh traffic**
**Relay: Minimal (only WebSocket signaling, not data transfer)**

---

## Implementation Priorities

### Phase 1: Light Mode (Immediate)
- [ ] Implement range set operations (compute_want_ranges, merge_ranges)
- [ ] Add WantList message types to relay.rs
- [ ] Implement slot ownership sync protocol
- [ ] Test mesh merging (529 nodes → unified mesh)

### Phase 2: Epidemic Gossip (Short Term)
- [ ] Background task: Send WantList to neighbors every 30s
- [ ] Deduplication cache for received entries
- [ ] Timestamp-based conflict resolution
- [ ] Measure convergence time

### Phase 3: Heavy Mode (Medium Term)
- [ ] Implement Roaring Bitmap integration
- [ ] Full DHT replication protocol
- [ ] Configurable mode selection (Light/Heavy per peer)
- [ ] Bandwidth monitoring and throttling

### Phase 4: Optimization (Long Term)
- [ ] Bloom filters for fast membership testing
- [ ] Merkle proofs for range verification
- [ ] Compression for large value transfers
- [ ] Multi-threaded gossip for parallelism

---

## Success Metrics

### Light Mode Goals:
- ✅ 100% slot ownership coverage across mesh
- ✅ <30 second convergence time for 529 nodes
- ✅ <10 KB/s sustained bandwidth per peer
- ✅ Automatic mesh island merging

### Heavy Mode Goals:
- ✅ 100% DHT replication across all peers
- ✅ <60 second initial sync for 1M keys
- ✅ <50 KB/s sustained bandwidth per peer
- ✅ O(1) local query latency

---

## Security Considerations

### Trust Model:
- **Light Mode:** Trust slot ownership from neighbors (verify via DHT)
- **Heavy Mode:** Trust full DHT from neighbors (verify via Merkle proofs)
- **Sybil Resistance:** Slot ownership via content-addressed IDs
- **Byzantine Fault Tolerance:** Timestamp-based conflict resolution

### Attack Vectors:
1. **False Slot Ownership:**
   - Mitigation: Verify owner's peer_id matches slot hash
   - Detection: Cross-check with multiple neighbors

2. **Range Exclusion Lies:**
   - Mitigation: Request ranges from multiple neighbors
   - Detection: Hash verification of received data

3. **DHT Poisoning:**
   - Mitigation: Timestamp-based last-write-wins
   - Detection: Merkle proofs for large ranges

---

## Conclusion

SPORE WantList protocol with range exclusions enables:
- ✅ Efficient Light mode (routing table only)
- ✅ Practical Heavy mode (full replication)
- ✅ Automatic mesh island merging
- ✅ 10x bandwidth savings via gap-only transfer
- ✅ Epidemic gossip for eventual consistency

**This is the protocol that makes the greatest P2P network practical.**

---

**Next Steps:** Implement Light mode WantList in lens-v2-node v0.8.62
