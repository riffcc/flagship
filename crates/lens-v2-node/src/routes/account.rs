use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::db::{Database, prefixes, make_key};
use crate::ubts::UBTSTransaction;
use uuid::Uuid;
use tokio::sync::mpsc;

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

/// Authorization transaction - a UBTS flat transaction
///
/// This transaction authorizes a public key for admin access.
/// Syncs via SPORE across all nodes - no local state files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationTransaction {
    /// Unique ID for this transaction (UUID)
    pub id: String,

    /// Public key being authorized
    pub public_key: String,

    /// Timestamp when authorization was created
    pub timestamp: u64,

    /// Optional role (currently "admin" is the only role)
    pub role: String,
}

/// Notification for immediate WantList broadcast
#[derive(Debug, Clone)]
pub enum BlockNotification {
    /// New block created - broadcast WantList immediately
    NewBlock(String),
}

/// Account state shared across handlers
///
/// Authorization is stored as UBTS transactions and synced via SPORE.
/// No local state files - everything is UBTS transactions in the database.
#[derive(Clone)]
pub struct AccountState {
    /// Database for querying UBTS authorization transactions
    pub db: Database,

    /// Channel to notify sync orchestrator of new blocks (for immediate broadcast)
    pub block_notify: Option<mpsc::UnboundedSender<BlockNotification>>,
}

impl AccountState {
    pub fn new(db: Database) -> Self {
        tracing::info!("AccountState: Using UBTS transactions for authorization (no local files)");
        Self {
            db,
            block_notify: None,
        }
    }

    pub fn with_notify(mut self, notify: mpsc::UnboundedSender<BlockNotification>) -> Self {
        self.block_notify = Some(notify);
        self
    }

    /// Check if a public key is authorized as admin by querying UBTS transactions
    pub async fn is_admin(&self, public_key: &str) -> bool {
        tracing::debug!("Checking admin status for {} in UBTS transactions", public_key);

        // Query all authorization transactions from database
        match self.db.get_all_with_prefix::<AuthorizationTransaction>(prefixes::AUTHORIZATION) {
            Ok(authorizations) => {
                // Check if any authorization transaction matches this public key
                let is_admin = authorizations.iter().any(|auth| {
                    auth.public_key == public_key && auth.role == "admin"
                });

                if is_admin {
                    tracing::info!("✅ Public key {} is authorized as admin (found in UBTS)", public_key);
                } else {
                    tracing::debug!("❌ Public key {} not found in authorization transactions", public_key);
                }

                is_admin
            }
            Err(e) => {
                tracing::error!("Failed to query authorization transactions: {}", e);
                false
            }
        }
    }

    /// Get roles for a public key from UBTS transactions
    pub async fn get_roles(&self, public_key: &str) -> Vec<String> {
        tracing::debug!("Getting roles for {} from UBTS transactions", public_key);

        match self.db.get_all_with_prefix::<AuthorizationTransaction>(prefixes::AUTHORIZATION) {
            Ok(authorizations) => {
                // Collect all roles for this public key
                authorizations
                    .iter()
                    .filter(|auth| auth.public_key == public_key)
                    .map(|auth| auth.role.clone())
                    .collect()
            }
            Err(e) => {
                tracing::error!("Failed to query authorization transactions: {}", e);
                Vec::new()
            }
        }
    }

    /// Authorize a public key as admin by creating a UBTS transaction
    ///
    /// This creates an AuthorizeAdmin transaction which will be:
    /// 1. Stored in local database
    /// 2. Announced in WantLists
    /// 3. Auto-synced to all other nodes via SPORE
    pub async fn authorize_admin(&self, public_key: String) -> anyhow::Result<()> {
        tracing::info!("Creating AuthorizeAdmin UBTS transaction for {}", public_key);

        // Create authorization transaction with unique UUID
        let auth_tx = AuthorizationTransaction {
            id: Uuid::new_v4().to_string(),
            public_key: public_key.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            role: "admin".to_string(),
        };

        // Store in database with authorization prefix
        let key = make_key(prefixes::AUTHORIZATION, &auth_tx.id);
        self.db.put(&key, &auth_tx)?;

        tracing::info!("✅ Created authorization transaction {} for {}", auth_tx.id, public_key);

        // Immediately notify sync orchestrator to broadcast updated WantList
        if let Some(ref notify) = self.block_notify {
            if let Err(e) = notify.send(BlockNotification::NewBlock(auth_tx.id.clone())) {
                tracing::warn!("Failed to send block notification: {}", e);
            } else {
                tracing::info!("🚀 INSTANT BROADCAST triggered for authorization transaction");
            }
        }

        Ok(())
    }

    /// Add a role to a public key via UBTS transaction
    pub async fn add_role(&self, public_key: String, role: String) -> anyhow::Result<()> {
        tracing::info!("Adding role {} to {} via UBTS transaction", role, public_key);

        // Create authorization transaction with unique UUID
        let auth_tx = AuthorizationTransaction {
            id: Uuid::new_v4().to_string(),
            public_key: public_key.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            role: role.clone(),
        };

        // Store in database with authorization prefix
        let key = make_key(prefixes::AUTHORIZATION, &auth_tx.id);
        self.db.put(&key, &auth_tx)?;

        tracing::info!("✅ Created role assignment transaction {} for {} with role {}", auth_tx.id, public_key, role);

        // Immediately notify sync orchestrator to broadcast updated WantList
        if let Some(ref notify) = self.block_notify {
            if let Err(e) = notify.send(BlockNotification::NewBlock(auth_tx.id.clone())) {
                tracing::warn!("Failed to send block notification: {}", e);
            } else {
                tracing::info!("🚀 INSTANT BROADCAST triggered for role assignment transaction");
            }
        }

        Ok(())
    }
}

// Note: No Default implementation - AccountState requires a Database parameter

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
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_account_state() {
        // Create temporary database
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        let state = AccountState::new(db);

        let public_key = "test_key_123".to_string();

        // Initially not admin
        assert!(!state.is_admin(&public_key).await);

        // Authorize as admin
        state.authorize_admin(public_key.clone()).await.unwrap();

        // Now is admin
        assert!(state.is_admin(&public_key).await);

        // Has admin role
        let roles = state.get_roles(&public_key).await;
        assert!(roles.contains(&"admin".to_string()));
    }

    #[tokio::test]
    async fn test_add_role() {
        // Create temporary database
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path()).unwrap();
        let state = AccountState::new(db);

        let public_key = "test_key_456".to_string();

        // Add creator role
        state.add_role(public_key.clone(), "creator".to_string()).await.unwrap();

        // Check role
        let roles = state.get_roles(&public_key).await;
        assert!(roles.contains(&"creator".to_string()));
    }
}
