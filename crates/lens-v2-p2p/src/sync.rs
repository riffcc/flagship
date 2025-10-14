//! Sync status tracking and management

use crate::{BlockId, BlockMeta, PeerId, Result, SyncStatus};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Peer type for tracking server vs browser peers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackedPeerType {
    Server,
    Browser,
}

/// Tracks synchronization state
pub struct SyncTracker {
    /// Blocks we have locally
    local_blocks: HashMap<BlockId, BlockMeta>,

    /// Blocks agreed upon by BFT consensus
    consensus_blocks: HashMap<BlockId, BlockMeta>,

    /// Blocks we're currently downloading
    downloading: HashSet<BlockId>,

    /// Known peers - all peers we've heard about (for awareness, O(n))
    known_peers: HashSet<PeerId>,

    /// Connected peers - only our 8 mesh neighbors (actual P2P connections, O(1))
    connected_peers: HashSet<PeerId>,

    /// Peer types (server vs browser)
    peer_types: HashMap<PeerId, TrackedPeerType>,

    /// Map of u64 peer ID -> original string peer ID (for /map endpoint)
    peer_id_strings: HashMap<PeerId, String>,

    /// Peers that are currently alive (heartbeated in current cycle)
    alive_peers: HashSet<PeerId>,

    /// Latest known network height
    network_height: u64,
}

impl SyncTracker {
    pub fn new() -> Self {
        Self {
            local_blocks: HashMap::new(),
            consensus_blocks: HashMap::new(),
            downloading: HashSet::new(),
            known_peers: HashSet::new(),
            connected_peers: HashSet::new(),
            peer_types: HashMap::new(),
            peer_id_strings: HashMap::new(),
            alive_peers: HashSet::new(),
            network_height: 0,
        }
    }

    /// Add a block to local storage
    pub fn add_local_block(&mut self, block: BlockMeta) {
        self.local_blocks.insert(block.id.clone(), block);
    }

    /// Update consensus view of blocks
    pub fn update_consensus(&mut self, blocks: Vec<BlockMeta>) {
        for block in blocks {
            if block.height > self.network_height {
                self.network_height = block.height;
            }
            self.consensus_blocks.insert(block.id.clone(), block);
        }
    }

    /// Mark a block as being downloaded
    pub fn mark_downloading(&mut self, block_id: BlockId) {
        self.downloading.insert(block_id);
    }

    /// Mark a block as downloaded
    pub fn mark_downloaded(&mut self, block_id: &BlockId) {
        self.downloading.remove(block_id);
    }

    /// Add a known peer (for awareness) - O(n) scalability
    pub fn add_known_peer(&mut self, peer_id: PeerId) {
        self.known_peers.insert(peer_id);
    }

    /// Add a known peer with original string ID (for /map endpoint)
    pub fn add_known_peer_with_string(&mut self, peer_id: PeerId, peer_id_string: String) {
        self.known_peers.insert(peer_id);
        self.peer_id_strings.insert(peer_id, peer_id_string);
        // Don't mark as alive yet - wait for explicit heartbeat
    }

    /// Add a connected peer (actual P2P connection to mesh neighbor) - O(1) scalability
    pub fn add_connected_peer(&mut self, peer_id: PeerId, peer_type: Option<TrackedPeerType>) {
        self.connected_peers.insert(peer_id);
        self.known_peers.insert(peer_id); // Connected peers are also known peers

        if let Some(ptype) = peer_type {
            self.peer_types.insert(peer_id, ptype);
        } else {
            // Default to Server if not specified
            self.peer_types.insert(peer_id, TrackedPeerType::Server);
        }
    }

    /// Add a peer with optional type (defaults to Server if not specified)
    /// DEPRECATED: Use add_connected_peer() for mesh neighbors or add_known_peer() for awareness
    pub fn add_peer(&mut self, peer_id: PeerId, peer_type: Option<TrackedPeerType>) {
        self.add_connected_peer(peer_id, peer_type);
    }

