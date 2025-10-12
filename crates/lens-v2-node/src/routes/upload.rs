use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::process::Command;

use super::account::AccountState;

/// Upload metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMetadata {
    pub upload_id: String,
    pub uploader_public_key: String,
    pub timestamp: DateTime<Utc>,
    pub filename: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub status: UploadStatus,
    pub auto_approved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfs_cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    Pending,
    Approved,
    Rejected,
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub upload_id: String,
    pub status: UploadStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfs_cid: Option<String>,
    pub message: String,
}

/// List uploads response
#[derive(Debug, Serialize)]
pub struct ListUploadsResponse {
    pub uploads: Vec<UploadMetadata>,
    pub total: usize,
}

/// Approval request
#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
}

/// Get the staging directory path
fn get_staging_dir() -> PathBuf {
    PathBuf::from(std::env::var("UPLOAD_STAGING_DIR").unwrap_or_else(|_| "data/staging".to_string()))
}

/// Get the approved directory path
fn get_approved_dir() -> PathBuf {
    PathBuf::from(std::env::var("UPLOAD_APPROVED_DIR").unwrap_or_else(|_| "data/approved".to_string()))
}

/// Extract public key from request headers
pub fn extract_public_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Public-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Check if user can auto-approve uploads
async fn can_auto_approve(state: &AccountState, public_key: &str) -> bool {
    let roles = state.get_roles(public_key).await;
    roles.contains(&"uploader".to_string())
        || roles.contains(&"moderator".to_string())
        || state.is_admin(public_key).await
}

/// Check if user can approve others' uploads
async fn can_approve(state: &AccountState, public_key: &str) -> bool {
    let roles = state.get_roles(public_key).await;
    roles.contains(&"moderator".to_string()) || state.is_admin(public_key).await
}

/// Save metadata to file
async fn save_metadata(staging_dir: &PathBuf, metadata: &UploadMetadata) -> anyhow::Result<()> {
    let meta_path = staging_dir.join(format!("{}.meta", metadata.upload_id));
    let json = serde_json::to_string_pretty(metadata)?;
    fs::write(&meta_path, json).await?;
    Ok(())
}

/// Load metadata from file
pub async fn load_metadata(upload_id: &str) -> anyhow::Result<UploadMetadata> {
    let staging_dir = get_staging_dir();
    let meta_path = staging_dir.join(format!("{}.meta", upload_id));
    let json = fs::read_to_string(&meta_path).await?;
    let metadata: UploadMetadata = serde_json::from_str(&json)?;
    Ok(metadata)
}

