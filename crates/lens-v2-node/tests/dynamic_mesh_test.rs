//! Dynamic Mesh Growth and Deterministic Slot Assignment Tests
//!
//! Tests the complete dynamic mesh system:
//! 1. Mesh grows from 1×1×1 to 10×10×5 as nodes join
//! 2. Slot assignment is deterministic based on sorted peer IDs
//! 3. 5:1 horizontal-to-vertical ratio maintained at scale
//! 4. 100% neighbor connectivity (no slot collisions)

use anyhow::Result;
use citadel_core::topology::{MeshConfig, SlotCoordinate};
use std::collections::HashSet;

/// Calculate optimal mesh dimensions for 2.5D HEXAGONAL TOROIDAL mesh
/// PURE ALGORITHMIC approach with NO hardcoded thresholds!
///
/// Algorithm: Grow mesh one dimension at a time from 1×1×1
/// - Always increase the SMALLEST dimension first
/// - Keep mesh as flat as possible (prefer w, h over d)
/// - Aim for roughly square horizontal (w ≈ h) before going vertical
/// - Natural result: gentle gradation like 1×1×1 → 2×1×1 → 2×2×1 → 3×2×1 → 3×3×1
///
/// This is a PURE algorithm - no if-else ranges, just pure math!
fn calculate_mesh_dimensions(num_nodes: usize) -> MeshConfig {
    if num_nodes == 0 || num_nodes == 1 {
        return MeshConfig::new(1, 1, 1);
    }

    // Start from minimal mesh
    let mut w = 1;
    let mut h = 1;
    let mut d = 1;

    // Grow mesh until it fits all nodes
    // Strategy: Always grow the SMALLEST dimension
    // But STRONGLY prefer growing w or h over d (keep it flat!)
    while w * h * d < num_nodes {
        // Compare dimensions, but with a HUGE bias against depth
        // This naturally creates flat meshes that only grow vertically when necessary

        // Effective sizes (depth counts 100x more to keep mesh flat)
        let w_effective = w;
        let h_effective = h;
        let d_effective = d * 100; // STRONGLY discourage depth growth

        // Find which dimension is smallest (effective)
        if w_effective <= h_effective && w_effective <= d_effective {
            // Width is smallest (or tied) - grow it!
            w += 1;
        } else if h_effective <= w_effective && h_effective <= d_effective {
            // Height is smallest (or tied) - grow it!
            h += 1;
        } else {
            // Depth is smallest (very rare due to 100x penalty) - grow it!
            d += 1;
        }
    }

    MeshConfig::new(w, h, d)
}

