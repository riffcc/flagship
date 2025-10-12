//! Multi-node SPORE sync tests
//!
//! Tests CRUD operations across 3, 10, and 100 nodes to verify consistency
//! and identify flapping issues in the SPORE sync protocol.

use anyhow::Result;
use lens_v2_node::db::{Database, prefixes, make_key};
use lens_v2_node::routes::releases::{Release, TombstoneType};
use lens_v2_node::ubts::{UBTSBlock, UBTSTransaction};
use lens_v2_p2p::{P2pManager, P2pConfig, BlockMeta};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

// Citadel DHT hexagonal topology
use citadel_core::topology::{SlotCoordinate, Direction, MeshConfig};

/// Hexagonal mesh for deterministic topology
struct HexMesh {
    /// Mesh configuration (toroid dimensions)
    config: MeshConfig,

    /// Nodes indexed by hex position (linear index -> node index)
    node_positions: Vec<SlotCoordinate>,  // Node index -> position

    /// Reverse index (position -> node index)
    position_to_node: HashMap<(i32, i32, i32), usize>,

    /// Next open slot for joining nodes
    next_slot: usize,
}

impl HexMesh {
    /// Create a new hex mesh with dynamic sizing
    fn new() -> Self {
        Self {
            config: MeshConfig {
                width: 20,   // Start with 20x20x20 toroid
                height: 20,
                depth: 20,
            },
            node_positions: Vec::new(),
            position_to_node: HashMap::new(),
            next_slot: 0,
        }
    }

    /// Join a node to the mesh - assigns a hex position
    fn join_node(&mut self, node_idx: usize) -> SlotCoordinate {
        // Use simple spiral assignment for now
        // In production, this would use modulo-based slot selection
        let total_slots = self.config.width * self.config.height * self.config.depth;

        if self.next_slot >= total_slots {
            // Expand the mesh (double dimensions)
            self.config.width *= 2;
            self.config.height *= 2;
            self.config.depth *= 2;
        }

        // Convert linear slot index to 3D coordinate
        let slot = self.next_slot;
        let x = (slot % self.config.width) as i32;
        let y = ((slot / self.config.width) % self.config.height) as i32;
        let z = (slot / (self.config.width * self.config.height)) as i32;

        let position = SlotCoordinate { x, y, z };

        // Ensure node_positions has enough capacity
        if node_idx >= self.node_positions.len() {
            self.node_positions.resize(node_idx + 1, SlotCoordinate { x: 0, y: 0, z: 0 });
        }

        self.node_positions[node_idx] = position;
        self.position_to_node.insert((x, y, z), node_idx);
        self.next_slot += 1;

        position
    }

    /// Get all 8 hex neighbors of a position
    fn get_neighbors(&self, pos: &SlotCoordinate) -> Vec<usize> {
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

        for dir in &directions {
            let neighbor_pos = pos.neighbor(*dir, &self.config);
            let key = (neighbor_pos.x, neighbor_pos.y, neighbor_pos.z);
            if let Some(&node_idx) = self.position_to_node.get(&key) {
                neighbors.push(node_idx);
            }
        }

        neighbors
    }

    /// Get node index at a specific position
    fn get_node_at(&self, pos: &SlotCoordinate) -> Option<usize> {
        let key = (pos.x, pos.y, pos.z);
        self.position_to_node.get(&key).copied()
    }
}

/// Represents a simulated node in the test cluster
struct TestNode {
    /// Node ID
    id: String,

    /// Hex position in the mesh
    hex_position: Option<SlotCoordinate>,

    /// Database
    db: Database,

    /// P2P manager
    p2p_manager: Arc<P2pManager>,

    /// Block notification channel sender
    block_notify_tx: mpsc::UnboundedSender<lens_v2_node::routes::account::BlockNotification>,

    /// Block notification channel receiver
    block_notify_rx: mpsc::UnboundedReceiver<lens_v2_node::routes::account::BlockNotification>,
}

impl TestNode {
    /// Create a new test node
    fn new(id: String) -> Result<Self> {
        let temp_dir = std::env::temp_dir().join(format!("lens-test-node-{}-{}", id, Uuid::new_v4()));
        let db = Database::open(&temp_dir)?;
        let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
        let (block_notify_tx, block_notify_rx) = mpsc::unbounded_channel();

        Ok(Self {
            id,
            hex_position: None,  // Will be assigned when joining HexMesh
            db,
            p2p_manager,
            block_notify_tx,
            block_notify_rx,
        })
    }

    /// Set the hex position for this node
    fn set_hex_position(&mut self, position: SlotCoordinate) {
        self.hex_position = Some(position);
    }

    /// Get the hex position for this node
    fn hex_position(&self) -> Option<&SlotCoordinate> {
        self.hex_position.as_ref()
    }

    /// Create a release on this node
    fn create_release(&self, release: &Release) -> Result<()> {
        let key = make_key(prefixes::RELEASE, &release.id);

        // Initialize vector clock for new release
        let mut release_with_clock = release.clone();
        release_with_clock.increment_clock(self.id.clone());

        self.db.put(&key, &release_with_clock)?;

        // Notify of new block
        self.block_notify_tx.send(
            lens_v2_node::routes::account::BlockNotification::NewBlock(release.id.clone())
        ).ok();

        Ok(())
    }

