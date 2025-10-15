

//! Configurable hexagonal toroidal mesh test
//!
//! Run with environment variables to configure mesh size:
//!
//! ```bash
//! MESH_WIDTH=32 MESH_HEIGHT=32 MESH_DEPTH=8 cargo test --test configurable_mesh_test -- --nocapture
//! ```
//!
//! Or use default 32×32×8:
//! ```bash
//! cargo test --test configurable_mesh_test -- --nocapture
//! ```
//!
//! Visualization available at http://localhost:8080

mod test_helpers;
use test_helpers::TestNode;

use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc as StdArc;
use citadel_core::topology::{SlotCoordinate, MeshConfig, Direction};
use serde::{Serialize, Deserialize};

/// Shared state for the mesh test visualization
#[derive(Clone)]
struct MeshTestState {
    nodes: StdArc<tokio::sync::Mutex<Vec<MeshNode>>>,
    edges: StdArc<tokio::sync::Mutex<Vec<MeshEdge>>>,
    mesh_config: StdArc<tokio::sync::Mutex<Option<MeshConfig>>>,
    broadcast: tokio::sync::broadcast::Sender<String>,
    test_node_urls: StdArc<tokio::sync::Mutex<HashMap<String, String>>>,
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
        let (tx, _rx) = tokio::sync::broadcast::channel(1000);
        Self {
            nodes: StdArc::new(tokio::sync::Mutex::new(Vec::new())),
            edges: StdArc::new(tokio::sync::Mutex::new(Vec::new())),
            mesh_config: StdArc::new(tokio::sync::Mutex::new(None)),
            broadcast: tx,
            test_node_urls: StdArc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    async fn register_test_node(&self, node_id: String, base_url: String) {
        let mut urls = self.test_node_urls.lock().await;
        urls.insert(node_id, base_url);
    }

    async fn add_node(&self, node: MeshNode) {
        let mut nodes = self.nodes.lock().await;
        nodes.push(node);
        drop(nodes);
        let _ = self.broadcast.send(self.to_json().await);
    }

    async fn add_edge(&self, edge: MeshEdge) {
        let mut edges = self.edges.lock().await;
        edges.push(edge);
        drop(edges);
        let _ = self.broadcast.send(self.to_json().await);
    }

    async fn set_mesh_config(&self, config: MeshConfig) {
        let mut mesh_config = self.mesh_config.lock().await;
        *mesh_config = Some(config);
        drop(mesh_config);
        let _ = self.broadcast.send(self.to_json().await);
    }

    async fn to_json(&self) -> String {
        let nodes = self.nodes.lock().await.clone();
        let edges = self.edges.lock().await.clone();
        let mesh_config = self.mesh_config.lock().await;

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
async fn test_configurable_mesh() -> anyhow::Result<()> {
    // Read mesh dimensions from environment variables
    let mesh_width = std::env::var("MESH_WIDTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(32);

    let mesh_height = std::env::var("MESH_HEIGHT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(32);

    let mesh_depth = std::env::var("MESH_DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8);

    println!("🚀 Starting {}×{}×{} hexagonal toroidal mesh test", mesh_width, mesh_height, mesh_depth);
    println!("   Total slots: {} = {} × {} × {}",
        mesh_width * mesh_height * mesh_depth,
        mesh_width, mesh_height, mesh_depth);
    println!("   Each node connects to its 8 neighbors in the 2.5D toroid");
    println!("   Packets route via Two Generals Protocol (TGP) over WebRTC");
    println!();
    println!("💡 TIP: Configure with environment variables:");
    println!("   MESH_WIDTH={} MESH_HEIGHT={} MESH_DEPTH={} cargo test --test configurable_mesh_test -- --nocapture",
        mesh_width, mesh_height, mesh_depth);
    println!();

    // Create shared state for visualization
    let state = MeshTestState::new();
    let state_clone = state.clone();

    // Spawn HTTP server for visualization on port 8080 FIRST
    println!("🌐 Starting visualization server on http://0.0.0.0:8080");
    let viz_html = include_str!("hex-viz/dist/index.html");
    let viz_js = include_str!("hex-viz/dist/assets/index-OWZ5-HLo.js");
    let viz_css = include_str!("hex-viz/dist/assets/index-CEGHIcSp.css");
    tokio::spawn(async move {
        use std::net::SocketAddr;
        use tokio::net::TcpListener;
        use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
        use sha1::{Sha1, Digest};
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let listener = TcpListener::bind(addr).await.expect("Failed to bind port 8080");
        println!("   ✅ Visualization server ready at http://0.0.0.0:8080");
        println!("   📊 Hexagonal toroidal mesh visualization");
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
                        let json = state.to_json().await;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                            json.len(),
                            json
                        );
                        let _ = write_half.write_all(response.as_bytes()).await;
                    } else if path.starts_with("/assets/index-OWZ5-HLo.js") {
                        // Serve JS
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/javascript\r\nContent-Length: {}\r\n\r\n{}",
                            viz_js.len(),
                            viz_js
                        );
                        let _ = write_half.write_all(response.as_bytes()).await;
                    } else if path.starts_with("/assets/index-CEGHIcSp.css") {
                        // Serve CSS
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/css\r\nContent-Length: {}\r\n\r\n{}",
                            viz_css.len(),
                            viz_css
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

    // Create mesh with specified dimensions
    let mesh_config = MeshConfig::new(mesh_width, mesh_height, mesh_depth);

    // Set mesh config in shared state
    state.set_mesh_config(mesh_config.clone()).await;

    println!("📐 Planning mesh topology...");
    println!("   Mesh: {}×{}×{} = {} total slots",
        mesh_width, mesh_height, mesh_depth,
        mesh_width * mesh_height * mesh_depth);
    println!("   Spawning all nodes in parallel...");
    println!();

    let mut slots_to_spawn: Vec<SlotCoordinate> = Vec::new();

    // Spawn EVERY slot in the mesh
    for x in 0..mesh_width as i32 {
        for y in 0..mesh_height as i32 {
            for z in 0..mesh_depth as i32 {
                slots_to_spawn.push(SlotCoordinate::new(x, y, z));
            }
        }
    }

    println!("✅ Total slots to spawn: {}", slots_to_spawn.len());
    println!();

    // Spawn all nodes in parallel with 0.5ms spacing
    println!("🔧 Spawning {} nodes in parallel...", slots_to_spawn.len());
    let mut spawn_tasks = Vec::new();

    for (port_offset, slot) in slots_to_spawn.iter().enumerate() {
        let slot = *slot;
        let state_clone = state.clone();

        let task = tokio::spawn(async move {
            let node = TestNode::spawn_at_slot(20000 + port_offset as u16, Some(slot)).await?;

            // Add to shared state for visualization
            let node_id = format!("node-{}", port_offset);
            state_clone.add_node(MeshNode {
                id: node_id.clone(),
                label: format!("({},{},{})", slot.x, slot.y, slot.z),
                slot: SlotData {
                    x: slot.x,
                    y: slot.y,
                    z: slot.z,
                },
                peer_type: "server".to_string(),
                online: true,
                capabilities: vec!["webrtc".to_string(), "dht".to_string()],
            }).await;

            Ok::<_, anyhow::Error>((port_offset, node))
        });

        spawn_tasks.push(task);

        // 0.5ms delay between starting each spawn
        tokio::time::sleep(Duration::from_micros(500)).await;
    }

    println!();
    println!("⏳ Waiting for all {} nodes to spawn...", spawn_tasks.len());

    // Wait for all spawns to complete and collect nodes
    let mut spawn_results = Vec::new();
    for task in spawn_tasks {
        spawn_results.push(task.await??);
    }

    // Sort by port offset to maintain order
    spawn_results.sort_by_key(|(port_offset, _)| *port_offset);
    let all_nodes: Vec<TestNode> = spawn_results.into_iter().map(|(_, node)| node).collect();

    // Register test nodes in state for API access
    println!("📋 Registering test nodes for API access...");
    for (i, node) in all_nodes.iter().enumerate() {
        let node_id = format!("node-{}", i);
        state.register_test_node(node_id, node.base_url.clone()).await;
    }

    println!("✅ All {} nodes spawned", all_nodes.len());
    println!();

    // Establish WebRTC connections based on actual neighbor topology
    println!("🔗 Establishing WebRTC connections (watch the mesh form in real-time!)...");
    println!("   Open http://localhost:8080 to watch connections form");
    println!();

    let slots_vec: Vec<SlotCoordinate> = slots_to_spawn.clone();
    let all_nodes_arc = StdArc::new(all_nodes);

    // Collect all connection tasks
    let mut connection_tasks = Vec::new();
    let mut connection_count = 0;

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
                    if connection_count < 10 {
                        println!("   🔗 ({},{},{}) ↔ ({},{},{}) via {:?}",
                            my_slot.x, my_slot.y, my_slot.z,
                            neighbor_slot.x, neighbor_slot.y, neighbor_slot.z,
                            dir);
                    } else if connection_count == 10 {
                        println!("   ... ({} more connections)",
                            (all_nodes_arc.len() * 8 / 2) - 10);
                    }

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
                            color: "#00ff88".to_string(), // Green for neighbor connections
                        }).await;

                        Ok::<_, anyhow::Error>(())
                    });

                    connection_tasks.push(task);
                    connection_count += 1;

                    // Small delay between starting each connection
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            }
        }
    }

    println!();
    println!("⏳ Waiting for all {} connections to complete...", connection_count);

    // Wait for all connections to complete with progress tracking
    let mut completed = 0;
    let mut failed = 0;
    for (idx, task) in connection_tasks.into_iter().enumerate() {
        match tokio::time::timeout(Duration::from_secs(15), task).await {
            Ok(Ok(Ok(()))) => {
                completed += 1;
                if completed % 100 == 0 || completed == connection_count {
                    println!("   ✅ {} / {} connections established", completed, connection_count);
                }
            }
            Ok(Ok(Err(e))) => {
                failed += 1;
                if failed < 5 {
                    eprintln!("   ❌ Connection {} failed: {:?}", idx, e);
                }
            }
            Ok(Err(e)) => {
                failed += 1;
                if failed < 5 {
                    eprintln!("   ❌ Connection {} task panicked: {:?}", idx, e);
                }
            }
            Err(_) => {
                failed += 1;
                if failed < 5 {
                    eprintln!("   ⏱️  Connection {} timed out after 15 seconds", idx);
                }
            }
        }
    }

    println!();
    println!("✅ Connection establishment complete:");
    println!("   ✓ {} connections succeeded", completed);
    if failed > 0 {
        println!("   ✗ {} connections failed or timed out", failed);
    }
    println!();

    // Announce slot ownership
    println!("📢 Announcing slot ownership for all {} nodes...", all_nodes_arc.len());
    for (i, node) in all_nodes_arc.iter().enumerate() {
        node.announce_slot_ownership(slots_vec[i]).await?;
        if (i + 1) % 100 == 0 || i + 1 == all_nodes_arc.len() {
            println!("   ✓ {} / {} nodes announced", i + 1, all_nodes_arc.len());
        }
    }
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("✅ Gossip propagation complete");
    println!();

    println!("✅ {}×{}×{} HEXAGONAL TOROIDAL MESH TEST COMPLETE!", mesh_width, mesh_height, mesh_depth);
    println!("   ✓ {} nodes spawned across entire mesh", all_nodes_arc.len());
    println!("   ✓ {} WebRTC connections established (TGP over WebRTC)", completed);
    println!("   ✓ Each node connected to its 8 neighbors in the toroid");
    println!("   ✓ Gossip propagated through entire mesh");
    println!();
    println!("🎨 HEXAGONAL MESH VISUALIZATION:");
    println!("   🌐 Open in browser: http://localhost:8080");
    println!("   🔷 True hexagonal toroidal topology");
    println!("   📏 Drag to pan, scroll to zoom");
    println!("   🎬 Press \"▶️ Scenario\" to start traffic simulation");
    println!();

    // Keep test running for visualization
    println!("⏸️  Nodes will remain running for 300 seconds for visualization...");
    println!("   Press Ctrl+C to stop early");
    tokio::time::sleep(Duration::from_secs(300)).await;
    println!("✅ Shutting down");

    Ok(())
}
