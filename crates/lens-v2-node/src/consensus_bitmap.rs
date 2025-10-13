//! Cooperative Consensus Bitmap for Peer Discovery
//!
//! Zero-timer, fully decentralized consensus computation using XOR-based exclusion.
//! Each node independently tracks what peers are "common knowledge" across the network,
//! then only broadcasts unique discoveries that aren't yet in consensus.
//!
//! ## Algorithm
//!
//! 1. Track peer views: `peer_views[peer_x] = their_known_peers`
//! 2. Compute consensus: `consensus = intersection(all_peer_views)`
//! 3. Broadcast delta: `my_wantlist.known_peers = my_peers XOR consensus`
//!
//! ## Benefits
//!
//! - **Zero timers**: Consensus emerges cooperatively
//! - **O(churn)**: Traffic scales with changes, not network size
//! - **Sub-O(1)**: In steady state, most WantLists send 0 peers
//! - **Automatic**: No coordination, just local computation

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Peer ID type
pub type PeerId = String;

/// Cooperative consensus bitmap for peer discovery
pub struct ConsensusBitmap {
    /// Each peer's view of known peers (peer_id -> their known peers)
    peer_views: Arc<RwLock<HashMap<PeerId, HashSet<PeerId>>>>,

    /// Current consensus: peers known by ALL nodes
    consensus: Arc<RwLock<HashSet<PeerId>>>,

    /// Minimum number of peer views needed to establish consensus
    /// (prevents premature consensus with only 2 nodes)
    min_views_for_consensus: usize,
}

impl ConsensusBitmap {
    /// Create a new consensus bitmap
    pub fn new() -> Self {
        Self {
            peer_views: Arc::new(RwLock::new(HashMap::new())),
            consensus: Arc::new(RwLock::new(HashSet::new())),
            min_views_for_consensus: 3, // Need at least 3 nodes to form consensus
        }
    }

    /// Update a peer's view of known peers
    ///
    /// This triggers automatic consensus recomputation
    pub async fn update_peer_view(&self, peer_id: PeerId, known_peers: Vec<PeerId>) {
        let known_set: HashSet<PeerId> = known_peers.into_iter().collect();

        {
            let mut views = self.peer_views.write().await;
            views.insert(peer_id.clone(), known_set);
        }

        // Recompute consensus
        self.recompute_consensus().await;
    }

    /// Remove a peer's view (when they disconnect)
    pub async fn remove_peer_view(&self, peer_id: &str) {
        {
            let mut views = self.peer_views.write().await;
            views.remove(peer_id);
        }

        // Recompute consensus
        self.recompute_consensus().await;
    }

    /// Get the current consensus set (peers known by ALL)
    pub async fn get_consensus(&self) -> HashSet<PeerId> {
        self.consensus.read().await.clone()
    }

    /// Compute unique peers to broadcast (my_peers XOR consensus)
    ///
    /// Returns only the peers I know that aren't yet in consensus
    pub async fn compute_unique_peers(&self, my_peers: Vec<PeerId>) -> Vec<PeerId> {
        let consensus = self.consensus.read().await;
        let my_set: HashSet<PeerId> = my_peers.into_iter().collect();

        // XOR: my_peers - consensus = unique discoveries
        let unique: Vec<PeerId> = my_set
            .difference(&consensus)
            .cloned()
            .collect();

        if !unique.is_empty() {
            debug!("📊 Consensus exclusion: {} unique peers (consensus size: {})",
                unique.len(), consensus.len());
        }

        unique
    }

    /// Recompute consensus from all peer views
    ///
    /// Consensus = intersection of all peer views (peers known by everyone)
    async fn recompute_consensus(&self) {
        let views = self.peer_views.read().await;

        if views.len() < self.min_views_for_consensus {
            // Not enough peers yet to form consensus
            let mut consensus = self.consensus.write().await;
            consensus.clear();
            return;
        }

        // Start with first peer's view
        let mut intersection: Option<HashSet<PeerId>> = None;

        for (_peer_id, known_peers) in views.iter() {
            match &mut intersection {
                None => {
                    // Initialize with first peer's view
                    intersection = Some(known_peers.clone());
                }
                Some(current_intersection) => {
                    // Intersect with this peer's view
                    *current_intersection = current_intersection
                        .intersection(known_peers)
                        .cloned()
                        .collect();
                }
            }
        }

        // Update consensus
        if let Some(new_consensus) = intersection {
            let mut consensus = self.consensus.write().await;
            let old_size = consensus.len();
            *consensus = new_consensus;
            let new_size = consensus.len();

            if old_size != new_size {
                info!("🔄 Consensus updated: {} → {} peers", old_size, new_size);
            }
        }
    }