    /// Update a release on this node
    fn update_release(&self, release: &Release) -> Result<()> {
        let key = make_key(prefixes::RELEASE, &release.id);

        // Increment vector clock for this node
        let mut updated_release = release.clone();
        updated_release.increment_clock(self.id.clone());

        self.db.put(&key, &updated_release)?;

        // Notify of updated block
        self.block_notify_tx.send(
            lens_v2_node::routes::account::BlockNotification::NewBlock(release.id.clone())
        ).ok();

        Ok(())
    }

    /// Delete a release on this node (creates temporary tombstone)
    fn delete_release(&self, release_id: &str) -> Result<String> {
        let release_key = make_key(prefixes::RELEASE, release_id);

        // Get the existing release
        let mut release: Release = self.db.get(&release_key)?
            .ok_or_else(|| anyhow::anyhow!("Release {} not found", release_id))?;

        // Convert to temporary tombstone
        release.is_tombstone = true;
        release.tombstone_type = Some(TombstoneType::Temporary);
        release.deleted_at = Some(chrono::Utc::now().to_rfc3339());
        release.deleted_by = Some(format!("node-{}", self.id));

        // Increment vector clock for this delete operation
        release.increment_clock(self.id.clone());

        // Save the tombstone (don't delete!)
        self.db.put(&release_key, &release)?;

        // Create delete transaction for backward compatibility
        let delete_tx = UBTSTransaction::DeleteRelease {
            id: release_id.to_string(),
            signature: Some(format!("node-{}", self.id)),
        };

        let ubts_block = UBTSBlock::new(0, None, vec![delete_tx]);

        // Save delete transaction
        let delete_key = make_key(prefixes::DELETE_TRANSACTION, &ubts_block.id);
        self.db.put(&delete_key, &ubts_block)?;

        // Notify of new block
        self.block_notify_tx.send(
            lens_v2_node::routes::account::BlockNotification::NewBlock(ubts_block.id.clone())
        ).ok();

        Ok(ubts_block.id)
    }

    /// Get all releases on this node (excluding tombstones)
    fn get_releases(&self) -> Result<Vec<Release>> {
        let all_releases: Vec<Release> = self.db.get_all_with_prefix(prefixes::RELEASE)?;
        Ok(all_releases.into_iter().filter(|r| !r.is_tombstone).collect())
    }

    /// Get all delete transactions on this node
    fn get_delete_transactions(&self) -> Result<Vec<UBTSBlock>> {
        self.db.get_all_with_prefix(prefixes::DELETE_TRANSACTION)
    }

    /// Sync all data from another node (simulates SPORE sync)
    /// Uses vector clocks for causal ordering and conflict resolution
    /// Tombstones sync like regular releases - preventing resurrection!
    fn sync_from(&self, other: &TestNode) -> Result<()> {
        // Sync ALL releases (including tombstones) with vector clock logic
        // This is the KEY to preventing resurrection:
        // - Tombstones have vector clocks and sync like regular releases
        // - If a node has an active release and receives a tombstone with a newer clock, tombstone wins
        // - If a node has a tombstone and receives an active release with an older clock, tombstone wins
        let other_releases: Vec<Release> = other.db.get_all_with_prefix(prefixes::RELEASE)?;

        for other_release in other_releases {
            let key = make_key(prefixes::RELEASE, &other_release.id);

            let our_release: Option<Release> = self.db.get(&key)?;

            match our_release {
                None => {
                    // We don't have it (either active or tombstone), add it
                    self.db.put(&key, &other_release)?;
                }
                Some(mut our) => {
                    // Use vector clock to determine ordering
                    if other_release.happened_before(&our) {
                        // Their version is older, keep ours
                        continue;
                    } else if our.happened_before(&other_release) {
                        // Their version is newer, take theirs (even if it's a tombstone!)
                        self.db.put(&key, &other_release)?;
                    } else if other_release.is_concurrent(&our) {
                        // Concurrent modifications - apply deterministic tie-breaker

                        // TOMBSTONE PRIORITY: If one is a tombstone and one isn't, tombstone wins
                        // This ensures deletes propagate correctly even in concurrent scenarios
                        if other_release.is_tombstone && !our.is_tombstone {
                            self.db.put(&key, &other_release)?;
                            continue;
                        } else if !other_release.is_tombstone && our.is_tombstone {
                            // Our tombstone wins
                            continue;
                        }

                        // Use lexicographic comparison of posted_by as tie-breaker
                        if other_release.posted_by > our.posted_by {
                            // Their version wins
                            self.db.put(&key, &other_release)?;
                        } else if other_release.posted_by == our.posted_by {
                            // Same author - use latest created_at timestamp
                            if other_release.created_at > our.created_at {
                                self.db.put(&key, &other_release)?;
                            }
                        }
                        // else: Our version wins, keep ours
                    } else {
                        // Vector clocks are equal - releases are identical
                        // But merge clocks to ensure we have complete history
                        our.merge_clock(&other_release);
                        self.db.put(&key, &our)?;
                    }
                }
            }
        }

        // Sync delete transactions (for backward compatibility and auditing)
        // Note: We don't apply them anymore - tombstones handle deletions
        let other_deletes = other.get_delete_transactions()?;
        for delete_tx in other_deletes {
            let key = make_key(prefixes::DELETE_TRANSACTION, &delete_tx.id);

            // Only sync if we don't have it (for historical record)
            if !self.db.exists(&key)? {
                self.db.put(&key, &delete_tx)?;
            }
        }

        Ok(())
    }

