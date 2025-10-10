//! Lens V2 P2P Layer
//!
//! Pure Rust P2P implementation with:
//! - BFT consensus for fault-tolerant agreement
//! - TGP for efficient block exchange
//! - WASM-compatible (< 500KiB soft goal)
//! - Sync status tracking with /ready endpoint support

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

pub mod sync;
pub mod manager;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(all(not(target_arch = "wasm32"), feature = "network"))]
pub mod network;

// Re-exports
pub use manager::{P2pConfig, P2pManager};
pub use sync::SyncTracker;

#[cfg(all(not(target_arch = "wasm32"), feature = "network"))]
pub use network::P2pNetwork;

// Re-export WantList from consensus-peerexc if available
#[cfg(feature = "consensus")]
pub use consensus_peerexc::wantlist::WantList;

/// P2P errors
#[derive(Debug, Error)]
pub enum P2pError {
    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Block exchange error: {0}")]
    BlockExchange(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid configuration: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, P2pError>;

/// Block identifier (CID or hash)
pub type BlockId = String;

/// Peer identifier
pub type PeerId = u64;

/// Block metadata tracked by consensus
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMeta {
    pub id: BlockId,
    pub height: u64,
    pub prev: Option<BlockId>,
    pub timestamp: u64,
}

/// Sync status for /ready endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Whether this node is fully synced
    pub is_synced: bool,

    /// Latest block height in the network (from BFT consensus)
    pub network_height: u64,

    /// Local block height
    pub local_height: u64,

    /// Number of blocks behind
    pub blocks_behind: u64,

    /// Number of connected peers
    pub peer_count: usize,

    /// Blocks currently being downloaded
    pub downloading: HashSet<BlockId>,
}

impl SyncStatus {
    pub fn new() -> Self {
        Self {
            is_synced: false,
            network_height: 0,
            local_height: 0,
            blocks_behind: 0,
            peer_count: 0,
            downloading: HashSet::new(),
        }
    }

    pub fn update_heights(&mut self, local: u64, network: u64) {
        self.local_height = local;
        self.network_height = network;
        self.blocks_behind = network.saturating_sub(local);
        self.is_synced = self.blocks_behind == 0 && self.peer_count > 0;
    }
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_new() {
        let status = SyncStatus::new();
        assert!(!status.is_synced);
        assert_eq!(status.blocks_behind, 0);
    }

    #[test]
    fn test_sync_status_update() {
        let mut status = SyncStatus::new();
        status.peer_count = 3;
        status.update_heights(100, 150);

        assert_eq!(status.local_height, 100);
        assert_eq!(status.network_height, 150);
        assert_eq!(status.blocks_behind, 50);
        assert!(!status.is_synced);
    }

    #[test]
    fn test_sync_status_synced() {
        let mut status = SyncStatus::new();
        status.peer_count = 5;
        status.update_heights(200, 200);

        assert!(status.is_synced);
        assert_eq!(status.blocks_behind, 0);
    }
}
