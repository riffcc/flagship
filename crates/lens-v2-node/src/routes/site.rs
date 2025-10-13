//! Site information endpoint
//!
//! Provides information about this Lens Node instance including SiteID, public key,
//! and optional site name.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::site_identity::{SiteIdentity, SiteInfoResponse};

/// State for site info endpoint
#[derive(Clone)]
pub struct SiteState {
    pub identity: Arc<SiteIdentity>,
}

impl SiteState {
    pub fn new(identity: Arc<SiteIdentity>) -> Self {
        Self { identity }
    }
}

/// GET /api/v1/site/info
///
/// Returns information about this Lens Node including:
/// - site_id: Unique identifier for this node (site-XXXXXXXXXXXXXXXX)
/// - public_key: Ed25519 public key in base58 format
/// - site_name: Optional friendly name for this node
/// - version: Node software version
pub async fn get_site_info(
    State(state): State<SiteState>,
) -> Result<Json<SiteInfoResponse>, (StatusCode, String)> {
    // Get current version from Cargo.toml
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let response = state.identity.to_info_response(VERSION);

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use axum::http::Request;
    use axum::Router;
    use axum::routing::get;
    use tempfile::TempDir;
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn test_get_site_info() {
        // Create test database and identity
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();

        let identity = SiteIdentity::initialize(&db, Some("Test Node".to_string()))
            .await
            .unwrap();

        let state = SiteState::new(Arc::new(identity));

        // Create router with endpoint
        let app = Router::new()
            .route("/api/v1/site/info", get(get_site_info))
            .with_state(state);

        // Make request
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/site/info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Parse response
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let info: SiteInfoResponse = serde_json::from_slice(&body).unwrap();

        assert!(info.site_id.starts_with("site-"));
        assert_eq!(info.site_id.len(), 21);
        assert!(!info.public_key.is_empty());
        assert_eq!(info.site_name, Some("Test Node".to_string()));
        assert!(!info.version.is_empty());
    }
}
