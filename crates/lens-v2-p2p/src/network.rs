//! P2P Network Layer
//!
//! Handles peer-to-peer networking for lens nodes.
//! All nodes are equal peers - no client/server dichotomy.

use crate::{BlockId, P2pError, Result};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;
use tracing::{debug, info, warn, error};
use citadel_core::topology::SlotCoordinate;

/// Peer ID (string identifier assigned by relay)
pub type PeerId = String;

/// Peer type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeerType {
    /// Server node (always ready for P2P)
    Server,
    /// Browser peer (WebRTC connection)
    Browser,
}

/// Peer connection for exchanging blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    pub peer_id: PeerId,
    pub latest_height: u64,
    pub score: f64,
    /// Peer type (server or browser)
    pub peer_type: PeerType,
    /// Slot coordinate in Citadel mesh (CRITICAL for DHT neighbor discovery)
    /// This is the ACTUAL slot the peer announced, not a recalculated one
    pub slot: Option<SlotCoordinate>,
}

/// Block request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRequest {
    pub block_ids: Vec<BlockId>,
}

/// Block response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub blocks: Vec<BlockData>,
}

/// Block data with full content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub id: BlockId,
    pub height: u64,
    pub data: Vec<u8>,
    pub prev: Option<BlockId>,
    pub timestamp: u64,
}

/// DHT replication message from relay (gossip replication of global DHT)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DhtReplication {
    #[serde(with = "serde_bytes")]
    pub key: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub value: Vec<u8>,
    pub timestamp: u64,
    pub source_peer_id: String,
}

/// DHT bootstrap response from relay (complete DHT snapshot for new peers)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DhtBootstrapResponse {
    pub dht_entries: Vec<DhtEntry>,
    pub entry_count: usize,
    pub timestamp: u64,
}

/// DHT entry (need to define here for deserialization)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DhtEntry {
    pub key: [u8; 32],
    pub value: Vec<u8>,
    pub timestamp: u64,
}

/// Network events from peers
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerConnection),

    /// Peer disconnected
    PeerDisconnected(PeerId),

    /// Received block data from peer
    BlockReceived(BlockData),

    /// Peer referral from relay
    PeerReferral(Vec<PeerConnection>),

    /// Assigned peer ID from relay
    PeerIdAssigned(PeerId),

    /// Received WantList from peer (SPORE announcement)
    WantListReceived(PeerId, consensus_peerexc::WantList),

    /// Received block request from peer
    BlockRequestReceived(PeerId, Vec<BlockId>),

    /// DHT replication from relay (gossip replication of GLOBAL DHT)
    DhtReplication(DhtReplication),

    /// DHT bootstrap response from relay (complete DHT snapshot)
    DhtBootstrapResponse(DhtBootstrapResponse),
}

/// P2P Network manager
pub struct P2pNetwork {
    /// Our peer ID (assigned by relay)
    peer_id: Arc<RwLock<Option<PeerId>>>,

    /// Our slot coordinate in the mesh (for DHT bootstrap)
    my_slot: SlotCoordinate,

    /// Connected peers
    peers: Arc<RwLock<HashMap<PeerId, PeerConnection>>>,

    /// Channel for receiving network events
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<NetworkEvent>>>,

    /// Channel for sending network events
    event_tx: mpsc::UnboundedSender<NetworkEvent>,

    /// Channel for sending outgoing WebSocket messages
    ws_tx: Arc<RwLock<Option<mpsc::UnboundedSender<Message>>>>,

    /// Relay WebSocket URL
    relay_url: String,
}

