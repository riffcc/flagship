//! DHT-Native Join/Leave Announcements for Citadel Recursive DHT
//!
//! Implements efficient single-message announcements for mesh topology changes.
//!
//! **Problem:** Traditional broadcast join/leave sends 80+ messages (8 neighbors × ~10 hops)
//! **Solution:** Single DHT PUT operation to announcement key (1 message)
//!
//! # Architecture
//!
//! - Join: PUT to `join_announcement_key(slot)` with JoinAnnouncement
//! - Leave: PUT tombstone to `leave_announcement_key(slot)` with LeaveAnnouncement
//! - Discovery: Periodic DHT queries to neighbor slots' join announcement keys
//!
//! # Message Reduction
//!
//! - Old broadcast approach: 80 messages (8 neighbors × 10 hops average)
//! - DHT-native approach: 1 message per operation
//! - **80× reduction in announcement traffic!**

use anyhow::{Context, Result};
use citadel_core::topology::{MeshConfig, SlotCoordinate};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::peer_registry::{get_neighbor_slots, slot_ownership_key};

/// Join announcement stored in DHT
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JoinAnnouncement {
    /// Peer ID joining the mesh
    pub peer_id: String,

    /// Slot coordinate they're joining
    pub slot: SlotCoordinate,

    /// Epoch number (increments on rejoin)
    pub epoch: u64,

    /// Timestamp of join (Unix seconds)
    pub timestamp: u64,

    /// Proof-of-work nonce (prevents spam)
    pub pow_nonce: u64,

    /// Relay URL for peer communication (optional)
    pub relay_url: Option<String>,

    /// Signature (ed25519 of serialized announcement)
    pub signature: Option<Vec<u8>>,
}

impl JoinAnnouncement {
    /// Create new join announcement
    pub fn new(peer_id: String, slot: SlotCoordinate, epoch: u64, relay_url: Option<String>) -> Self {
        Self {
            peer_id,
            slot,
            epoch,
            timestamp: now(),
            pow_nonce: 0,
            relay_url,
            signature: None,
        }
    }

    /// Check if announcement is stale (older than 5 minutes)
    pub fn is_stale(&self) -> bool {
        let age_secs = now().saturating_sub(self.timestamp);
        age_secs > 300 // 5 minutes
    }

    /// Verify proof-of-work
    pub fn verify_pow(&self, difficulty: u8) -> bool {
        let hash = self.compute_hash();
        count_leading_zeros(&hash) >= difficulty
    }

    /// Compute proof-of-work
    pub fn compute_pow(&mut self, difficulty: u8) {
        loop {
            let hash = self.compute_hash();
            if count_leading_zeros(&hash) >= difficulty {
                break;
            }
            self.pow_nonce += 1;
        }
    }

    /// Compute Blake3 hash for PoW verification
    fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.peer_id.as_bytes());
        hasher.update(&self.slot.x.to_le_bytes());
        hasher.update(&self.slot.y.to_le_bytes());
        hasher.update(&self.slot.z.to_le_bytes());
        hasher.update(&self.epoch.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.pow_nonce.to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}

/// Leave announcement (tombstone) stored in DHT
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LeaveAnnouncement {
    /// Peer ID leaving the mesh
    pub peer_id: String,

    /// Slot coordinate they're leaving
    pub slot: SlotCoordinate,

    /// Epoch number (must match join epoch)
    pub epoch: u64,

    /// Timestamp of leave (Unix seconds)
    pub timestamp: u64,

    /// Proof-of-work nonce (prevents spam)
    pub pow_nonce: u64,

    /// Signature (ed25519 of serialized announcement)
    pub signature: Option<Vec<u8>>,
}

impl LeaveAnnouncement {
    /// Create new leave announcement
    pub fn new(peer_id: String, slot: SlotCoordinate, epoch: u64) -> Self {
        Self {
            peer_id,
            slot,
            epoch,
            timestamp: now(),
            pow_nonce: 0,
            signature: None,
        }
    }

    /// Verify proof-of-work
    pub fn verify_pow(&self, difficulty: u8) -> bool {
        let hash = self.compute_hash();
        count_leading_zeros(&hash) >= difficulty
    }

    /// Compute proof-of-work
    pub fn compute_pow(&mut self, difficulty: u8) {
        loop {
            let hash = self.compute_hash();
            if count_leading_zeros(&hash) >= difficulty {
                break;
            }
            self.pow_nonce += 1;
        }
    }

    /// Compute Blake3 hash for PoW verification
    fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.peer_id.as_bytes());
        hasher.update(&self.slot.x.to_le_bytes());
        hasher.update(&self.slot.y.to_le_bytes());
        hasher.update(&self.slot.z.to_le_bytes());
        hasher.update(&self.epoch.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.pow_nonce.to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}

