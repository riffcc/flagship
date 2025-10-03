pub mod health;

use axum::{routing::get, Router};

/// Create the main API router with all endpoints
pub fn create_router() -> Router {
    Router::new()
        .route("/api/v1/health", get(health::health_check))
}

#[cfg(test)]
pub fn create_test_app() -> Router {
    create_router()
}
