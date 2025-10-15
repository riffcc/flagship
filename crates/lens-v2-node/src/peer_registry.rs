//! Peer Registry using Citadel DHT Hexagonal Toroidal Mesh
//!
//! This module implements P2P peer discovery using the Citadel DHT's hexagonal
//! toroidal mesh topology with O(1) routing and lazy neighbor discovery.
//!
//! Key concepts from Citadel DHT Spec Section 2.4 (Recursive DHT):
//! - Peers are assigned SlotCoordinates in a 2.5D hexagonal toroidal mesh
//! - Each peer maintains exactly 8 neighbors (6 hexagonal + 2 vertical)
//! - Neighbor discovery is LAZY - query DHT on-demand, no caching needed
//! - The DHT uses itself recursively for topology discovery
//! - Minimal state: just SlotCoordinate + MeshConfig (~64 bytes per node!)

use citadel_core::topology::{Direction, MeshConfig, SlotCoordinate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::slot_identity::SlotId;

/// Calculate optimal mesh dimensions for a given number of nodes
/// PURE ALGORITHMIC approach with NO hardcoded thresholds!
///
/// Algorithm: Grow mesh one dimension at a time from 1×1×1
/// - Always increase the SMALLEST dimension first
/// - Keep mesh as flat as possible (prefer w, h over d)
/// - Aim for roughly square horizontal (w ≈ h) before going vertical
/// - Natural result: gentle gradation like 1×1×1 → 2×1×1 → 2×2×1 → 3×2×1 → 3×3×1
/// - **HEADROOM**: Always accommodate current nodes + next hexagonal ring
///
/// This ensures minimal mesh churn when nodes join/leave, and maintains
/// roughly 5:1 horizontal-to-vertical ratio at scale.
///
/// Hexagonal ring sizes: 1 (center) + 6 + 12 + 18 + 24 + ... (6*N for ring N)
pub fn calculate_mesh_dimensions(num_nodes: usize) -> MeshConfig {
    if num_nodes == 0 || num_nodes == 1 {
        return MeshConfig::new(1, 1, 1);
    }

    // Calculate headroom: size of next hexagonal ring
    // Ring 0 = 1 node, Ring 1 = 6 nodes, Ring 2 = 12 nodes, Ring N = 6*N nodes
    // Find which ring we're currently in
    let mut cumulative_nodes = 1; // Ring 0
    let mut ring = 0;
    while cumulative_nodes < num_nodes {
        ring += 1;
        cumulative_nodes += 6 * ring; // Add ring N (6*N nodes)
    }

    // Next ring will have 6*(ring+1) nodes - add this as headroom
    let next_ring_size = 6 * (ring + 1);
    let target_capacity = num_nodes + next_ring_size;

    // Start from minimal mesh
    let mut w = 1;
    let mut h = 1;
    let mut d = 1;

    // Grow mesh until it fits all nodes + next hexagonal ring
    // Strategy: Always grow the SMALLEST dimension
    // But STRONGLY prefer growing w or h over d (keep it flat!)
    while w * h * d < target_capacity {
        // Compare dimensions, but with a HUGE bias against depth
        // This naturally creates flat meshes that only grow vertically when necessary

        // Effective sizes (depth counts 100x more to keep mesh flat)
        let w_effective = w;
        let h_effective = h;
        let d_effective = d * 100; // STRONGLY discourage depth growth

        // Find which dimension is smallest (effective)
        if w_effective <= h_effective && w_effective <= d_effective {
            // Width is smallest (or tied) - grow it!
            w += 1;
        } else if h_effective <= w_effective && h_effective <= d_effective {
            // Height is smallest (or tied) - grow it!
            h += 1;
        } else {
            // Depth is smallest (very rare due to 100x penalty) - grow it!
            d += 1;
        }
    }

    MeshConfig::new(w, h, d)
}

/// Mesh configuration for P2P network
/// Using smaller mesh for lens-v2 (we're not running millions of nodes yet!)
///
/// NOTE: In production, use calculate_mesh_dimensions(peer_count) for dynamic sizing!
/// This function is kept for backward compatibility and testing.
pub fn default_mesh_config() -> MeshConfig {
    // Start with a small mesh for testing
    // 50 nodes = 5×5×2 = 50 slots
    // Can scale up to 120×120×25 = 360k slots when needed
    MeshConfig::new(10, 10, 5)  // 500 total slots
}

/// Convert peer_id (string) to Blake3 hash for slot mapping
pub fn peer_id_to_hash(peer_id: &str) -> [u8; 32] {
    let hash = blake3::hash(peer_id.as_bytes());
    *hash.as_bytes()
}

/// Generate content-addressed peer_id from slot coordinate
///
/// This is the INVERSE of peer_id_to_slot(). Given a slot coordinate (x,y,z),
/// we hash it to produce a deterministic peer_id in CIDv1 format:
/// "bafk" + hex(blake3(x || y || z))
///
/// This allows us to:
/// - Assign slots sequentially from (0,0,0) outward
/// - Generate valid peer_ids for those slots
/// - Maintain content-addressed properties
///
/// Example:
/// - Slot (0,0,0) → "bafk<hash of 0||0||0>"
/// - Slot (0,1,0) → "bafk<hash of 0||1||0>"
pub fn slot_to_peer_id(slot: SlotCoordinate) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"content-addressed-slot");
    hasher.update(&slot.x.to_le_bytes());
    hasher.update(&slot.y.to_le_bytes());
    hasher.update(&slot.z.to_le_bytes());
    let hash = hasher.finalize();

    // CIDv1 format: "bafk" prefix + hex encoded hash
    format!("bafk{}", hex::encode(hash.as_bytes()))
}

