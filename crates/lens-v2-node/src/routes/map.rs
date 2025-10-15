//! Network Map API
//!
//! Provides live visualization data for the hexagonal toroidal mesh topology.
//! Accessible via `/api/v1/map` and visualized in the frontend with "batmanbatmanbatman" trigger.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use citadel_core::topology::{SlotCoordinate, MeshConfig};

use crate::peer_registry::{
    peer_id_to_slot, get_neighbor_slots, default_mesh_config,
    SlotOwnership, peer_location_key, slot_ownership_key,
};
use crate::slot_identity::SlotId;
use crate::latency::latency_to_hex;
use super::{RelayState, SyncState};

/// Combined state for map endpoint (needs both relay and sync state)
#[derive(Clone)]
pub struct MapState {
    pub relay: RelayState,
    pub sync: SyncState,
}

/// Network map response containing nodes and edges for force-directed graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMap {
    /// Mesh configuration (dimensions of the hexagonal toroidal mesh)
    pub mesh_config: MeshConfigData,

    /// All active peers in the network
    pub nodes: Vec<PeerNode>,

    /// Connections between peers (8-neighbor mesh topology)
    pub edges: Vec<PeerEdge>,

    /// Statistics about the network
    pub stats: NetworkStats,
}

/// Mesh configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfigData {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub total_slots: usize,
}

/// A peer node in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerNode {
    /// Peer ID (anonymized for browser peers)
    pub id: String,

    /// Display label for the node
    pub label: String,

    /// Permanent content-addressed slot ID (Blake3)
    pub slot_id: String,

    /// Slot coordinate in the hexagonal mesh
    pub slot: SlotCoordinateData,

    /// Type of peer (server node vs browser peer)
    pub peer_type: PeerType,

    /// Last heartbeat timestamp (Unix seconds)
    pub last_heartbeat: u64,

    /// Capabilities (webrtc, dht, spore, etc.)
    pub capabilities: Vec<String>,

    /// Whether this peer is currently online
    pub online: bool,

    /// Average latency to 8 neighbors (milliseconds, if available)
    pub avg_neighbor_latency_ms: Option<u64>,
}

/// Slot coordinate data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotCoordinateData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<SlotCoordinate> for SlotCoordinateData {
    fn from(coord: SlotCoordinate) -> Self {
        Self {
            x: coord.x,
            y: coord.y,
            z: coord.z,
        }
    }
}

/// Type of peer in the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PeerType {
    /// Server node (lens-v2-node instance)
    Server,

    /// Browser peer (ephemeral, anonymized)
    Browser,
}

/// An edge connecting two peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEdge {
    /// Source peer ID
    pub from: String,

    /// Target peer ID
    pub to: String,

    /// Type of connection (neighbor, relay, etc.)
    pub connection_type: ConnectionType,

    /// Measured latency in milliseconds (if available)
    pub latency_ms: Option<u64>,

    /// Hex color for visualization (rainbow gradient based on latency)
    /// - #00ff00 = 0ms (green)
    /// - #ffff00 = 50ms (yellow)
    /// - #ff0000 = 100ms+ (red)
    pub color: String,
}

/// Type of connection between peers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    /// 8-neighbor mesh connection
    Neighbor,

    /// Active relay connection
    Relay,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Total number of peers
    pub total_peers: usize,

    /// Number of server nodes
    pub server_nodes: usize,

    /// Number of browser peers
    pub browser_peers: usize,

    /// Total mesh edges (8-neighbor connections)
    pub mesh_edges: usize,

    /// Total relay connections
    pub relay_connections: usize,

    /// Mesh occupancy percentage
    pub occupancy_percent: f64,
}

