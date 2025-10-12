//! UBTS - Unified Block Transaction System
//!
//! A unified transaction system where ALL operations (creates, updates, deletes)
//! are represented as transactions in blocks. This provides a clean, extensible
//! architecture compared to TGPQL's separate block types.

use serde::{Deserialize, Serialize};
use crate::routes::releases::Release;
use crate::delete_block::DeleteReason;

/// UBTS Transaction - unified representation of all state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UBTSTransaction {
    /// Create a new release
    CreateRelease {
        release: Release,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Update an existing release
    UpdateRelease {
        id: String,
        patch: ReleasePatch,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Delete a release (simple deletion, no proof-of-erasure)
    DeleteRelease {
        id: String,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Delete with proof-of-erasure consensus
    /// This propagates through the network, requires all peers to confirm erasure
    DeleteWithConsensus {
        /// Unique ID for this delete transaction
        delete_id: String,
        /// Block IDs to be deleted
        deleted_block_ids: Vec<String>,
        /// Reason for deletion
        reason: DeleteReason,
        /// Admin who initiated the delete
        deleted_by: String,
        /// Required number of peer confirmations
        required_confirmations: usize,
        /// Timestamp when delete was initiated
        timestamp: u64,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Confirm erasure of blocks (sent by peers after deletion)
    ConfirmErasure {
        /// ID of the delete transaction being confirmed
        delete_tx_id: String,
        /// Peer ID confirming erasure
        peer_id: String,
        /// Block IDs that were successfully erased
        erased_block_ids: Vec<String>,
        /// Timestamp of confirmation
        timestamp: u64,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Create tombstone after proof-of-erasure
    /// Tombstones are anonymously double-hashed markers that prevent re-creation
    /// Can be overridden by admin re-uploading content
    CreateTombstone {
        /// Double-hashed ID of the deleted item (prevents linking)
        tombstone_hash: String,
        /// Reason for tombstone creation
        reason: DeleteReason,
        /// Timestamp when tombstone was created
        timestamp: u64,
    },

    /// Remove tombstone (admin override to allow re-upload)
    RemoveTombstone {
        /// Double-hashed ID of the tombstone to remove
        tombstone_hash: String,
        /// Admin who is removing the tombstone
        removed_by: String,
        /// Timestamp of removal
        timestamp: u64,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Authorize an admin
    AuthorizeAdmin {
        public_key: String,
        authorized_by: String,
        timestamp: u64,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Revoke admin authorization
    RevokeAdmin {
        public_key: String,
        revoked_by: String,
        timestamp: u64,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Set featured releases list (with full FeaturedRelease objects to preserve UUIDs)
    SetFeatured {
        featured_releases: Vec<crate::routes::featured::FeaturedRelease>,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Add releases to featured list
    AddFeatured {
        release_ids: Vec<String>,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Remove releases from featured list
    RemoveFeatured {
        release_ids: Vec<String>,
        #[serde(default)]
        signature: Option<String>,
    },

    /// Delete a specific featured release by UUID
    DeleteFeaturedRelease {
        id: String,
        #[serde(default)]
        signature: Option<String>,
    },
}

/// Patch for updating a release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_cid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_cid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// UBTS Block - contains multiple transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UBTSBlock {
    /// Block ID (hash of transactions)
    pub id: String,

    /// Block height in the chain
    pub height: u64,

    /// Previous block ID
    pub prev: Option<String>,

    /// Block timestamp
    pub timestamp: u64,

    /// Transactions in this block
    pub transactions: Vec<UBTSTransaction>,

    /// Block signature (by proposer)
    #[serde(default)]
    pub signature: Option<String>,
}

impl UBTSBlock {
    /// Create a new UBTS block
    pub fn new(height: u64, prev: Option<String>, transactions: Vec<UBTSTransaction>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Generate block ID from transactions
        let id = Self::compute_id(&transactions, height, timestamp);

        Self {
            id,
            height,
            prev,
            timestamp,
            transactions,
            signature: None,
        }
    }

    /// Compute block ID from transactions
    fn compute_id(transactions: &[UBTSTransaction], height: u64, timestamp: u64) -> String {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();

        // Hash transactions
        for tx in transactions {
            if let Ok(tx_bytes) = serde_json::to_vec(tx) {
                hasher.update(&tx_bytes);
            }
        }

        // Hash height and timestamp
        hasher.update(height.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());

        let result = hasher.finalize();
        format!("ubts-{}", hex::encode(result))
    }

    /// Add a transaction to this block
    pub fn add_transaction(&mut self, tx: UBTSTransaction) {
        self.transactions.push(tx);
        // Recompute block ID
        self.id = Self::compute_id(&self.transactions, self.height, self.timestamp);
    }

    /// Get number of transactions
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }
}

impl UBTSTransaction {
    /// Get the type name of this transaction
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::CreateRelease { .. } => "CreateRelease",
            Self::UpdateRelease { .. } => "UpdateRelease",
            Self::DeleteRelease { .. } => "DeleteRelease",
            Self::DeleteWithConsensus { .. } => "DeleteWithConsensus",
            Self::ConfirmErasure { .. } => "ConfirmErasure",
            Self::CreateTombstone { .. } => "CreateTombstone",
            Self::RemoveTombstone { .. } => "RemoveTombstone",
            Self::AuthorizeAdmin { .. } => "AuthorizeAdmin",
            Self::RevokeAdmin { .. } => "RevokeAdmin",
            Self::SetFeatured { .. } => "SetFeatured",
            Self::AddFeatured { .. } => "AddFeatured",
            Self::RemoveFeatured { .. } => "RemoveFeatured",
            Self::DeleteFeaturedRelease { .. } => "DeleteFeaturedRelease",
        }
    }

    /// Check if this transaction requires admin authorization
    pub fn requires_admin(&self) -> bool {
        match self {
            Self::CreateRelease { .. } => true,
            Self::UpdateRelease { .. } => true,
            Self::DeleteRelease { .. } => true,
            Self::DeleteWithConsensus { .. } => true,
            Self::ConfirmErasure { .. } => false, // Peers confirm, not admins
            Self::CreateTombstone { .. } => false, // Auto-created after consensus
            Self::RemoveTombstone { .. } => true,  // Admin override
            Self::AuthorizeAdmin { .. } => true,
            Self::RevokeAdmin { .. } => true,
            Self::SetFeatured { .. } => true,
            Self::AddFeatured { .. } => true,
            Self::RemoveFeatured { .. } => true,
            Self::DeleteFeaturedRelease { .. } => true,
        }
    }

    /// Create a double-hashed tombstone ID from a block ID
    /// This prevents linking the tombstone back to the original content
    pub fn create_tombstone_hash(block_id: &str) -> String {
        use sha2::{Sha256, Digest};

        // First hash
        let mut hasher = Sha256::new();
        hasher.update(block_id.as_bytes());
        let first_hash = hasher.finalize();

        // Second hash (double hash for anonymity)
        let mut hasher = Sha256::new();
        hasher.update(&first_hash);
        let second_hash = hasher.finalize();

        format!("tombstone-{}", hex::encode(second_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ubts_block_creation() {
        let tx = UBTSTransaction::CreateRelease {
            release: Release {
                id: "test-123".to_string(),
                name: "Test Release".to_string(),
                category_id: "cat-1".to_string(),
                category_slug: "test".to_string(),
                content_cid: "QmTest123".to_string(),
                thumbnail_cid: None,
                metadata: Some(serde_json::json!({"description": "A test release"})),
                site_address: "local".to_string(),
                posted_by: "test-user".to_string(),
                created_at: "2025-01-01T00:00:00Z".to_string(),
                vector_clock: std::collections::HashMap::new(),
                is_tombstone: false,
                tombstone_type: None,
                deleted_at: None,
                deleted_by: None,
            },
            signature: None,
        };

        let block = UBTSBlock::new(1, None, vec![tx]);

        assert_eq!(block.height, 1);
        assert_eq!(block.transaction_count(), 1);
        assert!(block.id.starts_with("ubts-"));
    }

    #[test]
    fn test_transaction_type_names() {
        let tx = UBTSTransaction::DeleteRelease {
            id: "test-123".to_string(),
            signature: None,
        };

        assert_eq!(tx.type_name(), "DeleteRelease");
        assert!(tx.requires_admin());
    }
}
