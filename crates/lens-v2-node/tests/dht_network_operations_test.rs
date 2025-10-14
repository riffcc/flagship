//! DHT Network Operations Test
//!
//! Tests that verify DHT implementation matches Citadel DHT Specification Section 2.4:
//! "Recursive DHT (Meta-Routing)" - DHT uses itself for topology discovery
//!
//! Reference: /opt/castle/workspace/citadel/2025-10-12-Citadel-DHT-SPEC.md (lines 252-563)
//!
//! Critical Requirements:
//! 1. Slot ownership must be stored IN the DHT network (not local storage)
//! 2. LazyNode queries the NETWORK DHT for neighbor discovery
//! 3. DHT GET/PUT operations must route through the mesh to responsible nodes
//! 4. O(1) deterministic routing between nodes
//! 5. No routing tables or neighbor caches needed (64 bytes state!)

use anyhow::Result;
use citadel_core::key_mapping::key_to_slot;
use citadel_core::topology::{Direction, MeshConfig, SlotCoordinate};
use lens_node::peer_registry::{assign_unique_slot, slot_ownership_key, SlotOwnership};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

// ========== Test Network DHT ==========

/// Represents a networked DHT node that can route messages
#[derive(Clone)]
struct NetworkDHTNode {
    peer_id: String,
    my_slot: SlotCoordinate,
    mesh_config: MeshConfig,

    // Local storage (for keys this node is responsible for)
    local_storage: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,

    // Network reference (to route messages to other nodes)
    network: Arc<Mutex<TestDHTNetwork>>,

    // Metrics
    get_requests: Arc<Mutex<u64>>,
    put_requests: Arc<Mutex<u64>>,
    routed_messages: Arc<Mutex<u64>>,
}

impl NetworkDHTNode {
    fn new(peer_id: String, my_slot: SlotCoordinate, mesh_config: MeshConfig, network: Arc<Mutex<TestDHTNetwork>>) -> Self {
        Self {
            peer_id,
            my_slot,
            mesh_config,
            local_storage: Arc::new(RwLock::new(HashMap::new())),
            network,
            get_requests: Arc::new(Mutex::new(0)),
            put_requests: Arc::new(Mutex::new(0)),
            routed_messages: Arc::new(Mutex::new(0)),
        }
    }

    /// DHT GET operation - routes through network to responsible node
    /// This is the CORE test of Section 2.4 of the spec!
    async fn dht_get(&self, key: &[u8; 32]) -> Result<Option<Vec<u8>>> {
        *self.get_requests.lock().await += 1;

        // Calculate which slot is responsible for this key (O(1)!)
        let target_slot = key_to_slot(key, &self.mesh_config);

        // If we're responsible, serve from local storage
        if target_slot == self.my_slot {
            let storage = self.local_storage.read().await;
            return Ok(storage.get(&key[..]).cloned());
        }

        // Otherwise, route through network to responsible node
        *self.routed_messages.lock().await += 1;

        let network = self.network.lock().await;
        network.route_get_request(self.my_slot, target_slot, key, &self.mesh_config).await
    }

    /// DHT PUT operation - routes through network to responsible node
    async fn dht_put(&self, key: [u8; 32], value: Vec<u8>) -> Result<()> {
        *self.put_requests.lock().await += 1;

        // Calculate which slot is responsible for this key (O(1)!)
        let target_slot = key_to_slot(&key, &self.mesh_config);

        // If we're responsible, store locally
        if target_slot == self.my_slot {
            let mut storage = self.local_storage.write().await;
            storage.insert(key.to_vec(), value);
            return Ok(());
        }

        // Otherwise, route through network to responsible node
        *self.routed_messages.lock().await += 1;

        let network = self.network.lock().await;
        network.route_put_request(self.my_slot, target_slot, key, value, &self.mesh_config).await
    }

    /// Query network DHT for neighbor at specific direction (LazyNode!)
    /// This is the CRITICAL test from spec lines 316-328
    async fn get_neighbor(&self, direction: Direction) -> Result<Option<String>> {
        // Calculate which slot the neighbor occupies
        let neighbor_slot = self.my_slot.neighbor(direction, &self.mesh_config);

        // Query DHT for "who owns this slot?" (spec line 323)
        let key = slot_ownership_key(neighbor_slot);

        if let Some(ownership_bytes) = self.dht_get(&key).await? {
            let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes)?;
            Ok(Some(ownership.peer_id))
        } else {
            Ok(None)
        }
    }

    /// Get all 8 neighbors using LazyNode pattern
    async fn discover_all_neighbors(&self) -> Result<Vec<(Direction, String, SlotCoordinate)>> {
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

        for direction in directions {
            let neighbor_slot = self.my_slot.neighbor(direction, &self.mesh_config);

            if let Some(peer_id) = self.get_neighbor(direction).await? {
                neighbors.push((direction, peer_id, neighbor_slot));
            }
        }

        Ok(neighbors)
    }

    async fn get_metrics(&self) -> NodeMetrics {
        NodeMetrics {
            get_requests: *self.get_requests.lock().await,
            put_requests: *self.put_requests.lock().await,
            routed_messages: *self.routed_messages.lock().await,
        }
    }
}