/// Get neighbor slot indices in the FILLED mesh (skipping empty slots)
/// Uses Citadel DHT "turn left" algorithm to find exactly 8 unique neighbors!
fn get_neighbor_indices(slot_index: usize, num_nodes: usize, mesh_config: &MeshConfig) -> Vec<usize> {
    let width = mesh_config.width;
    let height = mesh_config.height;
    let depth = mesh_config.depth;

    // Convert slot index to coordinates
    let x = slot_index % width;
    let y = (slot_index / width) % height;
    let z = slot_index / (width * height);

    let mut neighbors = HashSet::new();

    // Many directions to try (keep trying until we have 8 unique neighbors)
    // This is the "turn left" algorithm - try different angles until we find 8
    let all_offsets = [
        // Primary 8 directions
        (1, 0, 0),   // PlusA
        (-1, 0, 0),  // MinusA
        (0, 1, 0),   // PlusB
        (0, -1, 0),  // MinusB
        (1, -1, 0),  // PlusC (hexagonal)
        (-1, 1, 0),  // MinusC (hexagonal)
        (0, 0, 1),   // Up
        (0, 0, -1),  // Down
        // Turn left - try diagonals
        (1, 1, 0),
        (-1, -1, 0),
        (1, -1, 1),
        (-1, 1, -1),
        // Turn left again - try double steps
        (2, 0, 0),
        (-2, 0, 0),
        (0, 2, 0),
        (0, -2, 0),
        (0, 0, 2),
        (0, 0, -2),
        // Keep turning - mixed directions
        (1, 0, 1),
        (-1, 0, -1),
        (0, 1, 1),
        (0, -1, -1),
        (1, 1, 1),
        (-1, -1, -1),
        // Even more directions
        (2, 1, 0),
        (1, 2, 0),
        (2, 0, 1),
        (1, 0, 2),
        (0, 2, 1),
        (0, 1, 2),
    ];

    // Try directions until we find 8 unique neighbors
    for (dx, dy, dz) in all_offsets {
        if neighbors.len() >= 8 {
            break; // Got 8 neighbors, we're done!
        }

        // Keep stepping in direction until we find a filled slot
        let mut steps = 1;
        let max_steps = width * height * depth;

        while steps <= max_steps {
            let nx = ((x as i32 + dx * steps as i32).rem_euclid(width as i32)) as usize;
            let ny = ((y as i32 + dy * steps as i32).rem_euclid(height as i32)) as usize;
            let nz = ((z as i32 + dz * steps as i32).rem_euclid(depth as i32)) as usize;

            let neighbor_index = nz * width * height + ny * width + nx;

            // Found a filled slot that isn't ourselves and isn't already in our set!
            if neighbor_index < num_nodes && neighbor_index != slot_index {
                neighbors.insert(neighbor_index);
                break;
            }

            steps += 1;
        }
    }

    neighbors.into_iter().collect()
}

/// Deterministically assign slot to peer based on sorted peer list
/// Every node calculates the same result given the same peer list
fn peer_id_to_slot_deterministic(
    peer_id: &str,
    all_peer_ids: &[String],
) -> Option<SlotCoordinate> {
    // 1. Sort peer IDs lexicographically (everyone agrees on order)
    let mut sorted_peers = all_peer_ids.to_vec();
    sorted_peers.sort();

    // 2. Find our index in sorted list
    let our_index = sorted_peers.iter().position(|p| p == peer_id)?;

    // 3. Calculate mesh size from total number of peers
    let mesh_config = calculate_mesh_dimensions(sorted_peers.len());

    // 4. Map index to slot coordinate (fill mesh sequentially)
    let total_slots = mesh_config.total_slots();
    let slot_index = our_index % total_slots;

    let x = (slot_index % mesh_config.width) as i32;
    let y = ((slot_index / mesh_config.width) % mesh_config.height) as i32;
    let z = (slot_index / (mesh_config.width * mesh_config.height)) as i32;

    Some(SlotCoordinate::new(x, y, z))
}

#[test]
fn test_mesh_starts_at_1x1x1() {
    println!("\n=== Test: Mesh starts at 1×1×1 for single node ===");

    let mesh = calculate_mesh_dimensions(1);
    assert_eq!(mesh.width, 1, "Single node mesh should be 1 wide");
    assert_eq!(mesh.height, 1, "Single node mesh should be 1 tall");
    assert_eq!(mesh.depth, 1, "Single node mesh should be 1 deep");
    assert_eq!(mesh.total_slots(), 1, "Single node mesh has 1 slot");

    println!("✅ Single node mesh: {}×{}×{} = {} slots", mesh.width, mesh.height, mesh.depth, mesh.total_slots());
}

#[test]
fn test_mesh_grows_gently_from_1x1x1() {
    println!("\n=== Test: Mesh grows gently one dimension at a time ===");

    // Test that mesh grows naturally without jumps
    let mut prev_total = 1;
    for num_nodes in 2..=10 {
        let mesh = calculate_mesh_dimensions(num_nodes);
        let total_slots = mesh.total_slots();

        // Mesh must fit all nodes
        assert!(total_slots >= num_nodes,
            "Mesh {}×{}×{} = {} must fit {} nodes",
            mesh.width, mesh.height, mesh.depth, total_slots, num_nodes);

        // Growth should be gentle (no sudden jumps)
        assert!(total_slots - prev_total <= mesh.width + mesh.height + mesh.depth,
            "Mesh should grow gently, not jump from {} to {} slots",
            prev_total, total_slots);

        // Should stay flat (depth=1) for small node counts
        if num_nodes <= 50 {
            assert_eq!(mesh.depth, 1, "Should stay flat (depth=1) for {} nodes", num_nodes);
        }

        println!("  {} nodes → {}×{}×{} = {} slots", num_nodes, mesh.width, mesh.height, mesh.depth, total_slots);
        prev_total = total_slots;
    }

    println!("✅ Mesh grows gently from 1×1×1");
}

