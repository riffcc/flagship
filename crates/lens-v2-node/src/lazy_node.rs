//! LazyNode - Recursive DHT with lazy-loaded topology
//!
//! Implements the LazyNode pattern from Citadel DHT SPEC Section 2.4:
//! - Minimal state (64 bytes: slot + peer_id + mesh_config + epoch)
//! - On-demand neighbor discovery via DHT queries
//! - Ephemeral cache (10s TTL) to reduce repeated DHT GETs
//! - No persistent neighbor cache, no routing tables
//!
//! This is the CORE of Citadel's recursive DHT architecture where the DHT
//! uses itself for topology discovery.
//!
//! Key concepts:
//! 1. **Lazy Loading**: Query neighbors only when needed
//! 2. **Ephemeral Cache**: Short-lived (10s) cache to reduce DHT queries
//! 3. **Minimal State**: ~64 bytes per node (no neighbor lists)
//! 4. **DHT Recursion**: DHT stores its own topology

use anyhow::{anyhow, Result};
use citadel_core::topology::{Direction, MeshConfig, SlotCoordinate};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

use crate::peer_registry::{get_neighbor_slots, slot_ownership_key, SlotOwnership};

/// Callback function for DHT GET operations
///
/// This routes DHT GET requests via network (relay or WebRTC) instead of querying local storage.
/// Returns Ok(Some(value)) if key found, Ok(None) if not found, Err if network error.
type DhtGetFn = Arc<dyn Fn([u8; 32]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>>> + Send>> + Send + Sync>;

/// Cache entry for a neighbor
#[derive(Debug, Clone)]
struct CacheEntry {
    direction: Direction,
    peer_id: String,
    cached_at: Instant,
}

/// LazyNode - Minimal state DHT node with lazy neighbor discovery
///
/// Key properties:
/// - Minimal state: ~64 bytes (slot + peer_id + mesh_config)
/// - Lazy neighbor discovery: query DHT on-demand
/// - Ephemeral cache: 10s TTL to reduce repeated queries
/// - No persistent neighbor cache
/// - No routing tables
pub struct LazyNode {
    /// Our slot coordinate in the mesh (12 bytes: 3 × i32)
    my_slot: SlotCoordinate,

    /// Our peer ID (~32 bytes)
    my_peer_id: String,

    /// Mesh configuration (12-24 bytes: 3 × usize)
    mesh_config: MeshConfig,

    /// DHT storage (shared with relay and sync orchestrator)
    /// Made pub(crate) so sync_orchestrator can populate DHT from PeerReferral events
    pub(crate) dht_storage: Arc<Mutex<crate::dht_state::DhtState>>,

    /// DHT GET callback for network routing
    /// Routes DHT GET requests via relay or WebRTC instead of querying local storage
    dht_get_fn: DhtGetFn,

    /// Ephemeral neighbor cache (10s TTL)
    /// Vec of (direction, peer_id, cached_at) tuples
    neighbor_cache: Arc<RwLock<Vec<CacheEntry>>>,

    /// Cache TTL (default: 10 seconds = 1 epoch)
    cache_ttl: Duration,
}

impl LazyNode {
    /// Create a new LazyNode with default 10s cache TTL
    ///
    /// # Arguments
    /// * `my_slot` - Our slot coordinate in the mesh
    /// * `my_peer_id` - Our peer ID
    /// * `mesh_config` - Mesh configuration
    /// * `dht_storage` - Shared DHT storage
    /// * `dht_get_fn` - Callback for network DHT GET routing
    pub fn new(
        my_slot: SlotCoordinate,
        my_peer_id: String,
        mesh_config: MeshConfig,
        dht_storage: Arc<Mutex<crate::dht_state::DhtState>>,
        dht_get_fn: DhtGetFn,
    ) -> Self {
        info!(
            "🔷 LazyNode created: peer={}, slot={:?}, mesh={}×{}×{}",
            my_peer_id, my_slot, mesh_config.width, mesh_config.height, mesh_config.depth
        );

        Self {
            my_slot,
            my_peer_id,
            mesh_config,
            dht_storage,
            dht_get_fn,
            neighbor_cache: Arc::new(RwLock::new(Vec::new())),
            cache_ttl: Duration::from_secs(10), // 10s TTL (1 epoch)
        }
    }

    /// Create a new LazyNode with custom cache TTL
    pub fn with_cache_ttl(
        my_slot: SlotCoordinate,
        my_peer_id: String,
        mesh_config: MeshConfig,
        dht_storage: Arc<Mutex<crate::dht_state::DhtState>>,
        dht_get_fn: DhtGetFn,
        cache_ttl: Duration,
    ) -> Self {
        info!(
            "🔷 LazyNode created: peer={}, slot={:?}, mesh={}×{}×{}, cache_ttl={:?}",
            my_peer_id, my_slot, mesh_config.width, mesh_config.height, mesh_config.depth, cache_ttl
        );

        Self {
            my_slot,
            my_peer_id,
            mesh_config,
            dht_storage,
            dht_get_fn,
            neighbor_cache: Arc::new(RwLock::new(Vec::new())),
            cache_ttl,
        }
    }

    /// Get neighbor in a specific direction (lazy load from DHT)
    ///
    /// This is the CORE of lazy neighbor discovery:
    /// 1. Check ephemeral cache (10s TTL)
    /// 2. If miss, calculate neighbor_slot = my_slot.neighbor(direction)
    /// 3. Query DHT: slot_ownership_key(neighbor_slot)
    /// 4. Parse SlotOwnership and extract peer_id
    /// 5. Update cache and return
    ///
    /// # Arguments
    /// * `direction` - Direction to neighbor (PlusA, MinusA, etc.)
    ///
    /// # Returns
    /// * `Ok(peer_id)` - Neighbor's peer ID
    /// * `Err(e)` - Neighbor not found or DHT error
    pub async fn get_neighbor(&self, direction: Direction) -> Result<String> {
        // 1. Check ephemeral cache first (10s TTL)
        {
            let cache = self.neighbor_cache.read().await;
            if let Some(entry) = cache.iter().find(|e| e.direction == direction) {
                if entry.cached_at.elapsed() < self.cache_ttl {
                    debug!(
                        "🔷 Cache HIT: neighbor {:?} = {} (age: {:?})",
                        direction,
                        entry.peer_id,
                        entry.cached_at.elapsed()
                    );
                    return Ok(entry.peer_id.clone());
                } else {
                    debug!(
                        "🔷 Cache EXPIRED: neighbor {:?} (age: {:?} > TTL: {:?})",
                        direction,
                        entry.cached_at.elapsed(),
                        self.cache_ttl
                    );
                }
            }
        }

        // 2. Cache miss - calculate neighbor slot
        let neighbor_slot = self.my_slot.neighbor(direction, &self.mesh_config);
        debug!(
            "🔷 Cache MISS: querying DHT for neighbor {:?} at slot {:?}",
            direction, neighbor_slot
        );

        // Event-driven heartbeat: Update BEFORE querying - proves we're active
        // This MUST happen before the query so it updates even if neighbor is stale!
        if let Err(e) = self.update_heartbeat().await {
            warn!("Failed to update heartbeat before DHT read: {}", e);
        }

        // 3. Query DHT via network routing (NOT local storage!)
        let ownership_key = slot_ownership_key(neighbor_slot);

        let ownership_bytes = (self.dht_get_fn)(ownership_key)
            .await?
            .ok_or_else(|| anyhow!("Neighbor slot {:?} not found in DHT", neighbor_slot))?;

        // 4. Parse SlotOwnership and extract peer_id
        let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes)
            .map_err(|e| anyhow!("Failed to parse SlotOwnership: {}", e))?;

        // Skip stale peers
        if ownership.is_stale() {
            return Err(anyhow!("Neighbor {} at slot {:?} is stale", ownership.peer_id, neighbor_slot));
        }

        // Skip ourselves
        if ownership.peer_id == self.my_peer_id {
            return Err(anyhow!("Neighbor slot {:?} contains ourselves", neighbor_slot));
        }

        // 5. Update ephemeral cache
        {
            let mut cache = self.neighbor_cache.write().await;
            // Remove old entry if exists
            cache.retain(|e| e.direction != direction);
            // Add new entry
            cache.push(CacheEntry {
                direction,
                peer_id: ownership.peer_id.clone(),
                cached_at: Instant::now(),
            });
        }

        info!(
            "🔷 DHT neighbor discovered: {:?} = {} at slot {:?}",
            direction, ownership.peer_id, neighbor_slot
        );

        // Event-driven heartbeat: Every DHT read proves we're active
        // This keeps our slot ownership fresh without timers
        if let Err(e) = self.update_heartbeat().await {
            // Log but don't fail the neighbor discovery if heartbeat update fails
            warn!("Failed to update heartbeat after DHT read: {}", e);
        }

        Ok(ownership.peer_id)
    }

    /// Get all 8 neighbors (lazy load from DHT)
    ///
    /// Queries all 8 directions:
    /// - 6 hexagonal (in-plane): ±A, ±B, ±C
    /// - 2 vertical: Up, Down
    ///
    /// # Returns
    /// * `Ok(Vec<peer_id>)` - List of neighbor peer IDs (may be <8 if some missing)
    /// * `Err(e)` - DHT error
    pub async fn get_all_neighbors(&self) -> Result<Vec<String>> {
        let directions = [
            Direction::PlusA,
            Direction::MinusA,
            Direction::PlusB,
            Direction::MinusB,
            Direction::PlusC,
            Direction::MinusC,
            Direction::Up,
            Direction::Down,
        ];

        let mut neighbors = Vec::new();
        let mut not_found_count = 0;

        for direction in &directions {
            match self.get_neighbor(*direction).await {
                Ok(peer_id) => {
                    neighbors.push(peer_id);
                }
                Err(e) => {
                    debug!("Neighbor not found in direction {:?}: {}", direction, e);
                    not_found_count += 1;
                }
            }
        }

        if not_found_count > 0 {
            warn!(
                "🔷 Lazy neighbor discovery: {}/8 neighbors found ({} missing)",
                neighbors.len(),
                not_found_count
            );
        } else {
            info!("🔷 Lazy neighbor discovery: 8/8 neighbors found");
        }

        Ok(neighbors)
    }

    /// Clear the ephemeral cache
    ///
    /// Useful for testing or forcing fresh DHT queries
    pub async fn clear_cache(&self) {
        let mut cache = self.neighbor_cache.write().await;
        cache.clear();
        debug!("🔷 Cleared ephemeral neighbor cache");
    }

    /// Get cache statistics
    ///
    /// Returns (entries, average_age)
    pub async fn cache_stats(&self) -> (usize, Option<Duration>) {
        let cache = self.neighbor_cache.read().await;
        let entries = cache.len();

        if entries == 0 {
            return (0, None);
        }

        let total_age: Duration = cache
            .iter()
            .map(|entry| entry.cached_at.elapsed())
            .sum();

        let avg_age = total_age / entries as u32;

        (entries, Some(avg_age))
    }

    /// Get our slot coordinate
    pub fn my_slot(&self) -> SlotCoordinate {
        self.my_slot
    }

    /// Get our peer ID
    pub fn my_peer_id(&self) -> &str {
        &self.my_peer_id
    }

    /// Get mesh configuration
    pub fn mesh_config(&self) -> &MeshConfig {
        &self.mesh_config
    }

    /// Check if a specific neighbor is cached
    pub async fn is_cached(&self, direction: Direction) -> bool {
        let cache = self.neighbor_cache.read().await;
        if let Some(entry) = cache.iter().find(|e| e.direction == direction) {
            entry.cached_at.elapsed() < self.cache_ttl
        } else {
            false
        }
    }

    /// Get neighbor from cache only (no DHT query)
    ///
    /// Returns `None` if not cached or expired
    pub async fn get_neighbor_cached(&self, direction: Direction) -> Option<String> {
        let cache = self.neighbor_cache.read().await;
        if let Some(entry) = cache.iter().find(|e| e.direction == direction) {
            if entry.cached_at.elapsed() < self.cache_ttl {
                return Some(entry.peer_id.clone());
            }
        }
        None
    }

    /// Announce our presence in the DHT
    ///
    /// Registers "I own this slot" in the DHT so neighbors can find us.
    /// Should be called periodically to maintain heartbeat.
    pub async fn announce_presence(&self, relay_url: Option<String>) -> Result<()> {
        let ownership = SlotOwnership::new(
            self.my_peer_id.clone(),
            self.my_slot,
            relay_url,
        );

        let ownership_key = slot_ownership_key(self.my_slot);
        let ownership_bytes = serde_json::to_vec(&ownership)?;

        let mut dht_storage = self.dht_storage.lock().await;
        dht_storage.insert_raw(ownership_key, ownership_bytes);

        info!("📍 Announced presence in DHT at slot {:?}", self.my_slot);
        Ok(())
    }

    /// Update our heartbeat in the DHT
    ///
    /// Keeps our slot ownership fresh so we don't appear stale to neighbors.
    pub async fn update_heartbeat(&self) -> Result<()> {
        let ownership_key = slot_ownership_key(self.my_slot);
        let mut dht_storage = self.dht_storage.lock().await;

        if let Some(ownership_bytes) = dht_storage.get_raw(&ownership_key) {
            if let Ok(mut ownership) = serde_json::from_slice::<SlotOwnership>(ownership_bytes) {
                ownership.update_heartbeat();
                let updated_bytes = serde_json::to_vec(&ownership)?;
                dht_storage.insert_raw(ownership_key, updated_bytes);
                debug!("💓 Updated heartbeat for slot {:?}", self.my_slot);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Create a DHT GET callback for tests (queries local storage)
    fn create_test_dht_get_fn(dht_storage: Arc<Mutex<crate::dht_state::DhtState>>) -> DhtGetFn {
        Arc::new(move |key: [u8; 32]| {
            let storage = dht_storage.clone();
            Box::pin(async move {
                let dht = storage.lock().await;
                Ok(dht.get_raw(&key).map(|v| v.clone()))
            })
        })
    }

    #[tokio::test]
    async fn test_lazy_node_creation() {
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));
        let dht_get_fn = create_test_dht_get_fn(dht_storage.clone());

        let lazy_node = LazyNode::new(
            my_slot,
            "peer-123".to_string(),
            mesh_config,
            dht_storage,
            dht_get_fn,
        );

        assert_eq!(lazy_node.my_peer_id(), "peer-123");
        assert_eq!(lazy_node.my_slot(), my_slot);
        assert_eq!(lazy_node.mesh_config().width, 10);
    }

    #[tokio::test]
    async fn test_get_all_neighbors_empty_dht() {
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));
        let dht_get_fn = create_test_dht_get_fn(dht_storage.clone());

        let lazy_node = LazyNode::new(
            my_slot,
            "peer-123".to_string(),
            mesh_config,
            dht_storage,
            dht_get_fn,
        );

        // Empty DHT should return no neighbors
        let neighbors = lazy_node.get_all_neighbors().await.unwrap();
        assert_eq!(neighbors.len(), 0);
    }

    #[tokio::test]
    async fn test_get_all_neighbors_with_populated_dht() {
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        // Populate DHT with some neighbors
        {
            let mut storage = dht_storage.lock().await;
            let neighbor_slots = get_neighbor_slots(&my_slot, &mesh_config);

            // Add 3 neighbors to DHT
            for i in 0..3 {
                let (_, neighbor_slot) = &neighbor_slots[i];
                let ownership = SlotOwnership::new(
                    format!("peer-neighbor-{}", i),
                    *neighbor_slot,
                    None,
                );
                let key = slot_ownership_key(*neighbor_slot);
                let value = serde_json::to_vec(&ownership).unwrap();
                storage.insert_raw(key, value);
            }
        }

        let dht_get_fn = create_test_dht_get_fn(dht_storage.clone());
        let lazy_node = LazyNode::new(
            my_slot,
            "peer-123".to_string(),
            mesh_config,
            dht_storage,
            dht_get_fn,
        );

        // Should find 3 neighbors
        let neighbors = lazy_node.get_all_neighbors().await.unwrap();
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&"peer-neighbor-0".to_string()));
        assert!(neighbors.contains(&"peer-neighbor-1".to_string()));
        assert!(neighbors.contains(&"peer-neighbor-2".to_string()));
    }

    #[tokio::test]
    async fn test_announce_presence() {
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));
        let dht_get_fn = create_test_dht_get_fn(dht_storage.clone());

        let lazy_node = LazyNode::new(
            my_slot,
            "peer-123".to_string(),
            mesh_config,
            dht_storage.clone(),
            dht_get_fn,
        );

        // Announce presence
        lazy_node.announce_presence(Some("ws://localhost:5000".to_string())).await.unwrap();

        // Verify stored in DHT
        let storage = dht_storage.lock().await;
        let ownership_key = slot_ownership_key(my_slot);
        let ownership_bytes = storage.get_raw(&ownership_key).unwrap();
        let ownership: SlotOwnership = serde_json::from_slice(ownership_bytes).unwrap();

        assert_eq!(ownership.peer_id, "peer-123");
        assert_eq!(ownership.slot, my_slot);
        assert!(!ownership.is_stale());
    }

    #[tokio::test]
    async fn test_update_heartbeat() {
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));
        let dht_get_fn = create_test_dht_get_fn(dht_storage.clone());

        let lazy_node = LazyNode::new(
            my_slot,
            "peer-123".to_string(),
            mesh_config,
            dht_storage.clone(),
            dht_get_fn,
        );

        // Announce presence
        lazy_node.announce_presence(None).await.unwrap();

        // Get initial heartbeat
        let initial_heartbeat = {
            let storage = dht_storage.lock().await;
            let ownership_key = slot_ownership_key(my_slot);
            let ownership_bytes = storage.get_raw(&ownership_key).unwrap();
            let ownership: SlotOwnership = serde_json::from_slice(ownership_bytes).unwrap();
            ownership.last_heartbeat
        };

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Update heartbeat
        lazy_node.update_heartbeat().await.unwrap();

        // Verify heartbeat updated
        let updated_heartbeat = {
            let storage = dht_storage.lock().await;
            let ownership_key = slot_ownership_key(my_slot);
            let ownership_bytes = storage.get_raw(&ownership_key).unwrap();
            let ownership: SlotOwnership = serde_json::from_slice(ownership_bytes).unwrap();
            ownership.last_heartbeat
        };

        assert!(updated_heartbeat >= initial_heartbeat);
    }

    #[tokio::test]
    async fn test_get_all_neighbors_skips_stale_peers() {
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        // Populate DHT with a stale neighbor
        {
            let mut storage = dht_storage.lock().await;
            let neighbor_slots = get_neighbor_slots(&my_slot, &mesh_config);
            let (_, neighbor_slot) = &neighbor_slots[0];

            let mut ownership = SlotOwnership::new(
                "peer-stale".to_string(),
                *neighbor_slot,
                None,
            );

            // Make it stale (6 minutes old)
            ownership.last_heartbeat = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() - 360;

            let key = slot_ownership_key(*neighbor_slot);
            let value = serde_json::to_vec(&ownership).unwrap();
            storage.insert_raw(key, value);
        }

        let dht_get_fn = create_test_dht_get_fn(dht_storage.clone());
        let lazy_node = LazyNode::new(
            my_slot,
            "peer-123".to_string(),
            mesh_config,
            dht_storage,
            dht_get_fn,
        );

        // Should skip stale neighbor
        let neighbors = lazy_node.get_all_neighbors().await.unwrap();
        assert_eq!(neighbors.len(), 0);
    }

    #[tokio::test]
    async fn test_get_all_neighbors_skips_self() {
        let mesh_config = MeshConfig::new(10, 10, 5);
        let my_slot = SlotCoordinate::new(5, 5, 2);
        let dht_storage = Arc::new(tokio::sync::Mutex::new(crate::dht_state::DhtState::new()));

        // Populate DHT with ourselves as a neighbor (shouldn't happen, but test it)
        {
            let mut storage = dht_storage.lock().await;
            let neighbor_slots = get_neighbor_slots(&my_slot, &mesh_config);
            let (_, neighbor_slot) = &neighbor_slots[0];

            let ownership = SlotOwnership::new(
                "peer-123".to_string(), // Same as our peer ID
                *neighbor_slot,
                None,
            );

            let key = slot_ownership_key(*neighbor_slot);
            let value = serde_json::to_vec(&ownership).unwrap();
            storage.insert_raw(key, value);
        }

        let dht_get_fn = create_test_dht_get_fn(dht_storage.clone());
        let lazy_node = LazyNode::new(
            my_slot,
            "peer-123".to_string(),
            mesh_config,
            dht_storage,
            dht_get_fn,
        );

        // Should skip ourselves
        let neighbors = lazy_node.get_all_neighbors().await.unwrap();
        assert_eq!(neighbors.len(), 0);
    }
}
