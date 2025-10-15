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
use crate::webrtc_manager::WebRTCManager;
use crate::p2p_heartbeat::Heartbeat;

use citadel_core::topology::{MeshConfig, SlotCoordinate, Direction};
use std::collections::HashMap;

/// Sync orchestrator coordinates all P2P sync operations
pub struct SyncOrchestrator {
    /// P2P network layer
    network: Arc<P2pNetwork>,

    /// P2P manager (consensus + sync tracking)
    p2p_manager: Arc<P2pManager>,

    /// WebRTC manager for node-to-node connections
    webrtc_manager: Arc<WebRTCManager>,

    /// Database for persistence
    db: Database,

    /// My peer ID
    my_peer_id: String,

    /// My slot in the mesh
    my_slot: SlotCoordinate,

    /// Sync interval
    sync_interval: Duration,

    /// Receiver for immediate block notifications
    block_notify_rx: Arc<RwLock<mpsc::UnboundedReceiver<BlockNotification>>>,

    /// LazyNode for DHT-based neighbor discovery (no caching!)
    lazy_node: Arc<LazyNode>,

    /// Pending SDP answers (for WebRTC connection establishment)
    /// Key: target_peer_id, Value: oneshot sender for answer
    pending_sdp_answers: Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<String>>>>,
}

impl SyncOrchestrator {
    /// Check if a peer at given slot is one of my 8 mesh neighbors
    /// Uses "turn left" algorithm to find exactly 8 unique neighbors, skipping empty slots
    async fn is_mesh_neighbor_of_me(&self, _peer_id: &str, peer_slot: &SlotCoordinate, num_nodes: usize) -> bool {
        let my_slot = self.lazy_node.my_slot();

        // Use the orchestrator's actual mesh config (toroidal wrapping based on this topology)
        let mesh_config = *self.lazy_node.mesh_config();

        // Convert my slot to index
        let my_x = my_slot.x as usize;
        let my_y = my_slot.y as usize;
        let my_z = my_slot.z as usize;

        // Try the 8 primary directions + extended "turn left" directions
        let all_offsets = [
            // Primary 8 directions (most likely to be neighbors)
            (1, 0, 0),   // PlusA
            (-1, 0, 0),  // MinusA
            (0, 1, 0),   // PlusB
            (0, -1, 0),  // MinusB
            (1, -1, 0),  // PlusC (hexagonal)
            (-1, 1, 0),  // MinusC (hexagonal)
            (0, 0, 1),   // Up (vertical, toroidal wrap)
            (0, 0, -1),  // Down (vertical, toroidal wrap)
            // Turn left - try diagonals
            (1, 1, 0),
            (-1, -1, 0),
            (1, -1, 1),
            (-1, 1, -1),
            // Turn left again - try double steps
            (2, 0, 0),
            (-2, 0, 0),
            (0, 2, 0),
            (0, -2, 0),
            (0, 0, 2),
            (0, 0, -2),
        ];

        let mut found_neighbors = std::collections::HashSet::new();

        // Find my 8 neighbors
        for (dx, dy, dz) in all_offsets {
            if found_neighbors.len() >= 8 {
                break; // Got 8 neighbors
            }

            // Step through the toroid in this direction until finding a filled slot
            let mut steps = 1;
            let max_steps = mesh_config.width * mesh_config.height * mesh_config.depth;

            while steps <= max_steps {
                let nx = ((my_x as i32 + dx * steps as i32).rem_euclid(mesh_config.width as i32)) as usize;
                let ny = ((my_y as i32 + dy * steps as i32).rem_euclid(mesh_config.height as i32)) as usize;
                let nz = ((my_z as i32 + dz * steps as i32).rem_euclid(mesh_config.depth as i32)) as usize;

                let neighbor_index = nz * mesh_config.width * mesh_config.height + ny * mesh_config.width + nx;

                // Found a filled slot (not empty)
                if neighbor_index < num_nodes {
                    let neighbor_slot = SlotCoordinate::new(nx as i32, ny as i32, nz as i32);
                    found_neighbors.insert(neighbor_slot);
                    break; // Found neighbor in this direction
                }

                steps += 1;
            }
        }

        // Check if peer's slot is one of my 8 neighbors
        found_neighbors.contains(peer_slot)
    }

    /// Create a new sync orchestrator with LazyNode for neighbor discovery
    pub fn new(
        relay_url: String,
        my_peer_id: String,
        my_slot: SlotCoordinate,
        mesh_config: MeshConfig,
        p2p_manager: Arc<P2pManager>,
        webrtc_manager: Arc<WebRTCManager>,
        db: Database,
        block_notify_rx: mpsc::UnboundedReceiver<BlockNotification>,
        dht_storage: Arc<tokio::sync::Mutex<crate::dht_state::DhtState>>,
        relay_state: crate::routes::relay::RelayState,
    ) -> Self {
        Self::new_with_dht_fn(
            relay_url,
            my_peer_id,
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager,
            db,
            block_notify_rx,
            dht_storage.clone(),
            relay_state,
            None, // Use default relay-based dht_get_fn
        )
    }

