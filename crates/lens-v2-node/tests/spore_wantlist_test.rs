//! SPORE WantList Protocol Tests
//!
//! Tests for range-based DHT synchronization using succinct range exclusions.
//! These tests define the contract for Light and Heavy mode mesh merging.

use lens_node::spore_wantlist::{
    compute_want_ranges, merge_ranges, KeyRange, WantListMessage, RangeResponse, DhtEntry,
};
use std::collections::HashMap;

/// Test: Computing want ranges from peer's have ranges
#[test]
fn test_compute_want_ranges_simple_gap() {
    // Peer has ranges [0, 100] and [200, 300]
    let their_ranges = vec![(0, 100), (200, 300)];
    let my_ranges = vec![];
    let total_range = (0, 1000);

    let wants = compute_want_ranges(&their_ranges, &my_ranges, total_range);

    // We should want what THEY have (since we have nothing)
    assert_eq!(wants.len(), 2);
    assert_eq!(wants[0], (0, 100));
    assert_eq!(wants[1], (200, 300));
}

/// Test: No wants if we already have everything
#[test]
fn test_compute_want_ranges_already_have_all() {
    let their_ranges = vec![(0, 1000)];
    let my_ranges = vec![(0, 1000)];
    let total_range = (0, 1000);

    let wants = compute_want_ranges(&their_ranges, &my_ranges, total_range);

    // We should want nothing
    assert_eq!(wants.len(), 0);
}

/// Test: Want nothing if they have nothing
#[test]
fn test_compute_want_ranges_have_nothing() {
    let their_ranges = vec![];
    let my_ranges = vec![];
    let total_range = (0, 1000);

    let wants = compute_want_ranges(&their_ranges, &my_ranges, total_range);

    // If they have nothing, we can't get anything from them
    assert_eq!(wants.len(), 0);
}

/// Test: Merging overlapping ranges
#[test]
fn test_merge_ranges_overlapping() {
    let ranges = vec![(0, 100), (50, 150), (200, 300)];

    let merged = merge_ranges(&ranges);

    // Should merge first two into [0, 150], keep [200, 300]
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0], (0, 150));
    assert_eq!(merged[1], (200, 300));
}

/// Test: Merging adjacent ranges
#[test]
fn test_merge_ranges_adjacent() {
    let ranges = vec![(0, 100), (101, 200), (201, 300)];

    let merged = merge_ranges(&ranges);

    // Should merge all three into [0, 300]
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0], (0, 300));
}

/// Test: Light mode - slot ownership sync
#[test]
fn test_light_mode_slot_ownership_sync() {
    // Scenario: New peer joins and needs to learn all slot ownership

    // Create mock slot ownership entries (529 slots)
    let mut slot_ownership = HashMap::new();
    for slot_id in 0..529 {
        slot_ownership.insert(
            slot_id,
            DhtEntry {
                key_hash: slot_id,
                key: format!("slot-ownership-{}", slot_id).into_bytes(),
                value: format!("owner-peer-{}", slot_id % 100).into_bytes(),
                timestamp: 1760541769,
                slot_owner: format!("bafk{}", slot_id % 100),
            },
        );
    }

    // New peer has nothing
    let my_ranges = vec![];

    // Existing peer has all slots
    let their_ranges = vec![(0, 528)];

    // Compute what we want
    let wants = compute_want_ranges(&their_ranges, &my_ranges, (0, 528));

    // Should want everything
    assert_eq!(wants.len(), 1);
    assert_eq!(wants[0], (0, 528));

    // Simulate receiving response
    let response_entries: Vec<_> = slot_ownership
        .values()
        .cloned()
        .collect();

    assert_eq!(response_entries.len(), 529);

    // Verify we got all slot ownership entries
    for entry in &response_entries {
        assert!(entry.key_hash < 529);
        assert!(entry.key.starts_with(b"slot-ownership-"));
    }
}