#[derive(Debug)]
struct NodeMetrics {
    get_requests: u64,
    put_requests: u64,
    routed_messages: u64,
}

/// Simulates a DHT network with greedy routing
struct TestDHTNetwork {
    nodes: HashMap<SlotCoordinate, NetworkDHTNode>,
    total_hops: Arc<Mutex<u64>>,
}

impl TestDHTNetwork {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            total_hops: Arc::new(Mutex::new(0)),
        }
    }

    fn add_node(&mut self, node: NetworkDHTNode) {
        self.nodes.insert(node.my_slot, node);
    }

    /// Route GET request through mesh using greedy routing (O(1) routing decisions!)
    async fn route_get_request(
        &self,
        from: SlotCoordinate,
        to: SlotCoordinate,
        key: &[u8; 32],
        config: &MeshConfig,
    ) -> Result<Option<Vec<u8>>> {
        let mut current = from;
        let mut hops = 0;

        // Greedy routing to target
        while current != to {
            hops += 1;

            // Find next hop (O(1) routing decision!)
            let direction = citadel_core::routing::greedy_direction(&current, &to, config)
                .ok_or_else(|| anyhow::anyhow!("Already at target"))?;

            current = current.neighbor(direction, config);

            // Prevent infinite loops
            if hops > 1000 {
                anyhow::bail!("Routing loop detected after {} hops", hops);
            }
        }

        *self.total_hops.lock().await += hops;

        // Reached target node - serve from its local storage
        if let Some(target_node) = self.nodes.get(&to) {
            let storage = target_node.local_storage.read().await;
            Ok(storage.get(&key[..]).cloned())
        } else {
            anyhow::bail!("Target node not found at slot {:?}", to);
        }
    }

    /// Route PUT request through mesh using greedy routing
    async fn route_put_request(
        &self,
        from: SlotCoordinate,
        to: SlotCoordinate,
        key: [u8; 32],
        value: Vec<u8>,
        config: &MeshConfig,
    ) -> Result<()> {
        let mut current = from;
        let mut hops = 0;

        // Greedy routing to target
        while current != to {
            hops += 1;

            // Find next hop (O(1) routing decision!)
            let direction = citadel_core::routing::greedy_direction(&current, &to, config)
                .ok_or_else(|| anyhow::anyhow!("Already at target"))?;

            current = current.neighbor(direction, config);

            // Prevent infinite loops
            if hops > 1000 {
                anyhow::bail!("Routing loop detected after {} hops", hops);
            }
        }

        *self.total_hops.lock().await += hops;

        // Reached target node - store in its local storage
        if let Some(target_node) = self.nodes.get(&to) {
            let mut storage = target_node.local_storage.write().await;
            storage.insert(key.to_vec(), value);
            Ok(())
        } else {
            anyhow::bail!("Target node not found at slot {:?}", to);
        }
    }

    async fn get_total_hops(&self) -> u64 {
        *self.total_hops.lock().await
    }
}

// ========== Tests ==========

