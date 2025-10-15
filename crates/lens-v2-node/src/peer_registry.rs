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
use std::collections::HashSet;
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
///
/// This ensures minimal mesh churn when nodes join/leave, and maintains
/// roughly 5:1 horizontal-to-vertical ratio at scale.
pub fn calculate_mesh_dimensions(num_nodes: usize) -> MeshConfig {
    if num_nodes == 0 || num_nodes == 1 {
        return MeshConfig::new(1, 1, 1);
    }

    // Start from minimal mesh
    let mut w = 1;
    let mut h = 1;
    let mut d = 1;

    // Grow mesh until it fits all nodes
    // Strategy: Always grow the SMALLEST dimension
    // But STRONGLY prefer growing w or h over d (keep it flat!)
    while w * h * d < num_nodes {
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
pub fn peer_id_to_slot(peer_id: &str, _config: &MeshConfig) -> SlotCoordinate {
    let hash = peer_id_to_hash(peer_id);

    // Map hash directly to coordinates in FIXED 256×256×256 space
    // This is INDEPENDENT of network size - true Content Addressed Slots!
    let x = hash[0] as i32;  // 0-255
    let y = hash[1] as i32;  // 0-255
    let z = hash[2] as i32;  // 0-255

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