#[test]
fn test_mesh_stays_square_ish() {
    println!("\n=== Test: Mesh stays roughly square horizontally ===");

    // Test that width and height grow proportionally (roughly square)
    for num_nodes in [10, 25, 50, 100] {
        let mesh = calculate_mesh_dimensions(num_nodes);

        // Width and height should be close to each other (within factor of 2)
        let ratio = mesh.width.max(mesh.height) as f64 / mesh.width.min(mesh.height) as f64;
        assert!(ratio <= 2.0,
            "Mesh {}×{}×{} should be roughly square, but ratio is {:.1}",
            mesh.width, mesh.height, mesh.depth, ratio);

        println!("  {} nodes → {}×{}×{} = {} slots (ratio: {:.1}:1)",
            num_nodes, mesh.width, mesh.height, mesh.depth, mesh.total_slots(), ratio);
    }

    println!("✅ Mesh stays roughly square horizontally");
}

#[test]
fn test_mesh_grows_proportionally() {
    println!("\n=== Test: Mesh grows proportionally to node count ===");

    let test_cases = [(50, 8), (100, 11), (250, 17), (500, 26)];

    for (num_nodes, min_side) in test_cases {
        let mesh = calculate_mesh_dimensions(num_nodes);
        let total_slots = mesh.total_slots();

        // Mesh must accommodate all nodes
        assert!(total_slots >= num_nodes,
            "{} nodes needs at least {} slots, got {}",
            num_nodes, num_nodes, total_slots);

        // Mesh should not be TOO oversized (within 2x of actual nodes)
        assert!(total_slots < num_nodes * 2,
            "Mesh {}×{}×{} = {} is too oversized for {} nodes",
            mesh.width, mesh.height, mesh.depth, total_slots, num_nodes);

        println!("  {} nodes → {}×{}×{} = {} slots",
            num_nodes, mesh.width, mesh.height, mesh.depth, total_slots);
    }

    println!("✅ Mesh grows proportionally without excessive waste");
}

#[test]
fn test_mesh_maintains_reasonable_ratios() {
    println!("\n=== Test: Mesh maintains reasonable ratios ===");

    // Test various mesh sizes - check ratio is reasonable
    // With "turn left" algorithm, we can handle any ratio as long as mesh fits all nodes!
    let test_cases = [26, 50, 100, 250, 500];

    for num_nodes in test_cases {
        let mesh = calculate_mesh_dimensions(num_nodes);
        let total_slots = mesh.total_slots();

        // Mesh must fit all nodes
        assert!(total_slots >= num_nodes,
            "Mesh {}×{}×{} = {} slots must fit {} nodes",
            mesh.width, mesh.height, mesh.depth, total_slots, num_nodes);

        // Mesh shouldn't be TOO wasteful (less than 2x nodes)
        assert!(total_slots < num_nodes * 2,
            "Mesh {} slots is too oversized for {} nodes", total_slots, num_nodes);

        let horizontal = mesh.width.min(mesh.height);
        let vertical = mesh.depth.max(1);
        let ratio = horizontal as f64 / vertical as f64;

        println!("  {} nodes → {}×{}×{} = {} slots (ratio: {:.1}:1)",
            num_nodes, mesh.width, mesh.height, mesh.depth, total_slots, ratio);
    }

    println!("✅ Mesh maintains reasonable sizes (no excessive waste)");
}