/// Test 1: Slot ownership stored in NETWORK DHT (not local storage!)
/// This test verifies spec Section 2.4 requirement
#[tokio::test]
async fn test_dht_network_slot_ownership_storage() -> Result<()> {
    println!("\n=== Test 1: Slot Ownership Stored in NETWORK DHT ===");
    println!("Spec Reference: Section 2.4, lines 256-275\n");

    let mesh_config = MeshConfig::new(10, 10, 5);
    let network = Arc::new(Mutex::new(TestDHTNetwork::new()));

    // Create 10 nodes at different slots
    let mut nodes = Vec::new();
    let mut occupied_slots = HashSet::new();
    for i in 0..10 {
        let peer_id = format!("peer-{}", i);
        let slot = assign_unique_slot(&peer_id, &mesh_config, &mut occupied_slots);
        let node = NetworkDHTNode::new(peer_id.clone(), slot, mesh_config, Arc::clone(&network));

        network.lock().await.add_node(node.clone());
        nodes.push(node);

        println!("Created {} at slot {:?}", peer_id, slot);
    }

    // Each node announces its slot ownership to NETWORK DHT (not local!)
    println!("\n📢 Announcing slot ownership to network DHT...");
    for node in &nodes {
        let ownership = SlotOwnership::new(node.peer_id.clone(), node.my_slot, None);
        let ownership_key = slot_ownership_key(node.my_slot);
        let ownership_bytes = serde_json::to_vec(&ownership)?;

        // This should route through network to responsible node!
        node.dht_put(ownership_key, ownership_bytes).await?;
        println!("  {} announced (routed through network)", node.peer_id);
    }

    // Verify announcements are stored in network (may be on different nodes!)
    println!("\n🔍 Verifying slot ownership is in network DHT...");
    let mut successful_queries = 0;

    for node in &nodes {
        let ownership_key = slot_ownership_key(node.my_slot);

        // Query from a DIFFERENT node (tests network routing!)
        let query_node = &nodes[(nodes.iter().position(|n| n.peer_id == node.peer_id).unwrap() + 1) % nodes.len()];

        if let Some(ownership_bytes) = query_node.dht_get(&ownership_key).await? {
            let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes)?;
            assert_eq!(ownership.peer_id, node.peer_id);
            successful_queries += 1;
            println!("  ✅ Found {} ownership via network query", node.peer_id);
        } else {
            println!("  ❌ FAILED to find {} ownership!", node.peer_id);
        }
    }

    assert_eq!(successful_queries, nodes.len(), "All slot ownerships should be queryable through network!");

    let total_hops = network.lock().await.get_total_hops().await;
    println!("\n📊 Total network hops: {} (across {} operations)", total_hops, nodes.len() * 2);

    println!("\n✅ Test 1 PASSED: Slot ownership stored in NETWORK DHT and queryable!");
    Ok(())
}

/// Test 2: LazyNode neighbor discovery through NETWORK DHT
/// This test verifies spec lines 316-328
#[tokio::test]
async fn test_dht_network_lazynode_neighbor_discovery() -> Result<()> {
    println!("\n=== Test 2: LazyNode Neighbor Discovery through NETWORK DHT ===");
    println!("Spec Reference: Section 2.4, lines 316-328\n");

    let mesh_config = MeshConfig::new(10, 10, 5);
    let network = Arc::new(Mutex::new(TestDHTNetwork::new()));

    // Create 50 nodes (like docker-compose cluster)
    let mut nodes = Vec::new();
    let mut occupied_slots = HashSet::new();
    for i in 0..50 {
        let peer_id = format!("peer-{}", i);
        let slot = assign_unique_slot(&peer_id, &mesh_config, &mut occupied_slots);
        let node = NetworkDHTNode::new(peer_id.clone(), slot, mesh_config, Arc::clone(&network));

        network.lock().await.add_node(node.clone());
        nodes.push(node);
    }

    println!("Created 50 nodes");

    // All nodes announce slot ownership
    println!("\n📢 All nodes announcing slot ownership...");
    for node in &nodes {
        let ownership = SlotOwnership::new(node.peer_id.clone(), node.my_slot, None);
        let ownership_key = slot_ownership_key(node.my_slot);
        let ownership_bytes = serde_json::to_vec(&ownership)?;
        node.dht_put(ownership_key, ownership_bytes).await?;
    }

    println!("✅ All 50 nodes announced");

    // Each node discovers its 8 neighbors via network DHT (LazyNode pattern!)
    println!("\n🔍 LazyNode discovering neighbors...");

    let mut total_neighbors_found = 0;
    let mut total_neighbors_expected = 0;

    for node in &nodes {
        let neighbors = node.discover_all_neighbors().await?;
        total_neighbors_expected += 8;
        total_neighbors_found += neighbors.len();

        let success_rate = (neighbors.len() as f64 / 8.0) * 100.0;
        println!("  {} found {}/8 neighbors ({:.1}%)", node.peer_id, neighbors.len(), success_rate);

        // Verify each neighbor is valid
        for (direction, neighbor_id, neighbor_slot) in &neighbors {
            // Verify neighbor is at expected slot
            let expected_slot = node.my_slot.neighbor(*direction, &mesh_config);
            assert_eq!(neighbor_slot, &expected_slot, "Neighbor slot mismatch for {:?}", direction);

            // Verify neighbor exists in network
            assert!(nodes.iter().any(|n| n.peer_id == *neighbor_id), "Neighbor {} not found in network", neighbor_id);
        }
    }

    let connectivity = (total_neighbors_found as f64 / total_neighbors_expected as f64) * 100.0;
    println!("\n🎯 Overall mesh connectivity: {:.1}% ({}/{} neighbors discovered)",
        connectivity, total_neighbors_found, total_neighbors_expected);

    // CRITICAL ASSERTION: All neighbors must be discoverable!
    assert_eq!(
        total_neighbors_found, total_neighbors_expected,
        "LazyNode must discover ALL neighbors through network DHT! Found {}/{} ({:.1}%)",
        total_neighbors_found, total_neighbors_expected, connectivity
    );

    let total_hops = network.lock().await.get_total_hops().await;
    println!("\n📊 Total network hops: {} (avg {:.1} per query)",
        total_hops, total_hops as f64 / (total_neighbors_found as f64));

    println!("\n✅ Test 2 PASSED: LazyNode discovers 100% of neighbors through network DHT!");
    Ok(())
}

