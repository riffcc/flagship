use axum::{
    extract::{Json, Path, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::account::AccountState;
use crate::db::{Database, prefixes, make_key};

/// Tombstone type for deleted releases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TombstoneType {
    /// Temporary deletion - anyone can re-upload
    Temporary,
    /// Soft deletion - only admin/moderator can re-upload
    Soft,
    /// Permanent deletion - CID is blacklisted, nobody can ever re-upload
    Permanent,
}

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

    /// Vector clock for causal ordering (node_id -> counter)
    /// Default to empty map for backward compatibility
    #[serde(rename = "vectorClock", default)]
    pub vector_clock: HashMap<String, u64>,

    /// Tombstone flag - true if this release is deleted
    #[serde(rename = "isTombstone", default)]
    pub is_tombstone: bool,

    /// Tombstone type - determines who can re-upload
    #[serde(rename = "tombstoneType", skip_serializing_if = "Option::is_none")]
    pub tombstone_type: Option<TombstoneType>,

    /// When this release was tombstoned (RFC3339 timestamp)
    #[serde(rename = "deletedAt", skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,

    /// Who tombstoned this release
    #[serde(rename = "deletedBy", skip_serializing_if = "Option::is_none")]
    pub deleted_by: Option<String>,
}

impl Release {
    /// Check if this release happened-before another release
    /// Returns true if self causally precedes other
    pub fn happened_before(&self, other: &Release) -> bool {
        // Self happened before other if:
        // - All of self's counters are <= other's counters
        // - At least one counter is strictly less OR self has fewer entries

        if self.vector_clock.is_empty() && other.vector_clock.is_empty() {
            return false; // Can't determine ordering for releases without vector clocks
        }

        let mut at_least_one_less = false;

        for (node_id, &self_count) in &self.vector_clock {
            let other_count = other.vector_clock.get(node_id).copied().unwrap_or(0);

            if self_count > other_count {
                // Self has a higher counter - not happened-before
                return false;
            }

            if self_count < other_count {
                at_least_one_less = true;
            }
        }

        // Check if other has any nodes we don't have (meaning other is strictly later)
        for node_id in other.vector_clock.keys() {
            if !self.vector_clock.contains_key(node_id) {
                at_least_one_less = true;
                break;
            }
        }

        at_least_one_less
    }

    /// Check if this release is concurrent with another release
    /// Returns true if neither happened-before the other
    pub fn is_concurrent(&self, other: &Release) -> bool {
        !self.happened_before(other) && !other.happened_before(self)
    }

    /// Increment the vector clock for a given node
    /// Call this when modifying a release
    pub fn increment_clock(&mut self, node_id: String) {
        let counter = self.vector_clock.entry(node_id).or_insert(0);
        *counter += 1;
    }

    /// Merge vector clocks from another release (taking maximum of each node's counter)
    /// Call this when receiving a release from another node during sync
    pub fn merge_clock(&mut self, other: &Release) {
        for (node_id, &other_count) in &other.vector_clock {
            let self_count = self.vector_clock.entry(node_id.clone()).or_insert(0);
            *self_count = (*self_count).max(other_count);
        }
    }
}

/// Request to create a new release
#[derive(Debug, Deserialize)]
pub struct CreateReleaseRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
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
    #[serde(rename = "publicKey")]
    pub public_key: String,
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

    /// Check if a CID is permanently tombstoned (blacklisted)
    pub async fn is_cid_permanently_tombstoned(&self, cid: &str) -> bool {
        // Get all tombstones and check if any are permanent for this CID
        match self.db.get_all_with_prefix::<Release>(prefixes::RELEASE) {
            Ok(releases) => {
                releases.iter().any(|r| {
                    r.is_tombstone
                        && r.content_cid == cid
                        && r.tombstone_type == Some(TombstoneType::Permanent)
                })
            }
            Err(_) => false,
        }
    }

    /// Get tombstone by CID (any type)
    pub async fn get_tombstone_by_cid(&self, cid: &str) -> Option<Release> {
        match self.db.get_all_with_prefix::<Release>(prefixes::RELEASE) {
            Ok(releases) => releases
                .into_iter()
                .find(|r| r.is_tombstone && r.content_cid == cid),
            Err(_) => None,
        }
    }
}