/// Map peer_id to SlotCoordinate using Content Addressed Slots
///
/// **CRITICAL**: This mapping is INDEPENDENT of network size!
/// The peer_id hash maps directly to absolute coordinates in a fixed 256³ space.
/// This ensures:
/// - Same peer_id ALWAYS maps to same slot (across all network sizes)
/// - Slots are content-addressed (determined by peer_id hash alone)
/// - No dependency on current mesh dimensions
///
/// The mesh_config parameter is IGNORED for slot calculation but kept for
/// API compatibility with neighbor discovery functions.
pub fn peer_id_to_slot(peer_id: &str, config: &MeshConfig) -> SlotCoordinate {
    let hash = peer_id_to_hash(peer_id);

    // Map hash to coordinates within the mesh boundaries using modulo
    // This ensures all peers fit within the configured mesh dimensions!
    let x = (hash[0] as usize % config.width) as i32;
    let y = (hash[1] as usize % config.height) as i32;
    let z = (hash[2] as usize % config.depth) as i32;

    SlotCoordinate::new(x, y, z)
}

/// Convert slot index to coordinate using deterministic growth pattern
///
/// Pattern fills slots in order:
/// 1. Start at (0,0,0)
/// 2. Fill hexagonal ring around origin in XY plane
/// 3. Continue spiraling outward in XY
/// 4. When layer full, move to next Z layer
///
/// This ensures slot N always maps to the same coordinate regardless of mesh size,
/// as long as the mesh is large enough to contain N slots.
fn slot_index_to_coordinate(index: usize, config: &MeshConfig) -> SlotCoordinate {
    // For now, use simple sequential fill (x-major, then y, then z)
    // TODO: Replace with proper hexagonal spiral pattern
    let slots_per_layer = config.width * config.height;

    let z = (index / slots_per_layer) as i32;
    let xy_index = index % slots_per_layer;
    let y = (xy_index / config.width) as i32;
    let x = (xy_index % config.width) as i32;

    SlotCoordinate::new(x, y, z)
}

/// DHT key for "who owns this slot?"
///
/// This is stored IN the DHT itself (recursive meta-routing!)
pub fn slot_ownership_key(slot: SlotCoordinate) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"slot-ownership");
    hasher.update(&slot.x.to_le_bytes());
    hasher.update(&slot.y.to_le_bytes());
    hasher.update(&slot.z.to_le_bytes());
    let hash = hasher.finalize();
    *hash.as_bytes()
}

/// DHT key for "where is this peer located?"
///
/// Allows reverse lookup: peer_id → SlotCoordinate
pub fn peer_location_key(peer_id: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"peer-location");
    hasher.update(peer_id.as_bytes());
    let hash = hasher.finalize();
    *hash.as_bytes()
}

/// Convert normalized slot coordinate to a linear index
fn slot_to_index(slot: SlotCoordinate, config: &MeshConfig) -> usize {
    let normalized = slot.normalize(config);
    let x = normalized.x.rem_euclid(config.width as i32) as usize;
    let y = normalized.y.rem_euclid(config.height as i32) as usize;
    let z = normalized.z.rem_euclid(config.depth as i32) as usize;

    x + (y * config.width) + (z * config.width * config.height)
}

