//! Neighbor Discovery via DHT Tests
//!
//! Validates that nodes discover neighbors by querying the DHT for slot ownership,
//! not by maintaining neighbor caches or using peer_id mappings.
//!
//! Key principles tested:
//! 1. Neighbors discovered by calculating neighbor slots geometrically
//! 2. DHT queried for "who owns this neighbor slot?"
//! 3. Lazy loading - no neighbor caches needed
//! 4. Recursive DHT - DHT uses itself for topology discovery

mod test_helpers;
use test_helpers::TestNode;

use std::time::Duration;
use citadel_core::topology::{SlotCoordinate, MeshConfig, Direction};
use lens_node::peer_registry::{slot_ownership_key, get_neighbor_slots, SlotOwnership};

#[tokio::test]
async fn test_discover_neighbor_via_dht_query() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Discover neighbor via DHT query");
    println!("   Validates lazy neighbor discovery through DHT\n");

    let mesh_config = MeshConfig::new(5, 5, 1);
    let my_slot = SlotCoordinate::new(2, 2, 0);
    let neighbor_slot = my_slot.neighbor(Direction::PlusA, &mesh_config); // (3, 2, 0)

    // Spawn two nodes: me and my neighbor
    println!("📝 Spawning node at (2,2,0)...");
    let node = TestNode::spawn_at_slot(31000, Some(my_slot)).await?;

    println!("📝 Spawning neighbor at (3,2,0)...");
    let neighbor = TestNode::spawn_at_slot(31001, Some(neighbor_slot)).await?;

    // Establish WebRTC connection FIRST (so DHT queries can route)
    println!("🔗 Establishing WebRTC connection between nodes...");
    node.establish_webrtc_connection(&neighbor).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // THEN announce slot ownership
    println!("📢 Announcing slot ownership...");
    node.announce_slot_ownership(my_slot).await?;
    neighbor.announce_slot_ownership(neighbor_slot).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Discover neighbor by querying DHT for neighbor slot
    println!("\n🔍 Querying DHT: Who owns slot (3,2,0)?");
    let ownership_key = slot_ownership_key(neighbor_slot);
    let ownership_bytes = node.dht_get(ownership_key).await?;

    assert!(ownership_bytes.is_some(), "❌ Neighbor not found in DHT!");

    let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes.unwrap())?;

    println!("✅ Found neighbor:");
    println!("   peer_id: {}", ownership.peer_id);
    println!("   slot: ({},{},{})", ownership.slot.x, ownership.slot.y, ownership.slot.z);

    assert_eq!(ownership.slot, neighbor_slot, "❌ Wrong neighbor slot!");

    println!("\n🎉 Neighbor discovered via DHT query (no caching needed)!\n");

    Ok(())
}