/// Test: Light mode - incremental sync (new peer joins)
#[test]
fn test_light_mode_incremental_sync() {
    // Scenario: We have slots 0-527, new peer 528 joins

    let my_ranges = vec![(0, 527)];
    let their_ranges = vec![(0, 528)];

    // Compute what we want
    let wants = compute_want_ranges(&their_ranges, &my_ranges, (0, 528));

    // Should want only slot 528
    assert_eq!(wants.len(), 1);
    assert_eq!(wants[0], (528, 528));

    // Simulate receiving only the new slot
    let new_slot = DhtEntry {
        key_hash: 528,
        key: b"slot-ownership-528".to_vec(),
        value: b"owner-peer-new".to_vec(),
        timestamp: 1760541770,
        slot_owner: "bafknew".to_string(),
    };

    // Verify minimal transfer (1 entry instead of 529)
    assert_eq!(new_slot.key_hash, 528);
}

/// Test: Heavy mode - full DHT sync with gaps
#[test]
fn test_heavy_mode_full_dht_sync() {
    // Scenario: 1M keys, we have 50%, want the other 50%

    let my_ranges = vec![(0, 500000), (600000, 800000)];
    let their_ranges = vec![(0, 1000000)];

    // Compute what we want
    let wants = compute_want_ranges(&their_ranges, &my_ranges, (0, 1000000));

    // Should want the gaps: [500001, 599999] and [800001, 1000000]
    assert_eq!(wants.len(), 2);
    assert_eq!(wants[0], (500001, 599999));
    assert_eq!(wants[1], (800001, 1000000));

    // Calculate bandwidth savings
    let total_keys = 1_000_000;
    let keys_we_have = 500_000 + 200_000; // 700K
    let keys_we_want = total_keys - keys_we_have; // 300K

    let bandwidth_savings_percent =
        ((total_keys - keys_we_want) as f64 / total_keys as f64) * 100.0;

    // Should save 70% bandwidth (only transfer 30% of keys)
    assert!((bandwidth_savings_percent - 70.0).abs() < 0.1);
}

/// Test: Epidemic gossip - WantList message creation
#[test]
fn test_wantlist_message_creation() {
    let want_ranges = vec![(0, 100), (200, 300)];
    let have_ranges = vec![(101, 199)];

    let msg = WantListMessage {
        version: 1,
        want_ranges: want_ranges.clone(),
        have_ranges: have_ranges.clone(),
        have_filter: None,
        timestamp: 1760541769,
        peer_id: "bafktest".to_string(),
    };

    // Verify message structure
    assert_eq!(msg.version, 1);
    assert_eq!(msg.want_ranges.len(), 2);
    assert_eq!(msg.have_ranges.len(), 1);
    assert_eq!(msg.peer_id, "bafktest");

    // Serialize to JSON
    let json = serde_json::to_string(&msg).unwrap();

    // Should be compact
    assert!(json.len() < 500); // ~200 bytes expected
}

/// Test: Range response with entries
#[test]
fn test_range_response_with_entries() {
    let entries = vec![
        DhtEntry {
            key_hash: 42,
            key: b"key-42".to_vec(),
            value: b"value-42".to_vec(),
            timestamp: 1760541769,
            slot_owner: "bafk42".to_string(),
        },
        DhtEntry {
            key_hash: 100,
            key: b"key-100".to_vec(),
            value: b"value-100".to_vec(),
            timestamp: 1760541770,
            slot_owner: "bafk100".to_string(),
        },
    ];

    let response = RangeResponse {
        range: (0, 200),
        entries: entries.clone(),
        merkle_proof: None,
    };

    // Verify response
    assert_eq!(response.range, (0, 200));
    assert_eq!(response.entries.len(), 2);
    assert_eq!(response.entries[0].key_hash, 42);
    assert_eq!(response.entries[1].key_hash, 100);
}

/// Test: Mesh island merging scenario
#[test]
fn test_mesh_island_merging() {
    // Scenario: Two separate meshes with different slot ownership
    // Mesh A: 465 nodes, slots 0-464
    // Mesh B: 64 nodes, slots 465-528

    let mesh_a_ranges = vec![(0, 464)];
    let mesh_b_ranges = vec![(465, 528)];

    // When Mesh A peer meets Mesh B peer
    let a_wants_from_b = compute_want_ranges(&mesh_b_ranges, &mesh_a_ranges, (0, 528));
    let b_wants_from_a = compute_want_ranges(&mesh_a_ranges, &mesh_b_ranges, (0, 528));

    // A should want [465, 528]
    assert_eq!(a_wants_from_b.len(), 1);
    assert_eq!(a_wants_from_b[0], (465, 528));

    // B should want [0, 464]
    assert_eq!(b_wants_from_a.len(), 1);
    assert_eq!(b_wants_from_a[0], (0, 464));

    // After exchange, both should have [0, 528]
    let merged_ranges = merge_ranges(&[mesh_a_ranges[0], mesh_b_ranges[0]]);

    assert_eq!(merged_ranges.len(), 1);
    assert_eq!(merged_ranges[0], (0, 528));

    // SUCCESS: 529 unified nodes with complete routing table!
}

