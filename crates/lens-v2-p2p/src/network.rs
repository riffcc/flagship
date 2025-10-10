//! P2P Network Layer
//!
//! Handles peer-to-peer networking for lens nodes.
//! All nodes are equal peers - no client/server dichotomy.

use crate::{BlockId, BlockMeta, P2pError, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info, warn, error};

/// Peer ID (string identifier assigned by relay)
pub type PeerId = String;

/// Peer connection for exchanging blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    pub peer_id: PeerId,
    pub latest_height: u64,
    pub score: f64,
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
}

/// P2P Network manager
pub struct P2pNetwork {
    /// Our peer ID (assigned by relay)
    peer_id: Arc<RwLock<Option<PeerId>>>,

    /// Connected peers
    peers: Arc<RwLock<HashMap<PeerId, PeerConnection>>>,

    /// Channel for receiving network events
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<NetworkEvent>>>,

    /// Channel for sending network events
    event_tx: mpsc::UnboundedSender<NetworkEvent>,

    /// Relay WebSocket URL
    relay_url: String,
}

impl P2pNetwork {
    /// Create a new P2P network instance
    pub fn new(relay_url: String) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            peer_id: Arc::new(RwLock::new(None)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_rx: Arc::new(RwLock::new(event_rx)),
            event_tx,
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
                                            let conn = PeerConnection {
                                                peer_id: peer_id.to_string(),
                                                latest_height: height,
                                                score,
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

    /// Send WantList to relay for peer discovery
    #[cfg(feature = "consensus")]
    pub async fn send_wantlist(&self, wantlist: &consensus_peerexc::WantList) -> Result<()> {
        info!("Sending WantList: gen={}, needs={}, offers={}",
            wantlist.generation, wantlist.has_needs(), wantlist.has_offers());

        // TODO: Send via WebSocket
        // For now, just log

        Ok(())
    }

    /// Request blocks from a peer
    pub async fn request_blocks(&self, peer_id: &PeerId, block_ids: Vec<BlockId>) -> Result<()> {
        info!("Requesting {} blocks from peer {}", block_ids.len(), peer_id);

        // TODO: Send block request to peer
        // For now, just log

        Ok(())
    }

    /// Receive next network event
    pub async fn next_event(&self) -> Option<NetworkEvent> {
        self.event_rx.write().await.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let network = P2pNetwork::new("ws://localhost:5002/api/v1/relay/ws".to_string());
        assert_eq!(network.relay_url, "ws://localhost:5002/api/v1/relay/ws");
    }

    #[tokio::test]
    async fn test_peer_id_initially_none() {
        let network = P2pNetwork::new("ws://localhost:5002/api/v1/relay/ws".to_string());
        assert!(network.peer_id().await.is_none());
    }

    #[tokio::test]
    async fn test_peers_initially_empty() {
        let network = P2pNetwork::new("ws://localhost:5002/api/v1/relay/ws".to_string());
        assert_eq!(network.peers().await.len(), 0);
    }
}
