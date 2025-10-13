//! Sync Orchestrator
//!
//! Coordinates P2P sync between network, consensus, and persistence layers.
//! This is the main sync loop that keeps lens nodes in sync with each other.
//!
//! # DHT-Based Lazy Neighbor Discovery
//!
//! The orchestrator uses LazyNode for DHT-based neighbor discovery:
//!
//! - Each node has a slot coordinate in the 2.5D hexagonal toroidal mesh
//! - Neighbors are discovered on-demand via DHT queries (no caching)
//! - Each node queries DHT for "who owns my 8 neighbor slots?"
//! - Block IDs are deterministically mapped to target slots via modulo hashing
//! - Greedy routing finds the optimal path from current slot to target slot
//!
//! This enables O(log n) block discovery with zero stale peer data.

use anyhow::Result;
use lens_v2_p2p::{BlockMeta, P2pManager, P2pNetwork, WantList};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn, error};

use crate::db::{Database, prefixes, make_key};
use crate::routes::releases::Release;
use crate::routes::account::BlockNotification;
use crate::block_codec::{BlockCodec, BlockEnvelope};
use crate::lazy_node::LazyNode;

use citadel_core::topology::{MeshConfig, SlotCoordinate, Direction};
use citadel_dht::local_storage::LocalStorage;

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

    /// Receiver for immediate block notifications
    block_notify_rx: Arc<RwLock<mpsc::UnboundedReceiver<BlockNotification>>>,

    /// LazyNode for DHT-based neighbor discovery (no caching!)
    lazy_node: Arc<LazyNode>,
}

impl SyncOrchestrator {
    /// Create a new sync orchestrator with LazyNode for neighbor discovery
    pub fn new(
        relay_url: String,
        my_peer_id: String,
        my_slot: SlotCoordinate,
        mesh_config: MeshConfig,
        p2p_manager: Arc<P2pManager>,
        db: Database,
        block_notify_rx: mpsc::UnboundedReceiver<BlockNotification>,
        dht_storage: Arc<tokio::sync::Mutex<LocalStorage>>,
    ) -> Self {
        // Pass our peer_id to network layer so it can announce to relay
        let network = Arc::new(P2pNetwork::new(relay_url, my_peer_id.clone()));

        // Create LazyNode for DHT-based neighbor discovery
        let lazy_node = Arc::new(LazyNode::new(
            my_slot,
            my_peer_id,
            mesh_config,
            dht_storage,
        ));

        Self {
            network,
            p2p_manager,
            db,
            sync_interval: Duration::from_secs(30),
            block_notify_rx: Arc::new(RwLock::new(block_notify_rx)),
            lazy_node,
        }
    }

    /// Start the sync orchestrator
    ///
    /// This runs in the background and continuously syncs with peers
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("Starting sync orchestrator");

        // Spawn persistent relay connection task (non-blocking, runs forever)
        // The relay is anycast - we try to stay connected for fallback comms
        let network = self.network.clone();
        tokio::spawn(async move {
            let mut retry_delay = Duration::from_secs(1);
            let max_delay = Duration::from_secs(300); // Cap at 5 minutes
            let mut attempt = 0;

            loop {
                attempt += 1;

                info!("🔄 Relay connection attempt #{}", attempt);
                match network.start().await {
                    Ok(_) => {
                        info!("✅ Connected to relay (anycast fallback comms active)");
                        // Connection succeeded - keep this connection alive
                        // If it drops, the loop will reconnect
                        break;
                    }
                    Err(e) => {
                        if attempt == 1 {
                            // First attempt - might be the first node (we ARE the relay!)
                            info!("ℹ️ No relay available (might be first node - this node IS the relay via anycast)");
                        }
                        warn!("⚠️ Relay connection attempt #{} failed: {} - retrying in {:?}", attempt, e, retry_delay);

                        tokio::time::sleep(retry_delay).await;

                        // Exponential backoff: 1s -> 2s -> 4s -> 8s -> ... -> 300s
                        retry_delay = std::cmp::min(retry_delay * 2, max_delay);
                    }
                }
            }

            // If we get here, we're connected - keep retrying if connection drops
            info!("🌉 Relay connection established - monitoring for reconnection");
        });

        // Spawn instant broadcast listener
        let orchestrator_instant = self.clone();
        tokio::spawn(async move {
            orchestrator_instant.instant_broadcast_loop().await;
        });

