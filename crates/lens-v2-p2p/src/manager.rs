//! P2P Manager coordinating BFT consensus and TGP block exchange

use crate::{BlockId, BlockMeta, P2pError, PeerId, Result, SyncStatus};
use crate::sync::SyncTracker;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;

#[cfg(target_arch = "wasm32")]
use std::sync::Mutex as RwLock; // WASM single-threaded

#[cfg(feature = "dht")]
use citadel_core::topology::{Direction, MeshConfig, SlotCoordinate};
#[cfg(feature = "dht")]
use citadel_core::routing::greedy_direction;

/// Configuration for P2P manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    /// Local node ID
    pub node_id: PeerId,

    /// Total number of nodes in BFT network
    pub n: usize,

    /// Maximum number of Byzantine faults tolerated
    pub f: usize,

    /// View timeout in milliseconds
    pub view_timeout_ms: u64,

    /// Maximum transmission unit for TGP
    pub mtu: usize,

    /// DHT mesh configuration (optional, enables DHT routing)
    #[cfg(feature = "dht")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh_config: Option<MeshConfig>,

    /// This node's slot coordinate in the DHT mesh (optional)
    #[cfg(feature = "dht")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_coordinate: Option<SlotCoordinate>,
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            node_id: 0,
            n: 4,           // 4 nodes
            f: 1,           // Tolerate 1 Byzantine fault
            view_timeout_ms: 5000,
            mtu: 1024 * 64, // 64 KB chunks
            #[cfg(feature = "dht")]
            mesh_config: Some(MeshConfig::new(10, 10, 5)), // Default to 500 slots for development
            #[cfg(feature = "dht")]
            slot_coordinate: None, // Must be assigned at runtime
        }
    }
}

/// P2P Manager
///
/// Coordinates BFT consensus for agreement on block set
/// and TGP for efficient block exchange.
pub struct P2pManager {
    config: P2pConfig,
    sync: Arc<RwLock<SyncTracker>>,
}

impl P2pManager {
    /// Create a new P2P manager
    pub fn new(config: P2pConfig) -> Self {
        Self {
            config,
            sync: Arc::new(RwLock::new(SyncTracker::new())),
        }
    }

    /// Get current sync status
    pub fn sync_status(&self) -> Result<SyncStatus> {
        #[cfg(not(target_arch = "wasm32"))]
        let status = self.sync.read()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .status();

        #[cfg(target_arch = "wasm32")]
        let status = self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .status();

        Ok(status)
    }

    /// Get all known peer IDs as strings (for /map endpoint)
    pub fn get_known_peer_strings(&self) -> Result<Vec<String>> {
        #[cfg(not(target_arch = "wasm32"))]
        let peers = self.sync.read()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .get_known_peer_strings();

        #[cfg(target_arch = "wasm32")]
        let peers = self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .get_known_peer_strings();

        Ok(peers)
    }

    /// Get all ALIVE peer IDs as strings (for /map endpoint)
    pub fn get_alive_peer_strings(&self) -> Result<Vec<String>> {
        #[cfg(not(target_arch = "wasm32"))]
        let peers = self.sync.read()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .get_alive_peer_strings();

        #[cfg(target_arch = "wasm32")]
        let peers = self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .get_alive_peer_strings();

        Ok(peers)
    }

    /// Mark peer as alive (received heartbeat)
    pub fn mark_peer_alive(&self, peer_id: PeerId) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .mark_peer_alive(peer_id);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .mark_peer_alive(peer_id);

