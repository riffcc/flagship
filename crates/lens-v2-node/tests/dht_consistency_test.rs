//! Test that DHT consistency endpoint returns correct node peer_id
//!
//! This test verifies:
//! 1. dht_consistency_handler uses node_peer_id from RelayState (NOT from WebSocket connections)
//! 2. my_peer_id in the response is never "unknown"
//! 3. The endpoint works even when no WebSocket peers are connected

use axum::extract::State;
use lens_node::routes::RelayState;
use lens_node::routes::relay::DhtConsistencyReport;

#[tokio::test]
async fn test_dht_consistency_shows_correct_peer_id() {
    // Create RelayState with a known node_peer_id
    let known_peer_id = "bafk2bzaceaabcdef1234567890abcdef1234567890abcdef1234567890abcd";
    let relay_state = RelayState::new()
        .with_node_peer_id(known_peer_id.to_string());

    // Call dht_consistency_handler
    let handler = lens_node::routes::relay::dht_consistency_handler;
    let result = handler(State(relay_state)).await;

    // Assert it returns Ok (not an error)
    assert!(result.is_ok(), "dht_consistency_handler should not return error");

    let response = result.unwrap();
    let report: DhtConsistencyReport = response.0;

    // Assert my_peer_id matches the node_peer_id we set
    assert_eq!(
        report.my_peer_id, known_peer_id,
        "my_peer_id should be the node_peer_id from RelayState, not 'unknown'"
    );

    // Assert it's not "unknown"
    assert_ne!(
        report.my_peer_id, "unknown",
        "my_peer_id should never be 'unknown'"
    );

    println!("✅ DHT consistency endpoint returns correct peer_id: {}", report.my_peer_id);
}

#[tokio::test]
async fn test_dht_consistency_works_without_websocket_peers() {
    // Create RelayState with no WebSocket connections
    let known_peer_id = "bafk2bzaceatest123456789test123456789test123456789test123456789";
    let relay_state = RelayState::new()
        .with_node_peer_id(known_peer_id.to_string());

    // Verify no WebSocket peers are connected
    let peer_senders = relay_state.peer_senders.read().await;
    assert_eq!(peer_senders.len(), 0, "Should have no WebSocket peers");
    drop(peer_senders);

    // Call dht_consistency_handler
    let handler = lens_node::routes::relay::dht_consistency_handler;
    let result = handler(State(relay_state)).await;

    // Assert it still works and returns our peer_id
    assert!(result.is_ok(), "Should work even without WebSocket peers");

    let response = result.unwrap();
    let report: DhtConsistencyReport = response.0;

    assert_eq!(
        report.my_peer_id, known_peer_id,
        "Should use node_peer_id even when no WebSocket peers connected"
    );

    println!("✅ DHT consistency works without WebSocket peers: {}", report.my_peer_id);
}
