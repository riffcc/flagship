use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    http::StatusCode,
    response::IntoResponse,
};
use consensus_peerexc::{
    relay::RelayServer,
    PeerInfo, PeerState,
};
use crate::spore_wantlist::{WantListMessage, RangeResponse, compute_want_ranges};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug};
use crate::tgp::{self as tgp_mod, PacketType, DhtGetRequest, DhtPutRequest, DhtResponse};
use crate::peer_registry::{
    SlotOwnership, peer_location_key, slot_ownership_key,
    peer_id_to_slot, get_neighbor_slots, default_mesh_config
};
use citadel_core::topology::{SlotCoordinate, MeshConfig};
use citadel_core::routing::greedy_direction;
use citadel_core::key_mapping::key_to_slot;
use std::time::{SystemTime, UNIX_EPOCH};

/// Convert SlotCoordinate to compact u64 representation for TGP routing
/// Packs (x, y, z) coordinates into 64 bits (21 bits each, supports ±1 million)
fn coord_to_u64(coord: SlotCoordinate) -> u64 {
    let x = (coord.x as u64) & 0x1FFFFF; // 21 bits
    let y = (coord.y as u64) & 0x1FFFFF; // 21 bits
    let z = (coord.z as u64) & 0x1FFFFF; // 21 bits
    (x << 42) | (y << 21) | z
}

/// Convert u64 back to SlotCoordinate for TGP routing
/// Unpacks 64-bit compact representation into (x, y, z) coordinates
fn u64_to_coord(val: u64) -> SlotCoordinate {
    let x = ((val >> 42) & 0x1FFFFF) as i32;
    let y = ((val >> 21) & 0x1FFFFF) as i32;
    let z = (val & 0x1FFFFF) as i32;
    SlotCoordinate::new(x, y, z)
}

/// WebRTC signaling message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalingMessage {
    /// WebRTC offer from one peer to another
    Offer {
        from: String,
        to: String,
        sdp: String,
    },
    /// WebRTC answer in response to an offer
    Answer {
        from: String,
        to: String,
        sdp: String,
    },
    /// ICE candidate for WebRTC connection establishment
    IceCandidate {
        from: String,
        to: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
}

/// Browser peer announcement - browsers share their discovered peers with relay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserPeerAnnouncement {
    /// List of peers this browser is connected to
    pub peers: Vec<BrowserDiscoveredPeer>,
}

/// A peer discovered by a browser via WebRTC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserDiscoveredPeer {
    pub peer_id: String,
    pub connected: bool,
    pub connection_quality: Option<String>, // "good", "poor", etc.
}

/// Peer type classification for relay tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeerType {
    /// Server node - participates in mesh, has slot assignment
    Server,
    /// Browser client - edge client, no slot assignment, anonymous
    Browser,
}

/// Mesh topology update event - broadcast when peers join/leave or latency changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTopologyUpdate {
    /// Type of topology change
    pub event_type: TopologyEventType,

    /// Peer ID affected by this change
    pub peer_id: String,

    /// Slot coordinate of affected peer (if applicable)
    pub slot: Option<SlotCoordinate>,

    /// Current total peer count
    pub total_peers: usize,

    /// Timestamp of the event
    pub timestamp: u64,
}

/// Global DHT consistency check - verifies all nodes see identical DHT state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtConsistencyReport {
    /// Total number of DHT entries this node sees
    pub total_entries: usize,

    /// Hash of all DHT keys (sorted) - should be IDENTICAL across all nodes
    pub dht_keys_hash: String,

    /// Number of peers this node knows about from DHT
    pub known_peers: usize,

    /// My peer ID
    pub my_peer_id: String,

    /// My slot coordinate
    pub my_slot: SlotCoordinate,

    /// Timestamp of this report
    pub timestamp: u64,
}

/// Type of mesh topology event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyEventType {
    /// New peer joined the mesh
    PeerJoined,

    /// Peer left the mesh
    PeerLeft,

    /// Peer's latency measurements updated
    LatencyUpdated,
}

/// DHT replication message - gossip DHT puts through the hexagonal toroidal mesh
/// Uses epidemic/gossip protocol: each peer forwards to its 8 neighbors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtReplication {
    /// DHT key being replicated (32-byte Blake3 hash)
    #[serde(with = "serde_bytes")]
    pub key: Vec<u8>,

    /// DHT value being replicated
    #[serde(with = "serde_bytes")]
    pub value: Vec<u8>,

    /// Timestamp of the replication (for deduplication)
    pub timestamp: u64,

    /// Source peer that initiated the put
    pub source_peer_id: String,

    /// Hop count (TTL) - prevents infinite propagation through toroid
    /// Starts at mesh diameter (e.g., 10) and decrements each hop
    pub hops_remaining: u8,

    /// Propagation path (for debugging) - list of peer IDs this has passed through
    pub propagation_path: Vec<String>,
}

/// DHT bootstrap request - sent by new peers to bootstrap from existing DHT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtBootstrapRequest {
    /// Requesting peer's ID
    pub peer_id: String,

    /// Requesting peer's slot (so relay can find nearby peers)
    pub slot: SlotCoordinate,
}

/// DHT bootstrap response - relay sends its entire DHT state to bootstrapping peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtBootstrapResponse {
    /// Complete DHT state from relay (serialized DhtState)
    pub dht_entries: Vec<crate::dht_state::DhtEntry>,

    /// Number of entries in the DHT
    pub entry_count: usize,

    /// Timestamp of this snapshot
    pub timestamp: u64,
}

/// Cached neighbor slot ownership with TTL
#[derive(Debug, Clone)]
struct NeighborCache {
    /// Slot coordinate
    slot: SlotCoordinate,
    /// Peer ID owning this slot
    peer_id: String,
    /// When this cache entry was created
    cached_at: SystemTime,
    /// TTL in seconds (default: 60)
    ttl_seconds: u64,
}

impl NeighborCache {
    fn new(slot: SlotCoordinate, peer_id: String) -> Self {
        Self {
            slot,
            peer_id,
            cached_at: SystemTime::now(),
            ttl_seconds: 60,
        }
    }

    fn is_stale(&self) -> bool {
        self.cached_at.elapsed().unwrap_or_default().as_secs() > self.ttl_seconds
    }
}

/// Pending DHT GET request waiting for response
pub struct PendingDhtGet {
    pub response_tx: tokio::sync::oneshot::Sender<Option<Vec<u8>>>,
    pub timestamp: std::time::SystemTime,
}

/// Relay state shared across WebSocket connections
#[derive(Clone)]
pub struct RelayState {
    pub relay: Arc<RwLock<RelayServer>>,
    pub peer_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
    pub webrtc_manager: Option<Arc<crate::webrtc_manager::WebRTCManager>>,
    /// Browser-discovered peers - browsers share their WebRTC connections to help nodes find each other
    pub browser_discovered_peers: Arc<RwLock<HashMap<String, Vec<BrowserDiscoveredPeer>>>>,
    /// DHT storage for P2P distributed key-value store (GLOBAL - replicated via mesh gossip)
    /// Uses DhtState with bootstrap and merge capabilities
    pub dht_storage: Arc<tokio::sync::Mutex<crate::dht_state::DhtState>>,
    /// Cached neighbor slot ownership (peer_id -> vec of neighbor caches)
    neighbor_cache: Arc<RwLock<HashMap<String, Vec<NeighborCache>>>>,
    /// DHT replication deduplication cache - tracks (key_hex, timestamp) to prevent re-gossip
    dht_seen_cache: Arc<RwLock<HashMap<String, u64>>>,
    /// This node's peer_id (for DHT routing calculations)
    node_peer_id: Arc<RwLock<Option<String>>>,
    /// This node's explicit slot coordinate (overrides peer_id_to_slot calculation)
    /// When set, routing uses this instead of hashing the peer_id
    my_slot: Arc<RwLock<Option<SlotCoordinate>>>,
    /// Pending DHT GET requests waiting for responses (key_hex -> response channel)
    pub pending_dht_gets: Arc<RwLock<HashMap<String, PendingDhtGet>>>,
    /// P2P manager for tracking known peers (for /map and /ready endpoints)
    pub p2p_manager: Option<Arc<lens_v2_p2p::P2pManager>>,
    /// Event channel: fires when a DHT key is written (key_hex)
    pub dht_write_events: Arc<tokio::sync::broadcast::Sender<String>>,
    /// Peer type tracking - distinguishes server nodes from browser clients
    peer_types: Arc<RwLock<HashMap<String, PeerType>>>,
    /// Peer slot tracking - stores each peer's CLAIMED/ANNOUNCED slot (from gossip)
    /// DO NOT recalculate slots using peer_id_to_slot() - slots are Content Addressed and claimable!
    peer_slots: Arc<RwLock<HashMap<String, SlotCoordinate>>>,
}

impl RelayState {
    pub fn new() -> Self {
        let (dht_write_tx, _) = tokio::sync::broadcast::channel(1000);

        Self {
            relay: Arc::new(RwLock::new(RelayServer::new())),
            peer_senders: Arc::new(RwLock::new(HashMap::new())),
            webrtc_manager: None,
            browser_discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            dht_storage: Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new())),
            neighbor_cache: Arc::new(RwLock::new(HashMap::new())),
            dht_seen_cache: Arc::new(RwLock::new(HashMap::new())),
            node_peer_id: Arc::new(RwLock::new(None)),
            my_slot: Arc::new(RwLock::new(None)),
            pending_dht_gets: Arc::new(RwLock::new(HashMap::new())),
            p2p_manager: None,
            dht_write_events: Arc::new(dht_write_tx),
            peer_types: Arc::new(RwLock::new(HashMap::new())),
            peer_slots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_p2p_manager(mut self, p2p_manager: Arc<lens_v2_p2p::P2pManager>) -> Self {
        self.p2p_manager = Some(p2p_manager);
        self
    }

    pub fn with_webrtc(mut self, manager: Arc<crate::webrtc_manager::WebRTCManager>) -> Self {
        self.webrtc_manager = Some(manager);
        self
    }

    pub fn with_dht_storage(mut self, storage: Arc<tokio::sync::Mutex<crate::dht_state::DhtState>>) -> Self {
        self.dht_storage = storage;
        self
    }

    pub fn with_node_peer_id(mut self, peer_id: String) -> Self {
        self.node_peer_id = Arc::new(RwLock::new(Some(peer_id)));
        self
    }

    pub fn with_my_slot(mut self, slot: SlotCoordinate) -> Self {
        self.my_slot = Arc::new(RwLock::new(Some(slot)));
        self
    }

    /// Mark a peer as a server node (participates in mesh)
    pub async fn mark_as_server(&self, peer_id: &str) {
        self.peer_types.write().await.insert(peer_id.to_string(), PeerType::Server);
        debug!("🖥️ Marked peer {} as Server node", peer_id);
    }

    /// Mark a peer as a browser client (edge client, no mesh participation)
    pub async fn mark_as_browser(&self, peer_id: &str) {
        self.peer_types.write().await.insert(peer_id.to_string(), PeerType::Browser);
        debug!("🌐 Marked peer {} as Browser client", peer_id);
    }

    /// Get peer type (defaults to Server if not set)
    pub async fn get_peer_type(&self, peer_id: &str) -> PeerType {
        self.peer_types.read().await.get(peer_id).copied().unwrap_or(PeerType::Server)
    }

    /// Check if peer is a browser client
    pub async fn is_browser(&self, peer_id: &str) -> bool {
        self.get_peer_type(peer_id).await == PeerType::Browser
    }