/// GET /api/v1/releases - List all releases (excluding tombstones)
pub async fn list_releases(State(state): State<ReleasesState>) -> impl IntoResponse {
    match state.db.get_all_with_prefix::<Release>(prefixes::RELEASE) {
        Ok(releases) => {
            // Filter out tombstoned releases
            let active_releases: Vec<Release> = releases
                .into_iter()
                .filter(|r| !r.is_tombstone)
                .collect();
            Json(active_releases).into_response()
        }
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
    Json(req): Json<CreateReleaseRequest>,
) -> impl IntoResponse {
    // Extract public key from request body
    let public_key = &req.public_key;

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

    // Get node ID from environment or use site_address as fallback
    let node_id = std::env::var("NODE_ID")
        .unwrap_or_else(|_| format!("node-{}", public_key));

    // Check if this CID is permanently tombstoned
    if state.is_cid_permanently_tombstoned(&req.content_cid).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "This CID is permanently blacklisted and cannot be uploaded"
            })),
        )
            .into_response();
    }

    // Check if this CID has a soft tombstone (only admin/moderator can re-upload)
    if let Some(tombstone) = state.get_tombstone_by_cid(&req.content_cid).await {
        if tombstone.tombstone_type == Some(TombstoneType::Soft) {
            // Check if user is admin or moderator
            let is_admin = state.account_state.is_admin(public_key).await;
            let roles = state.account_state.get_roles(public_key).await;
            let is_moderator = roles.iter().any(|r| r == "moderator");

            if !is_admin && !is_moderator {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "error": "This CID has been soft-deleted and can only be re-uploaded by admins or moderators"
                    })),
                )
                    .into_response();
            }
        }
    }

    let mut release = Release {
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
        vector_clock: HashMap::new(),
        is_tombstone: false,
        tombstone_type: None,
        deleted_at: None,
        deleted_by: None,
    };

    // Increment vector clock for this node (initial creation)
    release.increment_clock(node_id);

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
    Path(id): Path<String>,
    Json(req): Json<UpdateReleaseRequest>,
) -> impl IntoResponse {
    // Extract public key from request body
    let public_key = &req.public_key;

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
    let is_owner = existing_release.posted_by == *public_key;

    if !is_admin && !is_owner {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "You do not have permission to update this release"
            })),
        )
            .into_response();
    }

    // Get node ID from environment or use site_address as fallback
    let node_id = std::env::var("NODE_ID")
        .unwrap_or_else(|_| format!("node-{}", public_key));

    // Update release
    let mut updated_release = Release {
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
        vector_clock: existing_release.vector_clock.clone(),
        is_tombstone: existing_release.is_tombstone,
        tombstone_type: existing_release.tombstone_type,
        deleted_at: existing_release.deleted_at,
        deleted_by: existing_release.deleted_by,
    };

    // Increment vector clock for this modification
    updated_release.increment_clock(node_id);

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

/// Delete release request with public key
#[derive(Debug, Deserialize)]
pub struct DeleteReleaseRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

/// DELETE /api/v1/releases/:id - Delete a release (creates temporary tombstone)
/// Requires admin permission or ownership
pub async fn delete_release(
    State(state): State<ReleasesState>,
    Path(id): Path<String>,
    Json(req): Json<DeleteReleaseRequest>,
) -> impl IntoResponse {
    // Extract public key from request body
    let public_key = &req.public_key;

    // Check if release exists
    let key = make_key(prefixes::RELEASE, &id);
    let mut existing_release = match state.db.get::<_, Release>(&key) {
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
    let is_owner = existing_release.posted_by == *public_key;

    if !is_admin && !is_owner {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "You do not have permission to delete this release"
            })),
        )
            .into_response();
    }

    // Get node ID for vector clock
    let node_id = std::env::var("NODE_ID")
        .unwrap_or_else(|_| format!("node-{}", public_key));

    // Mark as temporary tombstone (anyone can re-upload)
    existing_release.is_tombstone = true;
    existing_release.tombstone_type = Some(TombstoneType::Temporary);
    existing_release.deleted_at = Some(chrono::Utc::now().to_rfc3339());
    existing_release.deleted_by = Some(public_key.to_string());
    existing_release.increment_clock(node_id);

    // Save tombstone to database (don't actually delete!)
    if let Err(e) = state.db.put(&key, &existing_release) {
        tracing::error!("Failed to save tombstone for release {}: {}", id, e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to save tombstone"
            })),
        )
            .into_response();
    }

    tracing::info!("Release tombstoned (temporary): {}", id);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "tombstoneType": "temporary"
        })),
    )
        .into_response()
}

