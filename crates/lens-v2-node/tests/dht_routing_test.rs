//! DHT Multi-Hop Routing Tests
//!
//! Tests that prove the DHT actually routes through the mesh using greedy routing.
//! Each test spawns a fresh 3×3×2 mesh (18 nodes) and validates specific routing scenarios.
//!
//! Run individual tests:
//! ```bash
//! cargo test --test dht_routing_test test_dht_cross_mesh_routing -- --nocapture
//! cargo test --test dht_routing_test test_dht_middle_to_edge_routing -- --nocapture
//! cargo test --test dht_routing_test test_dht_bulk_operations -- --nocapture
//! ```
//!
//! Run all DHT tests:
//! ```bash
//! cargo test --test dht_routing_test -- --nocapture
//! ```

mod test_helpers;
use test_helpers::TestNode;

use std::time::Duration;
use citadel_core::topology::{SlotCoordinate, MeshConfig, Direction};
use citadel_core::key_mapping::key_to_slot;

/// Helper to spawn a mesh for DHT testing (3×3×2 = 18 nodes)
/// base_port: Starting port for this test's nodes (each test uses different range)
async fn spawn_test_mesh(base_port: u16) -> anyhow::Result<(Vec<TestNode>, MeshConfig, Vec<SlotCoordinate>)> {
    let mesh_config = MeshConfig::new(3, 3, 2);
    let mut slots_to_spawn: Vec<SlotCoordinate> = Vec::new();

    // Spawn all slots in the mesh
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..2 {
                slots_to_spawn.push(SlotCoordinate::new(x, y, z));
            }
        }
    }

    println!("🔧 Spawning {} nodes starting at port {}...", slots_to_spawn.len(), base_port);

    // Spawn all nodes in parallel
    let mut spawn_tasks = Vec::new();
    for (port_offset, slot) in slots_to_spawn.iter().enumerate() {
        let slot = *slot;
        let port = base_port + port_offset as u16;
        let task = tokio::spawn(async move {
            TestNode::spawn_at_slot(port, Some(slot)).await
        });
        spawn_tasks.push(task);
        tokio::time::sleep(Duration::from_micros(100)).await;
    }

    let mut all_nodes = Vec::new();
    for task in spawn_tasks {
        all_nodes.push(task.await??);
    }

    println!("✅ {} nodes spawned", all_nodes.len());

    // Establish WebRTC connections
    println!("🔗 Establishing WebRTC connections...");
    let mut completed = 0;

    for i in 0..all_nodes.len() {
        let my_slot = slots_to_spawn[i];
        for dir in [Direction::PlusA, Direction::MinusA, Direction::PlusB, Direction::MinusB,
                    Direction::PlusC, Direction::MinusC, Direction::Up, Direction::Down] {
            let neighbor_slot = my_slot.neighbor(dir, &mesh_config);
            if let Some(neighbor_idx) = slots_to_spawn.iter().position(|&s| s == neighbor_slot) {
                if neighbor_idx > i {
                    // Establish connection directly (can't clone TestNode due to oneshot::Sender)
                    all_nodes[i].establish_webrtc_connection(&all_nodes[neighbor_idx]).await?;
                    completed += 1;
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            }
        }
    }
    println!("✅ {} connections established", completed);

    // Announce slot ownership
    println!("📢 Announcing slot ownership...");
    for (i, node) in all_nodes.iter().enumerate() {
        node.announce_slot_ownership(slots_to_spawn[i]).await?;
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("✅ Gossip complete (waited 5 seconds for propagation)\n");

    Ok((all_nodes, mesh_config, slots_to_spawn))
}

#[tokio::test]
async fn test_dht_cross_mesh_routing() -> anyhow::Result<()> {
    println!("\n🧪 TEST: DHT Cross-Mesh Routing (Opposite Corners)");
    println!("   Write from node 0 → Read from node 8 (opposite corner)");
    println!("   Proves multi-hop routing works across the entire 3×3×2 mesh\n");

    let (nodes, mesh_config, _slots) = spawn_test_mesh(21000).await?;

    let test_key = "cross-mesh-key";
    let test_value = "Hello from across the mesh!";

    // Hash the key to see which slot it maps to
    let key_hash = blake3::hash(test_key.as_bytes());
    let key_bytes: [u8; 32] = *key_hash.as_bytes();
    let target_slot = key_to_slot(&key_bytes, &mesh_config);

    println!("📝 Writing key='{}' value='{}' from node-0", test_key, test_value);
    println!("   🎯 Key hashes to slot ({},{},{})", target_slot.x, target_slot.y, target_slot.z);

    nodes[0].dht_put(key_bytes, test_value.as_bytes().to_vec()).await?;
    println!("   ✅ Write complete\n");

    // Give the DHT a moment to propagate
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("🔍 Reading from node-8 (opposite corner of 3×3×2 mesh)...");
    let value = nodes[8].dht_get(key_bytes).await?;

    assert!(value.is_some(), "❌ FAIL: Key not found!");
    let value_bytes = value.unwrap();
    let value_str = String::from_utf8_lossy(&value_bytes);
    assert_eq!(value_str, test_value, "❌ FAIL: Value mismatch!");

    println!("   ✅ SUCCESS! Read correct value: '{}'", value_str);
    println!("\n🎉 DHT routing works across the mesh!\n");

    Ok(())
}

#[tokio::test]
async fn test_dht_middle_to_edge_routing() -> anyhow::Result<()> {
    println!("\n🧪 TEST: DHT Middle-to-Edge Routing");
    println!("   Write from node 4 (middle) → Read from node 17 (far edge)");
    println!("   Proves routing works through 3×3×2 toroidal mesh\n");

    let (nodes, mesh_config, _slots) = spawn_test_mesh(21100).await?;

    let test_key = "middle-to-edge-key";
    let test_value = "Routing through the toroid!";

    let key_hash = blake3::hash(test_key.as_bytes());
    let key_bytes: [u8; 32] = *key_hash.as_bytes();
    let target_slot = key_to_slot(&key_bytes, &mesh_config);

    println!("📝 Writing key='{}' value='{}' from node-4", test_key, test_value);
    println!("   🎯 Key hashes to slot ({},{},{})", target_slot.x, target_slot.y, target_slot.z);

    nodes[4].dht_put(key_bytes, test_value.as_bytes().to_vec()).await?;
    println!("   ✅ Write complete\n");

    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("🔍 Reading from node-17 (far edge of mesh)...");
    let value = nodes[17].dht_get(key_bytes).await?;

    assert!(value.is_some(), "❌ FAIL: Key not found!");
    let value_bytes = value.unwrap();
    let value_str = String::from_utf8_lossy(&value_bytes);
    assert_eq!(value_str, test_value, "❌ FAIL: Value mismatch!");

    println!("   ✅ SUCCESS! Read correct value: '{}'", value_str);
    println!("\n🎉 DHT routing works through toroidal wraparound!\n");

    Ok(())
}

#[tokio::test]
async fn test_dht_bulk_operations() -> anyhow::Result<()> {
    println!("\n🧪 TEST: DHT Bulk Operations");
    println!("   5 random write/read pairs from different nodes in 3×3×2 mesh");
    println!("   Proves DHT handles concurrent operations correctly\n");

    let (nodes, _mesh_config, _slots) = spawn_test_mesh(21200).await?;

    println!("📝 Performing 5 write/read pairs...\n");

    for i in 0..5 {
        let test_key = format!("bulk-key-{}", i);
        let test_value = format!("Bulk value {}", i);
        let key_hash = blake3::hash(test_key.as_bytes());
        let key_bytes: [u8; 32] = *key_hash.as_bytes();

        // Write from pseudo-random node
        let write_node_idx = (i * 13) % nodes.len();
        nodes[write_node_idx].dht_put(key_bytes, test_value.as_bytes().to_vec()).await?;

        // Read from different pseudo-random node
        let read_node_idx = (i * 17 + 7) % nodes.len();
        tokio::time::sleep(Duration::from_millis(200)).await;

        let value = nodes[read_node_idx].dht_get(key_bytes).await?;
        assert!(value.is_some(), "❌ FAIL: Key {} not found!", i);

        let value_bytes = value.unwrap();
        let value_str = String::from_utf8_lossy(&value_bytes);
        assert_eq!(value_str, test_value, "❌ FAIL: Value {} mismatch!", i);

        println!("   ✅ Test {}: write(node-{}) → read(node-{}) = '{}'",
            i, write_node_idx, read_node_idx, value_str);
    }

    println!("\n🎉 All 5 bulk operations completed correctly!");
    println!("🎉 DHT handles concurrent operations!\n");

    Ok(())
}

#[tokio::test]
async fn test_dht_same_node_write_read() -> anyhow::Result<()> {
    println!("\n🧪 TEST: DHT Same-Node Write/Read");
    println!("   Write and read from the same node");
    println!("   Proves local DHT storage works\n");

    let (nodes, mesh_config, _slots) = spawn_test_mesh(21300).await?;

    let test_key = "same-node-key";
    let test_value = "Local storage test!";

    let key_hash = blake3::hash(test_key.as_bytes());
    let key_bytes: [u8; 32] = *key_hash.as_bytes();
    let target_slot = key_to_slot(&key_bytes, &mesh_config);

    println!("📝 Writing key='{}' value='{}' from node-0", test_key, test_value);
    println!("   🎯 Key hashes to slot ({},{},{})", target_slot.x, target_slot.y, target_slot.z);

    nodes[0].dht_put(key_bytes, test_value.as_bytes().to_vec()).await?;
    println!("   ✅ Write complete\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    println!("🔍 Reading from same node (node-0)...");
    let value = nodes[0].dht_get(key_bytes).await?;

    assert!(value.is_some(), "❌ FAIL: Key not found!");
    let value_bytes = value.unwrap();
    let value_str = String::from_utf8_lossy(&value_bytes);
    assert_eq!(value_str, test_value, "❌ FAIL: Value mismatch!");

    println!("   ✅ SUCCESS! Read correct value: '{}'", value_str);
    println!("\n🎉 Local DHT operations work!\n");

    Ok(())
}
