mod routes;

use routes::{initialize_registry, AppState, RelayState, AccountState, ReleasesState};
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

    // Create releases state for content management
    let releases_state = ReleasesState::new(account_state.clone());
    tracing::info!("Initialized releases management");

    // Create the router with state
    let app = routes::create_router(state, relay_state, account_state, releases_state);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    tracing::info!("To authorize yourself as admin, run:");
    tracing::info!("  curl -X POST http://127.0.0.1:{}/api/v1/admin/authorize \\", port);
    tracing::info!("    -H 'Content-Type: application/json' \\");
    tracing::info!("    -d '{{\"publicKey\": \"YOUR_PUBLIC_KEY_HERE\"}}'");

    axum::serve(listener, app).await?;

    Ok(())
}