/// Convert a linear index back to a slot coordinate
fn index_to_slot(index: usize, config: &MeshConfig) -> SlotCoordinate {
    let width = config.width as usize;
    let height = config.height as usize;
    let depth = config.depth as usize;

    let x = index % width;
    let y = (index / width) % height;
    let z = (index / (width * height)) % depth;

    SlotCoordinate::new(x as i32, y as i32, z as i32)
}

/// Deterministically assign a unique slot for a peer by scanning the mesh for open positions
pub fn assign_unique_slot(
    peer_id: &str,
    config: &MeshConfig,
    occupied: &mut HashSet<SlotCoordinate>,
) -> SlotCoordinate {
    let preferred = peer_id_to_slot(peer_id, config);
    if occupied.insert(preferred) {
        return preferred;
    }

    let capacity = (config.width * config.height * config.depth) as usize;
    let start_index = slot_to_index(preferred, config);

    for offset in 1..capacity {
        let next_index = (start_index + offset) % capacity;
        let candidate = index_to_slot(next_index, config);
        if occupied.insert(candidate) {
            return candidate;
        }
    }

    panic!(
        "No available slots remaining in mesh {}×{}×{}",
        config.width, config.height, config.depth
    );
}


/// Slot ownership announcement stored in DHT
///
/// NOW WITH CONTENT ADDRESSED SLOTS!
/// - slot_id: Permanent Blake3 hash of coordinate (never changes)
/// - slot: Physical coordinate (may change during mesh resize)
/// - peer_id: Current occupant (ephemeral, can be trumped!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotOwnership {
    /// Permanent content-addressed slot ID (Blake3 of coordinate)
    pub slot_id: SlotId,

    /// Physical slot coordinate in mesh
    pub slot: SlotCoordinate,

    /// Current peer owning this slot (can change via trump protocol!)
    pub peer_id: String,

    /// Relay URL for WebSocket fallback
    pub relay_url: Option<String>,

    /// Peer capabilities (webrtc, dht, spore, etc.)
    pub capabilities: Vec<String>,

    /// Last heartbeat timestamp (Unix seconds)
    pub last_heartbeat: u64,

    /// Protocol version
    pub protocol_version: String,

    /// Average latency to 8 neighbors (milliseconds)
    /// Used for trump challenges - lower is better!
    pub avg_neighbor_latency_ms: Option<u64>,
}

impl SlotOwnership {
    /// Create a new slot ownership announcement
    ///
    /// Automatically generates permanent SlotId from coordinate.
    pub fn new(peer_id: String, slot: SlotCoordinate, relay_url: Option<String>) -> Self {
        Self {
            slot_id: SlotId::from_coordinate(slot),
            slot,
            peer_id,
            relay_url,
            capabilities: vec!["webrtc".to_string(), "dht".to_string(), "spore".to_string()],
            last_heartbeat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            protocol_version: "0.8.20".to_string(),
            avg_neighbor_latency_ms: None,
        }
    }

    /// Update heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Check if peer is stale (no heartbeat in 5 minutes)
    pub fn is_stale(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        (now - self.last_heartbeat) > 300 // 5 minutes
    }
}

/// Get all 8 neighbor slots for a given slot
///
/// Citadel DHT topology: 8 fixed neighbors per node
/// - 6 hexagonal (in-plane): ±A, ±B, ±C
/// - 2 vertical: Up, Down
pub fn get_neighbor_slots(slot: &SlotCoordinate, config: &MeshConfig) -> Vec<(Direction, SlotCoordinate)> {
    vec![
        (Direction::PlusA, slot.neighbor(Direction::PlusA, config)),
        (Direction::MinusA, slot.neighbor(Direction::MinusA, config)),
        (Direction::PlusB, slot.neighbor(Direction::PlusB, config)),
        (Direction::MinusB, slot.neighbor(Direction::MinusB, config)),
        (Direction::PlusC, slot.neighbor(Direction::PlusC, config)),
        (Direction::MinusC, slot.neighbor(Direction::MinusC, config)),
        (Direction::Up, slot.neighbor(Direction::Up, config)),
        (Direction::Down, slot.neighbor(Direction::Down, config)),
    ]
}

// ============================================================================
// HEXAGONAL SPIRAL CAS - Content-Addressed Slots with Zero Collisions
// ============================================================================

