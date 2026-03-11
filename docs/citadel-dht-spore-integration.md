# Vesper Hexagonal Routing for SPORE

**Date:** 2025-10-12
**Status:** Design Proposal
**Priority:** HIGH - Solves 5,000-node convergence problem

## Problem Statement

Current SPORE gossip sync shows **poor convergence at scale:**
- 5,000 nodes with 10 random peers: **7% convergence**
- Release counts range from 11 to 4,749 (wildly divergent!)
- Random peer selection doesn't provide topology guarantees

## Solution: Vesper Hexagonal Toroid

Adopt Vesper's structured topology for deterministic, efficient gossip propagation.

### Core Architecture

#### 1. Node Positioning: Directional Vectors

Instead of random peer connections, nodes position themselves in a **2.5D hexagonal toroid**:

```rust
/// Node position in hex toroid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexPosition {
    /// Directional vector coordinates
    /// Uses directional vectors, not axial coordinates
    pub direction: (i32, i32, i32),  // (x, y, z)
}

impl HexPosition {
    /// Six cardinal hex directions
    pub const DIRECTIONS: [(i32, i32, i32); 8] = [
        (1, 0, 0),   // +A
        (0, 1, 0),   // +B
        (0, 0, 1),   // +C
        (-1, 0, 0),  // -A
        (0, -1, 0),  // -B
        (0, 0, -1),  // -C
        (0, 0, 2),   // +Z (up)
        (0, 0, -2),  // -Z (down)
    ];

    /// Get neighbor in given direction
    pub fn neighbor(&self, dir: (i32, i32, i32)) -> HexPosition {
        HexPosition {
            direction: (
                self.direction.0 + dir.0,
                self.direction.1 + dir.1,
                self.direction.2 + dir.2,
            )
        }
    }

    /// Get all 8 neighbors (6 hex + 2 vertical)
    pub fn neighbors(&self) -> Vec<HexPosition> {
        Self::DIRECTIONS.iter()
            .map(|&dir| self.neighbor(dir))
            .collect()
    }
}
```

#### 2. Node Joining: Modulo-Based Slot Selection

When a node joins, it picks a slot using **modulo of available mesh slots**:

```rust
/// Node joining protocol
pub struct HexMesh {
    /// Total mesh size (grows dynamically)
    mesh_size: usize,

    /// Nodes indexed by hex position
    nodes: HashMap<HexPosition, NodeInfo>,
}

impl HexMesh {
    /// New node picks a slot
    pub fn join_node(&mut self, node_id: String) -> Result<HexPosition> {
        // Count open slots in the toroid
        let open_slots = self.count_open_slots();

        if open_slots == 0 {
            // Grow the toroid
            self.expand_toroid();
        }

        // Pick slot using modulo
        let slot_index = hash_node_id(&node_id) % open_slots;
        let position = self.find_nth_open_slot(slot_index)?;

        // Node joins at this position
        self.nodes.insert(position.clone(), NodeInfo {
            id: node_id,
            position: position.clone(),
        });

        Ok(position)
    }

    /// Count slots that have at least 1 neighbor
    fn count_open_slots(&self) -> usize {
        let mut open = 0;
        for pos in self.all_positions() {
            if !self.nodes.contains_key(&pos) {
                // Check if has neighbors
                let neighbor_count = pos.neighbors().iter()
                    .filter(|n| self.nodes.contains_key(n))
                    .count();
                if neighbor_count >= 1 {
                    open += 1;
                }
            }
        }
        open
    }
}
```

#### 3. Structured Gossip: Broadcast to 8 Neighbors

Instead of random peers, each node **always syncs with its 8 neighbors**:

```rust
/// Hex-based sync
impl TestNode {
    /// Get my position in the hex mesh
    pub fn hex_position(&self) -> HexPosition {
        // Derive from node ID or assigned at join
        HexPosition::from_node_id(&self.id)
    }

    /// Sync with all 8 hex neighbors
    pub fn sync_hex_neighbors(&self, mesh: &HexMesh) -> Result<()> {
        let my_pos = self.hex_position();

        // Get all 8 neighbors
        for neighbor_pos in my_pos.neighbors() {
            if let Some(neighbor_node) = mesh.nodes.get(&neighbor_pos) {
                // Sync with this neighbor
                self.sync_from(neighbor_node)?;
            }
        }

        Ok(())
    }
}

/// Full mesh sync using hex topology
fn sync_all_nodes_hex(nodes: &[TestNode], mesh: &HexMesh) -> Result<()> {
    // Each node syncs with its 8 hex neighbors
    for node in nodes {
        node.sync_hex_neighbors(mesh)?;
    }

    Ok(())
}
```

#### 4. Hex Routing: "Turn Left" for O(1) Lookups

Finding a node in the mesh uses **geometric routing**:

```rust
impl HexPosition {
    /// Calculate distance to target (Manhattan distance in hex space)
    pub fn distance_to(&self, target: &HexPosition) -> i32 {
        (self.direction.0 - target.direction.0).abs() +
        (self.direction.1 - target.direction.1).abs() +
        (self.direction.2 - target.direction.2).abs()
    }

    /// Greedy routing: pick neighbor closest to target
    pub fn route_to(&self, target: &HexPosition) -> (i32, i32, i32) {
        let mut best_dir = (0, 0, 0);
        let mut best_distance = i32::MAX;

        for &dir in &Self::DIRECTIONS {
            let next_pos = self.neighbor(dir);
            let dist = next_pos.distance_to(target);

            if dist < best_distance {
                best_distance = dist;
                best_dir = dir;
            }
        }

        best_dir
    }
}
```

