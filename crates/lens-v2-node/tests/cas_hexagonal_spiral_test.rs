//! Content-Addressed Slots (CAS) - Hexagonal Spiral Algorithm Tests
//!
//! Tests for the NEW hexagonal spiral CAS implementation that:
//! 1. Generates N slots for N nodes (no collisions)
//! 2. Spirals from (0,0,0) in hexagonal pattern
//! 3. Maintains 5:1 width-to-depth ratio
//! 4. Ranks slots by neighbor count (8 > 7 > 6)
//! 5. Assigns nodes to best available slots
//! 6. Supports dynamic pruning (future)

use citadel_core::topology::SlotCoordinate;
use std::collections::{HashMap, HashSet};

// Import the actual implementations from peer_registry
use lens_node::peer_registry::{
    generate_available_slots,
    count_neighbors,
    rank_slots_by_neighbors,
    assign_slot,
    assign_slots_batch,
};

/// Generate a deterministic peer_id from an index (blake3 double hash of ed25519 pubkey)
fn generate_peer_id(index: usize) -> String {
    let pubkey = format!("ed25519-pubkey-{}", index);
    let hash1 = blake3::hash(pubkey.as_bytes());
    let hash2 = blake3::hash(hash1.as_bytes());
    hex::encode(hash2.as_bytes())
}

// ============================================================================
// TEST SUITE 1: Hexagonal Spiral Generation
// ============================================================================

#[test]
fn test_spiral_starts_at_origin() {
    let slots = generate_available_slots(1);
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0], SlotCoordinate::new(0, 0, 0),
        "First slot should be at origin (0,0,0)");
}

#[test]
fn test_spiral_first_ring_six_neighbors() {
    // First ring around origin should have 6 slots in hexagonal pattern
    let slots = generate_available_slots(7);
    assert_eq!(slots.len(), 7, "Should have 7 slots (origin + 6 neighbors)");

    // Origin should be first
    assert_eq!(slots[0], SlotCoordinate::new(0, 0, 0));

    // DEBUG: Print what we actually got
    println!("Generated slots:");
    for (i, slot) in slots.iter().enumerate() {
        println!("  {}: ({}, {}, {})", i, slot.x, slot.y, slot.z);
    }

    // Verify the 6 neighbors form a hexagonal ring
    // In hexagonal coordinates, the 6 neighbors are:
    // (+1, 0), (+1, -1), (0, -1), (-1, 0), (-1, +1), (0, +1)
    let expected_ring = vec![
        SlotCoordinate::new(1, 0, 0),
        SlotCoordinate::new(1, -1, 0),
        SlotCoordinate::new(0, -1, 0),
        SlotCoordinate::new(-1, 0, 0),
        SlotCoordinate::new(-1, 1, 0),
        SlotCoordinate::new(0, 1, 0),
    ];

    for expected in expected_ring {
        assert!(slots.contains(&expected),
            "First ring should contain slot {:?}", expected);
    }
}

#[test]
fn test_spiral_generates_exact_node_count() {
    // For any node count, should generate exactly that many slots
    for node_count in [1, 7, 19, 37, 61, 100, 114, 200] {
        let slots = generate_available_slots(node_count);
        assert_eq!(slots.len(), node_count,
            "Should generate exactly {} slots for {} nodes", node_count, node_count);
    }
}

#[test]
fn test_spiral_no_duplicate_slots() {
    let slots = generate_available_slots(100);
    let unique_slots: HashSet<_> = slots.iter().collect();
    assert_eq!(unique_slots.len(), slots.len(),
        "All generated slots should be unique (no duplicates)");
}

// ============================================================================
// TEST SUITE 2: 5:1 Width-to-Depth Ratio
// ============================================================================

#[test]
fn test_maintains_5_to_1_ratio_small_mesh() {
    // With few nodes, Z layer growth is acceptable
    let slots = generate_available_slots(37);

    let max_z = slots.iter().map(|s| s.z).max().unwrap();
    assert!(max_z <= 1, "With 37 nodes, should have Z <= 1 layer");
}