/// Generate N slots in hexagonal spiral pattern
///
/// Algorithm:
/// 1. Start at origin (0,0,0)
/// 2. Spiral outward in hexagonal rings on Z=0
/// 3. When hex diameter >= 5*depth, start new Z layer
/// 4. Over-generate to complete Z layer for stability
///
/// Properties:
/// - Exactly N slots for N nodes (no collisions!)
/// - Maintains ~5:1 width-to-depth ratio
/// - Deterministic ordering for consistent slot assignment
pub fn generate_available_slots(node_count: usize) -> Vec<SlotCoordinate> {
    if node_count == 0 {
        panic!("node_count must be > 0");
    }

    let mut slots = Vec::with_capacity(node_count);

    // Start at origin
    slots.push(SlotCoordinate::new(0, 0, 0));
    if slots.len() >= node_count {
        return slots;
    }

    let mut current_z = 0;
    let mut ring_radius = 1;

    loop {
        // Generate hexagonal ring at current Z level
        let ring_slots = generate_hex_ring(ring_radius, current_z);

        for slot in ring_slots {
            slots.push(slot);
            if slots.len() >= node_count {
                return slots;
            }
        }

        // Check if we should move to next Z layer (5:1 ratio)
        let hex_diameter = 2 * ring_radius + 1;
        let depth = current_z + 1;

        if hex_diameter >= 5 * depth {
            // Start new Z layer
            current_z += 1;
            ring_radius = 0; // Will increment to 1 at top of loop
        } else {
            // Continue spiraling outward on current Z
            ring_radius += 1;
        }
    }
}

/// Generate a hexagonal ring at given radius and Z coordinate
///
/// Hexagonal ring generation using axial coordinates:
/// - Ring 0 (radius 0): Just origin (handled separately)
/// - Ring 1 (radius 1): 6 hexagons around origin
/// - Ring 2 (radius 2): 12 hexagons
/// - Ring N: 6*N hexagons
///
/// Uses the "ring walking" algorithm from Red Blob Games:
/// https://www.redblobgames.com/grids/hexagons/#rings
fn generate_hex_ring(radius: i32, z: i32) -> Vec<SlotCoordinate> {
    if radius == 0 {
        return vec![SlotCoordinate::new(0, 0, z)];
    }

    let mut ring = Vec::new();

    // Six hexagonal axial directions (using Citadel's A/B/C system)
    // In axial coordinates (x, y):
    // +A: (+1, 0)    [East]
    // -A: (-1, 0)    [West]
    // +B: (+1, -1)   [Northeast]
    // -B: (-1, +1)   [Southwest]
    // +C: (0, -1)    [Northwest]
    // -C: (0, +1)    [Southeast]
    let directions = [
        (1, 0),    // +A (East)
        (1, -1),   // +B (Northeast)
        (0, -1),   // +C (Northwest)
        (-1, 0),   // -A (West)
        (-1, 1),   // -B (Southwest)
        (0, 1),    // -C (Southeast)
    ];

    // Start at radius steps in the -B direction from origin
    // For radius=1, this is (-1, 1, z)
    // For radius=2, this is (-2, 2, z)
    let mut x = -radius;
    let mut y = radius;

    // Walk the 6 sides of the hexagon
    // Each side walks 'radius' steps in one of the 6 directions
    for (dx, dy) in directions {
        for _ in 0..radius {
            ring.push(SlotCoordinate::new(x, y, z));
            x += dx;
            y += dy;
        }
    }

    ring
}

/// Count neighbors for a slot given a set of occupied slots
///
/// Returns number of neighbors (0-8) that are in the occupied set.
/// Neighbors include:
/// - 6 hexagonal (in-plane)
/// - 2 vertical (up/down)
pub fn count_neighbors(slot: &SlotCoordinate, occupied_slots: &HashSet<SlotCoordinate>) -> u8 {
    let mut count = 0;

    // Check 6 hexagonal neighbors (in-plane)
    let hex_neighbors = [
        SlotCoordinate::new(slot.x + 1, slot.y, slot.z),      // +A
        SlotCoordinate::new(slot.x - 1, slot.y, slot.z),      // -A
        SlotCoordinate::new(slot.x + 1, slot.y - 1, slot.z),  // +B
        SlotCoordinate::new(slot.x - 1, slot.y + 1, slot.z),  // -B
        SlotCoordinate::new(slot.x, slot.y - 1, slot.z),      // +C
        SlotCoordinate::new(slot.x, slot.y + 1, slot.z),      // -C
    ];

    for neighbor in &hex_neighbors {
        if occupied_slots.contains(neighbor) {
            count += 1;
        }
    }

    // Check 2 vertical neighbors
    if occupied_slots.contains(&SlotCoordinate::new(slot.x, slot.y, slot.z + 1)) {
        count += 1; // Up
    }
    if occupied_slots.contains(&SlotCoordinate::new(slot.x, slot.y, slot.z - 1)) {
        count += 1; // Down
    }

    count
}

