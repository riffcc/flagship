use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use super::releases::{Release, ReleasesState};
use crate::db::{prefixes, make_key};

/// Default timestamp for backward compatibility
fn default_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Featured release structure - comprehensive curation and tagging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedRelease {
    /// Unique ID for this featured item
    pub id: String,

    /// ID of the release being featured
    #[serde(rename = "releaseId")]
    pub release_id: String,

    /// Priority/weight for sorting (higher = more prominent)
    #[serde(default)]
    pub priority: i32,

    /// Promoted to hero/banner positions
    #[serde(default)]
    pub promoted: bool,

    /// Flexible tags for categorization (seasonal, trending, staff-pick, new, etc.)
    #[serde(default)]
    pub tags: Vec<String>,

    /// Visibility scheduling
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    #[serde(rename = "endTime")]
    pub end_time: Option<String>,

    /// Display overrides for featured context
    #[serde(rename = "customTitle")]
    pub custom_title: Option<String>,
    #[serde(rename = "customDescription")]
    pub custom_description: Option<String>,
    #[serde(rename = "customThumbnail")]
    pub custom_thumbnail: Option<String>,

    /// Targeting filters
    pub regions: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,

    /// Analytics tracking
    #[serde(default)]
    pub views: u64,
    #[serde(default)]
    pub clicks: u64,

    /// A/B testing variant identifier
    pub variant: Option<String>,

    /// Flexible metadata for custom fields
    pub metadata: Option<serde_json::Value>,

    /// Timestamps
    #[serde(rename = "createdAt", alias = "created_at", default = "default_timestamp")]
    pub created_at: String,
    #[serde(rename = "updatedAt", alias = "updated_at")]
    pub updated_at: Option<String>,
}

/// GET /api/v1/featured-releases - Get featured releases
/// Auto-features newest releases with intelligent defaults
pub async fn list_featured_releases(
    State(state): State<ReleasesState>,
) -> impl IntoResponse {
    // Check if we have persisted featured releases
    let featured_count = state.db.count_prefix(prefixes::FEATURED.as_bytes()).unwrap_or(0);

    // If the store is empty, auto-populate it with the newest releases
    if featured_count == 0 {
        let releases_vec_result = state.db.get_all_with_prefix::<Release>(prefixes::RELEASE);

        // Get first 12 releases (or all if less than 12)
        let mut releases_vec = match releases_vec_result {
            Ok(vec) => vec,
            Err(e) => {
                tracing::error!("Failed to get releases: {}", e);
                return Json(vec![]);
            }
        };

        // Sort by creation date (newest first)
        releases_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Take first 12 and convert to comprehensive FeaturedRelease structures
        for (index, release) in releases_vec.into_iter().take(12).enumerate() {
            // Auto-assign priority based on recency (12 = newest, 1 = oldest)
            let priority = (12 - index) as i32 * 10;

            // Auto-tag based on category
            let mut tags = vec![release.category_slug.clone()];
            if index < 3 {
                tags.push("new".to_string());
            }

            // Set intelligent defaults: feature from now until 30 days from now
            let now = chrono::Utc::now();
            let thirty_days_from_now = now + chrono::Duration::days(30);

            let featured_id = uuid::Uuid::new_v4().to_string();
            let featured_release = FeaturedRelease {
                id: featured_id.clone(),
                release_id: release.id,
                priority,
                promoted: index == 0, // Promote the newest
                tags,
                start_time: Some(now.to_rfc3339()),
                end_time: Some(thirty_days_from_now.to_rfc3339()),
                custom_title: None,
                custom_description: None,
                custom_thumbnail: None,
                regions: None,
                languages: None,
                views: 0,
                clicks: 0,
                variant: None,
                metadata: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: None,
            };

            let key = make_key(prefixes::FEATURED, &featured_id);
            if let Err(e) = state.db.put(&key, &featured_release) {
                tracing::error!("Failed to save featured release: {}", e);
            }
        }

        tracing::debug!("Auto-populated featured releases with intelligent defaults");
    }

    // Return the featured releases from the database
    match state.db.get_all_with_prefix::<FeaturedRelease>(prefixes::FEATURED) {
        Ok(featured) => Json(featured),
        Err(e) => {
            tracing::error!("Failed to get featured releases: {}", e);
            Json(vec![])
        }
    }
}

/// Request to update a featured release
#[derive(Debug, Deserialize)]
pub struct UpdateFeaturedReleaseRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub id: String,
    #[serde(rename = "releaseId")]
    pub release_id: Option<String>,
    pub priority: Option<i32>,
    pub promoted: Option<bool>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    #[serde(rename = "endTime")]
    pub end_time: Option<String>,
    #[serde(rename = "customTitle")]
    pub custom_title: Option<String>,
    #[serde(rename = "customDescription")]
    pub custom_description: Option<String>,
    #[serde(rename = "customThumbnail")]
    pub custom_thumbnail: Option<String>,
    pub regions: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub variant: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Request to create a new featured release
