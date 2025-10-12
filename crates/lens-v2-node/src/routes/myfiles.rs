use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::account::AccountState;
use super::upload::extract_public_key;

/// Virtual folder in user's filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualFolder {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Virtual file entry (references IPFS upload)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualFile {
    pub id: String,
    pub name: String,
    pub folder_id: String,
    pub upload_id: String,
    pub ipfs_cid: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

/// Folder contents response
#[derive(Debug, Serialize)]
pub struct FolderContents {
    pub folder: VirtualFolder,
    pub folders: Vec<VirtualFolder>,
    pub files: Vec<VirtualFile>,
    pub breadcrumbs: Vec<VirtualFolder>,
}

/// Create folder request
#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// Rename folder request
#[derive(Debug, Deserialize)]
pub struct RenameFolderRequest {
    pub name: String,
}

/// Add file request
#[derive(Debug, Deserialize)]
pub struct AddFileRequest {
    pub upload_id: String,
    pub folder_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<String>,
}

/// Move folder request
#[derive(Debug, Deserialize)]
pub struct MoveFolderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_parent_id: Option<String>,
}

/// Move file request
#[derive(Debug, Deserialize)]
pub struct MoveFileRequest {
    pub new_folder_id: String,
}

/// Rename file request
#[derive(Debug, Deserialize)]
pub struct RenameFileRequest {
    pub name: String,
}

// TODO: Implement RocksDB storage
// For now, using in-memory HashMap (will be replaced with proper DB)
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

type FolderStore = Arc<RwLock<HashMap<String, VirtualFolder>>>;
type FileStore = Arc<RwLock<HashMap<String, VirtualFile>>>;

pub struct MyFilesState {
    folders: FolderStore,
    files: FileStore,
}

