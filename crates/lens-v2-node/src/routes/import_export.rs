use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::releases::{Release, ReleasesState};
use super::account::AccountState;

/// Legacy Lens SDK v1 export format
#[derive(Debug, Deserialize)]
pub struct LegacyExport {
    pub version: String,
    #[serde(rename = "exportDate")]
    pub export_date: String,
    pub releases: Vec<LegacyRelease>,
}

/// Legacy release format from Lens SDK v1
#[derive(Debug, Deserialize)]
pub struct LegacyRelease {
    pub id: String,
    #[serde(rename = "postedBy")]
    pub posted_by: LegacyPublicKey,
    #[serde(rename = "siteAddress")]
    pub site_address: String,
    pub name: String,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    #[serde(rename = "categorySlug")]
    pub category_slug: Option<String>,
    #[serde(rename = "contentCID")]
    pub content_cid: String,
    #[serde(rename = "thumbnailCID")]
    pub thumbnail_cid: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Legacy public key format (byte array object)
#[derive(Debug, Deserialize)]
pub struct LegacyPublicKey {
    #[serde(rename = "publicKey")]
    pub public_key: Option<HashMap<String, u8>>,
}

/// New export format (Lens V2)
#[derive(Debug, Serialize)]
pub struct ExportData {
    pub version: String,
    pub export_date: String,
    pub releases: Vec<Release>,
}

/// Import response
#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub success: bool,
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

/// Convert legacy public key bytes to ed25519 format
fn convert_legacy_public_key(legacy_key: &LegacyPublicKey) -> String {
    if let Some(key_map) = &legacy_key.public_key {
        // Extract bytes in order
        let mut bytes = Vec::new();
        for i in 0..32 {
            if let Some(&byte) = key_map.get(&i.to_string()) {
                bytes.push(byte);
            }
        }

        // Convert to hex and format as ed25519p
        let hex = bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        format!("ed25119p/{}", hex)
    } else {
        // Fallback for unknown format
        "ed25119p/unknown".to_string()
    }
}

/// POST /api/v1/import - Import releases from legacy format
pub async fn import_releases(
    State(state): State<ReleasesState>,
    Json(import_data): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut imported = 0;
    let mut skipped = 0;
    let mut errors = Vec::new();

    // Try to parse as legacy format
    match serde_json::from_value::<LegacyExport>(import_data.clone()) {
        Ok(legacy_export) => {
            tracing::info!("Importing {} releases from legacy format v{}",
                legacy_export.releases.len(), legacy_export.version);

            let mut releases_lock = state.releases.write().await;

            for legacy_release in legacy_export.releases {
                // Check if release already exists
                if releases_lock.contains_key(&legacy_release.id) {
                    tracing::debug!("Skipping existing release: {}", legacy_release.id);
                    skipped += 1;
                    continue;
                }

                // Convert to new format
                let posted_by = convert_legacy_public_key(&legacy_release.posted_by);

                // Use categorySlug as the new categoryId for compatibility with frontend
                let category_slug = legacy_release.category_slug.unwrap_or_else(|| "unknown".to_string());

                let release = Release {
                    id: legacy_release.id.clone(),
                    name: legacy_release.name,
                    category_id: category_slug.clone(), // Use slug as ID for frontend compatibility
                    category_slug,
                    content_cid: legacy_release.content_cid,
                    thumbnail_cid: legacy_release.thumbnail_cid,
                    metadata: legacy_release.metadata,
                    site_address: legacy_release.site_address,
                    posted_by,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                releases_lock.insert(legacy_release.id.clone(), release);
                imported += 1;
            }

            tracing::info!("Import complete: {} imported, {} skipped", imported, skipped);
        }
        Err(e) => {
            let error_msg = format!("Failed to parse import data: {}", e);
            tracing::error!("{}", error_msg);
            errors.push(error_msg);
        }
    }

    (
        StatusCode::OK,
        Json(ImportResponse {
            success: errors.is_empty(),
            imported,
            skipped,
            errors,
        }),
    )
}

/// GET /api/v1/export - Export all releases in new format
pub async fn export_releases(
    State(state): State<ReleasesState>,
) -> impl IntoResponse {
    let releases = state.releases.read().await;
    let releases_vec: Vec<Release> = releases.values().cloned().collect();

    let export_data = ExportData {
        version: "2.0".to_string(),
        export_date: chrono::Utc::now().to_rfc3339(),
        releases: releases_vec,
    };

    tracing::info!("Exporting {} releases", export_data.releases.len());

    Json(export_data)
}

/// DELETE /api/v1/releases - Delete ALL releases (use with caution!)
/// Requires admin permission
pub async fn delete_all_releases(
    State(state): State<ReleasesState>,
) -> impl IntoResponse {
    // TODO: Add admin check here
    let mut releases = state.releases.write().await;
    let count = releases.len();
    releases.clear();

    tracing::warn!("Deleted all {} releases", count);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "deleted": count
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_legacy_public_key() {
        let mut key_map = HashMap::new();
        for i in 0..32 {
            key_map.insert(i.to_string(), i as u8);
        }

        let legacy_key = LegacyPublicKey {
            public_key: Some(key_map),
        };

        let result = convert_legacy_public_key(&legacy_key);
        assert!(result.starts_with("ed25119p/"));
        assert_eq!(result.len(), 9 + 64); // prefix + 32 bytes in hex
    }
}