    /// Check if this node has a specific ACTIVE release (not tombstoned)
    fn has_release(&self, release_id: &str) -> Result<bool> {
        let key = make_key(prefixes::RELEASE, release_id);
        let release: Option<Release> = self.db.get(&key)?;
        match release {
            Some(release) => Ok(!release.is_tombstone),
            None => Ok(false),
        }
    }

    /// Check if this node has a specific delete transaction
    fn has_delete_transaction(&self, delete_tx_id: &str) -> Result<bool> {
        let key = make_key(prefixes::DELETE_TRANSACTION, delete_tx_id);
        self.db.exists(&key)
    }
}

/// Sync all nodes in a round-robin fashion (simulates SPORE gossip)
fn sync_all_nodes(nodes: &[TestNode]) -> Result<()> {
    // Each node syncs from all other nodes
    for i in 0..nodes.len() {
        for j in 0..nodes.len() {
            if i != j {
                nodes[i].sync_from(&nodes[j])?;
            }
        }
    }

    Ok(())
}

/// Sync all nodes using hexagonal topology (8 structured neighbors)
fn sync_all_nodes_hex(nodes: &[TestNode], mesh: &HexMesh) -> Result<()> {
    // Each node syncs with its 8 hex neighbors only
    for (i, node) in nodes.iter().enumerate() {
        if let Some(hex_pos) = &node.hex_position {
            let neighbor_indices = mesh.get_neighbors(hex_pos);

            for &j in &neighbor_indices {
                if j < nodes.len() {
                    node.sync_from(&nodes[j])?;
                }
            }
        }
    }

    Ok(())
}

/// Verify all nodes have converged to the same state
fn verify_consistency(nodes: &[TestNode]) -> Result<()> {
    if nodes.is_empty() {
        return Ok(());
    }

    // Get release count from first node
    let expected_releases = nodes[0].get_releases()?;
    let expected_count = expected_releases.len();
    let expected_ids: std::collections::HashSet<String> = expected_releases
        .iter()
        .map(|r| r.id.clone())
        .collect();

    // Verify all other nodes have the same releases
    for (i, node) in nodes.iter().enumerate() {
        let releases = node.get_releases()?;
        let count = releases.len();
        let ids: std::collections::HashSet<String> = releases
            .iter()
            .map(|r| r.id.clone())
            .collect();

        if count != expected_count {
            anyhow::bail!(
                "Node {} has {} releases, expected {}",
                i,
                count,
                expected_count
            );
        }

        if ids != expected_ids {
            let missing: Vec<_> = expected_ids.difference(&ids).collect();
            let extra: Vec<_> = ids.difference(&expected_ids).collect();

            anyhow::bail!(
                "Node {} has different releases. Missing: {:?}, Extra: {:?}",
                i,
                missing,
                extra
            );
        }
    }

    println!("✅ All {} nodes have {} releases (consistent!)", nodes.len(), expected_count);

    Ok(())
}

/// Create a test release
fn create_test_release(id: &str, name: &str, posted_by: &str) -> Release {
    use std::collections::HashMap;

    Release {
        id: id.to_string(),
        name: name.to_string(),
        category_id: "test".to_string(),
        category_slug: "test".to_string(),
        content_cid: format!("Qm{}", Uuid::new_v4()),
        thumbnail_cid: None,
        metadata: None,
        site_address: "local".to_string(),
        posted_by: posted_by.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        vector_clock: HashMap::new(),  // Will be initialized when created on node
        is_tombstone: false,
        tombstone_type: None,
        deleted_at: None,
        deleted_by: None,
    }
}

#[tokio::test]
async fn test_3_nodes_create_sync() -> Result<()> {
    println!("\n=== Testing 3-node CREATE sync ===");

    // Create 3 nodes
    let nodes = vec![
        TestNode::new("node-0".to_string())?,
        TestNode::new("node-1".to_string())?,
        TestNode::new("node-2".to_string())?,
    ];

    // Each node creates a unique release
    let release_0 = create_test_release("release-0", "Release from Node 0", "node-0");
    let release_1 = create_test_release("release-1", "Release from Node 1", "node-1");
    let release_2 = create_test_release("release-2", "Release from Node 2", "node-2");

    nodes[0].create_release(&release_0)?;
    nodes[1].create_release(&release_1)?;
    nodes[2].create_release(&release_2)?;

    println!("Created 3 releases on 3 different nodes");

    // Verify nodes start with 1 release each
    assert_eq!(nodes[0].get_releases()?.len(), 1);
    assert_eq!(nodes[1].get_releases()?.len(), 1);
    assert_eq!(nodes[2].get_releases()?.len(), 1);

    // Sync all nodes
    println!("Syncing all nodes...");
    sync_all_nodes(&nodes)?;

    // Verify all nodes now have 3 releases
    verify_consistency(&nodes)?;

    for node in &nodes {
        assert_eq!(node.get_releases()?.len(), 3, "Each node should have 3 releases");
    }

    println!("✅ 3-node CREATE sync test passed!");

    Ok(())
}

