//! Peer Discovery Integration Test
//!
//! Tests that peer_id coordination and DHT-based neighbor discovery work correctly.
//! This test reproduces the 4.5% connectivity fragmentation bug.

use anyhow::Result;
use citadel_core::topology::{MeshConfig, SlotCoordinate};
use citadel_dht::local_storage::LocalStorage;
use lens_node::peer_registry::{peer_id_to_slot, slot_ownership_key, SlotOwnership};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test that peer_id consistently maps to the same slot
#[tokio::test]
async fn test_peer_id_to_slot_deterministic() -> Result<()> {
    println!("\n=== Test: peer_id_to_slot is deterministic ===");

    let mesh_config = MeshConfig::new(10, 10, 5);

    // Test multiple peer_ids
    for i in 0..10 {
        let peer_id = format!("peer-{}", i);

        // Call twice
        let slot1 = peer_id_to_slot(&peer_id, &mesh_config);
        let slot2 = peer_id_to_slot(&peer_id, &mesh_config);

        assert_eq!(slot1, slot2, "peer_id {} should map to same slot", peer_id);
        println!("  {} -> {:?}", peer_id, slot1);
    }

    println!("✅ Test passed: peer_id_to_slot is deterministic");
    Ok(())
}

/// Test that slot ownership keys are deterministic
#[tokio::test]
async fn test_slot_ownership_key_deterministic() -> Result<()> {
    println!("\n=== Test: slot_ownership_key is deterministic ===");

    let slot = SlotCoordinate::new(5, 5, 2);

    // Call twice
    let key1 = slot_ownership_key(slot);
    let key2 = slot_ownership_key(slot);

    assert_eq!(key1, key2, "Slot ownership key should be deterministic");
    println!("  Slot {:?} -> {}", slot, hex::encode(&key1[..8]));

    println!("✅ Test passed: slot_ownership_key is deterministic");
    Ok(())
}

/// Test that peers can announce and discover each other via DHT
/// This is the CRITICAL test that should expose the 4.5% connectivity bug!
#[tokio::test]
async fn test_peer_discovery_via_dht() -> Result<()> {
    println!("\n=== Test: Peer Discovery via DHT (Reproduces 4.5% Bug) ===");

    let mesh_config = MeshConfig::new(10, 10, 5);

    // Create 50 peers (like docker-compose cluster)
    let num_peers = 50;
    let mut peers: Vec<(String, SlotCoordinate)> = Vec::new();

    for i in 0..num_peers {
        let peer_id = format!("peer-{}", i);
        let my_slot = peer_id_to_slot(&peer_id, &mesh_config);
        peers.push((peer_id, my_slot));
        println!("  Peer {} at slot {:?}", i, my_slot);
    }

    // Create a shared DHT (simulating relay's DHT storage)
    let dht_storage = Arc::new(Mutex::new(LocalStorage::new()));

    // Step 1: All peers announce their slot ownership to DHT
    println!("\n📢 Step 1: Announcing slot ownership...");
    {
        let mut dht = dht_storage.lock().await;
        for (peer_id, my_slot) in &peers {
            let ownership = SlotOwnership::new(peer_id.clone(), *my_slot, None);
            let ownership_key = slot_ownership_key(*my_slot);
            let ownership_bytes = serde_json::to_vec(&ownership)?;

            dht.put(ownership_key, ownership_bytes.into());
            println!("  {} announced at {:?}", peer_id, my_slot);
        }
    }

    println!("✅ All {} peers announced their slot ownership", num_peers);

    // Step 2: Each peer queries DHT for its 8 neighbors
    println!("\n🔍 Step 2: Querying DHT for neighbors...");

    let mut total_neighbors_found = 0;
    let mut total_neighbors_expected = 0;

    for (peer_id, my_slot) in &peers {
        // Get the 8 neighbor slots
        let neighbor_slots = lens_node::peer_registry::get_neighbor_slots(my_slot, &mesh_config);
        total_neighbors_expected += neighbor_slots.len();

        let mut found = 0;

        let dht = dht_storage.lock().await;
        for (_direction, neighbor_slot) in &neighbor_slots {
            let ownership_key = slot_ownership_key(*neighbor_slot);

            // Query DHT for this neighbor
            if let Some(ownership_bytes) = dht.get(&ownership_key) {
                if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(&ownership_bytes) {
                    found += 1;
                    total_neighbors_found += 1;
                    println!("  {} found neighbor: {} at {:?}", peer_id, ownership.peer_id, neighbor_slot);
                }
            }
        }

        let success_rate = (found as f64 / neighbor_slots.len() as f64) * 100.0;
        println!("  {} found {}/{} neighbors ({:.1}%)", peer_id, found, neighbor_slots.len(), success_rate);
    }

    let connectivity = (total_neighbors_found as f64 / total_neighbors_expected as f64) * 100.0;
    println!("\n🎯 Overall connectivity: {:.1}% ({}/{} neighbors found)",
        connectivity, total_neighbors_found, total_neighbors_expected);

    // Assert 100% connectivity (this will FAIL, exposing the bug!)
    assert_eq!(
        total_neighbors_found, total_neighbors_expected,
        "Should find ALL neighbors in DHT. Found {}/{} ({:.1}%)",
        total_neighbors_found, total_neighbors_expected, connectivity
    );

    println!("✅ Test passed: 100% neighbor discovery via DHT");
    Ok(())
}

