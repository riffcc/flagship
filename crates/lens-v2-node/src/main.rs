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
    eprintln!("=== Lens Node v2 - Version 0.6.4 - Starting ===");

    // Print startup banner
    tracing::info!("╔══════════════════════════════════════════════╗");
    tracing::info!("║       Lens Node v2 - Version 0.6.4          ║");
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
    let dht_storage = Arc::new(tokio::sync::Mutex::new(citadel_dht::local_storage::LocalStorage::new()));
    tracing::info!("Initialized shared DHT storage for Citadel hexagonal mesh");

    // Create relay state for P2P peer discovery with WebRTC support and shared DHT
    let relay_state = RelayState::new()
        .with_webrtc(webrtc_manager.clone())
        .with_dht_storage(dht_storage.clone());
    tracing::info!("Initialized P2P relay with WebRTC support and shared DHT storage");

    // Generate peer_id for this node (in production, load from disk or generate once)
    // Using random u64 for unique peer_id across restarts
    // The hello protocol ensures relay uses our peer_id instead of generating its own
    let my_peer_id = format!("peer-{}", rand::random::<u64>());
    let mesh_config = peer_registry::default_mesh_config();
    let my_slot = peer_registry::peer_id_to_slot(&my_peer_id, &mesh_config);

    tracing::info!("🎯 My peer ID: {}", my_peer_id);
    tracing::info!("🎯 My slot: {:?} in mesh {}×{}×{}",
        my_slot, mesh_config.width, mesh_config.height, mesh_config.depth);

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

    // Create P2P manager for sync status tracking
    let p2p_config = P2pConfig::default();
    let p2p_manager = Arc::new(P2pManager::new(p2p_config));
    let sync_state = SyncState { p2p: p2p_manager.clone() };
    tracing::info!("Initialized P2P sync manager");

    // Initialize site identity (SiteID and SiteKey for defederation)
    let site_name = env::var("SITE_NAME").ok();
    let identity = SiteIdentity::initialize(&db, site_name.clone()).await?;
    tracing::info!("🆔 Site ID: {}", identity.site_id);
    tracing::info!("🔑 Site Public Key: {}", identity.site_key.public_key_base58());
    if let Some(ref name) = identity.site_name {
        tracing::info!("📛 Site Name: {}", name);
    }
    let site_state = SiteState::new(Arc::new(identity));

    // Create the router with state
    // Note: DHT state is None for now as we're using RocksDB for persistence
    // When DHT storage is actively used, we'll pass Some(dht_state) here
    let app = routes::create_router(state, relay_state, account_state, releases_state, sync_state, None, site_state);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    tracing::info!("To authorize yourself as admin, run:");
    tracing::info!("  curl -X POST http://127.0.0.1:{}/api/v1/admin/authorize \\", port);
    tracing::info!("    -H 'Content-Type: application/json' \\");
    tracing::info!("    -d '{{\"publicKey\": \"YOUR_PUBLIC_KEY_HERE\"}}'");

    // Announce our slot ownership to DHT (1 message - no relay needed!)
    tracing::info!("📢 Announcing slot ownership to DHT mesh...");
    let ownership = peer_registry::SlotOwnership::new(
        my_peer_id.clone(),
        my_slot,
        None, // No relay URL - we're pure DHT now!
    );
    let ownership_key = peer_registry::slot_ownership_key(my_slot);
    let ownership_bytes = serde_json::to_vec(&ownership)?;
    {
        let mut dht = dht_storage.lock().await;
        dht.put(ownership_key, ownership_bytes.into());
    }
    tracing::info!("✅ Announced slot ownership: {:?} -> {}", my_slot, my_peer_id);

    // Create orchestrator with LazyNode (NO RELAY!)
    // Note: relay_url is still required for now but will be removed in parallel task
    let relay_url = env::var("LENS_RELAY_URL")
        .unwrap_or_else(|_| format!("ws://localhost:{}/api/v1/relay/ws", port));

    let orchestrator = Arc::new(SyncOrchestrator::new(
        relay_url,
        my_peer_id.clone(),
        my_slot,
        mesh_config,
        p2p_manager.clone(),
        db.clone(),
        block_notify_rx,
        dht_storage.clone(),
    ));

    // Spawn server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Server error: {}", e);
        }
    });

    // Give server a moment to start accepting connections
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Now start the orchestrator (pure DHT mesh - no relay!)
    if let Err(e) = orchestrator.start().await {
        tracing::error!("Failed to start sync orchestrator: {}", e);
    } else {
        tracing::info!("✅ P2P sync orchestrator started successfully (pure DHT mesh)");
    }

    // Wait for server to complete (which it never will unless there's an error)
    server_handle.await?;

    Ok(())
}