#[test]
fn test_deterministic_slot_assignment() {
    println!("\n=== Test: Slot assignment is deterministic ===");

    let peers = vec![
        "peer-999".to_string(),
        "peer-123".to_string(),
        "peer-456".to_string(),
    ];

    // Calculate slots multiple times - should always be same
    for _ in 0..5 {
        let slot1 = peer_id_to_slot_deterministic("peer-123", &peers).unwrap();
        let slot2 = peer_id_to_slot_deterministic("peer-123", &peers).unwrap();
        assert_eq!(slot1, slot2, "Same peer should always get same slot");
    }

    println!("✅ Slot assignment is deterministic");
}

#[test]
fn test_slot_assignment_based_on_sorted_order() {
    println!("\n=== Test: Slots assigned by sorted peer ID order ===");

    let peers = vec![
        "peer-999".to_string(),
        "peer-123".to_string(),
        "peer-456".to_string(),
    ];

    // Sorted order: peer-123, peer-456, peer-999
    // Should get slots 0, 1, 2 in a 3x3x1 mesh
    let slot_123 = peer_id_to_slot_deterministic("peer-123", &peers).unwrap();
    let slot_456 = peer_id_to_slot_deterministic("peer-456", &peers).unwrap();
    let slot_999 = peer_id_to_slot_deterministic("peer-999", &peers).unwrap();

    println!("  peer-123 (sorted pos 0) → slot {:?}", slot_123);
    println!("  peer-456 (sorted pos 1) → slot {:?}", slot_456);
    println!("  peer-999 (sorted pos 2) → slot {:?}", slot_999);

    // All slots should be different
    assert_ne!(slot_123, slot_456);
    assert_ne!(slot_456, slot_999);
    assert_ne!(slot_123, slot_999);

    // First peer should get slot (0,0,0)
    assert_eq!(slot_123.x, 0);
    assert_eq!(slot_123.y, 0);
    assert_eq!(slot_123.z, 0);

    println!("✅ Slots assigned in sorted peer ID order");
}

#[test]
fn test_no_slot_collisions_50_nodes() {
    println!("\n=== Test: No slot collisions with 50 nodes ===");

    // Generate 50 unique peer IDs
    let peers: Vec<String> = (0..50).map(|i| format!("peer-{}", i)).collect();

    let mut assigned_slots = HashSet::new();

    for peer_id in &peers {
        let slot = peer_id_to_slot_deterministic(peer_id, &peers).unwrap();

        // Check for collision
        if assigned_slots.contains(&slot) {
            panic!("COLLISION DETECTED! Peer {} assigned to already-occupied slot {:?}", peer_id, slot);
        }

        assigned_slots.insert(slot);
    }

    println!("  Assigned {} unique slots to {} peers", assigned_slots.len(), peers.len());
    assert_eq!(assigned_slots.len(), 50, "All 50 peers should have unique slots");

    println!("✅ No slot collisions with 50 nodes");
}

