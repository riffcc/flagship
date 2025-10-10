//! Sync Orchestrator
//!
//! Coordinates P2P sync between network, consensus, and persistence layers.
//! This is the main sync loop that keeps lens nodes in sync with each other.

use anyhow::Result;
use lens_v2_p2p::{BlockMeta, P2pManager, P2pNetwork, WantList};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, info, warn, error};

use crate::db::{Database, prefixes, make_key};
use crate::routes::releases::Release;
use crate::block_codec::{BlockCodec, BlockEnvelope};

/// Sync orchestrator coordinates all P2P sync operations
pub struct SyncOrchestrator {
    /// P2P network layer
    network: Arc<P2pNetwork>,

    /// P2P manager (consensus + sync tracking)
    p2p_manager: Arc<P2pManager>,

    /// Database for persistence
    db: Database,

    /// Sync interval
    sync_interval: Duration,
}

impl SyncOrchestrator {
    /// Create a new sync orchestrator
    pub fn new(
        relay_url: String,
        p2p_manager: Arc<P2pManager>,
        db: Database,
    ) -> Self {
        let network = Arc::new(P2pNetwork::new(relay_url));

        Self {
            network,
            p2p_manager,
            db,
            sync_interval: Duration::from_secs(30),
        }
    }

    /// Start the sync orchestrator
    ///
    /// This runs in the background and continuously syncs with peers
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("Starting sync orchestrator");

        // Connect to relay
        self.network.start().await?;

        // Spawn sync loop
        let orchestrator = self.clone();
        tokio::spawn(async move {
            orchestrator.sync_loop().await;
        });

        Ok(())
    }

    /// Main sync loop
    async fn sync_loop(&self) {
        let mut interval = time::interval(self.sync_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.sync_iteration().await {
                error!("Sync iteration failed: {}", e);
            }
        }
    }

    /// Single sync iteration
    async fn sync_iteration(&self) -> Result<()> {
        debug!("Starting sync iteration");

        // 1. Build WantList from current state
        let wantlist = self.build_wantlist().await?;

        // 2. Send WantList to relay for peer discovery
        if wantlist.has_needs() || wantlist.has_offers() {
            info!("Sending WantList to network");
            self.network.send_wantlist(&wantlist).await?;
        }

        // 3. Check for missing blocks
        let missing = self.p2p_manager.missing_blocks()?;
        if !missing.is_empty() {
            info!("Need to fetch {} missing blocks", missing.len());

            // 4. Request missing blocks from peers
            let peers = self.network.peers().await;
            if !peers.is_empty() {
                // Round-robin through peers
                for (i, block_id) in missing.iter().enumerate() {
                    let peer = &peers[i % peers.len()];
                    self.network.request_blocks(&peer.peer_id, vec![block_id.clone()]).await?;
                    self.p2p_manager.mark_downloading(block_id.clone())?;
                }
            }
        }

        // 5. Process incoming network events
        while let Some(event) = self.network.next_event().await {
            if let Err(e) = self.handle_network_event(event).await {
                warn!("Failed to handle network event: {}", e);
            }
        }

        debug!("Sync iteration complete");
        Ok(())
    }

    /// Build WantList from current local state
    async fn build_wantlist(&self) -> Result<WantList> {
        let mut wantlist = WantList::new(1); // TODO: Track generation properly

        // Get local blocks from database
        let local_blocks = self.get_local_blocks().await?;

        // Add to WantList
        for block in local_blocks {
            wantlist.add_have_block(block.id);
        }

        // Get consensus blocks and find what we're missing
        let sync_status = self.p2p_manager.sync_status()?;
        if !sync_status.is_synced {
            // We're behind, request missing blocks
            let missing = self.p2p_manager.missing_blocks()?;
            for block_id in missing {
                wantlist.add_need_block(block_id);
            }
        }

        Ok(wantlist)
    }

    /// Get local blocks from database
    async fn get_local_blocks(&self) -> Result<Vec<BlockMeta>> {
        // Get all releases from database
        let releases: Vec<Release> = self.db.get_all_with_prefix(prefixes::RELEASE)?;

        // Convert to block metadata
        let mut blocks = Vec::new();
        for (i, release) in releases.iter().enumerate() {
            let height = i as u64 + 1; // Height starts at 1
            let prev = if i > 0 {
                Some(format!("block-{}", i - 1))
            } else {
                None
            };

            let block_data = BlockCodec::encode_release(release.clone(), height, prev)?;
            blocks.push(BlockMeta {
                id: block_data.id,
                height: block_data.height,
                prev: block_data.prev,
                timestamp: block_data.timestamp,
            });
        }

        Ok(blocks)
    }

    /// Handle a network event
    async fn handle_network_event(&self, event: lens_v2_p2p::network::NetworkEvent) -> Result<()> {
        use lens_v2_p2p::network::NetworkEvent;

        match event {
            NetworkEvent::PeerConnected(peer) => {
                info!("Peer connected: {} at height {}", peer.peer_id, peer.latest_height);
                // Convert peer_id string to u64 for P2pManager
                // TODO: Fix PeerId type mismatch between network and manager
            }

            NetworkEvent::PeerDisconnected(peer_id) => {
                info!("Peer disconnected: {}", peer_id);
            }

            NetworkEvent::BlockReceived(block_data) => {
                info!("Received block: {} at height {}", block_data.id, block_data.height);

                // Add to local storage
                self.save_block(block_data).await?;
            }

            NetworkEvent::PeerReferral(peers) => {
                info!("Received referral for {} peers", peers.len());
                for peer in peers {
                    info!("  - {} at height {} (score: {})", peer.peer_id, peer.latest_height, peer.score);
                }
            }

            NetworkEvent::PeerIdAssigned(peer_id) => {
                info!("Assigned peer ID: {}", peer_id);
            }
        }

        Ok(())
    }

    /// Save a block to local storage
    async fn save_block(&self, block_data: lens_v2_p2p::network::BlockData) -> Result<()> {
        info!("Saving block {} to database", block_data.id);

        // Mark as downloaded
        self.p2p_manager.mark_downloaded(&block_data.id)?;

        // Add to local blocks
        let meta = BlockMeta {
            id: block_data.id.clone(),
            height: block_data.height,
            prev: block_data.prev.clone(),
            timestamp: block_data.timestamp,
        };
        self.p2p_manager.add_local_block(meta)?;

        // Decode releases from block
        let releases = BlockCodec::decode_releases(&block_data)?;
        info!("Block contains {} releases", releases.len());

        // Save each release to database
        for release in releases {
            let key = make_key(prefixes::RELEASE, &release.id);
            self.db.put(&key, &release)?;
            info!("Saved release: {}", release.id);
        }

        // Decode and save featured list if present
        let featured = BlockCodec::decode_featured(&block_data)?;
        if !featured.is_empty() {
            info!("Block contains {} featured releases", featured.len());
            // TODO: Update featured list in database
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lens_v2_p2p::P2pConfig;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            p2p_manager,
            db,
        );

        assert_eq!(orchestrator.sync_interval, Duration::from_secs(30));
    }
}