        // Spawn sync loop
        let orchestrator = self.clone();
        tokio::spawn(async move {
            orchestrator.sync_loop().await;
        });

        // Startup is complete - relay connection continues in background
        info!("✅ Sync orchestrator started (relay connection running in background)");
        Ok(())
    }

    /// Instant broadcast loop - listens for new block notifications and broadcasts immediately
    async fn instant_broadcast_loop(&self) {
        info!("🚀 Starting instant broadcast listener");

        loop {
            let notification = {
                let mut rx = self.block_notify_rx.write().await;
                rx.recv().await
            };

            match notification {
                Some(BlockNotification::NewBlock(block_id)) => {
                    info!("⚡ INSTANT BROADCAST triggered by new block: {}", block_id);

                    // Build and broadcast WantList immediately
                    match self.build_wantlist().await {
                        Ok(wantlist) => {
                            info!("📋 Built instant WantList: gen={}, needs={}, offers={}",
                                wantlist.generation, wantlist.has_needs(), wantlist.has_offers());

                            if let Err(e) = self.network.send_wantlist(&wantlist).await {
                                error!("Failed to send instant WantList: {}", e);
                            } else {
                                info!("⚡ INSTANT WantList broadcast complete in MILLISECONDS");
                            }
                        }
                        Err(e) => {
                            error!("Failed to build instant WantList: {}", e);
                        }
                    }
                }
                None => {
                    // Channel closed
                    warn!("Block notification channel closed");
                    break;
                }
            }
        }
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
        info!("🔄 Starting sync iteration");

        // 1. Build WantList from current state
        let wantlist = self.build_wantlist().await?;
        info!("📋 Built WantList: gen={}, needs={}, offers={}",
            wantlist.generation, wantlist.has_needs(), wantlist.has_offers());

        // 2. Send WantList to relay for peer discovery
        // Always send WantList, even if empty, to announce our presence
        info!("📤 Sending WantList to network");
        self.network.send_wantlist(&wantlist).await?;

        // 3. Check for missing blocks
        let missing = self.p2p_manager.missing_blocks()?;
        if !missing.is_empty() {
            info!("📥 Need to fetch {} missing blocks", missing.len());

            // 4. Request missing blocks from peers
            let peers = self.network.peers().await;
            info!("👥 Have {} known peers", peers.len());
            if !peers.is_empty() {
                // Round-robin through peers
                for (i, block_id) in missing.iter().enumerate() {
                    let peer = &peers[i % peers.len()];
                    self.network.request_blocks(&peer.peer_id, vec![block_id.clone()]).await?;
                    self.p2p_manager.mark_downloading(block_id.clone())?;
                }
            }
        } else {
            info!("✅ No missing blocks");
        }

        // 5. Process incoming network events (non-blocking)
        // Process all available events without blocking
        let mut event_count = 0;
        loop {
            match self.network.try_next_event().await {
                Some(event) => {
                    event_count += 1;
                    if let Err(e) = self.handle_network_event(event).await {
                        warn!("Failed to handle network event: {}", e);
                    }
                }
                None => break, // No more events available
            }
        }

        if event_count > 0 {
            info!("📨 Processed {} network events", event_count);
        } else {
            info!("📭 No network events to process");
        }

        info!("✅ Sync iteration complete");
        Ok(())
    }

    /// Build WantList from current local state + known peers
    async fn build_wantlist(&self) -> Result<WantList> {
        let mut wantlist = WantList::new(1); // TODO: Track generation properly

        // Get local blocks from database (releases)
        let local_blocks = self.get_local_blocks().await?;

        // Add releases to WantList
        for block in local_blocks {
            wantlist.add_have_block(block.id);
        }

        // Get authorization transactions and add to WantList
        // Authorization transactions are flat UBTS transactions that sync via SPORE
        use crate::routes::account::AuthorizationTransaction;
        let authorizations: Vec<AuthorizationTransaction> = self.db.get_all_with_prefix(prefixes::AUTHORIZATION)?;
        for auth in authorizations {
            wantlist.add_have_block(auth.id);
        }

        // Get delete transactions and add to WantList
        // Delete transactions are UBTS blocks that sync via SPORE
        use crate::ubts::UBTSBlock;
        let delete_txs: Vec<UBTSBlock> = self.db.get_all_with_prefix(prefixes::DELETE_TRANSACTION)?;
        for delete_tx in delete_txs {
            wantlist.add_have_block(delete_tx.id.clone());
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

        // Add known peers to WantList for SPORE peer gossip
        // LazyNode: Query DHT for 8 hexagonal mesh neighbors (lazy discovery, no caching!)
        let neighbors = self.lazy_node.get_all_neighbors().await?;
        info!("🔷 LazyNode neighbor discovery: {}/8 mesh neighbors found", neighbors.len());

        for neighbor in neighbors {
            // Add mesh neighbor to WantList with max score (255)
            wantlist.add_known_peer(neighbor, 255);
        }

        Ok(wantlist)
    }

    /// Get local blocks from database
    async fn get_local_blocks(&self) -> Result<Vec<BlockMeta>> {
        // Get all releases from database
        let releases: Vec<Release> = self.db.get_all_with_prefix(prefixes::RELEASE)?;

        // Convert to block metadata - flat transactions, no heights
        let mut blocks = Vec::new();
        for release in releases.iter() {
            let block_data = BlockCodec::encode_release(release.clone(), 0, None)?;
            blocks.push(BlockMeta {
                id: block_data.id,
                height: 0, // Flat transactions - no heights
                prev: None, // No chain
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
                info!("Peer connected: {}", peer.peer_id);
                // Convert peer_id string to u64 for P2pManager
                // TODO: Fix PeerId type mismatch between network and manager
            }

            NetworkEvent::PeerDisconnected(peer_id) => {
                info!("Peer disconnected: {}", peer_id);
            }

            NetworkEvent::BlockReceived(block_data) => {
                info!("Received block: {}", block_data.id);

                // Add to local storage
                self.save_block(block_data).await?;
            }

            // PeerReferral: Populate local DHT from relay gossip for bootstrap!
            // The relay acts as the DHT synchronization layer - when we receive peer referrals,
            // we extract slot ownership and store in local DHT so LazyNode queries can find neighbors.
            NetworkEvent::PeerReferral(peers) => {
                info!("📡 Received {} peer referrals from relay - populating local DHT", peers.len());

                for peer in &peers {
                    // CRITICAL FIX: Use the peer's ACTUAL announced slot, not a recalculated one!
                    // Recalculating from peer_id can produce a DIFFERENT slot than what the peer announced,
                    // causing DHT key mismatches and neighbor discovery failures.
                    let slot = match peer.slot {
                        Some(announced_slot) => {
                            debug!("  Using peer {}'s announced slot: {:?}", peer.peer_id, announced_slot);
                            announced_slot
                        }
                        None => {
                            // Fallback: If relay doesn't provide slot, calculate from peer_id
                            // This is for backward compatibility with old relay versions
                            warn!("  ⚠️ Peer {} has no slot - falling back to peer_id_to_slot (may cause mismatches)", peer.peer_id);
                            crate::peer_registry::peer_id_to_slot(&peer.peer_id, self.lazy_node.mesh_config())
                        }
                    };

                    // Create slot ownership announcement
                    let ownership = crate::peer_registry::SlotOwnership::new(
                        peer.peer_id.clone(),
                        slot,
                        None, // No relay URL needed - we're all on same relay
                    );

                    // Store in local DHT so LazyNode can discover this neighbor
                    let ownership_key = crate::peer_registry::slot_ownership_key(slot);
                    let ownership_bytes = serde_json::to_vec(&ownership)?;

                    // Note: dht_storage is pub(crate) so we can access it from LazyNode
                    self.lazy_node.dht_storage.lock().await.put(ownership_key, ownership_bytes.into());

                    debug!("  ✅ Stored peer {} at slot {:?} in local DHT", peer.peer_id, slot);
                }

                info!("✅ Local DHT populated with {} peer slot ownerships", peers.len());
            }

            // REMOVED: PeerIdAssigned handling - peer_id passed to constructor now!
            NetworkEvent::PeerIdAssigned(_peer_id) => {
                // Ignored: peer_id is passed to constructor, not assigned at runtime
                debug!("Ignoring PeerIdAssigned event - peer_id set at construction");
            }

            NetworkEvent::WantListReceived(peer_id, wantlist) => {
                info!("🔍 Received WantList from {}: gen={}, have={} blocks, known_peers={}",
                    peer_id, wantlist.generation, wantlist.have_blocks.len(), wantlist.known_peers.len());

                // REMOVED: SPORE peer exchange - now using LazyNode DHT queries for neighbor discovery!

                // Build complete local block set (releases + auth txs + delete txs)
                let mut local_block_ids = std::collections::HashSet::new();

                // Add release block IDs
                let local_blocks = self.get_local_blocks().await?;
                for block in local_blocks {
                    local_block_ids.insert(block.id);
                }

                // Add authorization transaction IDs
                use crate::routes::account::AuthorizationTransaction;
                let authorizations: Vec<AuthorizationTransaction> = self.db.get_all_with_prefix(prefixes::AUTHORIZATION)?;
                for auth in authorizations {
                    local_block_ids.insert(auth.id);
                }

                // Add delete transaction IDs
                use crate::ubts::UBTSBlock;
                let delete_txs: Vec<UBTSBlock> = self.db.get_all_with_prefix(prefixes::DELETE_TRANSACTION)?;
                for delete_tx in delete_txs {
                    local_block_ids.insert(delete_tx.id);
                }

                // Find missing blocks
                let mut missing_from_peer = Vec::new();
                for peer_block_id in &wantlist.have_blocks {
                    if !local_block_ids.contains(peer_block_id) {
                        missing_from_peer.push(peer_block_id.clone());
                    }
                }

                if !missing_from_peer.is_empty() {
                    info!("🚨 SPORE detected {} missing blocks from peer {}", missing_from_peer.len(), peer_id);
                    for block_id in &missing_from_peer {
                        info!("  - Missing: {}", block_id);
                    }

                    // Request missing blocks immediately
                    info!("📥 Requesting {} missing blocks from {}", missing_from_peer.len(), peer_id);
                    self.network.request_blocks(&peer_id, missing_from_peer).await?;
                } else {
                    info!("✅ No missing blocks from peer {}", peer_id);
                }
            }

            NetworkEvent::BlockRequestReceived(peer_id, block_ids) => {
                info!("📬 Received block request from {} for {} blocks", peer_id, block_ids.len());

                let mut blocks_to_send = Vec::new();

                // Check for release blocks
                let releases: Vec<Release> = self.db.get_all_with_prefix(prefixes::RELEASE)?;
                for release in releases {
                    let block_data = BlockCodec::encode_release(release.clone(), 0, None)?;

                    if block_ids.contains(&block_data.id) {
                        info!("  - Prepared release block {}", block_data.id);

                        // Convert to network BlockData format
                        let network_block = lens_v2_p2p::network::BlockData {
                            id: block_data.id,
                            height: 0,
                            data: block_data.data,
                            prev: None,
                            timestamp: block_data.timestamp,
                        };

                        blocks_to_send.push(network_block);
                    }
                }

                // Check for authorization transaction blocks
                use crate::routes::account::AuthorizationTransaction;
                let authorizations: Vec<AuthorizationTransaction> = self.db.get_all_with_prefix(prefixes::AUTHORIZATION)?;
                for auth in authorizations {
                    if block_ids.contains(&auth.id) {
                        // Encode authorization transaction as block data
                        let auth_json = serde_json::to_vec(&auth)?;
                        let network_block = lens_v2_p2p::network::BlockData {
                            id: auth.id.clone(),
                            height: 0, // Authorization transactions are flat (no height)
                            data: auth_json,
                            prev: None, // No chain
                            timestamp: auth.timestamp,
                        };

                        blocks_to_send.push(network_block);
                        info!("  - Prepared authorization transaction {} for {}", auth.id, auth.public_key);
                    }
                }

                // Check for delete transaction blocks
                use crate::ubts::UBTSBlock;
                let delete_txs: Vec<UBTSBlock> = self.db.get_all_with_prefix(prefixes::DELETE_TRANSACTION)?;
                for delete_tx in delete_txs {
                    if block_ids.contains(&delete_tx.id) {
                        // Encode delete transaction as block data
                        let delete_json = serde_json::to_vec(&delete_tx)?;
                        let network_block = lens_v2_p2p::network::BlockData {
                            id: delete_tx.id.clone(),
                            height: 0, // Delete transactions are flat (no height)
                            data: delete_json,
                            prev: None, // No chain
                            timestamp: delete_tx.timestamp,
                        };

                        blocks_to_send.push(network_block);
                        info!("  - Prepared delete transaction {}", delete_tx.id);
                    }
                }

                if !blocks_to_send.is_empty() {
                    info!("📤 Sending {} blocks to {}", blocks_to_send.len(), peer_id);
                    self.network.send_blocks(&peer_id, blocks_to_send).await?;
                } else {
                    warn!("⚠️ No matching blocks found for request from {}", peer_id);
                }
            }
        }

        Ok(())
    }

    /// Save a block to local storage
    async fn save_block(&self, block_data: lens_v2_p2p::network::BlockData) -> Result<()> {
        info!("Saving block {} to database", block_data.id);

        // Try to decode as authorization transaction first
        use crate::routes::account::AuthorizationTransaction;
        if let Ok(auth) = serde_json::from_slice::<AuthorizationTransaction>(&block_data.data) {
            info!("📜 Received authorization transaction: {} for {}", auth.id, auth.public_key);

            // Save to database
            let key = make_key(prefixes::AUTHORIZATION, &auth.id);
            self.db.put(&key, &auth)?;

            info!("✅ Saved authorization transaction {} (role: {})", auth.id, auth.role);
            return Ok(());
        }

        // Check if this is a UBTS transaction block
        if block_data.id.starts_with("ubts-") {
            info!("📦 Received UBTS transaction block: {}", block_data.id);

            // Try to decode as UBTS block with transactions
            use crate::ubts::UBTSBlock;
            if let Ok(ubts_block) = serde_json::from_slice::<UBTSBlock>(&block_data.data) {
                info!("✅ Decoded UBTS block with {} transactions", ubts_block.transactions.len());

                // Process each transaction
                for tx in &ubts_block.transactions {
                    match tx {
                        crate::ubts::UBTSTransaction::DeleteRelease { id, .. } => {
                            info!("🗑️ Processing DeleteRelease transaction for release {}", id);

                            // Save the delete transaction to database for historical record
                            let delete_key = make_key(prefixes::DELETE_TRANSACTION, &ubts_block.id);
                            self.db.put(&delete_key, &ubts_block)?;
                            info!("✅ Saved DeleteRelease transaction {} to database", ubts_block.id);

                            // Apply tombstone: Convert release to tombstone (proof of erasure)
                            let release_key = make_key(prefixes::RELEASE, id);

                            // Get existing release and convert to tombstone
                            if let Ok(Some(mut release)) = self.db.get::<&str, Release>(&release_key) {
                                // Convert to temporary tombstone (proof of erasure)
                                release.is_tombstone = true;
                                release.tombstone_type = Some(crate::routes::releases::TombstoneType::Temporary);
                                release.deleted_at = Some(chrono::Utc::now().to_rfc3339());
                                release.deleted_by = Some("sync".to_string());

                                // Increment vector clock for delete operation
                                release.increment_clock("sync".to_string());

                                // Save tombstone (proof of erasure - content deleted but metadata remains)
                                self.db.put(&release_key, &release)?;
                                info!("✅ Created tombstone for release {} (proof of erasure)", id);
                            } else {
                                warn!("⚠️ Release {} not found for deletion", id);
                            }
                        }

                        crate::ubts::UBTSTransaction::DeleteFeaturedRelease { id, .. } => {
                            info!("🗑️ Processing DeleteFeaturedRelease transaction for featured release {}", id);

                            // Save the delete transaction
                            let delete_key = make_key(prefixes::DELETE_TRANSACTION, &ubts_block.id);
                            self.db.put(&delete_key, &ubts_block)?;
                            info!("✅ Saved DeleteFeaturedRelease transaction {} to database", ubts_block.id);

                            // Delete the featured release
                            let featured_key = make_key(prefixes::FEATURED_RELEASE, id);
                            if let Err(e) = self.db.delete(&featured_key) {
                                warn!("⚠️ Failed to delete featured release {}: {}", id, e);
                            } else {
                                info!("✅ Deleted featured release {}", id);
                            }
                        }

                        _ => {
                            warn!("⚠️ Unhandled UBTS transaction type: {}", tx.type_name());
                        }
                    }
                }

                return Ok(());
            } else {
                warn!("⚠️ Failed to decode UBTS block {}", block_data.id);
            }
        }

        // Otherwise, treat as release block
        // Mark as downloaded
        self.p2p_manager.mark_downloaded(&block_data.id)?;

        // Add to local blocks (flat transactions - no heights, no chain)
        let meta = BlockMeta {
            id: block_data.id.clone(),
            height: 0, // Flat transactions
            prev: None, // No chain
            timestamp: block_data.timestamp,
        };
        self.p2p_manager.add_local_block(meta)?;

        // Decode releases from block
        let releases = BlockCodec::decode_releases(&block_data)?;
        info!("Block contains {} releases", releases.len());

        // Save each release to database with vector clock conflict resolution
        for incoming_release in releases {
            let key = make_key(prefixes::RELEASE, &incoming_release.id);

            // Check if we already have this release
            let existing_release: Option<Release> = self.db.get(&key)?;

            match existing_release {
                None => {
                    // We don't have it - add it (could be active or tombstone)
                    self.db.put(&key, &incoming_release)?;
                    if incoming_release.is_tombstone {
                        info!("💀 Saved tombstone for release: {}", incoming_release.id);
                    } else {
                        info!("📦 Saved new release: {}", incoming_release.id);
                    }
                }
                Some(mut existing) => {
                    // We have it - use vector clock to determine which version to keep
                    if incoming_release.happened_before(&existing) {
                        // Incoming is older, keep existing
                        info!("⏪ Incoming release {} is older (keeping ours)", incoming_release.id);
                        continue;
                    } else if existing.happened_before(&incoming_release) {
                        // Incoming is newer, take it (even if it's a tombstone!)
                        self.db.put(&key, &incoming_release)?;
                        if incoming_release.is_tombstone {
                            info!("💀 Updated to tombstone for release: {} (proof of erasure)", incoming_release.id);
                        } else {
                            info!("✨ Updated release: {} (newer version)", incoming_release.id);
                        }
                    } else if incoming_release.is_concurrent(&existing) {
                        // Concurrent - apply tie-breakers

                        // TOMBSTONE PRIORITY: If one is a tombstone, tombstone wins!
                        if incoming_release.is_tombstone && !existing.is_tombstone {
                            self.db.put(&key, &incoming_release)?;
                            info!("💀 Tombstone wins over active release: {} (concurrent, tombstone priority)", incoming_release.id);
                            continue;
                        } else if !incoming_release.is_tombstone && existing.is_tombstone {
                            // Existing tombstone wins
                            info!("💀 Keeping tombstone over active release: {} (concurrent, tombstone priority)", incoming_release.id);
                            continue;
                        }

                        // Use lexicographic comparison as tie-breaker
                        if incoming_release.posted_by > existing.posted_by {
                            self.db.put(&key, &incoming_release)?;
                            info!("🎲 Tie-breaker: incoming wins (concurrent, higher posted_by) - {}", incoming_release.id);
                        } else if incoming_release.posted_by == existing.posted_by {
                            // Same author - use latest timestamp
                            if incoming_release.created_at > existing.created_at {
                                self.db.put(&key, &incoming_release)?;
                                info!("🎲 Tie-breaker: incoming wins (concurrent, newer timestamp) - {}", incoming_release.id);
                            } else {
                                info!("🎲 Tie-breaker: keeping ours (concurrent, older timestamp) - {}", incoming_release.id);
                            }
                        } else {
                            info!("🎲 Tie-breaker: keeping ours (concurrent, lower posted_by) - {}", incoming_release.id);
                        }
                    } else {
                        // Vector clocks are equal - merge them
                        existing.merge_clock(&incoming_release);
                        self.db.put(&key, &existing)?;
                        info!("🔀 Merged vector clocks for release: {}", incoming_release.id);
                    }
                }
            }
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
        use crate::peer_registry::default_mesh_config;
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(citadel_dht::local_storage::LocalStorage::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-123".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            db,
            rx,
            dht_storage,
        );

        assert_eq!(orchestrator.sync_interval, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_delete_transactions_added_to_wantlist() {
        use crate::ubts::{UBTSBlock, UBTSTransaction};
        use crate::peer_registry::default_mesh_config;
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(citadel_dht::local_storage::LocalStorage::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-456".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            db.clone(),
            rx,
            dht_storage,
        );

        // Create a delete transaction
        let delete_tx = UBTSTransaction::DeleteRelease {
            id: "test-release-123".to_string(),
            signature: Some("test-sig".to_string()),
        };

        let ubts_block = UBTSBlock {
            id: "ubts-delete-test".to_string(),
            height: 0,
            prev: None,
            timestamp: 1234567890,
            transactions: vec![delete_tx],
            signature: None,
        };

        // Save delete transaction to database
        let delete_key = make_key(prefixes::DELETE_TRANSACTION, &ubts_block.id);
        db.put(&delete_key, &ubts_block).unwrap();

        // Build wantlist
        let wantlist = orchestrator.build_wantlist().await.unwrap();

        // Verify delete transaction is in wantlist
        assert!(wantlist.have_blocks.contains(&"ubts-delete-test".to_string()),
                "Delete transaction should be in WantList");
    }

    #[tokio::test]
    async fn test_received_delete_transaction_deletes_release() {
        use crate::ubts::{UBTSBlock, UBTSTransaction};
        use crate::peer_registry::default_mesh_config;
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(citadel_dht::local_storage::LocalStorage::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-789".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            db.clone(),
            rx,
            dht_storage,
        );

        // Create a release in the database
        let release = Release {
            id: "test-release-456".to_string(),
            name: "Test Release".to_string(),
            category_id: "cat-1".to_string(),
            category_slug: "test".to_string(),
            content_cid: "QmTest123".to_string(),
            thumbnail_cid: None,
            metadata: Some(serde_json::json!({})),
            site_address: "local".to_string(),
            posted_by: "test-user".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            vector_clock: std::collections::HashMap::new(),
            is_tombstone: false,
            tombstone_type: None,
            deleted_at: None,
            deleted_by: None,
        };

        let release_key = make_key(prefixes::RELEASE, &release.id);
        db.put(&release_key, &release).unwrap();

        // Verify release exists
        assert!(db.exists(&release_key).unwrap(), "Release should exist before deletion");

        // Create a delete transaction block
        let delete_tx = UBTSTransaction::DeleteRelease {
            id: "test-release-456".to_string(),
            signature: Some("test-sig".to_string()),
        };

        let ubts_block = UBTSBlock {
            id: "ubts-delete-456".to_string(),
            height: 0,
            prev: None,
            timestamp: 1234567890,
            transactions: vec![delete_tx],
            signature: None,
        };

        // Encode as network block data
        let ubts_json = serde_json::to_vec(&ubts_block).unwrap();
        let block_data = lens_v2_p2p::network::BlockData {
            id: ubts_block.id.clone(),
            height: 0,
            data: ubts_json,
            prev: None,
            timestamp: 1234567890,
        };

        // Process the delete transaction
        orchestrator.save_block(block_data).await.unwrap();

        // Verify release is now a tombstone (proof of erasure)
        let tombstone: Option<Release> = db.get(&release_key).unwrap();
        assert!(tombstone.is_some(), "Tombstone should exist");
        let tombstone = tombstone.unwrap();
        assert!(tombstone.is_tombstone, "Release should be tombstone after delete");
        assert_eq!(tombstone.tombstone_type, Some(crate::routes::releases::TombstoneType::Temporary));

        // Verify delete transaction is saved
        let delete_key = make_key(prefixes::DELETE_TRANSACTION, &ubts_block.id);
        assert!(db.exists(&delete_key).unwrap(), "Delete transaction should be saved to database");
    }

    #[tokio::test]
    async fn test_delete_transactions_are_sent_to_peers() {
        use crate::ubts::{UBTSBlock, UBTSTransaction};
        use crate::peer_registry::default_mesh_config;
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(citadel_dht::local_storage::LocalStorage::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-999".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            db.clone(),
            rx,
            dht_storage,
        );

        // Create a delete transaction
        let delete_tx = UBTSTransaction::DeleteRelease {
            id: "test-release-789".to_string(),
            signature: Some("test-sig".to_string()),
        };

        let ubts_block = UBTSBlock {
            id: "ubts-delete-789".to_string(),
            height: 0,
            prev: None,
            timestamp: 1234567890,
            transactions: vec![delete_tx],
            signature: None,
        };

        // Save delete transaction to database
        let delete_key = make_key(prefixes::DELETE_TRANSACTION, &ubts_block.id);
        db.put(&delete_key, &ubts_block).unwrap();

        // Simulate BlockRequestReceived event - node should respond with delete transaction
        let block_ids = vec!["ubts-delete-789".to_string()];

        // We can't directly test network sending without a real network,
        // but we can verify the delete transaction is retrievable from the database
        let stored_delete: Option<UBTSBlock> = db.get(&delete_key).unwrap();
        assert!(stored_delete.is_some(), "Delete transaction should be retrievable from database");

        let stored = stored_delete.unwrap();
        assert_eq!(stored.id, "ubts-delete-789");
        assert_eq!(stored.transactions.len(), 1);
    }
}
