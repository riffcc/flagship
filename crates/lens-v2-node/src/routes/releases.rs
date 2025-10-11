use axum::{
    extract::{Json, Path, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::account::AccountState;
use crate::db::{Database, prefixes, make_key};

/// Release data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub name: String,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    #[serde(rename = "categorySlug")]
    pub category_slug: String,
    #[serde(rename = "contentCID")]
    pub content_cid: String,
    #[serde(rename = "thumbnailCID")]
    pub thumbnail_cid: Option<String>,
    pub metadata: Option<serde_json::Value>,
    #[serde(rename = "siteAddress")]
    pub site_address: String,
    #[serde(rename = "postedBy")]
    pub posted_by: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Request to create a new release
#[derive(Debug, Deserialize)]
pub struct CreateReleaseRequest {
    pub name: String,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    #[serde(rename = "categorySlug", default = "default_category_slug")]
    pub category_slug: String,
    #[serde(rename = "contentCID")]
    pub content_cid: String,
    #[serde(rename = "thumbnailCID")]
    pub thumbnail_cid: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

fn default_category_slug() -> String {
    "unknown".to_string()
}

/// Request to update a release
#[derive(Debug, Deserialize)]
pub struct UpdateReleaseRequest {
    pub name: String,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    #[serde(rename = "contentCID")]
    pub content_cid: String,
    #[serde(rename = "thumbnailCID")]
    pub thumbnail_cid: Option<String>,
    pub metadata: Option<serde_json::Value>,
    #[serde(rename = "siteAddress")]
    pub site_address: String,
    #[serde(rename = "postedBy")]
    pub posted_by: String,
}

/// Response for successful operations
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub id: String,
}

/// Releases state shared across handlers
#[derive(Clone)]
pub struct ReleasesState {
    /// RocksDB database for persistent storage
    pub db: Database,
    /// Account state for authorization
    pub account_state: AccountState,
}

impl ReleasesState {
    /// Create new ReleasesState without database (for testing)
    pub fn new(account_state: AccountState) -> Self {
        // This creates a temporary in-memory database for testing
        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", uuid::Uuid::new_v4()));
        let db = Database::open(&temp_dir).expect("Failed to create test database");
        Self {
            db,
            account_state,
        }
    }

    /// Create new ReleasesState with database and load existing releases
    pub fn with_db(account_state: AccountState, db: Database) -> anyhow::Result<Self> {
        let state = Self {
            db,
            account_state,
        };

        // Load existing releases count
        if let Ok(count) = state.db.count_prefix(prefixes::RELEASE.as_bytes()) {
            tracing::info!("Loaded {} releases from RocksDB", count);
        }

        if let Ok(count) = state.db.count_prefix(prefixes::FEATURED.as_bytes()) {
            tracing::info!("Loaded {} featured releases from RocksDB", count);
        }

        Ok(state)
    }

    /// Check if a public key has upload permission
    pub async fn can_upload(&self, public_key: &str) -> bool {
        self.account_state.is_admin(public_key).await
            || self
                .account_state
                .get_roles(public_key)
                .await
                .iter()
                .any(|r| r == "creator" || r == "moderator")
    }
}

/// GET /api/v1/releases - List all releases
pub async fn list_releases(State(state): State<ReleasesState>) -> impl IntoResponse {
    match state.db.get_all_with_prefix::<Release>(prefixes::RELEASE) {
        Ok(releases) => Json(releases).into_response(),
        Err(e) => {
            tracing::error!("Failed to list releases: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to list releases"
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/releases/:id - Get a specific release
pub async fn get_release(
    State(state): State<ReleasesState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let key = make_key(prefixes::RELEASE, &id);

    match state.db.get::<_, Release>(&key) {
        Ok(Some(release)) => (StatusCode::OK, Json(release)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Release not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get release {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get release"
                })),
            )
                .into_response()
        }
    }
}

/// POST /api/v1/releases - Create a new release
/// Requires upload permission
pub async fn create_release(
    State(state): State<ReleasesState>,
    headers: HeaderMap,
    Json(req): Json<CreateReleaseRequest>,
) -> impl IntoResponse {
    // Extract public key from X-Public-Key header
    let public_key = match headers.get("X-Public-Key") {
        Some(key) => key.to_str().unwrap_or(""),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Missing X-Public-Key header"
                })),
            )
                .into_response();
        }
    };

    // Check permissions
    if !state.can_upload(public_key).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "You do not have permission to upload releases"
            })),
        )
            .into_response();
    }

    // Generate new release ID
    let id = Uuid::new_v4().to_string();

    let release = Release {
        id: id.clone(),
        name: req.name,
        category_id: req.category_id,
        category_slug: req.category_slug,
        content_cid: req.content_cid,
        thumbnail_cid: req.thumbnail_cid,
        metadata: req.metadata,
        site_address: "local".to_string(), // TODO: Get from config
        posted_by: public_key.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Store release in RocksDB
    let key = make_key(prefixes::RELEASE, &id);
    if let Err(e) = state.db.put(&key, &release) {
        tracing::error!("Failed to save release {}: {}", id, e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to save release"
            })),
        )
            .into_response();
    }

    tracing::info!("Release created and saved: {}", id);

    (
        StatusCode::CREATED,
        Json(SuccessResponse {
            success: true,
            id,
        }),
    )
        .into_response()
}