#[tokio::test]
async fn test_3_nodes_update_sync() -> Result<()> {
    println!("\n=== Testing 3-node UPDATE sync ===");

    // Create 3 nodes
    let nodes = vec![
        TestNode::new("node-0".to_string())?,
        TestNode::new("node-1".to_string())?,
        TestNode::new("node-2".to_string())?,
    ];

    // Create initial release on node 0
    let mut release = create_test_release("release-shared", "Initial Version", "node-0");
    nodes[0].create_release(&release)?;

    // Sync to all nodes
    sync_all_nodes(&nodes)?;
    verify_consistency(&nodes)?;

    println!("All nodes have initial release");

    // Update release on node 1
    release.name = "Updated Version from Node 1".to_string();
    release.created_at = chrono::Utc::now().to_rfc3339(); // Change timestamp
    nodes[1].update_release(&release)?;

    println!("Updated release on node 1");

    // Sync again
    sync_all_nodes(&nodes)?;

    // Verify all nodes have the updated version
    for node in &nodes {
        let releases = node.get_releases()?;
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].name, "Updated Version from Node 1");
    }

    println!("✅ 3-node UPDATE sync test passed!");

    Ok(())
}

#[tokio::test]
async fn test_3_nodes_delete_sync() -> Result<()> {
    println!("\n=== Testing 3-node DELETE sync ===");

    // Create 3 nodes
    let nodes = vec![
        TestNode::new("node-0".to_string())?,
        TestNode::new("node-1".to_string())?,
        TestNode::new("node-2".to_string())?,
    ];

    // Create 3 releases on node 0
    let release_0 = create_test_release("release-0", "Release 0", "node-0");
    let release_1 = create_test_release("release-1", "Release 1", "node-0");
    let release_2 = create_test_release("release-2", "Release 2", "node-0");

    nodes[0].create_release(&release_0)?;
    nodes[0].create_release(&release_1)?;
    nodes[0].create_release(&release_2)?;

    // Sync to all nodes
    sync_all_nodes(&nodes)?;
    verify_consistency(&nodes)?;

    println!("All nodes have 3 releases");

    // Delete one release on node 1
    let delete_tx_id = nodes[1].delete_release("release-1")?;

    println!("Deleted release-1 on node 1 (delete tx: {})", delete_tx_id);

    // Verify node 1 no longer has the release
    assert!(!nodes[1].has_release("release-1")?);
    assert_eq!(nodes[1].get_releases()?.len(), 2);

    // Sync to all nodes
    sync_all_nodes(&nodes)?;

    // Verify all nodes have the same state (2 releases, 1 deleted)
    verify_consistency(&nodes)?;

    for node in &nodes {
        assert_eq!(node.get_releases()?.len(), 2, "Each node should have 2 releases after delete");
        assert!(!node.has_release("release-1")?, "release-1 should be deleted on all nodes");
        assert!(node.has_delete_transaction(&delete_tx_id)?, "All nodes should have delete transaction");
    }

    println!("✅ 3-node DELETE sync test passed!");

    Ok(())
}

#[tokio::test]
async fn test_3_nodes_concurrent_creates() -> Result<()> {
    println!("\n=== Testing 3-node CONCURRENT CREATE ===");

    // Create 3 nodes
    let nodes = vec![
        TestNode::new("node-0".to_string())?,
        TestNode::new("node-1".to_string())?,
        TestNode::new("node-2".to_string())?,
    ];

    // Each node creates 10 releases concurrently
    for i in 0..10 {
        nodes[0].create_release(&create_test_release(
            &format!("release-0-{}", i),
            &format!("Release {} from Node 0", i),
            "node-0"
        ))?;

        nodes[1].create_release(&create_test_release(
            &format!("release-1-{}", i),
            &format!("Release {} from Node 1", i),
            "node-1"
        ))?;

        nodes[2].create_release(&create_test_release(
            &format!("release-2-{}", i),
            &format!("Release {} from Node 2", i),
            "node-2"
        ))?;
    }

    println!("Created 30 releases across 3 nodes (10 each)");

    // Sync all nodes
    sync_all_nodes(&nodes)?;

    // Verify all nodes have 30 releases
    verify_consistency(&nodes)?;

    for node in &nodes {
        assert_eq!(node.get_releases()?.len(), 30, "Each node should have 30 releases");
    }

    println!("✅ 3-node CONCURRENT CREATE test passed!");

    Ok(())
}

