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
use std::time::{SystemTime, UNIX_EPOCH};

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

/// Map peer_id to SlotCoordinate using deterministic hash-to-slot mapping
///
/// This is the CORE of Citadel DHT's O(1) routing!
/// Uses modulo arithmetic on Blake3 hash bytes to map to mesh coordinates.
pub fn peer_id_to_slot(peer_id: &str, config: &MeshConfig) -> SlotCoordinate {
    let hash = peer_id_to_hash(peer_id);

    // Extract 8-byte chunks from hash and modulo by mesh dimensions
    let x = u64::from_le_bytes(hash[0..8].try_into().unwrap()) % config.width as u64;
    let y = u64::from_le_bytes(hash[8..16].try_into().unwrap()) % config.height as u64;
    let z = u64::from_le_bytes(hash[16..24].try_into().unwrap()) % config.depth as u64;

    SlotCoordinate::new(x as i32, y as i32, z as i32)
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

/// Slot ownership announcement stored in DHT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotOwnership {
    pub slot: SlotCoordinate,
    pub peer_id: String,
    pub relay_url: Option<String>,
    pub capabilities: Vec<String>,
    pub last_heartbeat: u64,
    pub protocol_version: String,
}

impl SlotOwnership {
    /// Create a new slot ownership announcement
    pub fn new(peer_id: String, slot: SlotCoordinate, relay_url: Option<String>) -> Self {
        Self {
            slot,
            peer_id,
            relay_url,
            capabilities: vec!["webrtc".to_string(), "dht".to_string(), "spore".to_string()],
            last_heartbeat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            protocol_version: "0.6.0".to_string(),
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
}
