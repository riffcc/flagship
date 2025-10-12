//! WebRTC Manager for Browser Peer Connections
//!
//! Allows Rust nodes to accept WebRTC DataChannel connections from browsers,
//! enabling direct browser-to-node P2P communication alongside WebSocket relay.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, error};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::ubts::{UBTSBlock};

/// WebRTC peer connection for a browser
pub struct WebRTCPeer {
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Option<Arc<RTCDataChannel>>,
    peer_id: String,
}

/// WebRTC Manager - handles browser WebRTC connections
pub struct WebRTCManager {
    /// Map of browser peer IDs to WebRTC connections
    peers: Arc<RwLock<HashMap<String, WebRTCPeer>>>,

    /// Channel for receiving UBTS blocks from browsers
    block_rx: Arc<RwLock<mpsc::UnboundedReceiver<(String, UBTSBlock)>>>,

    /// Channel for sending UBTS blocks from browsers
    block_tx: mpsc::UnboundedSender<(String, UBTSBlock)>,

    /// WebRTC API
    api: Arc<webrtc::api::API>,
}

impl WebRTCManager {
    /// Create a new WebRTC manager
    pub fn new() -> Result<Self> {
        // Create media engine
        let mut media_engine = MediaEngine::default();

        // Create interceptor registry
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)?;

        // Create WebRTC API
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        let (block_tx, block_rx) = mpsc::unbounded_channel();

        Ok(Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            block_rx: Arc::new(RwLock::new(block_rx)),
            block_tx,
            api: Arc::new(api),
        })
    }

    /// Create a new WebRTC peer connection for a browser
    pub async fn create_peer_connection(&self, peer_id: String) -> Result<Arc<RTCPeerConnection>> {
        info!("🔗 Creating WebRTC peer connection for browser: {}", peer_id);

        // Configure ICE servers (STUN/TURN)
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        // Create peer connection
        let peer_connection = Arc::new(self.api.new_peer_connection(config).await?);

        // Set up event handlers
        let peer_id_clone = peer_id.clone();
        peer_connection.on_ice_connection_state_change(Box::new(move |state: RTCIceConnectionState| {
            info!("🧊 ICE connection state changed for {}: {:?}", peer_id_clone, state);
            Box::pin(async {})
        }));

        let peer_id_clone = peer_id.clone();
        peer_connection.on_peer_connection_state_change(Box::new(move |state: RTCPeerConnectionState| {
            info!("🔗 Peer connection state changed for {}: {:?}", peer_id_clone, state);
            Box::pin(async {})
        }));

        // Handle incoming DataChannel from browser
        let block_tx = self.block_tx.clone();
        let peer_id_clone = peer_id.clone();
        peer_connection.on_data_channel(Box::new(move |data_channel: Arc<RTCDataChannel>| {
            let block_tx = block_tx.clone();
            let peer_id = peer_id_clone.clone();

            Box::pin(async move {
                info!("📦 DataChannel opened from browser {}: {}", peer_id, data_channel.label());

                let peer_id_clone = peer_id.clone();
                data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
                    let peer_id = peer_id_clone.clone();
                    let block_tx = block_tx.clone();

                    Box::pin(async move {
                        info!("📨 Received message from browser {} via DataChannel", peer_id);

                        // Parse UBTS block
                        match serde_json::from_slice::<UBTSBlock>(&msg.data) {
                            Ok(block) => {
                                info!("✅ Parsed UBTS block from browser {}", peer_id);
                                if let Err(e) = block_tx.send((peer_id.clone(), block)) {
                                    error!("Failed to forward block from browser {}: {}", peer_id, e);
                                }
                            }
                            Err(e) => {
                                error!("❌ Failed to parse UBTS block from browser {}: {}", peer_id, e);
                            }
                        }
                    })
                }));
            })
        }));

        // Store peer connection
        let webrtc_peer = WebRTCPeer {
            peer_connection: peer_connection.clone(),
            data_channel: None,
            peer_id: peer_id.clone(),
        };

        self.peers.write().await.insert(peer_id.clone(), webrtc_peer);

        info!("✅ WebRTC peer connection created for browser {}", peer_id);
        Ok(peer_connection)
    }

    /// Handle WebRTC signaling (SDP offer from browser)
    pub async fn handle_offer(&self, peer_id: String, sdp: String) -> Result<String> {
        info!("📨 Handling SDP offer from browser {}", peer_id);

        // Get or create peer connection
        let peer_connection = {
            let peers = self.peers.read().await;
            if let Some(peer) = peers.get(&peer_id) {
                peer.peer_connection.clone()
            } else {
                drop(peers);
                self.create_peer_connection(peer_id.clone()).await?
            }
        };

        // Set remote description (browser's offer)
        let offer = RTCSessionDescription::offer(sdp)?;
        peer_connection.set_remote_description(offer).await?;

        // Create answer
        let answer = peer_connection.create_answer(None).await?;
        let answer_sdp = answer.sdp.clone();

        // Set local description
        peer_connection.set_local_description(answer).await?;

        info!("✅ Created SDP answer for browser {}", peer_id);
        Ok(answer_sdp)
    }

    /// Handle ICE candidate from browser
    pub async fn handle_ice_candidate(
        &self,
        peer_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    ) -> Result<()> {
        info!("🧊 Handling ICE candidate from browser {}", peer_id);

        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(&peer_id) {
            // Add ICE candidate
            let ice_candidate = serde_json::json!({
                "candidate": candidate,
                "sdpMid": sdp_mid,
                "sdpMLineIndex": sdp_mline_index,
            });

            peer.peer_connection
                .add_ice_candidate(serde_json::from_value(ice_candidate)?)
                .await?;

            info!("✅ Added ICE candidate for browser {}", peer_id);
        } else {
            warn!("⚠️  No peer connection found for browser {}", peer_id);
        }

        Ok(())
    }

    /// Broadcast UBTS block to all connected browser peers via WebRTC
    pub async fn broadcast_block(&self, block: &UBTSBlock) -> Result<usize> {
        let block_json = serde_json::to_vec(block)?;
        let peers = self.peers.read().await;
        let mut sent_count = 0;

        for (peer_id, peer) in peers.iter() {
            if let Some(ref dc) = peer.data_channel {
                let bytes = axum::body::Bytes::from(block_json.clone());
                match dc.send(&bytes).await {
                    Ok(_) => {
                        info!("✅ Sent UBTS block to browser {} via WebRTC", peer_id);
                        sent_count += 1;
                    }
                    Err(e) => {
                        error!("❌ Failed to send to browser {}: {}", peer_id, e);
                    }
                }
            }
        }

        Ok(sent_count)
    }

    /// Get next UBTS block from browsers
    pub async fn next_block(&self) -> Option<(String, UBTSBlock)> {
        self.block_rx.write().await.recv().await
    }

    /// Remove disconnected peer
    pub async fn remove_peer(&self, peer_id: &str) {
        if let Some(peer) = self.peers.write().await.remove(peer_id) {
            let _ = peer.peer_connection.close().await;
            info!("🔌 Removed WebRTC peer: {}", peer_id);
        }
    }
}
