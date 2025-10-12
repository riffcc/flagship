use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::releases::{Release, ReleasesState};
use crate::db::{prefixes, make_key};

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

            for legacy_release in legacy_export.releases {
                let key = make_key(prefixes::RELEASE, &legacy_release.id);

                // Check if release already exists
                if let Ok(true) = state.db.exists(&key) {
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
                    vector_clock: std::collections::HashMap::new(),  // Imported releases start with empty vector clock
                    is_tombstone: false,
                    tombstone_type: None,
                    deleted_at: None,
                    deleted_by: None,
                };

                if let Err(e) = state.db.put(&key, &release) {
                    errors.push(format!("Failed to save release {}: {}", legacy_release.id, e));
                } else {
                    imported += 1;
                }
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
    match state.db.get_all_with_prefix::<Release>(prefixes::RELEASE) {
        Ok(releases) => {
            let export_data = ExportData {
                version: "2.0".to_string(),
                export_date: chrono::Utc::now().to_rfc3339(),
                releases,
            };

            tracing::info!("Exporting {} releases", export_data.releases.len());
            Json(export_data).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to export releases: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to export releases"
                })),
            )
                .into_response()
        }
    }
}

/// DELETE /api/v1/releases - Delete ALL releases (use with caution!)
/// Requires admin permission
pub async fn delete_all_releases(
    State(state): State<ReleasesState>,
) -> impl IntoResponse {
    use crate::ubts::{UBTSBlock, UBTSTransaction};

    // TODO: Add admin check here
    match state.db.iter_prefix::<Release>(prefixes::RELEASE.as_bytes()) {
        Ok(items) => {
            let count = items.len();

            // Collect all release IDs
            let release_ids: Vec<String> = items.iter()
                .filter_map(|(_, release)| Some(release.id.clone()))
                .collect();

            if release_ids.is_empty() {
                tracing::info!("No releases to delete");
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "deleted": 0
                    })),
                )
                    .into_response();
            }

            // Create a single UBTS block with all delete transactions
            let delete_transactions: Vec<UBTSTransaction> = release_ids.iter()
                .map(|id| UBTSTransaction::DeleteRelease {
                    id: id.clone(),
                    signature: Some("system".to_string()), // System-initiated bulk delete
                })
                .collect();

            let ubts_block = UBTSBlock::new(0, None, delete_transactions);

            // Save the delete transaction block for SPORE sync
            let delete_key = make_key(prefixes::DELETE_TRANSACTION, &ubts_block.id);
            if let Err(e) = state.db.put(&delete_key, &ubts_block) {
                tracing::error!("Failed to save bulk delete transaction {}: {}", ubts_block.id, e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Failed to save delete transaction"
                    })),
                )
                    .into_response();
            }

            tracing::info!("Created bulk delete transaction {} for {} releases", ubts_block.id, release_ids.len());

            // Convert all releases to temporary tombstones (proof of erasure)
            let mut tombstoned = 0;
            for (key, mut release) in items {
                // Convert to temporary tombstone
                release.is_tombstone = true;
                release.tombstone_type = Some(crate::routes::releases::TombstoneType::Temporary);
                release.deleted_at = Some(chrono::Utc::now().to_rfc3339());
                release.deleted_by = Some("bulk-delete".to_string());

                // Increment vector clock for this delete operation
                release.increment_clock("bulk-delete".to_string());

                // Save tombstone (proof of erasure - content deleted but metadata remains)
                if let Err(e) = state.db.put(&key, &release) {
                    tracing::error!("Failed to create tombstone for release {}: {}", key, e);
                } else {
                    tombstoned += 1;
                }
            }

            tracing::warn!("Created {} temporary tombstones (proof of erasure for all releases)", tombstoned);

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "deleted": count,
                    "delete_transaction_id": ubts_block.id
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to delete releases: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to delete releases"
                })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::account::AccountState;
    use crate::db::Database;
    use uuid::Uuid;

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

    #[tokio::test]
    async fn test_bulk_delete_creates_single_ubts_block() {
        use crate::ubts::UBTSBlock;

        let temp_dir = std::env::temp_dir().join(format!("lens-test-{}", Uuid::new_v4()));
        let db = Database::open(&temp_dir).unwrap();
        let account_state = AccountState::new(db.clone());
        let state = ReleasesState::with_db(account_state, db.clone()).unwrap();

        // Create multiple test releases
        for i in 0..5 {
            let release = Release {
                id: format!("test-release-{}", i),
                name: format!("Test Release {}", i),
                category_id: "test".to_string(),
                category_slug: "test".to_string(),
                content_cid: format!("QmTest{}", i),
                thumbnail_cid: None,
                metadata: None,
                site_address: "local".to_string(),
                posted_by: "test-user".to_string(),
                created_at: "2025-01-01T00:00:00Z".to_string(),
                vector_clock: std::collections::HashMap::new(),
                is_tombstone: false,
                tombstone_type: None,
                deleted_at: None,
                deleted_by: None,
            };

            let key = make_key(prefixes::RELEASE, &release.id);
            db.put(&key, &release).unwrap();
        }

        // Verify we have 5 releases
        let releases: Vec<Release> = db.get_all_with_prefix(prefixes::RELEASE).unwrap();
        assert_eq!(releases.len(), 5, "Should have 5 releases before delete");

        // Delete all releases
        let response = delete_all_releases(axum::extract::State(state)).await;

        // Verify response
        let (status, json_response) = match response.into_response().status() {
            axum::http::StatusCode::OK => {
                // Extract JSON from response (this is simplified - in real test would parse properly)
                (axum::http::StatusCode::OK, true)
            }
            _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, false),
        };

        assert_eq!(status, axum::http::StatusCode::OK);
        assert!(json_response, "Delete should succeed");

        // Verify all releases are now tombstones (proof of erasure)
        let releases_after: Vec<Release> = db.get_all_with_prefix(prefixes::RELEASE).unwrap();
        assert_eq!(releases_after.len(), 5, "Tombstones should still exist in database");

        // Verify all releases are tombstones
        for release in &releases_after {
            assert!(release.is_tombstone, "Release {} should be a tombstone", release.id);
            assert_eq!(release.tombstone_type, Some(crate::routes::releases::TombstoneType::Temporary),
                "Release {} should have Temporary tombstone type", release.id);
            assert!(release.deleted_at.is_some(), "Release {} should have deleted_at timestamp", release.id);
            assert_eq!(release.deleted_by, Some("bulk-delete".to_string()),
                "Release {} should have deleted_by set to bulk-delete", release.id);
        }

        // Verify a single UBTS delete transaction block was created
        let delete_txs: Vec<UBTSBlock> = db.get_all_with_prefix(prefixes::DELETE_TRANSACTION).unwrap();
        assert_eq!(delete_txs.len(), 1, "Should have exactly 1 bulk delete transaction block");

        // Verify the delete transaction block contains 5 DeleteRelease transactions
        let delete_tx_block = &delete_txs[0];
        assert_eq!(delete_tx_block.transactions.len(), 5, "Delete block should contain 5 DeleteRelease transactions");

        // Verify all transactions are DeleteRelease types
        for tx in &delete_tx_block.transactions {
            match tx {
                crate::ubts::UBTSTransaction::DeleteRelease { id, .. } => {
                    assert!(id.starts_with("test-release-"), "Transaction should be for a test release");
                }
                _ => panic!("Transaction should be DeleteRelease type"),
            }
        }
    }
}