    /// Get current mesh config based on actual peer count (DYNAMIC!)
    /// This is the SINGLE SOURCE OF TRUTH for mesh dimensions.
    ///
    /// NOTE: For slot assignment during connection, the caller should use
    /// get_mesh_config_for_assignment() instead, which accounts for the peer being added.
    async fn get_mesh_config(&self) -> MeshConfig {
        let peer_senders = self.peer_senders.read().await;
        let peer_count = peer_senders.len();
        drop(peer_senders);

        crate::peer_registry::calculate_mesh_dimensions(peer_count)
    }

    /// Get mesh config for initial slot assignment (includes the peer being added)
    async fn get_mesh_config_for_assignment(&self) -> MeshConfig {
        let peer_senders = self.peer_senders.read().await;
        let peer_count = peer_senders.len() + 1; // +1 for the peer being added!
        drop(peer_senders);

        crate::peer_registry::calculate_mesh_dimensions(peer_count)
    }

    /// Get cached neighbors for a peer, or query DHT if cache is stale
    async fn get_cached_neighbors(&self, peer_id: &str, my_slot: SlotCoordinate) -> Vec<(String, SlotCoordinate)> {
        let mesh_config = self.get_mesh_config().await;

        // Check cache first
        {
            let cache = self.neighbor_cache.read().await;
            if let Some(cached_neighbors) = cache.get(peer_id) {
                // Check if all cached entries are still fresh
                if cached_neighbors.iter().all(|n| !n.is_stale()) {
                    debug!("🔷 Using cached neighbors for peer {} ({} neighbors)", peer_id, cached_neighbors.len());
                    return cached_neighbors
                        .iter()
                        .map(|n| (n.peer_id.clone(), n.slot))
                        .collect();
                }
            }
        }

        // Cache miss or stale - query DHT for all 8 neighbors
        debug!("🔷 Cache miss for peer {}, querying DHT for neighbors", peer_id);
        let neighbor_slots = get_neighbor_slots(&my_slot, &mesh_config);
        let storage = self.dht_storage.lock().await;

        let mut neighbors = Vec::new();
        let mut cache_entries = Vec::new();

        for (_direction, neighbor_slot) in neighbor_slots {
            let slot_key = slot_ownership_key(neighbor_slot);

            if let Some(ownership_bytes) = storage.get_raw(&slot_key) {
                if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(ownership_bytes) {
                    // Skip self and stale entries
                    if ownership.peer_id != peer_id && !ownership.is_stale() {
                        neighbors.push((ownership.peer_id.clone(), neighbor_slot));
                        cache_entries.push(NeighborCache::new(neighbor_slot, ownership.peer_id.clone()));
                    }
                }
            }
        }

        drop(storage);

        // Update cache
        {
            let mut cache = self.neighbor_cache.write().await;
            cache.insert(peer_id.to_string(), cache_entries);
        }

        debug!("🔷 Cached {} neighbors for peer {}", neighbors.len(), peer_id);
        neighbors
    }

    /// Find the closest peer to a target slot using greedy routing
    /// Checks BOTH WebSocket peers AND WebRTC peers
    /// Looks up ACTUAL slot ownership from DHT storage (not peer_id_to_slot hash)
    async fn find_closest_peer(&self, target_slot: SlotCoordinate) -> Option<(String, SlotCoordinate, i32)> {
        println!("🔍 find_closest_peer: Looking for peers closest to slot ({}, {}, {})", target_slot.x, target_slot.y, target_slot.z);
        let mesh_config = self.get_mesh_config().await;

        let mut closest_peer: Option<(String, SlotCoordinate, i32)> = None;
        let mut min_distance = i32::MAX;

        // Check WebSocket peers - look up their ACTUAL slots from DHT
        let peer_senders = self.peer_senders.read().await;
        println!("🔍 find_closest_peer: Found {} WebSocket peers", peer_senders.len());
        let dht_storage = self.dht_storage.lock().await;

        for peer_id in peer_senders.keys() {
            // Try to get actual slot from DHT first
            let peer_slot = {
                let location_key = peer_location_key(peer_id);
                if let Some(ownership_bytes) = dht_storage.get_raw(&location_key) {
                    if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(ownership_bytes) {
                        if !ownership.is_stale() {
                            ownership.slot
                        } else {
                            peer_id_to_slot(peer_id, &mesh_config)
                        }
                    } else {
                        peer_id_to_slot(peer_id, &mesh_config)
                    }
                } else {
                    peer_id_to_slot(peer_id, &mesh_config)
                }
            };

            let (dx, dy, dz) = peer_slot.distance_to(&target_slot, &mesh_config);
            let distance = dx.abs() + dy.abs() + dz.abs(); // Manhattan distance

            if distance < min_distance {
                min_distance = distance;
                closest_peer = Some((peer_id.clone(), peer_slot, distance));
            }
        }
        drop(peer_senders);

        // ALSO check WebRTC peers!
        if let Some(ref webrtc_mgr) = self.webrtc_manager {
            let webrtc_peers = webrtc_mgr.peers.read().await;
            println!("🔍 find_closest_peer: Found {} WebRTC peers", webrtc_peers.len());
            for peer_id in webrtc_peers.keys() {
                println!("🔍 find_closest_peer: Checking WebRTC peer: {}", peer_id);
                // Try to get actual slot from DHT first
                let peer_slot = {
                    let location_key = peer_location_key(peer_id);
                    if let Some(ownership_bytes) = dht_storage.get_raw(&location_key) {
                        if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(ownership_bytes) {
                            if !ownership.is_stale() {
                                ownership.slot
                            } else {
                                peer_id_to_slot(peer_id, &mesh_config)
                            }
                        } else {
                            peer_id_to_slot(peer_id, &mesh_config)
                        }
                    } else {
                        peer_id_to_slot(peer_id, &mesh_config)
                    }
                };

                let (dx, dy, dz) = peer_slot.distance_to(&target_slot, &mesh_config);
                let distance = dx.abs() + dy.abs() + dz.abs(); // Manhattan distance

                info!("🔍 find_closest_peer: WebRTC peer {} at slot ({}, {}, {}) has distance {}",
                    peer_id, peer_slot.x, peer_slot.y, peer_slot.z, distance);

                if distance < min_distance {
                    min_distance = distance;
                    closest_peer = Some((peer_id.clone(), peer_slot, distance));
                    info!("✅ find_closest_peer: Found closer peer via WebRTC: {} at distance {}", peer_id, distance);
                }
            }
            drop(webrtc_peers);
        }

        drop(dht_storage);
        closest_peer
    }

    /// Route a DHT GET to the slot that owns this key (Citadel spec)
    /// Returns the value if found, None if not found or unreachable
    pub async fn dht_get(&self, key: [u8; 32]) -> Option<Vec<u8>> {
        let mesh_config = self.get_mesh_config().await;
        let target_slot = key_to_slot(&key, &mesh_config);

        // Get this node's peer_id to determine our slot
        let node_peer_id_lock = self.node_peer_id.read().await;
        let my_peer_id = match node_peer_id_lock.as_ref() {
            Some(id) => id.clone(),
            None => {
                warn!("⚠️  DHT GET: Node peer_id not set, cannot route");
                return None;
            }
        };
        drop(node_peer_id_lock);

        // Check if explicit slot is set (for tests), otherwise calculate from peer_id
        let my_slot = {
            let slot_lock = self.my_slot.read().await;
            if let Some(slot) = slot_lock.as_ref() {
                *slot
            } else {
                peer_id_to_slot(&my_peer_id, &mesh_config)
            }
        };

        if target_slot == my_slot {
            // This key belongs to OUR slot! Read locally
            let storage = self.dht_storage.lock().await;
            storage.get_raw(&key).map(|v| v.to_vec())
        } else {
            // Key belongs to DIFFERENT slot - route DHT GET request via greedy routing
            debug!("DHT GET: Routing key to remote slot {:?}", target_slot);

            // Find closest peer toward target slot (or ANY peer if we can't find closest)
            let closest_peer_result = self.find_closest_peer(target_slot).await;

            let closest_peer_id = match closest_peer_result {
                Some((peer_id, _slot, _dist)) => peer_id,
                None => {
                    // No WebRTC peers - use relay WebSocket as proxy (per CONTEXT.md)
                    // The relay will route DHT GET to the appropriate node
                    debug!("DHT GET: No WebRTC peers, using relay WebSocket as proxy");

                    // Grab ANY connected WebSocket peer (relay acts as dumb proxy)
                    let peer_senders = self.peer_senders.read().await;
                    if let Some(any_peer_id) = peer_senders.keys().next().cloned() {
                        drop(peer_senders);
                        debug!("DHT GET: Using relay peer {} as proxy for key routing", any_peer_id);
                        any_peer_id
                    } else {
                        // No WebSocket connections at all - fall back to local storage
                        debug!("DHT GET: No connected peers at all, checking local storage");
                        drop(peer_senders);
                        let storage = self.dht_storage.lock().await;
                        return storage.get_raw(&key).map(|v| v.to_vec());
                    }
                }
            };

            // Create response channel
            let (response_tx, response_rx) = tokio::sync::oneshot::channel();
            let key_hex = hex::encode(&key);

            // Register pending request
            {
                let mut pending = self.pending_dht_gets.write().await;
                pending.insert(key_hex.clone(), PendingDhtGet {
                    response_tx,
                    timestamp: std::time::SystemTime::now(),
                });
            }

            // Create TGP DhtGetRequest packet
            let request = crate::tgp::DhtGetRequest { key };
            let payload = match serde_json::to_vec(&request) {
                Ok(p) => p,
                Err(e) => {
                    warn!("DHT GET: Failed to serialize request: {}", e);
                    // Clean up pending request
                    let mut pending = self.pending_dht_gets.write().await;
                    pending.remove(&key_hex);
                    return None;
                }
            };

            // Calculate compact u64 representations of coordinates for TGP routing
            let my_slot_u64 = coord_to_u64(my_slot);
            let target_slot_u64 = coord_to_u64(target_slot);

            // Send TGP packet to closest peer
            let packet = crate::tgp::create_packet(
                crate::tgp::PacketType::DhtGet.as_u8(),
                target_slot_u64,  // dest_hex - where the key lives
                my_slot_u64,      // source_hex - where to send responses back
                &payload
            );

            // Try WebRTC DataChannel first, fall back to WebSocket
            let mut sent_via_webrtc = false;
            if let Some(ref webrtc_mgr) = self.webrtc_manager {
                if webrtc_mgr.is_peer_connected(&closest_peer_id).await {
                    // Send via WebRTC DataChannel!
                    if let Ok(_) = webrtc_mgr.send_binary_to_peer(&closest_peer_id, packet.clone()).await {
                        debug!("📡 DHT GET sent via WebRTC DataChannel to {}", closest_peer_id);
                        sent_via_webrtc = true;
                    } else {
                        debug!("⚠️  WebRTC send failed for {}, falling back to WebSocket", closest_peer_id);
                    }
                }
            }

            // Fall back to WebSocket if WebRTC not available or failed
            if !sent_via_webrtc {
                let senders = self.peer_senders.read().await;
                if let Some(tx) = senders.get(&closest_peer_id) {
                    if let Err(e) = tx.send(Message::Binary(packet)) {
                        warn!("DHT GET: Failed to send to {}: {}", closest_peer_id, e);
                        // Clean up pending request
                        let mut pending = self.pending_dht_gets.write().await;
                        pending.remove(&key_hex);
                        return None;
                    }
                } else {
                    warn!("DHT GET: Peer {} not found in senders", closest_peer_id);
                    // Clean up pending request
                    let mut pending = self.pending_dht_gets.write().await;
                    pending.remove(&key_hex);
                    return None;
                }
            }

            // Wait for response with timeout
            match tokio::time::timeout(std::time::Duration::from_secs(5), response_rx).await {
                Ok(Ok(value)) => {
                    debug!("DHT GET: Received response for key {}", key_hex);
                    value
                }
                Ok(Err(_)) => {
                    warn!("DHT GET: Response channel closed for key {}", key_hex);
                    None
                }
                Err(_) => {
                    warn!("DHT GET: Timeout waiting for response for key {}", key_hex);
                    // Clean up pending request
                    let mut pending = self.pending_dht_gets.write().await;
                    pending.remove(&key_hex);
                    None
                }
            }
        }
    }