    /// Remove a peer from both known and connected sets
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.known_peers.remove(peer_id);
        self.connected_peers.remove(peer_id);
        self.peer_types.remove(peer_id);
    }

    /// Get current sync status
    pub fn status(&self) -> SyncStatus {
        let local_height = self
            .local_blocks
            .values()
            .map(|b| b.height)
            .max()
            .unwrap_or(0);

        let mut status = SyncStatus::new();
        status.known_peers = self.known_peers.len();
        status.connected_peers = self.connected_peers.len();
        status.peer_count = self.connected_peers.len(); // Backward compatibility
        status.update_heights(local_height, self.network_height);
        status.downloading = self.downloading.clone();
        status
    }

    /// Mark peer as alive (received heartbeat this cycle)
    pub fn mark_peer_alive(&mut self, peer_id: PeerId) {
        self.alive_peers.insert(peer_id);
    }

    /// Clear all alive peers (start new heartbeat cycle)
    pub fn clear_alive_peers(&mut self) {
        self.alive_peers.clear();
    }

    /// Get all known peer IDs as strings (for /map endpoint)
    pub fn get_known_peer_strings(&self) -> Vec<String> {
        self.known_peers.iter()
            .filter_map(|peer_id| self.peer_id_strings.get(peer_id).cloned())
            .collect()
    }

    /// Get all ALIVE known peer IDs as strings (for /map endpoint)
    /// Only returns peers that heartbeated in the current cycle
    pub fn get_alive_peer_strings(&self) -> Vec<String> {
        self.alive_peers.iter()
            .filter_map(|peer_id| self.peer_id_strings.get(peer_id).cloned())
            .collect()
    }

    /// Get all known peer IDs as u64 (for internal use)
    pub fn get_known_peers(&self) -> Vec<PeerId> {
        self.known_peers.iter().cloned().collect()
    }

    /// Get blocks we need to download (in consensus but not local)
    pub fn missing_blocks(&self) -> Vec<BlockId> {
        self.consensus_blocks
            .keys()
            .filter(|id| !self.local_blocks.contains_key(*id))
            .filter(|id| !self.downloading.contains(*id))
            .cloned()
            .collect()
    }

    /// Get number of blocks behind
    pub fn blocks_behind(&self) -> u64 {
        let local_height = self
            .local_blocks
            .values()
            .map(|b| b.height)
            .max()
            .unwrap_or(0);
        self.network_height.saturating_sub(local_height)
    }

    /// Check if we're synced
    ///
    /// Returns true if we're caught up with the network (0 blocks behind).
    /// This works for both bootstrap (0 peers, 0 behind) and normal operation (N peers, 0 behind).
    pub fn is_synced(&self) -> bool {
        self.blocks_behind() == 0
    }
}

impl Default for SyncTracker {
    fn default() -> Self {
        Self::new()
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
    fn test_sync_tracker_new() {
        let tracker = SyncTracker::new();
        assert_eq!(tracker.blocks_behind(), 0);
        // Bootstrap case: 0 peers, 0 blocks behind → synced
        assert!(tracker.is_synced());
    }

    #[test]
    fn test_add_local_block() {
        let mut tracker = SyncTracker::new();
        tracker.add_local_block(make_block("block1", 1));
        tracker.add_local_block(make_block("block2", 2));

        let status = tracker.status();
        assert_eq!(status.local_height, 2);
    }

    #[test]
    fn test_update_consensus() {
        let mut tracker = SyncTracker::new();
        tracker.update_consensus(vec![
            make_block("block1", 1),
            make_block("block2", 2),
            make_block("block3", 3),
        ]);

        assert_eq!(tracker.network_height, 3);
    }

    #[test]
    fn test_missing_blocks() {
        let mut tracker = SyncTracker::new();

        // We have block1 locally
        tracker.add_local_block(make_block("block1", 1));

        // Consensus says we should have block1, block2, block3
        tracker.update_consensus(vec![
            make_block("block1", 1),
            make_block("block2", 2),
            make_block("block3", 3),
        ]);

        let missing = tracker.missing_blocks();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"block2".to_string()));
        assert!(missing.contains(&"block3".to_string()));
    }

    #[test]
    fn test_blocks_behind() {
        let mut tracker = SyncTracker::new();

        tracker.add_local_block(make_block("block1", 1));
        tracker.update_consensus(vec![
            make_block("block1", 1),
            make_block("block2", 2),
            make_block("block3", 3),
        ]);

        assert_eq!(tracker.blocks_behind(), 2);
    }

    #[test]
    fn test_is_synced() {
        let mut tracker = SyncTracker::new();

        // Bootstrap case: 0 peers, 0 blocks behind → synced (we ARE the network)
        assert!(tracker.is_synced());

        // Add peer (defaults to Server type)
        tracker.add_peer(1, None);

        // Synced at height 0 with peer
        assert!(tracker.is_synced());

        // Add blocks
        tracker.add_local_block(make_block("block1", 1));
        tracker.update_consensus(vec![make_block("block1", 1)]);

        // Still synced
        assert!(tracker.is_synced());

        // Add consensus block we don't have
        tracker.update_consensus(vec![make_block("block2", 2)]);

        // No longer synced (behind by 1 block)
        assert!(!tracker.is_synced());
    }

    #[test]
    fn test_downloading_tracking() {
        let mut tracker = SyncTracker::new();

        tracker.update_consensus(vec![
            make_block("block1", 1),
            make_block("block2", 2),
        ]);

        // Mark block1 as downloading
        tracker.mark_downloading("block1".to_string());

        // Should only see block2 as missing
        let missing = tracker.missing_blocks();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "block2");

        // Status should show downloading
        let status = tracker.status();
        assert!(status.downloading.contains("block1"));
    }
}
