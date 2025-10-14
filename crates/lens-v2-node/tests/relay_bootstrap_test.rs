//! Relay DHT Bootstrap Tests
//!
//! Tests for relay handling DHT bootstrap requests and responses.
//! Ensures peers can bootstrap from relay's global DHT on connection.

use lens_node::routes::relay::{RelayState, DhtBootstrapRequest, DhtBootstrapResponse};
use lens_node::dht_state::{DhtEntry, DhtState};
use citadel_core::topology::SlotCoordinate;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(byte: u8, value: &str, timestamp: u64) -> DhtEntry {
        let mut key = [0u8; 32];
        key[0] = byte;
        DhtEntry {
            key,
            value: value.as_bytes().to_vec(),
            timestamp,
        }
    }

    #[tokio::test]
    async fn test_relay_starts_with_empty_dht() {
        // When relay starts, DHT should be empty
        let relay_state = RelayState::new();

        let dht = relay_state.dht_storage.lock().await;
        assert_eq!(dht.len(), 0, "Relay starts with empty DHT");
    }

    #[tokio::test]
    async fn test_relay_stores_dht_entry() {
        // Relay should be able to store DHT entries
        let relay_state = RelayState::new();

        let entry = make_entry(1, "peer_1_slot", 1000);

        {
            let mut dht = relay_state.dht_storage.lock().await;
            dht.insert(entry.clone());
        }

        let dht = relay_state.dht_storage.lock().await;
        assert_eq!(dht.len(), 1, "DHT has 1 entry");

        let stored = dht.get(&entry.key).expect("Entry exists");
        assert_eq!(stored.value, b"peer_1_slot");
        assert_eq!(stored.timestamp, 1000);
    }

    #[tokio::test]
    async fn test_relay_provides_bootstrap_snapshot() {
        // Relay should provide complete DHT snapshot for bootstrapping
        let relay_state = RelayState::new();

        // Populate relay DHT with multiple entries
        {
            let mut dht = relay_state.dht_storage.lock().await;
            dht.insert(make_entry(1, "peer_1", 1000));
            dht.insert(make_entry(2, "peer_2", 1001));
            dht.insert(make_entry(3, "peer_3", 1002));
        }

        // Get bootstrap snapshot
        let dht = relay_state.dht_storage.lock().await;
        let entries = dht.to_sorted_vec();
        drop(dht);

        assert_eq!(entries.len(), 3, "Bootstrap includes all 3 entries");

        // Verify entries are sorted by key
        assert_eq!(entries[0].key[0], 1);
        assert_eq!(entries[1].key[0], 2);
        assert_eq!(entries[2].key[0], 3);
    }

    #[tokio::test]
    async fn test_first_peer_bootstraps_from_empty_dht() {
        // First peer connects to empty relay DHT
        let relay_state = RelayState::new();

        let dht = relay_state.dht_storage.lock().await;
        assert_eq!(dht.len(), 0, "Relay DHT is empty");

        // First peer should receive empty bootstrap response
        let bootstrap_entries = dht.to_sorted_vec();
        assert_eq!(bootstrap_entries.len(), 0, "First peer gets empty bootstrap");
    }

    #[tokio::test]
    async fn test_second_peer_bootstraps_from_first_peer_dht() {
        // Scenario: First peer announced, second peer bootstraps from it
        let relay_state = RelayState::new();

        // First peer announces its slot
        {
            let mut dht = relay_state.dht_storage.lock().await;
            dht.insert(make_entry(1, "first_peer_slot", 1000));
        }

        // Second peer connects and requests bootstrap
        let dht = relay_state.dht_storage.lock().await;
        let bootstrap_entries = dht.to_sorted_vec();

        assert_eq!(bootstrap_entries.len(), 1, "Second peer bootstraps from first peer");
        assert_eq!(bootstrap_entries[0].value, b"first_peer_slot");
    }

    #[tokio::test]
    async fn test_relay_merges_peer_dht_on_connect() {
        // Scenario: Peer connects with its own DHT (from network partition)
        let relay_state = RelayState::new();

        // Relay has some entries
        {
            let mut dht = relay_state.dht_storage.lock().await;
            dht.insert(make_entry(1, "relay_peer_1", 1000));
            dht.insert(make_entry(2, "relay_peer_2", 1001));
        }

        // Peer connects with different DHT entries
        let mut peer_dht = DhtState::new();
        peer_dht.insert(make_entry(3, "peer_3", 1002));
        peer_dht.insert(make_entry(4, "peer_4", 1003));

        // Merge peer DHT into relay
        {
            let mut relay_dht = relay_state.dht_storage.lock().await;
            relay_dht.merge(&peer_dht);
        }

        // Relay should now have all 4 entries
        let dht = relay_state.dht_storage.lock().await;
        assert_eq!(dht.len(), 4, "Relay merged peer DHT");
        assert!(dht.get(&make_entry(1, "", 0).key).is_some());
        assert!(dht.get(&make_entry(2, "", 0).key).is_some());
        assert!(dht.get(&make_entry(3, "", 0).key).is_some());
        assert!(dht.get(&make_entry(4, "", 0).key).is_some());
    }

    #[tokio::test]
    async fn test_relay_merge_resolves_conflicts_with_latest_timestamp() {
        // Scenario: Relay and peer have conflicting entries, latest wins
        let relay_state = RelayState::new();

        // Relay has older entry
        {
            let mut dht = relay_state.dht_storage.lock().await;
            dht.insert(make_entry(1, "old_value", 1000));
        }

        // Peer has newer entry for same key
        let mut peer_dht = DhtState::new();
        peer_dht.insert(make_entry(1, "new_value", 2000)); // Newer timestamp

        // Merge
        {
            let mut relay_dht = relay_state.dht_storage.lock().await;
            relay_dht.merge(&peer_dht);
        }

        // Check conflict resolution
        let dht = relay_state.dht_storage.lock().await;
        let entry = dht.get(&make_entry(1, "", 0).key).expect("Entry exists");
        assert_eq!(entry.value, b"new_value", "Newer timestamp won");
        assert_eq!(entry.timestamp, 2000);
    }

    #[tokio::test]
    async fn test_bootstrap_request_includes_peer_slot() {
        // Bootstrap request should include peer's slot for proximity queries
        let request = DhtBootstrapRequest {
            peer_id: "peer-123".to_string(),
            slot: SlotCoordinate { x: 1, y: 2, z: 0 },
        };

        assert_eq!(request.peer_id, "peer-123");
        assert_eq!(request.slot.x, 1);
        assert_eq!(request.slot.y, 2);
    }

    #[tokio::test]
    async fn test_bootstrap_response_serializable() {
        // Bootstrap response should be serializable for WebSocket transmission
        let entries = vec![
            make_entry(1, "peer_1", 1000),
            make_entry(2, "peer_2", 1001),
        ];

        let response = DhtBootstrapResponse {
            dht_entries: entries.clone(),
            entry_count: entries.len(),
            timestamp: 123456,
        };

        assert_eq!(response.entry_count, 2);
        assert_eq!(response.dht_entries.len(), 2);

        // Verify serialization works
        let json = serde_json::to_string(&response).expect("Serializes to JSON");
        assert!(json.contains("dht_entries"));
    }
}
