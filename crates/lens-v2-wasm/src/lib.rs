use wasm_bindgen::prelude::*;
use web_sys::{
    WebSocket, MessageEvent, ErrorEvent, CloseEvent,
    RtcPeerConnection, RtcConfiguration, RtcDataChannel, RtcDataChannelInit,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

mod ubts;
use ubts::{UBTSBlock, UBTSTransaction};

mod tgp_udp;
use tgp_udp::{peer_id_to_hex, create_tgp_packet, parse_tgp_packet, packet_types};
use js_sys::Uint8Array;

pub mod dht_minimal;
pub use dht_minimal::{DHTClient, hash_key};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// WebRTC peer connection state
#[allow(dead_code)]
struct PeerConnection {
    rtc: RtcPeerConnection,
    data_channel: Option<RtcDataChannel>,
    peer_id: String,
}

/// P2P Client for browser - uses WebRTC DataChannels for direct P2P communication
#[wasm_bindgen]
pub struct P2pClient {
    relay_url: String,
    websocket: Arc<Mutex<Option<WebSocket>>>,  // Used only for signaling
    peer_id: String,
    peer_hex: u64,  // Hex coordinate for TGP routing
    on_block: Arc<Mutex<Option<js_sys::Function>>>,
    on_peer_discovered: Arc<Mutex<Option<js_sys::Function>>>,
    peer_connections: Arc<Mutex<HashMap<String, PeerConnection>>>,
}

#[wasm_bindgen]
impl P2pClient {
    /// Create a new P2P client
    #[wasm_bindgen(constructor)]
    pub fn new(relay_url: String) -> P2pClient {
        console_error_panic_hook::set_once();

        let peer_id = format!("browser-{}", js_sys::Math::random());
        let peer_hex = peer_id_to_hex(&peer_id);

        console_log!("🌐 Creating WebRTC P2P client with peer_id: {}", peer_id);
        console_log!("🔷 Peer hex coordinate: {:016x}", peer_hex);

        P2pClient {
            relay_url,
            websocket: Arc::new(Mutex::new(None)),
            peer_id,
            peer_hex,
            on_block: Arc::new(Mutex::new(None)),
            on_peer_discovered: Arc::new(Mutex::new(None)),
            peer_connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Connect to the relay WebSocket
    pub fn connect(&mut self) -> Result<(), JsValue> {
        console_log!("🔌 Connecting to relay: {}", self.relay_url);

        let ws = WebSocket::new(&self.relay_url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let ws_clone = ws.clone();
        let peer_id = self.peer_id.clone();
        let on_block = self.on_block.clone();

        // On open - announce as browser peer
        let peer_id_for_announce = self.peer_id.clone();
        let ws_for_announce = ws.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            console_log!("✅ WebSocket connected! Peer: {}", peer_id);

            // Announce as browser peer
            let announce_msg = serde_json::json!({
                "type": "browser_announce",
                "peer_id": peer_id_for_announce,
                "capabilities": ["webrtc", "ubts"]
            });

            if let Ok(json_str) = serde_json::to_string(&announce_msg) {
                if let Err(e) = ws_for_announce.send_with_str(&json_str) {
                    console_log!("⚠️ Failed to send browser announce: {:?}", e);
                } else {
                    console_log!("📣 Announced as browser peer");
                }
            }
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // On message - handle signaling, peer discovery, and UBTS blocks
        let peer_connections_for_msg = self.peer_connections.clone();
        let my_peer_id = self.peer_id.clone();
        let on_peer_discovered = self.on_peer_discovered.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let msg_str = txt.as_string().unwrap_or_default();
                console_log!("📨 Received message: {}", msg_str);

                // Try to parse as JSON first
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&msg_str) {
                    // Check message type
                    if let Some(msg_type) = json.get("type").and_then(|v| v.as_str()) {
                        match msg_type {
                            "browser_peers" => {
                                // Received list of browser peers to connect to
                                if let Some(peers_arr) = json.get("peers").and_then(|v| v.as_array()) {
                                    console_log!("👥 Received {} browser peer(s) to connect to", peers_arr.len());

                                    // Call JavaScript callback for each peer
                                    if let Some(ref callback) = *on_peer_discovered.lock().unwrap() {
                                        for peer_info in peers_arr {
                                            if let Some(peer_id_str) = peer_info.get("peer_id").and_then(|v| v.as_str()) {
                                                if peer_id_str != my_peer_id {
                                                    console_log!("🔗 Discovered browser peer: {}", peer_id_str);
                                                    let peer_id_js = JsValue::from_str(peer_id_str);
                                                    let _ = callback.call1(&JsValue::NULL, &peer_id_js);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            "browser_joined" => {
                                // New browser joined - notify JavaScript
                                if let Some(peer_id_str) = json.get("peer_id").and_then(|v| v.as_str()) {
                                    if peer_id_str != my_peer_id {
                                        console_log!("👋 New browser peer joined: {}", peer_id_str);

                                        // Call JavaScript callback
                                        if let Some(ref callback) = *on_peer_discovered.lock().unwrap() {
                                            let peer_id_js = JsValue::from_str(peer_id_str);
                                            let _ = callback.call1(&JsValue::NULL, &peer_id_js);
                                        }
                                    }
                                }
                            }
                            "offer" | "answer" => {
                                console_log!("🔗 Received WebRTC {}", msg_type);
                                // Handle SDP offer/answer
                                // TODO: Process signaling
                            }
                            "ice-candidate" => {
                                console_log!("🧊 Received ICE candidate");
                                // Handle ICE candidate
                                // TODO: Process ICE
                            }
                            _ => {}
                        }
                    }
                    // Check if it's a UBTS block
                    if let Ok(block) = serde_json::from_value::<UBTSBlock>(json.clone()) {
                        console_log!("🔷 Received UBTS block via relay");

                        // Call JavaScript callback if registered
                        if let Some(ref callback) = *on_block.lock().unwrap() {
                            if let Ok(js_block) = block.to_js() {
                                let _ = callback.call1(&JsValue::NULL, &js_block);
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws_clone.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // On error
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            console_log!("❌ WebSocket error: {:?}", e);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // On close
        let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
            console_log!("🔌 WebSocket closed: code={} reason={}", e.code(), e.reason());
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        *self.websocket.lock().unwrap() = Some(ws);

        Ok(())
    }

    /// Set callback for when blocks are received
    pub fn on_block_received(&mut self, callback: js_sys::Function) {
        *self.on_block.lock().unwrap() = Some(callback);
    }

    /// Set callback for when new browser peers are discovered
    pub fn on_peer_discovered(&mut self, callback: js_sys::Function) {
        *self.on_peer_discovered.lock().unwrap() = Some(callback);
    }

    /// Broadcast a UBTS block to the network
    pub fn broadcast_block(&self, block_json: JsValue) -> Result<(), JsValue> {
        let block = UBTSBlock::from_js(block_json)?;

        console_log!("📡 Broadcasting UBTS block");

        let ws = self.websocket.lock().unwrap();
        if let Some(ref ws) = *ws {
            let block_str = serde_json::to_string(&block)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            ws.send_with_str(&block_str)?;
            console_log!("✅ Block broadcasted successfully");
        } else {
            return Err(JsValue::from_str("WebSocket not connected"));
        }

        Ok(())
    }

    /// Create a DeleteRelease transaction and wrap in a block
    pub fn create_delete_release_block(&self, release_id: String) -> Result<JsValue, JsValue> {
        let tx = UBTSTransaction::DeleteRelease {
            id: release_id,
            signature: None,
        };

        let height = 0; // SPORE: height unused, async convergence
        let block = UBTSBlock::new(height, None, vec![tx]);

        block.to_js()
    }

    /// Create a DeleteFeaturedRelease transaction and wrap in a block
    pub fn create_delete_featured_release_block(&self, featured_release_id: String) -> Result<JsValue, JsValue> {
        let tx = UBTSTransaction::DeleteFeaturedRelease {
            id: featured_release_id,
            signature: None,
        };

        let height = 0; // SPORE: height unused, async convergence
        let block = UBTSBlock::new(height, None, vec![tx]);

        block.to_js()
    }

    /// Create a UBTS block with transactions (transactions should be JSON array)
    pub fn create_block(&self, transactions_json: String, height: u64) -> Result<JsValue, JsValue> {
        let transactions: Vec<UBTSTransaction> = serde_json::from_str(&transactions_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let block = UBTSBlock::new(height, None, transactions);

        block.to_js()
    }

    /// Get peer ID
    pub fn peer_id(&self) -> String {
        self.peer_id.clone()
    }

    /// Disconnect from relay
    pub fn disconnect(&self) -> Result<(), JsValue> {
        let ws = self.websocket.lock().unwrap();
        if let Some(ref ws) = *ws {
            ws.close()?;
            console_log!("🔌 Disconnected from relay");
        }
        Ok(())
    }

    /// Get peer hex coordinate (for TGP routing)
    pub fn peer_hex(&self) -> String {
        format!("{:016x}", self.peer_hex)
    }

    /// Create a WebRTC connection to a peer
    pub fn create_peer_connection(&mut self, peer_id: String) -> Result<(), JsValue> {
        console_log!("🔗 Creating WebRTC connection to peer: {}", peer_id);

        // Configure ICE servers - disable STUN/TURN, use relay only
        // We use TGP-over-UDP or WebSocket relay, not traditional WebRTC ICE
        let rtc_config = {
            let mut cfg = RtcConfiguration::new();
            let ice_servers = js_sys::Array::new();

            // No STUN/TURN servers - rely on WebSocket relay and TGP-over-UDP
            // This prevents "ICE failed, add a TURN server" warnings

            cfg.set_ice_servers(&ice_servers);

            // Set ICE transport policy to relay to suppress direct connection attempts
            cfg.set_ice_transport_policy(web_sys::RtcIceTransportPolicy::Relay);

            cfg
        };

        // Create RTCPeerConnection
        let rtc = RtcPeerConnection::new_with_configuration(&rtc_config)?;

        // Create DataChannel for UBTS blocks
        let dc_init = {
            let mut dc = RtcDataChannelInit::new();
            dc.set_ordered(true);
            dc.set_max_retransmits(3);
            dc
        };

        let data_channel = rtc.create_data_channel_with_data_channel_dict("ubts", &dc_init);

        // Set up DataChannel event handlers
        let on_block = self.on_block.clone();
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            // Try binary TGP packet first
            if let Ok(array_buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                let bytes = uint8_array.to_vec();

                console_log!("📦 Received {} bytes via DataChannel", bytes.len());

                // Parse TGP packet
                if let Some((header, payload)) = parse_tgp_packet(&bytes) {
                    console_log!(
                        "🔷 TGP packet: type={} src={:016x} dst={:016x} len={}",
                        header.packet_type,
                        header.source_hex,
                        header.dest_hex,
                        header.payload_length
                    );

                    // Handle UBTS_BLOCK packets
                    if header.packet_type == packet_types::UBTS_BLOCK {
                        if let Ok(block) = serde_json::from_slice::<UBTSBlock>(payload) {
                            console_log!("✅ Parsed UBTS block from TGP packet");
                            if let Some(ref callback) = *on_block.lock().unwrap() {
                                if let Ok(js_block) = block.to_js() {
                                    let _ = callback.call1(&JsValue::NULL, &js_block);
                                }
                            }
                        }
                    }
                } else {
                    console_log!("⚠️ Failed to parse TGP packet");
                }
            }
            // Fallback to plain JSON for backwards compatibility
            else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let msg_str = txt.as_string().unwrap_or_default();
                console_log!("📦 Received plain JSON via DataChannel (legacy)");

                // Parse and deliver UBTS block
                if let Ok(block) = serde_json::from_str::<UBTSBlock>(&msg_str) {
                    if let Some(ref callback) = *on_block.lock().unwrap() {
                        if let Ok(js_block) = block.to_js() {
                            let _ = callback.call1(&JsValue::NULL, &js_block);
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        data_channel.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let onopen = Closure::wrap(Box::new(move |_: JsValue| {
            console_log!("✅ DataChannel opened - direct P2P connection established!");
        }) as Box<dyn FnMut(JsValue)>);
        data_channel.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // Store peer connection
        let peer_conn = PeerConnection {
            rtc: rtc.clone(),
            data_channel: Some(data_channel),
            peer_id: peer_id.clone(),
        };
        self.peer_connections.lock().unwrap().insert(peer_id.clone(), peer_conn);

        // Set up ICE candidate handler
        let ws = self.websocket.clone();
        let peer_id_clone = peer_id.clone();
        let my_peer_id = self.peer_id.clone();
        let onicecandidate = Closure::wrap(Box::new(move |e: web_sys::RtcPeerConnectionIceEvent| {
            if let Some(candidate) = e.candidate() {
                console_log!("🧊 Generated ICE candidate");

                // Send ICE candidate to peer via relay
                let ice_msg = serde_json::json!({
                    "type": "ice-candidate",
                    "from": my_peer_id,
                    "to": peer_id_clone,
                    "candidate": candidate.candidate(),
                    "sdpMid": candidate.sdp_mid(),
                    "sdpMLineIndex": candidate.sdp_m_line_index(),
                });

                if let Ok(msg_str) = serde_json::to_string(&ice_msg) {
                    if let Some(ref ws) = *ws.lock().unwrap() {
                        let _ = ws.send_with_str(&msg_str);
                    }
                }
            }
        }) as Box<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>);
        rtc.set_onicecandidate(Some(onicecandidate.as_ref().unchecked_ref()));
        onicecandidate.forget();

        console_log!("✅ WebRTC peer connection created for {}", peer_id);
        Ok(())
    }

    /// Broadcast via WebRTC DataChannels with TGP packet format
    pub fn broadcast_block_direct(&self, block_json: JsValue) -> Result<(), JsValue> {
        let block = UBTSBlock::from_js(block_json)?;
        let block_bytes = serde_json::to_vec(&block)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        console_log!("📡 Broadcasting UBTS block via TGP-over-DataChannels");

        let peer_conns = self.peer_connections.lock().unwrap();
        let mut sent_count = 0;

        for (peer_id, conn) in peer_conns.iter() {
            if let Some(ref dc) = conn.data_channel {
                if dc.ready_state() == web_sys::RtcDataChannelState::Open {
                    // Calculate destination hex from peer_id
                    let dest_hex = peer_id_to_hex(peer_id);

                    // Create TGP packet
                    let tgp_packet = create_tgp_packet(
                        packet_types::UBTS_BLOCK,
                        self.peer_hex,
                        dest_hex,
                        &block_bytes
                    );

                    console_log!(
                        "📦 Sending TGP packet: src={:016x} dst={:016x} type=UBTS_BLOCK len={}",
                        self.peer_hex,
                        dest_hex,
                        tgp_packet.len()
                    );

                    // Send TGP packet via DataChannel
                    let array = Uint8Array::from(&tgp_packet[..]);
                    match dc.send_with_array_buffer(&array.buffer()) {
                        Ok(_) => {
                            console_log!("✅ Sent TGP packet to peer {} via DataChannel", peer_id);
                            sent_count += 1;
                        }
                        Err(e) => {
                            console_log!("❌ Failed to send to peer {}: {:?}", peer_id, e);
                        }
                    }
                }
            }
        }

        console_log!("📡 TGP packets sent to {} peer(s) via direct P2P", sent_count);

        if sent_count == 0 {
            return Err(JsValue::from_str("No active DataChannels available"));
        }

        Ok(())
    }

    /// Get list of connected peer IDs
    /// Returns a JSON array of peer IDs that have active WebRTC connections
    pub fn get_connected_peers(&self) -> Result<JsValue, JsValue> {
        let peer_conns = self.peer_connections.lock().unwrap();
        let mut connected_peers = Vec::new();

        for (peer_id, conn) in peer_conns.iter() {
            if let Some(ref dc) = conn.data_channel {
                if dc.ready_state() == web_sys::RtcDataChannelState::Open {
                    connected_peers.push(serde_json::json!({
                        "peer_id": peer_id,
                        "connected": true,
                        "connection_quality": "good"
                    }));
                }
            }
        }

        let result = serde_json::to_value(&connected_peers)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert to JsValue: {}", e)))
    }

    /// Announce discovered peers to relay (browsers help nodes find each other!)
    /// This sends a browser_peer_announcement message with all connected peers
    pub fn announce_discovered_peers(&self) -> Result<(), JsValue> {
        let peer_conns = self.peer_connections.lock().unwrap();
        let mut connected_peers = Vec::new();

        for (peer_id, conn) in peer_conns.iter() {
            if let Some(ref dc) = conn.data_channel {
                let connection_quality = match dc.ready_state() {
                    web_sys::RtcDataChannelState::Open => "good",
                    web_sys::RtcDataChannelState::Connecting => "connecting",
                    _ => "poor"
                };

                connected_peers.push(serde_json::json!({
                    "peer_id": peer_id,
                    "connected": dc.ready_state() == web_sys::RtcDataChannelState::Open,
                    "connection_quality": connection_quality
                }));
            }
        }

        if connected_peers.is_empty() {
            console_log!("🌉 No connected peers to announce");
            return Ok(());
        }

        // Send browser peer announcement to relay
        let announcement = serde_json::json!({
            "type": "browser_peer_announcement",
            "peers": connected_peers
        });

        let ws = self.websocket.lock().unwrap();
        if let Some(ref ws) = *ws {
            let announcement_str = serde_json::to_string(&announcement)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            ws.send_with_str(&announcement_str)?;
            console_log!("🌉 Announced {} discovered peers to relay (helping nodes find each other!)", connected_peers.len());
        } else {
            return Err(JsValue::from_str("WebSocket not connected"));
        }

        Ok(())
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