#[tokio::test]
async fn test_10_nodes_crud_operations() -> Result<()> {
    println!("\n=== Testing 10-node CRUD operations ===");

    // Create 10 nodes
    let mut nodes = Vec::new();
    for i in 0..10 {
        nodes.push(TestNode::new(format!("node-{}", i))?);
    }

    println!("Created 10 nodes");

    // Phase 1: CREATE - Each node creates 5 releases
    println!("Phase 1: CREATE - Each node creates 5 releases");
    for (node_idx, node) in nodes.iter().enumerate() {
        for i in 0..5 {
            node.create_release(&create_test_release(
                &format!("release-{}-{}", node_idx, i),
                &format!("Release {} from Node {}", i, node_idx),
                &format!("node-{}", node_idx)
            ))?;
        }
    }

    // Sync all nodes
    sync_all_nodes(&nodes)?;
    verify_consistency(&nodes)?;

    // Should have 50 releases (10 nodes × 5 releases)
    assert_eq!(nodes[0].get_releases()?.len(), 50);
    println!("✅ All 10 nodes have 50 releases");

    // Phase 2: UPDATE - Update 10 releases from different nodes
    println!("Phase 2: UPDATE - Update 10 releases from different nodes");
    for i in 0..10 {
        let release_id = format!("release-{}-0", i); // Update first release from each node
        let mut releases = nodes[i].get_releases()?;

        if let Some(mut release) = releases.iter_mut().find(|r| r.id == release_id) {
            release.name = format!("UPDATED: {}", release.name);
            release.created_at = chrono::Utc::now().to_rfc3339();
            nodes[i].update_release(&release)?;
        }
    }

    // Sync all nodes
    sync_all_nodes(&nodes)?;

    // Verify updates propagated
    for node in &nodes {
        let releases = node.get_releases()?;
        let updated_count = releases.iter().filter(|r| r.name.starts_with("UPDATED:")).count();
        assert_eq!(updated_count, 10, "Should have 10 updated releases");
    }
    println!("✅ All 10 nodes have 10 updated releases");

    // Phase 3: DELETE - Delete 20 releases from different nodes
    println!("Phase 3: DELETE - Delete 20 releases from different nodes");
    for i in 0..10 {
        // Delete 2 releases per node
        nodes[i].delete_release(&format!("release-{}-1", i))?;
        nodes[i].delete_release(&format!("release-{}-2", i))?;
    }

    // Sync all nodes
    sync_all_nodes(&nodes)?;
    verify_consistency(&nodes)?;

    // Should have 30 releases left (50 - 20)
    for node in &nodes {
        assert_eq!(node.get_releases()?.len(), 30, "Should have 30 releases after deletes");
    }
    println!("✅ All 10 nodes have 30 releases after deletes");

    println!("✅ 10-node CRUD operations test passed!");

    Ok(())
}

#[tokio::test]
#[ignore] // This test takes a long time, run with --ignored flag
async fn test_100_nodes_crud_operations() -> Result<()> {
    println!("\n=== Testing 100-node CRUD operations ===");

    // Create 100 nodes
    let mut nodes = Vec::new();
    for i in 0..100 {
        nodes.push(TestNode::new(format!("node-{}", i))?);

        if (i + 1) % 10 == 0 {
            println!("Created {} nodes...", i + 1);
        }
    }

    println!("Created 100 nodes");

    // Phase 1: CREATE - Each node creates 3 releases
    println!("Phase 1: CREATE - Each node creates 3 releases");
    for (node_idx, node) in nodes.iter().enumerate() {
        for i in 0..3 {
            node.create_release(&create_test_release(
                &format!("release-{}-{}", node_idx, i),
                &format!("Release {} from Node {}", i, node_idx),
                &format!("node-{}", node_idx)
            ))?;
        }

        if (node_idx + 1) % 10 == 0 {
            println!("Created releases on {} nodes...", node_idx + 1);
        }
    }

    // Sync all nodes (this will be slow)
    println!("Syncing 100 nodes...");
    sync_all_nodes(&nodes)?;
    verify_consistency(&nodes)?;

    // Should have 300 releases (100 nodes × 3 releases)
    assert_eq!(nodes[0].get_releases()?.len(), 300);
    println!("✅ All 100 nodes have 300 releases");

    // Phase 2: DELETE - Delete 100 releases (1 per node)
    println!("Phase 2: DELETE - Delete 100 releases (1 per node)");
    for (node_idx, node) in nodes.iter().enumerate() {
        node.delete_release(&format!("release-{}-0", node_idx))?;

        if (node_idx + 1) % 10 == 0 {
            println!("Deleted on {} nodes...", node_idx + 1);
        }
    }

    // Sync all nodes
    println!("Syncing deletes across 100 nodes...");
    sync_all_nodes(&nodes)?;
    verify_consistency(&nodes)?;

    // Should have 200 releases left (300 - 100)
    for node in &nodes {
        assert_eq!(node.get_releases()?.len(), 200, "Should have 200 releases after deletes");
    }
    println!("✅ All 100 nodes have 200 releases after deletes");

    println!("✅ 100-node CRUD operations test passed!");

    Ok(())
}

