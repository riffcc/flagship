mod routes;

use routes::{initialize_registry, AppState};
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

    // Create the router with state
    let app = routes::create_router(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
