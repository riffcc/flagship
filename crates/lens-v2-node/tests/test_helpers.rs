//! Shared test helpers for lens-v2-node integration tests
//!
//! Provides TestNode helper for spawning real lens-node instances in tests.

use std::sync::Arc;
use tokio::time::Duration;

/// Test helper to spawn a real lens-node instance
pub struct TestNode {
    pub port: u16,
    pub peer_id: String,
    pub base_url: String,
    pub _shutdown: tokio::sync::oneshot::Sender<()>,
}

impl TestNode {
    #[allow(dead_code)]
    pub async fn spawn(port: u16) -> anyhow::Result<Self> {
        Self::spawn_at_slot(port, None).await
    }

    /// Spawn a test node at a specific slot coordinate
    /// If slot is None, uses a random peer_id (random slot assignment)
    /// If slot is Some, generates a content-addressed peer_id for that slot
    pub async fn spawn_at_slot(
        port: u16,
        slot: Option<citadel_core::topology::SlotCoordinate>,
    ) -> anyhow::Result<Self> {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        // Set up temporary database for this test node
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("rocksdb");

        // Initialize database
        let db = lens_node::db::Database::open(db_path.to_str().unwrap())?;

        // Initialize DHT encryption
        let _dht_enc = lens_node::dht_encryption::DHTEncryption::init_or_generate(
            &db,
            lens_node::dht_encryption::SiteMode::Normal,
        )?;

        // Initialize schema registry
        let registry = Arc::new(lens_node::routes::initialize_registry());
        let state = lens_node::routes::AppState { registry };

        // Initialize WebRTC manager
        let webrtc_manager = Arc::new(lens_node::webrtc_manager::WebRTCManager::new()?);

        // Create shared DHT storage
        let dht_storage = Arc::new(tokio::sync::Mutex::new(
            lens_node::dht_state::DhtState::new(),
        ));

        // Generate peer_id from random ed25519 key (stable identity)
        // Peer ID is ALWAYS derived from ed25519 public key, NOT from slot
        // Slots are explicitly set separately and are decoupled from peer identity
        use ed25519_dalek::SigningKey;
        let signing_key = SigningKey::from_bytes(&rand::random());
        let verifying_key = signing_key.verifying_key();
        let public_key_bytes = verifying_key.to_bytes();
        let hash = blake3::hash(&public_key_bytes);
        let my_peer_id = format!("bafk{}", hex::encode(hash.as_bytes()));

        // Create P2P manager
        let p2p_config = lens_v2_p2p::P2pConfig::default();
        let p2p_manager = Arc::new(lens_v2_p2p::P2pManager::new(p2p_config));
        let sync_state = lens_node::routes::sync::SyncState {
            p2p: p2p_manager.clone(),
        };

        // Create relay state
        let mut relay_state = lens_node::routes::RelayState::new()
            .with_webrtc(webrtc_manager.clone())
            .with_dht_storage(dht_storage.clone())
            .with_node_peer_id(my_peer_id.clone())
            .with_p2p_manager(p2p_manager.clone());

        // Set explicit slot if provided (overrides peer_id_to_slot calculation)
        if let Some(explicit_slot) = slot {
            relay_state = relay_state.with_my_slot(explicit_slot);
        }

        // Create broadcast channel
        let (block_notify_tx, _block_notify_rx) = tokio::sync::mpsc::unbounded_channel();

        // Create account state
        let account_state = lens_node::routes::AccountState::new(db.clone())
            .with_notify(block_notify_tx.clone());

        // Create releases state
        let releases_state =
            lens_node::routes::ReleasesState::with_db(account_state.clone(), db.clone())?
                .with_notify(block_notify_tx);

        // Initialize site identity
        let identity = lens_node::site_identity::SiteIdentity::initialize(&db, None).await?;
        let site_state = lens_node::routes::SiteState::new(Arc::new(identity));

        // Create router
        let app = lens_node::routes::create_router(
            state,
            relay_state.clone(),
            account_state,
            releases_state,
            sync_state,
            None,
            site_state,
        );

        // Start server
        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        let base_url = format!("http://127.0.0.1:{}", port);

        // Spawn WebRTC TGP packet handler (same as in main.rs)
        let webrtc_clone = webrtc_manager.clone();
        let relay_clone = relay_state.clone();
        let my_peer_id_clone = my_peer_id.clone();
        tokio::spawn(async move {
            loop {
                if let Some((peer_id, packet_bytes)) = webrtc_clone.next_tgp_packet().await {
                    // Check if this is a TEXT message (gossip) by checking for "TEXT:" prefix
                    if peer_id.starts_with("TEXT:") {
                        let actual_peer_id = peer_id.strip_prefix("TEXT:").unwrap_or(&peer_id);
                        println!(
                            "📨 [{}] Processing text message from WebRTC peer {}",
                            my_peer_id_clone, actual_peer_id
                        );

                        // Try to parse as JSON gossip message
                        if let Ok(text) = std::str::from_utf8(&packet_bytes) {
                            if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(text) {
                                // Check for slot ownership gossip
                                if let Some("slot_ownership_gossip") =
                                    msg_json.get("type").and_then(|v| v.as_str())
                                {
                                    println!(
                                        "📢 [{}] Received slot ownership gossip from WebRTC peer {}",
                                        my_peer_id_clone, actual_peer_id
                                    );

                                    if let (Some(gossiped_peer_id), Some(slot_obj), Some(ownership_hex)) = (
                                        msg_json.get("peer_id").and_then(|v| v.as_str()),
                                        msg_json.get("slot"),
                                        msg_json.get("ownership_bytes").and_then(|v| v.as_str()),
                                    ) {
                                        if let (Some(x), Some(y), Some(z)) = (
                                            slot_obj.get("x").and_then(|v| v.as_u64()),
                                            slot_obj.get("y").and_then(|v| v.as_u64()),
                                            slot_obj.get("z").and_then(|v| v.as_u64()),
                                        ) {
                                            use citadel_core::topology::SlotCoordinate;
                                            use lens_node::peer_registry::{
                                                peer_location_key, slot_ownership_key,
                                            };
                                            let slot = SlotCoordinate::new(x as i32, y as i32, z as i32);
                                            println!(
                                                "📍 [{}] Storing gossiped slot ownership: {} → ({}, {}, {})",
                                                my_peer_id_clone, gossiped_peer_id, x, y, z
                                            );

                                            // Decode ownership bytes
                                            if let Ok(ownership_bytes) = hex::decode(ownership_hex) {
                                                // Store in local DHT
                                                let mut storage = relay_clone.dht_storage.lock().await;
                                                let location_key = peer_location_key(gossiped_peer_id);
                                                let slot_key = slot_ownership_key(slot);
                                                storage.insert_raw(location_key, ownership_bytes.clone());
                                                storage.insert_raw(slot_key, ownership_bytes);
                                                drop(storage);

                                                println!(
                                                    "✅ [{}] Stored gossiped slot ownership locally from WebRTC peer",
                                                    my_peer_id_clone
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        continue;
                    }

                    println!(
                        "🔀 [{}] Processing TGP packet from WebRTC peer {}",
                        my_peer_id_clone, peer_id
                    );
                    // Parse TGP packet
                    use lens_node::tgp;
                    if let Some((header, payload)) = tgp::parse_packet(&packet_bytes) {
                        use lens_node::tgp::{DhtGetRequest, DhtPutRequest, DhtResponse, PacketType};
                        if let Some(packet_type) = PacketType::from_u8(header.packet_type) {
                            match packet_type {
                                PacketType::DhtPut => {
                                    if let Ok(request) =
                                        serde_json::from_slice::<DhtPutRequest>(payload)
                                    {
                                        println!(
                                            "📥 [{}] DHT PUT from WebRTC peer {}: key={}",
                                            my_peer_id_clone,
                                            peer_id,
                                            hex::encode(&request.key)
                                        );
                                        relay_clone.dht_put(request.key, request.value).await;
                                        println!("✅ [{}] DHT PUT processed", my_peer_id_clone);
                                    }
                                }
                                PacketType::DhtGet => {
                                    if let Ok(request) =
                                        serde_json::from_slice::<DhtGetRequest>(payload)
                                    {
                                        println!(
                                            "📥 [{}] DHT GET from WebRTC peer {}: key={}",
                                            my_peer_id_clone,
                                            peer_id,
                                            hex::encode(&request.key)
                                        );
                                        let value = {
                                            let storage = relay_clone.dht_storage.lock().await;
                                            storage.get_raw(&request.key).cloned()
                                        };

                                        let response = DhtResponse {
                                            key: request.key,
                                            value: value.clone(),
                                        };
                                        let response_payload = serde_json::to_vec(&response).unwrap();
                                        let response_packet = lens_node::tgp::create_packet(
                                            PacketType::DhtResponse.as_u8(),
                                            header.dest_hex,
                                            header.source_hex,
                                            &response_payload,
                                        );

                                        println!(
                                            "📤 [{}] Sending DHT RESPONSE back to {}: value={:?}",
                                            my_peer_id_clone,
                                            peer_id,
                                            value.is_some()
                                        );
                                        let _ = webrtc_clone
                                            .send_binary_to_peer(&peer_id, response_packet)
                                            .await;
                                    }
                                }
                                PacketType::DhtResponse => {
                                    if let Ok(response) = serde_json::from_slice::<DhtResponse>(payload)
                                    {
                                        let key_hex = hex::encode(&response.key);
                                        println!(
                                            "📬 [{}] DHT RESPONSE from WebRTC peer {}: key={}",
                                            my_peer_id_clone, peer_id, key_hex
                                        );
                                        let mut pending = relay_clone.pending_dht_gets.write().await;
                                        if let Some(pending_get) = pending.remove(&key_hex) {
                                            drop(pending);
                                            let _ = pending_get.response_tx.send(response.value);
                                            println!(
                                                "✅ [{}] Delivered DHT response to local dht_get()",
                                                my_peer_id_clone
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        });

        // Spawn server task
        tokio::spawn(async move {
            let server = axum::serve(listener, app);

            tokio::select! {
                _ = server => {},
                _ = shutdown_rx => {
                    // Server shutdown requested
                }
            }
        });

        // EVENT-DRIVEN: Wait for server to be ready by polling /ready endpoint
        let base_url_check = base_url.clone();
        let client = reqwest::Client::new();
        for _ in 0..50 {  // 50 attempts * 100ms = 5 second max wait
            if let Ok(response) = client.get(format!("{}/api/v1/ready", base_url_check)).send().await {
                if response.status().is_success() {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let test_node = TestNode {
            port,
            peer_id: my_peer_id.clone(),
            base_url,
            _shutdown: shutdown_tx,
        };

        // Announce slot ownership if explicit slot was provided
        if let Some(explicit_slot) = slot {
            test_node.announce_slot_ownership(explicit_slot).await?;
        }

        Ok(test_node)
    }

    /// Announce slot ownership via gossip (broadcasts to all connected peers)
    pub async fn announce_slot_ownership(
        &self,
        slot: citadel_core::topology::SlotCoordinate,
    ) -> anyhow::Result<()> {
        println!(
            "📢 Gossiping slot ownership for {} at ({}, {}, {})",
            self.peer_id, slot.x, slot.y, slot.z
        );

        // Call the gossip_slot_ownership HTTP endpoint
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/dht/gossip_slot_ownership", self.base_url);

        let body = serde_json::json!({
            "peer_id": self.peer_id,
            "slot": {
                "x": slot.x,
                "y": slot.y,
                "z": slot.z,
            },
        });

        client.post(&url).json(&body).send().await?;

        println!(
            "✅ Gossiped slot ownership for {} at ({}, {}, {})",
            self.peer_id, slot.x, slot.y, slot.z
        );

        Ok(())
    }

    pub async fn dht_put(&self, key: [u8; 32], value: Vec<u8>) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/dht/put", self.base_url);

        let body = serde_json::json!({
            "key": hex::encode(key),
            "value": hex::encode(value),
        });

        client.post(&url).json(&body).send().await?;

        Ok(())
    }

    /// Query DHT via network routing (routes via relay/WebRTC)
    pub async fn dht_get(&self, key: [u8; 32]) -> anyhow::Result<Option<Vec<u8>>> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/api/v1/dht/get/{}",
            self.base_url,
            hex::encode(key)
        );

        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(value_hex) = data.get("value").and_then(|v| v.as_str()) {
                Ok(Some(hex::decode(value_hex)?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Query LOCAL DHT storage directly (no network routing)
    /// This is used in tests to verify that data is NOT in local storage
    /// (proving that dht_get() routes via network)
    pub async fn dht_get_local(&self, key: [u8; 32]) -> anyhow::Result<Option<Vec<u8>>> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/api/v1/dht/get_local/{}",
            self.base_url,
            hex::encode(key)
        );

        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(value_hex) = data.get("value").and_then(|v| v.as_str()) {
                Ok(Some(hex::decode(value_hex)?))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Create a WebRTC offer to connect to another node
    async fn create_webrtc_offer(&self, to_peer_id: String) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/webrtc/offer", self.base_url);

        let body = serde_json::json!({
            "to_peer_id": to_peer_id,
        });

        let response = client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create WebRTC offer: {}", response.status());
        }

        let data: serde_json::Value = response.json().await?;
        let sdp = data
            .get("sdp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No SDP in response"))?;

        Ok(sdp.to_string())
    }

    /// Handle a WebRTC offer from another node and return an answer
    async fn handle_webrtc_offer(
        &self,
        from_peer_id: String,
        offer_sdp: String,
    ) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/webrtc/answer", self.base_url);

        let body = serde_json::json!({
            "from_peer_id": from_peer_id,
            "sdp": offer_sdp,
        });

        let response = client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to handle WebRTC offer: {}", response.status());
        }

        let data: serde_json::Value = response.json().await?;
        let sdp = data
            .get("sdp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No SDP in response"))?;

        Ok(sdp.to_string())
    }

    /// Complete the WebRTC connection by sending the answer back to the offerer
    async fn handle_webrtc_answer(
        &self,
        from_peer_id: String,
        answer_sdp: String,
    ) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/webrtc/complete", self.base_url);

        let body = serde_json::json!({
            "from_peer_id": from_peer_id,
            "sdp": answer_sdp,
        });

        let response = client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to complete WebRTC connection: {}",
                response.status()
            );
        }

        Ok(())
    }

    /// Establish a full WebRTC connection with another node (convenience method)
    pub async fn establish_webrtc_connection(&self, other_node: &TestNode) -> anyhow::Result<()> {
        println!(
            "🤝 Establishing WebRTC connection: {} → {}",
            self.peer_id, other_node.peer_id
        );

        // Step 1: This node creates an offer
        let offer_sdp = self.create_webrtc_offer(other_node.peer_id.clone()).await?;
        println!("  ✅ Created offer (SDP length: {} bytes)", offer_sdp.len());

        // Step 2: Other node handles the offer and creates an answer
        let answer_sdp = other_node
            .handle_webrtc_offer(self.peer_id.clone(), offer_sdp)
            .await?;
        println!(
            "  ✅ Received answer (SDP length: {} bytes)",
            answer_sdp.len()
        );

        // Step 3: This node completes the connection with the answer
        self.handle_webrtc_answer(other_node.peer_id.clone(), answer_sdp)
            .await?;
        println!("  ✅ Connection established!");

        // Wait for ICE connection and DataChannel to be fully established and OPEN
        // The DataChannel needs time to transition to the "open" state after connection
        println!("  ⏳ Waiting 2 seconds for DataChannel to open...");
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("  ✅ Wait complete!");

        Ok(())
    }
}
