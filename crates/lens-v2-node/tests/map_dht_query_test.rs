//! Test that /map can query slot ownership records from the distributed DHT
//!
//! This test spins up 10 real nodes (ports 11301-11310) and verifies that:
//! 1. Each node can announce its slot ownership to the DHT
//! 2. Any node can query the DHT and discover ALL other nodes
//! 3. The /map endpoint returns all 10 peers

use lens_node::routes::{RelayState, SyncState};
use lens_node::peer_registry::{SlotOwnership, slot_ownership_key, peer_id_to_slot, calculate_mesh_dimensions};
use lens_v2_p2p::{P2pManager, P2pConfig};
use citadel_core::topology::SlotCoordinate;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use futures::SinkExt;

#[tokio::test]
async fn test_map_queries_dht_with_10_nodes() {
    // Spawn 10 ACTUAL Lens nodes on ports 11301-11310
    // Each node will have a full SyncOrchestrator that connects to OTHER nodes
    let num_nodes = 10;
    let base_port = 11301;

    let mut node_handles = vec![];

    // Start all 10 nodes
    for i in 0..num_nodes {
        let port = base_port + i;

        // Each node connects to the first node (11301) as its initial peer
        let relay_url = format!("ws://localhost:{}/api/v1/relay/ws", base_port);

        let handle = tokio::spawn(async move {
            start_full_test_node(port, relay_url).await
        });

        node_handles.push(handle);

        // Stagger startup to ensure first node is ready
        if i == 0 {
            sleep(Duration::from_millis(1000)).await;
        } else {
            sleep(Duration::from_millis(200)).await;
        }
    }

    // Give nodes time to connect and announce to DHT
    println!("⏳ Waiting for all nodes to announce...");
    sleep(Duration::from_secs(3)).await;

    // Query one node's /map endpoint
    let map_url = format!("http://localhost:{}/api/v1/map", base_port);
    let response = reqwest::get(&map_url).await.expect("Failed to query /map");
    let map_data: serde_json::Value = response.json().await.expect("Failed to parse /map response");

    println!("📊 Map response: {}", serde_json::to_string_pretty(&map_data).unwrap());

    let nodes = map_data["nodes"].as_array().expect("nodes should be an array");

    // Assert we found all 10 peers
    assert_eq!(nodes.len(), num_nodes as usize, "Should find all {} peers in DHT", num_nodes);

    // Cleanup: stop all nodes
    for handle in node_handles {
        handle.abort();
    }
}

