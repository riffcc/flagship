use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use super::releases::{Release, ReleasesState};

/// Featured release structure - comprehensive curation and tagging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedRelease {
    /// Unique ID for this featured item
    pub id: String,

    /// ID of the release being featured
    #[serde(rename = "releaseId")]
    pub release_id: String,

    /// Priority/weight for sorting (higher = more prominent)
    pub priority: i32,

    /// Promoted to hero/banner positions
    pub promoted: bool,

    /// Flexible tags for categorization (seasonal, trending, staff-pick, new, etc.)
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
    pub views: u64,
    pub clicks: u64,

    /// A/B testing variant identifier
    pub variant: Option<String>,

    /// Flexible metadata for custom fields
    pub metadata: Option<serde_json::Value>,

    /// Timestamps
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

/// GET /api/v1/featured-releases - Get featured releases
/// Auto-features newest releases with intelligent defaults
pub async fn list_featured_releases(
    State(state): State<ReleasesState>,
) -> impl IntoResponse {
    let releases = state.releases.read().await;

    // Get first 12 releases (or all if less than 12)
    let mut releases_vec: Vec<Release> = releases.values().cloned().collect();

    // Sort by creation date (newest first)
    releases_vec.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Take first 12 and convert to comprehensive FeaturedRelease structures
    let featured: Vec<FeaturedRelease> = releases_vec
        .into_iter()
        .take(12)
        .enumerate()
        .map(|(index, release)| {
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

            FeaturedRelease {
                id: uuid::Uuid::new_v4().to_string(),
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
            }
        })
        .collect();

    tracing::debug!("Returning {} featured releases with intelligent defaults", featured.len());

    Json(featured)
}