        Ok(())
    }

    /// Clear all alive peers (start new heartbeat cycle)
    pub fn clear_alive_peers(&self) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .clear_alive_peers();

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .clear_alive_peers();

        Ok(())
    }

    /// Get all known peer IDs as u64 (for internal use)
    pub fn get_known_peers(&self) -> Result<Vec<PeerId>> {
        #[cfg(not(target_arch = "wasm32"))]
        let peers = self.sync.read()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .get_known_peers();

        #[cfg(target_arch = "wasm32")]
        let peers = self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .get_known_peers();

        Ok(peers)
    }

    /// Add a known peer (for awareness) - O(n) scalability
    pub fn add_known_peer(&self, peer_id: PeerId) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_known_peer(peer_id);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_known_peer(peer_id);

        Ok(())
    }

    /// Add a known peer with original string ID (for /map endpoint)
    pub fn add_known_peer_with_string(&self, peer_id: PeerId, peer_id_string: String) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_known_peer_with_string(peer_id, peer_id_string);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_known_peer_with_string(peer_id, peer_id_string);

        Ok(())
    }

    /// Add a connected peer (actual P2P connection to mesh neighbor) - O(1) scalability
    pub fn add_connected_peer(&self, peer_id: PeerId, peer_type: Option<crate::sync::TrackedPeerType>) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_connected_peer(peer_id, peer_type);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_connected_peer(peer_id, peer_type);

        Ok(())
    }

    /// Add a peer to the network with optional type (defaults to Server if not specified)
    /// DEPRECATED: Use add_connected_peer() for mesh neighbors or add_known_peer() for awareness
    pub fn add_peer(&self, peer_id: PeerId, peer_type: Option<crate::sync::TrackedPeerType>) -> Result<()> {
        self.add_connected_peer(peer_id, peer_type)
    }

    /// Remove a peer from the network
    pub fn remove_peer(&self, peer_id: &PeerId) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .remove_peer(peer_id);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .remove_peer(peer_id);

        Ok(())
    }

    /// Update consensus view of blocks
    pub fn update_consensus(&self, blocks: Vec<BlockMeta>) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .update_consensus(blocks);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .update_consensus(blocks);

        Ok(())
    }

    /// Add a local block
    pub fn add_local_block(&self, block: BlockMeta) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_local_block(block);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_local_block(block);

        Ok(())
    }

    /// Get missing blocks
    pub fn missing_blocks(&self) -> Result<Vec<BlockId>> {
        #[cfg(not(target_arch = "wasm32"))]
        let missing = self.sync.read()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .missing_blocks();

        #[cfg(target_arch = "wasm32")]
        let missing = self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .missing_blocks();

        Ok(missing)
    }

    /// Mark a block as being downloaded
    pub fn mark_downloading(&self, block_id: BlockId) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .mark_downloading(block_id);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .mark_downloading(block_id);

        Ok(())
    }

    /// Mark a block as downloaded
    pub fn mark_downloaded(&self, block_id: &BlockId) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .mark_downloaded(block_id);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .mark_downloaded(block_id);

        Ok(())
    }

    /// Calculate DHT slot coordinate for a given key (block ID or peer ID)
    ///
    /// Uses modulo mapping to determine which slot in the mesh should store
    /// this key. This enables O(1) key lookups when routing through the mesh.
    #[cfg(feature = "dht")]
    pub fn key_to_slot(&self, key: &str) -> Option<SlotCoordinate> {
        let mesh_config = self.config.mesh_config.as_ref()?;

        // Hash the key to get a deterministic slot index
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let total_slots = mesh_config.total_slots() as u64;
        let slot_index = (hash % total_slots) as usize;

        // Convert flat index to 3D coordinates
        let z = slot_index / (mesh_config.width * mesh_config.height);
        let remainder = slot_index % (mesh_config.width * mesh_config.height);
        let y = remainder / mesh_config.width;
        let x = remainder % mesh_config.width;

        Some(SlotCoordinate::new(x as i32, y as i32, z as i32))
    }

    /// Calculate greedy routing direction from our slot to the target key's slot
    ///
    /// Returns None if:
    /// - DHT is not enabled
    /// - Our slot coordinate is not set
    /// - We are already at the target slot
    ///
    /// This implements Citadel DHT's greedy routing algorithm which provides
    /// O(log n) routing with provably optimal paths.
    #[cfg(feature = "dht")]
    pub fn greedy_direction_for_key(&self, key: &str) -> Option<Direction> {
        let mesh_config = self.config.mesh_config.as_ref()?;
        let our_slot = self.config.slot_coordinate.as_ref()?;
        let target_slot = self.key_to_slot(key)?;

        greedy_direction(our_slot, &target_slot, mesh_config)
    }

    /// Find the next hop peer for routing to a given key
    ///
    /// Returns the peer ID that is in the greedy direction toward the target key.
    /// This enables DHT-aware block request routing:
    /// - If we have the block, serve it
    /// - If we don't, forward request to peer in greedy direction
    ///
    /// Returns None if:
    /// - DHT is not enabled
    /// - We are at the target slot (no forwarding needed)
    /// - No peer is known in the greedy direction
    #[cfg(feature = "dht")]
    pub fn next_hop_for_key(&self, key: &str, peer_slots: &std::collections::HashMap<PeerId, SlotCoordinate>) -> Option<PeerId> {
        let direction = self.greedy_direction_for_key(key)?;
        let mesh_config = self.config.mesh_config.as_ref()?;
        let our_slot = self.config.slot_coordinate.as_ref()?;

        // Calculate the neighboring slot in the greedy direction
        let next_slot = our_slot.neighbor(direction, mesh_config);

        // Find peer at that slot (if we know about them)
        peer_slots.iter()
            .find(|(_, slot)| **slot == next_slot)
            .map(|(peer_id, _)| *peer_id)
    }

    /// Get mesh configuration (if DHT is enabled)
    #[cfg(feature = "dht")]
    pub fn mesh_config(&self) -> Option<&MeshConfig> {
        self.config.mesh_config.as_ref()
    }

    /// Get our slot coordinate (if DHT is enabled and assigned)
    #[cfg(feature = "dht")]
    pub fn slot_coordinate(&self) -> Option<&SlotCoordinate> {
        self.config.slot_coordinate.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block(id: &str, height: u64) -> BlockMeta {
        BlockMeta {
            id: id.to_string(),
            height,
            prev: None,
            timestamp: 0,
        }
    }

    #[test]
    fn test_p2p_manager_new() {
        let config = P2pConfig::default();
        let manager = P2pManager::new(config);
        let status = manager.sync_status().unwrap();

        // Bootstrap case: 0 peers, 0 blocks behind → synced (we ARE the network)
        assert!(status.is_synced);
        assert_eq!(status.blocks_behind, 0);
        assert_eq!(status.known_peers, 0);
        assert_eq!(status.connected_peers, 0);
        assert_eq!(status.peer_count, 0);
    }

    #[test]
    fn test_add_peer() {
        let manager = P2pManager::new(P2pConfig::default());
        manager.add_peer(1, None).unwrap();
        manager.add_peer(2, None).unwrap();

        let status = manager.sync_status().unwrap();
        assert_eq!(status.peer_count, 2);
    }

    #[test]
    fn test_remove_peer() {
        let manager = P2pManager::new(P2pConfig::default());
        manager.add_peer(1, None).unwrap();
        manager.add_peer(2, None).unwrap();
        manager.remove_peer(&1).unwrap();

        let status = manager.sync_status().unwrap();
        assert_eq!(status.peer_count, 1);
    }

    #[test]
    fn test_update_consensus() {
        let manager = P2pManager::new(P2pConfig::default());

        manager.update_consensus(vec![
            make_block("block1", 1),
            make_block("block2", 2),
        ]).unwrap();

        let status = manager.sync_status().unwrap();
        assert_eq!(status.network_height, 2);
    }

    #[test]
    fn test_missing_blocks() {
        let manager = P2pManager::new(P2pConfig::default());

        // Add local block
        manager.add_local_block(make_block("block1", 1)).unwrap();

        // Update consensus
        manager.update_consensus(vec![
            make_block("block1", 1),
            make_block("block2", 2),
            make_block("block3", 3),
        ]).unwrap();

        // Should be missing block2 and block3
        let missing = manager.missing_blocks().unwrap();
        assert_eq!(missing.len(), 2);
    }

    #[test]
    fn test_sync_workflow() {
        let manager = P2pManager::new(P2pConfig::default());

        // Bootstrap case: Start with no peers, synced
        let status = manager.sync_status().unwrap();
        assert!(status.is_synced);

        // Add peer
        manager.add_peer(1, None).unwrap();

        // Add consensus blocks
        manager.update_consensus(vec![
            make_block("block1", 1),
            make_block("block2", 2),
        ]).unwrap();

        // Not synced yet (missing blocks)
        let status = manager.sync_status().unwrap();
        assert!(!status.is_synced);
        assert_eq!(status.blocks_behind, 2);

        // Download blocks
        manager.add_local_block(make_block("block1", 1)).unwrap();
        manager.add_local_block(make_block("block2", 2)).unwrap();

        // Now synced
        let status = manager.sync_status().unwrap();
        assert!(status.is_synced);
        assert_eq!(status.blocks_behind, 0);
    }

    #[test]
    fn test_known_peers_vs_connected_peers() {
        let manager = P2pManager::new(P2pConfig::default());

        // Add 49 known peers (O(n) awareness)
        for i in 1..=49 {
            manager.add_known_peer(i).unwrap();
        }

        // Add only 8 connected peers (O(1) mesh neighbors)
        for i in 1..=8 {
            manager.add_connected_peer(100 + i, None).unwrap();
        }

        let status = manager.sync_status().unwrap();

        // Should have 49 known peers
        assert_eq!(status.known_peers, 49 + 8, "Should know about all 57 peers (49 known + 8 connected)");

        // Should have only 8 connected peers (mesh neighbors)
        assert_eq!(status.connected_peers, 8, "Should only connect to 8 mesh neighbors");

        // peer_count should match connected_peers (backward compat)
        assert_eq!(status.peer_count, 8, "peer_count should match connected_peers for backward compat");
    }

    #[test]
    fn test_scalability_50_nodes() {
        let manager = P2pManager::new(P2pConfig::default());

        // Simulate 50-node cluster: know about all 49 other peers
        for i in 1..=49 {
            manager.add_known_peer(i).unwrap();
        }

        // But only connect to 8 mesh neighbors
        for i in 1..=8 {
            manager.add_connected_peer(100 + i, None).unwrap();
        }

        let status = manager.sync_status().unwrap();

        // O(n) awareness: know about all 49 peers + 8 connected = 57 total
        assert_eq!(status.known_peers, 57);

        // O(1) scalability: only 8 connections regardless of network size
        assert_eq!(status.connected_peers, 8);
    }

    #[test]
    fn test_scalability_1000_nodes() {
        let manager = P2pManager::new(P2pConfig::default());

        // Simulate 1000-node cluster: know about all 999 other peers
        for i in 1..=999 {
            manager.add_known_peer(i).unwrap();
        }

        // But only connect to 8 mesh neighbors
        for i in 1..=8 {
            manager.add_connected_peer(10000 + i, None).unwrap();
        }

        let status = manager.sync_status().unwrap();

        // O(n) awareness: know about all 999 peers + 8 connected = 1007 total
        assert_eq!(status.known_peers, 1007);

        // O(1) scalability: STILL only 8 connections (proves O(1) scaling!)
        assert_eq!(status.connected_peers, 8, "Should have exactly 8 connections even with 1000 nodes!");
    }

    #[test]
    fn test_connected_peer_also_added_to_known_peers() {
        let manager = P2pManager::new(P2pConfig::default());

        // Add a connected peer
        manager.add_connected_peer(1, None).unwrap();

        let status = manager.sync_status().unwrap();

        // Should appear in both known and connected
        assert_eq!(status.known_peers, 1);
        assert_eq!(status.connected_peers, 1);
    }

    #[test]
    fn test_backward_compatibility_add_peer() {
        let manager = P2pManager::new(P2pConfig::default());

        // Legacy add_peer() should work like add_connected_peer()
        manager.add_peer(1, None).unwrap();
        manager.add_peer(2, None).unwrap();

        let status = manager.sync_status().unwrap();

        // Should be treated as connected peers
        assert_eq!(status.connected_peers, 2);
        assert_eq!(status.known_peers, 2);
        assert_eq!(status.peer_count, 2);
    }

    #[test]
    #[cfg(feature = "dht")]
    fn test_key_to_slot_deterministic() {
        let manager = P2pManager::new(P2pConfig::default());

        let block_id = "QmTest123";
        let slot1 = manager.key_to_slot(block_id);
        let slot2 = manager.key_to_slot(block_id);

        assert!(slot1.is_some());
        assert_eq!(slot1, slot2, "Same key should map to same slot");
    }

    #[test]
    #[cfg(feature = "dht")]
    fn test_key_to_slot_different_keys() {
        let manager = P2pManager::new(P2pConfig::default());

        let slot1 = manager.key_to_slot("QmBlock1");
        let slot2 = manager.key_to_slot("QmBlock2");

        assert!(slot1.is_some());
        assert!(slot2.is_some());
        // Different keys will usually map to different slots (though collisions possible)
    }

    #[test]
    #[cfg(feature = "dht")]
    fn test_greedy_direction_no_slot_assigned() {
        let manager = P2pManager::new(P2pConfig::default());

        let direction = manager.greedy_direction_for_key("QmTest");
        assert!(direction.is_none(), "Should return None when our slot not assigned");
    }

    #[test]
    #[cfg(feature = "dht")]
    fn test_greedy_direction_with_slot() {
        use citadel_core::topology::{MeshConfig, SlotCoordinate};

        let mut config = P2pConfig::default();
        config.mesh_config = Some(MeshConfig::new(10, 10, 5));
        config.slot_coordinate = Some(SlotCoordinate::new(5, 5, 2));

        let manager = P2pManager::new(config);

        // Test with a key that maps far away
        let direction = manager.greedy_direction_for_key("QmFarAwayBlock");
        // Should get a direction (unless key maps to our exact slot)
        // We can't assert specific direction since hash is deterministic but unknown
    }

    #[test]
    #[cfg(feature = "dht")]
    fn test_next_hop_for_key() {
        use citadel_core::topology::{MeshConfig, SlotCoordinate};
        use std::collections::HashMap;

        let mut config = P2pConfig::default();
        config.mesh_config = Some(MeshConfig::new(10, 10, 5));
        config.slot_coordinate = Some(SlotCoordinate::new(5, 5, 2));

        let manager = P2pManager::new(config);

        // Create peer slots map
        let mut peer_slots = HashMap::new();
        peer_slots.insert(1, SlotCoordinate::new(6, 5, 2)); // Neighbor in +A direction
        peer_slots.insert(2, SlotCoordinate::new(5, 6, 2)); // Neighbor in +B direction

        // Find next hop for a key
        // Can't assert specific result without knowing which direction the key maps to
        let next_hop = manager.next_hop_for_key("QmTestBlock", &peer_slots);
        // Next hop should be one of our neighbors if key is not at our slot
    }

    #[test]
    #[cfg(feature = "dht")]
    fn test_mesh_config_accessor() {
        let manager = P2pManager::new(P2pConfig::default());
        assert!(manager.mesh_config().is_some());
    }

    #[test]
    #[cfg(feature = "dht")]
    fn test_slot_coordinate_accessor() {
        let mut config = P2pConfig::default();
        config.slot_coordinate = Some(citadel_core::topology::SlotCoordinate::new(5, 5, 2));

        let manager = P2pManager::new(config);
        assert!(manager.slot_coordinate().is_some());
    }
}