impl MyFilesState {
    pub fn new() -> Self {
        Self {
            folders: Arc::new(RwLock::new(HashMap::new())),
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Ensure root folder exists for user
    async fn ensure_root_folder(&self, owner: &str) -> anyhow::Result<VirtualFolder> {
        let mut folders = self.folders.write().await;

        // Check if root folder exists
        let root_id = format!("root_{}", owner);

        if let Some(root) = folders.get(&root_id) {
            return Ok(root.clone());
        }

        // Create root folder
        let root = VirtualFolder {
            id: root_id.clone(),
            name: "My Files".to_string(),
            parent_id: None,
            owner: owner.to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        folders.insert(root_id, root.clone());
        Ok(root)
    }

    /// Get folder by ID
    async fn get_folder(&self, folder_id: &str, owner: &str) -> anyhow::Result<VirtualFolder> {
        let folders = self.folders.read().await;

        let folder = folders
            .get(folder_id)
            .ok_or_else(|| anyhow::anyhow!("Folder not found"))?;

        // Verify ownership
        if folder.owner != owner {
            anyhow::bail!("Access denied");
        }

        Ok(folder.clone())
    }

    /// Get child folders
    async fn get_child_folders(&self, parent_id: &str, owner: &str) -> Vec<VirtualFolder> {
        let folders = self.folders.read().await;

        folders
            .values()
            .filter(|f| f.owner == owner && f.parent_id.as_deref() == Some(parent_id))
            .cloned()
            .collect()
    }

    /// Get files in folder
    async fn get_files_in_folder(&self, folder_id: &str, owner: &str) -> Vec<VirtualFile> {
        let files = self.files.read().await;

        files
            .values()
            .filter(|f| f.owner == owner && f.folder_id == folder_id)
            .cloned()
            .collect()
    }

    /// Get breadcrumb trail to folder
    async fn get_breadcrumbs(&self, folder_id: &str, owner: &str) -> Vec<VirtualFolder> {
        let folders = self.folders.read().await;
        let mut breadcrumbs = Vec::new();
        let mut current_id = Some(folder_id.to_string());

        while let Some(id) = current_id {
            if let Some(folder) = folders.get(&id) {
                if folder.owner != owner {
                    break;
                }
                breadcrumbs.insert(0, folder.clone());
                current_id = folder.parent_id.clone();
            } else {
                break;
            }
        }

        breadcrumbs
    }

    /// Create folder
    async fn create_folder(&self, name: String, parent_id: Option<String>, owner: String) -> anyhow::Result<VirtualFolder> {
        let mut folders = self.folders.write().await;

        // Verify parent exists and is owned by user
        if let Some(parent) = &parent_id {
            if let Some(parent_folder) = folders.get(parent) {
                if parent_folder.owner != owner {
                    anyhow::bail!("Access denied");
                }
            } else {
                anyhow::bail!("Parent folder not found");
            }
        }

        let folder = VirtualFolder {
            id: Uuid::new_v4().to_string(),
            name,
            parent_id,
            owner,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        folders.insert(folder.id.clone(), folder.clone());
        Ok(folder)
    }

    /// Add file to folder
    async fn add_file(
        &self,
        upload_id: String,
        folder_id: String,
        custom_name: Option<String>,
        owner: String,
    ) -> anyhow::Result<VirtualFile> {
        // Verify folder exists and is owned by user
        self.get_folder(&folder_id, &owner).await?;

        // Load upload metadata to get file info
        let upload_metadata = super::upload::load_metadata(&upload_id).await?;

        // Verify upload is owned by user
        if upload_metadata.uploader_public_key != owner {
            anyhow::bail!("You can only add your own uploads");
        }

        // Verify upload is approved (has CID)
        let ipfs_cid = upload_metadata
            .ipfs_cid
            .ok_or_else(|| anyhow::anyhow!("Upload not yet approved"))?;

        let file = VirtualFile {
            id: Uuid::new_v4().to_string(),
            name: custom_name.unwrap_or(upload_metadata.filename),
            folder_id,
            upload_id,
            ipfs_cid,
            size_bytes: upload_metadata.size_bytes,
            mime_type: upload_metadata.mime_type,
            owner,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        let mut files = self.files.write().await;
        files.insert(file.id.clone(), file.clone());
        Ok(file)
    }

    /// Move folder to new parent
    async fn move_folder(&self, folder_id: String, new_parent_id: Option<String>, owner: String) -> anyhow::Result<VirtualFolder> {
        // Get the folder to move
        let folder = self.get_folder(&folder_id, &owner).await?;

        // Verify new parent exists and is owned by user (if specified)
        if let Some(parent_id) = &new_parent_id {
            self.get_folder(parent_id, &owner).await?;

            // Prevent moving folder into itself or its descendants
            if self.is_descendant(&folder_id, parent_id).await {
                anyhow::bail!("Cannot move folder into itself or its descendants");
            }
        }

        // Update folder
        let mut folders = self.folders.write().await;
        if let Some(f) = folders.get_mut(&folder_id) {
            f.parent_id = new_parent_id;
            f.modified_at = Utc::now();
            Ok(f.clone())
        } else {
            anyhow::bail!("Folder not found");
        }
    }

    /// Move file to new folder
    async fn move_file(&self, file_id: String, new_folder_id: String, owner: String) -> anyhow::Result<VirtualFile> {
        // Verify destination folder exists and is owned by user
        self.get_folder(&new_folder_id, &owner).await?;

        // Get and verify file ownership
        let mut files = self.files.write().await;
        if let Some(file) = files.get_mut(&file_id) {
            if file.owner != owner {
                anyhow::bail!("Access denied");
            }

            file.folder_id = new_folder_id;
            file.modified_at = Utc::now();
            Ok(file.clone())
        } else {
            anyhow::bail!("File not found");
        }
    }

    /// Rename folder
    async fn rename_folder(&self, folder_id: String, new_name: String, owner: String) -> anyhow::Result<VirtualFolder> {
        let mut folders = self.folders.write().await;
        if let Some(folder) = folders.get_mut(&folder_id) {
            if folder.owner != owner {
                anyhow::bail!("Access denied");
            }

            folder.name = new_name;
            folder.modified_at = Utc::now();
            Ok(folder.clone())
        } else {
            anyhow::bail!("Folder not found");
        }
    }

    /// Rename file
    async fn rename_file(&self, file_id: String, new_name: String, owner: String) -> anyhow::Result<VirtualFile> {
        let mut files = self.files.write().await;
        if let Some(file) = files.get_mut(&file_id) {
            if file.owner != owner {
                anyhow::bail!("Access denied");
            }

            file.name = new_name;
            file.modified_at = Utc::now();
            Ok(file.clone())
        } else {
            anyhow::bail!("File not found");
        }
    }

    /// Check if target_id is a descendant of folder_id
    async fn is_descendant(&self, folder_id: &str, target_id: &str) -> bool {
        if folder_id == target_id {
            return true;
        }

        let folders = self.folders.read().await;
        let mut current_id = Some(target_id.to_string());

        while let Some(id) = current_id {
            if id == folder_id {
                return true;
            }

            if let Some(folder) = folders.get(&id) {
                current_id = folder.parent_id.clone();
            } else {
                break;
            }
        }

        false
    }

    /// Delete folder (must be empty)
    async fn delete_folder(&self, folder_id: String, owner: String) -> anyhow::Result<()> {
        // Verify ownership
        let folder = self.get_folder(&folder_id, &owner).await?;

        // Check if folder is root
        if folder.parent_id.is_none() {
            anyhow::bail!("Cannot delete root folder");
        }

        // Check if folder has children
        let child_folders = self.get_child_folders(&folder_id, &owner).await;
        let files = self.get_files_in_folder(&folder_id, &owner).await;

        if !child_folders.is_empty() || !files.is_empty() {
            anyhow::bail!("Cannot delete non-empty folder");
        }

        // Delete folder
        let mut folders = self.folders.write().await;
        folders.remove(&folder_id);
        Ok(())
    }

    /// Delete file
    async fn delete_file(&self, file_id: String, owner: String) -> anyhow::Result<()> {
        let mut files = self.files.write().await;

        // Verify ownership before deleting
        if let Some(file) = files.get(&file_id) {
            if file.owner != owner {
                anyhow::bail!("Access denied");
            }
        } else {
            anyhow::bail!("File not found");
        }

        files.remove(&file_id);
        Ok(())
    }
}

impl Clone for MyFilesState {
    fn clone(&self) -> Self {
        Self {
            folders: Arc::clone(&self.folders),
            files: Arc::clone(&self.files),
        }
    }
}

/// GET /api/v1/myfiles - Get root folder contents
pub async fn get_root(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    // Ensure root folder exists
    let root = match state.ensure_root_folder(&public_key).await {
        Ok(folder) => folder,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to get root folder: {}", e)})),
            )
                .into_response();
        }
    };

    // Get contents of root folder
    let folders = state.get_child_folders(&root.id, &public_key).await;
    let files = state.get_files_in_folder(&root.id, &public_key).await;
    let breadcrumbs = vec![root.clone()];

    (
        StatusCode::OK,
        Json(FolderContents {
            folder: root,
            folders,
            files,
            breadcrumbs,
        }),
    )
        .into_response()
}

/// GET /api/v1/myfiles/folder/:id - Get folder contents
pub async fn get_folder_contents(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Path(folder_id): Path<String>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    // Get folder
    let folder = match state.get_folder(&folder_id, &public_key).await {
        Ok(f) => f,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Folder not found"})),
            )
                .into_response();
        }
    };

    // Get contents
    let folders = state.get_child_folders(&folder_id, &public_key).await;
    let files = state.get_files_in_folder(&folder_id, &public_key).await;
    let breadcrumbs = state.get_breadcrumbs(&folder_id, &public_key).await;

    (
        StatusCode::OK,
        Json(FolderContents {
            folder,
            folders,
            files,
            breadcrumbs,
        }),
    )
        .into_response()
}

/// POST /api/v1/myfiles/folder - Create folder
pub async fn create_folder(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Json(req): Json<CreateFolderRequest>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    // If no parent specified, use root
    let parent_id = if let Some(pid) = req.parent_id {
        Some(pid)
    } else {
        let root = match state.ensure_root_folder(&public_key).await {
            Ok(f) => f,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to get root: {}", e)})),
                )
                    .into_response();
            }
        };
        Some(root.id)
    };

    match state.create_folder(req.name, parent_id, public_key).await {
        Ok(folder) => (StatusCode::OK, Json(folder)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to create folder: {}", e)})),
        )
            .into_response(),
    }
}

