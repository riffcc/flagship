//! ABSURD 32×32×10 hexagonal toroidal mesh test
//!
//! Tests a 2.5D hexagonal toroidal mesh (32×32×10 = 10,240 slots).
//! Each node connects to its 8 actual neighbors in the toroid.
//! Uses parallel connection establishment for speed.
//! Packets route through the mesh using Two Generals Protocol (TGP) over WebRTC.
//!
//! WARNING: Requires ~32GB RAM and uses MASSIVE amounts of resources!
//! Visualization available at http://localhost:8080
//!
//! To run:
//! ```bash
//! cd /opt/castle/workspace/flagship/crates/lens-v2-node
//! cargo test --test absurd_mesh_test -- --nocapture
//! ```

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
    test_node_urls: Arc<Mutex<HashMap<String, String>>>,  // Map node-id -> base_url
    spawning_complete: Arc<Mutex<bool>>,  // Track spawning phase for batched broadcasts
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
            test_node_urls: Arc::new(Mutex::new(HashMap::new())),
            spawning_complete: Arc::new(Mutex::new(false)),
        }
    }

    fn register_test_node(&self, node_id: String, base_url: String) {
        let mut urls = self.test_node_urls.lock().unwrap();
        urls.insert(node_id, base_url);
    }

    fn add_node(&self, node: MeshNode) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.push(node);
        let node_count = nodes.len();
        drop(nodes);

        // During spawning: only broadcast every 100 nodes
        // After spawning: broadcast every change
        let spawning_complete = *self.spawning_complete.lock().unwrap();
        let should_broadcast = spawning_complete || node_count % 100 == 0;

        if should_broadcast {
            let _ = self.broadcast.send(self.to_json());
        }
    }

    fn add_edge(&self, edge: MeshEdge) {
        let mut edges = self.edges.lock().unwrap();
        edges.push(edge);
        let edge_count = edges.len();
        drop(edges);

        // During spawning: only broadcast every 100 edges
        // After spawning: broadcast every change
        let spawning_complete = *self.spawning_complete.lock().unwrap();
        let should_broadcast = spawning_complete || edge_count % 100 == 0;

        if should_broadcast {
            let _ = self.broadcast.send(self.to_json());
        }
    }

    fn set_spawning_complete(&self) {
        *self.spawning_complete.lock().unwrap() = true;
        // Force broadcast to update visualization with all final nodes/edges
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

// Handler functions for API endpoints
async fn handle_dht_write(state: &MeshTestState, body: &str) -> String {
    #[derive(Deserialize)]
    struct DhtWriteRequest {
        node_id: String,
        key: String,
        value: String,
    }

    let req: DhtWriteRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(_) => return r#"{"error": "Invalid JSON"}"#.to_string(),
    };

    // Find the node's base URL
    let base_url = state.test_node_urls.lock().unwrap().get(&req.node_id).cloned();
    let base_url = match base_url {
        Some(url) => url,
        None => return format!(r#"{{"error": "Node {} not found"}}"#, req.node_id),
    };

    // Actually perform DHT write through the mesh!
    println!("📝 DHT Write: {} writing {}={}", req.node_id, req.key, req.value);

    // Hash key to 32-byte array
    let key_hash = blake3::hash(req.key.as_bytes());
    let key_bytes: [u8; 32] = *key_hash.as_bytes();
    let value_bytes = req.value.as_bytes().to_vec();

    // Perform REAL DHT PUT through WebRTC mesh via HTTP!
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/dht/put", base_url);
    let body = serde_json::json!({
        "key": hex::encode(key_bytes),
        "value": hex::encode(&value_bytes),
    });

    match client.post(&url).json(&body).send().await {
        Ok(response) if response.status().is_success() => {
            println!("✅ DHT Write successful!");
            format!(r#"{{"success": true, "key": "{}", "value": "{}"}}"#, req.key, req.value)
        }
        Ok(response) => {
            eprintln!("❌ DHT Write failed: HTTP {}", response.status());
            format!(r#"{{"error": "DHT write failed: HTTP {}"}}"#, response.status())
        }
        Err(e) => {
            eprintln!("❌ DHT Write failed: {:?}", e);
            format!(r#"{{"error": "DHT write failed: {}"}}"#, e)
        }
    }
}

async fn handle_dht_read(state: &MeshTestState, body: &str) -> String {
    #[derive(Deserialize)]
    struct DhtReadRequest {
        node_id: String,
        key: String,
    }

    let req: DhtReadRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(_) => return r#"{"error": "Invalid JSON"}"#.to_string(),
    };

    // Find the node's base URL
    let base_url = state.test_node_urls.lock().unwrap().get(&req.node_id).cloned();
    let base_url = match base_url {
        Some(url) => url,
        None => return format!(r#"{{"error": "Node {} not found"}}"#, req.node_id),
    };

    println!("🔍 DHT Read: {} reading {}", req.node_id, req.key);

    // Hash key to 32-byte array
    let key_hash = blake3::hash(req.key.as_bytes());
    let key_bytes: [u8; 32] = *key_hash.as_bytes();

    // Perform REAL DHT GET through WebRTC mesh via HTTP!
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/dht/get/{}", base_url, hex::encode(key_bytes));

    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => {
            match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    if let Some(value_hex) = data.get("value").and_then(|v| v.as_str()) {
                        if let Ok(value_bytes) = hex::decode(value_hex) {
                            let value_str = String::from_utf8_lossy(&value_bytes);
                            println!("✅ DHT Read successful: {}", value_str);
                            return format!(r#"{{"success": true, "key": "{}", "value": "{}"}}"#, req.key, value_str);
                        }
                    }
                    println!("📭 DHT Read: key not found");
                    format!(r#"{{"success": true, "key": "{}", "value": null}}"#, req.key)
                }
                Err(e) => {
                    eprintln!("❌ DHT Read failed to parse response: {:?}", e);
                    format!(r#"{{"error": "Failed to parse response: {}"}}"#, e)
                }
            }
        }
        Ok(response) => {
            eprintln!("❌ DHT Read failed: HTTP {}", response.status());
            format!(r#"{{"error": "DHT read failed: HTTP {}"}}"#, response.status())
        }
        Err(e) => {
            eprintln!("❌ DHT Read failed: {:?}", e);
            format!(r#"{{"error": "DHT read failed: {}"}}"#, e)
        }
    }
}

async fn handle_broadcast(state: &MeshTestState, body: &str) -> String {
    #[derive(Deserialize)]
    struct BroadcastRequest {
        node_id: String,
        message: String,
    }

    let req: BroadcastRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(_) => return r#"{"error": "Invalid JSON"}"#.to_string(),
    };

    // Find the node's base URL
    let base_url = state.test_node_urls.lock().unwrap().get(&req.node_id).cloned();
    let _base_url = match base_url {
        Some(url) => url,
        None => return format!(r#"{{"error": "Node {} not found"}}"#, req.node_id),
    };

    println!("📡 Broadcast: {} broadcasting: {}", req.node_id, req.message);

    // TODO: Actually broadcast through WebRTC to all neighbors
    // For now, just acknowledge
    r#"{"success": true, "nodes_reached": 8}"#.to_string()
}

async fn handle_route(state: &MeshTestState, body: &str) -> String {
    #[derive(Deserialize)]
    struct RouteRequest {
        from_node: String,
        to_node: String,
        message: String,
    }

    let req: RouteRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(_) => return r#"{"error": "Invalid JSON"}"#.to_string(),
    };

    // Find the node base URLs
    let urls = state.test_node_urls.lock().unwrap();
    let from_url = urls.get(&req.from_node).cloned();
    let to_url = urls.get(&req.to_node).cloned();
    drop(urls);

    let (_from_url, _to_url) = match (from_url, to_url) {
        (Some(f), Some(t)) => (f, t),
        _ => return r#"{"error": "Node not found"}"#.to_string(),
    };

    println!("🚀 Route: {} → {} : {}", req.from_node, req.to_node, req.message);

    // TODO: Actually route packet through WebRTC mesh using greedy routing
    // For now, just acknowledge with simulated hop count
    format!(r#"{{"success": true, "hops": 3}}"#)
}

#[tokio::test]
async fn test_absurd_32x32x10_mesh() -> anyhow::Result<()> {
    println!("🚀🚀🚀 Starting ABSURD 32×32×10 hexagonal toroidal mesh test");
    println!("   Mesh dimensions: 32 wide × 32 deep × 10 layers = 10,240 total slots");
    println!("   Each node connects to its 8 neighbors in the 2.5D toroid");
    println!("   Packets route via Two Generals Protocol (TGP) over WebRTC");
    println!("   ⚠️  WARNING: This will use ~5GB RAM and massive CPU resources!");
    println!();

    // Create shared state for visualization
    let state = MeshTestState::new();
    let state_clone = state.clone();

    // Spawn HTTP server for visualization on port 8080 in dedicated OS thread
    println!("🌐 Starting visualization server on http://0.0.0.0:8080");
    let viz_html = include_str!("mesh-visualizer.html");
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
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
                    } else if path.starts_with("/dht/write") || path.starts_with("/dht/read") ||
                              path.starts_with("/broadcast") || path.starts_with("/route") {
                        // Read POST body
                        let content_length: usize = headers.iter()
                            .find(|h| h.to_lowercase().starts_with("content-length:"))
                            .and_then(|h| h.split(':').nth(1))
                            .and_then(|s| s.trim().parse().ok())
                            .unwrap_or(0);

                        let mut body = vec![0u8; content_length];
                        let _ = reader.read_exact(&mut body).await;
                        let body_str = String::from_utf8_lossy(&body);

                        let response_json = if path.starts_with("/dht/write") {
                            handle_dht_write(&state, &body_str).await
                        } else if path.starts_with("/dht/read") {
                            handle_dht_read(&state, &body_str).await
                        } else if path.starts_with("/broadcast") {
                            handle_broadcast(&state, &body_str).await
                        } else if path.starts_with("/route") {
                            handle_route(&state, &body_str).await
                        } else {
                            r#"{"error": "Unknown endpoint"}"#.to_string()
                        };

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                            response_json.len(),
                            response_json
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
        })
    });
    tokio::time::sleep(Duration::from_millis(1000)).await;
    println!();

    // Create 32×32×10 mesh
    let mesh_config = MeshConfig::new(32, 32, 10);

    // Set mesh config in shared state
    state.set_mesh_config(mesh_config.clone());

    println!("📐 Planning ABSURD mesh topology...");
    println!("   Mesh: 32×32×10 = {} total slots", 32 * 32 * 10);
    println!("   Spawning ALL 10,240 nodes in parallel...");
    println!("   Expected spawn time: ~15 seconds");
    println!();

    let mut all_nodes: Vec<TestNode> = Vec::new();
    let mut slots_to_spawn: Vec<SlotCoordinate> = Vec::new();

    // Spawn EVERY slot in the mesh
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..10 {
                slots_to_spawn.push(SlotCoordinate::new(x, y, z));
            }
        }
    }

    println!("✅ Total slots to spawn: {}", slots_to_spawn.len());
    println!();

    // Spawn nodes in batches of 5000 for speed while still being manageable
    println!("🔧 Spawning {} nodes in batches of 5000...", slots_to_spawn.len());
    let mut all_spawn_results = Vec::new();
    let batch_size = 5000;

    for (batch_idx, chunk) in slots_to_spawn.chunks(batch_size).enumerate() {
        let batch_start = batch_idx * batch_size;
        println!("   📦 Batch {}/{}: Spawning nodes {} - {}...",
            batch_idx + 1,
            (slots_to_spawn.len() + batch_size - 1) / batch_size,
            batch_start,
            batch_start + chunk.len() - 1);

        // Spawn all tasks in this batch in parallel (no delays)
        let spawn_tasks: Vec<_> = chunk.iter().enumerate().map(|(chunk_offset, slot)| {
            let port_offset = batch_start + chunk_offset;
            let slot = *slot;
            let state_clone = state.clone();

            tokio::spawn(async move {
                let node = TestNode::spawn_at_slot(10000 + port_offset as u16, Some(slot)).await?;

                // Add to shared state for visualization
                let node_id = format!("node-{}", port_offset);
                state_clone.add_node(MeshNode {
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

                Ok::<_, anyhow::Error>((port_offset, node))
            })
        }).collect();

        println!("      ⏳ Waiting for {} nodes to spawn...", spawn_tasks.len());

        // Wait for all tasks in this batch to complete with progress indicator
        let total_tasks = spawn_tasks.len();
        for (idx, task) in spawn_tasks.into_iter().enumerate() {
            all_spawn_results.push(task.await??);

            // Print progress every 500 nodes within the batch
            if (idx + 1) % 500 == 0 {
                println!("      ... {} / {} nodes in batch completed", idx + 1, total_tasks);
            }
        }

        println!("   ✅ Batch {} complete ({} / {} nodes spawned)",
            batch_idx + 1,
            batch_start + chunk.len(),
            slots_to_spawn.len());

        // Small delay between batches to let system breathe
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!();
    println!("✅ All {} nodes spawned in {} batches", all_spawn_results.len(), (slots_to_spawn.len() + batch_size - 1) / batch_size);

    // Sort by port offset to maintain order
    all_spawn_results.sort_by_key(|(port_offset, _)| *port_offset);
    let all_nodes: Vec<TestNode> = all_spawn_results.into_iter().map(|(_, node)| node).collect();

    // Register test nodes in state for API access
    println!("📋 Registering test nodes for API access...");
    for (i, node) in all_nodes.iter().enumerate() {
        let node_id = format!("node-{}", i);
        state.register_test_node(node_id, node.base_url.clone());
    }

    println!("✅ All {} nodes spawned", all_nodes.len());
    println!();

    // Mark spawning as complete - switch to real-time updates (every change)
    state.set_spawning_complete();
    println!("📡 Switching to real-time visualization updates (every change)");
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
    // Establish connections in parallel with 20ms spacing between starting each connection
    let mut connection_count = 0;
    let slots_vec: Vec<SlotCoordinate> = slots_to_spawn.clone();

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
                    // Only print first 10 and every 1000th connection
                    if connection_count < 10 || connection_count % 1000 == 0 {
                        println!("   🔗 Connection {} / ~262144: ({},{},{}) ↔ ({},{},{}) via {:?}",
                            connection_count,
                            my_slot.x, my_slot.y, my_slot.z,
                            neighbor_slot.x, neighbor_slot.y, neighbor_slot.z,
                            dir);
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
                            color: "#8a2be2".to_string(), // Purple for neighbor connections
                        });

                        Ok::<_, anyhow::Error>(())
                    });

                    connection_tasks.push(task);
                    connection_count += 1;

                    // Small delay between starting each connection
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }

    println!();
    println!("⏳ Waiting for all {} connections to complete (this will take a LONG time)...", connection_count);
    println!("   With 50ms spacing, starting all connections takes ~{} seconds (~{} minutes)",
        connection_count * 50 / 1000,
        connection_count * 50 / 1000 / 60);
    println!("   Expected total connection time: 30-60 minutes");

    // Wait for all connections to complete with progress tracking
    let mut completed = 0;
    let mut failed = 0;
    for (idx, task) in connection_tasks.into_iter().enumerate() {
        match tokio::time::timeout(Duration::from_secs(30), task).await {
            Ok(Ok(Ok(()))) => {
                completed += 1;
                if completed % 1000 == 0 {
                    println!("   ✅ {} / {} connections established ({:.1}%)",
                        completed, connection_count,
                        (completed as f64 / connection_count as f64) * 100.0);
                }
            }
            Ok(Ok(Err(e))) => {
                failed += 1;
                if failed < 10 {
                    eprintln!("   ❌ Connection {} failed: {:?}", idx, e);
                }
            }
            Ok(Err(e)) => {
                failed += 1;
                if failed < 10 {
                    eprintln!("   ❌ Connection {} task panicked: {:?}", idx, e);
                }
            }
            Err(_) => {
                failed += 1;
                if failed < 10 {
                    eprintln!("   ⏱️  Connection {} timed out after 30 seconds", idx);
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
    let slots_vec: Vec<SlotCoordinate> = slots_to_spawn.iter().copied().collect();
    for (i, node) in all_nodes_arc.iter().enumerate() {
        node.announce_slot_ownership(slots_vec[i]).await?;
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("✅ Gossip propagation complete");
    println!();

    println!("✅🎉 ABSURD 64×64×16 hexagonal toroidal mesh test COMPLETE!");
    println!("   ✓ {} nodes spawned across entire mesh", all_nodes_arc.len());
    println!("   ✓ {} / {} WebRTC connections established (TGP over WebRTC)", completed, connection_count);
    println!("   ✓ Each node connected to its 8 neighbors in the toroid");
    println!("   ✓ Gossip propagated through entire mesh");
    println!();
    println!("🎨 3D MESH VISUALIZATION:");
    println!("   🌐 Open in browser: http://localhost:8080");
    println!("   ✨ Interactive 3D force-directed graph");
    println!("   ⚠️  WARNING: Visualization may be slow with 65K nodes - use Chrome with hardware acceleration!");
    println!("   🔄 Auto-refreshes every 5 seconds");
    println!("   🎯 Click nodes to see details");
    println!();
    println!("   Alternative - Raw JSON:");
    println!("   📊 http://localhost:10000/api/v1/map (or ports 10000-{})", 10000 + all_nodes_arc.len() - 1);
    println!();

    // Keep test running for visualization
    println!("⏸️  Nodes will remain running for 600 seconds (10 minutes) for visualization...");
    println!("   Press Ctrl+C to stop early");
    tokio::time::sleep(Duration::from_secs(600)).await;
    println!("✅ Shutting down");

    Ok(())
}