#[tokio::test]
async fn test_discover_all_eight_neighbors() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Discover all 8 neighbors via DHT");
    println!("   Validates 6 hexagonal + 2 vertical neighbor discovery\n");

    let mesh_config = MeshConfig::new(10, 10, 2);
    let center_slot = SlotCoordinate::new(0, 0, 0);

    // Spawn center node
    println!("📝 Spawning center node at (0,0,0)...");
    let center = TestNode::spawn_at_slot(31100, Some(center_slot)).await?;

    // Get all 8 neighbor slots
    let neighbors = get_neighbor_slots(&center_slot, &mesh_config);

    println!("📝 8 neighbor slots:");
    for (dir, slot) in &neighbors {
        println!("   {:?} → ({},{},{})", dir, slot.x, slot.y, slot.z);
    }

    // Spawn nodes at all 8 neighbor positions
    let mut neighbor_nodes = Vec::new();
    for (i, (_dir, slot)) in neighbors.iter().enumerate() {
        let node = TestNode::spawn_at_slot(31101 + i as u16, Some(*slot)).await?;
        neighbor_nodes.push(node);
    }

    // Establish WebRTC connections between center and all neighbors
    println!("🔗 Establishing WebRTC connections...");
    for neighbor in &neighbor_nodes {
        center.establish_webrtc_connection(neighbor).await?;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;

    // THEN announce slot ownership
    println!("📢 Announcing slot ownership...");
    center.announce_slot_ownership(center_slot).await?;
    for (i, (_dir, slot)) in neighbors.iter().enumerate() {
        neighbor_nodes[i].announce_slot_ownership(*slot).await?;
    }
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Discover all neighbors via DHT queries
    println!("\n🔍 Discovering all 8 neighbors via DHT queries...");
    let mut discovered = 0;

    for (dir, neighbor_slot) in neighbors {
        let ownership_key = slot_ownership_key(neighbor_slot);
        let ownership_bytes = center.dht_get(ownership_key).await?;

        if let Some(bytes) = ownership_bytes {
            let ownership: SlotOwnership = serde_json::from_slice(&bytes)?;
            println!("   ✅ {:?}: peer_id {}", dir, ownership.peer_id);
            discovered += 1;
        } else {
            println!("   ❌ {:?}: not found!", dir);
        }
    }

    assert_eq!(discovered, 8, "❌ Should discover all 8 neighbors!");

    println!("\n🎉 All 8 neighbors discovered via DHT queries!");
    println!("🎉 Lazy loading works - no neighbor caches needed!\n");

    Ok(())
}

#[tokio::test]
async fn test_neighbor_discovery_with_toroidal_wraparound() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Neighbor discovery with toroidal wraparound");
    println!("   Validates that neighbors at mesh edges wrap correctly\n");

    let mesh_config = MeshConfig::new(5, 5, 2);
    let edge_slot = SlotCoordinate::new(4, 4, 1); // At edge of mesh

    // Spawn node at edge
    println!("📝 Spawning node at edge (4,4,1)...");
    let edge_node = TestNode::spawn_at_slot(31200, Some(edge_slot)).await?;

    // Calculate neighbor in PlusA direction (should wrap to x=0)
    let neighbor_slot = edge_slot.neighbor(Direction::PlusA, &mesh_config);
    println!("📐 PlusA from (4,4,1) wraps to: ({},{},{})",
        neighbor_slot.x, neighbor_slot.y, neighbor_slot.z);

    assert_eq!(neighbor_slot.x, 0, "❌ Should wrap to x=0!");

    // Spawn wrapped neighbor
    let neighbor = TestNode::spawn_at_slot(31201, Some(neighbor_slot)).await?;

    // Establish WebRTC connection FIRST
    println!("🔗 Establishing WebRTC connection...");
    edge_node.establish_webrtc_connection(&neighbor).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // THEN announce slot ownership
    println!("📢 Announcing slot ownership...");
    edge_node.announce_slot_ownership(edge_slot).await?;
    neighbor.announce_slot_ownership(neighbor_slot).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Discover wrapped neighbor via DHT
    println!("\n🔍 Discovering wrapped neighbor...");
    let ownership_key = slot_ownership_key(neighbor_slot);
    let ownership_bytes = edge_node.dht_get(ownership_key).await?;

    assert!(ownership_bytes.is_some(), "❌ Wrapped neighbor not found!");

    let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes.unwrap())?;
    println!("✅ Found wrapped neighbor: peer_id {}", ownership.peer_id);

    println!("\n🎉 Toroidal wraparound works correctly!");
    println!("🎉 Edge nodes discover neighbors across wraparound!\n");

    Ok(())
}

