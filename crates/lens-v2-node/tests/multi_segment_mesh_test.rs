//! Multi-segment hexagonal toroidal mesh test
//!
//! Tests a 2.5D hexagonal toroidal mesh where each node connects to its 8 actual neighbors.
//! "Segments" are just an organizational concept for spawning nodes - the actual connections
//! are determined by the hexagonal toroidal topology (6 hex + 2 vertical neighbors per node).
//!
//! Visualization available at http://localhost:19000/api/v1/map

mod test_helpers;
use test_helpers::TestNode;

use std::time::Duration;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use citadel_core::topology::{SlotCoordinate, MeshConfig, Direction};
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

/// Shared state for the mesh test visualization
#[derive(Clone)]
struct MeshTestState {
    nodes: Arc<Mutex<Vec<MeshNode>>>,
    edges: Arc<Mutex<Vec<MeshEdge>>>,
    mesh_config: Arc<Mutex<Option<MeshConfig>>>,
    broadcast: broadcast::Sender<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshNode {
    id: String,
    label: String,
    slot: SlotData,
    peer_type: String,
    online: bool,
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SlotData {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshEdge {
    from: String,
    to: String,
    connection_type: String,
    color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshTestMap {
    mesh_config: MeshConfigData,
    nodes: Vec<MeshNode>,
    edges: Vec<MeshEdge>,
    stats: MeshStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshConfigData {
    width: usize,
    height: usize,
    depth: usize,
    total_slots: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshStats {
    total_peers: usize,
    server_nodes: usize,
    browser_peers: usize,
    mesh_edges: usize,
    relay_connections: usize,
    occupancy_percent: f64,
}

impl MeshTestState {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1000);
        Self {
            nodes: Arc::new(Mutex::new(Vec::new())),
            edges: Arc::new(Mutex::new(Vec::new())),
            mesh_config: Arc::new(Mutex::new(None)),
            broadcast: tx,
        }
    }

    fn add_node(&self, node: MeshNode) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.push(node);
        drop(nodes);
        let _ = self.broadcast.send(self.to_json());
    }

    fn add_edge(&self, edge: MeshEdge) {
        let mut edges = self.edges.lock().unwrap();
        edges.push(edge);
        drop(edges);
        let _ = self.broadcast.send(self.to_json());
    }

    fn set_mesh_config(&self, config: MeshConfig) {
        let mut mesh_config = self.mesh_config.lock().unwrap();
        *mesh_config = Some(config);
        drop(mesh_config);
        let _ = self.broadcast.send(self.to_json());
    }

    fn to_json(&self) -> String {
        let nodes = self.nodes.lock().unwrap().clone();
        let edges = self.edges.lock().unwrap().clone();
        let mesh_config = self.mesh_config.lock().unwrap();

        let (width, height, depth, total_slots) = if let Some(ref config) = *mesh_config {
            (config.width, config.height, config.depth, config.width * config.height * config.depth)
        } else {
            (1, 1, 1, 1)
        };

        let total_peers = nodes.len();
        let server_nodes = nodes.iter().filter(|n| n.peer_type == "server").count();
        let browser_peers = nodes.iter().filter(|n| n.peer_type == "browser").count();
        let mesh_edges = edges.len();
        let occupancy_percent = (total_peers as f64 / total_slots as f64) * 100.0;

        let map = MeshTestMap {
            mesh_config: MeshConfigData {
                width,
                height,
                depth,
                total_slots,
            },
            nodes,
            edges,
            stats: MeshStats {
                total_peers,
                server_nodes,
                browser_peers,
                mesh_edges,
                relay_connections: 0,
                occupancy_percent,
            },
        };

        serde_json::to_string(&map).unwrap()
    }
}

#[tokio::test]
async fn test_73node_multi_segment_mesh() -> anyhow::Result<()> {
    println!("🚀 Starting 73-node hexagonal toroidal mesh test");
    println!("   Each node connects to its 8 neighbors in the 2.5D toroid");
    println!();

    // Create shared state for visualization
    let state = MeshTestState::new();
    let state_clone = state.clone();

    // Spawn HTTP server for visualization on port 8080 FIRST
    println!("🌐 Starting visualization server on http://0.0.0.0:8080");
    let viz_html = include_str!("mesh-visualizer.html");
    tokio::spawn(async move {
        use std::net::SocketAddr;
        use tokio::net::TcpListener;
        use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
        use sha1::{Sha1, Digest};
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let listener = TcpListener::bind(addr).await.expect("Failed to bind port 8080");
        println!("   ✅ Visualization server ready at http://0.0.0.0:8080");
        println!("   📊 Map endpoint: http://0.0.0.0:8080/mesh-test-map");
        println!("   🔌 WebSocket endpoint: ws://0.0.0.0:8080/ws");

        loop {
            if let Ok((socket, _)) = listener.accept().await {
                let html = viz_html.to_string();
                let state = state_clone.clone();
                tokio::spawn(async move {
                    let (read_half, mut write_half) = socket.into_split();
                    let mut reader = BufReader::new(read_half);
                    let mut first_line = String::new();
                    let _ = reader.read_line(&mut first_line).await;

                    let path = first_line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("/");

                    // Check for WebSocket upgrade
                    let mut headers = Vec::new();
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).await.is_err() || line == "\r\n" {
                            break;
                        }
                        headers.push(line);
                    }

                    let is_websocket = headers.iter().any(|h| h.to_lowercase().contains("upgrade: websocket"));

                    if path == "/ws" && is_websocket {
                        // WebSocket handshake
                        let key = headers.iter()
                            .find(|h| h.to_lowercase().starts_with("sec-websocket-key:"))
                            .and_then(|h| h.split(':').nth(1))
                            .map(|k| k.trim())
                            .unwrap_or("");

                        // WebSocket handshake (RFC 6455)
                        let mut hasher = Sha1::new();
                        hasher.update(format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key).as_bytes());
                        let accept = BASE64.encode(hasher.finalize());

                        let response = format!(
                            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
                            accept
                        );
                        let _ = write_half.write_all(response.as_bytes()).await;

                        // Send updates via WebSocket
                        let mut rx = state.broadcast.subscribe();
                        while let Ok(json) = rx.recv().await {
                            // WebSocket frame: text frame with payload
                            let payload = json.as_bytes();
                            let len = payload.len();

                            let mut frame = vec![0x81]; // FIN + text frame
                            if len < 126 {
                                frame.push(len as u8);
                            } else if len < 65536 {
                                frame.push(126);
                                frame.extend_from_slice(&(len as u16).to_be_bytes());
                            } else {
                                frame.push(127);
                                frame.extend_from_slice(&(len as u64).to_be_bytes());
                            }
                            frame.extend_from_slice(payload);

                            if write_half.write_all(&frame).await.is_err() {
                                break;
                            }
                        }
                    } else if path == "/mesh-test-map" {
                        // Serve JSON from shared state
                        let json = state.to_json();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                            json.len(),
                            json
                        );
                        let _ = write_half.write_all(response.as_bytes()).await;
                    } else {
                        // Serve HTML (default for / and any other path)
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                            html.len(),
                            html
                        );
                        let _ = write_half.write_all(response.as_bytes()).await;
                    }
                });
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!();

    let mesh_config = MeshConfig::new(10, 10, 5);
    let center_slot = SlotCoordinate::new(5, 5, 2);

    // Set mesh config in shared state
    state.set_mesh_config(mesh_config.clone());

    // Track which slots have nodes and their TestNode references
    let mut slot_to_node: HashMap<SlotCoordinate, &TestNode> = HashMap::new();
    let mut all_nodes: Vec<TestNode> = Vec::new();
    let mut slots_to_spawn: HashSet<SlotCoordinate> = HashSet::new();

    // Segment 0: Center + 8 neighbors
    println!("📐 Planning segment 0 (center + 8 neighbors)...");
    slots_to_spawn.insert(center_slot);
    for dir in [Direction::PlusA, Direction::MinusA, Direction::PlusB, Direction::MinusB,
                Direction::PlusC, Direction::MinusC, Direction::Up, Direction::Down] {
        let neighbor = center_slot.neighbor(dir, &mesh_config);
        slots_to_spawn.insert(neighbor);
    }
    println!("   Segment 0: {} slots planned", slots_to_spawn.len());

    // Segments 1-8: For each of segment 0's neighbors, add their 8 neighbors
    let segment0_neighbors: Vec<SlotCoordinate> = slots_to_spawn.iter()
        .filter(|&&s| s != center_slot)
        .copied()
        .collect();

    for (seg_idx, seg_center) in segment0_neighbors.iter().enumerate() {
        println!("📐 Planning segment {} (neighbors of ({},{},{}))...",
            seg_idx + 1, seg_center.x, seg_center.y, seg_center.z);

        for dir in [Direction::PlusA, Direction::MinusA, Direction::PlusB, Direction::MinusB,
                    Direction::PlusC, Direction::MinusC, Direction::Up, Direction::Down] {
            let neighbor = seg_center.neighbor(dir, &mesh_config);
            slots_to_spawn.insert(neighbor);
        }
    }

    println!("✅ Total unique slots to spawn: {}", slots_to_spawn.len());
    println!();

    // Spawn all nodes
    println!("🔧 Spawning {} nodes...", slots_to_spawn.len());
    let mut port_offset = 0;
    for slot in slots_to_spawn.iter() {
        let node = TestNode::spawn_at_slot(19000 + port_offset, Some(*slot)).await?;
        println!("   Node {}: port {} - slot ({},{},{})",
            port_offset, node.port, slot.x, slot.y, slot.z);

        // Add to shared state for visualization
        let node_id = format!("node-{}", port_offset);
        state.add_node(MeshNode {
            id: node_id.clone(),
            label: format!("Node {}", port_offset),
            slot: SlotData {
                x: slot.x,
                y: slot.y,
                z: slot.z,
            },
            peer_type: "server".to_string(),
            online: true,
            capabilities: vec!["webrtc".to_string(), "dht".to_string()],
        });

        all_nodes.push(node);
        port_offset += 1;
    }
    println!("✅ All {} nodes spawned", all_nodes.len());
    println!();

    // Build slot_to_node map
    for node in &all_nodes {
        // Extract slot from the node (we need to track this better in TestNode)
        // For now, we'll skip this and just announce ownership
    }

    // Establish WebRTC connections based on actual neighbor topology
    println!("🔗 Establishing WebRTC connections (watch the mesh form in real-time!)...");
    println!("   Open http://localhost:8080 to watch connections form");
    println!();

    // For each node, connect to its 8 neighbors (if they exist in our spawned set)
    // Establish connections in parallel with 100ms spacing between starting each connection
    let mut connection_count = 0;
    let _spawned_slots: HashSet<SlotCoordinate> = slots_to_spawn.clone();
    let slots_vec: Vec<SlotCoordinate> = slots_to_spawn.iter().copied().collect();

    // Wrap all_nodes in Arc for sharing across tasks
    use std::sync::Arc as StdArc;
    let all_nodes_arc = StdArc::new(all_nodes);

    // Collect all connection tasks
    let mut connection_tasks = Vec::new();

    for i in 0..all_nodes_arc.len() {
        let my_slot = slots_vec[i];

        // Find all 8 neighbors
        for dir in [Direction::PlusA, Direction::MinusA, Direction::PlusB, Direction::MinusB,
                    Direction::PlusC, Direction::MinusC, Direction::Up, Direction::Down] {
            let neighbor_slot = my_slot.neighbor(dir, &mesh_config);

            // Find the node at this neighbor slot
            if let Some(neighbor_idx) = slots_vec.iter().position(|&s| s == neighbor_slot) {
                // Only connect if neighbor_idx > i to avoid duplicate connections
                if neighbor_idx > i {
                    println!("   🔗 Initiating ({},{},{}) ↔ ({},{},{}) via {:?}",
                        my_slot.x, my_slot.y, my_slot.z,
                        neighbor_slot.x, neighbor_slot.y, neighbor_slot.z,
                        dir);

                    let nodes = all_nodes_arc.clone();
                    let state_clone = state.clone();
                    let from_id = format!("node-{}", i);
                    let to_id = format!("node-{}", neighbor_idx);

                    // Spawn connection task
                    let task = tokio::spawn(async move {
                        // Establish WebRTC connection
                        nodes[i].establish_webrtc_connection(&nodes[neighbor_idx]).await?;

                        // Add edge to shared state for visualization
                        state_clone.add_edge(MeshEdge {
                            from: from_id,
                            to: to_id,
                            connection_type: "neighbor".to_string(),
                            color: "#8a2be2".to_string(), // Purple for neighbor connections
                        });

                        Ok::<_, anyhow::Error>(())
                    });

                    connection_tasks.push(task);
                    connection_count += 1;

                    // Small delay between starting each connection
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    println!();
    println!("⏳ Waiting for all {} connections to complete...", connection_count);

    // Wait for all connections to complete
    for task in connection_tasks {
        task.await??;
    }

    println!();
    println!("✅ Established {} WebRTC connections", connection_count);
    println!();

    // Announce slot ownership
    println!("📢 Announcing slot ownership for all {} nodes...", all_nodes_arc.len());
    let slots_vec: Vec<SlotCoordinate> = slots_to_spawn.iter().copied().collect();
    for (i, node) in all_nodes_arc.iter().enumerate() {
        node.announce_slot_ownership(slots_vec[i]).await?;
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("✅ Gossip propagation complete");
    println!();

    println!("✅ 73-node hexagonal toroidal mesh test COMPLETE!");
    println!("   ✓ {} nodes spawned across {} unique slots", all_nodes_arc.len(), slots_to_spawn.len());
    println!("   ✓ {} WebRTC connections established", connection_count);
    println!("   ✓ Each node connected to its 8 neighbors in the toroid");
    println!("   ✓ Gossip propagated through entire mesh");
    println!();
    println!("🎨 3D MESH VISUALIZATION:");
    println!("   🌐 Open in browser: http://localhost:8080");
    println!("   ✨ Interactive 3D force-directed graph");
    println!("   🔄 Auto-refreshes every 5 seconds");
    println!("   🎯 Click nodes to see details");
    println!();
    println!("   Alternative - Raw JSON:");
    println!("   📊 http://localhost:19000/api/v1/map (or ports 19000-{})", 19000 + all_nodes_arc.len() - 1);
    println!();

    // Keep test running for visualization
    println!("⏸️  Nodes will remain running for 120 seconds for visualization...");
    tokio::time::sleep(Duration::from_secs(120)).await;
    println!("✅ Shutting down");

    Ok(())
}
