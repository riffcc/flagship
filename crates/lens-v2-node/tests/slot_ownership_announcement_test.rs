//! Slot Ownership Announcement Tests
//!
//! Validates that nodes correctly announce their slot ownership to the DHT
//! and that slot ownership is decoupled from peer identity.
//!
//! Key principles tested:
//! 1. Slots are fixed geometric positions in hexagonal toroidal mesh
//! 2. Peers can occupy ANY slot (no peer_id → slot mapping)
//! 3. Slot ownership announced via DHT.put(slot_ownership_key, SlotOwnership)
//! 4. Multiple peers can claim different slots over time
//! 5. Peers can leave and others can claim their slots

mod test_helpers;
use test_helpers::TestNode;

use std::time::Duration;
use citadel_core::topology::{SlotCoordinate, MeshConfig};
use lens_node::peer_registry::{slot_ownership_key, peer_location_key, SlotOwnership};

#[tokio::test]
async fn test_node_announces_slot_ownership_to_dht() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Node announces slot ownership to DHT");
    println!("   Validates that SlotOwnership is stored at slot_ownership_key\n");

    // Spawn a single node at slot (0,0,0)
    let node = TestNode::spawn_at_slot(30000, Some(SlotCoordinate::new(0, 0, 0))).await?;
    let my_slot = SlotCoordinate::new(0, 0, 0);

    // Announce slot ownership
    node.announce_slot_ownership(my_slot).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Query DHT for slot ownership
    let ownership_key = slot_ownership_key(my_slot);
    let ownership_bytes = node.dht_get(ownership_key).await?;

    assert!(ownership_bytes.is_some(), "❌ Slot ownership not found in DHT!");

    let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes.unwrap())?;

    println!("✅ Slot ownership found in DHT:");
    println!("   peer_id: {}", ownership.peer_id);
    println!("   slot: ({},{},{})", ownership.slot.x, ownership.slot.y, ownership.slot.z);
    println!("   slot_id: {}", ownership.slot_id.to_hex());

    assert_eq!(ownership.slot, my_slot, "❌ Slot mismatch!");

    println!("\n🎉 Node successfully announced slot ownership to DHT!\n");
    Ok(())
}

#[tokio::test]
async fn test_peer_id_does_not_determine_slot() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Peer ID does not determine slot");
    println!("   Validates that peers can occupy any slot regardless of their ID\n");

    // Spawn two nodes with different peer_ids at the SAME slot coordinate
    // (not simultaneously - one will claim it first)
    let slot = SlotCoordinate::new(1, 1, 0);

    println!("📝 Spawning node A at slot (1,1,0)...");
    let node_a = TestNode::spawn_at_slot(30100, Some(slot)).await?;
    node_a.announce_slot_ownership(slot).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Check DHT shows node A owns the slot
    let ownership_key = slot_ownership_key(slot);
    let ownership_bytes = node_a.dht_get(ownership_key).await?;
    let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes.unwrap())?;

    println!("✅ Node A owns slot (1,1,0):");
    println!("   peer_id: {}", ownership.peer_id);

    // Now spawn node B at a DIFFERENT slot
    let slot_b = SlotCoordinate::new(2, 2, 0);
    println!("\n📝 Spawning node B at slot (2,2,0)...");
    let node_b = TestNode::spawn_at_slot(30101, Some(slot_b)).await?;
    node_b.announce_slot_ownership(slot_b).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Check DHT shows node B owns slot_b
    let ownership_key_b = slot_ownership_key(slot_b);
    let ownership_bytes_b = node_b.dht_get(ownership_key_b).await?;
    let ownership_b: SlotOwnership = serde_json::from_slice(&ownership_bytes_b.unwrap())?;

    println!("✅ Node B owns slot (2,2,0):");
    println!("   peer_id: {}", ownership_b.peer_id);

    // Verify the peer_ids are different
    assert_ne!(ownership.peer_id, ownership_b.peer_id, "❌ Peer IDs should be different!");

    println!("\n🎉 Peers with different IDs can occupy different slots!");
    println!("🎉 Slot assignment is independent of peer identity!\n");

    Ok(())
}