/// Tombstone request with public key
#[derive(Debug, Deserialize)]
pub struct TombstoneRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

/// POST /api/v1/admin/tombstone/soft/:id - Create soft tombstone
/// Only admin/moderator can re-upload this CID
/// Requires admin permission
pub async fn create_soft_tombstone(
    State(state): State<ReleasesState>,
    Path(id): Path<String>,
    Json(req): Json<TombstoneRequest>,
) -> impl IntoResponse {
    let public_key = &req.public_key;

    if !state.account_state.is_admin(public_key).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin permission required"})),
        )
            .into_response();
    }

    let key = make_key(prefixes::RELEASE, &id);
    let mut release = match state.db.get::<_, Release>(&key) {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Release not found"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get release: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get release"})),
            )
                .into_response();
        }
    };

    let node_id = std::env::var("NODE_ID")
        .unwrap_or_else(|_| format!("node-{}", public_key));

    release.is_tombstone = true;
    release.tombstone_type = Some(TombstoneType::Soft);
    release.deleted_at = Some(chrono::Utc::now().to_rfc3339());
    release.deleted_by = Some(public_key.to_string());
    release.increment_clock(node_id);

    if let Err(e) = state.db.put(&key, &release) {
        tracing::error!("Failed to save soft tombstone: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to save tombstone"})),
        )
            .into_response();
    }

    tracing::info!("Release soft-tombstoned: {} by admin {}", id, public_key);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "tombstoneType": "soft",
            "cid": release.content_cid
        })),
    )
        .into_response()
}

/// POST /api/v1/admin/tombstone/permanent/:id - Create permanent tombstone + BadBits entry
/// CID is permanently blacklisted, nobody can ever re-upload
/// Requires admin permission
pub async fn create_permanent_tombstone(
    State(state): State<ReleasesState>,
    Path(id): Path<String>,
    Json(req): Json<TombstoneRequest>,
) -> impl IntoResponse {
    let public_key = &req.public_key;

    if !state.account_state.is_admin(public_key).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin permission required"})),
        )
            .into_response();
    }

    let key = make_key(prefixes::RELEASE, &id);
    let mut release = match state.db.get::<_, Release>(&key) {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Release not found"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get release: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to get release"})),
            )
                .into_response();
        }
    };

    let node_id = std::env::var("NODE_ID")
        .unwrap_or_else(|_| format!("node-{}", public_key));

    // Generate BadBits denylist entry (base58btc-encoded multihash)
    let badbits_hash = generate_badbits_hash(&release.content_cid);

    release.is_tombstone = true;
    release.tombstone_type = Some(TombstoneType::Permanent);
    release.deleted_at = Some(chrono::Utc::now().to_rfc3339());
    release.deleted_by = Some(public_key.to_string());
    release.increment_clock(node_id);

    if let Err(e) = state.db.put(&key, &release) {
        tracing::error!("Failed to save permanent tombstone: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to save tombstone"})),
        )
            .into_response();
    }

    // TODO: Add to local BadBits denylist file
    // Format: /ipfs/{badbits_hash}
    // Path: /etc/lens/badbits.deny or ~/.config/lens/badbits.deny

    tracing::warn!(
        "Release permanently tombstoned: {} (CID: {}) by admin {}. BadBits hash: {}",
        id,
        release.content_cid,
        public_key,
        badbits_hash
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "tombstoneType": "permanent",
            "cid": release.content_cid,
            "badbitsHash": badbits_hash,
            "warning": "This CID is now permanently blacklisted"
        })),
    )
        .into_response()
}

/// Generate BadBits denylist hash for a CID
/// Uses base58btc-encoded multihash format per IPFS spec
fn generate_badbits_hash(cid: &str) -> String {
    use sha2::{Sha256, Digest};

    // Hash the CID path
    let path = format!("/ipfs/{}", cid);
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let hash = hasher.finalize();

    // Convert to hex (legacy format) or base58btc (modern format)
    // For now, using hex format for simplicity
    hex::encode(hash)
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
