//! DHT Bootstrap and Merge Tests
//!
//! Tests for bootstrapping the GLOBAL DHT from peers and merging multiple DHT views.
//!
//! Per Citadel spec section 2.4: "Bootstrap from ANY node in network"
//! Key scenarios:
//! 1. First two peers exchange DHT state to bootstrap together
//! 2. Subsequent peers bootstrap from existing DHT
//! 3. Multiple isolated DHTs safely merge when networks reconnect

// Import the actual implementation from the lens-node crate
use lens_node::dht_state::{DhtEntry, DhtState};

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(byte: u8) -> [u8; 32] {
        let mut key = [0u8; 32];
        key[0] = byte;
        key
    }

    fn make_entry(byte: u8, value: &[u8], timestamp: u64) -> DhtEntry {
        DhtEntry {
            key: make_key(byte),
            value: value.to_vec(),
            timestamp,
        }
    }

    #[test]
    fn test_bootstrap_handshake_between_two_peers() {
        // Scenario: First two peers in network bootstrap together

        // Peer A joins first (empty DHT)
        let mut peer_a = DhtState::new();
        assert_eq!(peer_a.len(), 0, "Peer A starts with empty DHT");

        // Peer A announces its slot ownership
        peer_a.insert(make_entry(1, b"peer_a_slot", 1000));
        assert_eq!(peer_a.len(), 1, "Peer A has its own announcement");

        // Peer B joins second (empty DHT)
        let mut peer_b = DhtState::new();
        assert_eq!(peer_b.len(), 0, "Peer B starts with empty DHT");

        // Peer B announces its slot ownership
        peer_b.insert(make_entry(2, b"peer_b_slot", 1001));

        // Bootstrap handshake: A and B exchange DHT state
        peer_a.bootstrap_from_peer(&peer_b);
        peer_b.bootstrap_from_peer(&peer_a);

        // Both peers should now have complete DHT with 2 entries
        assert_eq!(peer_a.len(), 2, "Peer A now knows about Peer B");
        assert_eq!(peer_b.len(), 2, "Peer B now knows about Peer A");

        assert!(peer_a.get(&make_key(2)).is_some(), "Peer A has Peer B's slot");
        assert!(peer_b.get(&make_key(1)).is_some(), "Peer B has Peer A's slot");

        // Both should have identical DHT state
        assert_eq!(
            peer_a.to_sorted_vec(),
            peer_b.to_sorted_vec(),
            "Both peers have identical DHT after bootstrap"
        );
    }

    #[test]
    fn test_third_peer_bootstraps_from_existing_dht() {
        // Scenario: Peer C joins network where A and B already have DHT

        // Existing network: Peers A and B with established DHT
        let mut peer_a = DhtState::new();
        peer_a.insert(make_entry(1, b"peer_a_slot", 1000));
        peer_a.insert(make_entry(2, b"peer_b_slot", 1001));

        let mut peer_b = DhtState::new();
        peer_b.insert(make_entry(1, b"peer_a_slot", 1000));
        peer_b.insert(make_entry(2, b"peer_b_slot", 1001));

        // New peer C joins
        let mut peer_c = DhtState::new();
        peer_c.insert(make_entry(3, b"peer_c_slot", 1002));

        // Peer C bootstraps from Peer A (could be any existing peer)
        peer_c.bootstrap_from_peer(&peer_a);

        // Peer C should now have all 3 entries
        assert_eq!(peer_c.len(), 3, "Peer C has A, B, and C's entries");
        assert!(peer_c.get(&make_key(1)).is_some(), "Peer C knows Peer A");
        assert!(peer_c.get(&make_key(2)).is_some(), "Peer C knows Peer B");
        assert!(peer_c.get(&make_key(3)).is_some(), "Peer C has its own entry");

        // A and B need to learn about C via gossip (separate test)
    }

    #[test]
    fn test_dht_merge_no_conflicts() {
        // Scenario: Two isolated DHTs merge with no overlapping keys

        let mut dht_a = DhtState::new();
        dht_a.insert(make_entry(1, b"peer_1", 1000));
        dht_a.insert(make_entry(2, b"peer_2", 1001));

        let mut dht_b = DhtState::new();
        dht_b.insert(make_entry(3, b"peer_3", 1002));
        dht_b.insert(make_entry(4, b"peer_4", 1003));

        // Merge B into A
        dht_a.merge(&dht_b);

        // A should now have all 4 entries
        assert_eq!(dht_a.len(), 4, "DHT A has all entries after merge");
        assert!(dht_a.get(&make_key(1)).is_some());
        assert!(dht_a.get(&make_key(2)).is_some());
        assert!(dht_a.get(&make_key(3)).is_some());
        assert!(dht_a.get(&make_key(4)).is_some());
    }

    #[test]
    fn test_dht_merge_with_conflicts_last_write_wins() {
        // Scenario: Two DHTs have conflicting values for same key
        // Latest timestamp wins

        let mut dht_a = DhtState::new();
        dht_a.insert(make_entry(1, b"old_value", 1000)); // Older timestamp
        dht_a.insert(make_entry(2, b"unique_to_a", 1001));

        let mut dht_b = DhtState::new();
        dht_b.insert(make_entry(1, b"new_value", 2000)); // Newer timestamp - should win
        dht_b.insert(make_entry(3, b"unique_to_b", 1002));

        // Merge B into A
        dht_a.merge(&dht_b);

        // Check conflict resolution
        assert_eq!(dht_a.len(), 3, "DHT A has 3 unique keys");

        let entry_1 = dht_a.get(&make_key(1)).expect("Key 1 exists");
        assert_eq!(entry_1.value, b"new_value", "Newer timestamp won conflict");
        assert_eq!(entry_1.timestamp, 2000, "Timestamp is from newer entry");

        assert_eq!(dht_a.get(&make_key(2)).unwrap().value, b"unique_to_a");
        assert_eq!(dht_a.get(&make_key(3)).unwrap().value, b"unique_to_b");
    }

    #[test]
    fn test_dht_merge_multiple_partitions() {
        // Scenario: 3 isolated DHTs (network partition) need to merge
        // Simulates: 3 separate groups of nodes formed independent DHTs

        let mut dht_partition_1 = DhtState::new();
        dht_partition_1.insert(make_entry(1, b"group1_peer1", 1000));
        dht_partition_1.insert(make_entry(2, b"group1_peer2", 1001));

        let mut dht_partition_2 = DhtState::new();
        dht_partition_2.insert(make_entry(3, b"group2_peer3", 1002));
        dht_partition_2.insert(make_entry(4, b"group2_peer4", 1003));

        let mut dht_partition_3 = DhtState::new();
        dht_partition_3.insert(make_entry(5, b"group3_peer5", 1004));
        dht_partition_3.insert(make_entry(6, b"group3_peer6", 1005));

        // Merge all partitions into partition 1
        dht_partition_1.merge(&dht_partition_2);
        dht_partition_1.merge(&dht_partition_3);

        // Final DHT should have all 6 entries
        assert_eq!(dht_partition_1.len(), 6, "All partitions merged");
        assert!(dht_partition_1.get(&make_key(1)).is_some());
        assert!(dht_partition_1.get(&make_key(2)).is_some());
        assert!(dht_partition_1.get(&make_key(3)).is_some());
        assert!(dht_partition_1.get(&make_key(4)).is_some());
        assert!(dht_partition_1.get(&make_key(5)).is_some());
        assert!(dht_partition_1.get(&make_key(6)).is_some());
    }

    #[test]
    fn test_dht_merge_preserves_newer_local_entries() {
        // Scenario: Our DHT has newer data than incoming merge
        // Make sure we don't regress to older data

        let mut our_dht = DhtState::new();
        our_dht.insert(make_entry(1, b"latest_value", 3000)); // Newer
        our_dht.insert(make_entry(2, b"our_data", 2000));

        let other_dht = DhtState::new();
        // Other DHT has stale data for key 1
        let mut temp = other_dht.clone();
        temp.insert(make_entry(1, b"stale_value", 1000)); // Older - should NOT win
        temp.insert(make_entry(3, b"other_data", 2001));

        // Merge other into ours
        our_dht.merge(&temp);

        // Our newer value should be preserved
        assert_eq!(our_dht.len(), 3);
        let entry_1 = our_dht.get(&make_key(1)).unwrap();
        assert_eq!(entry_1.value, b"latest_value", "Newer local value preserved");
        assert_eq!(entry_1.timestamp, 3000, "Newer timestamp preserved");
    }

    #[test]
    fn test_bootstrap_does_not_overwrite_existing_entries() {
        // Scenario: Node already has some DHT entries (from gossip)
        // Bootstrap should only ADD missing entries, not overwrite

        let mut our_dht = DhtState::new();
        our_dht.insert(make_entry(1, b"our_existing", 2000));
        our_dht.insert(make_entry(2, b"our_data", 2001));

        let peer_dht = DhtState::new();
        let mut temp = peer_dht.clone();
        temp.insert(make_entry(1, b"peer_version", 1000)); // Older version
        temp.insert(make_entry(3, b"new_peer", 2002)); // New entry we don't have

        // Bootstrap from peer
        our_dht.bootstrap_from_peer(&temp);

        // Our existing entries should NOT be overwritten
        assert_eq!(our_dht.len(), 3);
        let entry_1 = our_dht.get(&make_key(1)).unwrap();
        assert_eq!(entry_1.value, b"our_existing", "Bootstrap doesn't overwrite existing");
        assert_eq!(entry_1.timestamp, 2000);

        // But we should have the new entry
        assert!(our_dht.get(&make_key(3)).is_some(), "Bootstrap added missing entry");
    }
}