/// Start a FULL test node with SyncOrchestrator that connects to other nodes
async fn start_full_test_node(port: u16, relay_url: String) {
    use lens_node::{routes, db::Database, site_identity::SiteIdentity, sync_orchestrator::SyncOrchestrator, peer_registry};

    // Create temporary database
    let temp_dir = std::env::temp_dir().join(format!("lens-test-node-{}", port));
    let _ = std::fs::remove_dir_all(&temp_dir);
    let db = Database::open(&temp_dir).unwrap();

    // Generate peer_id for this node
    let my_peer_id = format!("peer-{}", rand::random::<u64>());

    // Calculate mesh config and slot (start with 1, will grow as peers join)
    let mesh_config = peer_registry::calculate_mesh_dimensions(1);
    let my_slot = peer_registry::peer_id_to_slot(&my_peer_id, &mesh_config);

    println!("🚀 Starting test node {} on port {} at slot {:?}", my_peer_id, port, my_slot);

    // Initialize states
    let registry = Arc::new(routes::initialize_registry());
    let state = routes::AppState { registry };

    let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));

    // Create shared DHT storage
    let dht_storage = Arc::new(tokio::sync::Mutex::new(lens_node::dht_state::DhtState::new()));

    let relay_state = RelayState::new()
        .with_node_peer_id(my_peer_id.clone())
        .with_p2p_manager(p2p_manager.clone())
        .with_dht_storage(dht_storage.clone());

    let (block_notify_tx, block_notify_rx) = tokio::sync::mpsc::unbounded_channel();

    let account_state = routes::AccountState::new(db.clone()).with_notify(block_notify_tx.clone());
    let releases_state = routes::ReleasesState::with_db(account_state.clone(), db.clone()).unwrap()
        .with_notify(block_notify_tx);

    let sync_state = SyncState { p2p: p2p_manager.clone() };

    let identity = SiteIdentity::initialize(&db, Some(format!("Test Node {}", port))).await.unwrap();
    let site_state = routes::SiteState::new(Arc::new(identity));

    // Create router
    let app = routes::create_router(
        state,
        relay_state.clone(),
        account_state,
        releases_state,
        sync_state,
        None,
        site_state,
    );

    // Start server
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Spawn server in background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create WebRTC manager
    let webrtc_manager = Arc::new(lens_node::webrtc_manager::WebRTCManager::new().unwrap());

    // Create and start SyncOrchestrator (this connects to OTHER nodes!)
    let orchestrator = Arc::new(SyncOrchestrator::new(
        relay_url.clone(),
        my_peer_id.clone(),
        my_slot,
        mesh_config,
        p2p_manager.clone(),
        webrtc_manager,
        db,
        block_notify_rx,
        dht_storage,
    ));

    println!("🔗 Node {} connecting to mesh via {}", my_peer_id, relay_url);

    // Start the orchestrator (this is what makes the node ACTUALLY connect!)
    tokio::spawn({
        let orch = orchestrator.clone();
        async move {
            if let Err(e) = orch.start().await {
                eprintln!("❌ Failed to start orchestrator: {}", e);
            }
        }
    });

    // Wait for at least one peer connection
    for _ in 0..100 {
        if let Ok(status) = p2p_manager.sync_status() {
            if status.known_peers > 0 {
                println!("✅ Node {} has {} peer(s), announcing to DHT", my_peer_id, status.known_peers);
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // NOW announce slot ownership to DHT (with peers connected!)
    let ownership = peer_registry::SlotOwnership::new(my_peer_id.clone(), my_slot, None);
    let ownership_bytes = serde_json::to_vec(&ownership).unwrap();
    relay_state.dht_put(peer_registry::slot_ownership_key(my_slot), ownership_bytes.clone()).await;
    relay_state.dht_put(peer_registry::peer_location_key(&my_peer_id), ownership_bytes).await;

    println!("📢 Node {} announced to DHT", my_peer_id);

    // Keep node running
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}


#[tokio::test]
async fn test_map_queries_empty_dht() {
    // Create relay state with NO peers announced
    let relay_state = RelayState::new();
    let p2p_manager = Arc::new(P2pManager::new(P2pConfig::default()));
    let sync_state = SyncState { p2p: p2p_manager.clone() };

    // Add known peers to P2P manager but DON'T announce to DHT
    p2p_manager.add_known_peer(1).unwrap();
    p2p_manager.add_known_peer(2).unwrap();

    let mesh_config = calculate_mesh_dimensions(2);

    // Query DHT for slot ownership (should return empty)
    let mut found_peers: Vec<String> = Vec::new();

    for x in 0..mesh_config.width as i32 {
        for y in 0..mesh_config.height as i32 {
            for z in 0..mesh_config.depth as i32 {
                let slot = SlotCoordinate::new(x, y, z);
                let slot_key = slot_ownership_key(slot);

                if let Some(ownership_bytes) = relay_state.dht_get(slot_key).await {
                    if let Ok(ownership) = serde_json::from_slice::<SlotOwnership>(&ownership_bytes) {
                        found_peers.push(ownership.peer_id.clone());
                    }
                }
            }
        }
    }

    // Assert we found 0 peers (DHT is empty)
    assert_eq!(found_peers.len(), 0, "Should find 0 peers when DHT is empty");
}