impl P2pNetwork {
    /// Create a new P2P network instance
    /// peer_id: Our peer ID to announce to the relay (ensures consistency across mesh)
    /// my_slot: Our slot coordinate for DHT bootstrap requests
    pub fn new(relay_url: String, peer_id: String, my_slot: SlotCoordinate) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            peer_id: Arc::new(RwLock::new(Some(peer_id))),
            my_slot,
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_rx: Arc::new(RwLock::new(event_rx)),
            event_tx,
            ws_tx: Arc::new(RwLock::new(None)),
            relay_url,
        }
    }

    /// Get our peer ID
    pub async fn peer_id(&self) -> Option<PeerId> {
        self.peer_id.read().await.clone()
    }

    /// Get connected peers
    pub async fn peers(&self) -> Vec<PeerConnection> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Start the network (connect to relay)
    pub async fn start(&self) -> Result<()> {
        info!("Connecting to relay at {}", self.relay_url);

        let (ws_stream, _) = connect_async(&self.relay_url)
            .await
            .map_err(|e| P2pError::Network(format!("Failed to connect to relay: {}", e)))?;

        info!("Connected to relay");

        let (mut write, mut read) = ws_stream.split();

        // Create channel for outgoing messages
        let (ws_tx, mut ws_rx) = mpsc::unbounded_channel::<Message>();
        *self.ws_tx.write().await = Some(ws_tx.clone());

        // Send hello message with our peer_id to relay
        // This ensures peer_id consistency across the mesh
        if let Some(ref our_peer_id) = *self.peer_id.read().await {
            let hello_msg = serde_json::json!({
                "type": "hello",
                "peer_id": our_peer_id,
            });
            if let Ok(hello_json) = serde_json::to_string(&hello_msg) {
                if let Err(e) = ws_tx.send(Message::Text(hello_json)) {
                    error!("Failed to send hello message: {}", e);
                }  else {
                    info!("Sent hello message to relay with peer_id: {}", our_peer_id);
                }
            }

            // Send DHT bootstrap request to get complete DHT snapshot
            // This populates our local DHT cache from the GLOBAL DHT stored at the relay
            let bootstrap_request = serde_json::json!({
                "type": "dht_bootstrap_request",
                "peer_id": our_peer_id,
                "slot": self.my_slot,
            });
            if let Ok(bootstrap_json) = serde_json::to_string(&bootstrap_request) {
                if let Err(e) = ws_tx.send(Message::Text(bootstrap_json)) {
                    error!("Failed to send DHT bootstrap request: {}", e);
                } else {
                    info!("🔄 Sent DHT bootstrap request to relay for slot {:?}", self.my_slot);
                }
            }
        }

        // Spawn task to handle outgoing messages
        tokio::spawn(async move {
            while let Some(msg) = ws_rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });

        let event_tx = self.event_tx.clone();
        let peer_id = self.peer_id.clone();
        let peers = self.peers.clone();

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        debug!("Received message: {} bytes", text.len());

                        // Try to parse as peer referral
                        if let Ok(referral) = serde_json::from_str::<serde_json::Value>(&text) {
                            if referral["type"] == "peer_referral" {
                                // We got our peer ID!
                                if let Some(our_id) = referral["your_peer_id"].as_str() {
                                    info!("Assigned peer ID: {}", our_id);
                                    *peer_id.write().await = Some(our_id.to_string());
                                    let _ = event_tx.send(NetworkEvent::PeerIdAssigned(our_id.to_string()));
                                }

                                // Parse peer list
                                if let Some(peer_list) = referral["peers"].as_array() {
                                    let mut connections = Vec::new();

                                    for peer in peer_list {
                                        if let (Some(peer_id), Some(height), Some(score)) = (
                                            peer["peer_id"].as_str(),
                                            peer["latest_height"].as_u64(),
                                            peer["score"].as_f64(),
                                        ) {
                                            // Determine peer type (default to Server if not specified)
                                            let peer_type = if peer_id.starts_with("browser-") || peer_id.contains("webrtc") {
                                                PeerType::Browser
                                            } else {
                                                PeerType::Server
                                            };

                                            // Extract slot if present (CRITICAL for DHT neighbor discovery)
                                            let slot = peer.get("slot")
                                                .and_then(|s| serde_json::from_value::<SlotCoordinate>(s.clone()).ok());

                                            let conn = PeerConnection {
                                                peer_id: peer_id.to_string(),
                                                latest_height: height,
                                                score,
                                                peer_type,
                                                slot,
                                            };

                                            // Store peer
                                            peers.write().await.insert(peer_id.to_string(), conn.clone());
                                            connections.push(conn);
                                        }
                                    }

                                    if !connections.is_empty() {
                                        info!("Received {} peer referrals", connections.len());
                                        let _ = event_tx.send(NetworkEvent::PeerReferral(connections));
                                    }
                                }
                            }
                            // Try to parse as wantlist_announcement
                            else if referral["type"] == "wantlist_announcement" {
                                if let (Some(from_peer_id), Some(wantlist_json)) = (
                                    referral["from_peer_id"].as_str(),
                                    referral.get("wantlist"),
                                ) {
                                    if let Ok(wantlist) = serde_json::from_value::<consensus_peerexc::WantList>(wantlist_json.clone()) {
                                        info!("Received WantList announcement from {}: gen={}, have={}, need={}",
                                            from_peer_id, wantlist.generation, wantlist.have_blocks.len(), wantlist.need_blocks.len());
                                        let _ = event_tx.send(NetworkEvent::WantListReceived(from_peer_id.to_string(), wantlist));
                                    }
                                }
                            }
                            // Try to parse as block_request
                            else if referral["type"] == "block_request" {
                                if let (Some(from_peer_id), Some(block_ids)) = (
                                    referral["from_peer_id"].as_str(),
                                    referral["block_ids"].as_array(),
                                ) {
                                    let block_ids: Vec<String> = block_ids
                                        .iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect();
                                    info!("Received block request from {} for {} blocks", from_peer_id, block_ids.len());
                                    let _ = event_tx.send(NetworkEvent::BlockRequestReceived(from_peer_id.to_string(), block_ids));
                                }
                            }
                            // Try to parse as block_response
                            else if referral["type"] == "block_response" {
                                if let Some(blocks_json) = referral["blocks"].as_array() {
                                    for block_json in blocks_json {
                                        if let Ok(block) = serde_json::from_value::<BlockData>(block_json.clone()) {
                                            info!("Received block: {} at height {}", block.id, block.height);
                                            let _ = event_tx.send(NetworkEvent::BlockReceived(block));
                                        }
                                    }
                                }
                            }
                            // Try to parse as dht_replication (GLOBAL DHT gossip)
                            else if referral["type"] == "dht_replication" {
                                if let Ok(replication) = serde_json::from_value::<DhtReplication>(referral.clone()) {
                                    debug!("Received DHT replication: key={} bytes from {}",
                                        replication.key.len(), replication.source_peer_id);
                                    let _ = event_tx.send(NetworkEvent::DhtReplication(replication));
                                }
                            }
                            // Try to parse as dht_bootstrap_response (complete DHT snapshot)
                            else if referral["type"] == "dht_bootstrap_response" {
                                if let Some(response_json) = referral.get("response") {
                                    if let Ok(response) = serde_json::from_value::<DhtBootstrapResponse>(response_json.clone()) {
                                        info!("Received DHT bootstrap response: {} entries",
                                            response.entry_count);
                                        let _ = event_tx.send(NetworkEvent::DhtBootstrapResponse(response));
                                    }
                                }
                            }
                        }

                        // Try to parse as block response
                        if let Ok(block_resp) = serde_json::from_str::<BlockResponse>(&text) {
                            info!("Received {} blocks", block_resp.blocks.len());
                            for block in block_resp.blocks {
                                let _ = event_tx.send(NetworkEvent::BlockReceived(block));
                            }
                        }
                    }
                    Ok(Message::Binary(_)) => {
                        debug!("Received binary message");
                    }
                    Ok(Message::Close(_)) => {
                        warn!("Relay closed connection");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Send heartbeat to relay (will be broadcast to all peers)
    pub async fn send_heartbeat(&self, peer_id: &str, slot: citadel_core::topology::SlotCoordinate) -> Result<()> {
        let heartbeat = serde_json::json!({
            "type": "heartbeat",
            "peer_id": peer_id,
            "slot": slot,
        });

        let json = serde_json::to_string(&heartbeat)
            .map_err(|e| P2pError::Network(format!("Failed to serialize heartbeat: {}", e)))?;

        // Send via WebSocket
        if let Some(tx) = self.ws_tx.read().await.as_ref() {
            tx.send(Message::Text(json))
                .map_err(|e| P2pError::Network(format!("Failed to send heartbeat: {}", e)))?;
            debug!("💓 Sent heartbeat for {}", peer_id);
        }

        Ok(())
    }

    /// Send WantList to relay for peer discovery
    #[cfg(feature = "consensus")]
    pub async fn send_wantlist(&self, wantlist: &consensus_peerexc::WantList) -> Result<()> {
        info!("Sending WantList: gen={}, needs={}, offers={}",
            wantlist.generation, wantlist.has_needs(), wantlist.has_offers());

        // Serialize WantList to JSON
        let json = serde_json::to_string(wantlist)
            .map_err(|e| P2pError::Network(format!("Failed to serialize WantList: {}", e)))?;

        // Send via WebSocket
        if let Some(tx) = self.ws_tx.read().await.as_ref() {
            tx.send(Message::Text(json))
                .map_err(|e| P2pError::Network(format!("Failed to send WantList: {}", e)))?;
            info!("Sent WantList to relay");
        } else {
            warn!("Cannot send WantList: not connected to relay");
        }

        Ok(())
    }

    /// Request blocks from a peer
    pub async fn request_blocks(&self, peer_id: &PeerId, block_ids: Vec<BlockId>) -> Result<()> {
        info!("Requesting {} blocks from peer {}", block_ids.len(), peer_id);

        // Get our peer ID
        let our_peer_id = self.peer_id.read().await.clone()
            .ok_or_else(|| P2pError::Network("Not connected to relay".to_string()))?;

        // Create block request message
        let request = serde_json::json!({
            "type": "block_request",
            "from_peer_id": our_peer_id,
            "to_peer_id": peer_id,
            "block_ids": block_ids,
        });

        // Send via WebSocket
        if let Some(tx) = self.ws_tx.read().await.as_ref() {
            let json = serde_json::to_string(&request)
                .map_err(|e| P2pError::Network(format!("Failed to serialize block request: {}", e)))?;
            tx.send(Message::Text(json))
                .map_err(|e| P2pError::Network(format!("Failed to send block request: {}", e)))?;
            info!("Sent block request to {}", peer_id);
        } else {
            warn!("Cannot send block request: not connected to relay");
        }

        Ok(())
    }

    /// Send block response to a peer
    pub async fn send_blocks(&self, peer_id: &PeerId, blocks: Vec<BlockData>) -> Result<()> {
        info!("Sending {} blocks to peer {}", blocks.len(), peer_id);

        // Get our peer ID
        let our_peer_id = self.peer_id.read().await.clone()
            .ok_or_else(|| P2pError::Network("Not connected to relay".to_string()))?;

        // Create block response message
        let response = serde_json::json!({
            "type": "block_response",
            "from_peer_id": our_peer_id,
            "to_peer_id": peer_id,
            "blocks": blocks,
        });

        // Send via WebSocket
        if let Some(tx) = self.ws_tx.read().await.as_ref() {
            let json = serde_json::to_string(&response)
                .map_err(|e| P2pError::Network(format!("Failed to serialize block response: {}", e)))?;
            tx.send(Message::Text(json))
                .map_err(|e| P2pError::Network(format!("Failed to send block response: {}", e)))?;
            info!("Sent {} blocks to {}", blocks.len(), peer_id);
        } else {
            warn!("Cannot send block response: not connected to relay");
        }

        Ok(())
    }

    /// Receive next network event (blocking)
    pub async fn next_event(&self) -> Option<NetworkEvent> {
        self.event_rx.write().await.recv().await
    }

    /// Try to receive next network event without blocking
    pub async fn try_next_event(&self) -> Option<NetworkEvent> {
        self.event_rx.write().await.try_recv().ok()
    }

    /// Broadcast heartbeat to ALL peers in the mesh (not just relay)
    pub async fn broadcast_heartbeat(&self, peer_id: &str, slot: citadel_core::topology::SlotCoordinate) -> Result<()> {
        let heartbeat = serde_json::json!({
            "type": "heartbeat",
            "peer_id": peer_id,
            "slot": slot,
        });

        let json = serde_json::to_string(&heartbeat)
            .map_err(|e| P2pError::Network(format!("Failed to serialize heartbeat: {}", e)))?;

        // Send through relay WebSocket - relay will broadcast to ALL peers
        if let Some(tx) = self.ws_tx.read().await.as_ref() {
            tx.send(Message::Text(json))
                .map_err(|e| P2pError::Network(format!("Failed to broadcast heartbeat: {}", e)))?;
            debug!("💓 Broadcast heartbeat for {} to entire mesh", peer_id);
        }

        Ok(())
    }

    /// Get list of all currently known peer IDs from peer referrals
    pub async fn get_peer_ids(&self) -> Vec<String> {
        self.peers.read().await.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let network = P2pNetwork::new("ws://localhost:5002/api/v1/relay/ws".to_string(), "peer-test-123".to_string());
        assert_eq!(network.relay_url, "ws://localhost:5002/api/v1/relay/ws");
    }

    #[tokio::test]
    async fn test_peer_id_set_from_constructor() {
        let network = P2pNetwork::new("ws://localhost:5002/api/v1/relay/ws".to_string(), "peer-test-456".to_string());
        assert_eq!(network.peer_id().await, Some("peer-test-456".to_string()));
    }

    #[tokio::test]
    async fn test_peers_initially_empty() {
        let network = P2pNetwork::new("ws://localhost:5002/api/v1/relay/ws".to_string(), "peer-test-789".to_string());
        assert_eq!(network.peers().await.len(), 0);
    }
}