#[tokio::test]
async fn test_no_neighbor_cache_needed() -> anyhow::Result<()> {
    println!("\n🧪 TEST: No neighbor cache needed (pure lazy loading)");
    println!("   Validates that neighbors can be queried repeatedly without caching\n");

    let mesh_config = MeshConfig::new(5, 5, 1);
    let my_slot = SlotCoordinate::new(2, 2, 0);
    let neighbor_slot = my_slot.neighbor(Direction::PlusB, &mesh_config);

    // Spawn nodes
    let node = TestNode::spawn_at_slot(31300, Some(my_slot)).await?;
    let neighbor = TestNode::spawn_at_slot(31301, Some(neighbor_slot)).await?;

    // Establish WebRTC connection FIRST
    println!("🔗 Establishing WebRTC connection...");
    node.establish_webrtc_connection(&neighbor).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // THEN announce slot ownership
    println!("📢 Announcing slot ownership...");
    node.announce_slot_ownership(my_slot).await?;
    neighbor.announce_slot_ownership(neighbor_slot).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let ownership_key = slot_ownership_key(neighbor_slot);

    // Query same neighbor multiple times (no cache, fresh DHT queries each time)
    println!("🔍 Querying same neighbor 5 times (no caching)...");
    for i in 1..=5 {
        let ownership_bytes = node.dht_get(ownership_key).await?;
        assert!(ownership_bytes.is_some(), "❌ Query {} failed!", i);

        let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes.unwrap())?;
        println!("   ✅ Query {}: Found peer_id {}", i, ownership.peer_id);
    }

    println!("\n🎉 Pure lazy loading works!");
    println!("🎉 No neighbor cache needed - DHT is the cache!\n");

    Ok(())
}

#[tokio::test]
async fn test_neighbor_leaves_and_is_replaced() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Neighbor leaves and slot is claimed by new peer");
    println!("   Validates dynamic slot ownership changes\n");

    let mesh_config = MeshConfig::new(5, 5, 1);
    let my_slot = SlotCoordinate::new(2, 2, 0);
    let neighbor_slot = my_slot.neighbor(Direction::PlusC, &mesh_config);

    // Spawn me and original neighbor
    let node = TestNode::spawn_at_slot(31400, Some(my_slot)).await?;

    println!("📝 Original neighbor claims slot...");
    let neighbor_a = TestNode::spawn_at_slot(31401, Some(neighbor_slot)).await?;

    // Establish WebRTC connection FIRST
    println!("🔗 Establishing WebRTC connection...");
    node.establish_webrtc_connection(&neighbor_a).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // THEN announce slot ownership
    println!("📢 Announcing slot ownership...");
    node.announce_slot_ownership(my_slot).await?;
    neighbor_a.announce_slot_ownership(neighbor_slot).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Query DHT for original neighbor
    let ownership_key = slot_ownership_key(neighbor_slot);
    let ownership_bytes_a = node.dht_get(ownership_key).await?;
    let ownership_a: SlotOwnership = serde_json::from_slice(&ownership_bytes_a.unwrap())?;
    println!("✅ Original neighbor: {}", ownership_a.peer_id);

    // Original neighbor leaves (simulated by new neighbor claiming slot)
    println!("\n📝 New neighbor claims the same slot...");
    let neighbor_b = TestNode::spawn_at_slot(31402, Some(neighbor_slot)).await?;

    // Establish WebRTC connection with new neighbor
    println!("🔗 Establishing WebRTC connection with new neighbor...");
    node.establish_webrtc_connection(&neighbor_b).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // New neighbor announces slot ownership
    println!("📢 New neighbor announcing slot ownership...");
    neighbor_b.announce_slot_ownership(neighbor_slot).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Query DHT again - should find new neighbor
    let ownership_bytes_b = node.dht_get(ownership_key).await?;
    let ownership_b: SlotOwnership = serde_json::from_slice(&ownership_bytes_b.unwrap())?;
    println!("✅ New neighbor: {}", ownership_b.peer_id);

    // Verify peer_ids are different
    assert_ne!(ownership_a.peer_id, ownership_b.peer_id,
        "❌ Should be different peers!");

    println!("\n🎉 Slot ownership dynamically updates!");
    println!("🎉 DHT reflects current slot occupancy!\n");

    Ok(())
}