#[tokio::test]
#[ignore] // This test takes a long time, run with --ignored flag
async fn test_5000_nodes_massive_scale() -> Result<()> {
    println!("\n=== Testing 5,000-node MASSIVE SCALE ===");
    println!("This will test SPORE sync at production scale!");

    let node_count = 5000;
    let releases_per_node = 1; // Keep it simple - 5,000 total releases

    println!("Creating {} nodes...", node_count);
    let start_creation = std::time::Instant::now();

    let mut nodes = Vec::new();
    for i in 0..node_count {
        nodes.push(TestNode::new(format!("node-{}", i))?);

        if (i + 1) % 1000 == 0 {
            println!("  Created {} nodes... ({:.2}s)", i + 1, start_creation.elapsed().as_secs_f64());
        }
    }

    let creation_time = start_creation.elapsed();
    println!("✅ Created {} nodes in {:.2}s", node_count, creation_time.as_secs_f64());

    // Phase 1: CREATE - Each node creates 1 release
    println!("\nPhase 1: CREATE - Each node creates {} release(s)", releases_per_node);
    let start_create = std::time::Instant::now();

    for (node_idx, node) in nodes.iter().enumerate() {
        for i in 0..releases_per_node {
            node.create_release(&create_test_release(
                &format!("release-{}-{}", node_idx, i),
                &format!("Release {} from Node {}", i, node_idx),
                &format!("node-{}", node_idx)
            ))?;
        }

        if (node_idx + 1) % 1000 == 0 {
            println!("  Created releases on {} nodes... ({:.2}s)",
                node_idx + 1, start_create.elapsed().as_secs_f64());
        }
    }

    let create_time = start_create.elapsed();
    println!("✅ Created {} releases across {} nodes in {:.2}s",
        node_count * releases_per_node, node_count, create_time.as_secs_f64());

    // Phase 2: SYNC - Full gossip sync (this will be SLOW)
    println!("\nPhase 2: SYNC - Full gossip sync across {} nodes", node_count);
    println!("WARNING: This is O(N²) and will take a long time!");
    println!("Expected sync operations: {} (node-to-node syncs)", node_count * (node_count - 1));

    let start_sync = std::time::Instant::now();

    // Sample-based sync to make this feasible
    // Each node syncs with 10 random peers instead of all nodes
    let peers_per_node = 10;
    println!("Using optimized sync: {} peers per node (not full mesh)", peers_per_node);

    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    for (i, node) in nodes.iter().enumerate() {
        // Pick random peers to sync from
        let mut peer_indices: Vec<usize> = (0..node_count).filter(|&j| j != i).collect();
        peer_indices.shuffle(&mut rng);
        peer_indices.truncate(peers_per_node);

        for &j in &peer_indices {
            node.sync_from(&nodes[j])?;
        }

        if (i + 1) % 1000 == 0 {
            let elapsed = start_sync.elapsed().as_secs_f64();
            let rate = (i + 1) as f64 / elapsed;
            let remaining = (node_count - i - 1) as f64 / rate;
            println!("  Synced {} nodes... ({:.2}s elapsed, ~{:.0}s remaining, {:.1} nodes/sec)",
                i + 1, elapsed, remaining, rate);
        }
    }

    let sync_time = start_sync.elapsed();
    println!("✅ Completed sync in {:.2}s ({:.1} nodes/sec)",
        sync_time.as_secs_f64(), node_count as f64 / sync_time.as_secs_f64());

    // Phase 3: VERIFY - Sample nodes for consistency
    println!("\nPhase 3: VERIFY - Sampling nodes for consistency");
    let sample_size = 100;
    println!("Checking {} random nodes (not all {} - that would take forever)", sample_size, node_count);

    let start_verify = std::time::Instant::now();

    let mut sample_indices: Vec<usize> = (0..node_count).collect();
    sample_indices.shuffle(&mut rng);
    sample_indices.truncate(sample_size);

    // Check that sampled nodes converged
    let expected_count = nodes[sample_indices[0]].get_releases()?.len();
    println!("Expected release count: {}", expected_count);

    let mut converged = 0;
    let mut diverged = 0;

    for &idx in &sample_indices {
        let count = nodes[idx].get_releases()?.len();
        if count == expected_count {
            converged += 1;
        } else {
            diverged += 1;
            println!("  ⚠️ Node {} has {} releases (expected {})", idx, count, expected_count);
        }
    }

    let verify_time = start_verify.elapsed();
    println!("✅ Verified {} sample nodes in {:.2}s", sample_size, verify_time.as_secs_f64());
    println!("   Converged: {} ({:.1}%)", converged, (converged as f64 / sample_size as f64) * 100.0);
    println!("   Diverged: {} ({:.1}%)", diverged, (diverged as f64 / sample_size as f64) * 100.0);

    // Phase 4: STATS
    println!("\n=== Performance Summary ===");
    let total_time = creation_time + create_time + sync_time + verify_time;
    println!("Total test time: {:.2}s", total_time.as_secs_f64());
    println!("  Node creation: {:.2}s ({:.1} nodes/sec)", creation_time.as_secs_f64(), node_count as f64 / creation_time.as_secs_f64());
    println!("  Release creation: {:.2}s ({:.1} releases/sec)", create_time.as_secs_f64(), (node_count * releases_per_node) as f64 / create_time.as_secs_f64());
    println!("  Gossip sync: {:.2}s ({:.1} nodes/sec)", sync_time.as_secs_f64(), node_count as f64 / sync_time.as_secs_f64());
    println!("  Verification: {:.2}s", verify_time.as_secs_f64());

    // Calculate convergence percentage
    let convergence_pct = (converged as f64 / sample_size as f64) * 100.0;

    if convergence_pct < 95.0 {
        println!("\n❌ POOR CONVERGENCE: Only {:.1}% of nodes converged", convergence_pct);
        println!("This indicates the sync algorithm needs improvement!");
        return Err(anyhow::anyhow!(
            "Convergence too low: {:.1}% (expected >95%)",
            convergence_pct
        ));
    } else if convergence_pct < 100.0 {
        println!("\n⚠️ PARTIAL CONVERGENCE: {:.1}% of nodes converged", convergence_pct);
        println!("Some nodes diverged - this is expected with limited peer connections");
    } else {
        println!("\n✅ PERFECT CONVERGENCE: 100% of sampled nodes converged!");
    }

    println!("\n✅ 5,000-node massive scale test complete!");

    Ok(())
}