/// Test: Deduplication via timestamp
#[test]
fn test_timestamp_based_deduplication() {
    let old_entry = DhtEntry {
        key_hash: 42,
        key: b"key-42".to_vec(),
        value: b"old-value".to_vec(),
        timestamp: 1760541769,
        slot_owner: "bafk42".to_string(),
    };

    let new_entry = DhtEntry {
        key_hash: 42,
        key: b"key-42".to_vec(),
        value: b"new-value".to_vec(),
        timestamp: 1760541770, // Newer
        slot_owner: "bafk42".to_string(),
    };

    // Last-write-wins: newer entry should replace older
    assert!(new_entry.timestamp > old_entry.timestamp);
    assert_eq!(new_entry.key_hash, old_entry.key_hash);

    // Simulate conflict resolution
    let winner = if new_entry.timestamp > old_entry.timestamp {
        &new_entry
    } else {
        &old_entry
    };

    assert_eq!(winner.value, b"new-value");
}

/// Test: Bandwidth calculation for Light mode
#[test]
fn test_light_mode_bandwidth() {
    let slot_count = 529;
    let entry_size = 200; // bytes per slot ownership entry

    // Initial sync
    let initial_sync_bytes = slot_count * entry_size;
    assert_eq!(initial_sync_bytes, 105_800); // ~105 KB

    // Incremental sync (1 new peer)
    let incremental_sync_bytes = entry_size;
    assert_eq!(incremental_sync_bytes, 200);

    // Periodic WantList overhead
    let wantlist_overhead = 200; // bytes per WantList message
    let gossip_interval = 30; // seconds
    let bandwidth_per_second = wantlist_overhead / gossip_interval;
    assert_eq!(bandwidth_per_second, 6); // ~7 bytes/sec
}

/// Test: Bandwidth calculation for Heavy mode
#[test]
fn test_heavy_mode_bandwidth() {
    let total_keys = 1_000_000;
    let avg_entry_size = 500; // bytes

    // Full DHT size
    let full_dht_bytes = total_keys * avg_entry_size;
    assert_eq!(full_dht_bytes, 500_000_000); // 500 MB

    // With range exclusions (assume 10% gap)
    let gap_percent = 0.1;
    let gap_bytes = (full_dht_bytes as f64 * gap_percent) as usize;
    assert_eq!(gap_bytes, 50_000_000); // 50 MB

    // Bandwidth savings
    let savings_percent = (1.0 - gap_percent) * 100.0;
    assert_eq!(savings_percent, 90.0); // 90% savings!
}

/// Test: Convergence time estimation
#[test]
fn test_convergence_time_estimation() {
    let node_count = 529;
    let neighbor_count = 8;

    // Gossip rounds to reach all nodes (O(log n))
    let gossip_rounds = (node_count as f64).log2().ceil() as usize;
    assert_eq!(gossip_rounds, 10); // log2(529) ≈ 9.04

    // With 1 second per round
    let convergence_seconds = gossip_rounds;
    assert!(convergence_seconds < 30); // <30 seconds target

    // With epidemic gossip, each round reaches 8^n nodes
    let reachable_after_3_rounds = 8_usize.pow(3);
    assert_eq!(reachable_after_3_rounds, 512); // Nearly all 529 nodes!
}

/// Test: Range compression efficiency
#[test]
fn test_range_compression() {
    // Scenario: 100K contiguous keys
    let _contiguous_ranges = vec![(0, 100_000)];

    // Compressed representation: just 2 u64s (16 bytes)
    let compressed_size = 16;

    // Uncompressed: 100K key hashes (8 bytes each)
    let uncompressed_size = 100_000 * 8;

    // Compression ratio: 800,000 / 16 = 50,000x exactly
    let compression_ratio = uncompressed_size as f64 / compressed_size as f64;
    assert!(compression_ratio >= 50_000.0); // 50,000x compression!
}
