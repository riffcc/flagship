use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    http::StatusCode,
    response::IntoResponse,
};
use consensus_peerexc::{
    relay::RelayServer,
    wantlist::WantList,
    PeerInfo, PeerState,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug};
use crate::tgp::{self as tgp_mod, PacketType, DhtGetRequest, DhtPutRequest, DhtResponse};
use citadel_dht::local_storage::LocalStorage;
use crate::peer_registry::{
    SlotOwnership, peer_location_key, slot_ownership_key,
    peer_id_to_slot, get_neighbor_slots, default_mesh_config
};
use citadel_core::topology::{SlotCoordinate, MeshConfig};
use citadel_core::routing::greedy_direction;
use std::time::{SystemTime, UNIX_EPOCH};

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

/// DHT mesh health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtMeshHealth {
    /// Total number of connected peers
    pub total_peers: usize,
    /// Number of 8-neighbor connections established (out of 8 possible)
    pub neighbor_connections: usize,
    /// Percentage of neighbors online (0.0 - 1.0)
    pub mesh_connectivity: f64,
    /// Whether the mesh is fragmented (connectivity < 50%)
    pub is_fragmented: bool,
    /// Timestamp of last health check
    pub last_check: u64,
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

/// Relay state shared across WebSocket connections
#[derive(Clone)]
pub struct RelayState {
    pub relay: Arc<RwLock<RelayServer>>,
    pub peer_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
    pub webrtc_manager: Option<Arc<crate::webrtc_manager::WebRTCManager>>,
    /// Browser-discovered peers - browsers share their WebRTC connections to help nodes find each other
    pub browser_discovered_peers: Arc<RwLock<HashMap<String, Vec<BrowserDiscoveredPeer>>>>,
    /// DHT storage for P2P distributed key-value store
    pub dht_storage: Arc<tokio::sync::Mutex<LocalStorage>>,
    /// Cached neighbor slot ownership (peer_id -> vec of neighbor caches)
    neighbor_cache: Arc<RwLock<HashMap<String, Vec<NeighborCache>>>>,
    /// DHT mesh health metrics
    mesh_health: Arc<RwLock<DhtMeshHealth>>,
}

impl RelayState {
    pub fn new() -> Self {
        Self {
            relay: Arc::new(RwLock::new(RelayServer::new())),
            peer_senders: Arc::new(RwLock::new(HashMap::new())),
            webrtc_manager: None,
            browser_discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            dht_storage: Arc::new(tokio::sync::Mutex::new(LocalStorage::new())),
            neighbor_cache: Arc::new(RwLock::new(HashMap::new())),
            mesh_health: Arc::new(RwLock::new(DhtMeshHealth {
                total_peers: 0,
                neighbor_connections: 0,
                mesh_connectivity: 0.0,
                is_fragmented: true,
                last_check: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })),
        }
    }

    pub fn with_webrtc(mut self, manager: Arc<crate::webrtc_manager::WebRTCManager>) -> Self {
        self.webrtc_manager = Some(manager);
        self
    }

    pub fn with_dht_storage(mut self, storage: Arc<tokio::sync::Mutex<LocalStorage>>) -> Self {
        self.dht_storage = storage;
        self
    }

