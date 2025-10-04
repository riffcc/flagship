//! Sync status tracking and management

use crate::{BlockId, BlockMeta, PeerId, Result, SyncStatus};
use std::collections::{HashMap, HashSet};

/// Tracks synchronization state
pub struct SyncTracker {
    /// Blocks we have locally
    local_blocks: HashMap<BlockId, BlockMeta>,

    /// Blocks agreed upon by BFT consensus
    consensus_blocks: HashMap<BlockId, BlockMeta>,

    /// Blocks we're currently downloading
    downloading: HashSet<BlockId>,

    /// Connected peers
    peers: HashSet<PeerId>,

    /// Latest known network height
    network_height: u64,
}

impl SyncTracker {
    pub fn new() -> Self {
        Self {
            local_blocks: HashMap::new(),
            consensus_blocks: HashMap::new(),
            downloading: HashSet::new(),
            peers: HashSet::new(),
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

    /// Add a peer
    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.peers.insert(peer_id);
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
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
        status.peer_count = self.peers.len();
        status.update_heights(local_height, self.network_height);
        status.downloading = self.downloading.clone();
        status
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
    pub fn is_synced(&self) -> bool {
        self.blocks_behind() == 0 && !self.peers.is_empty()
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
        assert!(!tracker.is_synced()); // No peers
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

        // Not synced: no peers
        assert!(!tracker.is_synced());

        // Add peer
        tracker.add_peer(1);

        // Still not synced: no blocks
        assert!(tracker.is_synced()); // Actually synced at height 0

        // Add blocks
        tracker.add_local_block(make_block("block1", 1));
        tracker.update_consensus(vec![make_block("block1", 1)]);

        // Now synced
        assert!(tracker.is_synced());

        // Add consensus block we don't have
        tracker.update_consensus(vec![make_block("block2", 2)]);

        // No longer synced
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