/// Pin file to IPFS cluster with metadata as pin name
async fn pin_to_ipfs(file_path: &PathBuf, metadata: &UploadMetadata) -> anyhow::Result<String> {
    // Construct pin name with useful metadata
    // Format: "title | filename | uploader | timestamp"
    let title = metadata
        .additional_metadata
        .as_ref()
        .and_then(|m| m.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or(&metadata.filename);

    // Truncate public key for display (first 8 chars)
    let uploader_short = if metadata.uploader_public_key.len() > 8 {
        &metadata.uploader_public_key[..8]
    } else {
        &metadata.uploader_public_key
    };

    let timestamp = metadata.timestamp.format("%Y-%m-%d %H:%M UTC");

    let pin_name = format!(
        "{} | {} | {} | {}",
        title,
        metadata.filename,
        uploader_short,
        timestamp
    );

    tracing::info!("Pinning to IPFS Cluster with name: {}", pin_name);

    let output = Command::new("ipfs-cluster-ctl")
        .arg("add")
        .arg("--name")
        .arg(&pin_name)
        .arg(file_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("IPFS cluster pin failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse CID from output like: "added QmXXX filename"
    let cid = stdout
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse CID from ipfs-cluster-ctl output"))?
        .to_string();

    Ok(cid)
}

/// POST /api/v1/upload - Upload a file
pub async fn upload_file(
    State(state): State<AccountState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Extract public key from headers
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Missing X-Public-Key header"
                })),
            )
                .into_response();
        }
    };

    // Check if user has upload permission
    let has_upload_permission = can_auto_approve(&state, &public_key).await;

    if !has_upload_permission {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "success": false,
                "error": "You don't have permission to upload files. Contact an administrator to get the 'uploader' role."
            })),
        )
            .into_response();
    }

    // Generate upload ID
    let upload_id = Uuid::new_v4().to_string();
    let staging_dir = get_staging_dir();

    // Create staging directory
    if let Err(e) = fs::create_dir_all(&staging_dir).await {
        tracing::error!("Failed to create staging directory: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Failed to create staging directory"}))).into_response();
    }

    let upload_dir = staging_dir.join(&upload_id);
    if let Err(e) = fs::create_dir_all(&upload_dir).await {
        tracing::error!("Failed to create upload directory: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Failed to create upload directory"}))).into_response();
    }

    let mut filename = String::new();
    let mut file_size = 0u64;
    let mut mime_type: Option<String> = None;
    let mut additional_metadata: Option<serde_json::Value> = None;

    // Process multipart fields
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            filename = field.file_name().unwrap_or("unnamed").to_string();
            mime_type = field.content_type().map(|s| s.to_string());

            let file_path = upload_dir.join(&filename);
            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    tracing::error!("Failed to read file data: {}", e);
                    return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "Failed to read file data"}))).into_response();
                }
            };

            file_size = data.len() as u64;

            if let Err(e) = fs::write(&file_path, &data).await {
                tracing::error!("Failed to write file: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Failed to save file"}))).into_response();
            }
        } else if field_name == "metadata" {
            match field.text().await {
                Ok(data) => {
                    additional_metadata = serde_json::from_str(&data).ok();
                }
                Err(e) => {
                    tracing::warn!("Failed to read metadata field: {}", e);
                }
            }
        }
    }

    if filename.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "No file provided"}))).into_response();
    }

    // Determine if auto-approve
    let auto_approved = can_auto_approve(&state, &public_key).await;
    let mut status = if auto_approved {
        UploadStatus::Approved
    } else {
        UploadStatus::Pending
    };

    // Create metadata
    let mut metadata = UploadMetadata {
        upload_id: upload_id.clone(),
        uploader_public_key: public_key.clone(),
        timestamp: Utc::now(),
        filename: filename.clone(),
        size_bytes: file_size,
        mime_type,
        status: status.clone(),
        auto_approved,
        approved_by: if auto_approved { Some(public_key.clone()) } else { None },
        approved_at: if auto_approved { Some(Utc::now()) } else { None },
        ipfs_cid: None,
        additional_metadata,
    };

    // If auto-approved, pin to IPFS immediately
    let mut ipfs_cid: Option<String> = None;
    if auto_approved {
        let file_path = upload_dir.join(&filename);
        match pin_to_ipfs(&file_path, &metadata).await {
            Ok(cid) => {
                metadata.ipfs_cid = Some(cid.clone());
                ipfs_cid = Some(cid);
                tracing::info!("Auto-approved upload {} pinned to IPFS: {}", upload_id, ipfs_cid.as_ref().unwrap());
            }
            Err(e) => {
                tracing::error!("Failed to pin to IPFS: {}", e);
                // Don't fail the upload, just mark as pending for manual review
                metadata.status = UploadStatus::Pending;
                metadata.auto_approved = false;
                metadata.approved_by = None;
                metadata.approved_at = None;
                status = UploadStatus::Pending;
            }
        }
    }

    // Save metadata
    if let Err(e) = save_metadata(&staging_dir, &metadata).await {
        tracing::error!("Failed to save metadata: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Failed to save metadata"}))).into_response();
    }

    tracing::info!("Upload {} from {} - Status: {:?}", upload_id, public_key, status);

    (
        StatusCode::OK,
        Json(UploadResponse {
            success: true,
            upload_id,
            status,
            ipfs_cid,
            message: if auto_approved {
                "File uploaded and automatically approved".to_string()
            } else {
                "File uploaded and awaiting approval".to_string()
            },
        }),
    )
        .into_response()
}

/// GET /api/v1/uploads/my-approved - List user's approved uploads
pub async fn list_my_approved_uploads(
    headers: HeaderMap,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"success": false, "error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    let staging_dir = get_staging_dir();
    let mut uploads = Vec::new();

    if let Ok(mut entries) = fs::read_dir(&staging_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                if let Ok(metadata) = load_metadata(
                    &path.file_stem().unwrap().to_string_lossy().to_string()
                ).await {
                    // Only return approved uploads owned by this user
                    if metadata.status == UploadStatus::Approved
                        && metadata.uploader_public_key == public_key
                        && metadata.ipfs_cid.is_some() {
                        uploads.push(metadata);
                    }
                }
            }
        }
    }

    // Sort by timestamp, newest first
    uploads.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    (
        StatusCode::OK,
        Json(ListUploadsResponse {
            total: uploads.len(),
            uploads,
        }),
    )
        .into_response()
}

