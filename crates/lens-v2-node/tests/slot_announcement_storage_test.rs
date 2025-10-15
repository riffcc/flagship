//! Test that slot announcements are stored correctly in DHT
//!
//! This test verifies:
//! 1. When a node announces its slot ownership, it should be stored LOCALLY
//! 2. Slot ownership should NOT be routed away via key_to_slot()
//! 3. Neighbors should be able to query slot ownership via DHT GET
//! 4. Slot ownership keys should be exempt from normal DHT routing

use lens_node::routes::RelayState;
use lens_node::peer_registry::{peer_id_to_slot, slot_ownership_key, SlotOwnership};
use citadel_core::topology::{MeshConfig, SlotCoordinate};
use citadel_core::key_mapping::key_to_slot;

#[tokio::test]
async fn test_slot_announcement_stored_locally() {
    // Create DHT storage
    let dht_storage = std::sync::Arc::new(tokio::sync::Mutex::new(lens_node::dht_state::DhtState::new()));

    // Create RelayState with node peer_id
    let my_peer_id = "bafk2bzaceaabcdef1234567890abcdef1234567890abcdef1234567890abcd".to_string();
    let relay_state = RelayState::new()
        .with_node_peer_id(my_peer_id.clone())
        .with_dht_storage(dht_storage.clone());

    // Calculate my slot
    let mesh_config = MeshConfig::new(10, 10, 5);
    let my_slot = peer_id_to_slot(&my_peer_id, &mesh_config);

    // Create slot ownership announcement
    let ownership = SlotOwnership::new(my_peer_id.clone(), my_slot, None);
    let slot_key = slot_ownership_key(my_slot);
    let ownership_bytes = serde_json::to_vec(&ownership).unwrap();

    // Calculate where THIS key would route to via normal DHT routing
    let routing_target_slot = key_to_slot(&slot_key, &mesh_config);

    println!("My slot: {:?}", my_slot);
    println!("Slot ownership key would route to: {:?}", routing_target_slot);
    println!("Keys match: {}", my_slot == routing_target_slot);

    // Announce via dht_put (current implementation)
    relay_state.dht_put(slot_key, ownership_bytes.clone()).await;

    // Check if stored locally
    let stored_locally = {
        let storage = dht_storage.lock().await;
        storage.get_raw(&slot_key).is_some()
    };

    // EXPECTED: Should be stored locally regardless of where key_to_slot() says
    // ACTUAL: Only stored locally if key_to_slot(slot_key) == my_slot
    assert!(
        stored_locally,
        "Slot ownership should be stored LOCALLY, not routed away! \
         My slot: {:?}, Key routes to: {:?}",
        my_slot, routing_target_slot
    );
}

#[tokio::test]
async fn test_multiple_nodes_store_own_slots() {
    // Create shared DHT storage (simulating distributed DHT)
    let dht_storage = std::sync::Arc::new(tokio::sync::Mutex::new(lens_node::dht_state::DhtState::new()));

    let mesh_config = MeshConfig::new(10, 10, 5);

    // Create 3 nodes with different peer_ids
    let peer_ids = vec![
        "bafk2bzaceaabcdef1111111111111111111111111111111111111111111111".to_string(),
        "bafk2bzaceaabcdef2222222222222222222222222222222222222222222222".to_string(),
        "bafk2bzaceaabcdef3333333333333333333333333333333333333333333333".to_string(),
    ];

    // Each node announces its slot
    for peer_id in &peer_ids {
        let my_slot = peer_id_to_slot(peer_id, &mesh_config);
        let ownership = SlotOwnership::new(peer_id.clone(), my_slot, None);
        let slot_key = slot_ownership_key(my_slot);
        let ownership_bytes = serde_json::to_vec(&ownership).unwrap();

        // Store directly (bypassing routing for this test)
        {
            let mut storage = dht_storage.lock().await;
            storage.insert_raw(slot_key, ownership_bytes);
        }
    }

    // Verify all 3 slot announcements are stored
    let storage = dht_storage.lock().await;
    for peer_id in &peer_ids {
        let my_slot = peer_id_to_slot(peer_id, &mesh_config);
        let slot_key = slot_ownership_key(my_slot);

        assert!(
            storage.get_raw(&slot_key).is_some(),
            "Slot ownership for {} at {:?} should be stored in DHT",
            peer_id, my_slot
        );
    }

    // Verify total entries
    assert_eq!(
        storage.len(),
        3,
        "DHT should contain 3 slot ownership announcements"
    );
}