/// Test the exact scenario from sync_orchestrator.rs
#[tokio::test]
async fn test_sync_orchestrator_peer_discovery() -> Result<()> {
    println!("\n=== Test: Sync Orchestrator Peer Discovery ===");

    let mesh_config = MeshConfig::new(10, 10, 5);
    let dht_storage = Arc::new(Mutex::new(LocalStorage::new()));

    // Simulate 10 peers received via PeerReferral
    let peer_referrals = vec![
        ("peer-100", SlotCoordinate::new(3, 4, 2)),
        ("peer-200", SlotCoordinate::new(5, 1, 3)),
        ("peer-300", SlotCoordinate::new(7, 8, 1)),
        ("peer-400", SlotCoordinate::new(2, 6, 4)),
        ("peer-500", SlotCoordinate::new(9, 3, 0)),
        ("peer-600", SlotCoordinate::new(1, 9, 2)),
        ("peer-700", SlotCoordinate::new(4, 4, 3)),
        ("peer-800", SlotCoordinate::new(6, 7, 1)),
        ("peer-900", SlotCoordinate::new(8, 2, 4)),
        ("peer-1000", SlotCoordinate::new(0, 5, 0)),
    ];

    // Populate DHT with peer slot ownerships (like sync_orchestrator does)
    println!("📢 Populating DHT with {} peer referrals...", peer_referrals.len());
    {
        let mut dht = dht_storage.lock().await;
        for (peer_id, slot) in &peer_referrals {
            let ownership = SlotOwnership::new(peer_id.to_string(), *slot, None);
            let ownership_key = slot_ownership_key(*slot);
            let ownership_bytes = serde_json::to_vec(&ownership)?;
            dht.put(ownership_key, ownership_bytes.into());
            println!("  Stored {} at {:?} with key {}", peer_id, slot, hex::encode(&ownership_key[..8]));
        }
    }

    // Now query for neighbors of peer-500 (like LazyNode does)
    let my_peer_id = "peer-500";
    let my_slot = SlotCoordinate::new(9, 3, 0);

    println!("\n🔍 Querying neighbors of {} at {:?}...", my_peer_id, my_slot);

    let neighbor_slots = lens_node::peer_registry::get_neighbor_slots(&my_slot, &mesh_config);
    println!("  Expected {} neighbors", neighbor_slots.len());

    let mut found_neighbors = Vec::new();
    {
        let dht = dht_storage.lock().await;
        for (_direction, neighbor_slot) in &neighbor_slots {
            let ownership_key = slot_ownership_key(*neighbor_slot);
            println!("  Querying for neighbor at {:?} with key {}", neighbor_slot, hex::encode(&ownership_key[..8]));

            if let Some(ownership_bytes) = dht.get(&ownership_key) {
                if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(&ownership_bytes) {
                    found_neighbors.push(ownership.peer_id.clone());
                    println!("    ✅ Found: {} at {:?}", ownership.peer_id, neighbor_slot);
                }
            } else {
                println!("    ❌ NOT FOUND at {:?}", neighbor_slot);
            }
        }
    }

    println!("\n🎯 Found {}/{} neighbors: {:?}",
        found_neighbors.len(), neighbor_slots.len(), found_neighbors);

    // For this test, we just want to see the results, not assert
    // (since we know it will fail with current code)

    println!("✅ Test complete: Sync orchestrator peer discovery simulation");
    Ok(())
}
