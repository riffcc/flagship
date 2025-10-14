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
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
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
use crate::p2p_heartbeat::Heartbeat;

/// Type of WebRTC peer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerType {
    /// Browser peer (ephemeral, connects to nodes)
    Browser,
    /// Server node peer (establishes direct P2P connections)
    Node,
}

/// WebRTC peer connection for a browser or node
pub struct WebRTCPeer {
    pub peer_connection: Arc<RTCPeerConnection>,
    pub data_channel: Option<Arc<RTCDataChannel>>,
    pub peer_id: String,
    pub peer_type: PeerType,
}

/// WebRTC Manager - handles WebRTC connections for both browsers and nodes
pub struct WebRTCManager {
    /// Map of peer IDs to WebRTC connections (both browser and node peers)
    pub peers: Arc<RwLock<HashMap<String, WebRTCPeer>>>,

    /// Channel for receiving UBTS blocks from browsers
    block_rx: Arc<RwLock<mpsc::UnboundedReceiver<(String, UBTSBlock)>>>,

    /// Channel for sending UBTS blocks from browsers
    block_tx: mpsc::UnboundedSender<(String, UBTSBlock)>,

    /// Channel for receiving heartbeat messages from peers
    heartbeat_rx: Arc<RwLock<mpsc::UnboundedReceiver<Heartbeat>>>,

    /// Channel for sending heartbeat messages
    heartbeat_tx: mpsc::UnboundedSender<Heartbeat>,

    /// Channel for receiving TGP packets (DHT GET/PUT/RESPONSE) from peers via WebRTC
    tgp_rx: Arc<RwLock<mpsc::UnboundedReceiver<(String, Vec<u8>)>>>,

    /// Channel for sending TGP packets
    tgp_tx: mpsc::UnboundedSender<(String, Vec<u8>)>,

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
        let (heartbeat_tx, heartbeat_rx) = mpsc::unbounded_channel();
        let (tgp_tx, tgp_rx) = mpsc::unbounded_channel();