/// POST /api/v1/myfiles/file - Add file to folder
pub async fn add_file(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Json(req): Json<AddFileRequest>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    match state
        .add_file(req.upload_id, req.folder_id, req.custom_name, public_key)
        .await
    {
        Ok(file) => (StatusCode::OK, Json(file)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to add file: {}", e)})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/myfiles/folder/:id/move - Move folder to new parent
pub async fn move_folder(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Path(folder_id): Path<String>,
    Json(req): Json<MoveFolderRequest>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    match state.move_folder(folder_id, req.new_parent_id, public_key).await {
        Ok(folder) => (StatusCode::OK, Json(folder)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to move folder: {}", e)})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/myfiles/file/:id/move - Move file to new folder
pub async fn move_file(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Path(file_id): Path<String>,
    Json(req): Json<MoveFileRequest>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    match state.move_file(file_id, req.new_folder_id, public_key).await {
        Ok(file) => (StatusCode::OK, Json(file)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to move file: {}", e)})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/myfiles/folder/:id - Rename folder
pub async fn rename_folder(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Path(folder_id): Path<String>,
    Json(req): Json<RenameFolderRequest>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    match state.rename_folder(folder_id, req.name, public_key).await {
        Ok(folder) => (StatusCode::OK, Json(folder)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to rename folder: {}", e)})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/myfiles/file/:id - Rename file
pub async fn rename_file(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Path(file_id): Path<String>,
    Json(req): Json<RenameFileRequest>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    match state.rename_file(file_id, req.name, public_key).await {
        Ok(file) => (StatusCode::OK, Json(file)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to rename file: {}", e)})),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/myfiles/folder/:id - Delete folder
pub async fn delete_folder(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Path(folder_id): Path<String>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    match state.delete_folder(folder_id, public_key).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to delete folder: {}", e)})),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/myfiles/file/:id - Delete file
pub async fn delete_file(
    State(state): State<MyFilesState>,
    headers: HeaderMap,
    Path(file_id): Path<String>,
) -> impl IntoResponse {
    let public_key = match extract_public_key(&headers) {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing X-Public-Key header"})),
            )
                .into_response();
        }
    };

    match state.delete_file(file_id, public_key).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("Failed to delete file: {}", e)})),
        )
            .into_response(),
    }
}