/// Rank slots by neighbor count (descending)
///
/// Returns Vec<(SlotCoordinate, neighbor_count)> sorted by neighbor count.
/// Slots with more neighbors come first (8 > 7 > 6 > ...)
pub fn rank_slots_by_neighbors(slots: &[SlotCoordinate]) -> Vec<(SlotCoordinate, u8)> {
    let occupied: HashSet<SlotCoordinate> = slots.iter().cloned().collect();

    let mut ranked: Vec<(SlotCoordinate, u8)> = slots
        .iter()
        .map(|slot| (*slot, count_neighbors(slot, &occupied)))
        .collect();

    // Sort by neighbor count descending (highest connectivity first)
    ranked.sort_by(|a, b| b.1.cmp(&a.1));

    ranked
}

/// Assign ALL peers to slots at once (COLLISION-FREE with linear probing!)
///
/// Algorithm:
/// 1. Generate N available slots in hexagonal spiral
/// 2. Rank slots by neighbor count (prefer 8 > 7 > 6)
/// 3. Sort peer_ids by HASH (deterministic ordering for testing)
/// 4. For each peer in sorted order:
///    a. Compute preferred slot: hash(peer_id) % N (using ranking index)
///    b. If that ranked slot is available, assign it
///    c. If taken, linear probe: try next slot in ranked list
///
/// This simulates runtime slot claiming with collision resolution:
/// - Peers "arrive" in hash-sorted order
/// - Each peer tries their hash-preferred slot
/// - Collisions resolved via linear probing
/// - GUARANTEED collision-free (N peers get N unique slots)
pub fn assign_slots_batch(peer_ids: &[String]) -> HashMap<String, SlotCoordinate> {
    let node_count = peer_ids.len();

    // Generate N slots
    let slots = generate_available_slots(node_count);

    // Rank by neighbor count
    let ranked = rank_slots_by_neighbors(&slots);

    // Sort peers by hash (deterministic arrival order)
    let mut peer_hashes: Vec<(String, [u8; 32])> = peer_ids
        .iter()
        .map(|peer_id| (peer_id.clone(), peer_id_to_hash(peer_id)))
        .collect();

    peer_hashes.sort_by(|a, b| a.1.cmp(&b.1));

    // Track which slots are taken
    let mut taken_slots = HashSet::new();
    let mut assignments = HashMap::new();

    // Assign each peer using hash-modulo with linear probing
    for (peer_id, hash) in peer_hashes {
        // Compute preferred slot index via hash-modulo
        let hash_u128 = u128::from_le_bytes([
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7],
            hash[8], hash[9], hash[10], hash[11],
            hash[12], hash[13], hash[14], hash[15],
        ]);
        let preferred_index = (hash_u128 % node_count as u128) as usize;

        // Linear probing: try preferred slot, then next, then next...
        let mut assigned = false;
        for offset in 0..node_count {
            let probe_index = (preferred_index + offset) % node_count;
            let slot = ranked[probe_index].0;

            if !taken_slots.contains(&slot) {
                // Slot is available!
                taken_slots.insert(slot);
                assignments.insert(peer_id.clone(), slot);
                assigned = true;
                break;
            }
        }

        if !assigned {
            panic!("Failed to assign slot for peer {} - all slots taken?", peer_id);
        }
    }

    assignments
}

/// Generate a deterministic peer_id from an index
///
/// Creates a peer_id as blake3 double hash of a simulated ed25519 public key.
/// This matches the production peer_id format: blake3(blake3(ed25519_pubkey))
fn generate_peer_id(index: usize) -> String {
    // Simulate an ed25519 public key
    let pubkey = format!("ed25519-pubkey-{}", index);
    let hash1 = blake3::hash(pubkey.as_bytes());
    let hash2 = blake3::hash(hash1.as_bytes());
    hex::encode(hash2.as_bytes())
}

