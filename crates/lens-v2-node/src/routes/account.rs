use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::persistence;

/// Account status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatus {
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// Authorization request
#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

/// Account state shared across handlers
#[derive(Clone)]
pub struct AccountState {
    /// Map of public keys to their roles
    pub authorized_keys: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Set of admin public keys
    pub admin_keys: Arc<RwLock<HashSet<String>>>,
}

impl AccountState {
    pub fn new() -> Self {
        let state = Self {
            authorized_keys: Arc::new(RwLock::new(HashMap::new())),
            admin_keys: Arc::new(RwLock::new(HashSet::new())),
        };

        // Try to load persisted admin keys
        if let Ok(Some(admin_keys)) = persistence::load_json::<Vec<String>>("admin_keys.json") {
            let admin_set: HashSet<String> = admin_keys.into_iter().collect();
            if let Ok(mut keys) = state.admin_keys.try_write() {
                *keys = admin_set.clone();
                tracing::info!("Loaded {} admin keys from persistence", keys.len());
            }

            // Also populate authorized_keys
            if let Ok(mut auth_keys) = state.authorized_keys.try_write() {
                for key in admin_set {
                    let mut roles = HashSet::new();
                    roles.insert("admin".to_string());
                    auth_keys.insert(key, roles);
                }
            }
        }

        state
    }

    /// Save admin keys to disk
    async fn save(&self) {
        let admin_keys = self.admin_keys.read().await;
        let keys_vec: Vec<String> = admin_keys.iter().cloned().collect();

        if let Err(e) = persistence::save_json("admin_keys.json", &keys_vec) {
            tracing::error!("Failed to save admin keys: {}", e);
        }
    }

    /// Check if a public key is authorized as admin
    pub async fn is_admin(&self, public_key: &str) -> bool {
        self.admin_keys.read().await.contains(public_key)
    }

    /// Get roles for a public key
    pub async fn get_roles(&self, public_key: &str) -> Vec<String> {
        self.authorized_keys
            .read()
            .await
            .get(public_key)
            .map(|roles| roles.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Authorize a public key as admin
    pub async fn authorize_admin(&self, public_key: String) {
        self.admin_keys.write().await.insert(public_key.clone());

        // Also add admin role to authorized_keys
        let mut keys = self.authorized_keys.write().await;
        keys.entry(public_key)
            .or_insert_with(HashSet::new)
            .insert("admin".to_string());

        // Save to disk
        drop(keys); // Release lock before saving
        self.save().await;
    }

    /// Add a role to a public key
    pub async fn add_role(&self, public_key: String, role: String) {
        let mut keys = self.authorized_keys.write().await;
        keys.entry(public_key)
            .or_insert_with(HashSet::new)
            .insert(role);
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self::new()
    }
}

/// GET /api/v1/account - Get account status
pub async fn get_account(State(_state): State<AccountState>) -> impl IntoResponse {
    // For now, return a default account response
    // In production, you'd extract the public key from a header or session
    let status = AccountStatus {
        is_admin: false,
        roles: vec![],
        permissions: vec![],
    };

    Json(status)
}

/// POST /api/v1/admin/authorize - Authorize a public key as admin
/// This is a temporary admin-only endpoint for initial setup
pub async fn authorize_admin(
    State(state): State<AccountState>,
    Json(req): Json<AuthorizeRequest>,
) -> impl IntoResponse {
    tracing::info!("Authorizing admin: {}", req.public_key);

    // Authorize the public key as admin
    state.authorize_admin(req.public_key.clone()).await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": format!("Public key {} authorized as admin", req.public_key)
        })),
    )
}

/// GET /api/v1/account/:public_key - Get account status for a specific public key
pub async fn get_account_status(
    State(state): State<AccountState>,
    axum::extract::Path(public_key): axum::extract::Path<String>,
) -> impl IntoResponse {
    let is_admin = state.is_admin(&public_key).await;
    let roles = state.get_roles(&public_key).await;

    let mut permissions = Vec::new();
    if is_admin {
        permissions.push("admin".to_string());
        permissions.push("upload".to_string());
        permissions.push("moderate".to_string());
    } else if roles.contains(&"moderator".to_string()) {
        permissions.push("upload".to_string());
        permissions.push("moderate".to_string());
    } else if roles.contains(&"creator".to_string()) {
        permissions.push("upload".to_string());
    }

    let status = AccountStatus {
        is_admin,
        roles,
        permissions,
    };

    Json(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account_state() {
        let state = AccountState::new();
        let public_key = "test_key_123".to_string();

        // Initially not admin
        assert!(!state.is_admin(&public_key).await);

        // Authorize as admin
        state.authorize_admin(public_key.clone()).await;

        // Now is admin
        assert!(state.is_admin(&public_key).await);

        // Has admin role
        let roles = state.get_roles(&public_key).await;
        assert!(roles.contains(&"admin".to_string()));
    }

    #[tokio::test]
    async fn test_add_role() {
        let state = AccountState::new();
        let public_key = "test_key_456".to_string();

        // Add creator role
        state.add_role(public_key.clone(), "creator".to_string()).await;

        // Check role
        let roles = state.get_roles(&public_key).await;
        assert!(roles.contains(&"creator".to_string()));
    }
}