/// PUT /api/v1/releases/:id - Update a release
/// Requires admin permission or ownership
pub async fn update_release(
    State(state): State<ReleasesState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateReleaseRequest>,
) -> impl IntoResponse {
    // Extract public key from X-Public-Key header
    let public_key = match headers.get("X-Public-Key") {
        Some(key) => key.to_str().unwrap_or(""),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Missing X-Public-Key header"
                })),
            )
                .into_response();
        }
    };

    // Check if release exists
    let key = make_key(prefixes::RELEASE, &id);
    let existing_release = match state.db.get::<_, Release>(&key) {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Release not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get release {}: {}", id, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get release"
                })),
            )
                .into_response();
        }
    };

    // Check permissions - must be admin or original poster
    let is_admin = state.account_state.is_admin(public_key).await;
    let is_owner = existing_release.posted_by == public_key;

    if !is_admin && !is_owner {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "You do not have permission to update this release"
            })),
        )
            .into_response();
    }

    // Update release
    let updated_release = Release {
        id: id.clone(),
        name: req.name,
        category_id: req.category_id,
        category_slug: existing_release.category_slug, // Keep existing category_slug
        content_cid: req.content_cid,
        thumbnail_cid: req.thumbnail_cid,
        metadata: req.metadata,
        site_address: req.site_address,
        posted_by: req.posted_by,
        created_at: existing_release.created_at,
    };

    // Save updated release to RocksDB
    if let Err(e) = state.db.put(&key, &updated_release) {
        tracing::error!("Failed to update release {}: {}", id, e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to update release"
            })),
        )
            .into_response();
    }

    tracing::info!("Release updated and saved: {}", id);

    (
        StatusCode::OK,
        Json(SuccessResponse {
            success: true,
            id,
        }),
    )
        .into_response()
}

/// DELETE /api/v1/releases/:id - Delete a release
/// Requires admin permission or ownership
pub async fn delete_release(
    State(state): State<ReleasesState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Extract public key from X-Public-Key header
    let public_key = match headers.get("X-Public-Key") {
        Some(key) => key.to_str().unwrap_or(""),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Missing X-Public-Key header"
                })),
            )
                .into_response();
        }
    };

    // Check if release exists
    let key = make_key(prefixes::RELEASE, &id);
    let existing_release = match state.db.get::<_, Release>(&key) {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Release not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get release {}: {}", id, e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to get release"
                })),
            )
                .into_response();
        }
    };

    // Check permissions - must be admin or original poster
    let is_admin = state.account_state.is_admin(public_key).await;
    let is_owner = existing_release.posted_by == public_key;

    if !is_admin && !is_owner {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "You do not have permission to delete this release"
            })),
        )
            .into_response();
    }

    // Create and save delete transaction block for P2P sync
    use crate::ubts::{UBTSBlock, UBTSTransaction};

    let delete_tx = UBTSTransaction::DeleteRelease {
        id: id.clone(),
        signature: Some(public_key.to_string()),
    };

    let ubts_block = UBTSBlock::new(0, None, vec![delete_tx]);

    // Save delete transaction to database for SPORE sync
    let delete_key = make_key(prefixes::DELETE_TRANSACTION, &ubts_block.id);
    if let Err(e) = state.db.put(&delete_key, &ubts_block) {
        tracing::error!("Failed to save delete transaction {}: {}", ubts_block.id, e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to save delete transaction"
            })),
        )
            .into_response();
    }

    tracing::info!("Delete transaction saved: {} for release {}", ubts_block.id, id);

    // Delete from RocksDB
    if let Err(e) = state.db.delete(&key) {
        tracing::error!("Failed to delete release {}: {}", id, e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to delete release"
            })),
        )
            .into_response();
    }

    tracing::info!("Release deleted from database: {}", id);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_can_upload() {
        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", uuid::Uuid::new_v4()));
        let db = crate::db::Database::open(&temp_dir).unwrap();
        let account_state = AccountState::new(db.clone());
        let state = ReleasesState::new(account_state.clone());

        // Initially cannot upload
        assert!(!state.can_upload("test_key").await);

        // After authorization, can upload
        account_state
            .authorize_admin("test_key".to_string())
            .await;
        assert!(state.can_upload("test_key").await);
    }
}