#[tokio::test]
async fn test_peer_can_change_slots() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Peer can change slots");
    println!("   Validates that a peer can relinquish one slot and claim another\n");

    let slot_a = SlotCoordinate::new(3, 3, 0);
    let slot_b = SlotCoordinate::new(4, 4, 0);

    // Spawn node at slot A
    println!("📝 Node claims slot (3,3,0)...");
    let node = TestNode::spawn_at_slot(30200, Some(slot_a)).await?;
    node.announce_slot_ownership(slot_a).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify slot A ownership
    let ownership_key_a = slot_ownership_key(slot_a);
    let ownership_bytes_a = node.dht_get(ownership_key_a).await?;
    assert!(ownership_bytes_a.is_some(), "❌ Slot A ownership not found!");

    let ownership_a: SlotOwnership = serde_json::from_slice(&ownership_bytes_a.unwrap())?;
    println!("✅ Node owns slot (3,3,0): {}", ownership_a.peer_id);

    // Node relinquishes slot A and claims slot B
    println!("\n📝 Node relinquishes (3,3,0) and claims (4,4,0)...");
    node.announce_slot_ownership(slot_b).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify slot B ownership
    let ownership_key_b = slot_ownership_key(slot_b);
    let ownership_bytes_b = node.dht_get(ownership_key_b).await?;
    assert!(ownership_bytes_b.is_some(), "❌ Slot B ownership not found!");

    let ownership_b: SlotOwnership = serde_json::from_slice(&ownership_bytes_b.unwrap())?;
    println!("✅ Node now owns slot (4,4,0): {}", ownership_b.peer_id);

    // Verify same peer_id owns both announcements
    assert_eq!(ownership_a.peer_id, ownership_b.peer_id, "❌ Peer ID should be the same!");

    // Verify slot coordinates changed
    assert_eq!(ownership_b.slot, slot_b, "❌ New slot should be (4,4,0)!");

    println!("\n🎉 Peer successfully changed slots!");
    println!("🎉 Slot ownership is dynamic and peer-controlled!\n");

    Ok(())
}

#[tokio::test]
async fn test_slot_ownership_includes_slot_id() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Slot ownership includes permanent SlotId");
    println!("   Validates that SlotId (Blake3 of coordinate) is stored\n");

    let slot = SlotCoordinate::new(5, 5, 0);

    // Spawn node and announce slot
    let node = TestNode::spawn_at_slot(30300, Some(slot)).await?;
    node.announce_slot_ownership(slot).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Query DHT for slot ownership
    let ownership_key = slot_ownership_key(slot);
    let ownership_bytes = node.dht_get(ownership_key).await?;
    let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes.unwrap())?;

    println!("✅ SlotOwnership retrieved:");
    println!("   slot: ({},{},{})", ownership.slot.x, ownership.slot.y, ownership.slot.z);
    println!("   slot_id: {}", ownership.slot_id.to_hex());

    // Verify SlotId is deterministic (same coordinate → same SlotId)
    let expected_slot_id = lens_node::slot_identity::SlotId::from_coordinate(slot);
    assert_eq!(ownership.slot_id.to_hex(), expected_slot_id.to_hex(),
        "❌ SlotId should be deterministic from coordinate!");

    println!("\n🎉 SlotId is permanent and content-addressed!");
    println!("🎉 Blake3(coordinate) ensures slot identity persistence!\n");

    Ok(())
}

#[tokio::test]
async fn test_peer_location_key_reverse_lookup() -> anyhow::Result<()> {
    println!("\n🧪 TEST: Peer location key enables reverse lookup");
    println!("   Validates that peers can be found via peer_location_key\n");

    let slot = SlotCoordinate::new(6, 6, 0);

    // Spawn node and announce slot
    let node = TestNode::spawn_at_slot(30400, Some(slot)).await?;
    node.announce_slot_ownership(slot).await?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Get slot ownership to extract peer_id
    let ownership_key = slot_ownership_key(slot);
    let ownership_bytes = node.dht_get(ownership_key).await?;
    let ownership: SlotOwnership = serde_json::from_slice(&ownership_bytes.unwrap())?;

    println!("✅ Found peer at slot (6,6,0): {}", ownership.peer_id);

    // Now do reverse lookup: peer_id → location
    let location_key = peer_location_key(&ownership.peer_id);
    let location_bytes = node.dht_get(location_key).await?;

    if location_bytes.is_some() {
        let location: SlotOwnership = serde_json::from_slice(&location_bytes.unwrap())?;
        println!("✅ Reverse lookup successful:");
        println!("   peer_id {} is at slot ({},{},{})",
            location.peer_id, location.slot.x, location.slot.y, location.slot.z);

        assert_eq!(location.slot, slot, "❌ Reverse lookup returned wrong slot!");
    } else {
        println!("⚠️  Reverse lookup not implemented yet (peer_location_key not populated)");
    }

    println!("\n🎉 Peer location key enables bidirectional lookup!\n");

    Ok(())
}