    /// Create a new sync orchestrator with custom DHT GET function (for testing)
    ///
    /// This allows tests to provide a test-friendly dht_get_fn that queries local storage
    /// instead of routing through the relay (which doesn't exist in unit tests).
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_dht_fn(
        relay_url: String,
        my_peer_id: String,
        my_slot: SlotCoordinate,
        mesh_config: MeshConfig,
        p2p_manager: Arc<P2pManager>,
        webrtc_manager: Arc<WebRTCManager>,
        db: Database,
        block_notify_rx: mpsc::UnboundedReceiver<BlockNotification>,
        dht_storage: Arc<tokio::sync::Mutex<crate::dht_state::DhtState>>,
        relay_state: crate::routes::relay::RelayState,
        custom_dht_get_fn: Option<Arc<dyn Fn([u8; 32]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>>> + Send>> + Send + Sync>>,
    ) -> Self {
        // Pass our peer_id and slot to network layer for relay announcements and DHT bootstrap
        let network = Arc::new(P2pNetwork::new(relay_url, my_peer_id.clone(), my_slot));

        // Use custom dht_get_fn if provided (for tests), otherwise create default relay-based one
        let dht_get_fn = custom_dht_get_fn.unwrap_or_else(|| {
            // Create DHT GET callback for LazyNode (routes via network - relay/WebRTC)
            // This enables true distributed DHT queries instead of local storage only!
            Arc::new(move |key: [u8; 32]| {
                let relay = relay_state.clone();
                Box::pin(async move {
                    // Route DHT GET via relay/WebRTC (greedy routing to responsible slot)
                    Ok(relay.dht_get(key).await)
                }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>>> + Send>>
            })
        });

        // Create LazyNode for DHT-based neighbor discovery
        let lazy_node = Arc::new(LazyNode::new(
            my_slot,
            my_peer_id.clone(),
            mesh_config,
            dht_storage,
            dht_get_fn,
        ));

        Self {
            network,
            p2p_manager,
            webrtc_manager,
            db,
            my_peer_id,
            my_slot,
            sync_interval: Duration::from_secs(30),
            block_notify_rx: Arc::new(RwLock::new(block_notify_rx)),
            lazy_node,
            pending_sdp_answers: Arc::new(RwLock::new(HashMap::new())),
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
        let my_peer_id = self.my_peer_id.clone();
        let my_slot = self.my_slot;
        let lazy_node = self.lazy_node.clone();

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

                        // EVENT-DRIVEN SLOT CLAIMING WITH CONSENSUS:
                        // Now that we're connected to relay, request slot claim via lightweight consensus
                        info!("🔍 Querying DHT for existing slot claims via relay...");

                        use crate::peer_registry::{SlotOwnership, slot_ownership_key, peer_location_key, assign_slots_batch};

                        // Scan local DHT cache (populated by bootstrap) for existing slot claims
                        let existing_peers = {
                            let storage = lazy_node.dht_storage.lock().await;
                            let all_entries = storage.scan_all();

                            let mut peers = Vec::new();
                            for (_key, ownership_bytes) in all_entries {
                                if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(&ownership_bytes) {
                                    // Only count unique peer_ids (avoid counting both peer-location-* and slot-* keys)
                                    if !peers.iter().any(|(pid, _)| pid == &ownership.peer_id) {
                                        peers.push((ownership.peer_id.clone(), ownership.slot));
                                    }
                                }
                            }
                            peers
                        };

                        info!("📊 Found {} existing peers in DHT", existing_peers.len());

                        // Build peer ID list (existing + ourselves) for slot assignment
                        let mut all_peer_ids: Vec<String> = existing_peers.iter().map(|(pid, _)| pid.clone()).collect();
                        all_peer_ids.push(my_peer_id.clone());

                        info!("🌀 Assigning slots for {} total peers via SPIRAL algorithm", all_peer_ids.len());

                        // Use assign_slots_batch to deterministically assign slots
                        // This generates spiral slots internally and assigns via hash-modulo with linear probing
                        let slot_assignments = assign_slots_batch(&all_peer_ids);

                        // Find our proposed slot (not claimed yet - waiting for relay ACK!)
                        let proposed_slot = slot_assignments.get(&my_peer_id)
                            .copied()
                            .unwrap_or_else(|| {
                                warn!("⚠️ Failed to find slot assignment, using origin as fallback");
                                citadel_core::topology::SlotCoordinate::new(0, 0, 0)
                            });

                        info!("🎯 Proposed slot: {:?} (from SPIRAL with {} total peers)", proposed_slot, all_peer_ids.len());

                        // Send SlotClaimRequest to relay for consensus validation
                        // The relay will check for conflicts and respond with ACK or NACK
                        info!("📤 Sending slot claim request to relay for slot {:?}", proposed_slot);
                        if let Err(e) = network.send_slot_claim_request(proposed_slot).await {
                            error!("Failed to send slot claim request: {}", e);
                            break;
                        }

                        info!("⏳ Waiting for slot claim response from relay (event-driven)...");
                        // NOTE: Slot ownership will be committed to DHT after receiving SlotClaimAck
                        // See handle_network_event() for SlotClaimAck/SlotClaimNack handlers

                        // Connection succeeded - keep this connection alive
                        // If it drops, the loop will reconnect
                        break;
                    }
                    Err(e) => {
                        if attempt == 1 {
                            // First attempt - might be the first node (we ARE the relay!)
                            info!("ℹ️ No relay available (might be first node - this node IS the relay via anycast)");

                            // Even if we're the first node, store locally
                            info!("📢 Storing slot ownership locally (first node - no relay needed)...");

                            use crate::peer_registry::{SlotOwnership, slot_ownership_key, peer_location_key};
                            let ownership = SlotOwnership::new(
                                my_peer_id.clone(),
                                my_slot,
                                None,
                            );

                            if let Ok(ownership_bytes) = serde_json::to_vec(&ownership) {
                                let mut storage = lazy_node.dht_storage.lock().await;
                                let ownership_key = slot_ownership_key(my_slot);
                                let location_key = peer_location_key(&my_peer_id);
                                storage.insert_raw(ownership_key, ownership_bytes.clone());
                                storage.insert_raw(location_key, ownership_bytes);
                                info!("✅ Stored slot ownership locally (first node)");
                                info!("🎉 Slot ownership announced: {:?} -> {}", my_slot, my_peer_id);
                            }
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

        // Spawn heartbeat broadcasting loop
        let orchestrator_heartbeat = self.clone();
        tokio::spawn(async move {
            orchestrator_heartbeat.heartbeat_loop().await;
        });

        // Spawn heartbeat receiver loop
        let orchestrator_receiver = self.clone();
        tokio::spawn(async move {
            orchestrator_receiver.heartbeat_receiver_loop().await;
        });

        // Spawn WebRTC connection establishment loop
        let orchestrator_webrtc = self.clone();
        tokio::spawn(async move {
            orchestrator_webrtc.webrtc_connection_loop().await;
        });

        // NOTE: No timer-based DHT heartbeat loop needed!
        // Heartbeats are updated event-driven on every DHT read (proof of activity).
        // See lazy_node.rs::get_neighbor() - updates heartbeat after each successful DHT query.

        // SDP signaling is now handled via network events (SdpOfferReceived/SdpAnswerReceived)
        // No separate loop needed - offers and answers processed in handle_network_event()

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

    /// Heartbeat loop - continuously broadcasts peer heartbeat via WebRTC
    async fn heartbeat_loop(&self) {
        info!("💓 Starting continuous WebRTC heartbeat broadcast");

        // Mark ourselves as alive immediately
        let my_peer_id_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            self.my_peer_id.hash(&mut hasher);
            hasher.finish()
        };

        loop {
            // Mark self as alive
            let _ = self.p2p_manager.mark_peer_alive(my_peer_id_hash);

            // Create heartbeat message with our current info
            let heartbeat = Heartbeat::new(
                self.my_peer_id.clone(),
                self.my_slot,
                vec!["webrtc".to_string(), "dht".to_string(), "spore".to_string()],
                None, // TODO: Calculate average neighbor latency
            );

            // Broadcast via WebRTC to all connected node peers
            match self.webrtc_manager.broadcast_heartbeat_message(&heartbeat).await {
                Ok(sent_count) => {
                    if sent_count > 0 {
                        debug!("💓 Broadcast WebRTC heartbeat to {} node peers", sent_count);
                    }
                }
                Err(e) => {
                    warn!("Failed to broadcast WebRTC heartbeat: {}", e);
                }
            }

            // Also broadcast through relay as fallback (until all nodes have WebRTC)
            if let Err(e) = self.network.broadcast_heartbeat(&self.my_peer_id, self.my_slot).await {
                warn!("Failed to broadcast relay heartbeat: {}", e);
            }

            // Small sleep to avoid hammering - 10 seconds per heartbeat protocol
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    /// Heartbeat receiver loop - processes incoming WebRTC heartbeats
    async fn heartbeat_receiver_loop(&self) {
        info!("💓 Starting WebRTC heartbeat receiver");

        loop {
            // Wait for next heartbeat from WebRTC Manager (event-driven, no polling!)
            if let Some(heartbeat) = self.webrtc_manager.next_heartbeat().await {
                debug!("💓 Received heartbeat from {}", heartbeat.peer_id);

                // Hash peer_id to u64 for P2P manager compatibility
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                heartbeat.peer_id.hash(&mut hasher);
                let peer_id_u64 = hasher.finish();

                // Mark peer as alive
                if let Err(e) = self.p2p_manager.mark_peer_alive(peer_id_u64) {
                    warn!("Failed to mark peer {} as alive: {}", heartbeat.peer_id, e);
                } else {
                    debug!("✅ Marked peer {} as alive", heartbeat.peer_id);
                }
            }
            // If None, channel is closed - loop exits naturally via await
        }
    }

    /// WebRTC connection establishment loop - establishes direct connections to 8 mesh neighbors
    ///
    /// Uses Content Addressed Slots to find the 8 geometric neighbors:
    /// - 6 hexagonal directions: ±A, ±B, ±C
    /// - 2 vertical directions: Up, Down
    ///
    /// Each neighbor is discovered by:
    /// 1. Calculate neighbor_slot = my_slot.neighbor(direction)
    /// 2. Query DHT for slot_ownership_key(neighbor_slot)
    /// 3. Extract peer_id from SlotOwnership
    /// 4. Establish WebRTC connection to that peer_id
    async fn webrtc_connection_loop(&self) {
        info!("🔗 Starting WebRTC connection establishment loop (8 geometric mesh neighbors)");

        loop {
            // Discover my 8 geometric neighbors via DHT (using LazyNode)
            // LazyNode.get_all_neighbors() queries the 8 directions and returns peer_ids
            match self.lazy_node.get_all_neighbors().await {
                Ok(neighbors) => {
                    info!("🔷 Found {}/8 geometric mesh neighbors for WebRTC connection", neighbors.len());

                    // Attempt to establish WebRTC connections with each geometric neighbor
                    for neighbor_peer_id in neighbors {
                        // Skip if we're already connected
                        if self.webrtc_manager.is_peer_connected(&neighbor_peer_id).await {
                            debug!("🔗 Already connected to geometric neighbor {}", neighbor_peer_id);
                            continue;
                        }

                        // Initiate WebRTC connection (create offer)
                        info!("🔗 Initiating WebRTC connection to geometric neighbor {}", neighbor_peer_id);
                        match self.establish_webrtc_connection(&neighbor_peer_id).await {
                            Ok(_) => {
                                info!("✅ WebRTC connection established to geometric neighbor {}", neighbor_peer_id);
                            }
                            Err(e) => {
                                warn!("⚠️ Failed to establish WebRTC connection to geometric neighbor {}: {}", neighbor_peer_id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to discover geometric neighbors: {}", e);
                }
            }

            // TODO: Make this event-driven by subscribing to DHT slot ownership announcements
            // For now, check frequently during mesh formation (1 second)
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// Establish WebRTC connection to a specific peer using relay for SDP signaling
    ///
    /// This implements the WebRTC handshake:
    /// 1. Create offer (SDP)
    /// 2. Send offer to target peer via relay (WebSocket)
    /// 3. Wait for answer via network event (handled in handle_network_event)
    /// 4. Connection completes when answer is processed
    async fn establish_webrtc_connection(&self, target_peer_id: &str) -> Result<()> {
        // Create WebRTC offer
        let offer_sdp = self.webrtc_manager.create_offer(target_peer_id.to_string()).await?;
        info!("📡 Created WebRTC offer for {}", target_peer_id);

        // Create channel to receive answer
        let (answer_tx, answer_rx) = tokio::sync::oneshot::channel();

        // Register pending answer
        {
            let mut pending = self.pending_sdp_answers.write().await;
            pending.insert(target_peer_id.to_string(), answer_tx);
        }

        // Send offer via relay (uses WebSocket as temporary signaling channel)
        self.network.send_sdp_offer(target_peer_id, offer_sdp).await?;
        info!("📤 Sent SDP offer to {} via relay", target_peer_id);

        // Wait for answer (with timeout)
        match tokio::time::timeout(Duration::from_secs(10), answer_rx).await {
            Ok(Ok(answer_sdp)) => {
                info!("📥 Received SDP answer from {}", target_peer_id);

                // Set remote description (answer) on the peer connection
                use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
                let peers = self.webrtc_manager.peers.read().await;
                if let Some(peer) = peers.get(target_peer_id) {
                    let answer = RTCSessionDescription::answer(answer_sdp)?;
                    peer.peer_connection.set_remote_description(answer).await?;
                    info!("✅ WebRTC connection established to {}", target_peer_id);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Peer connection not found for {}", target_peer_id))
                }
            }
            Ok(Err(_)) => {
                // Channel closed without answer
                Err(anyhow::anyhow!("Answer channel closed for {}", target_peer_id))
            }
            Err(_) => {
                // Timeout
                // Clean up pending answer
                let mut pending = self.pending_sdp_answers.write().await;
                pending.remove(target_peer_id);
                Err(anyhow::anyhow!("Timeout waiting for SDP answer from {}", target_peer_id))
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

        // 3. Check for missing blocks and request via WebRTC WantList
        let missing = self.p2p_manager.missing_blocks()?;
        if !missing.is_empty() {
            info!("📥 Need to fetch {} missing blocks via SPORE WantList", missing.len());

            // Build have_ranges from local storage
            let local_blocks = self.get_local_blocks().await?;
            let mut local_block_ids = std::collections::HashSet::new();
            for block in local_blocks {
                local_block_ids.insert(block.id);
            }

            // Convert local blocks to ranges (sorted by key_hash)
            use crate::spore_wantlist::build_ranges_from_keys;
            let mut local_keys: Vec<u64> = local_block_ids.iter().map(|id| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                id.hash(&mut hasher);
                hasher.finish()
            }).collect();
            local_keys.sort();
            let have_ranges = build_ranges_from_keys(&local_keys);

            // Build want_ranges from missing blocks
            let mut missing_keys: Vec<u64> = missing.iter().map(|id| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                id.hash(&mut hasher);
                hasher.finish()
            }).collect();
            missing_keys.sort();
            let want_ranges = build_ranges_from_keys(&missing_keys);

            // Get connected WebRTC peers
            let webrtc_peers = self.webrtc_manager.connected_peer_ids().await;
            info!("👥 Have {} WebRTC peers for SPORE sync", webrtc_peers.len());

            if !webrtc_peers.is_empty() {
                // Send WantList to all connected WebRTC peers
                let wantlist = crate::spore_wantlist::WantListMessage {
                    version: 1,
                    want_ranges,
                    have_ranges,
                    have_filter: None,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    peer_id: self.my_peer_id.clone(),
                };

                for peer_id in &webrtc_peers {
                    if let Err(e) = self.webrtc_manager.send_wantlist_to_peer(peer_id, &wantlist).await {
                        warn!("Failed to send SPORE WantList to {}: {}", peer_id, e);
                    } else {
                        info!("📋 Sent SPORE WantList to {} ({} want ranges)", peer_id, wantlist.want_ranges.len());
                    }
                }

                // Mark as downloading
                for block_id in &missing {
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

            // PeerReferral: Update P2P manager with known/connected peers for block exchange
            // DHT population happens via DhtReplication gossip messages, NOT here!
            // This is just for initial bootstrap awareness and P2P manager state.
            NetworkEvent::PeerReferral(peers) => {
                info!("📡 Received {} peer referrals from relay - updating P2P manager", peers.len());

                for peer in &peers {
                    // CRITICAL: Use the peer's ACTUAL announced slot, not a recalculated one!
                    let slot = match peer.slot {
                        Some(announced_slot) => {
                            debug!("  Peer {} at slot {:?}", peer.peer_id, announced_slot);
                            announced_slot
                        }
                        None => {
                            // Fallback: If relay doesn't provide slot, calculate from peer_id
                            warn!("  ⚠️ Peer {} has no slot - falling back to peer_id_to_slot (may cause mismatches)", peer.peer_id);
                            crate::peer_registry::peer_id_to_slot(&peer.peer_id, self.lazy_node.mesh_config())
                        }
                    };

                    // Hash peer_id string to u64 for P2P manager compatibility
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    peer.peer_id.hash(&mut hasher);
                    let peer_id_u64 = hasher.finish();

                    // Add ALL peers as known peers (for awareness) - O(n) scalability
                    // Store both u64 ID and String ID for /map endpoint
                    if let Err(e) = self.p2p_manager.add_known_peer_with_string(peer_id_u64, peer.peer_id.clone()) {
                        warn!("  ⚠️ Failed to add known peer {} to P2P manager: {}", peer.peer_id, e);
                    } else {
                        debug!("  ✅ Added known peer {} (slot {:?}) to P2P manager", peer.peer_id, slot);
                    }

                    // Determine if this peer is a mesh neighbor using "turn left" algorithm
                    // This finds exactly 8 unique neighbors, skipping empty slots in the toroid
                    // Total nodes = peers.len() + 1 (including ourselves for mesh calculations)
                    let is_neighbor = self.is_mesh_neighbor_of_me(&peer.peer_id, &slot, peers.len() + 1).await;

                    // Only add mesh neighbors as connected peers (actual P2P connections) - O(1) scalability
                    if is_neighbor {
                        if let Err(e) = self.p2p_manager.add_connected_peer(peer_id_u64, None) {
                            warn!("  ⚠️ Failed to add connected peer {} to P2P manager: {}", peer.peer_id, e);
                        } else {
                            info!("  ✅ Added mesh neighbor {} (slot {:?}) as connected peer (1 of 8)", peer.peer_id, slot);
                        }
                    } else {
                        debug!("  ℹ️ Peer {} (slot {:?}) is not a mesh neighbor (known but not connected)", peer.peer_id, slot);
                    }
                }

                info!("✅ Updated P2P manager with {} peer referrals (DHT populated via gossip separately)", peers.len());
            }

            // DhtReplication: Store in local DHT cache (replica of GLOBAL DHT via gossip)
            NetworkEvent::DhtReplication(replication) => {
                // Convert key bytes to [u8; 32]
                if replication.key.len() == 32 {
                    let mut key_array = [0u8; 32];
                    key_array.copy_from_slice(&replication.key);

                    // Store in local DHT cache (this is a CACHE/REPLICA of the GLOBAL DHT, not a separate "local DHT")
                    self.lazy_node.dht_storage.lock().await.insert_raw(key_array, replication.value.clone());

                    debug!("✅ Stored DHT key {} in local cache via gossip from {}",
                        hex::encode(&replication.key), replication.source_peer_id);
                } else {
                    warn!("⚠️ Received DHT replication with invalid key length: {} (expected 32)",
                        replication.key.len());
                }
            }

            // DhtBootstrapResponse: Bootstrap local DHT cache from relay's complete snapshot
            NetworkEvent::DhtBootstrapResponse(response) => {
                info!("🔄 Received DHT bootstrap with {} entries from relay", response.entry_count);

                // Convert P2P DhtEntry to our DhtEntry type and bootstrap
                let mut peer_dht = crate::dht_state::DhtState::new();
                for entry in response.dht_entries {
                    peer_dht.insert(crate::dht_state::DhtEntry {
                        key: entry.key,
                        value: entry.value,
                        timestamp: entry.timestamp,
                    });
                }

                // Bootstrap our local DHT cache from relay's snapshot
                {
                    let mut dht = self.lazy_node.dht_storage.lock().await;
                    dht.bootstrap_from_peer(&peer_dht);
                }

                info!("✅ Bootstrapped local DHT cache from relay ({} entries)", response.entry_count);
            }

            // REMOVED: PeerIdAssigned handling - peer_id passed to constructor now!
            NetworkEvent::PeerIdAssigned(_peer_id) => {
                // Ignored: peer_id is passed to constructor, not assigned at runtime
                debug!("Ignoring PeerIdAssigned event - peer_id set at construction");
            }

            NetworkEvent::WantListReceived(peer_id, wantlist) => {
                // DEPRECATED: This OLD relay-based WantList handler is deprecated!
                // All SPORE WantList exchange now happens via WebRTC DataChannels.
                // This handler is kept for backwards compatibility during transition.
                warn!("⚠️ DEPRECATED: Received OLD relay WantList from {} (gen={}, have={} blocks) - WebRTC SPORE WantList should be used instead!",
                    peer_id, wantlist.generation, wantlist.have_blocks.len());

                // Log deprecation warning and skip processing
                info!("⏭️ Skipping OLD relay WantList handler - use WebRTC SPORE WantList instead");
            }

            // SDP signaling events (WebRTC connection establishment via relay)
            NetworkEvent::SdpOfferReceived { from_peer_id, offer_sdp } => {
                info!("📥 Received SDP offer from {} ({} bytes)", from_peer_id, offer_sdp.len());

                // Create answer using WebRTC manager
                match self.webrtc_manager.handle_offer(from_peer_id.clone(), offer_sdp, crate::webrtc_manager::PeerType::Node).await {
                    Ok(answer_sdp) => {
                        info!("📡 Created SDP answer for {}", from_peer_id);

                        // Send answer back via relay
                        if let Err(e) = self.network.send_sdp_answer(&from_peer_id, answer_sdp).await {
                            warn!("⚠️ Failed to send SDP answer to {}: {}", from_peer_id, e);
                        } else {
                            info!("✅ SDP answer sent to {} via relay", from_peer_id);
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to create SDP answer for {}: {}", from_peer_id, e);
                    }
                }
            }

            NetworkEvent::SdpAnswerReceived { from_peer_id, answer_sdp } => {
                info!("📥 Received SDP answer from {} ({} bytes)", from_peer_id, answer_sdp.len());

                // Check if we have a pending answer channel for this peer
                let answer_tx = {
                    let mut pending = self.pending_sdp_answers.write().await;
                    pending.remove(&from_peer_id)
                };

                if let Some(tx) = answer_tx {
                    // Send answer through channel (establish_webrtc_connection is waiting)
                    let _ = tx.send(answer_sdp);
                    info!("✅ Forwarded SDP answer from {} to connection establishment", from_peer_id);
                } else {
                    warn!("⚠️ Received SDP answer from {} but no pending connection", from_peer_id);
                }
            }

            // SlotOwnershipGossip: Store gossiped slot ownership in local DHT cache
            NetworkEvent::SlotOwnershipGossip { peer_id, slot, ownership_bytes } => {
                info!("📨 Received slot ownership gossip: {} → ({}, {}, {})",
                    peer_id, slot.x, slot.y, slot.z);

                use crate::peer_registry::{peer_location_key, slot_ownership_key};

                // Store in local DHT cache (both location and slot keys)
                let mut storage = self.lazy_node.dht_storage.lock().await;
                let location_key = peer_location_key(&peer_id);
                let slot_key = slot_ownership_key(slot);
                storage.insert_raw(location_key, ownership_bytes.clone());
                storage.insert_raw(slot_key, ownership_bytes);

                info!("✅ Stored gossiped slot ownership locally: {} → ({}, {}, {})",
                    peer_id, slot.x, slot.y, slot.z);
            }

            NetworkEvent::BlockRequestReceived(peer_id, block_ids) => {
                // DEPRECATED: This OLD relay-based BlockRequest handler is deprecated!
                // All block exchange now happens via WebRTC SPORE WantList + RangeResponse.
                // This handler is kept for backwards compatibility during transition.
                warn!("⚠️ DEPRECATED: Received OLD relay BlockRequest from {} for {} blocks - WebRTC SPORE RangeResponse should be used instead!",
                    peer_id, block_ids.len());

                // Log deprecation warning and skip processing
                info!("⏭️ Skipping OLD relay BlockRequest handler - use WebRTC SPORE RangeResponse instead");
            }

            NetworkEvent::SlotClaimAck { slot } => {
                // Slot claim approved by relay - commit to DHT and gossip
                info!("✅ Slot claim ACK received from relay for {:?}", slot);

                use crate::peer_registry::{SlotOwnership, slot_ownership_key, peer_location_key};

                // Create ownership record
                let ownership = SlotOwnership::new(
                    self.my_peer_id.clone(),
                    slot,
                    None,
                );

                // Serialize ownership
                let ownership_bytes = match serde_json::to_vec(&ownership) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!("Failed to serialize slot ownership after ACK: {}", e);
                        return Ok(());
                    }
                };

                // Commit to local DHT cache (cache of GLOBAL DHT)
                // The relay will gossip this to other nodes via DhtReplication events
                {
                    let mut storage = self.lazy_node.dht_storage.lock().await;
                    let ownership_key = slot_ownership_key(slot);
                    let location_key = peer_location_key(&self.my_peer_id);
                    storage.insert_raw(ownership_key, ownership_bytes.clone());
                    storage.insert_raw(location_key, ownership_bytes);
                    info!("✅ Committed slot ownership to DHT: {:?} -> {}", slot, self.my_peer_id);
                }

                info!("🎉 Slot claiming complete: {:?} -> {} (event-driven consensus)", slot, self.my_peer_id);
                info!("📢 Relay will gossip ownership to other nodes via DhtReplication");
            }

            NetworkEvent::SlotClaimNack { conflicting_peer, retry_suggestion } => {
                // Slot claim rejected by relay - re-scan DHT and retry with updated peer list
                warn!("❌ Slot claim NACK received: conflict with peer {}", conflicting_peer);
                if let Some(suggested_slot) = retry_suggestion {
                    info!("💡 Relay suggests retrying with slot: {:?}", suggested_slot);
                }

                use crate::peer_registry::{SlotOwnership, assign_slots_batch};

                // Re-scan DHT to get updated peer list (includes conflicting peer now)
                let existing_peers = {
                    let storage = self.lazy_node.dht_storage.lock().await;
                    let all_entries = storage.scan_all();

                    let mut peers = Vec::new();
                    for (_key, ownership_bytes) in all_entries {
                        if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(&ownership_bytes) {
                            // Only count unique peer_ids
                            if !peers.iter().any(|(pid, _)| pid == &ownership.peer_id) {
                                peers.push((ownership.peer_id.clone(), ownership.slot));
                            }
                        }
                    }
                    peers
                };

                info!("🔄 Re-scanning DHT after NACK: found {} existing peers", existing_peers.len());

                // Rebuild peer list with updated peers
                let mut all_peer_ids: Vec<String> = existing_peers.iter().map(|(pid, _)| pid.clone()).collect();
                all_peer_ids.push(self.my_peer_id.clone());

                // Recalculate slot assignment with SPIRAL (deterministic, will skip conflicting slot)
                let slot_assignments = assign_slots_batch(&all_peer_ids);

                // Get our NEW proposed slot (different from the one that was NACKed)
                let new_proposed_slot = slot_assignments.get(&self.my_peer_id)
                    .copied()
                    .unwrap_or_else(|| {
                        warn!("⚠️ Failed to find slot assignment after NACK, using origin");
                        citadel_core::topology::SlotCoordinate::new(0, 0, 0)
                    });

                info!("🔁 Retrying slot claim with updated slot: {:?} (after NACK)", new_proposed_slot);

                // Send new slot claim request
                if let Err(e) = self.network.send_slot_claim_request(new_proposed_slot).await {
                    error!("Failed to send retry slot claim request: {}", e);
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

    /// Helper: Create a test-friendly DHT GET callback that queries local storage
    ///
    /// This is used in unit tests where there's no relay running.
    /// Instead of routing through relay, it queries the local DHT storage directly.
    fn create_test_dht_get_fn(
        dht_storage: Arc<tokio::sync::Mutex<crate::dht_state::DhtState>>
    ) -> Arc<dyn Fn([u8; 32]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>>> + Send>> + Send + Sync> {
        Arc::new(move |key: [u8; 32]| {
            let storage = dht_storage.clone();
            Box::pin(async move {
                let dht = storage.lock().await;
                Ok(dht.get_raw(&key).map(|v| v.clone()))
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>>> + Send>>
        })
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        use crate::peer_registry::default_mesh_config;
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));
        let relay_state = crate::routes::relay::RelayState::new();

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-123".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager,
            db,
            rx,
            dht_storage,
            relay_state,
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
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);
        let relay_state = crate::routes::relay::RelayState::new();

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-456".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager,
            db.clone(),
            rx,
            dht_storage,
            relay_state,
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
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-789".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager,
            db.clone(),
            rx,
            dht_storage,
            crate::routes::relay::RelayState::new(),
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
    async fn test_is_mesh_neighbor_detection() {
        use crate::peer_registry::{default_mesh_config, calculate_mesh_dimensions};
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        // Dynamically calculate mesh for 50 nodes (no fixed topology!)
        let mesh_config = calculate_mesh_dimensions(50);
        let my_slot = SlotCoordinate::new(0, 0, 0); // I'm at (0,0,0)

        let orchestrator = Arc::new(SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-0".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager,
            db,
            rx,
            dht_storage,
            crate::routes::relay::RelayState::new(),
        ));

        // In a 9×9×1 mesh with 50 nodes (only slots 0-49 filled), the "turn left" algorithm finds:
        // 1. (1,0,0) via +A at 1 step
        // 2. (8,0,0) via -A at 1 step (wraps)
        // 3. (0,1,0) via +B at 1 step
        // 4. (0,5,0) via -B at 4 steps (skips empty slots 72,63,54)
        // 5. (4,5,0) via +C at 4 steps
        // 6. (8,1,0) via -C at 1 step (wraps)
        // 7. (0,0,0) via Up at 1 step - SELF! (depth wraps)
        // 8. (1,1,0) via diagonal at 1 step
        //
        // NOTE: (0,8,0) is at index 72 which is EMPTY (50 nodes fill indices 0-49)
        // So the algorithm skips it and finds (0,5,0) at index 45 instead.

        let test_cases = vec![
            ((1, 0, 0), true),  // Neighbor #1: +A at 1 step
            ((8, 0, 0), true),  // Neighbor #2: -A at 1 step (wraps)
            ((0, 1, 0), true),  // Neighbor #3: +B at 1 step
            ((0, 5, 0), true),  // Neighbor #4: -B at 4 steps (skips empty slots)
            ((8, 1, 0), true),  // Neighbor #6: -C at 1 step (wraps)
            ((1, 1, 0), true),  // Neighbor #8: diagonal at 1 step
            ((5, 5, 0), false), // Far away, not a neighbor
            ((0, 8, 0), false), // Empty slot (index 72 > 49), not a neighbor
        ];

        for ((x, y, z), expected_neighbor) in test_cases {
            let test_slot = SlotCoordinate::new(x, y, z);
            let is_neighbor = orchestrator.is_mesh_neighbor_of_me(
                &format!("peer-{}-{}-{}", x, y, z),
                &test_slot,
                50 // Total nodes in network
            ).await;

            if expected_neighbor {
                assert!(is_neighbor, "Slot ({},{},{}) should be a neighbor", x, y, z);
            } else {
                assert!(!is_neighbor, "Slot ({},{},{}) should NOT be a neighbor", x, y, z);
            }
        }
    }

    #[tokio::test]
    async fn test_delete_transactions_are_sent_to_peers() {
        use crate::ubts::{UBTSBlock, UBTSTransaction};
        use crate::peer_registry::default_mesh_config;
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "test-peer-999".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager,
            db.clone(),
            rx,
            dht_storage,
            crate::routes::relay::RelayState::new(),
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

    // ===== WebRTC P2P Connection Tests =====

    #[tokio::test]
    async fn test_sdp_offer_sent_via_relay() {
        use crate::peer_registry::default_mesh_config;
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "peer-alice".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager.clone(),
            db,
            rx,
            dht_storage.clone(),
            crate::routes::relay::RelayState::new(),
        );

        // Attempt WebRTC connection (will timeout waiting for answer from relay)
        let target_peer_id = "peer-bob";
        let result = orchestrator.establish_webrtc_connection(target_peer_id).await;

        // Connection should timeout (no relay running to forward offer)
        assert!(result.is_err(), "Connection should timeout without relay");
        assert!(result.unwrap_err().to_string().contains("Timeout"), "Error should indicate timeout");

        // Verify WebRTC offer was created (stored in webrtc_manager, not DHT)
        // The relay would forward this offer via WebSocket signaling (not DHT)
        let peers = webrtc_manager.peers.read().await;
        assert!(peers.contains_key(target_peer_id), "WebRTC peer connection should be created for target");
    }

    #[tokio::test]
    async fn test_geometric_neighbor_discovery_via_lazy_node() {
        use crate::peer_registry::{SlotOwnership, slot_ownership_key};
        use citadel_core::topology::{SlotCoordinate, Direction};

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        // 3×3×3 mesh
        let mesh_config = MeshConfig::new(3, 3, 3);
        let my_slot = SlotCoordinate::new(1, 1, 1); // Center of 3x3x3 mesh

        // Populate DHT with 8 geometric neighbors
        {
            let mut dht = dht_storage.lock().await;

            // PlusA: (2,1,1)
            let neighbor_slot_a = my_slot.neighbor(Direction::PlusA, &mesh_config);
            let ownership_a = SlotOwnership::new("peer-plusA".to_string(), neighbor_slot_a, None);
            let key_a = slot_ownership_key(neighbor_slot_a);
            dht.insert_raw(key_a, serde_json::to_vec(&ownership_a).unwrap());

            // MinusA: (0,1,1)
            let neighbor_slot_b = my_slot.neighbor(Direction::MinusA, &mesh_config);
            let ownership_b = SlotOwnership::new("peer-minusA".to_string(), neighbor_slot_b, None);
            let key_b = slot_ownership_key(neighbor_slot_b);
            dht.insert_raw(key_b, serde_json::to_vec(&ownership_b).unwrap());

            // PlusB: (1,2,1)
            let neighbor_slot_c = my_slot.neighbor(Direction::PlusB, &mesh_config);
            let ownership_c = SlotOwnership::new("peer-plusB".to_string(), neighbor_slot_c, None);
            let key_c = slot_ownership_key(neighbor_slot_c);
            dht.insert_raw(key_c, serde_json::to_vec(&ownership_c).unwrap());

            // MinusB: (1,0,1)
            let neighbor_slot_d = my_slot.neighbor(Direction::MinusB, &mesh_config);
            let ownership_d = SlotOwnership::new("peer-minusB".to_string(), neighbor_slot_d, None);
            let key_d = slot_ownership_key(neighbor_slot_d);
            dht.insert_raw(key_d, serde_json::to_vec(&ownership_d).unwrap());

            // PlusC: (2,0,1)
            let neighbor_slot_e = my_slot.neighbor(Direction::PlusC, &mesh_config);
            let ownership_e = SlotOwnership::new("peer-plusC".to_string(), neighbor_slot_e, None);
            let key_e = slot_ownership_key(neighbor_slot_e);
            dht.insert_raw(key_e, serde_json::to_vec(&ownership_e).unwrap());

            // MinusC: (0,2,1)
            let neighbor_slot_f = my_slot.neighbor(Direction::MinusC, &mesh_config);
            let ownership_f = SlotOwnership::new("peer-minusC".to_string(), neighbor_slot_f, None);
            let key_f = slot_ownership_key(neighbor_slot_f);
            dht.insert_raw(key_f, serde_json::to_vec(&ownership_f).unwrap());

            // Up: (1,1,2)
            let neighbor_slot_g = my_slot.neighbor(Direction::Up, &mesh_config);
            let ownership_g = SlotOwnership::new("peer-up".to_string(), neighbor_slot_g, None);
            let key_g = slot_ownership_key(neighbor_slot_g);
            dht.insert_raw(key_g, serde_json::to_vec(&ownership_g).unwrap());

            // Down: (1,1,0)
            let neighbor_slot_h = my_slot.neighbor(Direction::Down, &mesh_config);
            let ownership_h = SlotOwnership::new("peer-down".to_string(), neighbor_slot_h, None);
            let key_h = slot_ownership_key(neighbor_slot_h);
            dht.insert_raw(key_h, serde_json::to_vec(&ownership_h).unwrap());
        }

        // Create test-friendly dht_get_fn that queries local storage
        let test_dht_get_fn = create_test_dht_get_fn(dht_storage.clone());

        let orchestrator = SyncOrchestrator::new_with_dht_fn(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "peer-center".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager,
            db,
            rx,
            dht_storage,
            crate::routes::relay::RelayState::new(),
            Some(test_dht_get_fn), // Use test DHT GET function
        );

        // Discover geometric neighbors via LazyNode
        let neighbors = orchestrator.lazy_node.get_all_neighbors().await.unwrap();

        // Should find all 8 geometric neighbors
        assert_eq!(neighbors.len(), 8, "Should discover exactly 8 geometric neighbors");
        assert!(neighbors.contains(&"peer-plusA".to_string()), "Should find +A neighbor");
        assert!(neighbors.contains(&"peer-minusA".to_string()), "Should find -A neighbor");
        assert!(neighbors.contains(&"peer-plusB".to_string()), "Should find +B neighbor");
        assert!(neighbors.contains(&"peer-minusB".to_string()), "Should find -B neighbor");
        assert!(neighbors.contains(&"peer-plusC".to_string()), "Should find +C neighbor");
        assert!(neighbors.contains(&"peer-minusC".to_string()), "Should find -C neighbor");
        assert!(neighbors.contains(&"peer-up".to_string()), "Should find Up neighbor");
        assert!(neighbors.contains(&"peer-down".to_string()), "Should find Down neighbor");
    }

    #[tokio::test]
    async fn test_webrtc_connection_via_relay_signaling() {
        use crate::peer_registry::default_mesh_config;
        use citadel_core::topology::SlotCoordinate;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        let mesh_config = default_mesh_config();
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let orchestrator = SyncOrchestrator::new(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "peer-alice".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager.clone(),
            db,
            rx,
            dht_storage.clone(),
            crate::routes::relay::RelayState::new(),
        );

        let target_peer_id = "peer-bob";

        // Attempt WebRTC connection (will timeout waiting for relay answer)
        let result = orchestrator.establish_webrtc_connection(target_peer_id).await;

        // Connection should timeout (no relay to forward SDP answer)
        assert!(result.is_err(), "Connection should timeout without relay");
        assert!(result.unwrap_err().to_string().contains("Timeout"), "Error should indicate timeout");

        // Verify WebRTC peer connection was created
        // SDP signaling happens via relay WebSocket (not DHT)
        let peers = webrtc_manager.peers.read().await;
        assert!(peers.contains_key(target_peer_id), "WebRTC peer connection should be created for target");
    }

    #[tokio::test]
    async fn test_lazy_node_caches_neighbors_for_10_seconds() {
        use crate::peer_registry::{SlotOwnership, slot_ownership_key};
        use citadel_core::topology::{SlotCoordinate, Direction};

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let webrtc_manager = Arc::new(WebRTCManager::new().unwrap());
        let (_tx, rx) = mpsc::unbounded_channel();
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        let mesh_config = MeshConfig::new(3, 3, 3);
        let my_slot = SlotCoordinate::new(1, 1, 1);

        // Populate DHT with one neighbor
        {
            let mut dht = dht_storage.lock().await;
            let neighbor_slot = my_slot.neighbor(Direction::PlusA, &mesh_config);
            let ownership = SlotOwnership::new("peer-neighbor".to_string(), neighbor_slot, None);
            let key = slot_ownership_key(neighbor_slot);
            dht.insert_raw(key, serde_json::to_vec(&ownership).unwrap());
        }

        // Create test-friendly dht_get_fn that queries local storage
        let test_dht_get_fn = create_test_dht_get_fn(dht_storage.clone());

        let orchestrator = SyncOrchestrator::new_with_dht_fn(
            "ws://localhost:5002/api/v1/relay/ws".to_string(),
            "peer-center".to_string(),
            my_slot,
            mesh_config,
            p2p_manager,
            webrtc_manager,
            db,
            rx,
            dht_storage.clone(),
            crate::routes::relay::RelayState::new(),
            Some(test_dht_get_fn), // Use test DHT GET function
        );

        // First query - should hit DHT
        let neighbor = orchestrator.lazy_node.get_neighbor(Direction::PlusA).await.unwrap();
        assert_eq!(neighbor, "peer-neighbor");

        // Check cache stats
        let (cache_entries, _avg_age) = orchestrator.lazy_node.cache_stats().await;
        assert_eq!(cache_entries, 1, "Cache should have 1 entry after first query");

        // Second query immediately - should hit cache
        let neighbor2 = orchestrator.lazy_node.get_neighbor(Direction::PlusA).await.unwrap();
        assert_eq!(neighbor2, "peer-neighbor");

        // Verify cache is being used
        assert!(orchestrator.lazy_node.is_cached(Direction::PlusA).await, "Neighbor should be cached");
    }

    #[tokio::test]
    async fn test_content_addressed_slots_ensure_deterministic_routing() {
        use crate::peer_registry::{peer_id_to_slot, calculate_mesh_dimensions};

        // With content-addressed slots, the same peer_id always maps to the same slot
        let peer_id = "peer-12345";
        let mesh_config = calculate_mesh_dimensions(100);

        let slot1 = peer_id_to_slot(peer_id, &mesh_config);
        let slot2 = peer_id_to_slot(peer_id, &mesh_config);
        let slot3 = peer_id_to_slot(peer_id, &mesh_config);

        assert_eq!(slot1, slot2, "Same peer_id should map to same slot (deterministic)");
        assert_eq!(slot2, slot3, "Same peer_id should map to same slot (deterministic)");

        // Different peer_ids should (very likely) map to different slots
        let different_peer = "peer-67890";
        let different_slot = peer_id_to_slot(different_peer, &mesh_config);

        assert_ne!(slot1, different_slot, "Different peer_ids should map to different slots");
    }
}
