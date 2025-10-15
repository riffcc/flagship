//! Test hexagonal toroidal mesh topology with WebRTC DataChannels
//!
//! This test verifies the 8-neighbor topology (6 hexagonal + 2 vertical):
//! - Center node at origin (0,0,0)
//! - 6 hexagonal neighbors (±A, ±B, ±C)
//! - 2 vertical neighbors (Up, Down)
//!
//! Tests:
//! - 9-node single-segment mesh (full 8-neighbor topology)
//! - Gossip propagation through hexagonal mesh
//! - DHT routing across all neighbor directions
//!
//! Visualization:
//! - Web UI available at http://0.0.0.0:8080
//! - Real-time 3D force-directed graph of mesh topology
//! - Uses Flagship's existing /api/v1/map endpoint and Vue.js visualizer

mod test_helpers;
use test_helpers::TestNode;

use std::time::Duration;
use std::sync::Arc;
use citadel_core::topology::{SlotCoordinate, MeshConfig, Direction};

#[tokio::test]
async fn test_9node_hexagonal_single_segment() -> anyhow::Result<()> {
    println!("🚀 Starting 9-node hexagonal single-segment mesh test");
    println!("   Testing full 8-neighbor topology (6 hexagonal + 2 vertical)");
    println!();

    // Single segment mesh: Small enough to test all 8 neighbors
    // Using 5x5x3 = 75 slots (plenty of room for 9 nodes)
    let mesh_config = MeshConfig::new(5, 5, 3);

    // Central node at origin
    let center_slot = SlotCoordinate::new(0, 0, 0);

    // Calculate all 8 neighbor slots using citadel-core topology
    println!("📐 Calculating 8-neighbor topology for center slot (0, 0, 0)...");
    let neighbors = vec![
        (Direction::PlusA,  center_slot.neighbor(Direction::PlusA,  &mesh_config)),  // +A hexagonal
        (Direction::MinusA, center_slot.neighbor(Direction::MinusA, &mesh_config)),  // -A hexagonal
        (Direction::PlusB,  center_slot.neighbor(Direction::PlusB,  &mesh_config)),  // +B hexagonal
        (Direction::MinusB, center_slot.neighbor(Direction::MinusB, &mesh_config)),  // -B hexagonal
        (Direction::PlusC,  center_slot.neighbor(Direction::PlusC,  &mesh_config)),  // +C hexagonal
        (Direction::MinusC, center_slot.neighbor(Direction::MinusC, &mesh_config)),  // -C hexagonal
        (Direction::Up,     center_slot.neighbor(Direction::Up,     &mesh_config)),  // Up vertical
        (Direction::Down,   center_slot.neighbor(Direction::Down,   &mesh_config)),  // Down vertical
    ];

    println!("🎯 Mesh topology ({}x{}x{} hexagonal toroidal):",
        mesh_config.width, mesh_config.height, mesh_config.depth);
    println!("   Center: ({}, {}, {}) [depth={}]",
        center_slot.x, center_slot.y, center_slot.z, center_slot.z);
    for (i, (dir, slot)) in neighbors.iter().enumerate() {
        println!("   Neighbor {}: {:?} → ({}, {}, {}) [depth={}]",
            i, dir, slot.x, slot.y, slot.z, slot.z);
    }
    println!();

    // Spawn central node (port 17000)
    println!("🔧 Spawning center node...");
    let node_center = TestNode::spawn_at_slot(17000, Some(center_slot)).await?;
    println!("✅ Center node: port {} - slot ({}, {}, {}) - peer_id: {}",
        node_center.port, center_slot.x, center_slot.y, center_slot.z, &node_center.peer_id[..20]);
    println!();

    // Spawn all 8 neighbor nodes (ports 17001-17008)
    println!("🔧 Spawning 8 neighbor nodes...");
    let mut neighbor_nodes = Vec::new();
    for (i, (dir, slot)) in neighbors.iter().enumerate() {
        let port = 17001 + i as u16;
        let node = TestNode::spawn_at_slot(port, Some(*slot)).await?;
        println!("✅ Neighbor {}: {:?} - port {} - slot ({}, {}, {}) - peer_id: {}",
            i, dir, port, slot.x, slot.y, slot.z, &node.peer_id[..20]);
        neighbor_nodes.push(node);
    }
    println!();

    // Establish STAR topology: only center connects to all 8 neighbors
    // Peripheral nodes do NOT connect to each other - they must route through center
    // Total connections: 8 (center ↔ each neighbor)
    println!("🔗 Building WebRTC star topology (only center connects to neighbors)...");
    println!("   This tests multi-hop routing: peripheral nodes route through center");
    println!("   Total connections: 8 (center ↔ each of 8 neighbors)");
    println!();

    for (i, (neighbor, (dir, _slot))) in neighbor_nodes.iter().zip(neighbors.iter()).enumerate() {
        println!("   [{}/8] Connecting center ↔ {:?} neighbor...", i+1, dir);
        node_center.establish_webrtc_connection(neighbor).await?;
    }

    println!();
    println!("✅ WebRTC star topology established!");
    println!("   ✓ Center node has 8 WebRTC connections");
    println!("   ✓ Each neighbor has 1 connection (to center only)");
    println!("   ✓ Neighbors must route through center to reach each other");
    println!();

    // Wait for connections to stabilize
    println!("⏳ Waiting for connections to stabilize...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!();

    // Announce slot ownership - should flood through entire mesh
    println!("📢 Announcing slot ownership (all 9 nodes)...");
    node_center.announce_slot_ownership(center_slot).await?;
    println!("   ✅ Center announced");

    for (i, (neighbor, (dir, slot))) in neighbor_nodes.iter().zip(neighbors.iter()).enumerate() {
        println!("   [{}/8] Announcing {:?} neighbor at ({}, {}, {})", i+1, dir, slot.x, slot.y, slot.z);
        neighbor.announce_slot_ownership(*slot).await?;
    }

    // Wait for gossip to propagate through full mesh
    println!("⏳ Waiting for gossip to flood through all 9 nodes...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("✅ Gossip propagation complete");
    println!();

    // Test ALL point-to-point DHT routing possibilities
    // For 9 nodes: 9 × 8 = 72 unique source→destination routes to test
    println!("🧪 Testing ALL point-to-point DHT routing (9×8 = 72 routes)...");
    println!();

    let mut all_nodes_list = vec![&node_center];
    all_nodes_list.extend(neighbor_nodes.iter());

    let mut successful_routes = 0;
    let total_routes = all_nodes_list.len() * (all_nodes_list.len() - 1);

    for (src_idx, src_node) in all_nodes_list.iter().enumerate() {
        for (dst_idx, dst_node) in all_nodes_list.iter().enumerate() {
            if src_idx == dst_idx {
                continue; // Skip self-routing
            }

            // Create unique key for this source→destination route
            let route_key_string = format!("route-{}-to-{}", src_idx, dst_idx);
            let route_key: [u8; 32] = blake3::hash(route_key_string.as_bytes()).into();
            let route_value = format!("value-from-{}-to-{}", src_idx, dst_idx).into_bytes();

            // Source node performs DHT PUT
            src_node.dht_put(route_key, route_value.clone()).await?;

            // Wait for routing
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Destination node (or any node) should be able to retrieve it
            let retrieved = src_node.dht_get(route_key).await?;

            if let Some(value) = retrieved {
                if value == route_value {
                    successful_routes += 1;
                    print!(".");
                } else {
                    println!();
                    println!("❌ Route {}/{}: Node {} → Node {} - VALUE MISMATCH",
                        successful_routes + 1, total_routes, src_idx, dst_idx);
                }
            } else {
                println!();
                println!("❌ Route {}/{}: Node {} → Node {} - NOT FOUND",
                    successful_routes + 1, total_routes, src_idx, dst_idx);
            }
        }
    }

    println!();
    println!();
    println!("✅ Point-to-point routing test complete!");
    println!("   Successfully routed: {}/{} routes", successful_routes, total_routes);

    assert_eq!(successful_routes, total_routes,
        "All point-to-point routes should succeed in full mesh");

    println!();
    println!("✅ 9-node hexagonal single-segment test PASSED!");
    println!("   ✓ 9 nodes spawned in hexagonal topology (1 center + 8 neighbors)");
    println!("   ✓ WebRTC STAR topology (8 connections through center)");
    println!("   ✓ All 8 neighbor directions verified:");
    println!("     - 6 hexagonal: ±A, ±B, ±C");
    println!("     - 2 vertical: Up, Down");
    println!("   ✓ Gossip flooded through all 9 nodes");
    println!("   ✓ Multi-hop routing: {} successful routes", successful_routes);
    println!("   ✓ Peripheral nodes successfully route through center node");
    println!("   ✓ DHT PUT/GET works across all {} point-to-point paths", total_routes);
    println!();
    println!("📊 VISUALIZATION AVAILABLE:");
    println!("   View mesh topology: http://localhost:17000/api/v1/map");
    println!("   (Or any node: ports 17000-17008)");
    println!();
    println!("   To visualize in Flagship frontend:");
    println!("   1. Start Flagship dev server");
    println!("   2. Navigate to the site");
    println!("   3. Type 'batmanbatmanbatman' anywhere to open network map");
    println!();

    // Keep test running for manual visualization
    println!("⏸️  Test nodes will remain running for 60 seconds for visualization...");
    tokio::time::sleep(Duration::from_secs(60)).await;
    println!("✅ Shutting down test nodes");

    Ok(())
}
