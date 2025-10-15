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
pub mod dht;
pub mod site;
pub mod upload;
pub mod map;

use axum::{routing::{get, post, put, delete, patch}, Router};
use tower_http::cors::{CorsLayer, Any};

pub use schemas::{initialize_registry, AppState};
pub use relay::RelayState;
pub use account::AccountState;
pub use releases::ReleasesState;
pub use sync::SyncState;
pub use dht::DHTState;
pub use site::SiteState;
pub use map::MapState;

/// Create the main API router with all endpoints
pub fn create_router(
    state: AppState,
    relay_state: RelayState,
    account_state: AccountState,
    releases_state: ReleasesState,
    sync_state: sync::SyncState,
    dht_state: Option<DHTState>,
    site_state: SiteState,
) -> Router {
    // Configure CORS to allow all origins for development
    // In production, you should restrict this to specific origins
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create separate routers for different states
    let sync_router = Router::new()
        .route("/api/v1/health", get(health::health_check))
        .route("/api/v1/ready", get(sync::ready_handler))
        .with_state(sync_state.clone());

    let schema_router = Router::new()
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
        .with_state(state);

    let relay_router = Router::new()
        .route("/api/v1/relay/ws", get(relay::relay_handler))
        .route("/api/v1/dht/consistency", get(relay::dht_consistency_handler))
        .route("/api/v1/dht/get/:key_hex", get(relay::dht_get_handler))
        .route("/api/v1/dht/put", post(relay::dht_put_handler))
        .route("/api/v1/dht/gossip_slot_ownership", post(relay::gossip_slot_ownership_handler))
        .route("/api/v1/webrtc/offer", post(relay::webrtc_offer_handler))
        .route("/api/v1/webrtc/answer", post(relay::webrtc_answer_handler))
        .route("/api/v1/webrtc/complete", post(relay::webrtc_complete_handler))
        .with_state(relay_state.clone());

    let account_router = Router::new()
        .route("/api/v1/content-categories", get(categories::list_categories))
        .route("/api/v1/structures", get(structures::list_structures))
        .route("/api/v1/structures/:id", get(structures::get_structure))
        .route("/api/v1/account", get(account::get_account))
        .route("/api/v1/account/:public_key", get(account::get_account_status))
        .with_state(account_state);

    let releases_router = Router::new()
        .route("/api/v1/releases", get(releases::list_releases))
        .route("/api/v1/releases", post(releases::create_release))
        .route("/api/v1/releases/:id", get(releases::get_release))
        .route("/api/v1/releases/:id", put(releases::update_release))
        .route("/api/v1/releases/:id", patch(releases::edit_release))
        .route("/api/v1/releases/:id", delete(releases::delete_release))
        .route("/api/v1/featured-releases", get(featured::list_featured_releases))
        .route("/api/v1/admin/featured-releases", post(featured::create_featured_release))
        .route("/api/v1/admin/featured-releases/:id", put(featured::update_featured_release))
        .route("/api/v1/admin/featured-releases/:id", delete(featured::delete_featured_release))
        .route("/api/v1/import", post(import_export::import_releases))
        .route("/api/v1/export", get(import_export::export_releases))
        .route("/api/v1/admin/releases", delete(import_export::delete_all_releases))
        .route("/api/v1/upload/release", post(upload::upload_release))
        .with_state(releases_state);

    let site_router = Router::new()
        .route("/api/v1/site/info", get(site::get_site_info))
        .with_state(site_state);

    let map_router = Router::new()
        .route("/api/v1/map", get(map::get_network_map))
        .with_state(MapState {
            relay: relay_state.clone(),
            sync: sync_state.clone(),
        });

    // Merge all routers
    let mut router = Router::new()
        .merge(sync_router)
        .merge(schema_router)
        .merge(relay_router)
        .merge(account_router)
        .merge(releases_router)
        .merge(site_router)
        .merge(map_router);

    // Add DHT health check endpoint if DHT state is available
    if let Some(dht_state) = dht_state {
        let dht_router = Router::new()
            .route("/api/v1/dht/health", get(dht::dht_health_check))
            .with_state(dht_state);
        router = router.merge(dht_router);
    }

    router.layer(cors)
}

pub fn create_test_app() -> Router {
    use std::sync::Arc;
    use lens_v2_p2p::{P2pManager, P2pConfig};
    use crate::db::Database;
    use crate::site_identity::SiteIdentity;

    let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", uuid::Uuid::new_v4()));
    let db = Database::open(&temp_dir).unwrap();

    let registry = Arc::new(initialize_registry());
    let state = AppState { registry };
    let relay_state = RelayState::new();
    let account_state = AccountState::new(db.clone());
    let releases_state = ReleasesState::new(account_state.clone());
    let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
    let sync_state = SyncState { p2p: p2p_manager };

    // Create site identity for tests
    let identity = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(SiteIdentity::initialize(&db, Some("Test Node".to_string())))
        .unwrap();
    let site_state = SiteState::new(Arc::new(identity));

    create_router(state, relay_state, account_state, releases_state, sync_state, None, site_state)
}