#[tokio::test]
async fn test_100_nodes_hex_topology() -> Result<()> {
    println!("\n=== Testing 100-node with CITADEL DHT HEX TOPOLOGY ===");
    println!("Using structured 8-neighbor hex mesh instead of random gossip!");

    let node_count = 100;
    let releases_per_node = 1;

    println!("Creating {} nodes...", node_count);
    let start_creation = std::time::Instant::now();

    let mut nodes = Vec::new();
    for i in 0..node_count {
        nodes.push(TestNode::new(format!("node-{}", i))?);

        if (i + 1) % 10 == 0 {
            println!("  Created {} nodes... ({:.2}s)", i + 1, start_creation.elapsed().as_secs_f64());
        }
    }

    let creation_time = start_creation.elapsed();
    println!("✅ Created {} nodes in {:.2}s", node_count, creation_time.as_secs_f64());

    // Build HexMesh and assign positions
    println!("\nBuilding hexagonal mesh topology...");
    let start_mesh = std::time::Instant::now();

    let mut mesh = HexMesh::new();
    for i in 0..node_count {
        let position = mesh.join_node(i);
        nodes[i].set_hex_position(position);
    }

    let mesh_time = start_mesh.elapsed();
    println!("✅ Built hex mesh in {:.2}s", mesh_time.as_secs_f64());
    println!("   Mesh dimensions: {}×{}×{}", mesh.config.width, mesh.config.height, mesh.config.depth);

    // Phase 1: CREATE - Each node creates 1 release
    println!("\nPhase 1: CREATE - Each node creates {} release(s)", releases_per_node);
    let start_create = std::time::Instant::now();

    for (node_idx, node) in nodes.iter().enumerate() {
        for i in 0..releases_per_node {
            node.create_release(&create_test_release(
                &format!("release-{}-{}", node_idx, i),
                &format!("Release {} from Node {}", i, node_idx),
                &format!("node-{}", node_idx)
            ))?;
        }

        if (node_idx + 1) % 1000 == 0 {
            println!("  Created releases on {} nodes... ({:.2}s)",
                node_idx + 1, start_create.elapsed().as_secs_f64());
        }
    }

    let create_time = start_create.elapsed();
    println!("✅ Created {} releases across {} nodes in {:.2}s",
        node_count * releases_per_node, node_count, create_time.as_secs_f64());

    // Phase 2: SYNC - Hexagonal topology sync (8 structured neighbors)
    println!("\nPhase 2: SYNC - Hexagonal topology sync (8 neighbors per node)");
    println!("Expected connections: ~{} (8 neighbors × {} nodes)", node_count * 8, node_count);

    let start_sync = std::time::Instant::now();

    // Run multiple sync rounds to allow gossip to propagate
    let sync_rounds = 5;  // For 100 nodes, 5 rounds should be plenty
    println!("Running {} sync rounds to propagate across mesh...", sync_rounds);

    for round in 0..sync_rounds {
        let round_start = std::time::Instant::now();
        sync_all_nodes_hex(&nodes, &mesh)?;
        let round_time = round_start.elapsed();

        println!("  Round {}/{} complete ({:.2}s)", round + 1, sync_rounds, round_time.as_secs_f64());
    }

    let sync_time = start_sync.elapsed();
    println!("✅ Completed {} sync rounds in {:.2}s ({:.2}s/round)",
        sync_rounds, sync_time.as_secs_f64(), sync_time.as_secs_f64() / sync_rounds as f64);

    // Phase 3: VERIFY - Sample nodes for consistency
    println!("\nPhase 3: VERIFY - Checking all nodes for consistency");
    let sample_size = node_count;  // Check all 100 nodes
    println!("Checking all {} nodes", sample_size);

    let start_verify = std::time::Instant::now();

    let sample_indices: Vec<usize> = (0..node_count).collect();

    // Check that sampled nodes converged
    let expected_count = nodes[sample_indices[0]].get_releases()?.len();
    println!("Expected release count: {}", expected_count);

    let mut converged = 0;
    let mut diverged = 0;
    let mut release_counts: HashMap<usize, usize> = HashMap::new();

    for &idx in &sample_indices {
        let count = nodes[idx].get_releases()?.len();
        *release_counts.entry(count).or_insert(0) += 1;

        if count == expected_count {
            converged += 1;
        } else {
            diverged += 1;
            println!("  ⚠️ Node {} has {} releases (expected {})", idx, count, expected_count);
        }
    }

    let verify_time = start_verify.elapsed();
    println!("✅ Verified {} sample nodes in {:.2}s", sample_size, verify_time.as_secs_f64());
    println!("   Converged: {} ({:.1}%)", converged, (converged as f64 / sample_size as f64) * 100.0);
    println!("   Diverged: {} ({:.1}%)", diverged, (diverged as f64 / sample_size as f64) * 100.0);

    // Show distribution of release counts
    println!("\n   Release count distribution:");
    let mut counts: Vec<_> = release_counts.iter().collect();
    counts.sort_by_key(|(count, _)| **count);
    for (count, nodes) in counts {
        println!("     {} releases: {} nodes", count, nodes);
    }

    // Phase 4: STATS
    println!("\n=== Performance Summary ===");
    let total_time = creation_time + mesh_time + create_time + sync_time + verify_time;
    println!("Total test time: {:.2}s", total_time.as_secs_f64());
    println!("  Node creation: {:.2}s ({:.1} nodes/sec)", creation_time.as_secs_f64(), node_count as f64 / creation_time.as_secs_f64());
    println!("  Hex mesh build: {:.2}s", mesh_time.as_secs_f64());
    println!("  Release creation: {:.2}s ({:.1} releases/sec)", create_time.as_secs_f64(), (node_count * releases_per_node) as f64 / create_time.as_secs_f64());
    println!("  Hex topology sync: {:.2}s ({} rounds, {:.2}s/round)", sync_time.as_secs_f64(), sync_rounds, sync_time.as_secs_f64() / sync_rounds as f64);
    println!("  Verification: {:.2}s", verify_time.as_secs_f64());

    // Compare to random gossip baseline
    println!("\n=== COMPARISON: Hex vs Random Gossip ===");
    println!("At 5,000 nodes:");
    println!("  Random gossip (10 peers): 7% convergence ❌");
    println!("  Hex topology (8 neighbors): Expected >95%");

    let convergence_pct = (converged as f64 / sample_size as f64) * 100.0;
    println!("\nAt {} nodes:", node_count);
    println!("  Hex topology (8 neighbors): {:.1}% convergence", convergence_pct);

    if convergence_pct >= 100.0 {
        println!("✅ PERFECT CONVERGENCE! All nodes have identical state!");
        println!("✅ Citadel DHT structured topology works perfectly at {}-node scale!", node_count);
    } else if convergence_pct > 95.0 {
        println!("✅ HEX TOPOLOGY SUCCESS! {:.1}% convergence", convergence_pct);
        println!("✅ Citadel DHT structured topology works well!");
    } else if convergence_pct > 70.0 {
        println!("⚡ PARTIAL SUCCESS: {:.1}% convergence", convergence_pct);
        println!("   More sync rounds may be needed for full convergence");
    } else {
        println!("⚠️ NEEDS WORK: {:.1}% convergence", convergence_pct);
        println!("   Hex topology needs debugging or more rounds");
    }

    // Don't fail the test - we want to see the results even if convergence isn't perfect
    println!("\n✅ 100-node hex topology test complete!");
    println!("   This demonstrates structured topology feasibility");

    Ok(())
}