#[test]
fn test_triggers_new_z_layer_at_correct_ratio() {
    // When hex diameter reaches 5*depth, should start new Z layer
    // A hex with diameter ~11 should trigger Z=1
    // Approximate node count for diameter 11 (5 rings): ~61 nodes
    let slots = generate_available_slots(62);

    let max_z = slots.iter().map(|s| s.z).max().unwrap();
    assert!(max_z >= 1,
        "With 62 nodes, should have triggered Z=1 layer (5:1 ratio)");
}

#[test]
fn test_maintains_5_to_1_ratio_large_mesh() {
    let slots = generate_available_slots(200);

    let max_x = slots.iter().map(|s| s.x.abs()).max().unwrap();
    let max_y = slots.iter().map(|s| s.y.abs()).max().unwrap();
    let max_z = slots.iter().map(|s| s.z).max().unwrap();

    // Hex diameter is roughly 2*max(max_x, max_y) + 1
    let hex_diameter = 2 * max_x.max(max_y) + 1;
    let depth = max_z + 1;

    let ratio = hex_diameter as f32 / depth as f32;
    assert!(ratio >= 3.0 && ratio <= 7.0,
        "Should maintain reasonable width-to-depth ratio, got {}:1", ratio);
}

#[test]
fn test_completes_z_layer_before_stopping() {
    // When generating slots, should complete entire Z layer
    // Don't leave half-filled Z layers
    let slots = generate_available_slots(100);

    // Count nodes per Z layer
    let mut z_counts: HashMap<i32, usize> = HashMap::new();
    for slot in &slots {
        *z_counts.entry(slot.z).or_insert(0) += 1;
    }

    // If we have multiple Z layers, verify each layer (except possibly the last)
    // is "reasonably full" (not just 1-2 nodes)
    if z_counts.len() > 1 {
        let max_z = *z_counts.keys().max().unwrap();
        for z in 0..max_z {
            let count = z_counts.get(&z).unwrap_or(&0);
            assert!(*count > 6,
                "Z layer {} should have more than 6 nodes, got {}", z, count);
        }
    }
}

// ============================================================================
// TEST SUITE 3: Neighbor Counting and Ranking
// ============================================================================

#[test]
fn test_count_neighbors_origin() {
    // Origin with first ring should have 6 neighbors
    let slots = generate_available_slots(7);
    let occupied: HashSet<_> = slots.iter().cloned().collect();

    let origin = SlotCoordinate::new(0, 0, 0);
    let neighbor_count = count_neighbors(&origin, &occupied);

    assert_eq!(neighbor_count, 6,
        "Origin surrounded by first ring should have 6 horizontal neighbors");
}

#[test]
fn test_count_neighbors_with_vertical() {
    // A slot with neighbors above and below should count them
    let mut occupied = HashSet::new();
    let center = SlotCoordinate::new(0, 0, 1);
    occupied.insert(center);
    occupied.insert(SlotCoordinate::new(0, 0, 0)); // Below
    occupied.insert(SlotCoordinate::new(0, 0, 2)); // Above

    let neighbor_count = count_neighbors(&center, &occupied);
    assert!(neighbor_count >= 2,
        "Should count vertical neighbors (above/below)");
}

#[test]
fn test_rank_slots_by_neighbor_count() {
    let slots = generate_available_slots(20);
    let ranked = rank_slots_by_neighbors(&slots);

    assert_eq!(ranked.len(), slots.len(),
        "Ranked list should have same length as input");

    // Verify descending order by neighbor count
    for i in 0..ranked.len()-1 {
        assert!(ranked[i].1 >= ranked[i+1].1,
            "Slots should be ranked by neighbor count (descending)");
    }
}

#[test]
fn test_rank_prefers_eight_neighbor_slots() {
    let slots = generate_available_slots(50);
    let ranked = rank_slots_by_neighbors(&slots);

    // Top-ranked slots should have 8 neighbors (6 horizontal + 2 vertical)
    // Or at least close to 8
    let top_slot_neighbors = ranked[0].1;
    assert!(top_slot_neighbors >= 6,
        "Top-ranked slot should have at least 6 neighbors, got {}", top_slot_neighbors);
}

// ============================================================================
// TEST SUITE 4: Collision-Free Slot Assignment
// ============================================================================