/// Generate DHT key for join announcements at a slot
pub fn join_announcement_key(slot: SlotCoordinate) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"join-announcement");
    hasher.update(&slot.x.to_le_bytes());
    hasher.update(&slot.y.to_le_bytes());
    hasher.update(&slot.z.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Generate DHT key for leave announcements at a slot
pub fn leave_announcement_key(slot: SlotCoordinate) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"leave-announcement");
    hasher.update(&slot.x.to_le_bytes());
    hasher.update(&slot.y.to_le_bytes());
    hasher.update(&slot.z.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Announce join via single DHT PUT operation (1 message vs 80!)
pub async fn announce_join(
    dht_storage: Arc<Mutex<HashMap<[u8; 32], Vec<u8>>>>,
    peer_id: String,
    slot: SlotCoordinate,
    epoch: u64,
    relay_url: Option<String>,
    pow_difficulty: u8,
) -> Result<()> {
    let mut announcement = JoinAnnouncement::new(peer_id.clone(), slot, epoch, relay_url);

    // Compute proof-of-work
    debug!("Computing PoW for join announcement (difficulty: {})", pow_difficulty);
    announcement.compute_pow(pow_difficulty);

    // Generate DHT key
    let key = join_announcement_key(slot);

    // Serialize announcement
    let value = serde_json::to_vec(&announcement)
        .context("Failed to serialize join announcement")?;

    // Single DHT PUT operation
    let mut storage = dht_storage.lock().await;
    storage.insert(key, value);

    info!("📢 Announced join via DHT (1 message): peer={}, slot={:?}, epoch={}",
        peer_id, slot, epoch);

    Ok(())
}

/// Announce leave via single DHT PUT operation (tombstone)
pub async fn announce_leave(
    dht_storage: Arc<Mutex<HashMap<[u8; 32], Vec<u8>>>>,
    peer_id: String,
    slot: SlotCoordinate,
    epoch: u64,
    pow_difficulty: u8,
) -> Result<()> {
    let mut announcement = LeaveAnnouncement::new(peer_id.clone(), slot, epoch);

    // Compute proof-of-work
    debug!("Computing PoW for leave announcement (difficulty: {})", pow_difficulty);
    announcement.compute_pow(pow_difficulty);

    // Generate DHT key
    let key = leave_announcement_key(slot);

    // Serialize tombstone
    let value = serde_json::to_vec(&announcement)
        .context("Failed to serialize leave announcement")?;

    // Single DHT PUT operation (tombstone)
    let mut storage = dht_storage.lock().await;
    storage.insert(key, value);

    info!("💀 Announced leave via DHT (1 message): peer={}, slot={:?}, epoch={}",
        peer_id, slot, epoch);

    Ok(())
}

/// Discover new neighbors by querying DHT for join announcements
///
/// Checks all 8 neighbor slots (6 hexagonal + 2 vertical) for recent join announcements.
/// Returns list of peer IDs that have recently joined neighbor slots.
pub async fn discover_new_neighbors(
    dht_storage: Arc<Mutex<HashMap<[u8; 32], Vec<u8>>>>,
    my_slot: SlotCoordinate,
    mesh_config: MeshConfig,
    pow_difficulty: u8,
) -> Vec<(String, SlotCoordinate)> {
    let neighbor_slots = get_neighbor_slots(&my_slot, &mesh_config);
    let mut discovered_peers = Vec::new();

    let storage = dht_storage.lock().await;

    for (direction, neighbor_slot) in neighbor_slots {
        // Check for join announcement at neighbor slot
        let join_key = join_announcement_key(neighbor_slot);

        if let Some(join_bytes) = storage.get(&join_key) {
            match serde_json::from_slice::<JoinAnnouncement>(join_bytes) {
                Ok(announcement) => {
                    // Verify announcement
                    if !announcement.verify_pow(pow_difficulty) {
                        warn!("⚠️ Invalid PoW for join announcement from {} at slot {:?}",
                            announcement.peer_id, neighbor_slot);
                        continue;
                    }

                    if announcement.is_stale() {
                        debug!("⏰ Stale join announcement from {} at slot {:?} (age: {}s)",
                            announcement.peer_id, neighbor_slot,
                            now().saturating_sub(announcement.timestamp));
                        continue;
                    }

                    // Check if peer has left (tombstone exists)
                    let leave_key = leave_announcement_key(neighbor_slot);
                    if let Some(leave_bytes) = storage.get(&leave_key) {
                        if let Ok(leave_announcement) = serde_json::from_slice::<LeaveAnnouncement>(leave_bytes) {
                            // If leave epoch >= join epoch, peer has left
                            if leave_announcement.epoch >= announcement.epoch {
                                debug!("💀 Peer {} has left slot {:?} (epoch {})",
                                    announcement.peer_id, neighbor_slot, leave_announcement.epoch);
                                continue;
                            }
                        }
                    }

                    // Valid join announcement!
                    info!("✨ Discovered neighbor via DHT: peer={}, slot={:?}, direction={:?}",
                        announcement.peer_id, neighbor_slot, direction);
                    discovered_peers.push((announcement.peer_id.clone(), neighbor_slot));
                }
                Err(e) => {
                    warn!("⚠️ Failed to deserialize join announcement at slot {:?}: {}",
                        neighbor_slot, e);
                }
            }
        }
    }

    info!("🔍 DHT neighbor discovery: found {}/8 neighbors", discovered_peers.len());
    discovered_peers
}

/// Get current Unix timestamp in seconds
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Count leading zero bits in hash (for PoW)
fn count_leading_zeros(hash: &[u8; 32]) -> u8 {
    let mut count: u16 = 0;  // Use u16 to prevent overflow
    for byte in hash {
        if *byte == 0 {
            count += 8;
        } else {
            count += byte.leading_zeros() as u16;
            break;
        }
    }
    count.min(255) as u8  // Cap at 255 (max u8 value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_announcement_creation() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let announcement = JoinAnnouncement::new(
            "peer-123".to_string(),
            slot,
            1,
            Some("ws://localhost:5000".to_string()),
        );

        assert_eq!(announcement.peer_id, "peer-123");
        assert_eq!(announcement.slot, slot);
        assert_eq!(announcement.epoch, 1);
        assert!(!announcement.is_stale());
    }

    #[test]
    fn test_leave_announcement_creation() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let announcement = LeaveAnnouncement::new("peer-123".to_string(), slot, 1);

        assert_eq!(announcement.peer_id, "peer-123");
        assert_eq!(announcement.slot, slot);
        assert_eq!(announcement.epoch, 1);
    }

    #[test]
    fn test_join_announcement_keys_deterministic() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let key1 = join_announcement_key(slot);
        let key2 = join_announcement_key(slot);

        assert_eq!(key1, key2, "Same slot should produce same key");
    }

    #[test]
    fn test_leave_announcement_keys_deterministic() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let key1 = leave_announcement_key(slot);
        let key2 = leave_announcement_key(slot);

        assert_eq!(key1, key2, "Same slot should produce same key");
    }

    #[test]
    fn test_join_and_leave_keys_different() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let join_key = join_announcement_key(slot);
        let leave_key = leave_announcement_key(slot);

        assert_ne!(join_key, leave_key, "Join and leave keys must be different");
    }

    #[test]
    fn test_pow_computation_low_difficulty() {
        let slot = SlotCoordinate::new(0, 0, 0);
        let mut announcement = JoinAnnouncement::new("test".to_string(), slot, 0, None);

        // Low difficulty should complete quickly
        announcement.compute_pow(4);
        assert!(announcement.verify_pow(4));
    }

    #[test]
    fn test_pow_verification_fails_wrong_nonce() {
        let slot = SlotCoordinate::new(0, 0, 0);
        let mut announcement = JoinAnnouncement::new("test".to_string(), slot, 0, None);

        announcement.compute_pow(4);
        let valid_nonce = announcement.pow_nonce;

        // Change nonce - should fail verification
        announcement.pow_nonce = valid_nonce + 1;
        assert!(!announcement.verify_pow(4));
    }

    #[test]
    fn test_staleness_detection() {
        let slot = SlotCoordinate::new(0, 0, 0);
        let mut announcement = JoinAnnouncement::new("test".to_string(), slot, 0, None);

        // Fresh announcement should not be stale
        assert!(!announcement.is_stale());

        // Old announcement should be stale
        announcement.timestamp = now() - 600; // 10 minutes ago
        assert!(announcement.is_stale());
    }

    #[tokio::test]
    async fn test_announce_join_single_message() {
        let storage = Arc::new(Mutex::new(HashMap::new()));
        let slot = SlotCoordinate::new(5, 10, 2);

        announce_join(
            storage.clone(),
            "peer-123".to_string(),
            slot,
            1,
            Some("ws://localhost:5000".to_string()),
            4,
        ).await.unwrap();

        // Verify announcement is in DHT
        let key = join_announcement_key(slot);
        let storage_lock = storage.lock().await;
        let stored = storage_lock.get(&key);

        assert!(stored.is_some(), "Join announcement should be stored in DHT");

        let announcement: JoinAnnouncement = serde_json::from_slice(stored.unwrap()).unwrap();
        assert_eq!(announcement.peer_id, "peer-123");
        assert_eq!(announcement.slot, slot);
        assert_eq!(announcement.epoch, 1);
    }

    #[tokio::test]
    async fn test_announce_leave_tombstone() {
        let storage = Arc::new(Mutex::new(HashMap::new()));
        let slot = SlotCoordinate::new(5, 10, 2);

        announce_leave(
            storage.clone(),
            "peer-123".to_string(),
            slot,
            1,
            4,
        ).await.unwrap();

        // Verify tombstone is in DHT
        let key = leave_announcement_key(slot);
        let storage_lock = storage.lock().await;
        let stored = storage_lock.get(&key);

        assert!(stored.is_some(), "Leave announcement (tombstone) should be stored in DHT");

        let announcement: LeaveAnnouncement = serde_json::from_slice(stored.unwrap()).unwrap();
        assert_eq!(announcement.peer_id, "peer-123");
        assert_eq!(announcement.slot, slot);
        assert_eq!(announcement.epoch, 1);
    }

    #[tokio::test]
    async fn test_discover_new_neighbors() {
        let storage = Arc::new(Mutex::new(HashMap::new()));
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);

        // Announce join for one of our neighbors
        let neighbor_slot = my_slot.neighbor(citadel_core::topology::Direction::PlusA, &mesh_config);
        announce_join(
            storage.clone(),
            "neighbor-peer".to_string(),
            neighbor_slot,
            1,
            Some("ws://localhost:5001".to_string()),
            4,
        ).await.unwrap();

        // Discover neighbors
        let discovered = discover_new_neighbors(storage.clone(), my_slot, mesh_config, 4).await;

        assert_eq!(discovered.len(), 1, "Should discover one neighbor");
        assert_eq!(discovered[0].0, "neighbor-peer");
        assert_eq!(discovered[0].1, neighbor_slot);
    }

    #[tokio::test]
    async fn test_discover_ignores_left_peers() {
        let storage = Arc::new(Mutex::new(HashMap::new()));
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);

        // Announce join for a neighbor
        let neighbor_slot = my_slot.neighbor(citadel_core::topology::Direction::PlusB, &mesh_config);
        announce_join(
            storage.clone(),
            "neighbor-peer".to_string(),
            neighbor_slot,
            1,
            Some("ws://localhost:5001".to_string()),
            4,
        ).await.unwrap();

        // Announce leave with same epoch
        announce_leave(
            storage.clone(),
            "neighbor-peer".to_string(),
            neighbor_slot,
            1,
            4,
        ).await.unwrap();

        // Discover neighbors - should not find the peer that left
        let discovered = discover_new_neighbors(storage.clone(), my_slot, mesh_config, 4).await;

        assert_eq!(discovered.len(), 0, "Should not discover peers that have left");
    }

    #[tokio::test]
    async fn test_discover_handles_rejoin_with_higher_epoch() {
        let storage = Arc::new(Mutex::new(HashMap::new()));
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);

        let neighbor_slot = my_slot.neighbor(citadel_core::topology::Direction::PlusC, &mesh_config);

        // First join (epoch 1)
        announce_join(
            storage.clone(),
            "neighbor-peer".to_string(),
            neighbor_slot,
            1,
            Some("ws://localhost:5001".to_string()),
            4,
        ).await.unwrap();

        // Leave (epoch 1)
        announce_leave(
            storage.clone(),
            "neighbor-peer".to_string(),
            neighbor_slot,
            1,
            4,
        ).await.unwrap();

        // Rejoin with higher epoch (epoch 2)
        announce_join(
            storage.clone(),
            "neighbor-peer".to_string(),
            neighbor_slot,
            2,
            Some("ws://localhost:5001".to_string()),
            4,
        ).await.unwrap();

        // Discover neighbors - should find the rejoined peer
        let discovered = discover_new_neighbors(storage.clone(), my_slot, mesh_config, 4).await;

        assert_eq!(discovered.len(), 1, "Should discover rejoined peer");
        assert_eq!(discovered[0].0, "neighbor-peer");
    }

    #[test]
    fn test_count_leading_zeros() {
        // All zeros (capped at 255, the max value for u8)
        let hash = [0u8; 32];
        assert_eq!(count_leading_zeros(&hash), 255); // 32*8=256 bits, but capped at 255

        // No leading zeros
        let mut hash = [0u8; 32];
        hash[0] = 0b10000000;
        assert_eq!(count_leading_zeros(&hash), 0);

        // 4 leading zeros
        let mut hash = [0u8; 32];
        hash[0] = 0b00001111;
        assert_eq!(count_leading_zeros(&hash), 4);

        // 8 leading zeros (first byte is 0)
        let mut hash = [0u8; 32];
        hash[1] = 0b10000000;
        assert_eq!(count_leading_zeros(&hash), 8);
    }
}