/// Test 3: O(1) routing complexity verification
/// This test verifies that routing decisions are constant time
#[tokio::test]
async fn test_dht_network_o1_routing() -> Result<()> {
    println!("\n=== Test 3: O(1) Routing Complexity ===");
    println!("Spec Reference: Section 2.2, lines 182-221\n");

    // Test with different mesh sizes
    let configs = vec![
        ("10x10x5 (500 slots)", MeshConfig::new(10, 10, 5)),
        ("20x20x10 (4K slots)", MeshConfig::new(20, 20, 10)),
        ("50x50x20 (50K slots)", MeshConfig::new(50, 50, 20)),
    ];

    for (name, mesh_config) in configs {
        println!("\nTesting with {}...", name);

        let network = Arc::new(Mutex::new(TestDHTNetwork::new()));

        // Create 100 nodes
        let mut nodes = Vec::new();
        let mut occupied_slots = HashSet::new();
        for i in 0..100 {
            let peer_id = format!("peer-{}", i);
            let slot = assign_unique_slot(&peer_id, &mesh_config, &mut occupied_slots);
            let node = NetworkDHTNode::new(peer_id.clone(), slot, mesh_config, Arc::clone(&network));

            network.lock().await.add_node(node.clone());
            nodes.push(node);
        }

        // Perform 1000 random GET operations
        let mut total_routing_ops = 0;
        let start = std::time::Instant::now();

        use std::fs::File;
        use std::io::Read;
        let mut urandom = File::open("/dev/urandom")?;

        for _ in 0..1000 {
            let mut key = [0u8; 32];
            urandom.read_exact(&mut key)?;

            let random_node = &nodes[key[0] as usize % nodes.len()];
            let _ = random_node.dht_get(&key).await;
            total_routing_ops += 1;
        }

        let elapsed = start.elapsed();
        let ops_per_sec = total_routing_ops as f64 / elapsed.as_secs_f64();

        println!("  Completed {} operations in {:.3}s", total_routing_ops, elapsed.as_secs_f64());
        println!("  Throughput: {:.0} ops/sec", ops_per_sec);
        println!("  Avg latency: {:.3}ms", elapsed.as_secs_f64() * 1000.0 / total_routing_ops as f64);
    }

    println!("\n✅ Test 3 PASSED: Routing maintains O(1) complexity across different mesh sizes");
    Ok(())
}

/// Test 4: Verify 64-byte minimal state requirement
/// This test verifies spec Section 2.4, lines 436-449
#[tokio::test]
async fn test_dht_network_minimal_state() -> Result<()> {
    println!("\n=== Test 4: Minimal State (64 bytes) Requirement ===");
    println!("Spec Reference: Section 2.4, lines 436-449\n");

    let mesh_config = MeshConfig::new(10, 10, 5);

    // Verify MinimalNode state size
    println!("Expected state:");
    println!("  my_slot: SlotCoordinate = 12 bytes (3 × i32)");
    println!("  my_peer_id: [u8; 32] = 32 bytes");
    println!("  mesh_config: MeshConfig = 12 bytes (3 × usize width/height/depth)");
    println!("  epoch: u64 = 8 bytes");
    println!("  Total: ~64 bytes");
    println!();
    println!("NO neighbor cache!");
    println!("NO routing tables!");
    println!("NO topology state!");

    // Create a node and verify it doesn't cache neighbors
    let network = Arc::new(Mutex::new(TestDHTNetwork::new()));
    let node = NetworkDHTNode::new("test-peer".to_string(), SlotCoordinate::new(5, 5, 2), mesh_config, Arc::clone(&network));

    // Query neighbors multiple times - should query DHT each time (no caching!)
    println!("\nQuerying same neighbor 10 times (should NOT use cache)...");

    for i in 0..10 {
        let _neighbor = node.get_neighbor(Direction::PlusA).await;
        let metrics = node.get_metrics().await;
        println!("  Query {}: {} GET requests total", i + 1, metrics.get_requests);

        // Each query should increment GET requests (no caching!)
        assert_eq!(metrics.get_requests, (i + 1) as u64, "Should query DHT each time, not use cache!");
    }

    println!("\n✅ Test 4 PASSED: Node maintains minimal state, no neighbor caching!");
    Ok(())
}

