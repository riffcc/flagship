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
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Relay state shared across WebSocket connections
#[derive(Clone)]
pub struct RelayState {
    pub relay: Arc<RwLock<RelayServer>>,
}

impl RelayState {
    pub fn new() -> Self {
        Self {
            relay: Arc::new(RwLock::new(RelayServer::new())),
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

    // Register peer with relay
    let peer_info = PeerInfo::new(peer_id.clone());
    {
        let mut relay = state.relay.write().await;
        if let Err(e) = relay.register_peer(peer_info.clone()) {
            warn!("Relay: Failed to register peer {}: {}", peer_id, e);
            return;
        }
    }

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Relay: Received text from {}: {} bytes", peer_id, text.len());

                // Try to parse as WantList
                if let Ok(wantlist) = serde_json::from_str::<WantList>(&text) {
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

                    // Send peer referrals back
                    if !providers.is_empty() {
                        let referral = serde_json::json!({
                            "type": "peer_referral",
                            "peers": providers.into_iter().take(5).map(|p| {
                                serde_json::json!({
                                    "peer_id": p.peer_id,
                                    "latest_height": p.latest_height,
                                    "score": p.score,
                                })
                            }).collect::<Vec<_>>(),
                        });

                        if let Ok(json) = serde_json::to_string(&referral) {
                            if let Err(e) = sender.send(Message::Text(json)).await {
                                warn!("Relay: Failed to send referral to {}: {}", peer_id, e);
                                break;
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
