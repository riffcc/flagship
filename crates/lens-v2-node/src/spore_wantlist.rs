//! SPORE WantList Protocol - Range-Based DHT Synchronization
//!
//! Implements efficient DHT sync using succinct proofs of range exclusions.
//! Peers exchange compact range representations and transfer only the gaps.

use serde::{Deserialize, Serialize};

/// Key range (inclusive): (start, end)
pub type KeyRange = (u64, u64);

/// WantList message for epidemic gossip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantListMessage {
    /// Protocol version
    pub version: u32,

    /// Ranges we DON'T have (requesting these)
    pub want_ranges: Vec<KeyRange>,

    /// Ranges we DO have (offering these)
    pub have_ranges: Vec<KeyRange>,

    /// Optional Bloom filter for fast membership testing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub have_filter: Option<BloomFilter>,

    /// Timestamp for freshness
    pub timestamp: u64,

    /// Peer ID of sender
    pub peer_id: String,
}

/// Response containing requested range data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeResponse {
    /// The range being satisfied
    pub range: KeyRange,

    /// DHT entries within this range
    pub entries: Vec<DhtEntry>,

    /// Optional: Merkle proof for verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merkle_proof: Option<MerkleProof>,
}

/// DHT entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtEntry {
    /// Blake3 hash of the key (u64 for range operations)
    pub key_hash: u64,

    /// Full key bytes
    pub key: Vec<u8>,

    /// Value bytes
    pub value: Vec<u8>,

    /// Timestamp for conflict resolution
    pub timestamp: u64,

    /// Slot owner (for verification)
    pub slot_owner: String,
}

/// Placeholder for Bloom filter (TODO: implement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomFilter {
    // TODO: Implement actual Bloom filter
    _placeholder: u8,
}

/// Placeholder for Merkle proof (TODO: implement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    // TODO: Implement actual Merkle proof
    _placeholder: u8,
}

/// Compute ranges we want from a specific peer
///
/// Given:
/// - `their_ranges`: Ranges the remote peer has
/// - `my_ranges`: Ranges we already have
/// - `total_range`: The full range we want to eventually have
///
/// Returns: Parts that THEY have and WE don't (intersection of their coverage and our gaps)
///
/// Protocol: Compute what parts of their coverage can fill our gaps.
/// This enables us to request specific ranges from peers who can fulfill them.
pub fn compute_want_ranges(
    their_ranges: &[KeyRange],
    my_ranges: &[KeyRange],
    total_range: KeyRange,
) -> Vec<KeyRange> {
    // If they have nothing, we want nothing from them
    if their_ranges.is_empty() {
        return vec![];
    }

    // Merge their ranges
    let their_merged = merge_ranges(their_ranges);

    // If we have nothing, we want everything they have (within total_range)
    if my_ranges.is_empty() {
        return their_merged;
    }

    // Merge our ranges
    let my_merged = merge_ranges(my_ranges);

    // Compute what they have that we don't (intersection)
    let mut wants = Vec::new();

    for &(their_start, their_end) in &their_merged {
        let mut cursor = their_start;

        // Walk through our ranges and find gaps in their range
        for &(my_start, my_end) in &my_merged {
            // Our range is entirely before cursor, skip it
            if my_end < cursor {
                continue;
            }

            // Our range starts after their range ends, done with this their_range
            if my_start > their_end {
                break;
            }

            // Gap before our range? We want it!
            if cursor < my_start {
                let gap_end = my_start - 1;
                wants.push((cursor, gap_end.min(their_end)));
            }

            // Move cursor past our range
            cursor = (my_end + 1).max(cursor);

            if cursor > their_end {
                break;
            }
        }

        // Remaining part of their range after all our ranges
        if cursor <= their_end {
            wants.push((cursor, their_end));
        }
    }

    wants
}

/// Merge overlapping or adjacent ranges
///
/// Takes a list of ranges and combines any that overlap or are adjacent.
/// Returns a minimal set of non-overlapping ranges.
pub fn merge_ranges(ranges: &[KeyRange]) -> Vec<KeyRange> {
    if ranges.is_empty() {
        return vec![];
    }

    // Sort by start position
    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|r| r.0);

    let mut merged = vec![sorted[0]];

    for &(start, end) in &sorted[1..] {
        let last_idx = merged.len() - 1;
        let (last_start, last_end) = merged[last_idx];

        // Overlapping or adjacent? (end + 1 == start means adjacent)
        if start <= last_end + 1 {
            // Merge by extending last range to max of both ends
            merged[last_idx] = (last_start, end.max(last_end));
        } else {
            // Separate range, add as new
            merged.push((start, end));
        }
    }

    merged
}

/// Build contiguous ranges from sorted key hashes
///
/// Takes a sorted list of u64 key hashes and builds contiguous ranges.
/// Adjacent keys are merged into a single range for efficient representation.
pub fn build_ranges_from_keys(sorted_keys: &[u64]) -> Vec<KeyRange> {
    if sorted_keys.is_empty() {
        return vec![];
    }

    let mut ranges = Vec::new();
    let mut range_start = sorted_keys[0];
    let mut range_end = sorted_keys[0];

    for &key in &sorted_keys[1..] {
        if key == range_end + 1 {
            // Adjacent key, extend range
            range_end = key;
        } else {
            // Gap found, save current range and start new one
            ranges.push((range_start, range_end));
            range_start = key;
            range_end = key;
        }
    }

    // Don't forget the last range
    ranges.push((range_start, range_end));

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_want_ranges_simple() {
        let their_ranges = vec![(0, 100), (200, 300)];
        let my_ranges = vec![];
        let total_range = (0, 1000);

        let wants = compute_want_ranges(&their_ranges, &my_ranges, total_range);

        assert_eq!(wants.len(), 2);
        assert_eq!(wants[0], (101, 199));
        assert_eq!(wants[1], (301, 1000));
    }

    #[test]
    fn test_merge_ranges_basic() {
        let ranges = vec![(0, 100), (50, 150), (200, 300)];
        let merged = merge_ranges(&ranges);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], (0, 150));
        assert_eq!(merged[1], (200, 300));
    }

    #[test]
    fn test_merge_ranges_adjacent() {
        let ranges = vec![(0, 100), (101, 200)];
        let merged = merge_ranges(&ranges);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], (0, 200));
    }
}
