mod routes;
mod db;
mod sync_orchestrator;
mod block_codec;
mod delete_block;
mod ubts;
mod webrtc_manager;

use routes::{initialize_registry, AppState, RelayState, AccountState, ReleasesState};
use routes::sync::SyncState;
use lens_v2_p2p::{P2pManager, P2pConfig};
use sync_orchestrator::SyncOrchestrator;
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
    eprintln!("=== Lens Node v2 - Version 0.5.5 - Starting ===");

    // Print startup banner
    tracing::info!("╔══════════════════════════════════════════════╗");
    tracing::info!("║       Lens Node v2 - Version 0.5.5          ║");
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

    // Initialize schema registry with built-in schemas
    let registry = Arc::new(initialize_registry());
    tracing::info!("Initialized schema registry");

    // Create application state
    let state = AppState { registry };

    // Initialize WebRTC manager for browser-to-node connections
    let webrtc_manager = Arc::new(webrtc_manager::WebRTCManager::new()?);
    tracing::info!("Initialized WebRTC manager for browser peers");

    // Create relay state for P2P peer discovery with WebRTC support
    let relay_state = RelayState::new().with_webrtc(webrtc_manager.clone());
    tracing::info!("Initialized P2P relay with WebRTC support");

    // Create broadcast channel for immediate WantList updates
    let (block_notify_tx, block_notify_rx) = tokio::sync::mpsc::unbounded_channel();

    // Create account state for authorization (UBTS-based, syncs via SPORE)
    let account_state = AccountState::new(db.clone()).with_notify(block_notify_tx);
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
    let releases_state = ReleasesState::with_db(account_state.clone(), db.clone())?;
    tracing::info!("Initialized releases management with RocksDB persistence");

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

    // Create the router with state
    let app = routes::create_router(state, relay_state, account_state, releases_state, sync_state);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    tracing::info!("To authorize yourself as admin, run:");
    tracing::info!("  curl -X POST http://127.0.0.1:{}/api/v1/admin/authorize \\", port);
    tracing::info!("    -H 'Content-Type: application/json' \\");
    tracing::info!("    -d '{{\"publicKey\": \"YOUR_PUBLIC_KEY_HERE\"}}'");

    // Start sync orchestrator AFTER server is listening
    // If LENS_RELAY_URL is set, use it; otherwise connect to own relay
    let relay_ws_url = env::var("LENS_RELAY_URL")
        .unwrap_or_else(|_| format!("ws://localhost:{}/api/v1/relay/ws", port));
    tracing::info!("Using relay URL: {}", relay_ws_url);

    let orchestrator = Arc::new(SyncOrchestrator::new(
        relay_ws_url,
        p2p_manager.clone(),
        db.clone(),
        block_notify_rx,
    ));

    // Spawn server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Server error: {}", e);
        }
    });

    // Give server a moment to start accepting connections
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Now start the orchestrator (this will connect to relay)
    if let Err(e) = orchestrator.start().await {
        tracing::error!("Failed to start sync orchestrator: {}", e);
    } else {
        tracing::info!("✅ P2P sync orchestrator started successfully");
    }

    // Wait for server to complete (which it never will unless there's an error)
    server_handle.await?;

    Ok(())
}
