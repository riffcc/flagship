pub mod health;
pub mod schemas;
pub mod sync;
pub mod relay;

use axum::{routing::get, Router};
use tower_http::cors::{CorsLayer, Any};

pub use schemas::{initialize_registry, AppState};
pub use relay::RelayState;

/// Create the main API router with all endpoints
pub fn create_router(state: AppState, relay_state: RelayState) -> Router {
    // Configure CORS to allow all origins for development
    // In production, you should restrict this to specific origins
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/health", get(health::health_check))
        .route("/api/v1/schemas", get(schemas::list_schemas))
        .route(
            "/api/v1/schemas/:schema_name",
            get(schemas::get_latest_schema),
        )
        .route(
            "/api/v1/schemas/:schema_name/versions",
            get(schemas::get_schema_versions),
        )
        .route(
            "/api/v1/schemas/:schema_name/versions/:version",
            get(schemas::get_schema),
        )
        .with_state(state)
        .route("/api/v1/relay/ws", get(relay::relay_handler))
        .with_state(relay_state)
        .layer(cors)
}

#[cfg(test)]
pub fn create_test_app() -> Router {
    use std::sync::Arc;

    let registry = Arc::new(initialize_registry());
    let state = AppState { registry };
    let relay_state = RelayState::new();
    create_router(state, relay_state)
}