    /// Route a DHT PUT to the slot that owns this key (Citadel spec)
    /// Keys are stored at the slot they hash to, NOT locally!
    pub async fn dht_put(&self, key: [u8; 32], value: Vec<u8>) {
        let mesh_config = self.get_mesh_config().await;
        let target_slot = key_to_slot(&key, &mesh_config);

        // Get this node's peer_id to determine our slot
        let node_peer_id_lock = self.node_peer_id.read().await;
        let my_peer_id = match node_peer_id_lock.as_ref() {
            Some(id) => id.clone(),
            None => {
                warn!("⚠️  DHT PUT: Node peer_id not set, cannot route");
                return;
            }
        };
        drop(node_peer_id_lock);

        // Check if explicit slot is set (for tests), otherwise calculate from peer_id
        let my_slot = {
            let slot_lock = self.my_slot.read().await;
            if let Some(slot) = slot_lock.as_ref() {
                *slot
            } else {
                peer_id_to_slot(&my_peer_id, &mesh_config)
            }
        };

        println!("🔑 DHT PUT: Key maps to slot ({}, {}, {}), my slot is ({}, {}, {})",
            target_slot.x, target_slot.y, target_slot.z,
            my_slot.x, my_slot.y, my_slot.z);

        if target_slot == my_slot {
            // This key belongs to OUR slot! Store locally
            println!("✅ DHT PUT: Key belongs to our slot, storing locally");
            let mut storage = self.dht_storage.lock().await;
            storage.insert_raw(key, value);
            println!("✅ DHT PUT: Value stored in local DHT storage");
        } else {
            // Key belongs to DIFFERENT slot! Route to closest peer
            println!("🔀 DHT PUT: Key belongs to different slot, routing via greedy routing");
            println!("🔍 DHT PUT: Calling find_closest_peer for target slot ({}, {}, {})", target_slot.x, target_slot.y, target_slot.z);

            if let Some((closest_peer_id, closest_slot, distance)) = self.find_closest_peer(target_slot).await {
                println!("🚀 DHT PUT: Found closest peer {} at slot ({}, {}, {}) (distance to target: {})",
                    closest_peer_id, closest_slot.x, closest_slot.y, closest_slot.z, distance);

                // Create TGP DhtPutRequest packet
                let request = crate::tgp::DhtPutRequest {
                    key,
                    value,
                };

                let payload = match serde_json::to_vec(&request) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("⚠️  DHT PUT: Failed to serialize request: {}", e);
                        return;
                    }
                };

                // Create TGP packet
                // TODO: Need proper source/dest slot IDs - for now use placeholder
                let packet = crate::tgp::create_packet(
                    crate::tgp::PacketType::DhtPut.as_u8(),
                    0, // dest_slot_id (placeholder)
                    0, // source_slot_id (placeholder)
                    &payload
                );

                // Try WebRTC DataChannel first, fall back to WebSocket
                let mut sent_via_webrtc = false;
                if let Some(ref webrtc_mgr) = self.webrtc_manager {
                    println!("🔍 DHT PUT: Checking if peer {} is connected via WebRTC", closest_peer_id);
                    info!("🔍 DHT PUT: Checking if peer {} is connected via WebRTC", closest_peer_id);
                    if webrtc_mgr.is_peer_connected(&closest_peer_id).await {
                        println!("✅ DHT PUT: Peer {} is connected via WebRTC, sending {} byte TGP packet", closest_peer_id, packet.len());
                        info!("✅ DHT PUT: Peer {} is connected via WebRTC, sending {} byte TGP packet", closest_peer_id, packet.len());
                        // Send via WebRTC DataChannel!
                        match webrtc_mgr.send_binary_to_peer(&closest_peer_id, packet.clone()).await {
                            Ok(_) => {
                                println!("📡 DHT PUT sent via WebRTC DataChannel to {}", closest_peer_id);
                                info!("📡 DHT PUT sent via WebRTC DataChannel to {}", closest_peer_id);
                                sent_via_webrtc = true;
                            }
                            Err(e) => {
                                println!("⚠️  WebRTC send failed for {}: {}, falling back to WebSocket", closest_peer_id, e);
                                warn!("⚠️  WebRTC send failed for {}: {}, falling back to WebSocket", closest_peer_id, e);
                            }
                        }
                    } else {
                        println!("⚠️  DHT PUT: Peer {} NOT connected via WebRTC", closest_peer_id);
                        info!("⚠️  DHT PUT: Peer {} NOT connected via WebRTC", closest_peer_id);
                    }
                } else {
                    println!("⚠️  DHT PUT: No WebRTC manager available");
                    info!("⚠️  DHT PUT: No WebRTC manager available");
                }

                // Fall back to WebSocket if WebRTC not available or failed
                if !sent_via_webrtc {
                    let senders = self.peer_senders.read().await;
                    if let Some(tx) = senders.get(&closest_peer_id) {
                        if let Err(e) = tx.send(Message::Binary(packet)) {
                            warn!("⚠️  DHT PUT: Failed to send to {}: {}", closest_peer_id, e);
                        } else {
                            info!("✅ DHT PUT: Sent to {} for routing", closest_peer_id);
                        }
                    } else {
                        warn!("⚠️  DHT PUT: Closest peer {} not found in senders", closest_peer_id);
                    }
                }
            } else {
                warn!("⚠️  DHT PUT: No route to target slot ({}, {}, {})", target_slot.x, target_slot.y, target_slot.z);
            }
        }
    }

    /// Gossip slot ownership announcement to ALL connected peers (both WebSocket and WebRTC)
    /// This enables distributed slot discovery without circular dependencies
    pub async fn gossip_slot_ownership(&self, peer_id: String, slot: SlotCoordinate) {
        use crate::peer_registry::{SlotOwnership, peer_location_key, slot_ownership_key};

        println!("📢 Gossiping slot ownership: {} → ({}, {}, {})", peer_id, slot.x, slot.y, slot.z);

        // Create ownership record
        let ownership = SlotOwnership::new(peer_id.clone(), slot, None);
        let ownership_bytes = match serde_json::to_vec(&ownership) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Failed to serialize slot ownership: {}", e);
                return;
            }
        };

        // Store locally first
        {
            let mut storage = self.dht_storage.lock().await;
            let location_key = peer_location_key(&peer_id);
            let slot_key = slot_ownership_key(slot);
            storage.insert_raw(location_key, ownership_bytes.clone());
            storage.insert_raw(slot_key, ownership_bytes.clone());
            println!("✅ Stored slot ownership locally: {} → ({}, {}, {})", peer_id, slot.x, slot.y, slot.z);
        }

        // Store claimed slot in peer_slots for DHT GET routing
        {
            let mut peer_slots = self.peer_slots.write().await;
            peer_slots.insert(peer_id.clone(), slot);
            println!("🎯 Stored claimed slot for {}: ({}, {}, {})", peer_id, slot.x, slot.y, slot.z);
        }

        // Use DHT PUT to store in the network (routes to responsible slot)
        println!("📤 DHT PUT: Storing slot ownership in DHT network via routing...");
        let location_key = peer_location_key(&peer_id);
        let slot_key = slot_ownership_key(slot);

        // Put both keys into the DHT network (with routing)
        self.dht_put(location_key, ownership_bytes.clone()).await;
        println!("✅ DHT PUT: peer_location_key routed and stored");

        self.dht_put(slot_key, ownership_bytes.clone()).await;
        println!("✅ DHT PUT: slot_ownership_key routed and stored");

        // Create gossip message
        let gossip_message = serde_json::json!({
            "type": "slot_ownership_gossip",
            "peer_id": peer_id,
            "slot": {
                "x": slot.x,
                "y": slot.y,
                "z": slot.z,
            },
            "ownership_bytes": hex::encode(&ownership_bytes),
        });

        let message_json = match serde_json::to_string(&gossip_message) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize gossip message: {}", e);
                return;
            }
        };

        let mut gossip_count = 0;

        // Broadcast to WebSocket peers
        let peer_senders = self.peer_senders.read().await;
        println!("🔍 Gossip: Found {} WebSocket peers", peer_senders.len());
        for (ws_peer_id, tx) in peer_senders.iter() {
            println!("🔍 Gossip: Checking WebSocket peer: {}", ws_peer_id);
            if ws_peer_id != &peer_id {  // Don't send back to origin
                if let Err(e) = tx.send(Message::Text(message_json.clone())) {
                    debug!("Failed to gossip to WebSocket peer {}: {}", ws_peer_id, e);
                } else {
                    gossip_count += 1;
                    println!("✅ Gossip: Sent to WebSocket peer {}", ws_peer_id);
                }
            } else {
                println!("⏭️  Gossip: Skipping origin peer {}", ws_peer_id);
            }
        }
        drop(peer_senders);

        // Broadcast to WebRTC peers
        if let Some(ref webrtc_mgr) = self.webrtc_manager {
            let webrtc_peers = webrtc_mgr.peers.read().await;
            println!("🔍 Gossip: Found {} WebRTC peers", webrtc_peers.len());
            for (rtc_peer_id, peer) in webrtc_peers.iter() {
                println!("🔍 Gossip: Checking WebRTC peer: {}, has_dc: {}", rtc_peer_id, peer.data_channel.is_some());
                if rtc_peer_id != &peer_id {  // Don't send back to origin
                    if let Some(ref dc) = peer.data_channel {
                        // Check DataChannel ready state
                        let ready_state = dc.ready_state();
                        println!("🔍 Gossip: DataChannel state for peer {}: {:?}", rtc_peer_id, ready_state);

                        match dc.send_text(message_json.clone()).await {
                            Ok(_) => {
                                gossip_count += 1;
                                println!("✅ Gossip: Sent to WebRTC peer {}", rtc_peer_id);
                            }
                            Err(e) => {
                                println!("❌ Gossip: Failed to send to WebRTC peer {}: {}", rtc_peer_id, e);
                            }
                        }
                    } else {
                        println!("⏭️  Gossip: WebRTC peer {} has no DataChannel yet", rtc_peer_id);
                    }
                } else {
                    println!("⏭️  Gossip: Skipping origin peer {}", rtc_peer_id);
                }
            }
        } else {
            println!("⚠️  Gossip: No WebRTC manager available");
        }

        println!("✅ Gossiped slot ownership to {} peers", gossip_count);
    }

    /// Broadcast a mesh topology update to all connected peers
    async fn broadcast_topology_update(&self, event_type: TopologyEventType, peer_id: &str, slot: Option<SlotCoordinate>) {
        let peer_senders = self.peer_senders.read().await;
        let total_peers = peer_senders.len();

        let update = MeshTopologyUpdate {
            event_type: event_type.clone(),
            peer_id: peer_id.to_string(),
            slot,
            total_peers,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let message = serde_json::json!({
            "type": "mesh_topology_update",
            "update": update,
        });

        if let Ok(json) = serde_json::to_string(&message) {
            let mut broadcast_count = 0;
            for (_peer_id, tx) in peer_senders.iter() {
                if let Err(e) = tx.send(Message::Text(json.clone())) {
                    warn!("Failed to broadcast topology update: {}", e);
                } else {
                    broadcast_count += 1;
                }
            }

            if broadcast_count > 0 {
                info!("📡 Broadcast {:?} event for peer {} to {} peers", event_type, peer_id, broadcast_count);
            }
        }
    }

    /// Gossip DHT PUT through hexagonal toroidal mesh to 8 neighbors
    /// **ELIMINATES LOCAL DHT CONCEPT** - epidemic propagation creates global DHT!
    /// Each peer forwards to its 8 mesh neighbors, who forward to THEIR neighbors, etc.
    pub async fn gossip_dht_put(&self, key: [u8; 32], value: Vec<u8>, source_peer_id: String, my_peer_id: String) {
        let mesh_config = self.get_mesh_config().await;

        // Check if explicit slot is set (for tests), otherwise calculate from peer_id
        let my_slot = {
            let slot_lock = self.my_slot.read().await;
            if let Some(slot) = slot_lock.as_ref() {
                *slot
            } else {
                peer_id_to_slot(&my_peer_id, &mesh_config)
            }
        };

        // Get my 8 mesh neighbors
        let neighbor_slots = get_neighbor_slots(&my_slot, &mesh_config);

        // Find connected peers at those neighbor slots
        let storage = self.dht_storage.lock().await;
        let mut neighbor_peer_ids = Vec::new();

        for (_direction, neighbor_slot) in neighbor_slots {
            let slot_key = slot_ownership_key(neighbor_slot);
            if let Some(ownership_bytes) = storage.get_raw(&slot_key) {
                if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(ownership_bytes) {
                    if ownership.peer_id != my_peer_id && !ownership.is_stale() {
                        neighbor_peer_ids.push(ownership.peer_id.clone());
                    }
                }
            }
        }

        drop(storage);

        // Create DHT replication message with hop limit
        let mesh_diameter = (mesh_config.width.max(mesh_config.height).max(mesh_config.depth) * 2) as u8;
        let mut replication = DhtReplication {
            key: key.to_vec(),
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source_peer_id: source_peer_id.clone(),
            hops_remaining: mesh_diameter,
            propagation_path: vec![my_peer_id.clone()],
        };

        let message = serde_json::json!({
            "type": "dht_replication",
            "replication": replication,
        });

        if let Ok(json) = serde_json::to_string(&message) {
            let peer_senders = self.peer_senders.read().await;
            let mut gossip_count = 0;

            for neighbor_id in &neighbor_peer_ids {
                if let Some(tx) = peer_senders.get(neighbor_id) {
                    if let Err(e) = tx.send(Message::Text(json.clone())) {
                        warn!("Failed to gossip DHT to neighbor {}: {}", neighbor_id, e);
                    } else {
                        gossip_count += 1;
                    }
                }
            }

            if gossip_count > 0 {
                debug!("🌐 Gossiped DHT PUT (key={}) to {} mesh neighbors (/{} found)",
                    hex::encode(&key), gossip_count, neighbor_peer_ids.len());
            }
        }
    }
}

