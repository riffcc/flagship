//! Delete Block System
//!
//! Implements proof-of-erasure where delete blocks propagate through the network,
//! each node confirms deletion, and once consensus is reached, the delete block
//! itself is pruned.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// DeleteBlock - tracks blocks to be deleted and erasure confirmations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteBlock {
    /// Unique ID for this delete block
    pub id: String,

    /// Block IDs to be deleted
    pub deleted_block_ids: Vec<String>,

    /// Reason for deletion (spam, abuse, copyright, etc.)
    pub reason: DeleteReason,

    /// Public key of admin who initiated the delete
    pub deleted_by: String,

    /// Timestamp when delete was initiated
    pub timestamp: u64,

    /// Peers that have confirmed erasure
    pub erasure_confirmations: HashSet<String>,

    /// Total number of peers that need to confirm
    pub required_confirmations: usize,

    /// Whether this delete block has achieved consensus
    pub consensus_achieved: bool,
}

/// Reason for deletion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeleteReason {
    /// Spam content
    Spam,

    /// Abusive or harmful content
    Abuse,

    /// Copyright violation
    Copyright,

    /// Malware or security threat
    Malware,

    /// User requested deletion
    UserRequest,

    /// Other reason with description
    Other(String),
}

/// Erasure confirmation message from a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureConfirmation {
    /// ID of the delete block being confirmed
    pub delete_block_id: String,

    /// Peer ID confirming erasure
    pub peer_id: String,

    /// Block IDs that were successfully erased
    pub erased_block_ids: Vec<String>,

    /// Timestamp of confirmation
    pub timestamp: u64,

    /// Signature proving this peer erased the blocks
    pub signature: Option<String>,
}

impl DeleteBlock {
    /// Create a new delete block
    pub fn new(
        deleted_block_ids: Vec<String>,
        reason: DeleteReason,
        deleted_by: String,
        required_confirmations: usize,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Generate unique ID from content
        let id = Self::compute_id(&deleted_block_ids, timestamp);

        Self {
            id,
            deleted_block_ids,
            reason,
            deleted_by,
            timestamp,
            erasure_confirmations: HashSet::new(),
            required_confirmations,
            consensus_achieved: false,
        }
    }

    /// Compute delete block ID
    fn compute_id(block_ids: &[String], timestamp: u64) -> String {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        for id in block_ids {
            hasher.update(id.as_bytes());
        }
        hasher.update(timestamp.to_le_bytes());

        let result = hasher.finalize();
        format!("delete-{}", hex::encode(&result[..16])) // Use first 16 bytes
    }

    /// Add an erasure confirmation from a peer
    pub fn add_confirmation(&mut self, peer_id: String) -> bool {
        let newly_added = self.erasure_confirmations.insert(peer_id);

        // Check if we've achieved consensus
        if !self.consensus_achieved && self.erasure_confirmations.len() >= self.required_confirmations {
            self.consensus_achieved = true;
        }

        newly_added
    }

    /// Check if consensus has been achieved
    pub fn has_consensus(&self) -> bool {
        self.consensus_achieved
    }

    /// Get progress toward consensus (0.0 to 1.0)
    pub fn consensus_progress(&self) -> f64 {
        if self.required_confirmations == 0 {
            return 1.0;
        }
        (self.erasure_confirmations.len() as f64) / (self.required_confirmations as f64)
    }

    /// Get the number of blocks to be deleted
    pub fn block_count(&self) -> usize {
        self.deleted_block_ids.len()
    }
}

impl ErasureConfirmation {
    /// Create a new erasure confirmation
    pub fn new(
        delete_block_id: String,
        peer_id: String,
        erased_block_ids: Vec<String>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            delete_block_id,
            peer_id,
            erased_block_ids,
            timestamp,
            signature: None,
        }
    }

    /// Add a signature to this confirmation
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }
}

impl std::fmt::Display for DeleteReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spam => write!(f, "spam"),
            Self::Abuse => write!(f, "abuse"),
            Self::Copyright => write!(f, "copyright"),
            Self::Malware => write!(f, "malware"),
            Self::UserRequest => write!(f, "user_request"),
            Self::Other(desc) => write!(f, "other: {}", desc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_block_creation() {
        let block = DeleteBlock::new(
            vec!["release-123".to_string(), "release-456".to_string()],
            DeleteReason::Spam,
            "admin-pubkey".to_string(),
            3,
        );

        assert_eq!(block.block_count(), 2);
        assert!(!block.has_consensus());
        assert_eq!(block.consensus_progress(), 0.0);
        assert!(block.id.starts_with("delete-"));
    }

    #[test]
    fn test_erasure_confirmations() {
        let mut block = DeleteBlock::new(
            vec!["release-123".to_string()],
            DeleteReason::Abuse,
            "admin".to_string(),
            3,
        );

        // Add confirmations
        assert!(block.add_confirmation("peer-1".to_string()));
        assert_eq!(block.consensus_progress(), 1.0 / 3.0);

        assert!(block.add_confirmation("peer-2".to_string()));
        assert_eq!(block.consensus_progress(), 2.0 / 3.0);
        assert!(!block.has_consensus());

        // Third confirmation achieves consensus
        assert!(block.add_confirmation("peer-3".to_string()));
        assert_eq!(block.consensus_progress(), 1.0);
        assert!(block.has_consensus());

        // Duplicate confirmation doesn't count
        assert!(!block.add_confirmation("peer-1".to_string()));
    }

    #[test]
    fn test_erasure_confirmation_creation() {
        let conf = ErasureConfirmation::new(
            "delete-abc123".to_string(),
            "peer-1".to_string(),
            vec!["release-123".to_string()],
        );

        assert_eq!(conf.delete_block_id, "delete-abc123");
        assert_eq!(conf.erased_block_ids.len(), 1);
        assert!(conf.signature.is_none());
    }
}
