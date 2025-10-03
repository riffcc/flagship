mod routes;

use std::env;
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

    // Create the router
    let app = routes::create_router();

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