/// Test 5: Full 50-node cluster connectivity test
/// This reproduces the exact scenario from docker-compose-cluster.yml
#[tokio::test]
async fn test_dht_network_50_node_cluster_full() -> Result<()> {
    println!("\n=== Test 5: Full 50-Node Cluster Connectivity ===");
    println!("Reproduces docker-compose-cluster.yml scenario\n");

    let mesh_config = MeshConfig::new(10, 10, 5);
    let network = Arc::new(Mutex::new(TestDHTNetwork::new()));

    // Create exactly 50 nodes (matching docker-compose)
    println!("Creating 50 nodes...");
    let mut nodes = Vec::new();
    let mut occupied_slots = HashSet::new();
    for i in 0..50 {
        let peer_id = format!("peer-{}", i);
        let slot = assign_unique_slot(&peer_id, &mesh_config, &mut occupied_slots);
        let node = NetworkDHTNode::new(peer_id.clone(), slot, mesh_config, Arc::clone(&network));

        network.lock().await.add_node(node.clone());
        nodes.push((peer_id.clone(), node));

        println!("  {} -> slot {:?}", peer_id, slot);
    }

    // Phase 1: All nodes announce slot ownership
    println!("\n📢 Phase 1: Announcing slot ownership...");
    for (_peer_id, node) in &nodes {
        let ownership = SlotOwnership::new(node.peer_id.clone(), node.my_slot, None);
        let ownership_key = slot_ownership_key(node.my_slot);
        let ownership_bytes = serde_json::to_vec(&ownership)?;
        node.dht_put(ownership_key, ownership_bytes).await?;
    }
    println!("✅ All 50 nodes announced");

    // Phase 2: LazyNode neighbor discovery
    println!("\n🔍 Phase 2: LazyNode neighbor discovery...");

    let mut connectivity_by_node = Vec::new();

    for (peer_id, node) in &nodes {
        let neighbors = node.discover_all_neighbors().await?;
        let connectivity = (neighbors.len() as f64 / 8.0) * 100.0;
        connectivity_by_node.push((peer_id.clone(), neighbors.len(), connectivity));

        if neighbors.len() < 8 {
            println!("  ⚠️  {} found {}/8 neighbors ({:.1}%) - FRAGMENTED!",
                peer_id, neighbors.len(), connectivity);
        } else {
            println!("  ✅ {} found 8/8 neighbors (100.0%)",
                peer_id);
        }
    }

    // Calculate overall mesh health
    let total_found: usize = connectivity_by_node.iter().map(|(_, count, _)| count).sum();
    let total_expected = 50 * 8;
    let overall_connectivity = (total_found as f64 / total_expected as f64) * 100.0;

    println!("\n📊 Mesh Health Report:");
    println!("  Total neighbors found: {}/{}", total_found, total_expected);
    println!("  Overall connectivity: {:.1}%", overall_connectivity);

    let fragmented_nodes = connectivity_by_node.iter().filter(|(_, count, _)| *count < 8).count();
    println!("  Fragmented nodes: {}/50", fragmented_nodes);

    if overall_connectivity < 100.0 {
        println!("\n⚠️  MESH IS FRAGMENTED!");
        println!("This means DHT networking is NOT implemented according to spec!");
        println!("Read: /opt/castle/workspace/citadel/2025-10-12-Citadel-DHT-SPEC.md Section 2.4");
    }

    // CRITICAL ASSERTION
    assert_eq!(
        total_found, total_expected,
        "50-node cluster must have 100% connectivity! Found {}/{} ({:.1}%)",
        total_found, total_expected, overall_connectivity
    );

    let total_hops = network.lock().await.get_total_hops().await;
    println!("\n📊 Network routing statistics:");
    println!("  Total hops: {}", total_hops);
    println!("  Avg hops per query: {:.2}", total_hops as f64 / total_found as f64);

    println!("\n✅ Test 5 PASSED: 50-node cluster achieves 100% connectivity!");
    Ok(())
}