#[test]
fn test_no_collisions_with_10_nodes() {
    let node_count = 10;

    // Generate proper peer_ids
    let peer_ids: Vec<String> = (0..node_count).map(|i| generate_peer_id(i)).collect();
    let assignments = assign_slots_batch(&peer_ids);

    println!("\nAssigning {} nodes:", node_count);
    for (peer_id, slot) in &assignments {
        println!("{}: ({}, {}, {})", peer_id, slot.x, slot.y, slot.z);
    }

    // Verify all unique slots
    let unique_slots = assignments.values().collect::<HashSet<_>>().len();
    assert_eq!(unique_slots, node_count,
        "All {} nodes should have unique slots", node_count);
}

#[test]
fn test_no_collisions_with_114_nodes() {
    // THE BIG TEST - this is the current production problem!
    // 114 nodes currently collide into 55 unique slots
    // This test verifies the NEW algorithm prevents collisions

    let node_count = 114;

    // Generate proper peer_ids
    let peer_ids: Vec<String> = (0..node_count).map(|i| generate_peer_id(i)).collect();
    let assignments = assign_slots_batch(&peer_ids);

    // Verify ALL slots are unique (no collisions)
    let unique_slots = assignments.values().collect::<HashSet<_>>().len();
    assert_eq!(unique_slots, node_count,
        "Expected {} unique slots, got {} (collision detected!)", node_count, unique_slots);
}

#[test]
fn test_deterministic_assignment() {
    // Same peer_id should always get same slot (for same node count)
    let peer_id = "test-peer-12345";
    let node_count = 50;

    let slot1 = assign_slot(peer_id, node_count);
    let slot2 = assign_slot(peer_id, node_count);
    let slot3 = assign_slot(peer_id, node_count);

    assert_eq!(slot1, slot2);
    assert_eq!(slot2, slot3);
}

#[test]
fn test_different_peers_get_different_slots() {
    let node_count = 50;

    let slot1 = assign_slot("peer-A", node_count);
    let slot2 = assign_slot("peer-B", node_count);
    let slot3 = assign_slot("peer-C", node_count);

    assert_ne!(slot1, slot2);
    assert_ne!(slot2, slot3);
    assert_ne!(slot1, slot3);
}

// ============================================================================
// TEST SUITE 5: Neighbor Preference in Assignment
// ============================================================================

#[test]
fn test_assigned_slots_prefer_high_neighbor_count() {
    // Assign 50 nodes and verify most get slots with good connectivity
    let node_count = 50;

    // Generate proper peer_ids
    let peer_ids: Vec<String> = (0..node_count).map(|i| generate_peer_id(i)).collect();
    let assignments = assign_slots_batch(&peer_ids);

    // Generate all available slots and rank them
    let all_slots = generate_available_slots(node_count);
    let ranked = rank_slots_by_neighbors(&all_slots);

    // Count how many assigned slots are in the top 50% of ranked slots
    let top_half_slots: HashSet<_> = ranked.iter()
        .take(node_count / 2)
        .map(|(slot, _)| slot)
        .collect();

    let well_connected = assignments.values()
        .filter(|slot| top_half_slots.contains(slot))
        .count();

    // At least 50% of nodes should get well-connected slots
    let percentage = (well_connected as f32 / node_count as f32) * 100.0;
    assert!(percentage >= 50.0,
        "Only {:.1}% of nodes got well-connected slots, expected >= 50%", percentage);
}

// ============================================================================
// TEST SUITE 6: Realistic Network Scenarios
// ============================================================================