/// GET /api/v1/map - Get network topology map
///
/// Returns the current state of the hexagonal toroidal mesh with all connected peers
/// and their 8-neighbor connections. Browser peers are anonymized.
/// **FAST**: Uses LOCAL DHT cache (replicated via epidemic gossip) for instant map generation!
pub async fn get_network_map(
    State(state): State<MapState>,
) -> Result<Json<NetworkMap>, StatusCode> {
    // For Content-Addressed Slots, we use the FIXED 256³ address space
    // The mesh dimensions are just for visualization - the actual topology
    // is determined by which slots peers choose to occupy via CAS
    let mesh_config = MeshConfig::new(256, 256, 256);

    // Build peer nodes with slot assignments
    let mut nodes = Vec::new();
    let mut node_slots: HashMap<String, SlotCoordinate> = HashMap::new();

    // LOCAL CACHE STRATEGY (powered by gossip replication):
    // 1. Get all known peers from P2P manager (populated via PeerReferral events)
    // 2. Query LOCAL DHT cache for each peer's slot ownership (replicated via epidemic gossip)
    // This is FAST because all slot ownership is replicated to every node via gossip!

    let all_peer_strings = state.sync.p2p.get_known_peer_strings().unwrap_or_default();
    tracing::info!("Map: Querying {} peers from P2P manager for slot ownership", all_peer_strings.len());

    // Lock DHT storage once for all queries (LOCAL CACHE - replicated via gossip)
    let dht_storage = state.relay.dht_storage.lock().await;

    for peer_id_ref in all_peer_strings {
        let peer_id = peer_id_ref.to_string(); // Convert &str to String

        // Query LOCAL DHT cache for this peer's slot ownership
        // Now that we have gossip replication, all slot ownership is replicated locally!
        let location_key = peer_location_key(&peer_id);

        if let Some(ownership_bytes) = dht_storage.get_raw(&location_key) {
            if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(&ownership_bytes) {
                let slot = ownership.slot;

                tracing::debug!("Map: Found peer {} at slot ({}, {}, {}) via local DHT cache", peer_id, slot.x, slot.y, slot.z);

                node_slots.insert(peer_id.clone(), slot);

                // Determine peer type from relay state tracking
                let relay_peer_type = state.relay.get_peer_type(&peer_id).await;
                let peer_type = match relay_peer_type {
                    crate::routes::relay::PeerType::Browser => PeerType::Browser,
                    crate::routes::relay::PeerType::Server => PeerType::Server,
                };

                // Anonymize browser peer IDs - only expose first 8 chars for privacy
                let (id, label) = if peer_type == PeerType::Browser {
                    // Take only first 8 chars (or less if peer_id is shorter)
                    let truncated = peer_id.chars().take(8).collect::<String>();
                    let anon_id = format!("browser-{}", truncated);
                    (anon_id.clone(), format!("Browser ({})", truncated))
                } else {
                    (peer_id.clone(), peer_id.clone())
                };

                // Calculate permanent SlotId from coordinate
                let slot_id = SlotId::from_coordinate(slot);

                let node = PeerNode {
                    id,
                    label,
                    slot_id: slot_id.to_hex(),
                    slot: slot.into(),
                    peer_type,
                    last_heartbeat: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    capabilities: vec!["webrtc".to_string(), "dht".to_string(), "spore".to_string()],
                    online: true, // All peers with distributed DHT announcements are considered online
                    avg_neighbor_latency_ms: None,
                };

                nodes.push(node);
            }
        }
    }

    tracing::info!("Map: Discovered {} peers from DHT slot ownership announcements", nodes.len());

    // Build edges based on 8-neighbor mesh topology
    let mut edges = Vec::new();
    let mut mesh_edge_count = 0;

    for (peer_id, slot) in &node_slots {
        // Get 8 neighbors for this slot
        let neighbors = get_neighbor_slots(slot, &mesh_config);

        for (_direction, neighbor_slot) in neighbors {
            // Find peer at neighbor slot
            if let Some((neighbor_id, _)) = node_slots.iter().find(|(_, s)| **s == neighbor_slot) {
                // Create edge (only if not duplicate - check if reverse edge exists)
                let from = if peer_id < neighbor_id { peer_id } else { neighbor_id };
                let to = if peer_id < neighbor_id { neighbor_id } else { peer_id };

                // Find the display IDs (anonymized for browser peers)
                let from_node = nodes.iter().find(|n| {
                    if n.peer_type == PeerType::Browser {
                        // Browser IDs are "browser-{first 8 chars}"
                        let from_prefix = from.chars().take(8).collect::<String>();
                        n.id == format!("browser-{}", from_prefix)
                    } else {
                        &n.id == from
                    }
                });

                let to_node = nodes.iter().find(|n| {
                    if n.peer_type == PeerType::Browser {
                        // Browser IDs are "browser-{first 8 chars}"
                        let to_prefix = to.chars().take(8).collect::<String>();
                        n.id == format!("browser-{}", to_prefix)
                    } else {
                        &n.id == to
                    }
                });

                if let (Some(from_node), Some(to_node)) = (from_node, to_node) {
                    // Get average latency between these two nodes (if available)
                    let from_latency = from_node.avg_neighbor_latency_ms;
                    let to_latency = to_node.avg_neighbor_latency_ms;

                    // Use average of both nodes' latencies, or None if neither has data
                    let edge_latency = match (from_latency, to_latency) {
                        (Some(a), Some(b)) => Some((a + b) / 2),
                        (Some(a), None) => Some(a),
                        (None, Some(b)) => Some(b),
                        (None, None) => None,
                    };

                    // Calculate color based on latency (default to green if no data)
                    let color = latency_to_hex(edge_latency.unwrap_or(0));

                    let edge = PeerEdge {
                        from: from_node.id.clone(),
                        to: to_node.id.clone(),
                        connection_type: ConnectionType::Neighbor,
                        latency_ms: edge_latency,
                        color,
                    };

                    // Only add if not already present
                    if !edges.iter().any(|e: &PeerEdge| {
                        (e.from == edge.from && e.to == edge.to) ||
                        (e.from == edge.to && e.to == edge.from)
                    }) {
                        edges.push(edge);
                        mesh_edge_count += 1;
                    }
                }
            }
        }
    }

    // Calculate statistics
    let total_peers = nodes.len();
    let server_nodes = nodes.iter().filter(|n| n.peer_type == PeerType::Server).count();
    let browser_peers = nodes.iter().filter(|n| n.peer_type == PeerType::Browser).count();
    let total_slots = mesh_config.width * mesh_config.height * mesh_config.depth;
    let occupancy_percent = (total_peers as f64 / total_slots as f64) * 100.0;

    let stats = NetworkStats {
        total_peers,
        server_nodes,
        browser_peers,
        mesh_edges: mesh_edge_count,
        relay_connections: 0, // TODO: Track relay connections separately
        occupancy_percent,
    };

    let map = NetworkMap {
        mesh_config: MeshConfigData {
            width: mesh_config.width,
            height: mesh_config.height,
            depth: mesh_config.depth,
            total_slots,
        },
        nodes,
        edges,
        stats,
    };

    Ok(Json(map))
}