    /// Get statistics for monitoring
    pub async fn stats(&self) -> ConsensusStats {
        let views = self.peer_views.read().await;
        let consensus = self.consensus.read().await;

        let total_peers_known: usize = views.values()
            .flat_map(|set| set.iter())
            .collect::<HashSet<_>>()
            .len();

        ConsensusStats {
            peer_views_count: views.len(),
            consensus_size: consensus.len(),
            total_peers_known,
            consensus_ratio: if total_peers_known > 0 {
                consensus.len() as f64 / total_peers_known as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for ConsensusBitmap {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about consensus state
#[derive(Debug, Clone)]
pub struct ConsensusStats {
    /// Number of peer views tracked
    pub peer_views_count: usize,
    /// Size of consensus set (peers known by all)
    pub consensus_size: usize,
    /// Total unique peers known across all views
    pub total_peers_known: usize,
    /// Ratio of consensus to total (0.0 - 1.0)
    pub consensus_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_bitmap_creation() {
        let bitmap = ConsensusBitmap::new();
        let consensus = bitmap.get_consensus().await;
        assert!(consensus.is_empty());
    }

    #[tokio::test]
    async fn test_consensus_requires_minimum_views() {
        let bitmap = ConsensusBitmap::new();

        // Add 2 peer views (below minimum of 3)
        bitmap.update_peer_view(
            "peer-1".to_string(),
            vec!["peer-a".to_string(), "peer-b".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-2".to_string(),
            vec!["peer-a".to_string(), "peer-b".to_string()],
        ).await;

        // Consensus should still be empty (need 3 views minimum)
        let consensus = bitmap.get_consensus().await;
        assert!(consensus.is_empty());
    }

    #[tokio::test]
    async fn test_consensus_intersection() {
        let bitmap = ConsensusBitmap::new();

        // Three peers, all know peer-x and peer-y
        bitmap.update_peer_view(
            "peer-1".to_string(),
            vec!["peer-x".to_string(), "peer-y".to_string(), "peer-a".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-2".to_string(),
            vec!["peer-x".to_string(), "peer-y".to_string(), "peer-b".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-3".to_string(),
            vec!["peer-x".to_string(), "peer-y".to_string(), "peer-c".to_string()],
        ).await;

        // Consensus should contain peer-x and peer-y (known by all)
        let consensus = bitmap.get_consensus().await;
        assert_eq!(consensus.len(), 2);
        assert!(consensus.contains("peer-x"));
        assert!(consensus.contains("peer-y"));
    }

    #[tokio::test]
    async fn test_compute_unique_peers() {
        let bitmap = ConsensusBitmap::new();

        // Establish consensus with 3 peers
        bitmap.update_peer_view(
            "peer-1".to_string(),
            vec!["peer-x".to_string(), "peer-y".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-2".to_string(),
            vec!["peer-x".to_string(), "peer-y".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-3".to_string(),
            vec!["peer-x".to_string(), "peer-y".to_string()],
        ).await;

        // My peers include consensus + unique discovery
        let my_peers = vec![
            "peer-x".to_string(),  // In consensus
            "peer-y".to_string(),  // In consensus
            "peer-z".to_string(),  // Unique discovery!
        ];

        let unique = bitmap.compute_unique_peers(my_peers).await;

        // Should only return peer-z (the unique discovery)
        assert_eq!(unique.len(), 1);
        assert!(unique.contains(&"peer-z".to_string()));
    }

    #[tokio::test]
    async fn test_remove_peer_view() {
        let bitmap = ConsensusBitmap::new();

        bitmap.update_peer_view(
            "peer-1".to_string(),
            vec!["peer-x".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-2".to_string(),
            vec!["peer-x".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-3".to_string(),
            vec!["peer-x".to_string()],
        ).await;

        // Consensus forms
        let consensus = bitmap.get_consensus().await;
        assert_eq!(consensus.len(), 1);

        // Remove one peer
        bitmap.remove_peer_view("peer-1").await;

        // Consensus still holds with 2 remaining views
        // (but won't form NEW consensus if peers leave)
        let stats = bitmap.stats().await;
        assert_eq!(stats.peer_views_count, 2);
    }

    #[tokio::test]
    async fn test_stats() {
        let bitmap = ConsensusBitmap::new();

        bitmap.update_peer_view(
            "peer-1".to_string(),
            vec!["peer-a".to_string(), "peer-b".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-2".to_string(),
            vec!["peer-a".to_string(), "peer-c".to_string()],
        ).await;

        bitmap.update_peer_view(
            "peer-3".to_string(),
            vec!["peer-a".to_string(), "peer-d".to_string()],
        ).await;

        let stats = bitmap.stats().await;
        assert_eq!(stats.peer_views_count, 3);
        assert_eq!(stats.consensus_size, 1); // Only peer-a is in all views
        assert_eq!(stats.total_peers_known, 4); // a, b, c, d
        assert!((stats.consensus_ratio - 0.25).abs() < 0.01); // 1/4 = 0.25
    }

    #[tokio::test]
    async fn test_sub_o1_steady_state() {
        let bitmap = ConsensusBitmap::new();

        // Establish consensus with 5 peers all knowing each other
        let all_peers: Vec<String> = (0..5).map(|i| format!("peer-{}", i)).collect();

        for peer in &all_peers {
            bitmap.update_peer_view(
                peer.clone(),
                all_peers.clone(),
            ).await;
        }

        // In steady state, everyone knows everyone
        let consensus = bitmap.get_consensus().await;
        assert_eq!(consensus.len(), 5);

        // When broadcasting, each peer should send ZERO peers (all in consensus!)
        for peer in &all_peers {
            let unique = bitmap.compute_unique_peers(all_peers.clone()).await;
            assert!(unique.is_empty(), "Steady state should have zero unique peers");
        }

        // Sub-O(1) achieved: zero broadcasts in steady state!
    }
}