#[derive(Debug, Deserialize)]
pub struct CreateFeaturedReleaseRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "releaseId")]
    pub release_id: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub promoted: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    #[serde(rename = "endTime")]
    pub end_time: Option<String>,
    #[serde(rename = "customTitle")]
    pub custom_title: Option<String>,
    #[serde(rename = "customDescription")]
    pub custom_description: Option<String>,
    #[serde(rename = "customThumbnail")]
    pub custom_thumbnail: Option<String>,
    pub regions: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
    pub variant: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Response for successful operations
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub id: String,
}

/// POST /api/v1/admin/featured-releases - Create a new featured release
pub async fn create_featured_release(
    State(state): State<ReleasesState>,
    Json(req): Json<CreateFeaturedReleaseRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, String)> {
    // Check if requester is admin
    if !state.account_state.is_admin(&req.public_key).await {
        return Err((StatusCode::FORBIDDEN, "Admin permission required".to_string()));
    }

    // Verify the release exists
    let release_key = make_key(prefixes::RELEASE, &req.release_id);
    state.db.get::<_, Release>(&release_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Release {} not found", req.release_id)))?;

    // Generate a new ID for the featured release
    let featured_id = uuid::Uuid::new_v4().to_string();

    // Create the featured release
    let featured_release = FeaturedRelease {
        id: featured_id.clone(),
        release_id: req.release_id,
        priority: req.priority,
        promoted: req.promoted,
        tags: req.tags,
        start_time: req.start_time,
        end_time: req.end_time,
        custom_title: req.custom_title,
        custom_description: req.custom_description,
        custom_thumbnail: req.custom_thumbnail,
        regions: req.regions,
        languages: req.languages,
        views: 0,
        clicks: 0,
        variant: req.variant,
        metadata: req.metadata,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: None,
    };

    let key = make_key(prefixes::FEATURED, &featured_id);
    state.db.put(&key, &featured_release)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save: {}", e)))?;

    tracing::info!("Created featured release {}", featured_id);

    Ok(Json(SuccessResponse { id: featured_id }))
}

/// PUT /api/v1/admin/featured-releases/:id - Update a featured release
pub async fn update_featured_release(
    Path(id): Path<String>,
    State(state): State<ReleasesState>,
    Json(req): Json<UpdateFeaturedReleaseRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, String)> {
    // Check if requester is admin
    if !state.account_state.is_admin(&req.public_key).await {
        return Err((StatusCode::FORBIDDEN, "Admin permission required".to_string()));
    }

    let key = make_key(prefixes::FEATURED, &id);

    // Check if the featured release exists
    let existing = state.db.get::<_, FeaturedRelease>(&key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Featured release {} not found", id)))?;

    // Update the featured release with provided fields
    let updated = FeaturedRelease {
        id: existing.id.clone(),
        release_id: req.release_id.unwrap_or(existing.release_id),
        priority: req.priority.unwrap_or(existing.priority),
        promoted: req.promoted.unwrap_or(existing.promoted),
        tags: req.tags.unwrap_or(existing.tags),
        start_time: req.start_time.or(existing.start_time),
        end_time: req.end_time.or(existing.end_time),
        custom_title: req.custom_title.or(existing.custom_title),
        custom_description: req.custom_description.or(existing.custom_description),
        custom_thumbnail: req.custom_thumbnail.or(existing.custom_thumbnail),
        regions: req.regions.or(existing.regions),
        languages: req.languages.or(existing.languages),
        views: existing.views,
        clicks: existing.clicks,
        variant: req.variant.or(existing.variant),
        metadata: req.metadata.or(existing.metadata),
        created_at: existing.created_at,
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    state.db.put(&key, &updated)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save: {}", e)))?;

    tracing::info!("Updated featured release {}", id);

    Ok(Json(SuccessResponse { id }))
}

/// DELETE /api/v1/admin/featured-releases/:id - Delete a featured release
/// Returns 200 OK even if the item doesn't exist (idempotent delete)
pub async fn delete_featured_release(
    Path(id): Path<String>,
    State(state): State<ReleasesState>,
) -> Result<Json<SuccessResponse>, (StatusCode, String)> {
    let key = make_key(prefixes::FEATURED, &id);

    // Check if the featured release exists
    match state.db.get::<_, FeaturedRelease>(&key) {
        Ok(Some(_)) => {
            // Delete the featured release
            state.db.delete(&key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete: {}", e)))?;
            tracing::info!("Deleted featured release {}", id);
        }
        Ok(None) => {
            // Already deleted - this is fine, return success (idempotent)
            tracing::debug!("Featured release {} already deleted", id);
        }
        Err(e) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)));
        }
    }

    Ok(Json(SuccessResponse { id }))
}