#[test]
fn test_realistic_114_node_global_network() {
    // Simulate the current production scenario:
    // 100 Perth nodes + 3 test nodes + 11 existing = 114 nodes

    let node_count = 114;

    // Generate realistic peer_ids (blake3 double hash of ed25519 public key)
    let peer_ids: Vec<String> = (0..node_count)
        .map(|i| {
            // Simulate an ed25519 public key
            let pubkey = format!("ed25519-pubkey-{}", i);
            let hash1 = blake3::hash(pubkey.as_bytes());
            let hash2 = blake3::hash(hash1.as_bytes());
            hex::encode(hash2.as_bytes())
        })
        .collect();

    let mut assigned_slots = HashMap::new();
    let mut isolated_nodes = Vec::new();

    // Assign all nodes using batch assignment
    use lens_node::peer_registry::assign_slots_batch;
    let assignments = assign_slots_batch(&peer_ids);

    for (peer_id, slot) in &assignments {
        assigned_slots.insert(peer_id.clone(), *slot);
    }

    // Check connectivity for each node
    let all_slots: HashSet<_> = assigned_slots.values().cloned().collect();

    for (peer_id, slot) in &assigned_slots {
        let neighbor_count = count_neighbors(slot, &all_slots);

        if neighbor_count == 0 {
            isolated_nodes.push(peer_id.clone());
        }
    }

    // NO nodes should be isolated!
    assert_eq!(isolated_nodes.len(), 0,
        "Found {} isolated nodes: {:?}", isolated_nodes.len(), isolated_nodes);

    // Verify we have 114 unique slots
    assert_eq!(all_slots.len(), 114,
        "Expected 114 unique slots, got {}", all_slots.len());
}

#[test]
fn test_small_cluster_3_nodes() {
    // Test the docker-compose-single.yml scenario (3 nodes)
    let node_count = 3;

    // Generate proper peer_ids
    let peer_ids: Vec<String> = (0..node_count).map(|i| generate_peer_id(i)).collect();
    let assignments = assign_slots_batch(&peer_ids);

    // All 3 nodes should get unique slots
    let unique_slots = assignments.values().collect::<HashSet<_>>().len();
    assert_eq!(unique_slots, 3, "3 nodes should get 3 unique slots");

    // All slots should be close together (within first ring)
    let max_distance = assignments.values()
        .map(|slot| slot.x.abs() + slot.y.abs() + slot.z.abs())
        .max()
        .unwrap();

    assert!(max_distance <= 2,
        "With only 3 nodes, all should be within first ring (distance <= 2)");
}

#[test]
fn test_cluster_growth_no_collisions() {
    // Simulate cluster growing from 10 to 100 nodes
    // Verify no collisions at each step

    for node_count in [10, 20, 30, 50, 75, 100] {
        // Generate proper peer_ids
        let peer_ids: Vec<String> = (0..node_count).map(|i| generate_peer_id(i)).collect();
        let assignments = assign_slots_batch(&peer_ids);

        // Verify all unique slots
        let unique_slots = assignments.values().collect::<HashSet<_>>().len();
        assert_eq!(unique_slots, node_count,
            "At node_count={}, expected {} unique slots", node_count, node_count);
    }
}

// ============================================================================
// TEST SUITE 7: Edge Cases
// ============================================================================

#[test]
fn test_single_node_network() {
    let slots = generate_available_slots(1);
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0], SlotCoordinate::new(0, 0, 0));

    let slot = assign_slot("only-peer", 1);
    assert_eq!(slot, SlotCoordinate::new(0, 0, 0));
}

#[test]
fn test_two_node_network() {
    let slots = generate_available_slots(2);
    assert_eq!(slots.len(), 2);

    // Should be origin + one neighbor
    assert!(slots.contains(&SlotCoordinate::new(0, 0, 0)));

    let slot1 = assign_slot("peer-1", 2);
    let slot2 = assign_slot("peer-2", 2);
    assert_ne!(slot1, slot2);
}

#[test]
fn test_large_network_500_nodes() {
    // Stress test with large network
    let node_count = 500;
    let slots = generate_available_slots(node_count);

    assert_eq!(slots.len(), node_count);

    // Verify no duplicates
    let unique: HashSet<_> = slots.iter().collect();
    assert_eq!(unique.len(), node_count);

    // Verify reasonable 5:1 ratio maintained
    let max_x = slots.iter().map(|s| s.x.abs()).max().unwrap();
    let max_z = slots.iter().map(|s| s.z).max().unwrap();
    let ratio = (2 * max_x + 1) as f32 / (max_z + 1) as f32;

    assert!(ratio >= 4.0 && ratio <= 7.0,
        "Large network should maintain ~5:1 ratio, got {:.2}:1", ratio);
}

#[test]
#[should_panic(expected = "node_count must be > 0")]
fn test_zero_nodes_panics() {
    generate_available_slots(0);
}