impl Default for RelayState {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket handler for P2P relay
pub async fn relay_handler(
    ws: WebSocketUpgrade,
    State(state): State<RelayState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: RelayState) {
    let (mut sender, mut receiver) = socket.split();

    // Wait for client's first message containing their peer_id
    // This ensures peer_id consistency across the mesh
    let peer_id = match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            // Try to parse as hello message: {"type": "hello", "peer_id": "peer-123"}
            if let Ok(hello) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some("hello") = hello.get("type").and_then(|v| v.as_str()) {
                    if let Some(client_peer_id) = hello.get("peer_id").and_then(|v| v.as_str()) {
                        info!("Relay: Server node announced peer_id: {}", client_peer_id);
                        client_peer_id.to_string()
                    } else {
                        warn!("Relay: Hello message missing peer_id, generating random");
                        format!("peer-{}", rand::random::<u32>())
                    }
                } else {
                    warn!("Relay: First message not a hello, generating random peer_id");
                    format!("peer-{}", rand::random::<u32>())
                }
            } else {
                warn!("Relay: First message not JSON, generating random peer_id");
                format!("peer-{}", rand::random::<u32>())
            }
        }
        _ => {
            warn!("Relay: No first message received, generating random peer_id");
            format!("peer-{}", rand::random::<u32>())
        }
    };

    info!("Relay: New peer connected: {}", peer_id);

    // Determine if this is a server node or will be marked as browser later
    // Default to server - browser_announce will override this
    state.mark_as_server(&peer_id).await;

    // Create channel for this peer's outgoing messages
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Store the sender in peer_senders map
    {
        let mut senders = state.peer_senders.write().await;
        senders.insert(peer_id.clone(), tx);
    }

    // Register peer with relay
    let peer_info = PeerInfo::new(peer_id.clone());
    {
        let mut relay = state.relay.write().await;
        if let Err(e) = relay.register_peer(peer_info.clone()) {
            warn!("Relay: Failed to register peer {}: {}", peer_id, e);
            return;
        }
    }

    // **CITADEL DHT MESH TOPOLOGY ANNOUNCEMENT** (Section 2.4 - Recursive DHT)
    // Calculate my SlotCoordinate and announce ownership!
    // SKIP FOR BROWSER CLIENTS - they don't participate in the mesh
    if !state.is_browser(&peer_id).await {
        let mesh_config = state.get_mesh_config_for_assignment().await;

        // Check if explicit slot is set (for tests), otherwise calculate from peer_id with collision resolution
        let my_slot = {
            let slot_lock = state.my_slot.read().await;
            if let Some(slot) = slot_lock.as_ref() {
                *slot
            } else {
                // Query DHT for already-occupied slots to avoid collisions
                use crate::peer_registry::assign_unique_slot;
                use std::collections::HashSet;

                let dht_storage = state.dht_storage.lock().await;
                let mut occupied_slots = HashSet::new();

                // Scan DHT for all slot ownership announcements
                for (_key, entry) in dht_storage.iter() {
                    // Try to parse as SlotOwnership
                    if let Ok(ownership) = serde_json::from_slice::<crate::peer_registry::SlotOwnership>(&entry.value) {
                        occupied_slots.insert(ownership.slot);
                    }
                }

                drop(dht_storage); // Release lock before calling assign_unique_slot

                info!("🔍 Found {} occupied slots in DHT before assigning slot for {}",
                    occupied_slots.len(), peer_id);

                assign_unique_slot(&peer_id, &mesh_config, &mut occupied_slots)
            }
        };

        info!("📢 Server node {} assigned to slot ({}, {}, {}) in hexagonal toroidal mesh (COLLISION-FREE!)",
            peer_id, my_slot.x, my_slot.y, my_slot.z);

        // GOSSIP slot ownership to ALL nodes via broadcast-through-the-mesh
        // Per architecture: "Keys like slot ownership should also be replicated to every node and cached"
        // This enables LOCAL DHT queries with event-driven replication
        // "How does anyone know who owns the slot if you're gone?" - Everyone needs to know!

        state.gossip_slot_ownership(peer_id.clone(), my_slot).await;

        info!("✅ Gossiped slot ownership for peer {} at ({}, {}, {}) to ALL nodes via broadcast-through-mesh",
            peer_id, my_slot.x, my_slot.y, my_slot.z);

        // Log 8 neighbor slots for visibility (lazy discovery will query these on-demand!)
        let neighbors = get_neighbor_slots(&my_slot, &mesh_config);
        info!("🔷 Server node {} has 8 mesh neighbors at slots: {:?}",
            peer_id, neighbors.iter().map(|(_, s)| (s.x, s.y, s.z)).collect::<Vec<_>>());

        // Notify P2P manager about this peer for /map and /ready endpoints
        if let Some(p2p) = &state.p2p_manager {
            // Hash peer_id string to u64 for P2pManager
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            peer_id.hash(&mut hasher);
            let peer_id_u64 = hasher.finish();

            if let Err(e) = p2p.add_known_peer_with_string(peer_id_u64, peer_id.clone()) {
                warn!("Failed to add peer {} to P2P manager: {}", peer_id, e);
            } else {
                info!("✅ Added peer {} to P2P manager (known_peers tracking)", peer_id);
            }

            // Mark peer as alive immediately on connection
            if let Err(e) = p2p.mark_peer_alive(peer_id_u64) {
                warn!("Failed to mark peer {} as alive: {}", peer_id, e);
            } else {
                info!("💓 Marked peer {} as alive on connection", peer_id);
            }
        }

        // Broadcast topology update to all connected peers
        state.broadcast_topology_update(TopologyEventType::PeerJoined, &peer_id, Some(my_slot)).await;
    } else {
        info!("🌐 Browser client {} connected - skipping mesh slot assignment", peer_id);
    }

    // Spawn task to handle outgoing messages
    let peer_id_clone = peer_id.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = sender.send(msg).await {
                warn!("Relay: Failed to send to {}: {}", peer_id_clone, e);
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Relay: Received text from {}: {} bytes", peer_id, text.len());
                info!("Relay: Message content: {}", text);

                // Check for browser_announce first
                if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(&text) {
                    info!("Relay: Parsed JSON, type = {:?}", msg_json.get("type"));
                    if let Some("browser_announce") = msg_json.get("type").and_then(|v| v.as_str()) {
                        info!("Relay: Browser peer announced: {}", peer_id);

                        // Mark as browser client (no mesh participation)
                        state.mark_as_browser(&peer_id).await;

                        // If WebRTC manager available, create connection to browser
                        if let Some(ref webrtc_mgr) = state.webrtc_manager {
                            let mgr = webrtc_mgr.clone();
                            let browser_peer_id = peer_id.clone();
                            tokio::spawn(async move {
                                if let Err(e) = mgr.create_peer_connection(browser_peer_id.clone(), crate::webrtc_manager::PeerType::Browser).await {
                                    warn!("Relay: Failed to create WebRTC connection to {}: {}", browser_peer_id, e);
                                }
                            });
                        }
                        continue;
                    }

                    // Check for heartbeat message
                    if let Some("heartbeat") = msg_json.get("type").and_then(|v| v.as_str()) {
                        if let Some(heartbeat_peer_id) = msg_json.get("peer_id").and_then(|v| v.as_str()) {
                            // Hash peer ID to u64
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};
                            let mut hasher = DefaultHasher::new();
                            heartbeat_peer_id.hash(&mut hasher);
                            let peer_id_u64 = hasher.finish();

                            // Mark peer as alive in P2P manager
                            if let Some(p2p) = &state.p2p_manager {
                                if let Err(e) = p2p.mark_peer_alive(peer_id_u64) {
                                    warn!("Failed to mark peer {} as alive: {}", heartbeat_peer_id, e);
                                } else {
                                    debug!("💓 Heartbeat received from {}", heartbeat_peer_id);
                                }
                            }

                            // Broadcast heartbeat to all other peers so everyone knows this peer is alive
                            let senders = state.peer_senders.read().await;
                            for (other_peer_id, tx) in senders.iter() {
                                if other_peer_id != &peer_id {
                                    let _ = tx.send(Message::Text(text.clone()));
                                }
                            }
                        }
                        continue;
                    }

                    // Check for slot ownership gossip
                    if let Some("slot_ownership_gossip") = msg_json.get("type").and_then(|v| v.as_str()) {
                        println!("📨 Received slot ownership gossip from {}", peer_id);

                        if let (Some(gossiped_peer_id), Some(slot_obj), Some(ownership_hex)) = (
                            msg_json.get("peer_id").and_then(|v| v.as_str()),
                            msg_json.get("slot"),
                            msg_json.get("ownership_bytes").and_then(|v| v.as_str())
                        ) {
                            if let (Some(x), Some(y), Some(z)) = (
                                slot_obj.get("x").and_then(|v| v.as_u64()),
                                slot_obj.get("y").and_then(|v| v.as_u64()),
                                slot_obj.get("z").and_then(|v| v.as_u64())
                            ) {
                                let slot = SlotCoordinate::new(x as i32, y as i32, z as i32);
                                println!("📍 Storing gossiped slot ownership: {} → ({}, {}, {})", gossiped_peer_id, x, y, z);

                                // Decode ownership bytes
                                if let Ok(ownership_bytes) = hex::decode(ownership_hex) {
                                    use crate::peer_registry::{peer_location_key, slot_ownership_key};

                                    // Store in local DHT
                                    let mut storage = state.dht_storage.lock().await;
                                    let location_key = peer_location_key(gossiped_peer_id);
                                    let slot_key = slot_ownership_key(slot);
                                    storage.insert_raw(location_key, ownership_bytes.clone());
                                    storage.insert_raw(slot_key, ownership_bytes);
                                    drop(storage);

                                    // Store claimed slot in peer_slots for DHT GET routing
                                    {
                                        let mut peer_slots = state.peer_slots.write().await;
                                        peer_slots.insert(gossiped_peer_id.to_string(), slot);
                                        println!("🎯 Stored claimed slot for {}: ({}, {}, {})", gossiped_peer_id, slot.x, slot.y, slot.z);
                                    }

                                    println!("✅ Stored gossiped slot ownership locally");

                                    // Re-gossip to other peers (flooding with TTL would be better)
                                    let senders = state.peer_senders.read().await;
                                    let mut regossip_count = 0;
                                    for (other_peer_id, tx) in senders.iter() {
                                        if other_peer_id != &peer_id && other_peer_id != gossiped_peer_id {
                                            if let Ok(_) = tx.send(Message::Text(text.clone())) {
                                                regossip_count += 1;
                                            }
                                        }
                                    }
                                    drop(senders);

                                    // Also re-gossip via WebRTC
                                    if let Some(ref webrtc_mgr) = state.webrtc_manager {
                                        let webrtc_peers = webrtc_mgr.peers.read().await;
                                        for (rtc_peer_id, peer) in webrtc_peers.iter() {
                                            if rtc_peer_id != &peer_id && rtc_peer_id != gossiped_peer_id {
                                                if let Some(ref dc) = peer.data_channel {
                                                    if let Ok(_) = dc.send_text(text.clone()).await {
                                                        regossip_count += 1;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if regossip_count > 0 {
                                        println!("🌐 Re-gossiped slot ownership to {} peers", regossip_count);
                                    }
                                }
                            }
                        }
                        continue;
                    }

                    // Check for browser peer announcement (browser sharing its discovered peers)
                    if let Some("browser_peer_announcement") = msg_json.get("type").and_then(|v| v.as_str()) {
                        if let Ok(announcement) = serde_json::from_str::<BrowserPeerAnnouncement>(&text) {
                            info!("Relay: Browser {} announced {} discovered peers", peer_id, announcement.peers.len());

                            // Store browser-discovered peers
                            {
                                let mut browser_peers = state.browser_discovered_peers.write().await;
                                browser_peers.insert(peer_id.clone(), announcement.peers.clone());
                            }

                            // Log discovered peers
                            for peer in &announcement.peers {
                                info!("  - Browser discovered peer: {} (connected: {})", peer.peer_id, peer.connected);
                            }
                        }
                        continue;
                    }
                }

                // Try to parse as SignalingMessage
                if let Ok(sig_msg) = serde_json::from_str::<SignalingMessage>(&text) {
                    info!("Relay: Received signaling message: {:?}", sig_msg);

                    // Route the message to the target peer
                    let target_id = match &sig_msg {
                        SignalingMessage::Offer { to, .. } => to,
                        SignalingMessage::Answer { to, .. } => to,
                        SignalingMessage::IceCandidate { to, .. } => to,
                    };

                    let senders = state.peer_senders.read().await;
                    if let Some(target_tx) = senders.get(target_id) {
                        if let Err(e) = target_tx.send(Message::Text(text)) {
                            warn!("Relay: Failed to route signaling to {}: {}", target_id, e);
                        } else {
                            info!("Relay: Routed signaling from {} to {}", peer_id, target_id);
                        }
                    } else {
                        warn!("Relay: Target peer {} not connected", target_id);
                    }
                }
                // Try to parse as WantList (SPORE range-based protocol)
                else if let Ok(wantlist) = serde_json::from_str::<WantListMessage>(&text) {
                    info!("Relay: Received WantList from {}: wants {} ranges, has {} ranges",
                        peer_id, wantlist.want_ranges.len(), wantlist.have_ranges.len());

                    // TODO: Store peer's have_ranges in peer registry for slot ownership tracking

                    // Broadcast WantList to all other peers (they'll respond if they can fulfill wants)
                    let wantlist_msg = serde_json::json!({
                        "type": "wantlist",
                        "from_peer_id": peer_id,
                        "version": wantlist.version,
                        "want_ranges": wantlist.want_ranges,
                        "have_ranges": wantlist.have_ranges,
                        "timestamp": wantlist.timestamp,
                    });

                    if let Ok(json) = serde_json::to_string(&wantlist_msg) {
                        let senders = state.peer_senders.read().await;
                        let mut broadcast_count = 0;
                        for (other_peer_id, tx) in senders.iter() {
                            if other_peer_id != &peer_id {
                                if let Err(e) = tx.send(Message::Text(json.clone())) {
                                    warn!("Relay: Failed to broadcast WantList to {}: {}", other_peer_id, e);
                                } else {
                                    broadcast_count += 1;
                                }
                            }
                        }
                        if broadcast_count > 0 {
                            info!("Relay: Broadcasted WantList from {} to {} peers", peer_id, broadcast_count);
                        }
                    }

                    // Respond with ranges we have that they want
                    if !wantlist.want_ranges.is_empty() {
                        // Get P2P manager to check our known peers (slot ownership)
                        if let Some(p2p) = &state.p2p_manager {
                            if let Ok(known_peer_ids) = p2p.get_known_peer_strings() {
                                if !known_peer_ids.is_empty() {
                                    let mesh_config = state.get_mesh_config().await;

                                    // Build ranges from known peer slots
                                    let mut slot_ids: Vec<u64> = known_peer_ids.iter()
                                        .map(|peer_id_str| {
                                            let slot = peer_id_to_slot(peer_id_str, &mesh_config);
                                            (slot.x as u64) * 65536 + (slot.y as u64) * 256 + (slot.z as u64)
                                        })
                                        .collect();

                                    slot_ids.sort();
                                    slot_ids.dedup();

                                    // Build contiguous ranges
                                    let mut our_ranges = Vec::new();
                                    if !slot_ids.is_empty() {
                                        let mut range_start = slot_ids[0];
                                        let mut range_end = slot_ids[0];

                                        for &slot in &slot_ids[1..] {
                                            if slot == range_end + 1 {
                                                range_end = slot;
                                            } else {
                                                our_ranges.push((range_start, range_end));
                                                range_start = slot;
                                                range_end = slot;
                                            }
                                        }
                                        our_ranges.push((range_start, range_end));
                                    }

                                    // Compute intersection: what they WANT that we HAVE
                                    // They want: wantlist.want_ranges
                                    // We have: our_ranges
                                    // Use compute_want_ranges to get intersection:
                                    // compute_want_ranges(what_they_want, gaps_in_what_we_have, total)
                                    // = parts of what_they_want that are IN what_we_have

                                    // First, compute gaps in what we have (inverse of our_ranges)
                                    let mut gaps_in_our_ranges = vec![];
                                    let mut cursor = 0u64;
                                    for &(start, end) in &our_ranges {
                                        if cursor < start {
                                            gaps_in_our_ranges.push((cursor, start - 1));
                                        }
                                        cursor = end + 1;
                                    }
                                    if cursor <= 528 {
                                        gaps_in_our_ranges.push((cursor, 528));
                                    }

                                    // Now compute: what_they_want NOT IN gaps = what_they_want IN our_ranges
                                    let to_send = compute_want_ranges(&wantlist.want_ranges, &gaps_in_our_ranges, (0, 528));

                                    if !to_send.is_empty() {
                                        info!("Relay: Peer {} wants {} ranges, we can provide {} ranges with {} peers",
                                            peer_id, wantlist.want_ranges.len(), to_send.len(), known_peer_ids.len());

                                        // Build and send RangeResponse
                                        let response = RangeResponse {
                                            range: (to_send[0].0, to_send[to_send.len() - 1].1),
                                            entries: known_peer_ids.iter().map(|peer_id_str| {
                                                use crate::spore_wantlist::DhtEntry;
                                                let slot = peer_id_to_slot(peer_id_str, &mesh_config);
                                                let slot_id = (slot.x as u64) * 65536 + (slot.y as u64) * 256 + (slot.z as u64);

                                                DhtEntry {
                                                    key_hash: slot_id,
                                                    key: format!("slot-ownership-{}", slot_id).into_bytes(),
                                                    value: serde_json::to_vec(&serde_json::json!({
                                                        "peer_id": peer_id_str,
                                                        "slot": {"x": slot.x, "y": slot.y, "z": slot.z},
                                                    })).unwrap(),
                                                    timestamp: std::time::SystemTime::now()
                                                        .duration_since(std::time::UNIX_EPOCH)
                                                        .unwrap()
                                                        .as_secs(),
                                                    slot_owner: peer_id_str.clone(),
                                                }
                                            }).collect(),
                                            merkle_proof: None,
                                        };

                                        if let Ok(json) = serde_json::to_string(&serde_json::json!({
                                            "type": "range_response",
                                            "response": response,
                                        })) {
                                            let senders = state.peer_senders.read().await;
                                            if let Some(tx) = senders.get(&peer_id) {
                                                if let Err(e) = tx.send(Message::Text(json)) {
                                                    warn!("Relay: Failed to send RangeResponse to {}: {}", peer_id, e);
                                                } else {
                                                    info!("Relay: Sent RangeResponse with {} entries to {}", response.entries.len(), peer_id);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Try to parse as DHT replication message (epidemic gossip protocol)
                else if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some("dht_replication") = msg_json.get("type").and_then(|v| v.as_str()) {
                        if let Ok(mut replication) = serde_json::from_str::<DhtReplication>(&text) {
                            let key_hex = hex::encode(&replication.key);
                            info!("🌐 Received DHT replication for key={} from {} (hops={}, path={:?})",
                                key_hex, peer_id, replication.hops_remaining, replication.propagation_path);

                            // Check deduplication cache - have we seen this (key, timestamp) before?
                            let cache_key = format!("{}:{}", key_hex, replication.timestamp);
                            let already_seen = {
                                let seen_cache = state.dht_seen_cache.read().await;
                                seen_cache.contains_key(&cache_key)
                            };

                            if already_seen {
                                debug!("🔄 DHT replication already seen, skipping (key={})", key_hex);
                                continue;
                            }

                            // New replication! Mark as seen
                            {
                                let mut seen_cache = state.dht_seen_cache.write().await;
                                seen_cache.insert(cache_key, replication.timestamp);
                            }

                            // Store in local DHT
                            if replication.key.len() == 32 {
                                let mut key_array = [0u8; 32];
                                key_array.copy_from_slice(&replication.key);

                                {
                                    let mut storage = state.dht_storage.lock().await;
                                    storage.insert_raw(key_array, replication.value.clone());
                                }

                                info!("✅ Stored DHT key={} locally via gossip", key_hex);

                                // Forward to 8 neighbors if hops remaining > 0
                                if replication.hops_remaining > 0 {
                                    replication.hops_remaining -= 1;
                                    replication.propagation_path.push(peer_id.clone());

                                    let mesh_config = state.get_mesh_config().await;
                                    let my_slot = peer_id_to_slot(&peer_id, &mesh_config);
                                    let neighbor_slots = get_neighbor_slots(&my_slot, &mesh_config);

                                    // Find connected neighbors
                                    let storage = state.dht_storage.lock().await;
                                    let mut neighbor_peer_ids = Vec::new();

                                    for (_direction, neighbor_slot) in neighbor_slots {
                                        let slot_key = slot_ownership_key(neighbor_slot);
                                        if let Some(ownership_bytes) = storage.get_raw(&slot_key) {
                                            if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(ownership_bytes) {
                                                if ownership.peer_id != peer_id && !ownership.is_stale() {
                                                    neighbor_peer_ids.push(ownership.peer_id.clone());
                                                }
                                            }
                                        }
                                    }

                                    drop(storage);

                                    // Forward to neighbors
                                    if !neighbor_peer_ids.is_empty() {
                                        let forward_msg = serde_json::json!({
                                            "type": "dht_replication",
                                            "replication": replication,
                                        });

                                        if let Ok(json) = serde_json::to_string(&forward_msg) {
                                            let peer_senders = state.peer_senders.read().await;
                                            let mut forward_count = 0;

                                            for neighbor_id in &neighbor_peer_ids {
                                                if let Some(tx) = peer_senders.get(neighbor_id) {
                                                    if let Err(e) = tx.send(Message::Text(json.clone())) {
                                                        warn!("Failed to forward DHT replication to {}: {}", neighbor_id, e);
                                                    } else {
                                                        forward_count += 1;
                                                    }
                                                }
                                            }

                                            if forward_count > 0 {
                                                debug!("🌐 Forwarded DHT replication (key={}) to {} neighbors (hops_remaining={})",
                                                    key_hex, forward_count, replication.hops_remaining);
                                            }
                                        }
                                    }
                                } else {
                                    debug!("⏱️  DHT replication TTL expired (key={}), not forwarding", key_hex);
                                }
                            } else {
                                warn!("Invalid DHT key length: {} (expected 32)", replication.key.len());
                            }
                        }
                        continue;
                    }

                    // DHT Bootstrap Request - node wants to bootstrap from relay's DHT
                    if let Some("dht_bootstrap_request") = msg_json.get("type").and_then(|v| v.as_str()) {
                        if let Ok(request) = serde_json::from_str::<DhtBootstrapRequest>(&text) {
                            info!("🔄 DHT bootstrap request from peer {} at slot {:?}",
                                request.peer_id, request.slot);

                            // Get complete DHT snapshot
                            let dht_snapshot = {
                                let dht = state.dht_storage.lock().await;
                                dht.to_sorted_vec()
                            };

                            // Send bootstrap response with entire DHT
                            let response = DhtBootstrapResponse {
                                dht_entries: dht_snapshot.clone(),
                                entry_count: dht_snapshot.len(),
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };

                            let response_msg = serde_json::json!({
                                "type": "dht_bootstrap_response",
                                "response": response,
                            });

                            if let Ok(json) = serde_json::to_string(&response_msg) {
                                let senders = state.peer_senders.read().await;
                                if let Some(peer_tx) = senders.get(&peer_id) {
                                    if let Err(e) = peer_tx.send(Message::Text(json)) {
                                        warn!("Failed to send DHT bootstrap response to {}: {}", peer_id, e);
                                    } else {
                                        info!("✅ Sent DHT bootstrap ({} entries) to peer {}",
                                            dht_snapshot.len(), peer_id);
                                    }
                                } else {
                                    warn!("Peer {} not found in senders map for bootstrap response", peer_id);
                                }
                            }
                        }
                        continue;
                    }

                    // **DEPRECATED: Block requests should use WebRTC DataChannels, not relay routing**
                    // Relay is ONLY for signaling and epidemic gossip, not data transfer
                    if let Some(msg_type) = msg_json.get("type").and_then(|v| v.as_str()) {
                        if msg_type == "block_request" || msg_type == "block_response" {
                            warn!("🚫 Relay: Ignoring {} from {} - blocks should use WebRTC DataChannels, not relay!",
                                msg_type, peer_id);
                            warn!("💡 Hint: Use SPORE WantList over WebRTC DataChannels for block exchange");
                            continue;
                        }
                    }

                    // Keep epidemic gossip for WantList broadcasting (peer discovery)
                    // But actual data exchange (WantList requests/responses) should use WebRTC
                    if let Some(msg_type) = msg_json.get("type").and_then(|v| v.as_str()) {
                        if msg_type == "wantlist_request" || msg_type == "range_response" {
                            warn!("🚫 Relay: Ignoring {} from {} - use WebRTC DataChannels for data exchange!",
                                msg_type, peer_id);
                            continue;
                        }
                    }

                    // OLD GREEDY FORWARDING CODE - REMOVED
                    // Relay should NOT route data, only broadcast for peer discovery
                    // All data exchange happens via direct WebRTC DataChannels
                    if let Some(msg_type) = msg_json.get("type").and_then(|v| v.as_str()) {
                        if msg_type == "block_request" || msg_type == "block_response" {
                            // This code path should never execute due to check above
                            // But keeping structure for clarity
                            if let Some(_to_peer_id) = msg_json.get("to_peer_id").and_then(|v| v.as_str()) {
                                warn!("🚫 Relay routing disabled for block messages - use WebRTC DataChannels!");
                                continue;
                            }
                        }
                    }

                    // OLD GREEDY FORWARDING CODE REMOVED
                    // Relay is now ONLY for:
                    // 1. WebRTC signaling (SDP/ICE)
                    // 2. WantList epidemic gossip (peer discovery)
                    // 3. Initial DHT bootstrap
                    //
                    // All data exchange uses WebRTC DataChannels!
                }
            }
            Ok(Message::Binary(data)) => {
                info!("Relay: Received binary from {}: {} bytes", peer_id, data.len());

                // Try to parse as TGP packet
                if let Some((header, payload)) = tgp_mod::parse_packet(&data) {
                    info!(
                        "Relay: TGP packet type={:02x} src={:016x} dst={:016x} len={}",
                        header.packet_type, header.source_hex, header.dest_hex, header.payload_length
                    );

                    // Handle DHT packets
                    if let Some(packet_type) = PacketType::from_u8(header.packet_type) {
                        match packet_type {
                            PacketType::DhtGet => {
                                // DHT GET request with hop-by-hop forwarding
                                if let Ok(request) = serde_json::from_slice::<DhtGetRequest>(payload) {
                                    info!("Relay: DHT GET request for key={}", hex::encode(&request.key));

                                    let mesh_config = state.get_mesh_config().await;

                                    // Calculate which slot this key maps to
                                    let target_slot = key_to_slot(&request.key, &mesh_config);

                                    // Get our claimed slot (NOT calculated - slots are Content Addressed!)
                                    let my_slot_opt = {
                                        let peer_slots = state.peer_slots.read().await;
                                        peer_slots.get(&peer_id).cloned()
                                    };

                                    let my_slot = match my_slot_opt {
                                        Some(slot) => {
                                            info!("🔑 Key maps to slot ({}, {}, {}), my claimed slot is ({}, {}, {})",
                                                target_slot.x, target_slot.y, target_slot.z,
                                                slot.x, slot.y, slot.z);
                                            slot
                                        },
                                        None => {
                                            warn!("⚠️ DHT GET: No claimed slot found for peer {}, cannot handle request", peer_id);
                                            continue;
                                        }
                                    };

                                    if target_slot == my_slot {
                                        // This key belongs to OUR slot! Query local storage
                                        info!("✅ Key belongs to our slot, querying local storage");

                                        let value = {
                                            let storage = state.dht_storage.lock().await;
                                            storage.get_raw(&request.key).cloned()
                                        };

                                        // Send DHT_RESPONSE back to requester
                                        let response = DhtResponse {
                                            key: request.key,
                                            value: value.clone(),
                                        };

                                        let response_payload = serde_json::to_vec(&response).unwrap();
                                        let response_packet = tgp_mod::create_packet(
                                            PacketType::DhtResponse.as_u8(),
                                            header.source_hex, // dest = original requester's slot
                                            header.dest_hex,   // source = our slot (where key lives)
                                            &response_payload
                                        );

                                        // Send response back to requester
                                        let senders = state.peer_senders.read().await;
                                        if let Some(tx) = senders.get(&peer_id) {
                                            if let Err(e) = tx.send(Message::Binary(response_packet)) {
                                                warn!("Relay: Failed to send DHT_RESPONSE to {}: {}", peer_id, e);
                                            } else {
                                                info!("Relay: Sent DHT_RESPONSE to {} (found={})", peer_id, value.is_some());
                                            }
                                        }
                                    } else {
                                        // Key belongs to DIFFERENT slot! Forward to closest neighbor
                                        info!("🔀 Key belongs to different slot, forwarding via hop-by-hop routing");

                                        if let Some((closest_peer_id, _closest_slot, distance)) = state.find_closest_peer(target_slot).await {
                                            if closest_peer_id == peer_id {
                                                // Requester is already the closest peer we know
                                                warn!("⚠️  Requester {} is the closest peer to target slot (distance={}), cannot forward", peer_id, distance);

                                                // Send empty response
                                                let response = DhtResponse {
                                                    key: request.key,
                                                    value: None,
                                                };

                                                let response_payload = serde_json::to_vec(&response).unwrap();
                                                let response_packet = tgp_mod::create_packet(
                                                    PacketType::DhtResponse.as_u8(),
                                                    header.source_hex, // dest = original requester's slot
                                                    header.dest_hex,   // source = our slot
                                                    &response_payload
                                                );

                                                let senders = state.peer_senders.read().await;
                                                if let Some(tx) = senders.get(&peer_id) {
                                                    let _ = tx.send(Message::Binary(response_packet));
                                                }
                                            } else {
                                                // Forward to closer peer!
                                                info!("🚀 Hop-by-hop: Forwarding DHT GET to {} (distance to target: {})", closest_peer_id, distance);

                                                // Decrement TTL
                                                let mut forward_header = header.clone();
                                                if forward_header.ttl > 0 {
                                                    forward_header.ttl -= 1;
                                                } else {
                                                    warn!("⏱️ DHT GET TTL expired, dropping packet");
                                                    continue;
                                                }

                                                // Create forwarded packet
                                                let forward_packet = {
                                                    let mut packet = forward_header.to_bytes();
                                                    packet.extend_from_slice(payload);
                                                    packet
                                                };

                                                // Send to closest peer
                                                let senders = state.peer_senders.read().await;
                                                if let Some(tx) = senders.get(&closest_peer_id) {
                                                    if let Err(e) = tx.send(Message::Binary(forward_packet)) {
                                                        warn!("Relay: Failed to forward DHT GET to {}: {}", closest_peer_id, e);
                                                    } else {
                                                        info!("✅ Forwarded DHT GET to {}", closest_peer_id);
                                                    }
                                                }
                                            }
                                        } else {
                                            warn!("⚠️  No connected peers available for DHT forwarding");

                                            // Send empty response
                                            let response = DhtResponse {
                                                key: request.key,
                                                value: None,
                                            };

                                            let response_payload = serde_json::to_vec(&response).unwrap();
                                            let response_packet = tgp_mod::create_packet(
                                                PacketType::DhtResponse.as_u8(),
                                                header.dest_hex,
                                                header.source_hex,
                                                &response_payload
                                            );

                                            let senders = state.peer_senders.read().await;
                                            if let Some(tx) = senders.get(&peer_id) {
                                                let _ = tx.send(Message::Binary(response_packet));
                                            }
                                        }
                                    }
                                }
                            }
                            PacketType::DhtPut => {
                                // DHT PUT request with hop-by-hop routing
                                if let Ok(request) = serde_json::from_slice::<DhtPutRequest>(payload) {
                                    info!("Relay: DHT PUT request for key={} value_len={}", hex::encode(&request.key), request.value.len());

                                    let mesh_config = state.get_mesh_config().await;

                                    // Calculate which slot this key maps to
                                    let target_slot = key_to_slot(&request.key, &mesh_config);

                                    // Calculate our own slot
                                    let my_slot = peer_id_to_slot(&peer_id, &mesh_config);

                                    info!("🔑 PUT: Key maps to slot ({}, {}, {}), my slot is ({}, {}, {})",
                                        target_slot.x, target_slot.y, target_slot.z,
                                        my_slot.x, my_slot.y, my_slot.z);

                                    if target_slot == my_slot {
                                        // This key belongs to OUR slot! Store locally
                                        info!("✅ PUT: Key belongs to our slot, storing locally");

                                        {
                                            let mut storage = state.dht_storage.lock().await;
                                            storage.insert_raw(request.key, request.value.clone());
                                        }

                                        // Send DHT_RESPONSE with success
                                        let response = DhtResponse {
                                            key: request.key,
                                            value: Some(request.value),
                                        };

                                        let response_payload = serde_json::to_vec(&response).unwrap();
                                        let response_packet = tgp_mod::create_packet(
                                            PacketType::DhtResponse.as_u8(),
                                            header.dest_hex,
                                            header.source_hex,
                                            &response_payload
                                        );

                                        // Send response back to requester
                                        let senders = state.peer_senders.read().await;
                                        if let Some(tx) = senders.get(&peer_id) {
                                            if let Err(e) = tx.send(Message::Binary(response_packet)) {
                                                warn!("Relay: Failed to send DHT_RESPONSE to {}: {}", peer_id, e);
                                            } else {
                                                info!("Relay: Sent DHT_RESPONSE to {} (stored successfully)", peer_id);
                                            }
                                        }
                                    } else {
                                        // Key belongs to DIFFERENT slot! Forward to closest neighbor
                                        info!("🔀 PUT: Key belongs to different slot, forwarding via hop-by-hop routing");

                                        if let Some((closest_peer_id, _closest_slot, distance)) = state.find_closest_peer(target_slot).await {
                                            if closest_peer_id == peer_id {
                                                // Requester is already the closest peer we know
                                                warn!("⚠️  PUT: Requester {} is the closest peer to target slot (distance={}), cannot forward", peer_id, distance);

                                                // Send empty response (PUT failed)
                                                let response = DhtResponse {
                                                    key: request.key,
                                                    value: None,
                                                };

                                                let response_payload = serde_json::to_vec(&response).unwrap();
                                                let response_packet = tgp_mod::create_packet(
                                                    PacketType::DhtResponse.as_u8(),
                                                    header.source_hex, // dest = original requester's slot
                                                    header.dest_hex,   // source = our slot
                                                    &response_payload
                                                );

                                                let senders = state.peer_senders.read().await;
                                                if let Some(tx) = senders.get(&peer_id) {
                                                    let _ = tx.send(Message::Binary(response_packet));
                                                }
                                            } else {
                                                // Forward to closer peer!
                                                info!("🚀 Hop-by-hop PUT: Forwarding DHT PUT to {} (distance to target: {})", closest_peer_id, distance);

                                                // Decrement TTL
                                                let mut forward_header = header.clone();
                                                if forward_header.ttl > 0 {
                                                    forward_header.ttl -= 1;
                                                } else {
                                                    warn!("⏱️ DHT PUT TTL expired, dropping packet");
                                                    continue;
                                                }

                                                // Create forwarded packet
                                                let forward_packet = {
                                                    let mut packet = forward_header.to_bytes();
                                                    packet.extend_from_slice(payload);
                                                    packet
                                                };

                                                // Send to closest peer
                                                let senders = state.peer_senders.read().await;
                                                if let Some(tx) = senders.get(&closest_peer_id) {
                                                    if let Err(e) = tx.send(Message::Binary(forward_packet)) {
                                                        warn!("Relay: Failed to forward DHT PUT to {}: {}", closest_peer_id, e);
                                                    } else {
                                                        info!("Relay: Forwarded DHT PUT to {} successfully", closest_peer_id);
                                                    }
                                                } else {
                                                    warn!("Relay: No connection to closest peer {}", closest_peer_id);
                                                }
                                            }
                                        } else {
                                            warn!("⚠️  PUT: No peers known to route towards target slot");

                                            // Send error response
                                            let response = DhtResponse {
                                                key: request.key,
                                                value: None,
                                            };

                                            let response_payload = serde_json::to_vec(&response).unwrap();
                                            let response_packet = tgp_mod::create_packet(
                                                PacketType::DhtResponse.as_u8(),
                                                header.source_hex, // dest = original requester's slot
                                                header.dest_hex,   // source = our slot
                                                &response_payload
                                            );

                                            let senders = state.peer_senders.read().await;
                                            if let Some(tx) = senders.get(&peer_id) {
                                                let _ = tx.send(Message::Binary(response_packet));
                                            }
                                        }
                                    }
                                }
                            }
                            PacketType::DhtResponse => {
                                // DHT RESPONSE - deliver locally or forward toward source
                                if let Ok(response) = serde_json::from_slice::<DhtResponse>(payload) {
                                    let key_hex = hex::encode(&response.key);
                                    info!("📬 Received DHT_RESPONSE for key={}", key_hex);

                                    // Check if we have a pending GET request for this key
                                    let mut pending = state.pending_dht_gets.write().await;
                                    if let Some(pending_get) = pending.remove(&key_hex) {
                                        // This response is for US! Deliver it.
                                        drop(pending);
                                        if pending_get.response_tx.send(response.value).is_err() {
                                            warn!("Failed to send DHT response for key={} (receiver dropped)", key_hex);
                                        } else {
                                            info!("✅ Delivered DHT response for key={} to local dht_get()", key_hex);
                                        }
                                    } else {
                                        // Not for us - forward toward source via hop-by-hop routing
                                        drop(pending);
                                        info!("🔀 DHT_RESPONSE not for us, forwarding toward source_hex={:016x}", header.source_hex);

                                        // Find source slot from source_hex (u64 compact coordinate)
                                        let mesh_config = state.get_mesh_config().await;
                                        let source_slot = u64_to_coord(header.source_hex);

                                        // Find closest peer toward source
                                        if let Some((closest_peer_id, _closest_slot, distance)) = state.find_closest_peer(source_slot).await {
                                            info!("🚀 Forwarding DHT_RESPONSE to {} (distance to source: {})", closest_peer_id, distance);

                                            // Decrement TTL
                                            let mut forward_header = header.clone();
                                            if forward_header.ttl > 0 {
                                                forward_header.ttl -= 1;
                                            } else {
                                                warn!("⏱️ DHT_RESPONSE TTL expired, dropping packet");
                                                continue;
                                            }

                                            // Create forwarded packet
                                            let forward_packet = {
                                                let mut packet = forward_header.to_bytes();
                                                packet.extend_from_slice(payload);
                                                packet
                                            };

                                            // Send to closest peer toward source
                                            let senders = state.peer_senders.read().await;
                                            if let Some(tx) = senders.get(&closest_peer_id) {
                                                if let Err(e) = tx.send(Message::Binary(forward_packet)) {
                                                    warn!("Failed to forward DHT_RESPONSE to {}: {}", closest_peer_id, e);
                                                } else {
                                                    info!("✅ Forwarded DHT_RESPONSE to {}", closest_peer_id);
                                                }
                                            }
                                        } else {
                                            warn!("⚠️  No route back to source for DHT_RESPONSE");
                                        }
                                    }
                                } else {
                                    warn!("Failed to parse DHT_RESPONSE packet");
                                }
                            }
                            _ => {
                                // Other TGP packet types (UBTS, WantList, etc.)
                                info!("Relay: TGP packet type {:?} (not DHT, no special handling)", packet_type);
                            }
                        }
                    }
                } else {
                    info!("Relay: Binary data is not a valid TGP packet");
                }
            }
            Ok(Message::Ping(_)) => {
                // Axum automatically handles pings
            }
            Ok(Message::Pong(_)) => {
                // Response to our ping
            }
            Ok(Message::Close(_)) => {
                info!("Relay: Peer {} closed connection", peer_id);
                break;
            }
            Err(e) => {
                warn!("Relay: WebSocket error for {}: {}", peer_id, e);
                break;
            }
        }
    }

    // Clean up peer sender
    {
        let mut senders = state.peer_senders.write().await;
        senders.remove(&peer_id);
    }

    // Unregister peer
    {
        let mut relay = state.relay.write().await;
        if let Err(e) = relay.unregister_peer(&peer_id) {
            warn!("Relay: Failed to unregister peer {}: {}", peer_id, e);
        }
    }

    // **CITADEL DHT MESH CLEANUP** - Remove slot ownership from DHT
    {
        info!("🧹 Cleaning up peer {} from DHT mesh", peer_id);

        let mesh_config = state.get_mesh_config().await;
        let my_slot = peer_id_to_slot(&peer_id, &mesh_config);

        // Remove both DHT keys:
        // 1. peer_location_key: peer_id → slot mapping
        // 2. slot_ownership_key: slot → peer_id mapping
        let location_key = peer_location_key(&peer_id);
        let slot_key = slot_ownership_key(my_slot);

        {
            let mut storage = state.dht_storage.lock().await;
            storage.remove(&location_key);
            storage.remove(&slot_key);
        }

        info!("✅ Removed peer {} from slot ({}, {}, {}) in DHT mesh",
            peer_id, my_slot.x, my_slot.y, my_slot.z);

        // Broadcast topology update to all remaining peers
        state.broadcast_topology_update(TopologyEventType::PeerLeft, &peer_id, Some(my_slot)).await;
    }

    info!("Relay: Peer {} disconnected", peer_id);
}

/// DHT consistency check endpoint - verifies global DHT state
/// GET /api/v1/dht/consistency
///
/// Returns a report with:
/// - Total DHT entries
/// - Hash of all DHT keys (should be IDENTICAL on all nodes with CAS + gossip!)
/// - Known peer count
/// - This node's peer ID and slot
pub async fn dht_consistency_handler(
    State(state): State<RelayState>,
) -> Result<axum::Json<DhtConsistencyReport>, StatusCode> {
    // Get my peer ID from RelayState (NOT from connected WebSocket peers!)
    let node_peer_id_lock = state.node_peer_id.read().await;
    let my_peer_id = node_peer_id_lock.as_ref()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());
    drop(node_peer_id_lock);

    let mesh_config = state.get_mesh_config().await;

    // Check if explicit slot is set (for tests), otherwise calculate from peer_id
    let my_slot = {
        let slot_lock = state.my_slot.read().await;
        if let Some(slot) = slot_lock.as_ref() {
            *slot
        } else {
            peer_id_to_slot(&my_peer_id, &mesh_config)
        }
    };

    // Get all DHT keys and hash them
    let storage = state.dht_storage.lock().await;
    let total_entries = storage.len();

    // Sort keys for consistent hashing
    let mut keys: Vec<[u8; 32]> = storage.keys().copied().collect();
    keys.sort();

    // Hash all keys together using Blake3
    use blake3;
    let mut hasher = blake3::Hasher::new();
    for key in &keys {
        hasher.update(key);
    }
    let dht_keys_hash = hex::encode(hasher.finalize().as_bytes());

    // Count known peers from DHT
    let known_peers = keys.iter()
        .filter(|key| {
            // Check if this is a peer_location_key (starts with 0x01)
            key[0] == 0x01
        })
        .count();

    drop(storage);

    let report = DhtConsistencyReport {
        total_entries,
        dht_keys_hash,
        known_peers,
        my_peer_id,
        my_slot,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    info!("📊 DHT Consistency: {} entries, hash={}, {} peers",
        report.total_entries, &report.dht_keys_hash[..16], report.known_peers);

    Ok(axum::Json(report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_state_creation() {
        let state = RelayState::new();
        assert!(Arc::strong_count(&state.relay) >= 1);
    }

    #[test]
    fn test_neighbor_cache_staleness() {
        let cache = NeighborCache::new(
            SlotCoordinate::new(5, 10, 2),
            "peer-123".to_string(),
        );
        assert!(!cache.is_stale());
    }
}

// Re-export rand for peer IDs
use rand;

/// DHT GET response for HTTP endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtGetHttpResponse {
    /// The key that was requested (hex encoded)
    pub key: String,
    /// The value if found (hex encoded), None if not found
    pub value: Option<String>,
}

/// DHT PUT request for HTTP endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtPutHttpRequest {
    /// The key to store (hex encoded)
    pub key: String,
    /// The value to store (hex encoded)
    pub value: String,
}

/// HTTP handler for DHT GET operations
/// GET /api/v1/dht/get/:key_hex
///
/// Returns the value associated with the key if found, or None if not found.
/// The key must be a hex-encoded 32-byte Blake3 hash.
pub async fn dht_get_handler(
    State(state): State<RelayState>,
    axum::extract::Path(key_hex): axum::extract::Path<String>,
) -> Result<axum::Json<DhtGetHttpResponse>, StatusCode> {
    // Decode hex key
    let key_bytes = hex::decode(&key_hex).map_err(|e| {
        warn!("DHT GET HTTP: Invalid hex key: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Validate key length
    if key_bytes.len() != 32 {
        warn!("DHT GET HTTP: Key must be 32 bytes, got {}", key_bytes.len());
        return Err(StatusCode::BAD_REQUEST);
    }

    // Convert to array
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    info!("DHT GET HTTP: key={}", key_hex);

    // Perform DHT GET (routes through the mesh)
    let value = state.dht_get(key).await;

    let response = DhtGetHttpResponse {
        key: key_hex,
        value: value.map(hex::encode),
    };

    info!("DHT GET HTTP: found={}", response.value.is_some());
    Ok(axum::Json(response))
}

/// HTTP handler for LOCAL DHT GET operations (no routing)
/// GET /api/v1/dht/get_local/:key_hex
///
/// Queries LOCAL storage only without routing through the mesh.
/// This is used in tests to verify that dht_get() actually routes via network.
/// Returns the value if found in local storage, or None if not found locally.
pub async fn dht_get_local_handler(
    State(state): State<RelayState>,
    axum::extract::Path(key_hex): axum::extract::Path<String>,
) -> Result<axum::Json<DhtGetHttpResponse>, StatusCode> {
    // Decode hex key
    let key_bytes = hex::decode(&key_hex).map_err(|e| {
        warn!("DHT GET LOCAL: Invalid hex key: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Validate key length
    if key_bytes.len() != 32 {
        warn!("DHT GET LOCAL: Key must be 32 bytes, got {}", key_bytes.len());
        return Err(StatusCode::BAD_REQUEST);
    }

    // Convert to array
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    info!("DHT GET LOCAL: key={} (local storage only, no routing)", key_hex);

    // Query LOCAL storage only (no routing!)
    let value = {
        let storage = state.dht_storage.lock().await;
        storage.get_raw(&key).map(|v| v.to_vec())
    };

    let response = DhtGetHttpResponse {
        key: key_hex,
        value: value.map(hex::encode),
    };

    info!("DHT GET LOCAL: found={}", response.value.is_some());
    Ok(axum::Json(response))
}

/// HTTP handler for DHT PUT operations
/// POST /api/v1/dht/put
///
/// Stores a key-value pair in the DHT. The key and value must be hex-encoded.
/// The key must be a 32-byte Blake3 hash.
pub async fn dht_put_handler(
    State(state): State<RelayState>,
    axum::Json(request): axum::Json<DhtPutHttpRequest>,
) -> Result<StatusCode, StatusCode> {
    // Decode hex key
    let key_bytes = hex::decode(&request.key).map_err(|e| {
        warn!("DHT PUT HTTP: Invalid hex key: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Validate key length
    if key_bytes.len() != 32 {
        warn!("DHT PUT HTTP: Key must be 32 bytes, got {}", key_bytes.len());
        return Err(StatusCode::BAD_REQUEST);
    }

    // Convert to array
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);

    // Decode hex value
    let value = hex::decode(&request.value).map_err(|e| {
        warn!("DHT PUT HTTP: Invalid hex value: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    info!("DHT PUT HTTP: key={} value_len={}", request.key, value.len());

    // Perform DHT PUT (routes through the mesh to the correct slot)
    state.dht_put(key, value).await;

    info!("DHT PUT HTTP: Success");
    Ok(StatusCode::OK)
}

/// Request body for gossip_slot_ownership HTTP endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipSlotOwnershipRequest {
    /// Peer ID announcing slot ownership
    pub peer_id: String,
    /// Slot coordinate
    pub slot: SlotCoordinate,
}

/// POST /api/v1/dht/gossip_slot_ownership
///
/// Gossips slot ownership announcement to all connected peers (both WebSocket and WebRTC).
/// This enables distributed slot discovery without circular dependencies.
pub async fn gossip_slot_ownership_handler(
    State(state): State<RelayState>,
    axum::Json(request): axum::Json<GossipSlotOwnershipRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("📢 HTTP: Gossiping slot ownership: {} → ({}, {}, {})",
        request.peer_id, request.slot.x, request.slot.y, request.slot.z);

    // Call the gossip method
    state.gossip_slot_ownership(request.peer_id, request.slot).await;

    info!("✅ HTTP: Slot ownership gossip initiated");
    Ok(StatusCode::OK)
}

/// WebRTC offer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcOfferRequest {
    /// Target peer ID to connect to
    pub to_peer_id: String,
}

/// WebRTC offer response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcOfferResponse {
    /// SDP offer string
    pub sdp: String,
}

/// WebRTC answer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcAnswerRequest {
    /// Source peer ID (who sent the offer)
    pub from_peer_id: String,
    /// SDP offer from the source peer
    pub sdp: String,
}

/// WebRTC answer response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcAnswerResponse {
    /// SDP answer string
    pub sdp: String,
}

/// WebRTC complete request (send answer back to offerer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcCompleteRequest {
    /// Source peer ID (who sent the answer)
    pub from_peer_id: String,
    /// SDP answer from the answerer
    pub sdp: String,
}

/// HTTP handler for creating WebRTC offers
/// POST /api/v1/webrtc/offer
///
/// Creates a WebRTC offer to connect to another node
pub async fn webrtc_offer_handler(
    State(state): State<RelayState>,
    axum::Json(request): axum::Json<WebRtcOfferRequest>,
) -> Result<axum::Json<WebRtcOfferResponse>, StatusCode> {
    info!("WebRTC offer request for peer: {}", request.to_peer_id);

    let webrtc_mgr = state.webrtc_manager
        .as_ref()
        .ok_or_else(|| {
            warn!("WebRTC manager not available");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create offer using WebRTCManager
    let sdp = webrtc_mgr.create_offer(request.to_peer_id.clone())
        .await
        .map_err(|e| {
            warn!("Failed to create WebRTC offer: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Created WebRTC offer (SDP length: {} bytes)", sdp.len());

    Ok(axum::Json(WebRtcOfferResponse { sdp }))
}

/// HTTP handler for handling WebRTC offers and creating answers
/// POST /api/v1/webrtc/answer
///
/// Receives a WebRTC offer and returns an answer
pub async fn webrtc_answer_handler(
    State(state): State<RelayState>,
    axum::Json(request): axum::Json<WebRtcAnswerRequest>,
) -> Result<axum::Json<WebRtcAnswerResponse>, StatusCode> {
    info!("WebRTC answer request from peer: {}", request.from_peer_id);

    let webrtc_mgr = state.webrtc_manager
        .as_ref()
        .ok_or_else(|| {
            warn!("WebRTC manager not available");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Handle offer and create answer
    let sdp = webrtc_mgr.handle_offer(
        request.from_peer_id.clone(),
        request.sdp,
        crate::webrtc_manager::PeerType::Node
    )
        .await
        .map_err(|e| {
            warn!("Failed to handle WebRTC offer: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Created WebRTC answer (SDP length: {} bytes)", sdp.len());

    Ok(axum::Json(WebRtcAnswerResponse { sdp }))
}

/// HTTP handler for completing WebRTC connection with answer
/// POST /api/v1/webrtc/complete
///
/// Completes the WebRTC connection by setting the remote answer
pub async fn webrtc_complete_handler(
    State(state): State<RelayState>,
    axum::Json(request): axum::Json<WebRtcCompleteRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("WebRTC complete request from peer: {}", request.from_peer_id);

    let webrtc_mgr = state.webrtc_manager
        .as_ref()
        .ok_or_else(|| {
            warn!("WebRTC manager not available");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get the peer connection
    let peers = webrtc_mgr.peers.read().await;
    let peer = peers.get(&request.from_peer_id)
        .ok_or_else(|| {
            warn!("Peer {} not found", request.from_peer_id);
            StatusCode::NOT_FOUND
        })?;

    // Set remote description (answer from the other peer)
    use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
    let answer = RTCSessionDescription::answer(request.sdp)
        .map_err(|e| {
            warn!("Failed to parse SDP answer: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    peer.peer_connection.set_remote_description(answer)
        .await
        .map_err(|e| {
            warn!("Failed to set remote description: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    drop(peers);

    info!("WebRTC connection completed with peer: {}", request.from_peer_id);

    Ok(StatusCode::OK)
}