/// GET /api/v1/admin/uploads/pending - List pending uploads
pub async fn list_pending_uploads(
    State(state): State<AccountState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"success": false, "error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    if !can_approve(&state, &public_key).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"success": false, "error": "Only moderators and admins can list pending uploads"})),
        )
            .into_response();
    }

    let staging_dir = get_staging_dir();
    let mut uploads = Vec::new();

    if let Ok(mut entries) = fs::read_dir(&staging_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                if let Ok(metadata) = load_metadata(
                    &path.file_stem().unwrap().to_string_lossy().to_string()
                ).await {
                    if metadata.status == UploadStatus::Pending {
                        uploads.push(metadata);
                    }
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(ListUploadsResponse {
            total: uploads.len(),
            uploads,
        }),
    )
        .into_response()
}

/// GET /api/v1/admin/uploads/:id - Get upload details
pub async fn get_upload(
    State(state): State<AccountState>,
    headers: HeaderMap,
    Path(upload_id): Path<String>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"success": false, "error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    if !can_approve(&state, &public_key).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"success": false, "error": "Only moderators and admins can view upload details"})),
        )
            .into_response();
    }

    match load_metadata(&upload_id).await {
        Ok(metadata) => (StatusCode::OK, Json(metadata)).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"success": false, "error": "Upload not found"})),
        )
            .into_response(),
    }
}

/// POST /api/v1/admin/uploads/:id/approve - Approve an upload
pub async fn approve_upload(
    State(state): State<AccountState>,
    headers: HeaderMap,
    Path(upload_id): Path<String>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"success": false, "error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    if !can_approve(&state, &public_key).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"success": false, "error": "Only moderators and admins can approve uploads"})),
        )
            .into_response();
    }

    let mut metadata = match load_metadata(&upload_id).await {
        Ok(m) => m,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"success": false, "error": "Upload not found"})),
            )
                .into_response();
        }
    };

    if metadata.status == UploadStatus::Approved {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"success": false, "error": "Upload already approved"})),
        )
            .into_response();
    }

    // Pin to IPFS
    let staging_dir = get_staging_dir();
    let upload_dir = staging_dir.join(&upload_id);
    let file_path = upload_dir.join(&metadata.filename);

    let ipfs_cid = match pin_to_ipfs(&file_path, &metadata).await {
        Ok(cid) => cid,
        Err(e) => {
            tracing::error!("Failed to pin to IPFS: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"success": false, "error": format!("Failed to pin to IPFS: {}", e)})),
            )
                .into_response();
        }
    };

    // Update metadata
    metadata.status = UploadStatus::Approved;
    metadata.approved_by = Some(public_key.clone());
    metadata.approved_at = Some(Utc::now());
    metadata.ipfs_cid = Some(ipfs_cid.clone());

    if let Err(e) = save_metadata(&staging_dir, &metadata).await {
        tracing::error!("Failed to save metadata: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"success": false, "error": "Failed to save metadata"})),
        )
            .into_response();
    }

    tracing::info!("Upload {} approved by {} - IPFS CID: {}", upload_id, public_key, ipfs_cid);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "upload_id": upload_id,
            "ipfs_cid": ipfs_cid,
            "message": "Upload approved and pinned to IPFS"
        })),
    )
        .into_response()
}

/// POST /api/v1/admin/uploads/:id/reject - Reject an upload
pub async fn reject_upload(
    State(state): State<AccountState>,
    headers: HeaderMap,
    Path(upload_id): Path<String>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"success": false, "error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    if !can_approve(&state, &public_key).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"success": false, "error": "Only moderators and admins can reject uploads"})),
        )
            .into_response();
    }

    let mut metadata = match load_metadata(&upload_id).await {
        Ok(m) => m,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"success": false, "error": "Upload not found"})),
            )
                .into_response();
        }
    };

    // Update metadata
    metadata.status = UploadStatus::Rejected;

    let staging_dir = get_staging_dir();
    if let Err(e) = save_metadata(&staging_dir, &metadata).await {
        tracing::error!("Failed to save metadata: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"success": false, "error": "Failed to save metadata"})),
        )
            .into_response();
    }

    // Optionally delete the file
    let upload_dir = staging_dir.join(&upload_id);
    if let Err(e) = fs::remove_dir_all(&upload_dir).await {
        tracing::warn!("Failed to delete rejected upload directory: {}", e);
    }

    tracing::info!("Upload {} rejected by {}", upload_id, public_key);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "upload_id": upload_id,
            "message": "Upload rejected and removed"
        })),
    )
        .into_response()
}
