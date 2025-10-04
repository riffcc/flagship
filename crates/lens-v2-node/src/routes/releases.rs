use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::account::AccountState;

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
    /// Map of release IDs to releases
    pub releases: Arc<RwLock<HashMap<String, Release>>>,
    /// Map of featured release IDs to featured release data
    pub featured_releases: Arc<RwLock<HashMap<String, super::featured::FeaturedRelease>>>,
    /// Account state for authorization
    pub account_state: AccountState,
}

impl ReleasesState {
    pub fn new(account_state: AccountState) -> Self {
        Self {
            releases: Arc::new(RwLock::new(HashMap::new())),
            featured_releases: Arc::new(RwLock::new(HashMap::new())),
            account_state,
        }
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
    let releases = state.releases.read().await;
    let releases_vec: Vec<Release> = releases.values().cloned().collect();
    Json(releases_vec)
}

/// GET /api/v1/releases/:id - Get a specific release
pub async fn get_release(
    State(state): State<ReleasesState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let releases = state.releases.read().await;

    match releases.get(&id) {
        Some(release) => (StatusCode::OK, Json(release.clone())).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Release not found"
            })),
        )
            .into_response(),
    }
}

/// POST /api/v1/releases - Create a new release
/// Requires upload permission
pub async fn create_release(
    State(state): State<ReleasesState>,
    Json(req): Json<CreateReleaseRequest>,
) -> impl IntoResponse {
    // TODO: Extract public key from Authorization header
    // For now, using a placeholder until we implement proper auth headers
    let public_key = "ed25119p/test_admin_key_12345";

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

    // Store release
    state.releases.write().await.insert(id.clone(), release);

    tracing::info!("Release created: {}", id);

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
    Path(id): Path<String>,
    Json(req): Json<UpdateReleaseRequest>,
) -> impl IntoResponse {
    // TODO: Extract public key from Authorization header
    let public_key = "ed25119p/test_admin_key_12345";

    // Check if release exists
    let mut releases = state.releases.write().await;
    let existing_release = match releases.get(&id) {
        Some(r) => r.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Release not found"
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

    releases.insert(id.clone(), updated_release);

    tracing::info!("Release updated: {}", id);

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
    Path(id): Path<String>,
) -> impl IntoResponse {
    // TODO: Extract public key from Authorization header
    let public_key = "ed25119p/test_admin_key_12345";

    // Check if release exists
    let mut releases = state.releases.write().await;
    let existing_release = match releases.get(&id) {
        Some(r) => r.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Release not found"
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

    releases.remove(&id);

    tracing::info!("Release deleted: {}", id);

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
        let account_state = AccountState::new();
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
