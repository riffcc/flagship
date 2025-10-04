//! P2P Manager coordinating BFT consensus and TGP block exchange

use crate::{BlockId, BlockMeta, P2pError, PeerId, Result, SyncStatus};
use crate::sync::SyncTracker;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::RwLock;

#[cfg(target_arch = "wasm32")]
use std::sync::Mutex as RwLock; // WASM single-threaded

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
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            node_id: 0,
            n: 4,           // 4 nodes
            f: 1,           // Tolerate 1 Byzantine fault
            view_timeout_ms: 5000,
            mtu: 1024 * 64, // 64 KB chunks
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

    /// Add a peer to the network
    pub fn add_peer(&self, peer_id: PeerId) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        self.sync.write()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_peer(peer_id);

        #[cfg(target_arch = "wasm32")]
        self.sync.lock()
            .map_err(|e| P2pError::Sync(format!("Lock error: {}", e)))?
            .add_peer(peer_id);

        Ok(())
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

        assert!(!status.is_synced);
        assert_eq!(status.blocks_behind, 0);
    }

    #[test]
    fn test_add_peer() {
        let manager = P2pManager::new(P2pConfig::default());
        manager.add_peer(1).unwrap();
        manager.add_peer(2).unwrap();

        let status = manager.sync_status().unwrap();
        assert_eq!(status.peer_count, 2);
    }

    #[test]
    fn test_remove_peer() {
        let manager = P2pManager::new(P2pConfig::default());
        manager.add_peer(1).unwrap();
        manager.add_peer(2).unwrap();
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

        // Start with no peers
        let status = manager.sync_status().unwrap();
        assert!(!status.is_synced);

        // Add peer
        manager.add_peer(1).unwrap();

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
}
