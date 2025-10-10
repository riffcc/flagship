mod routes;
mod db;
mod sync_orchestrator;
mod block_codec;

use routes::{initialize_registry, AppState, RelayState, AccountState, ReleasesState};
use routes::sync::SyncState;
use lens_v2_p2p::{P2pManager, P2pConfig};
use sync_orchestrator::SyncOrchestrator;
use std::env;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lens_v2_node=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get port from environment or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "5002".to_string())
        .parse::<u16>()?;

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting Lens Node v2 on {}", addr);

    // Initialize RocksDB database
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| ".lens-node-data/rocksdb".to_string());
    let db = db::Database::open(&db_path)?;
    tracing::info!("Initialized RocksDB at: {}", db_path);

    // Initialize schema registry with built-in schemas
    let registry = Arc::new(initialize_registry());
    tracing::info!("Initialized schema registry");

    // Create application state
    let state = AppState { registry };

    // Create relay state for P2P peer discovery
    let relay_state = RelayState::new();
    tracing::info!("Initialized P2P relay");

    // Create account state for authorization
    let account_state = AccountState::new();
    tracing::info!("Initialized account management");

    // Create releases state for content management with RocksDB persistence
    let releases_state = ReleasesState::with_db(account_state.clone(), db.clone())?;
    tracing::info!("Initialized releases management with RocksDB persistence");

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
    ));

    tokio::spawn(async move {
        // Give the server a moment to fully start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        if let Err(e) = orchestrator.start().await {
            tracing::error!("Failed to start sync orchestrator: {}", e);
        } else {
            tracing::info!("Started P2P sync orchestrator");
        }
    });

    axum::serve(listener, app).await?;

    Ok(())
}
