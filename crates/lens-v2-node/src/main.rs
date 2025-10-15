// Import from library instead of redeclaring modules
use lens_node::{
    routes::{self, initialize_registry, AppState, RelayState, AccountState, ReleasesState, SiteState, sync::SyncState},
    db::{self, prefixes, make_key},
    sync_orchestrator::SyncOrchestrator,
    ubts,
    dht_encryption,
    site_identity::SiteIdentity,
    peer_registry,
    webrtc_manager,
    tgp::{DhtGetRequest, DhtPutRequest, DhtResponse},
};
use lens_v2_p2p::{P2pManager, P2pConfig};
use std::env;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing - default to info level for cleaner production logs
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lens_node=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Force a flush to ensure logs appear immediately
    eprintln!("=== Lens Node v2 - Version 0.8.36 - Starting ===");

    // Print startup banner
    tracing::info!("╔══════════════════════════════════════════════╗");
    tracing::info!("║       Lens Node v2 - Version 0.8.59          ║");
    tracing::info!("║   P2P Content Distribution & Sync Node       ║");
    tracing::info!("╚══════════════════════════════════════════════╝");

    // Get port from environment or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "5002".to_string())
        .parse::<u16>()?;

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("🚀 Starting server on {}", addr);

    // Initialize RocksDB database
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| ".lens-node-data/rocksdb".to_string());
    let db = db::Database::open(&db_path)?;
    tracing::info!("Initialized RocksDB at: {}", db_path);

    // Initialize DHT encryption
    let site_mode_str = env::var("SITE_MODE").unwrap_or_else(|_| "normal".to_string());
    let site_mode = dht_encryption::SiteMode::from_str(&site_mode_str)?;
    let dht_enc = dht_encryption::DHTEncryption::init_or_generate(&db, site_mode)?;
    tracing::info!("Initialized DHT encryption in {:?} mode", site_mode);

    if site_mode == dht_encryption::SiteMode::Enterprise {
        tracing::warn!("🔒 ENTERPRISE MODE ENABLED - All DHT values will be encrypted");
        tracing::warn!("🔒 DHT data will NOT be shareable with other nodes");
    } else {
        tracing::info!("🔓 Normal mode - DHT values are shareable");
        tracing::info!("🔑 Site Key (for sharing): {}", dht_enc.site_key_hex());
    }

    // Initialize schema registry with built-in schemas
    let registry = Arc::new(initialize_registry());
    tracing::info!("Initialized schema registry");

    // Create application state
    let state = AppState { registry };

    // Initialize WebRTC manager for browser-to-node connections
    let webrtc_manager = Arc::new(webrtc_manager::WebRTCManager::new()?);
    tracing::info!("Initialized WebRTC manager for browser peers");

    // Create shared DHT storage for Citadel mesh topology (recursive DHT)
    // This storage is SHARED between relay and orchestrator for slot ownership discovery
    // Uses DhtState with bootstrap and merge capabilities for global DHT consensus
    let dht_storage = Arc::new(tokio::sync::Mutex::new(lens_node::dht_state::DhtState::new()));
    tracing::info!("Initialized shared DHT storage for Citadel hexagonal mesh with bootstrap/merge");

    // Generate or load persistent NODE keypair (separate from site identity!)
    // Each node in the mesh needs its own keypair for peer identity
    use ed25519_dalek::{SigningKey, VerifyingKey};

    let node_keypair_key = b"node:ed25519_keypair".to_vec();
    let (node_signing_key, node_public_key): (SigningKey, VerifyingKey) =
        if let Some(stored_keypair) = db.get::<Vec<u8>, Vec<u8>>(node_keypair_key.clone())? {
            // Load existing keypair
            let key_bytes: [u8; 32] = stored_keypair.as_slice().try_into()
                .map_err(|_| anyhow::anyhow!("Invalid stored keypair length"))?;
            let signing_key = SigningKey::from_bytes(&key_bytes);
            let verifying_key = signing_key.verifying_key();
            tracing::info!("✅ Loaded persistent node keypair from database");
            (signing_key, verifying_key)
        } else {
            // Generate new keypair and store it
            let signing_key = SigningKey::from_bytes(&rand::random());
            let verifying_key = signing_key.verifying_key();
            db.put(node_keypair_key, signing_key.as_bytes())?;
            tracing::info!("✅ Generated new persistent node keypair");
            (signing_key, verifying_key)
        };

    // Generate peer_id as CIDv1 BLAKE3 hash of our NODE public key
    // This makes peer_id self-certifying and deterministic!
    // Format: b<base32-encoded-multihash> (CIDv1 with raw codec)
    let public_key_bytes = node_public_key.to_bytes();
    let hash = blake3::hash(&public_key_bytes);

    // Build CIDv1: <multicodec-prefix><multibase-prefix><multihash>
    // CIDv1 = 0x01 (version) + 0x55 (raw codec) + 0x1e (blake3) + 0x20 (32 bytes) + hash
    let mut cid_bytes = Vec::with_capacity(36);
    cid_bytes.push(0x01); // CIDv1
    cid_bytes.push(0x55); // raw codec
    cid_bytes.push(0x1e); // blake3 multihash code
    cid_bytes.push(0x20); // 32 bytes length
    cid_bytes.extend_from_slice(hash.as_bytes());

    // Encode as hex (simpler than base32, still content-addressed and self-certifying)
    // Format: bafk<hex> where "bafk" indicates BLAKE3 hash
    let my_peer_id = format!("bafk{}", hex::encode(hash.as_bytes()));

    tracing::info!("🎯 Node Public Key: {}", hex::encode(public_key_bytes));
    tracing::info!("🎯 Peer ID (CIDv1 BLAKE3 of node public key): {}", my_peer_id);

    // FIXED MESH: Use environment variables or default to 8×7×1 (56 slots for ~50 nodes)
    // This ensures all nodes use the SAME mesh dimensions for consistent neighbor discovery!
    let mesh_width = env::var("MESH_WIDTH")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(8);
    let mesh_height = env::var("MESH_HEIGHT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(7);
    let mesh_depth = env::var("MESH_DEPTH")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    let mesh_config = citadel_core::topology::MeshConfig::new(mesh_width, mesh_height, mesh_depth);
    let my_slot = peer_registry::peer_id_to_slot(&my_peer_id, &mesh_config);

    tracing::info!("🎯 My slot: {:?} in FIXED mesh {}×{}×{} ({} total slots)",
        my_slot, mesh_config.width, mesh_config.height, mesh_config.depth,
        mesh_config.width * mesh_config.height * mesh_config.depth);

    // Create P2P manager for sync status tracking
    let p2p_config = P2pConfig::default();
    let p2p_manager = Arc::new(P2pManager::new(p2p_config));
    let sync_state = SyncState { p2p: p2p_manager.clone() };
    tracing::info!("Initialized P2P sync manager");

    // Create relay state for P2P peer discovery with WebRTC support and shared DHT
    let relay_state = RelayState::new()
        .with_webrtc(webrtc_manager.clone())
        .with_dht_storage(dht_storage.clone())
        .with_node_peer_id(my_peer_id.clone())
        .with_p2p_manager(p2p_manager.clone());
    tracing::info!("Initialized P2P relay with WebRTC support, shared DHT storage, node peer_id, and P2P manager");

    // Create broadcast channel for immediate WantList updates
    let (block_notify_tx, block_notify_rx) = tokio::sync::mpsc::unbounded_channel();

    // Create account state for authorization (UBTS-based, syncs via SPORE)
    let account_state = AccountState::new(db.clone()).with_notify(block_notify_tx.clone());
    tracing::info!("Initialized account management with UBTS transaction storage and instant broadcast");

    // Auto-authorize admin keys from environment variable if set (comma-separated)
    if let Ok(admin_keys) = env::var("ADMIN_PUBLIC_KEY") {
        if !admin_keys.is_empty() {
            for admin_key in admin_keys.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                match account_state.authorize_admin(admin_key.to_string()).await {
                    Ok(_) => tracing::info!("✅ Auto-authorized admin key: {}", admin_key),
                    Err(e) => tracing::warn!("⚠️ Failed to auto-authorize admin key {}: {}", admin_key, e),
                }
            }
        }
    }

    // Create releases state for content management with RocksDB persistence
    let releases_state = ReleasesState::with_db(account_state.clone(), db.clone())?
        .with_notify(block_notify_tx);
    tracing::info!("Initialized releases management with RocksDB persistence and instant broadcast");

    // Reconcile delete transactions on startup
    // Process all delete transactions that exist in the database to ensure consistency
    tracing::info!("🔄 Reconciling delete transactions from database...");
    use ubts::UBTSBlock;
    use db::{prefixes, make_key};

    let delete_txs: Vec<UBTSBlock> = db.get_all_with_prefix(prefixes::DELETE_TRANSACTION)?;
    let mut deleted_count = 0;

    for delete_tx_block in delete_txs {
        for tx in &delete_tx_block.transactions {
            match tx {
                ubts::UBTSTransaction::DeleteRelease { id, .. } => {
                    let release_key = make_key(prefixes::RELEASE, id);
                    if db.exists(&release_key)? {
                        db.delete(&release_key)?;
                        deleted_count += 1;
                        tracing::debug!("🗑️ Reconciled delete for release: {}", id);
                    }
                }
                ubts::UBTSTransaction::DeleteFeaturedRelease { id, .. } => {
                    let featured_key = make_key(prefixes::FEATURED_RELEASE, id);
                    if db.exists(&featured_key)? {
                        db.delete(&featured_key)?;
                        deleted_count += 1;
                        tracing::debug!("🗑️ Reconciled delete for featured release: {}", id);
                    }
                }
                _ => {}
            }
        }
    }

    if deleted_count > 0 {
        tracing::info!("✅ Reconciliation complete: processed {} delete transactions", deleted_count);
    } else {
        tracing::info!("✅ Reconciliation complete: no pending deletes");
    }

    // Initialize site identity (SiteID and SiteKey for defederation - separate from node/peer identity!)
    let site_name = env::var("SITE_NAME").ok();
    let identity = SiteIdentity::initialize(&db, site_name.clone()).await?;
    tracing::info!("🆔 Site ID: {}", identity.site_id);
    tracing::info!("🔑 Site Public Key: {}", identity.site_key.public_key_base58());
    if let Some(ref name) = identity.site_name {
        tracing::info!("📛 Site Name: {}", name);
    }
    let site_state = SiteState::new(Arc::new(identity));

    // Create the router with state (clone relay_state for later use in gossip task)
    // Note: DHT state is None for now as we're using RocksDB for persistence
    // When DHT storage is actively used, we'll pass Some(dht_state) here
    let app = routes::create_router(state, relay_state.clone(), account_state, releases_state, sync_state, None, site_state);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    tracing::info!("To authorize yourself as admin, run:");
    tracing::info!("  curl -X POST http://127.0.0.1:{}/api/v1/admin/authorize \\", port);
    tracing::info!("    -H 'Content-Type: application/json' \\");
    tracing::info!("    -d '{{\"publicKey\": \"YOUR_PUBLIC_KEY_HERE\"}}'");

    // Slot ownership announcement moved to SyncOrchestrator after relay connection
    // This ensures we can route through relay even with 0 WebRTC neighbors
    tracing::info!("ℹ️  Slot ownership will be announced after relay connection established");

    // Create orchestrator with LazyNode (NO RELAY!)
    // Note: relay_url is still required for now but will be removed in parallel task
    let relay_url = env::var("RELAY_URL")
        .unwrap_or_else(|_| format!("ws://localhost:{}/api/v1/relay/ws", port));

    let orchestrator = Arc::new(SyncOrchestrator::new(
        relay_url,
        my_peer_id.clone(),
        my_slot,
        mesh_config,
        p2p_manager.clone(),
        webrtc_manager.clone(),
        db.clone(),
        block_notify_rx,
        dht_storage.clone(),
        relay_state.clone(),
    ));

    // Spawn server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Server error: {}", e);
        }
    });

    // Spawn WebRTC TGP packet handler - processes DHT GET/PUT/RESPONSE packets AND gossip messages from WebRTC peers
    let webrtc_clone = webrtc_manager.clone();
    let relay_clone = relay_state.clone();
    tokio::spawn(async move {
        tracing::info!("📡 Starting WebRTC TGP/gossip message handler...");
        loop {
            if let Some((peer_id, packet_bytes)) = webrtc_clone.next_tgp_packet().await {
                // Check if this is a TEXT message (gossip) by checking for "TEXT:" prefix
                if peer_id.starts_with("TEXT:") {
                    let actual_peer_id = peer_id.strip_prefix("TEXT:").unwrap_or(&peer_id);
                    tracing::info!("📨 Processing text message from WebRTC peer {}", actual_peer_id);

                    // Try to parse as JSON gossip message
                    if let Ok(text) = std::str::from_utf8(&packet_bytes) {
                        if let Ok(msg_json) = serde_json::from_str::<serde_json::Value>(text) {
                            // Check for slot ownership gossip
                            if let Some("slot_ownership_gossip") = msg_json.get("type").and_then(|v| v.as_str()) {
                                tracing::info!("📢 Received slot ownership gossip from WebRTC peer {}", actual_peer_id);

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
                                        tracing::info!("📍 Storing gossiped slot ownership: {} → ({}, {}, {})", gossiped_peer_id, x, y, z);

                                        // Decode ownership bytes
                                        if let Ok(ownership_bytes) = hex::decode(ownership_hex) {
                                            // Store in local DHT
                                            let mut storage = relay_clone.dht_storage.lock().await;
                                            let location_key = peer_location_key(gossiped_peer_id);
                                            let slot_key = slot_ownership_key(slot);
                                            storage.insert_raw(location_key, ownership_bytes.clone());
                                            storage.insert_raw(slot_key, ownership_bytes);
                                            drop(storage);

                                            tracing::info!("✅ Stored gossiped slot ownership locally from WebRTC peer");

                                            // Re-gossip to other peers (WebSocket and WebRTC)
                                            let senders = relay_clone.peer_senders.read().await;
                                            let mut regossip_count = 0;
                                            for (other_peer_id, tx) in senders.iter() {
                                                if other_peer_id != actual_peer_id && other_peer_id != gossiped_peer_id {
                                                    if let Ok(_) = tx.send(axum::extract::ws::Message::Text(text.to_string())) {
                                                        regossip_count += 1;
                                                    }
                                                }
                                            }
                                            drop(senders);

                                            // Also re-gossip via WebRTC to other peers
                                            if let Some(ref webrtc_mgr) = relay_clone.webrtc_manager {
                                                let webrtc_peers = webrtc_mgr.peers.read().await;
                                                for (rtc_peer_id, peer) in webrtc_peers.iter() {
                                                    if rtc_peer_id != actual_peer_id && rtc_peer_id != gossiped_peer_id {
                                                        if let Some(ref dc) = peer.data_channel {
                                                            if let Ok(_) = dc.send_text(text.to_string()).await {
                                                                regossip_count += 1;
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if regossip_count > 0 {
                                                tracing::info!("🌐 Re-gossiped slot ownership to {} peers from WebRTC", regossip_count);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    continue;
                }

                tracing::info!("🔀 Processing TGP packet from WebRTC peer {}", peer_id);

                // Parse TGP packet (same logic as WebSocket binary handler in relay.rs)
                use lens_node::tgp;
                if let Some((header, payload)) = tgp::parse_packet(&packet_bytes) {
                    tracing::info!(
                        "TGP packet: type={:02x} src={:016x} dst={:016x} len={}",
                        header.packet_type, header.source_hex, header.dest_hex, header.payload_length
                    );

                    // Handle DHT packets
                    use lens_node::tgp::PacketType;
                    if let Some(packet_type) = PacketType::from_u8(header.packet_type) {
                        match packet_type {
                            PacketType::DhtPut => {
                                if let Ok(request) = serde_json::from_slice::<DhtPutRequest>(payload) {
                                    tracing::info!("📥 DHT PUT from WebRTC peer {}: key={}", peer_id, hex::encode(&request.key));
                                    relay_clone.dht_put(request.key, request.value).await;
                                }
                            }
                            PacketType::DhtGet => {
                                if let Ok(request) = serde_json::from_slice::<DhtGetRequest>(payload) {
                                    tracing::info!("📥 DHT GET from WebRTC peer {}: key={}", peer_id, hex::encode(&request.key));

                                    // Look up the value (may be local or need to route)
                                    // For now, we'll just check locally and respond
                                    let value = {
                                        let storage = relay_clone.dht_storage.lock().await;
                                        storage.get_raw(&request.key).cloned()
                                    };

                                    // Create DHT RESPONSE packet
                                    let response = DhtResponse {
                                        key: request.key,
                                        value,
                                    };
                                    let response_payload = serde_json::to_vec(&response).unwrap();
                                    let response_packet = lens_node::tgp::create_packet(
                                        PacketType::DhtResponse.as_u8(),
                                        header.dest_hex, // source = us (we received at dest_hex)
                                        header.source_hex, // dest = original requester
                                        &response_payload
                                    );

                                    // Send response back via WebRTC
                                    if let Err(e) = webrtc_clone.send_binary_to_peer(&peer_id, response_packet).await {
                                        tracing::warn!("Failed to send DHT RESPONSE via WebRTC to {}: {}", peer_id, e);
                                    } else {
                                        tracing::info!("✅ Sent DHT RESPONSE via WebRTC to {}", peer_id);
                                    }
                                }
                            }
                            PacketType::DhtResponse => {
                                if let Ok(response) = serde_json::from_slice::<DhtResponse>(payload) {
                                    let key_hex = hex::encode(&response.key);
                                    tracing::info!("📬 DHT RESPONSE from WebRTC peer {}: key={}", peer_id, key_hex);

                                    // Check if we have a pending GET request for this key
                                    let mut pending = relay_clone.pending_dht_gets.write().await;
                                    if let Some(pending_get) = pending.remove(&key_hex) {
                                        // This response is for US! Deliver it.
                                        drop(pending);
                                        if pending_get.response_tx.send(response.value).is_err() {
                                            tracing::warn!("Failed to send DHT response for key={} (receiver dropped)", key_hex);
                                        } else {
                                            tracing::info!("✅ Delivered DHT response for key={} to local dht_get()", key_hex);
                                        }
                                    } else {
                                        drop(pending);
                                        tracing::warn!("⚠️  Received DHT RESPONSE for key={} but no pending request found", key_hex);
                                    }
                                }
                            }
                            _ => {
                                tracing::debug!("Ignoring non-DHT TGP packet type: {:?}", packet_type);
                            }
                        }
                    }
                } else {
                    tracing::warn!("⚠️  Failed to parse TGP packet from WebRTC peer {}", peer_id);
                }
            }
        }
    });

    // Start the orchestrator immediately (pure DHT mesh - no relay!)
    // Server and orchestrator start in parallel - no waits needed!
    if let Err(e) = orchestrator.start().await {
        tracing::error!("Failed to start sync orchestrator: {}", e);
    } else {
        tracing::info!("✅ P2P sync orchestrator started successfully (pure DHT mesh)");
    }

    // Setup graceful shutdown on SIGTERM/SIGINT
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either shutdown signal or server error
    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("🛑 Received Ctrl+C signal, shutting down gracefully...");
        }
        _ = terminate => {
            tracing::info!("🛑 Received SIGTERM signal, shutting down gracefully...");
        }
        result = server_handle => {
            if let Err(e) = result {
                tracing::error!("Server task failed: {}", e);
            }
        }
    }

    // Graceful shutdown sequence
    tracing::info!("🔄 Sync orchestrator has stopped");
    tracing::info!("🔄 Closing database connections...");
    // RocksDB will flush on drop

    tracing::info!("✅ Shutdown complete");

    Ok(())
}
