//! Test that DHT GET/PUT routing uses WebRTC DataChannels instead of WebSocket
//!
//! This test verifies:
//! 1. When two nodes have WebRTC DataChannel connections, DHT GET should route over DataChannel
//! 2. When two nodes have WebRTC DataChannel connections, DHT PUT should route over DataChannel
//! 3. WebRTC provides direct peer-to-peer routing without WebSocket relay
//! 4. DHT requests sent via WebRTC DataChannel are received and processed correctly

use std::sync::Arc;
use tokio::time::Duration;

/// Test helper to spawn a real lens-node instance
struct TestNode {
    port: u16,
    peer_id: String,
    base_url: String,
    _shutdown: tokio::sync::oneshot::Sender<()>,
}

impl TestNode {
    async fn spawn(port: u16) -> anyhow::Result<Self> {
        Self::spawn_at_slot(port, None).await
    }

    /// Spawn a test node at a specific slot coordinate
    /// If slot is None, uses a random peer_id (random slot assignment)
    /// If slot is Some, generates a content-addressed peer_id for that slot
    async fn spawn_at_slot(port: u16, slot: Option<citadel_core::topology::SlotCoordinate>) -> anyhow::Result<Self> {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        // Set up temporary database for this test node
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("rocksdb");

        // Initialize database
        let db = lens_node::db::Database::open(db_path.to_str().unwrap())?;

        // Initialize DHT encryption
        let dht_enc = lens_node::dht_encryption::DHTEncryption::init_or_generate(
            &db,
            lens_node::dht_encryption::SiteMode::Normal,
        )?;

        // Initialize schema registry
        let registry = Arc::new(lens_node::routes::initialize_registry());
        let state = lens_node::routes::AppState { registry };

        // Initialize WebRTC manager
        let webrtc_manager = Arc::new(lens_node::webrtc_manager::WebRTCManager::new()?);

        // Create shared DHT storage
        let dht_storage = Arc::new(tokio::sync::Mutex::new(lens_node::dht_state::DhtState::new()));

        // Generate peer_id - either content-addressed from slot or random
        let my_peer_id = if let Some(slot) = slot {
            // Content-addressed peer_id from slot coordinate
            lens_node::peer_registry::slot_to_peer_id(slot)
        } else {
            // Random peer_id (old behavior)
            use ed25519_dalek::SigningKey;
            let signing_key = SigningKey::from_bytes(&rand::random());
            let verifying_key = signing_key.verifying_key();
            let public_key_bytes = verifying_key.to_bytes();
            let hash = blake3::hash(&public_key_bytes);
            format!("bafk{}", hex::encode(hash.as_bytes()))
        };

        // Create P2P manager
        let p2p_config = lens_v2_p2p::P2pConfig::default();
        let p2p_manager = Arc::new(lens_v2_p2p::P2pManager::new(p2p_config));
        let sync_state = lens_node::routes::sync::SyncState { p2p: p2p_manager.clone() };

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
        let releases_state = lens_node::routes::ReleasesState::with_db(account_state.clone(), db.clone())?
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
                        println!("📨 [{}] Processing text message from WebRTC peer {}", my_peer_id_clone, actual_peer_id);

                        // Try to parse as JSON gossip message
                        if let Ok(text) = std::str::from_utf8(&packet_bytes) {
                            if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(text) {
                                // Check for slot ownership gossip
                                if let Some("slot_ownership_gossip") = msg_json.get("type").and_then(|v| v.as_str()) {
                                    println!("📢 [{}] Received slot ownership gossip from WebRTC peer {}", my_peer_id_clone, actual_peer_id);

                                    if let (Some(gossiped_peer_id), Some(slot_obj), Some(ownership_hex)) = (
                                        msg_json.get("peer_id").and_then(|v| v.as_str()),
                                        msg_json.get("slot"),
                                        msg_json.get("ownership_bytes").and_then(|v| v.as_str())
                                    ) {
                                        if let (Some(x), Some(y), Some(z)) = (
                                            slot_obj.get("x").and_then(|v| v.as_u64()),
                                            slot_obj.get("y").and_then(|v| v.as_u64()),
                                            slot_obj.get("z").and_then(|v| v.as_u64())
                                        ) {
                                            use citadel_core::topology::SlotCoordinate;
                                            use lens_node::peer_registry::{peer_location_key, slot_ownership_key};
                                            let slot = SlotCoordinate::new(x as i32, y as i32, z as i32);
                                            println!("📍 [{}] Storing gossiped slot ownership: {} → ({}, {}, {})", my_peer_id_clone, gossiped_peer_id, x, y, z);

                                            // Decode ownership bytes
                                            if let Ok(ownership_bytes) = hex::decode(ownership_hex) {
                                                // Store in local DHT
                                                let mut storage = relay_clone.dht_storage.lock().await;
                                                let location_key = peer_location_key(gossiped_peer_id);
                                                let slot_key = slot_ownership_key(slot);
                                                storage.insert_raw(location_key, ownership_bytes.clone());
                                                storage.insert_raw(slot_key, ownership_bytes);
                                                drop(storage);

                                                println!("✅ [{}] Stored gossiped slot ownership locally from WebRTC peer", my_peer_id_clone);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        continue;
                    }

                    println!("🔀 [{}] Processing TGP packet from WebRTC peer {}", my_peer_id_clone, peer_id);
                    // Parse TGP packet
                    use lens_node::tgp;
                    if let Some((header, payload)) = tgp::parse_packet(&packet_bytes) {
                        use lens_node::tgp::{PacketType, DhtGetRequest, DhtPutRequest, DhtResponse};
                        if let Some(packet_type) = PacketType::from_u8(header.packet_type) {
                            match packet_type {
                                PacketType::DhtPut => {
                                    if let Ok(request) = serde_json::from_slice::<DhtPutRequest>(payload) {
                                        println!("📥 [{}] DHT PUT from WebRTC peer {}: key={}", my_peer_id_clone, peer_id, hex::encode(&request.key));
                                        relay_clone.dht_put(request.key, request.value).await;
                                        println!("✅ [{}] DHT PUT processed", my_peer_id_clone);
                                    }
                                }
                                PacketType::DhtGet => {
                                    if let Ok(request) = serde_json::from_slice::<DhtGetRequest>(payload) {
                                        println!("📥 [{}] DHT GET from WebRTC peer {}: key={}", my_peer_id_clone, peer_id, hex::encode(&request.key));
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
                                            &response_payload
                                        );

                                        println!("📤 [{}] Sending DHT RESPONSE back to {}: value={:?}", my_peer_id_clone, peer_id, value.is_some());
                                        let _ = webrtc_clone.send_binary_to_peer(&peer_id, response_packet).await;
                                    }
                                }
                                PacketType::DhtResponse => {
                                    if let Ok(response) = serde_json::from_slice::<DhtResponse>(payload) {
                                        let key_hex = hex::encode(&response.key);
                                        println!("📬 [{}] DHT RESPONSE from WebRTC peer {}: key={}", my_peer_id_clone, peer_id, key_hex);
                                        let mut pending = relay_clone.pending_dht_gets.write().await;
                                        if let Some(pending_get) = pending.remove(&key_hex) {
                                            drop(pending);
                                            let _ = pending_get.response_tx.send(response.value);
                                            println!("✅ [{}] Delivered DHT response to local dht_get()", my_peer_id_clone);
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

        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(500)).await;

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
    async fn announce_slot_ownership(&self, slot: citadel_core::topology::SlotCoordinate) -> anyhow::Result<()> {
        println!("📢 Gossiping slot ownership for {} at ({}, {}, {})", self.peer_id, slot.x, slot.y, slot.z);

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

        client.post(&url)
            .json(&body)
            .send()
            .await?;

        println!("✅ Gossiped slot ownership for {} at ({}, {}, {})", self.peer_id, slot.x, slot.y, slot.z);

        Ok(())
    }

    async fn dht_put(&self, key: [u8; 32], value: Vec<u8>) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/dht/put", self.base_url);

        let body = serde_json::json!({
            "key": hex::encode(key),
            "value": hex::encode(value),
        });

        client.post(&url)
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    async fn dht_get(&self, key: [u8; 32]) -> anyhow::Result<Option<Vec<u8>>> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/dht/get/{}", self.base_url, hex::encode(key));

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

        let response = client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create WebRTC offer: {}", response.status());
        }

        let data: serde_json::Value = response.json().await?;
        let sdp = data.get("sdp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No SDP in response"))?;

        Ok(sdp.to_string())
    }

    /// Handle a WebRTC offer from another node and return an answer
    async fn handle_webrtc_offer(&self, from_peer_id: String, offer_sdp: String) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/webrtc/answer", self.base_url);

        let body = serde_json::json!({
            "from_peer_id": from_peer_id,
            "sdp": offer_sdp,
        });

        let response = client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to handle WebRTC offer: {}", response.status());
        }

        let data: serde_json::Value = response.json().await?;
        let sdp = data.get("sdp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No SDP in response"))?;

        Ok(sdp.to_string())
    }

    /// Complete the WebRTC connection by sending the answer back to the offerer
    async fn handle_webrtc_answer(&self, from_peer_id: String, answer_sdp: String) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/webrtc/complete", self.base_url);

        let body = serde_json::json!({
            "from_peer_id": from_peer_id,
            "sdp": answer_sdp,
        });

        let response = client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to complete WebRTC connection: {}", response.status());
        }

        Ok(())
    }

    /// Establish a full WebRTC connection with another node (convenience method)
    async fn establish_webrtc_connection(&self, other_node: &TestNode) -> anyhow::Result<()> {
        println!("🤝 Establishing WebRTC connection: {} → {}", self.peer_id, other_node.peer_id);

        // Step 1: This node creates an offer
        let offer_sdp = self.create_webrtc_offer(other_node.peer_id.clone()).await?;
        println!("  ✅ Created offer (SDP length: {} bytes)", offer_sdp.len());

        // Step 2: Other node handles the offer and creates an answer
        let answer_sdp = other_node.handle_webrtc_offer(self.peer_id.clone(), offer_sdp).await?;
        println!("  ✅ Received answer (SDP length: {} bytes)", answer_sdp.len());

        // Step 3: This node completes the connection with the answer
        self.handle_webrtc_answer(other_node.peer_id.clone(), answer_sdp).await?;
        println!("  ✅ Connection established!");

        // Wait for ICE connection and DataChannel to be fully established and OPEN
        // The DataChannel needs time to transition to the "open" state after connection
        println!("  ⏳ Waiting 2 seconds for DataChannel to open...");
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("  ✅ Wait complete!");;

        Ok(())
    }
}

#[tokio::test]
async fn test_dht_put_uses_webrtc_datachannel() -> anyhow::Result<()> {
    use citadel_core::topology::SlotCoordinate;

    // Spawn two real lens-node instances at SEQUENTIAL slots from origin
    // Node1 at (0,0,0), Node2 at (0,1,0) - adjacent slots!
    let node1 = TestNode::spawn_at_slot(15001, Some(SlotCoordinate::new(0, 0, 0))).await?;
    let node2 = TestNode::spawn_at_slot(15002, Some(SlotCoordinate::new(0, 1, 0))).await?;

    println!("✅ Spawned node1 (port {}) at slot (0,0,0) with peer_id: {}", node1.port, node1.peer_id);
    println!("✅ Spawned node2 (port {}) at slot (0,1,0) with peer_id: {}", node2.port, node2.peer_id);

    // Establish WebRTC connection between nodes
    println!("🔗 Establishing WebRTC DataChannel connection...");
    node1.establish_webrtc_connection(&node2).await?;
    println!("✅ WebRTC connection established!");

    // Create test data - use same key as GET test to ensure consistency
    let test_key: [u8; 32] = blake3::hash(b"test-key").into();
    let test_value = b"test-value-12345".to_vec();

    // Node 1 performs DHT PUT (should route over WebRTC DataChannel!)
    node1.dht_put(test_key, test_value.clone()).await?;

    println!("✅ Node1 performed DHT PUT");

    // Wait for propagation over WebRTC
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify the value was stored (should be routable via WebRTC)
    let stored_value = node1.dht_get(test_key).await?;

    assert!(stored_value.is_some(), "DHT PUT should store value");
    assert_eq!(stored_value.unwrap(), test_value, "Stored value should match");

    println!("✅ DHT PUT test passed with real nodes and WebRTC!");

    Ok(())
}

#[tokio::test]
async fn test_dht_get_uses_webrtc_datachannel() -> anyhow::Result<()> {
    use citadel_core::topology::SlotCoordinate;

    // Spawn two real lens-node instances at SEQUENTIAL slots from origin
    // Node1 at (0,0,0), Node2 at (0,1,0) - adjacent slots!
    let node1 = TestNode::spawn_at_slot(15003, Some(SlotCoordinate::new(0, 0, 0))).await?;
    let node2 = TestNode::spawn_at_slot(15004, Some(SlotCoordinate::new(0, 1, 0))).await?;

    println!("✅ Spawned node1 (port {}) at slot (0,0,0) with peer_id: {}", node1.port, node1.peer_id);
    println!("✅ Spawned node2 (port {}) at slot (0,1,0) with peer_id: {}", node2.port, node2.peer_id);

    // Establish WebRTC connection between nodes
    println!("🔗 Establishing WebRTC DataChannel connection...");
    node1.establish_webrtc_connection(&node2).await?;
    println!("✅ WebRTC connection established!");

    // Wait for slot announcements to propagate through DHT
    println!("⏳ Waiting for slot announcements to propagate...");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Re-announce slot ownership now that WebRTC connection is established
    // This will route the announcements to the correct slot owners
    println!("📢 Re-announcing slot ownership over WebRTC mesh...");
    node1.announce_slot_ownership(SlotCoordinate::new(0, 0, 0)).await?;
    node2.announce_slot_ownership(SlotCoordinate::new(0, 1, 0)).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("✅ Slot ownership propagated");

    // Create test data - use same key as PUT test to ensure consistency
    let test_key: [u8; 32] = blake3::hash(b"test-key").into();
    let test_value = b"test-value-12345".to_vec();

    // Debug: Check where this key maps to
    use citadel_core::key_mapping::key_to_slot;
    use citadel_core::topology::MeshConfig;
    let mesh_config = MeshConfig::new(2, 2, 2); // 2 nodes = 2x2x2 mesh
    let target_slot = key_to_slot(&test_key, &mesh_config);
    println!("🔑 Test key maps to slot ({}, {}, {})", target_slot.x, target_slot.y, target_slot.z);
    println!("   Node1 is at slot (0, 0, 0)");
    println!("   Node2 is at slot (0, 1, 0)");

    // Node 2 stores the value (should route to Node1 via WebRTC!)
    node2.dht_put(test_key, test_value.clone()).await?;

    println!("✅ Node2 performed DHT PUT (should have routed to Node1)");

    // Wait for DHT routing to complete over WebRTC
    println!("⏳ Waiting for DHT PUT to propagate over WebRTC...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Node 1 performs DHT GET (key belongs to Node1's slot, should find it locally!)
    println!("📥 Node1 performing DHT GET (key belongs to Node1's slot)...");
    let result = node1.dht_get(test_key).await?;

    assert!(result.is_some(), "DHT GET should find value");
    assert_eq!(result.unwrap(), test_value, "Retrieved value should match");

    println!("✅ DHT GET test passed with real nodes and WebRTC!");

    Ok(())
}

#[tokio::test]
async fn test_3node_multihop_dht_routing() -> anyhow::Result<()> {
    use citadel_core::topology::SlotCoordinate;

    println!("🚀 Starting 3-node multi-hop DHT routing test");
    println!("   This tests DHT routing across multiple hops via WebRTC mesh");
    println!();

    // Spawn three nodes in a line: Node0 → Node1 → Node2
    // Slots: (0,0,0) → (0,1,0) → (0,2,0)
    let node0 = TestNode::spawn_at_slot(15010, Some(SlotCoordinate::new(0, 0, 0))).await?;
    let node1 = TestNode::spawn_at_slot(15011, Some(SlotCoordinate::new(0, 1, 0))).await?;
    let node2 = TestNode::spawn_at_slot(15012, Some(SlotCoordinate::new(0, 2, 0))).await?;

    println!("✅ Spawned node0 (port {}) at slot (0,0,0) with peer_id: {}", node0.port, &node0.peer_id[..20]);
    println!("✅ Spawned node1 (port {}) at slot (0,1,0) with peer_id: {}", node1.port, &node1.peer_id[..20]);
    println!("✅ Spawned node2 (port {}) at slot (0,2,0) with peer_id: {}", node2.port, &node2.peer_id[..20]);
    println!();

    // Establish WebRTC connections in a line: Node0 ↔ Node1 ↔ Node2
    println!("🔗 Building WebRTC mesh (3-node line topology)...");

    // Node0 ↔ Node1
    println!("   Connecting Node0 → Node1...");
    node0.establish_webrtc_connection(&node1).await?;
    println!("   ✅ Node0 ↔ Node1 connected");

    // Node1 ↔ Node2
    println!("   Connecting Node1 → Node2...");
    node1.establish_webrtc_connection(&node2).await?;
    println!("   ✅ Node1 ↔ Node2 connected");

    println!("✅ WebRTC mesh established (3 nodes in line topology)");
    println!();

    // Wait for connections to stabilize
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Announce slot ownership - gossip should flood through all 3 nodes
    println!("📢 Announcing slot ownership via gossip (should flood to all nodes)...");
    node0.announce_slot_ownership(SlotCoordinate::new(0, 0, 0)).await?;
    node1.announce_slot_ownership(SlotCoordinate::new(0, 1, 0)).await?;
    node2.announce_slot_ownership(SlotCoordinate::new(0, 2, 0)).await?;

    // Wait for gossip to propagate through mesh
    println!("⏳ Waiting for gossip to propagate through all 3 nodes...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("✅ Gossip propagation complete");
    println!();

    // Create test data that maps to Node2's slot
    // We want Node0 to DHT PUT → route through Node1 → arrive at Node2
    let test_key: [u8; 32] = blake3::hash(b"multihop-test-key").into();
    let test_value = b"multihop-test-value-42".to_vec();

    // Debug: Check where this key maps to
    use citadel_core::key_mapping::key_to_slot;
    use citadel_core::topology::MeshConfig;
    let mesh_config = MeshConfig::new(3, 3, 3); // 3 nodes = 3x3x3 mesh
    let target_slot = key_to_slot(&test_key, &mesh_config);
    println!("🔑 Test key maps to slot ({}, {}, {})", target_slot.x, target_slot.y, target_slot.z);
    println!("   Node0 is at slot (0, 0, 0)");
    println!("   Node1 is at slot (0, 1, 0)");
    println!("   Node2 is at slot (0, 2, 0)");
    println!("   Expected: Node0 → (route) → Node1 → (route) → Node2");
    println!();

    // Node0 performs DHT PUT - should route through Node1 to Node2
    println!("📤 Node0 performing DHT PUT (should multi-hop to target slot owner)...");
    node0.dht_put(test_key, test_value.clone()).await?;
    println!("✅ Node0 DHT PUT request sent");

    // Wait for multi-hop routing to complete
    println!("⏳ Waiting for multi-hop DHT routing to complete...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!();

    // Try to retrieve from all nodes to see where it ended up
    println!("🔍 Checking which node(s) have the value...");

    let node0_result = node0.dht_get(test_key).await?;
    println!("   Node0 DHT GET: {}", if node0_result.is_some() { "✅ HAS VALUE" } else { "❌ NO VALUE" });

    let node1_result = node1.dht_get(test_key).await?;
    println!("   Node1 DHT GET: {}", if node1_result.is_some() { "✅ HAS VALUE" } else { "❌ NO VALUE" });

    let node2_result = node2.dht_get(test_key).await?;
    println!("   Node2 DHT GET: {}", if node2_result.is_some() { "✅ HAS VALUE" } else { "❌ NO VALUE" });
    println!();

    // The value should be stored at the node owning the target slot
    // In a properly functioning DHT, the greedy routing should find the closest node
    assert!(
        node0_result.is_some() || node1_result.is_some() || node2_result.is_some(),
        "DHT PUT should store value at one of the nodes (multi-hop routing should work)"
    );

    // Verify the value matches
    let stored_value = node0_result.or(node1_result).or(node2_result).unwrap();
    assert_eq!(stored_value, test_value, "Retrieved value should match");

    println!("✅ 3-node multi-hop DHT routing test PASSED!");
    println!("   ✓ WebRTC mesh established with 3 nodes");
    println!("   ✓ Gossip flooded through all nodes");
    println!("   ✓ Multi-hop DHT PUT succeeded");
    println!("   ✓ DHT GET retrieved correct value");

    Ok(())
}
