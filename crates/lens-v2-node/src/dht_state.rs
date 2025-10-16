//! DHT State Management - Bootstrap and Merge
//!
//! Implements the GLOBAL DHT with bootstrap and merge capabilities.
//! Per Citadel spec section 2.4: "Bootstrap from ANY node in network"

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// DHT entry with timestamp for conflict resolution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DhtEntry {
    pub key: [u8; 32],
    pub value: Vec<u8>,
    pub timestamp: u64,
}

/// DHT state that can be bootstrapped and merged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtState {
    pub entries: HashMap<[u8; 32], DhtEntry>,
}

impl DhtState {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entry: DhtEntry) {
        self.entries.insert(entry.key, entry);
    }

    pub fn get(&self, key: &[u8; 32]) -> Option<&DhtEntry> {
        self.entries.get(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Insert raw key-value pair (for compatibility with existing code)
    pub fn insert_raw(&mut self, key: [u8; 32], value: Vec<u8>) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.insert(DhtEntry { key, value, timestamp });
    }

    /// Get raw value bytes (for compatibility)
    pub fn get_raw(&self, key: &[u8; 32]) -> Option<&Vec<u8>> {
        self.entries.get(key).map(|entry| &entry.value)
    }

    /// Remove an entry by key
    pub fn remove(&mut self, key: &[u8; 32]) -> Option<DhtEntry> {
        self.entries.remove(key)
    }

    /// Get iterator over keys
    pub fn keys(&self) -> impl Iterator<Item = &[u8; 32]> {
        self.entries.keys()
    }

    /// Get iterator over entries
    pub fn iter(&self) -> impl Iterator<Item = (&[u8; 32], &DhtEntry)> {
        self.entries.iter()
    }

    /// Bootstrap: Receive DHT state from a peer and populate local cache
    /// Only adds missing entries, does NOT overwrite existing ones
    pub fn bootstrap_from_peer(&mut self, peer_state: &DhtState) {
        for (key, entry) in &peer_state.entries {
            // Only insert if we don't have this key yet (bootstrap scenario)
            if !self.entries.contains_key(key) {
                self.entries.insert(*key, entry.clone());
            }
        }
    }

    /// Merge: Combine two DHT states, using timestamp for conflict resolution
    /// Latest timestamp wins (last-write-wins strategy)
    pub fn merge(&mut self, other: &DhtState) {
        for (key, other_entry) in &other.entries {
            match self.entries.get(key) {
                Some(our_entry) => {
                    // Conflict: both have this key - use timestamp to resolve
                    if other_entry.timestamp > our_entry.timestamp {
                        // Other entry is newer, replace ours
                        self.entries.insert(*key, other_entry.clone());
                    }
                    // If our entry is newer or equal, keep it (do nothing)
                }
                None => {
                    // No conflict: we don't have this key, add it
                    self.entries.insert(*key, other_entry.clone());
                }
            }
        }
    }

    /// Get all entries as a sorted vec (for consistent hashing/comparison)
    pub fn to_sorted_vec(&self) -> Vec<DhtEntry> {
        let mut entries: Vec<DhtEntry> = self.entries.values().cloned().collect();
        entries.sort_by_key(|e| e.key);
        entries
    }

    /// Scan all entries in DHT (for map endpoint to discover all peers)
    /// Returns all key-value pairs as Vec<([u8; 32], Vec<u8>)>
    pub fn scan_all(&self) -> Vec<([u8; 32], Vec<u8>)> {
        self.entries
            .iter()
            .map(|(key, entry)| (*key, entry.value.clone()))
            .collect()
    }
}

impl Default for DhtState {
    fn default() -> Self {
        Self::new()
    }
}