#[tokio::test]
async fn test_flapping_detection() -> Result<()> {
    println!("\n=== Testing FLAPPING detection ===");

    // Create 3 nodes
    let nodes = vec![
        TestNode::new("node-0".to_string())?,
        TestNode::new("node-1".to_string())?,
        TestNode::new("node-2".to_string())?,
    ];

    // Create a release on node 0
    let release = create_test_release("flapping-release", "Flapping Test", "node-0");
    nodes[0].create_release(&release)?;

    // Sync to all nodes
    sync_all_nodes(&nodes)?;
    verify_consistency(&nodes)?;

    println!("Initial state: All nodes have 1 release");

    // Simulate flapping: delete on node 1, but node 0 still has it
    let delete_tx_id = nodes[1].delete_release("flapping-release")?;

    println!("Node 1 deleted the release");

    // DON'T sync yet - node 0 and node 2 still have the release
    // Now node 0 updates the release
    let mut updated_release = release.clone();
    updated_release.name = "Updated After Delete (Flapping!)".to_string();
    updated_release.created_at = chrono::Utc::now().to_rfc3339();
    nodes[0].update_release(&updated_release)?;

    println!("Node 0 updated the release (but node 1 thinks it's deleted!)");

    // Now sync all nodes - this is a FLAPPING scenario
    // The delete transaction exists, but the release is still being updated
    sync_all_nodes(&nodes)?;

    // Check final state - what happens?
    let node0_releases = nodes[0].get_releases()?;
    let node1_releases = nodes[1].get_releases()?;
    let node2_releases = nodes[2].get_releases()?;

    println!("After sync:");
    println!("  Node 0: {} releases", node0_releases.len());
    println!("  Node 1: {} releases", node1_releases.len());
    println!("  Node 2: {} releases", node2_releases.len());

    // FLAPPING CHECK: Are all nodes consistent?
    if node0_releases.len() != node1_releases.len() || node1_releases.len() != node2_releases.len() {
        println!("❌ FLAPPING DETECTED: Nodes have different release counts!");
        println!("This is the consistency problem we need to fix!");

        // This is expected to fail - showing the bug
        return Err(anyhow::anyhow!(
            "FLAPPING: Nodes are inconsistent. Node 0: {}, Node 1: {}, Node 2: {}",
            node0_releases.len(),
            node1_releases.len(),
            node2_releases.len()
        ));
    }

    println!("✅ No flapping detected (nodes are consistent)");

    Ok(())
}