    /// Get cached neighbors for a peer, or query DHT if cache is stale
    async fn get_cached_neighbors(&self, peer_id: &str, my_slot: SlotCoordinate) -> Vec<(String, SlotCoordinate)> {
        let mesh_config = default_mesh_config();

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

            if let Some(ownership_bytes) = storage.get(&slot_key) {
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

    /// Update mesh health metrics
    async fn update_mesh_health(&self) {
        let mesh_config = default_mesh_config();
        let peer_senders = self.peer_senders.read().await;
        let total_peers = peer_senders.len();

        if total_peers == 0 {
            return;
        }

        // Count how many 8-neighbor connections exist
        let mut total_neighbor_connections = 0;
        let mut total_possible_connections = 0;

        for peer_id in peer_senders.keys() {
            let my_slot = peer_id_to_slot(peer_id, &mesh_config);
            let neighbors = self.get_cached_neighbors(peer_id, my_slot).await;

            total_neighbor_connections += neighbors.len();
            total_possible_connections += 8; // Each peer should have 8 neighbors
        }

        drop(peer_senders);

        let mesh_connectivity = if total_possible_connections > 0 {
            total_neighbor_connections as f64 / total_possible_connections as f64
        } else {
            0.0
        };

        let is_fragmented = mesh_connectivity < 0.5;

        let mut health = self.mesh_health.write().await;
        health.total_peers = total_peers;
        health.neighbor_connections = total_neighbor_connections;
        health.mesh_connectivity = mesh_connectivity;
        health.is_fragmented = is_fragmented;
        health.last_check = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if is_fragmented {
            warn!("⚠️  DHT mesh is FRAGMENTED! Connectivity: {:.1}% ({}/{} neighbors)",
                mesh_connectivity * 100.0, total_neighbor_connections, total_possible_connections);
        } else {
            info!("✅ DHT mesh healthy: {:.1}% connectivity ({}/{} neighbors)",
                mesh_connectivity * 100.0, total_neighbor_connections, total_possible_connections);
        }
    }

    /// Find the closest peer to a target slot using greedy routing
    async fn find_closest_peer(&self, target_slot: SlotCoordinate) -> Option<(String, SlotCoordinate, i32)> {
        let mesh_config = default_mesh_config();
        let peer_senders = self.peer_senders.read().await;

        let mut closest_peer: Option<(String, SlotCoordinate, i32)> = None;
        let mut min_distance = i32::MAX;

        for peer_id in peer_senders.keys() {
            let peer_slot = peer_id_to_slot(peer_id, &mesh_config);
            let (dx, dy, dz) = peer_slot.distance_to(&target_slot, &mesh_config);
            let distance = dx.abs() + dy.abs() + dz.abs(); // Manhattan distance

            if distance < min_distance {
                min_distance = distance;
                closest_peer = Some((peer_id.clone(), peer_slot, distance));
            }
        }

        drop(peer_senders);
        closest_peer
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
                        info!("Relay: Client announced peer_id: {}", client_peer_id);
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
    {
        let mesh_config = default_mesh_config();
        let my_slot = peer_id_to_slot(&peer_id, &mesh_config);

        info!("📢 Peer {} assigned to slot ({}, {}, {}) in hexagonal toroidal mesh",
            peer_id, my_slot.x, my_slot.y, my_slot.z);

        // Create slot ownership announcement
        let ownership = SlotOwnership::new(peer_id.clone(), my_slot, None);
        let ownership_bytes = serde_json::to_vec(&ownership).unwrap_or_default();

        // Announce at TWO DHT keys for recursive discovery:
        // 1. peer_location_key: "where is this peer?" (peer_id → slot)
        // 2. slot_ownership_key: "who owns this slot?" (slot → peer_id)

        let location_key = peer_location_key(&peer_id);
        let slot_key = slot_ownership_key(my_slot);

        {
            let mut storage = state.dht_storage.lock().await;
            storage.put(location_key, ownership_bytes.clone());
            storage.put(slot_key, ownership_bytes);
        }

        info!("✅ Announced slot ownership for peer {} at ({}, {}, {})",
            peer_id, my_slot.x, my_slot.y, my_slot.z);

        // Log 8 neighbor slots for visibility (lazy discovery will query these on-demand!)
        let neighbors = get_neighbor_slots(&my_slot, &mesh_config);
        info!("🔷 Peer {} has 8 mesh neighbors at slots: {:?}",
            peer_id, neighbors.iter().map(|(_, s)| (s.x, s.y, s.z)).collect::<Vec<_>>());
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

    // **ENHANCEMENT 3: Periodic Mesh Health Monitoring**
    // Spawn task to update mesh health every 30 seconds
    let state_clone = state.clone();
    let peer_id_health = peer_id.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;

            // Check if this peer is still connected
            {
                let senders = state_clone.peer_senders.read().await;
                if !senders.contains_key(&peer_id_health) {
                    debug!("Mesh health monitor stopping for disconnected peer {}", peer_id_health);
                    break;
                }
            }

            // Update mesh health metrics
            state_clone.update_mesh_health().await;
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

                        // If WebRTC manager available, create connection to browser
                        if let Some(ref webrtc_mgr) = state.webrtc_manager {
                            let mgr = webrtc_mgr.clone();
                            let browser_peer_id = peer_id.clone();
                            tokio::spawn(async move {
                                if let Err(e) = mgr.create_peer_connection(browser_peer_id.clone()).await {
                                    warn!("Relay: Failed to create WebRTC connection to {}: {}", browser_peer_id, e);
                                }
                            });
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
                // Try to parse as WantList (must be before generic JSON check)
                else if let Ok(wantlist) = serde_json::from_str::<WantList>(&text) {
                    info!("Relay: Received WantList from {}: gen={}, needs={}, offers={}",
                        peer_id, wantlist.generation, wantlist.has_needs(), wantlist.has_offers());

                    // Index the WantList
                    {
                        let mut relay = state.relay.write().await;
                        relay.index_wantlist(peer_id.clone(), &wantlist);
                    }

                    // Broadcast WantList to all other peers for SPORE comparison
                    let wantlist_msg = serde_json::json!({
                        "type": "wantlist_announcement",
                        "from_peer_id": peer_id,
                        "wantlist": wantlist,
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

                    // Find providers for this peer's needs
                    let providers = {
                        let relay = state.relay.read().await;
                        relay.find_providers(&wantlist)
                    };

                    info!("Relay: Found {} providers for {}", providers.len(), peer_id);

                    // Get all connected peers for peer discovery
                    let all_peers = {
                        let relay = state.relay.read().await;
                        relay.get_peers()
                    };

                    // Filter out self and combine with providers
                    let mut peers_to_send: Vec<_> = all_peers
                        .into_iter()
                        .filter(|p| p.peer_id != peer_id)
                        .collect();

                    // Add specific providers if available
                    for provider in providers {
                        if !peers_to_send.iter().any(|p| p.peer_id == provider.peer_id) {
                            peers_to_send.push(provider);
                        }
                    }

                    // Add browser-discovered peers (browsers help nodes find each other!)
                    {
                        let browser_peers = state.browser_discovered_peers.read().await;
                        let mut added_browser_peers = 0;

                        for (_browser_id, discovered_peers) in browser_peers.iter() {
                            for discovered_peer in discovered_peers {
                                // Only include connected peers
                                if discovered_peer.connected {
                                    // Convert browser-discovered peer to PeerInfo format
                                    // Use score=100 for browser-discovered peers (they're direct connections)
                                    if !peers_to_send.iter().any(|p| p.peer_id == discovered_peer.peer_id)
                                       && discovered_peer.peer_id != peer_id {
                                        let mut peer_info = PeerInfo::new(discovered_peer.peer_id.clone());
                                        peer_info.score = 100; // High score for direct browser connections
                                        peer_info.state = PeerState::Discovered; // Discovered via browser
                                        peers_to_send.push(peer_info);
                                        added_browser_peers += 1;
                                    }
                                }
                            }
                        }

                        if added_browser_peers > 0 {
                            info!("Relay: Added {} browser-discovered peers to referral for {}", added_browser_peers, peer_id);
                        }
                    }

                    // **CITADEL DHT MESH PEER DISCOVERY** (Lazy 8-neighbor query!)
                    // Query DHT for peers in 8 neighboring slots - TRUE MESH TOPOLOGY!
                    {
                        let mesh_config = default_mesh_config();
                        let my_slot = peer_id_to_slot(&peer_id, &mesh_config);
                        let neighbor_slots = get_neighbor_slots(&my_slot, &mesh_config);

                        let storage = state.dht_storage.lock().await;
                        let mut added_mesh_peers = 0;

                        for (_direction, neighbor_slot) in neighbor_slots {
                            // Query DHT for "who owns this neighbor slot?"
                            let slot_key = slot_ownership_key(neighbor_slot);

                            if let Some(ownership_bytes) = storage.get(&slot_key) {
                                if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(ownership_bytes) {
                                    // Skip self
                                    if ownership.peer_id == peer_id {
                                        continue;
                                    }

                                    // Skip if already in list
                                    if peers_to_send.iter().any(|p| p.peer_id == ownership.peer_id) {
                                        continue;
                                    }

                                    // Check if peer is still fresh (not stale)
                                    if !ownership.is_stale() {
                                        let mut peer_info = PeerInfo::new(ownership.peer_id.clone());
                                        peer_info.score = 100; // High score for mesh neighbors
                                        peer_info.state = PeerState::Discovered;
                                        peers_to_send.push(peer_info);
                                        added_mesh_peers += 1;
                                    }
                                }
                            }
                        }

                        if added_mesh_peers > 0 {
                            info!("🔷 MESH DISCOVERY: Added {} mesh neighbors to referral for {}",
                                added_mesh_peers, peer_id);
                        }
                    }

                    // **ENHANCEMENT 1: Add DHT Routing Hints to Peer Referrals**
                    // Include SlotCoordinate for each peer so recipients can calculate routing distances!
                    if !peers_to_send.is_empty() {
                        let mesh_config = default_mesh_config();

                        let referral = serde_json::json!({
                            "type": "peer_referral",
                            "your_peer_id": peer_id,
                            "your_slot": {
                                "x": peer_id_to_slot(&peer_id, &mesh_config).x,
                                "y": peer_id_to_slot(&peer_id, &mesh_config).y,
                                "z": peer_id_to_slot(&peer_id, &mesh_config).z,
                            },
                            "peers": peers_to_send.into_iter().take(10).map(|p| {
                                let peer_slot = peer_id_to_slot(&p.peer_id, &mesh_config);
                                serde_json::json!({
                                    "peer_id": p.peer_id,
                                    "latest_height": p.latest_height,
                                    "score": p.score,
                                    "slot": {
                                        "x": peer_slot.x,
                                        "y": peer_slot.y,
                                        "z": peer_slot.z,
                                    },
                                })
                            }).collect::<Vec<_>>(),
                        });

                        info!("Relay: Sending referral to {} with {} peers (with DHT routing hints)",
                            peer_id, referral["peers"].as_array().map(|a| a.len()).unwrap_or(0));

                        let senders = state.peer_senders.read().await;
                        if let Some(tx) = senders.get(&peer_id) {
                            if let Ok(json) = serde_json::to_string(&referral) {
                                if let Err(e) = tx.send(Message::Text(json)) {
                                    warn!("Relay: Failed to send referral to {}: {}", peer_id, e);
                                } else {
                                    info!("Relay: Successfully sent peer referral with routing hints to {}", peer_id);
                                }
                            }
                        }
                    } else {
                        info!("Relay: No other peers to refer to {}", peer_id);
                    }
                }
                // **ENHANCEMENT 2: Greedy Message Forwarding**
                // Try to parse as block_request or block_response and route them using greedy forwarding
                else if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(msg_type) = msg_json.get("type").and_then(|v| v.as_str()) {
                        if msg_type == "block_request" || msg_type == "block_response" {
                            if let Some(to_peer_id) = msg_json.get("to_peer_id").and_then(|v| v.as_str()) {
                                let senders = state.peer_senders.read().await;

                                // Direct connection available?
                                if let Some(target_tx) = senders.get(to_peer_id).cloned() {
                                    drop(senders);
                                    info!("Relay: Direct routing {} from {} to {}", msg_type, peer_id, to_peer_id);
                                    if let Err(e) = target_tx.send(Message::Text(text.clone())) {
                                        warn!("Relay: Failed to route {} to {}: {}", msg_type, to_peer_id, e);
                                    } else {
                                        info!("Relay: Routed {} from {} to {}", msg_type, peer_id, to_peer_id);
                                    }
                                } else {
                                    // No direct connection - use GREEDY FORWARDING!
                                    drop(senders);

                                    let mesh_config = default_mesh_config();
                                    let target_slot = peer_id_to_slot(to_peer_id, &mesh_config);

                                    // Find closest peer to target
                                    if let Some((closest_peer_id, closest_slot, distance)) = state.find_closest_peer(target_slot).await {
                                        if closest_peer_id == peer_id {
                                            // We're already the closest peer - can't forward
                                            warn!("Relay: Target peer {} not connected and we're the closest (distance={})", to_peer_id, distance);
                                        } else {
                                            // Forward to closer peer!
                                            info!("🔀 Greedy forwarding {} from {} → {} (hop towards {}), distance: {}",
                                                msg_type, peer_id, closest_peer_id, to_peer_id, distance);

                                            let senders = state.peer_senders.read().await;
                                            if let Some(forward_tx) = senders.get(&closest_peer_id) {
                                                // Add routing metadata to track hops
                                                let mut forwarded_msg = msg_json.clone();
                                                if let Some(obj) = forwarded_msg.as_object_mut() {
                                                    // Track routing path
                                                    let mut hops = obj.get("routing_hops")
                                                        .and_then(|v| v.as_array())
                                                        .cloned()
                                                        .unwrap_or_default();
                                                    hops.push(serde_json::json!({
                                                        "relay": peer_id,
                                                        "forwarded_to": closest_peer_id,
                                                        "distance_to_target": distance,
                                                    }));
                                                    obj.insert("routing_hops".to_string(), serde_json::Value::Array(hops));
                                                }

                                                if let Ok(forwarded_json) = serde_json::to_string(&forwarded_msg) {
                                                    if let Err(e) = forward_tx.send(Message::Text(forwarded_json)) {
                                                        warn!("Relay: Failed to greedy forward to {}: {}", closest_peer_id, e);
                                                    } else {
                                                        info!("Relay: Greedy forwarded {} from {} → {} (towards {})",
                                                            msg_type, peer_id, closest_peer_id, to_peer_id);
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        warn!("Relay: No connected peers available for greedy forwarding to {}", to_peer_id);
                                    }
                                }
                            }
                        }
                    }
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
                                // DHT GET request
                                if let Ok(request) = serde_json::from_slice::<DhtGetRequest>(payload) {
                                    info!("Relay: DHT GET request for key={}", hex::encode(&request.key));

                                    // Query local DHT storage
                                    let value = {
                                        let storage = state.dht_storage.lock().await;
                                        storage.get(&request.key).cloned()
                                    };

                                    // Send DHT_RESPONSE back to requester
                                    let response = DhtResponse {
                                        key: request.key,
                                        value,
                                    };

                                    let response_payload = serde_json::to_vec(&response).unwrap();
                                    let response_packet = tgp_mod::create_packet(
                                        PacketType::DhtResponse.as_u8(),
                                        header.dest_hex, // We are the destination, respond to source
                                        header.source_hex, // Send to original source
                                        &response_payload
                                    );

                                    // Send response back to requester
                                    let senders = state.peer_senders.read().await;
                                    if let Some(tx) = senders.get(&peer_id) {
                                        if let Err(e) = tx.send(Message::Binary(response_packet)) {
                                            warn!("Relay: Failed to send DHT_RESPONSE to {}: {}", peer_id, e);
                                        } else {
                                            info!("Relay: Sent DHT_RESPONSE to {} (found={}", peer_id, response.value.is_some());
                                        }
                                    }
                                }
                            }
                            PacketType::DhtPut => {
                                // DHT PUT request
                                if let Ok(request) = serde_json::from_slice::<DhtPutRequest>(payload) {
                                    info!("Relay: DHT PUT request for key={} value_len={}", hex::encode(&request.key), request.value.len());

                                    // Store in local DHT
                                    {
                                        let mut storage = state.dht_storage.lock().await;
                                        storage.put(request.key, request.value.clone());
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
                                }
                            }
                            PacketType::DhtResponse => {
                                // DHT RESPONSE - route to destination peer
                                info!("Relay: Routing DHT_RESPONSE to peer");

                                // Find the destination peer and forward the packet
                                let target_hex = header.dest_hex;
                                let senders = state.peer_senders.read().await;

                                // TODO: Implement hex-based routing to find closest peer
                                // For now, just log that we received it
                                info!("Relay: DHT_RESPONSE for target={:016x} (routing not yet implemented)", target_hex);
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

        let mesh_config = default_mesh_config();
        let my_slot = peer_id_to_slot(&peer_id, &mesh_config);

        // Remove both DHT keys:
        // 1. peer_location_key: peer_id → slot mapping
        // 2. slot_ownership_key: slot → peer_id mapping
        let location_key = peer_location_key(&peer_id);
        let slot_key = slot_ownership_key(my_slot);

        {
            let mut storage = state.dht_storage.lock().await;
            storage.delete(&location_key);
            storage.delete(&slot_key);
        }

        info!("✅ Removed peer {} from slot ({}, {}, {}) in DHT mesh",
            peer_id, my_slot.x, my_slot.y, my_slot.z);
    }

    info!("Relay: Peer {} disconnected", peer_id);
}

/// **ENHANCEMENT 3: DHT Health Monitoring API**
/// GET /api/v1/dht/health - Get DHT mesh health metrics
///
/// Returns mesh connectivity statistics including:
/// - Total connected peers
/// - Number of 8-neighbor connections
/// - Mesh connectivity percentage
/// - Fragmentation status
pub async fn dht_health_handler(
    State(state): State<RelayState>,
) -> Result<axum::Json<DhtMeshHealth>, StatusCode> {
    let health = state.mesh_health.read().await;
    Ok(axum::Json(health.clone()))
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

    #[tokio::test]
    async fn test_dht_mesh_health_default() {
        let state = RelayState::new();
        let health = state.mesh_health.read().await.clone();
        assert_eq!(health.total_peers, 0);
        assert!(health.is_fragmented);
    }
}

// Re-export rand for peer IDs
use rand;
