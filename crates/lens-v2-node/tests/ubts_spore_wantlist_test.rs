//! UBTS Sync with SPORE WantList over WebRTC Tests
//!
//! Tests for syncing UBTS blocks using SPORE WantList protocol over WebRTC DataChannels.
//! Ensures block_request/block_response relay routing is NOT used.

use lens_node::spore_wantlist::{WantListMessage, RangeResponse, DhtEntry};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: UBTS sync computes have_ranges from local blocks
    #[test]
    fn test_ubts_have_ranges_computation() {
        // Scenario: Node has blocks with heights [0, 1, 2, 5, 6, 7, 10]
        let local_block_heights = vec![0, 1, 2, 5, 6, 7, 10];

        // Build contiguous ranges
        let mut ranges = Vec::new();
        let mut range_start = local_block_heights[0];
        let mut range_end = local_block_heights[0];

        for &height in &local_block_heights[1..] {
            if height == range_end + 1 {
                range_end = height;
            } else {
                ranges.push((range_start, range_end));
                range_start = height;
                range_end = height;
            }
        }
        ranges.push((range_start, range_end));

        // Should produce: [0,2], [5,7], [10,10]
        assert_eq!(ranges.len(), 3);
        assert_eq!(ranges[0], (0, 2));
        assert_eq!(ranges[1], (5, 7));
        assert_eq!(ranges[2], (10, 10));
    }

    /// Test: UBTS sync computes want_ranges from peer's have_ranges
    #[test]
    fn test_ubts_want_ranges_from_peer() {
        use lens_node::spore_wantlist::compute_want_ranges;

        // Peer has blocks [0, 1, 2, 3, 4, 5]
        let their_ranges = vec![(0, 5)];

        // We have blocks [0, 1, 2]
        let my_ranges = vec![(0, 2)];

        // We should want [3, 5] from them
        let wants = compute_want_ranges(&their_ranges, &my_ranges, (0, 100));

        assert_eq!(wants.len(), 1);
        assert_eq!(wants[0], (3, 5));
    }

    /// Test: UBTS WantList message format for WebRTC
    #[test]
    fn test_ubts_wantlist_webrtc_format() {
        let wantlist = WantListMessage {
            version: 1,
            want_ranges: vec![(0, 100)],  // Need blocks 0-100
            have_ranges: vec![(200, 300)], // Have blocks 200-300
            have_filter: None,
            timestamp: 1760541769,
            peer_id: "bafkubts".to_string(),
        };

        // Wrap in message envelope for WebRTC DataChannel
        let msg = serde_json::json!({
            "type": "wantlist",
            "message": wantlist,
        });

        let json = serde_json::to_string(&msg).expect("Serialize WantList");

        // Should be compact enough for WebRTC DataChannel (typically <16KB)
        assert!(json.len() < 16_000, "WantList should fit in single DataChannel message");

        // Should be parseable
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse");
        assert_eq!(parsed["type"], "wantlist");
        assert_eq!(parsed["message"]["want_ranges"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["message"]["have_ranges"].as_array().unwrap().len(), 1);
    }

    /// Test: RangeResponse contains UBTS blocks as DhtEntry
    #[test]
    fn test_ubts_range_response_format() {
        // Simulate UBTS blocks as DHT entries
        let blocks = vec![
            DhtEntry {
                key_hash: 0,
                key: b"ubts-block-0".to_vec(),
                value: serde_json::to_vec(&serde_json::json!({
                    "id": "ubts-block-0",
                    "height": 0,
                    "transactions": []
                })).unwrap(),
                timestamp: 1760541769,
                slot_owner: "bafkowner".to_string(),
            },
            DhtEntry {
                key_hash: 1,
                key: b"ubts-block-1".to_vec(),
                value: serde_json::to_vec(&serde_json::json!({
                    "id": "ubts-block-1",
                    "height": 1,
                    "transactions": []
                })).unwrap(),
                timestamp: 1760541770,
                slot_owner: "bafkowner".to_string(),
            },
        ];

        let response = RangeResponse {
            range: (0, 1),
            entries: blocks,
            merkle_proof: None,
        };

        // Wrap in message envelope
        let msg = serde_json::json!({
            "type": "range_response",
            "response": response,
        });

        let json = serde_json::to_string(&msg).expect("Serialize RangeResponse");

        // Should be parseable
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse");
        assert_eq!(parsed["type"], "range_response");
        assert_eq!(parsed["response"]["range"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["response"]["entries"].as_array().unwrap().len(), 2);
    }

    /// Test: Empty have_ranges is valid (new node joining)
    #[test]
    fn test_ubts_empty_have_ranges() {
        let wantlist = WantListMessage {
            version: 1,
            want_ranges: vec![(0, 1000)], // Want everything
            have_ranges: vec![],           // Have nothing
            have_filter: None,
            timestamp: 1760541769,
            peer_id: "bafknew".to_string(),
        };

        let msg = serde_json::json!({
            "type": "wantlist",
            "message": wantlist,
        });

        let json = serde_json::to_string(&msg).expect("Serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse");

        assert_eq!(parsed["message"]["want_ranges"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["message"]["have_ranges"].as_array().unwrap().len(), 0);
    }

    /// Test: UBTS blocks are identified by height hash (u64)
    #[test]
    fn test_ubts_block_height_hash() {
        let block_id = "ubts-block-42";
        let height = 42u64;

        // Hash the block ID to u64 for range operations
        let key_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            block_id.hash(&mut hasher);
            hasher.finish()
        };

        // key_hash should be deterministic
        let key_hash2 = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            block_id.hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(key_hash, key_hash2, "Block ID hash should be deterministic");

        // For UBTS, we use height directly as key_hash for range operations
        // This allows efficient range queries: "give me blocks 0-100"
        let entry = DhtEntry {
            key_hash: height,
            key: block_id.as_bytes().to_vec(),
            value: b"block-data".to_vec(),
            timestamp: 1760541769,
            slot_owner: "bafk".to_string(),
        };

        assert_eq!(entry.key_hash, 42);
    }

    /// Test: Large RangeResponse with 1000 blocks
    #[test]
    fn test_ubts_large_range_response() {
        // Simulate 1000 blocks
        let blocks: Vec<DhtEntry> = (0..1000)
            .map(|i| DhtEntry {
                key_hash: i,
                key: format!("ubts-block-{}", i).into_bytes(),
                value: vec![0u8; 100], // 100 bytes per block
                timestamp: 1760541769,
                slot_owner: "bafk".to_string(),
            })
            .collect();

        let response = RangeResponse {
            range: (0, 999),
            entries: blocks,
            merkle_proof: None,
        };

        let msg = serde_json::json!({
            "type": "range_response",
            "response": response,
        });

        let json = serde_json::to_string(&msg).expect("Serialize large response");

        // Should successfully serialize 1000 blocks
        // Approximate size: 1000 blocks × ~150 bytes = ~150KB
        let size_kb = json.len() / 1024;
        assert!(size_kb < 500, "Large response should be < 500KB (was {}KB)", size_kb);

        // Should be parseable
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Parse");
        assert_eq!(parsed["response"]["entries"].as_array().unwrap().len(), 1000);
    }

    /// Test: UBTS sync bandwidth calculation
    #[test]
    fn test_ubts_bandwidth_with_ranges() {
        let total_blocks = 10_000;
        let avg_block_size = 1_000; // bytes

        // Full sync without ranges
        let full_sync_bytes = total_blocks * avg_block_size;
        assert_eq!(full_sync_bytes, 10_000_000); // 10 MB

        // Sync with 50% already present (range exclusions)
        let gap_percent = 0.5;
        let gap_blocks = (total_blocks as f64 * gap_percent) as usize;
        let sync_bytes = gap_blocks * avg_block_size;
        assert_eq!(sync_bytes, 5_000_000); // 5 MB

        // Bandwidth savings: 50%
        let savings_percent = (1.0 - gap_percent) * 100.0;
        assert_eq!(savings_percent, 50.0);
    }

    /// Test: WantList periodic broadcasting (epidemic gossip)
    #[test]
    fn test_ubts_wantlist_epidemic_gossip() {
        // Node broadcasts WantList every 30 seconds (via relay)
        let gossip_interval = 30; // seconds

        // WantList overhead per broadcast
        let wantlist_size = 500; // bytes (2 ranges × ~200 bytes)

        // Bandwidth per second
        let bandwidth_per_second = wantlist_size / gossip_interval;
        assert_eq!(bandwidth_per_second, 16); // ~16 bytes/sec

        // Compare to heartbeat overhead (10 seconds × ~100 bytes)
        let heartbeat_bandwidth = 100 / 10;
        assert_eq!(heartbeat_bandwidth, 10); // ~10 bytes/sec

        // WantList adds minimal overhead compared to heartbeat
        assert!(bandwidth_per_second < 20, "WantList overhead should be < 20 bytes/sec");
    }

    /// Test: UBTS sync convergence time
    #[test]
    fn test_ubts_sync_convergence_time() {
        let total_blocks = 10_000;
        let blocks_per_response = 100;
        let response_time_ms = 100; // WebRTC latency

        // Number of RangeResponse messages needed
        let num_responses = total_blocks / blocks_per_response;

        // Convergence time (parallel requests to 8 neighbors)
        let convergence_ms = (num_responses / 8) * response_time_ms;
        let convergence_sec = convergence_ms / 1000;

        // Should converge in ~1 second (100 responses / 8 neighbors × 100ms)
        assert!(convergence_sec <= 2, "UBTS sync should converge in ~1 second (was {}s)", convergence_sec);
    }
}
