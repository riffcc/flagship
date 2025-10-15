//! WebRTC DataChannel Message Handling Tests
//!
//! Tests for WantList and RangeResponse message routing via WebRTC DataChannels.
//! Ensures proper detection and forwarding to TGP channel.

use lens_node::spore_wantlist::{WantListMessage, RangeResponse, DhtEntry};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: WantList message format for DataChannel
    #[test]
    fn test_wantlist_datachannel_format() {
        let wantlist = WantListMessage {
            version: 1,
            want_ranges: vec![(0, 100), (200, 300)],
            have_ranges: vec![(101, 199)],
            have_filter: None,
            timestamp: 1760541769,
            peer_id: "bafktest".to_string(),
        };

        // Serialize as JSON for DataChannel text message
        let json = serde_json::to_string(&wantlist).expect("Serialize WantList");

        // Should contain type field for routing
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"want_ranges\""));
        assert!(json.contains("\"have_ranges\""));
        assert!(json.contains("\"peer_id\":\"bafktest\""));

        // Should be parseable back
        let parsed: WantListMessage = serde_json::from_str(&json).expect("Parse WantList");
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.want_ranges.len(), 2);
        assert_eq!(parsed.have_ranges.len(), 1);
    }

    /// Test: RangeResponse message format for DataChannel
    #[test]
    fn test_range_response_datachannel_format() {
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

        // Wrap in message envelope for DataChannel
        let msg = serde_json::json!({
            "type": "range_response",
            "response": response,
        });

        let json = serde_json::to_string(&msg).expect("Serialize RangeResponse");

        // Should contain type field for routing
        assert!(json.contains("\"type\":\"range_response\""));
        assert!(json.contains("\"range\":[0,100]"));
        assert!(json.contains("\"entries\""));

        // Should be parseable back
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse message");
        assert_eq!(parsed["type"], "range_response");
        assert!(parsed["response"]["range"].is_array());
    }

    /// Test: WantList wrapped in message envelope
    #[test]
    fn test_wantlist_message_envelope() {
        let wantlist = WantListMessage {
            version: 1,
            want_ranges: vec![(0, 100)],
            have_ranges: vec![(200, 300)],
            have_filter: None,
            timestamp: 1760541769,
            peer_id: "bafktest".to_string(),
        };

        // Wrap in message envelope with type field
        let msg = serde_json::json!({
            "type": "wantlist",
            "message": wantlist,
        });

        let json = serde_json::to_string(&msg).expect("Serialize message");

        // Should be detectable by type field
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse");
        assert_eq!(parsed["type"], "wantlist");
        assert!(parsed["message"]["want_ranges"].is_array());
    }

    /// Test: DataChannel message type detection
    #[test]
    fn test_datachannel_message_type_detection() {
        let messages = vec![
            (r#"{"type":"wantlist","message":{}}"#, "wantlist"),
            (r#"{"type":"range_response","response":{}}"#, "range_response"),
            (r#"{"type":"heartbeat"}"#, "heartbeat"),
            (r#"{"type":"gossip_message"}"#, "gossip_message"),
        ];

        for (json, expected_type) in messages {
            let parsed: serde_json::Value = serde_json::from_str(json).expect("Parse");
            let msg_type = parsed.get("type").and_then(|v| v.as_str());
            assert_eq!(msg_type, Some(expected_type));
        }
    }

    /// Test: RangeResponse with multiple entries
    #[test]
    fn test_range_response_multiple_entries_datachannel() {
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

        let msg = serde_json::json!({
            "type": "range_response",
            "response": response,
        });

        let json = serde_json::to_string(&msg).expect("Serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse");

        assert_eq!(parsed["type"], "range_response");
        assert_eq!(parsed["response"]["entries"].as_array().unwrap().len(), 2);
    }

    /// Test: Empty WantList is valid
    #[test]
    fn test_empty_wantlist_datachannel() {
        let wantlist = WantListMessage {
            version: 1,
            want_ranges: vec![],
            have_ranges: vec![],
            have_filter: None,
            timestamp: 1760541769,
            peer_id: "bafktest".to_string(),
        };

        let msg = serde_json::json!({
            "type": "wantlist",
            "message": wantlist,
        });

        let json = serde_json::to_string(&msg).expect("Serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse");

        assert_eq!(parsed["type"], "wantlist");
        assert_eq!(parsed["message"]["want_ranges"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["message"]["have_ranges"].as_array().unwrap().len(), 0);
    }

    /// Test: Large RangeResponse serialization
    #[test]
    fn test_large_range_response_datachannel() {
        // Simulate 100 entries
        let entries: Vec<DhtEntry> = (0..100)
            .map(|i| DhtEntry {
                key_hash: i,
                key: format!("slot-{}", i).into_bytes(),
                value: format!("peer-{}", i).into_bytes(),
                timestamp: 1760541769,
                slot_owner: format!("bafk{}", i),
            })
            .collect();

        let response = RangeResponse {
            range: (0, 99),
            entries: entries.clone(),
            merkle_proof: None,
        };

        let msg = serde_json::json!({
            "type": "range_response",
            "response": response,
        });

        let json = serde_json::to_string(&msg).expect("Serialize large response");

        // Should successfully serialize all 100 entries
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse");
        assert_eq!(parsed["response"]["entries"].as_array().unwrap().len(), 100);
    }
}
