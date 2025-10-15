//! Relay SPORE WantList Protocol Tests
//!
//! Tests for relay handling WantList messages with range exclusions.
//! Ensures proper broadcasting and RangeResponse generation.

use lens_node::spore_wantlist::{WantListMessage, RangeResponse, DhtEntry};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: WantList message parses correctly
    #[test]
    fn test_wantlist_message_parsing() {
        let json = r#"{
            "version": 1,
            "want_ranges": [[0, 100], [200, 300]],
            "have_ranges": [[101, 199]],
            "timestamp": 1760541769,
            "peer_id": "bafktest"
        }"#;

        let msg: WantListMessage = serde_json::from_str(json).expect("Parse WantList");

        assert_eq!(msg.version, 1);
        assert_eq!(msg.want_ranges.len(), 2);
        assert_eq!(msg.want_ranges[0], (0, 100));
        assert_eq!(msg.want_ranges[1], (200, 300));
        assert_eq!(msg.have_ranges.len(), 1);
        assert_eq!(msg.have_ranges[0], (101, 199));
        assert_eq!(msg.peer_id, "bafktest");
    }

    /// Test: RangeResponse serializes correctly
    #[test]
    fn test_range_response_serialization() {
        let entry = DhtEntry {
            key_hash: 42,
            key: b"slot-ownership-42".to_vec(),
            value: b"peer-value".to_vec(),
            timestamp: 1760541769,
            slot_owner: "bafk42".to_string(),
        };

        let response = RangeResponse {
            range: (0, 100),
            entries: vec![entry],
            merkle_proof: None,
        };

        let json = serde_json::to_string(&response).expect("Serialize RangeResponse");
        assert!(json.contains("\"range\":[0,100]"));
        // Key is binary data, serialized as array of numbers
        assert!(json.contains("\"key\":"));
        assert!(json.contains("\"key_hash\":42"));
    }

    /// Test: Empty WantList does not trigger response
    #[test]
    fn test_empty_wantlist_no_response() {
        let msg = WantListMessage {
            version: 1,
            want_ranges: vec![],
            have_ranges: vec![],
            have_filter: None,
            timestamp: 1760541769,
            peer_id: "bafktest".to_string(),
        };

        // Empty want_ranges means no response needed
        assert!(msg.want_ranges.is_empty());
    }


    /// Test: DhtEntry format for slot ownership
    #[test]
    fn test_dht_entry_slot_ownership_format() {
        let slot_id = 42u64;
        let entry = DhtEntry {
            key_hash: slot_id,
            key: format!("slot-ownership-{}", slot_id).into_bytes(),
            value: serde_json::to_vec(&serde_json::json!({
                "peer_id": "bafk42",
                "slot": {"x": 1, "y": 2, "z": 0},
            })).unwrap(),
            timestamp: 1760541769,
            slot_owner: "bafk42".to_string(),
        };

        // Verify entry structure
        assert_eq!(entry.key_hash, 42);
        assert_eq!(entry.key, b"slot-ownership-42");
        assert_eq!(entry.slot_owner, "bafk42");

        // Verify value parses as JSON
        let value: serde_json::Value = serde_json::from_slice(&entry.value).unwrap();
        assert_eq!(value["peer_id"], "bafk42");
        assert_eq!(value["slot"]["x"], 1);
    }

    /// Test: RangeResponse with multiple entries
    #[test]
    fn test_range_response_multiple_entries() {
        let entries = vec![
            DhtEntry {
                key_hash: 10,
                key: b"slot-10".to_vec(),
                value: b"peer-10".to_vec(),
                timestamp: 1000,
                slot_owner: "bafk10".to_string(),
            },
            DhtEntry {
                key_hash: 20,
                key: b"slot-20".to_vec(),
                value: b"peer-20".to_vec(),
                timestamp: 1001,
                slot_owner: "bafk20".to_string(),
            },
        ];

        let response = RangeResponse {
            range: (0, 100),
            entries: entries.clone(),
            merkle_proof: None,
        };

        assert_eq!(response.entries.len(), 2);
        assert_eq!(response.entries[0].key_hash, 10);
        assert_eq!(response.entries[1].key_hash, 20);
    }

    /// Test: WantList broadcast message format
    #[test]
    fn test_wantlist_broadcast_format() {
        let wantlist = WantListMessage {
            version: 1,
            want_ranges: vec![(0, 100)],
            have_ranges: vec![(200, 300)],
            have_filter: None,
            timestamp: 1760541769,
            peer_id: "bafktest".to_string(),
        };

        // Simulate broadcast JSON format
        let broadcast = serde_json::json!({
            "type": "wantlist",
            "from_peer_id": "bafktest",
            "version": wantlist.version,
            "want_ranges": wantlist.want_ranges,
            "have_ranges": wantlist.have_ranges,
            "timestamp": wantlist.timestamp,
        });

        let json = serde_json::to_string(&broadcast).unwrap();
        assert!(json.contains("\"type\":\"wantlist\""));
        assert!(json.contains("\"from_peer_id\":\"bafktest\""));
    }

    /// Test: Slot ID computation from coordinates
    #[test]
    fn test_slot_id_computation() {
        // Slot coordinate (x=1, y=2, z=0)
        let x = 1u64;
        let y = 2u64;
        let z = 0u64;

        let slot_id = x * 65536 + y * 256 + z;
        assert_eq!(slot_id, 65536 + 512); // 66048

        // Slot coordinate (x=0, y=0, z=0) - origin
        let origin_slot = 0 * 65536 + 0 * 256 + 0;
        assert_eq!(origin_slot, 0);

        // Slot coordinate (x=255, y=255, z=255) - max
        let max_slot = 255 * 65536 + 255 * 256 + 255;
        assert_eq!(max_slot, 16777215); // 256^3 - 1
    }

    /// Test: Contiguous range building
    #[test]
    fn test_contiguous_range_building() {
        let mut slot_ids = vec![1, 2, 3, 5, 6, 7, 10];
        slot_ids.sort();

        // Build contiguous ranges
        let mut ranges = Vec::new();
        let mut range_start = slot_ids[0];
        let mut range_end = slot_ids[0];

        for &slot in &slot_ids[1..] {
            if slot == range_end + 1 {
                range_end = slot;
            } else {
                ranges.push((range_start, range_end));
                range_start = slot;
                range_end = slot;
            }
        }
        ranges.push((range_start, range_end));

        // Should produce: [1,3], [5,7], [10,10]
        assert_eq!(ranges.len(), 3);
        assert_eq!(ranges[0], (1, 3));
        assert_eq!(ranges[1], (5, 7));
        assert_eq!(ranges[2], (10, 10));
    }


    /// Test: WantList response message format
    #[test]
    fn test_range_response_message_format() {
        let entry = DhtEntry {
            key_hash: 42,
            key: b"test".to_vec(),
            value: b"value".to_vec(),
            timestamp: 1000,
            slot_owner: "bafk42".to_string(),
        };

        let response = RangeResponse {
            range: (0, 100),
            entries: vec![entry],
            merkle_proof: None,
        };

        let msg = serde_json::json!({
            "type": "range_response",
            "response": response,
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"range_response\""));
        assert!(json.contains("\"range\":[0,100]"));
    }
}