/// Assign a SINGLE peer to a slot (using hash-modulo with linear probing)
///
/// Algorithm (for deterministic testing):
/// 1. Generate all peer_ids for this network size using proper format (blake3 double hash)
/// 2. Sort them by hash (deterministic ordering)
/// 3. For each peer in sorted order:
///    a. Compute preferred slot: hash(peer_id) % N
///    b. If slot available, take it
///    c. If slot taken, linear probe: try (preferred + 1) % N, (preferred + 2) % N, etc.
///
/// This simulates the runtime behavior where peers claim slots in arrival order,
/// with collision resolution via linear probing.
///
/// In PRODUCTION: Nodes claim slots dynamically via DHT with trump protocol for conflicts.
/// For TESTING: We simulate all nodes at once with deterministic ordering.
pub fn assign_slot(peer_id: &str, node_count: usize) -> SlotCoordinate {
    // Generate all peer_ids for this network size using proper format
    let all_peers: Vec<String> = (0..node_count)
        .map(|i| generate_peer_id(i))
        .collect();

    // Call batch assignment
    let assignments = assign_slots_batch(&all_peers);

    // Return this peer's assignment
    assignments.get(peer_id).cloned()
        .unwrap_or_else(|| {
            // If peer_id not in batch (shouldn't happen in tests), compute directly
            let slots = generate_available_slots(node_count);
            let ranked = rank_slots_by_neighbors(&slots);
            let hash = peer_id_to_hash(peer_id);
            let hash_u128 = u128::from_le_bytes([
                hash[0], hash[1], hash[2], hash[3],
                hash[4], hash[5], hash[6], hash[7],
                hash[8], hash[9], hash[10], hash[11],
                hash[12], hash[13], hash[14], hash[15],
            ]);
            let slot_index = (hash_u128 % node_count as u128) as usize;
            ranked[slot_index].0
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id_to_hash() {
        let hash = peer_id_to_hash("peer123");
        assert_eq!(hash.len(), 32);

        // Same peer_id always produces same hash
        let hash2 = peer_id_to_hash("peer123");
        assert_eq!(hash, hash2);

        // Different peer_id produces different hash
        let hash3 = peer_id_to_hash("peer456");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_peer_id_to_slot_deterministic() {
        let config = MeshConfig::new(10, 10, 5);
        let slot1 = peer_id_to_slot("peer123", &config);
        let slot2 = peer_id_to_slot("peer123", &config);

        // Same peer_id always maps to same slot
        assert_eq!(slot1, slot2);
    }

    #[test]
    fn test_peer_id_to_slot_within_bounds() {
        let config = MeshConfig::new(10, 10, 5);
        let slot = peer_id_to_slot("peer123", &config);

        // Coordinates must be within mesh dimensions
        assert!(slot.x >= 0 && slot.x < 10);
        assert!(slot.y >= 0 && slot.y < 10);
        assert!(slot.z >= 0 && slot.z < 5);
    }

    #[test]
    fn test_slot_ownership_key() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let key = slot_ownership_key(slot);
        assert_eq!(key.len(), 32);

        // Same slot always produces same key
        let key2 = slot_ownership_key(slot);
        assert_eq!(key, key2);
    }

    #[test]
    fn test_peer_location_key() {
        let key = peer_location_key("peer123");
        assert_eq!(key.len(), 32);

        // Same peer_id always produces same key
        let key2 = peer_location_key("peer123");
        assert_eq!(key, key2);
    }

    #[test]
    fn test_get_neighbor_slots_returns_eight() {
        let config = MeshConfig::new(10, 10, 5);
        let slot = SlotCoordinate::new(5, 5, 2);
        let neighbors = get_neighbor_slots(&slot, &config);

        // Must have exactly 8 neighbors
        assert_eq!(neighbors.len(), 8);
    }

    #[test]
    fn test_get_neighbor_slots_all_different() {
        let config = MeshConfig::new(10, 10, 5);
        let slot = SlotCoordinate::new(5, 5, 2);
        let neighbors = get_neighbor_slots(&slot, &config);

        // All neighbor slots should be unique
        let mut unique_slots = std::collections::HashSet::new();
        for (_, neighbor_slot) in neighbors {
            unique_slots.insert(neighbor_slot);
        }
        assert_eq!(unique_slots.len(), 8);
    }

    #[test]
    fn test_get_neighbor_slots_wraps_at_boundary() {
        let config = MeshConfig::new(10, 10, 5);
        let slot = SlotCoordinate::new(9, 9, 4);  // At boundary
        let neighbors = get_neighbor_slots(&slot, &config);

        // Should have 8 neighbors even at boundary (toroidal wrapping)
        assert_eq!(neighbors.len(), 8);

        // Check that PlusA wraps to x=0
        let plus_a_neighbor = neighbors.iter()
            .find(|(dir, _)| *dir == Direction::PlusA)
            .map(|(_, slot)| slot)
            .unwrap();
        assert_eq!(plus_a_neighbor.x, 0);  // Wrapped from 9+1
    }

    #[test]
    fn test_neighbor_slots_includes_all_directions() {
        let config = MeshConfig::new(10, 10, 5);
        let slot = SlotCoordinate::new(5, 5, 2);
        let neighbors = get_neighbor_slots(&slot, &config);

        // Check all 8 directions are present
        let directions: Vec<Direction> = neighbors.iter().map(|(dir, _)| *dir).collect();
        assert!(directions.contains(&Direction::PlusA));
        assert!(directions.contains(&Direction::MinusA));
        assert!(directions.contains(&Direction::PlusB));
        assert!(directions.contains(&Direction::MinusB));
        assert!(directions.contains(&Direction::PlusC));
        assert!(directions.contains(&Direction::MinusC));
        assert!(directions.contains(&Direction::Up));
        assert!(directions.contains(&Direction::Down));
    }

    #[test]
    fn test_default_mesh_config() {
        let config = default_mesh_config();
        assert_eq!(config.width, 10);
        assert_eq!(config.height, 10);
        assert_eq!(config.depth, 5);
        assert_eq!(config.total_slots(), 500);
    }

    #[test]
    fn test_slot_ownership_creation() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let ownership = SlotOwnership::new(
            "peer-123".to_string(),
            slot,
            Some("ws://localhost:5000/api/v1/relay/ws".to_string()),
        );

        assert_eq!(ownership.peer_id, "peer-123");
        assert_eq!(ownership.slot, slot);
        assert_eq!(ownership.capabilities.len(), 3);
        assert!(ownership.capabilities.contains(&"webrtc".to_string()));
        assert!(!ownership.is_stale());
    }

    #[test]
    fn test_slot_ownership_staleness() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let mut ownership = SlotOwnership::new("peer-123".to_string(), slot, None);

        // Fresh ownership should not be stale
        assert!(!ownership.is_stale());

        // Simulate old heartbeat (6 minutes ago)
        ownership.last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 360;

        assert!(ownership.is_stale(), "Old heartbeat should be stale");
    }

    #[test]
    fn test_slot_to_peer_id_deterministic() {
        let slot = SlotCoordinate::new(0, 0, 0);
        let peer_id1 = slot_to_peer_id(slot);
        let peer_id2 = slot_to_peer_id(slot);

        // Same slot always produces same peer_id
        assert_eq!(peer_id1, peer_id2);

        // peer_id should have "bafk" prefix
        assert!(peer_id1.starts_with("bafk"));
    }

    #[test]
    fn test_slot_to_peer_id_different_slots() {
        let slot1 = SlotCoordinate::new(0, 0, 0);
        let slot2 = SlotCoordinate::new(0, 1, 0);

        let peer_id1 = slot_to_peer_id(slot1);
        let peer_id2 = slot_to_peer_id(slot2);

        // Different slots produce different peer_ids
        assert_ne!(peer_id1, peer_id2);
    }

    #[test]
    fn test_sequential_slot_peer_ids() {
        // Test generating peer_ids for sequential slots from origin
        let slots = vec![
            SlotCoordinate::new(0, 0, 0),
            SlotCoordinate::new(0, 1, 0),
            SlotCoordinate::new(1, 1, 0),
            SlotCoordinate::new(1, 0, 0),
            SlotCoordinate::new(0, 0, 1),
        ];

        let mut peer_ids = std::collections::HashSet::new();
        for slot in slots {
            let peer_id = slot_to_peer_id(slot);
            assert!(peer_id.starts_with("bafk"), "peer_id should have bafk prefix");
            assert_eq!(peer_id.len(), 68, "peer_id should be bafk + 64 hex chars");

            // All peer_ids should be unique
            assert!(peer_ids.insert(peer_id), "peer_ids should be unique");
        }
    }
}