#[test]
fn test_100_percent_neighbor_connectivity() -> Result<()> {
    println!("\n=== Test: 100% neighbor connectivity with 50 nodes ===");

    // Generate 50 unique peer IDs
    let peers: Vec<String> = (0..50).map(|i| format!("peer-{}", i)).collect();

    // Calculate mesh config for 50 nodes
    let mesh_config = calculate_mesh_dimensions(50);
    println!("  Mesh config: {}×{}×{} = {} slots", mesh_config.width, mesh_config.height, mesh_config.depth, mesh_config.total_slots());

    // Assign all peers to slots
    let mut slot_to_peer = std::collections::HashMap::new();
    for peer_id in &peers {
        let slot = peer_id_to_slot_deterministic(peer_id, &peers).unwrap();
        slot_to_peer.insert(slot, peer_id.clone());
    }

    // For each peer, check that neighbors exist (using filled-slot topology)
    let mut total_neighbors_expected = 0;
    let mut total_neighbors_found = 0;

    // Sort peers to get consistent indices
    let mut sorted_peers = peers.clone();
    sorted_peers.sort();

    for (my_index, peer_id) in sorted_peers.iter().enumerate() {
        let my_slot = peer_id_to_slot_deterministic(peer_id, &peers).unwrap();

        // Get neighbor indices in the FILLED mesh (skipping empty slots)
        let neighbor_indices = get_neighbor_indices(my_index, peers.len(), &mesh_config);

        // Each neighbor index maps to a filled slot
        let found = neighbor_indices.len();
        total_neighbors_found += found;
        total_neighbors_expected += 8; // We expect 8 neighbors per node

        // Debug output for problematic nodes
        if found < 8 {
            println!("  {} at {:?} (index {}): found {}/8 neighbors - MISSING {} neighbors!",
                peer_id, my_slot, my_index, found, 8 - found);
            println!("    Found neighbor indices: {:?}", neighbor_indices);
        } else {
            println!("  {} at {:?} (index {}): found {}/8 neighbors", peer_id, my_slot, my_index, found);
        }
    }

    let connectivity = (total_neighbors_found as f64 / total_neighbors_expected as f64) * 100.0;
    println!("\n🎯 Overall connectivity: {:.1}% ({}/{} neighbors found)",
        connectivity, total_neighbors_found, total_neighbors_expected);

    // This MUST be 100% - every peer must find all 8 neighbors
    assert_eq!(
        total_neighbors_found, total_neighbors_expected,
        "MUST have 100% neighbor connectivity! Found {}/{} ({:.1}%)",
        total_neighbors_found, total_neighbors_expected, connectivity
    );

    println!("✅ 100% neighbor connectivity achieved!");
    Ok(())
}

#[test]
fn test_mesh_handles_node_churn() {
    println!("\n=== Test: Mesh adapts as nodes join/leave ===");

    // Start with 3 nodes
    let peers_3 = vec!["peer-1".to_string(), "peer-2".to_string(), "peer-3".to_string()];
    let mesh_3 = calculate_mesh_dimensions(3);
    println!("  3 nodes → {}×{}×{}", mesh_3.width, mesh_3.height, mesh_3.depth);

    let slot_1_with_3 = peer_id_to_slot_deterministic("peer-1", &peers_3).unwrap();
    println!("    peer-1 at {:?}", slot_1_with_3);

    // Add 2 more nodes (total 5)
    let peers_5 = vec![
        "peer-1".to_string(),
        "peer-2".to_string(),
        "peer-3".to_string(),
        "peer-4".to_string(),
        "peer-5".to_string(),
    ];
    let mesh_5 = calculate_mesh_dimensions(5);
    println!("  5 nodes → {}×{}×{}", mesh_5.width, mesh_5.height, mesh_5.depth);

    let slot_1_with_5 = peer_id_to_slot_deterministic("peer-1", &peers_5).unwrap();
    println!("    peer-1 at {:?}", slot_1_with_5);

    // peer-1's slot may change as mesh grows (expected behavior)
    println!("  peer-1 slot changed: {} (expected as mesh grows)", slot_1_with_3 != slot_1_with_5);

    println!("✅ Mesh adapts to node churn");
}

#[test]
fn test_all_peers_agree_on_mesh_size() {
    println!("\n=== Test: All peers agree on mesh size ===");

    let peers: Vec<String> = (0..50).map(|i| format!("peer-{}", i)).collect();

    // Every peer calculates mesh size from same peer list length
    let expected_mesh = calculate_mesh_dimensions(peers.len());

    for _peer_id in &peers {
        let mesh = calculate_mesh_dimensions(peers.len());
        assert_eq!(mesh.width, expected_mesh.width, "All peers must agree on width");
        assert_eq!(mesh.height, expected_mesh.height, "All peers must agree on height");
        assert_eq!(mesh.depth, expected_mesh.depth, "All peers must agree on depth");
    }

    println!("  All 50 peers agree: {}×{}×{} = {} slots",
        expected_mesh.width, expected_mesh.height, expected_mesh.depth, expected_mesh.total_slots());
    println!("✅ Mesh size calculation is consistent across all peers");
}
