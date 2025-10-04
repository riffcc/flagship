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
use tracing::{info, warn};

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

/// Relay state shared across WebSocket connections
#[derive(Clone)]
pub struct RelayState {
    pub relay: Arc<RwLock<RelayServer>>,
    pub peer_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>,
}

impl RelayState {
    pub fn new() -> Self {
        Self {
            relay: Arc::new(RwLock::new(RelayServer::new())),
            peer_senders: Arc::new(RwLock::new(HashMap::new())),
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
    let peer_id = format!("peer-{}", rand::random::<u32>());

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

                // Try to parse as SignalingMessage first
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
                // Try to parse as WantList
                else if let Ok(wantlist) = serde_json::from_str::<WantList>(&text) {
                    info!("Relay: Received WantList from {}: gen={}, needs={}, offers={}",
                        peer_id, wantlist.generation, wantlist.has_needs(), wantlist.has_offers());

                    // Index the WantList
                    {
                        let mut relay = state.relay.write().await;
                        relay.index_wantlist(peer_id.clone(), &wantlist);
                    }

                    // Find providers for this peer's needs
                    let providers = {
                        let relay = state.relay.read().await;
                        relay.find_providers(&wantlist)
                    };

                    info!("Relay: Found {} providers for {}", providers.len(), peer_id);

                    // Send peer referrals back with peer IDs
                    if !providers.is_empty() {
                        let referral = serde_json::json!({
                            "type": "peer_referral",
                            "your_peer_id": peer_id,
                            "peers": providers.into_iter().take(5).map(|p| {
                                serde_json::json!({
                                    "peer_id": p.peer_id,
                                    "latest_height": p.latest_height,
                                    "score": p.score,
                                })
                            }).collect::<Vec<_>>(),
                        });

                        let senders = state.peer_senders.read().await;
                        if let Some(tx) = senders.get(&peer_id) {
                            if let Ok(json) = serde_json::to_string(&referral) {
                                if let Err(e) = tx.send(Message::Text(json)) {
                                    warn!("Relay: Failed to send referral to {}: {}", peer_id, e);
                                }
                            }
                        }
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                info!("Relay: Received binary from {}: {} bytes", peer_id, data.len());
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

    info!("Relay: Peer {} disconnected", peer_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_state_creation() {
        let state = RelayState::new();
        assert!(Arc::strong_count(&state.relay) >= 1);
    }
}

// Re-export rand for peer IDs
use rand;