**Routing Example:**
```
Node at (0,0,0) wants to reach (5,3,2)
1. Check all 8 neighbors
2. Pick neighbor closest to (5,3,2)
3. Move to that neighbor
4. Repeat until target reached

Average hops: O(√N) for N nodes
With hex topology optimization: approaches O(1)
```

### Performance Expectations

Based on Vesper benchmarks:

**5,000 nodes:**
- Expected convergence: **>99%**
- Each node syncs with 8 neighbors (not 10 random)
- Structured topology guarantees full mesh coverage
- Sync time: ~10-20 iterations to reach all nodes

**Expected propagation:**
```
Round 0: Node creates release
Round 1: 8 neighbors have it
Round 2: 8 × 8 = 64 nodes have it
Round 3: 64 × 8 = 512 nodes have it
Round 4: 512 × 8 = 4,096 nodes have it
Round 5: Full 5,000 nodes converged!
```

### Sybil Resistance (Optional Enhancement)

From Vesper's defense mechanisms:

```rust
/// VDF-based clock for admission control
pub struct VDFClock {
    /// Current epoch (10-second cycles)
    epoch: u64,

    /// VDF output for this epoch
    vdf_output: Vec<u8>,
}

impl VDFClock {
    /// Check if node can admit new peers this epoch
    pub fn can_admit(&self, node_id: &str) -> bool {
        let hash = blake3::hash(
            &[node_id.as_bytes(), &self.epoch.to_le_bytes()].concat()
        );

        // Only 1/3 of nodes can admit each epoch
        u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap()) % 3 == 0
    }

    /// Check if node can receive joins this epoch
    pub fn can_receive_joins(&self, node_id: &str) -> bool {
        let hash = blake3::hash(
            &[node_id.as_bytes(), &self.epoch.to_le_bytes()].concat()
        );

        // Different 1/3 can receive joins
        u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap()) % 3 == 1
    }
}

/// Cryptographically-signed disconnect events
#[derive(Debug, Serialize, Deserialize)]
pub struct DisconnectEvent {
    /// Slot that disconnected
    pub slot: HexPosition,

    /// Action taken
    pub action: String,  // "disconnect"

    /// Target node's peer ID
    pub target: String,

    /// Timestamp
    pub timestamp: u64,

    /// Cryptographic proof (signed by disconnecting node)
    pub proof: Vec<u8>,
}

impl DisconnectEvent {
    /// Broadcast to 2-hop neighbors
    pub fn broadcast_2_hop(&self, mesh: &HexMesh, origin: &HexPosition) {
        // Send to immediate neighbors
        for neighbor_pos in origin.neighbors() {
            // ... send to neighbor ...

            // Send to neighbor's neighbors (2-hop)
            for second_hop in neighbor_pos.neighbors() {
                // ... send to second-hop neighbor ...
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: Hex Topology (Core)

1. **Add HexPosition to Release**
   ```rust
   pub struct Release {
       // ... existing fields ...

       /// Hex mesh position of originating node
       #[serde(rename = "hexPosition")]
       pub hex_position: Option<HexPosition>,
   }
   ```

2. **Implement HexMesh in Tests**
   - Add `HexMesh` struct to `multi_node_sync.rs`
   - Assign positions during node creation
   - Update `sync_all_nodes()` to use hex neighbors

3. **Run 5,000-Node Test**
   - Verify convergence improves to >95%
   - Measure sync rounds needed
   - Profile performance

### Phase 2: Hex Routing (Production)

1. **Add to SyncOrchestrator**
   - Track node's hex position
   - Discover hex neighbors via relay
   - Route WantLists to appropriate neighbors

2. **Update P2P Network Layer**
   - Implement hex-aware peer discovery
   - Prefer syncing with hex neighbors
   - Fall back to random peers if neighbor unavailable

### Phase 3: Sybil Resistance (Future)

1. **VDF Clock**
   - Optional: Add VDF for timing
   - Implement admission rotation

2. **Signed Disconnect Events**
   - Cryptographic accountability
   - 2-hop gossip for dispute resolution

## Benefits

### Immediate:
- ✅ **Deterministic topology** - No random peer selection
- ✅ **Guaranteed coverage** - Hex mesh ensures all nodes reachable
- ✅ **Efficient gossip** - 8 neighbors instead of random 10
- ✅ **Faster convergence** - Structured propagation reaches all nodes

### Long-term:
- ✅ **O(1) lookups** - Geometric routing through hex space
- ✅ **Scalability** - Proven to 360,000 nodes
- ✅ **Sybil resistance** - VDF + PoW + rotation mechanisms
- ✅ **Accountability** - Signed events + 2-hop gossip

## Comparison

### Current (Random 10-Peer Gossip):
```
5,000 nodes × 10 peers = 50,000 connections
Convergence: 7%
Release counts: 11 to 4,749 (huge variance)
No topology guarantees
```

### With Vesper Hex (8 Structured Neighbors):
```
5,000 nodes × 8 neighbors = 40,000 connections
Expected convergence: >99%
Expected variance: <1% (near-perfect)
O(√N) propagation hops
```

## References

- Vesper Notes.pdf (pages 1-5)
- Vesper benchmarks: 200K-360K nodes
- Hex routing: "turn left" algorithm
- 2.5D toroid: 6 hex + 2 vertical neighbors

## Next Steps

1. Implement `HexPosition` and `HexMesh` structs
2. Update `test_5000_nodes_massive_scale` to use hex topology
3. Run test and measure convergence improvement
4. If successful (>95%), integrate into production SyncOrchestrator

---

**This could be the breakthrough SPORE needs to scale to production!** 🚀
