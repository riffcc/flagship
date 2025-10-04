pub mod health;
pub mod schemas;
pub mod sync;
pub mod relay;
pub mod account;
pub mod releases;
pub mod import_export;
pub mod categories;
pub mod featured;
pub mod structures;
pub mod persistence;

use axum::{routing::{get, post, put, delete}, Router};
use tower_http::cors::{CorsLayer, Any};

pub use schemas::{initialize_registry, AppState};
pub use relay::RelayState;
pub use account::AccountState;
pub use releases::ReleasesState;

/// Create the main API router with all endpoints
pub fn create_router(state: AppState, relay_state: RelayState, account_state: AccountState, releases_state: ReleasesState) -> Router {
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
        .with_state(relay_state.clone())
        .route("/api/v1/content-categories", get(categories::list_categories))
        .route("/api/v1/structures", get(structures::list_structures))
        .route("/api/v1/structures/:id", get(structures::get_structure))
        .route("/api/v1/account", get(account::get_account))
        .route("/api/v1/account/:public_key", get(account::get_account_status))
        .route("/api/v1/admin/authorize", post(account::authorize_admin))
        .with_state(account_state)
        .route("/api/v1/releases", get(releases::list_releases))
        .route("/api/v1/releases", post(releases::create_release))
        .route("/api/v1/releases/:id", get(releases::get_release))
        .route("/api/v1/releases/:id", put(releases::update_release))
        .route("/api/v1/releases/:id", delete(releases::delete_release))
        .route("/api/v1/featured-releases", get(featured::list_featured_releases))
        .route("/api/v1/import", post(import_export::import_releases))
        .route("/api/v1/export", get(import_export::export_releases))
        .route("/api/v1/admin/releases", delete(import_export::delete_all_releases))
        .with_state(releases_state)
        .layer(cors)
}

#[cfg(test)]
pub fn create_test_app() -> Router {
    use std::sync::Arc;

    let registry = Arc::new(initialize_registry());
    let state = AppState { registry };
    let relay_state = RelayState::new();
    let account_state = AccountState::new();
    let releases_state = ReleasesState::new(account_state.clone());
    create_router(state, relay_state, account_state, releases_state)
}