        Ok(Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            block_rx: Arc::new(RwLock::new(block_rx)),
            block_tx,
            heartbeat_rx: Arc::new(RwLock::new(heartbeat_rx)),
            heartbeat_tx,
            tgp_rx: Arc::new(RwLock::new(tgp_rx)),
            tgp_tx,
            api: Arc::new(api),
        })
    }

    /// Create a new WebRTC peer connection for a browser or node
    pub async fn create_peer_connection(&self, peer_id: String, peer_type: PeerType) -> Result<Arc<RTCPeerConnection>> {
        let peer_type_str = match peer_type {
            PeerType::Browser => "browser",
            PeerType::Node => "node",
        };
        info!("🔗 Creating WebRTC peer connection for {}: {}", peer_type_str, peer_id);

        // Configure ICE servers (STUN/TURN)
        // For localhost/test environments, we don't need STUN/TURN - local candidates work fine
        // For production, add STUN/TURN servers for NAT traversal
        let config = RTCConfiguration {
            ice_servers: vec![
                // Local loopback for testing (no STUN needed)
                RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                },
            ],
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

        // Handle incoming DataChannel from browser or node
        let block_tx = self.block_tx.clone();
        let heartbeat_tx = self.heartbeat_tx.clone();
        let tgp_tx = self.tgp_tx.clone();
        let peer_id_clone = peer_id.clone();
        let peers_clone = self.peers.clone();
        peer_connection.on_data_channel(Box::new(move |data_channel: Arc<RTCDataChannel>| {
            let block_tx = block_tx.clone();
            let heartbeat_tx = heartbeat_tx.clone();
            let tgp_tx = tgp_tx.clone();
            let peer_id = peer_id_clone.clone();
            let peers = peers_clone.clone();

            Box::pin(async move {
                let peer_type_str = match peer_type {
                    PeerType::Browser => "browser",
                    PeerType::Node => "node",
                };
                info!("📦 DataChannel opened from {} {}: {}", peer_type_str, peer_id, data_channel.label());

                // Set up on_open handler first
                let peer_id_for_open = peer_id.clone();
                data_channel.on_open(Box::new(move || {
                    info!("🟢 DataChannel OPENED for peer {}", peer_id_for_open);
                    Box::pin(async {})
                }));

                // Store the DataChannel in the peer's data_channel field
                {
                    let mut peers_guard = peers.write().await;
                    if let Some(peer) = peers_guard.get_mut(&peer_id) {
                        peer.data_channel = Some(data_channel.clone());
                        info!("✅ Stored DataChannel for peer {}", peer_id);
                    }
                }

                let peer_id_clone = peer_id.clone();
                data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
                    let peer_id = peer_id_clone.clone();
                    let block_tx = block_tx.clone();
                    let heartbeat_tx = heartbeat_tx.clone();
                    let tgp_tx = tgp_tx.clone();

                    Box::pin(async move {
                        println!("📨 Received message from peer {} via DataChannel ({} bytes)", peer_id, msg.data.len());
                        info!("📨 Received message from peer {} via DataChannel ({} bytes)", peer_id, msg.data.len());

                        // Check if this is a TGP binary packet (DHT GET/PUT/RESPONSE)
                        // TGP packets start with version byte (0x01) followed by packet type
                        // Packet types: 0x01-0x08 are valid TGP packets
                        if msg.data.len() >= 21 && msg.data[0] == 0x01 && msg.data[1] >= 0x01 && msg.data[1] <= 0x08 {
                            info!("🔀 Received TGP packet from peer {} via WebRTC DataChannel", peer_id);
                            // Forward to TGP channel for relay processing
                            if let Err(e) = tgp_tx.send((peer_id.clone(), msg.data.to_vec())) {
                                error!("Failed to forward TGP packet from peer {}: {}", peer_id, e);
                            } else {
                                info!("✅ Forwarded TGP packet to relay handler");
                            }
                            return;
                        }

                        // Check if this is a text message (UTF-8 encoded JSON)
                        // Text messages include: heartbeats, gossip messages, UBTS blocks
                        if let Ok(text) = std::str::from_utf8(&msg.data) {
                            // Try to parse as JSON to determine message type
                            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(text) {
                                // Check for slot ownership gossip
                                if let Some("slot_ownership_gossip") = json_value.get("type").and_then(|v| v.as_str()) {
                                    info!("📢 Received slot ownership gossip from peer {} via WebRTC", peer_id);
                                    // Forward as text message to TGP channel with special marker
                                    // We'll use peer_id prefixed with "gossip:" to indicate this is gossip, not TGP
                                    // Actually, let's create a separate channel for gossip or handle it here
                                    // For now, we'll just log it - the actual gossip handling should be in the TGP task
                                    // Let's forward the text as bytes with a special marker
                                    // Wait - TGP channel expects binary packets. We need a different approach.
                                    // Let's just forward ALL text messages as-is and handle them in the TGP task
                                    if let Err(e) = tgp_tx.send((format!("TEXT:{}", peer_id), msg.data.to_vec())) {
                                        error!("Failed to forward text message from peer {}: {}", peer_id, e);
                                    }
                                    return;
                                }
                            }

                            // Try to parse as heartbeat
                            if let Ok(heartbeat) = serde_json::from_str::<Heartbeat>(text) {
                                info!("💓 Received heartbeat from peer {}", peer_id);
                                if let Err(e) = heartbeat_tx.send(heartbeat) {
                                    error!("Failed to forward heartbeat from peer {}: {}", peer_id, e);
                                }
                                return;
                            }

                            // Try to parse as UBTS block
                            if let Ok(block) = serde_json::from_str::<UBTSBlock>(text) {
                                info!("✅ Parsed UBTS block from peer {}", peer_id);
                                if let Err(e) = block_tx.send((peer_id.clone(), block)) {
                                    error!("Failed to forward block from peer {}: {}", peer_id, e);
                                }
                                return;
                            }
                        }

                        error!("❌ Failed to parse message from peer {} (not TGP, heartbeat, gossip, or UBTS block)", peer_id);
                    })
                }));
            })
        }));

        // Store peer connection
        let webrtc_peer = WebRTCPeer {
            peer_connection: peer_connection.clone(),
            data_channel: None,
            peer_id: peer_id.clone(),
            peer_type,
        };

        self.peers.write().await.insert(peer_id.clone(), webrtc_peer);

        let peer_type_str = match peer_type {
            PeerType::Browser => "browser",
            PeerType::Node => "node",
        };
        info!("✅ WebRTC peer connection created for {} {}", peer_type_str, peer_id);
        Ok(peer_connection)
    }

    /// Handle WebRTC signaling (SDP offer from browser or node)
    pub async fn handle_offer(&self, peer_id: String, sdp: String, peer_type: PeerType) -> Result<String> {
        let peer_type_str = match peer_type {
            PeerType::Browser => "browser",
            PeerType::Node => "node",
        };
        info!("📨 Handling SDP offer from {} {}", peer_type_str, peer_id);

        // Get or create peer connection
        let peer_connection = {
            let peers = self.peers.read().await;
            if let Some(peer) = peers.get(&peer_id) {
                peer.peer_connection.clone()
            } else {
                drop(peers);
                self.create_peer_connection(peer_id.clone(), peer_type).await?
            }
        };

        // Set remote description (browser's offer)
        let offer = RTCSessionDescription::offer(sdp)?;
        peer_connection.set_remote_description(offer).await?;

        // Create answer
        let answer = peer_connection.create_answer(None).await?;
        let answer_sdp = answer.sdp.clone();

        // Get the gathering complete promise BEFORE setting local description
        let mut gathering_complete = peer_connection.gathering_complete_promise().await;

        // Set local description (this starts ICE gathering)
        peer_connection.set_local_description(answer).await?;

        // Wait for ICE gathering to complete
        info!("⏳ Waiting for ICE gathering to complete for answerer {}", peer_id);
        let _ = gathering_complete.recv().await;
        info!("✅ ICE gathering complete for answerer {}", peer_id);

        // Get the final answer SDP with all ICE candidates
        let final_answer = peer_connection.local_description().await;
        let final_sdp = if let Some(desc) = final_answer {
            desc.sdp.clone()
        } else {
            answer_sdp
        };

        info!("✅ Created SDP answer for {} {} with ICE candidates", peer_type_str, peer_id);
        Ok(final_sdp)
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

    /// Send heartbeat message to specific peer via WebRTC data channel
    pub async fn send_heartbeat_to_peer(&self, peer_id: &str, heartbeat_json: String) -> Result<()> {
        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(peer_id) {
            if let Some(dc) = &peer.data_channel {
                dc.send_text(heartbeat_json).await?;
            }
        }
        Ok(())
    }

    /// Broadcast heartbeat to ALL connected WebRTC peers
    pub async fn broadcast_heartbeat(&self, heartbeat_json: String) -> usize {
        let peers = self.peers.read().await;
        let mut sent_count = 0;

        for (peer_id, peer) in peers.iter() {
            if let Some(dc) = &peer.data_channel {
                if dc.send_text(heartbeat_json.clone()).await.is_ok() {
                    sent_count += 1;
                } else {
                    warn!("Failed to send heartbeat to peer {}", peer_id);
                }
            }
        }

        sent_count
    }

    /// Create an SDP offer to initiate connection to another node
    ///
    /// This is used for node-to-node P2P connections where this node is the initiator.
    /// The offer SDP should be sent to the target node via DHT or relay.
    pub async fn create_offer(&self, peer_id: String) -> Result<String> {
        info!("📤 Creating SDP offer for node {}", peer_id);

        // Create peer connection
        let peer_connection = self.create_peer_connection(peer_id.clone(), PeerType::Node).await?;

        // Create data channel for sending data (initiator creates the channel)
        let data_channel_config = RTCDataChannelInit {
            ordered: Some(true),
            ..Default::default()
        };

        let data_channel = peer_connection
            .create_data_channel("data", Some(data_channel_config))
            .await?;

        // CRITICAL: Set up on_open handler BEFORE signaling (create_offer/set_local_description)
        // According to webrtc-rs docs, on_open must be set before signaling to catch the open event
        let peer_id_for_open = peer_id.clone();
        data_channel.on_open(Box::new(move || {
            info!("🟢 DataChannel OPENED for initiator peer {}", peer_id_for_open);
            Box::pin(async {})
        }));

        // Set up message handler for this data channel (also before signaling)
        let block_tx = self.block_tx.clone();
        let heartbeat_tx = self.heartbeat_tx.clone();
        let tgp_tx = self.tgp_tx.clone();
        let peer_id_clone = peer_id.clone();
        data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
            let peer_id = peer_id_clone.clone();
            let block_tx = block_tx.clone();
            let heartbeat_tx = heartbeat_tx.clone();
            let tgp_tx = tgp_tx.clone();

            Box::pin(async move {
                println!("📨 Received message from node {} via DataChannel ({} bytes)", peer_id, msg.data.len());
                info!("📨 Received message from node {} via DataChannel ({} bytes)", peer_id, msg.data.len());

                // Check if this is a TGP binary packet (DHT GET/PUT/RESPONSE)
                // TGP packets start with version byte (0x01) followed by packet type
                // Packet types: 0x01-0x08 are valid TGP packets
                if msg.data.len() >= 2 {
                    println!("🔍 First two bytes (version, type): 0x{:02x} 0x{:02x}", msg.data[0], msg.data[1]);
                }
                if msg.data.len() >= 21 && msg.data[0] == 0x01 && msg.data[1] >= 0x01 && msg.data[1] <= 0x08 {
                    println!("🔀 Received TGP packet from peer {} via WebRTC DataChannel", peer_id);
                    info!("🔀 Received TGP packet from peer {} via WebRTC DataChannel", peer_id);
                    // Forward to TGP channel for relay processing
                    if let Err(e) = tgp_tx.send((peer_id.clone(), msg.data.to_vec())) {
                        error!("Failed to forward TGP packet from peer {}: {}", peer_id, e);
                    } else {
                        println!("✅ Forwarded TGP packet to relay handler");
                        info!("✅ Forwarded TGP packet to relay handler");
                    }
                    return;
                }

                // Check if this is a text message (UTF-8 encoded JSON)
                // Text messages include: heartbeats, gossip messages, UBTS blocks
                if let Ok(text) = std::str::from_utf8(&msg.data) {
                    // Try to parse as JSON to determine message type
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(text) {
                        // Check for slot ownership gossip
                        if let Some("slot_ownership_gossip") = json_value.get("type").and_then(|v| v.as_str()) {
                            info!("📢 Received slot ownership gossip from node {} via WebRTC", peer_id);
                            // Forward text message to TGP channel with TEXT: prefix
                            if let Err(e) = tgp_tx.send((format!("TEXT:{}", peer_id), msg.data.to_vec())) {
                                error!("Failed to forward gossip message from node {}: {}", peer_id, e);
                            }
                            return;
                        }
                    }

                    // Try to parse as heartbeat
                    if let Ok(heartbeat) = serde_json::from_str::<Heartbeat>(text) {
                        info!("💓 Received heartbeat from node {}", peer_id);
                        if let Err(e) = heartbeat_tx.send(heartbeat) {
                            error!("Failed to forward heartbeat from node {}: {}", peer_id, e);
                        }
                        return;
                    }

                    // Try to parse as UBTS block
                    if let Ok(block) = serde_json::from_str::<UBTSBlock>(text) {
                        info!("✅ Parsed UBTS block from node {}", peer_id);
                        if let Err(e) = block_tx.send((peer_id.clone(), block)) {
                            error!("Failed to forward block from node {}: {}", peer_id, e);
                        }
                        return;
                    }
                }

                error!("❌ Failed to parse message from node {} (not TGP, heartbeat, gossip, or UBTS block)", peer_id);
            })
        }));

        // Store data channel reference BEFORE signaling
        {
            let mut peers = self.peers.write().await;
            if let Some(peer) = peers.get_mut(&peer_id) {
                peer.data_channel = Some(data_channel.clone());
                info!("✅ Stored DataChannel for initiator peer {}", peer_id);
            }
        }

        // NOW create offer (signaling starts here)
        let offer = peer_connection.create_offer(None).await?;
        let offer_sdp = offer.sdp.clone();

        // Get the gathering complete promise BEFORE setting local description
        let mut gathering_complete = peer_connection.gathering_complete_promise().await;

        // Set local description (this starts ICE gathering)
        peer_connection.set_local_description(offer).await?;

        // Wait for ICE gathering to complete
        info!("⏳ Waiting for ICE gathering to complete for node {}", peer_id);
        let _ = gathering_complete.recv().await;
        info!("✅ ICE gathering complete for node {}", peer_id);

        // Get the final offer SDP with all ICE candidates
        let final_offer = peer_connection.local_description().await;
        let final_sdp = if let Some(desc) = final_offer {
            desc.sdp.clone()
        } else {
            offer_sdp
        };

        info!("✅ Created SDP offer for node {} with ICE candidates", peer_id);
        Ok(final_sdp)
    }

    /// Get all connected node peers (excludes browsers)
    pub async fn get_connected_nodes(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        peers
            .iter()
            .filter(|(_, peer)| peer.peer_type == PeerType::Node)
            .map(|(peer_id, _)| peer_id.clone())
            .collect()
    }

    /// Get all connected browser peers
    pub async fn get_connected_browsers(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        peers
            .iter()
            .filter(|(_, peer)| peer.peer_type == PeerType::Browser)
            .map(|(peer_id, _)| peer_id.clone())
            .collect()
    }

    /// Check if peer is connected
    pub async fn is_peer_connected(&self, peer_id: &str) -> bool {
        let peers = self.peers.read().await;
        peers.contains_key(peer_id)
    }

    /// Send heartbeat to a specific peer
    pub async fn send_heartbeat_to(&self, peer_id: &str, heartbeat: &Heartbeat) -> Result<()> {
        let heartbeat_json = heartbeat.to_json()
            .map_err(|e| anyhow::anyhow!("Failed to serialize heartbeat: {}", e))?;

        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(peer_id) {
            if let Some(dc) = &peer.data_channel {
                dc.send_text(heartbeat_json).await?;
                info!("💓 Sent heartbeat to peer {}", peer_id);
            }
        }
        Ok(())
    }

    /// Broadcast heartbeat to all connected node peers (not browsers)
    pub async fn broadcast_heartbeat_message(&self, heartbeat: &Heartbeat) -> Result<usize> {
        let heartbeat_json = heartbeat.to_json()
            .map_err(|e| anyhow::anyhow!("Failed to serialize heartbeat: {}", e))?;

        let peers = self.peers.read().await;
        let mut sent_count = 0;

        for (peer_id, peer) in peers.iter() {
            // Only send heartbeats to node peers (not browsers)
            if peer.peer_type != PeerType::Node {
                continue;
            }

            if let Some(dc) = &peer.data_channel {
                if dc.send_text(heartbeat_json.clone()).await.is_ok() {
                    sent_count += 1;
                } else {
                    warn!("Failed to send heartbeat to node peer {}", peer_id);
                }
            }
        }

        if sent_count > 0 {
            info!("💓 Broadcast heartbeat to {} node peers", sent_count);
        }

        Ok(sent_count)
    }

    /// Get next heartbeat from any peer
    pub async fn next_heartbeat(&self) -> Option<Heartbeat> {
        self.heartbeat_rx.write().await.recv().await
    }

    /// Get next TGP packet (DHT GET/PUT/RESPONSE) from any peer via WebRTC
    /// Returns (peer_id, packet_bytes)
    pub async fn next_tgp_packet(&self) -> Option<(String, Vec<u8>)> {
        self.tgp_rx.write().await.recv().await
    }

    /// Send binary data (e.g., DHT packets) to a specific peer via WebRTC DataChannel
    /// Returns Ok(()) if sent successfully, Err if peer not connected or no DataChannel
    pub async fn send_binary_to_peer(&self, peer_id: &str, data: Vec<u8>) -> Result<()> {
        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(peer_id) {
            if let Some(dc) = &peer.data_channel {
                let bytes = axum::body::Bytes::from(data);
                dc.send(&bytes).await?;
                info!("✅ Sent {} bytes to peer {} via WebRTC DataChannel", bytes.len(), peer_id);
                return Ok(());
            } else {
                anyhow::bail!("Peer {} has no DataChannel", peer_id);
            }
        }
        anyhow::bail!("Peer {} not connected", peer_id);
    }
}
