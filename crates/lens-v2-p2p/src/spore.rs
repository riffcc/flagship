//! SPORE - Succinct Proof of Range Exclusions
//!
//! Compact representation of which blocks a node has, allowing efficient
//! discovery of missing blocks through XOR operations.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// SPORE - Succinct Proof of Range Exclusions
///
/// A compact bitmap representing which blocks a node has.
/// Nodes XOR SPOREs to identify missing blocks automatically.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Spore {
    /// Bitmap of block hashes (for small sets)
    /// Each bit represents presence/absence of a block
    pub bitmap: Vec<u8>,

    /// Height range this SPORE covers
    pub min_height: u64,
    pub max_height: u64,

    /// Block IDs included in this SPORE
    /// This allows recipients to identify exactly which blocks are missing
    pub block_ids: Vec<String>,
}

impl Spore {
    /// Create a new empty SPORE
    pub fn new() -> Self {
        Self {
            bitmap: Vec::new(),
            min_height: 0,
            max_height: 0,
            block_ids: Vec::new(),
        }
    }

    /// Create SPORE from list of block IDs
    pub fn from_blocks(blocks: &[String]) -> Self {
        let mut spore = Self::new();

        if blocks.is_empty() {
            return spore;
        }

        // Store block IDs for reconstruction
        spore.block_ids = blocks.to_vec();

        // For now, use a simple hash-based bitmap
        // Each block ID gets hashed to a bit position
        let bitmap_size = (blocks.len() / 8).max(32); // Min 32 bytes (256 bits)
        spore.bitmap = vec![0u8; bitmap_size];

        for block_id in blocks {
            let bit_index = spore.hash_to_bit(block_id);
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;

            if byte_index < spore.bitmap.len() {
                spore.bitmap[byte_index] |= 1 << bit_offset;
            }
        }

        spore
    }

    /// Hash a block ID to a bit index
    fn hash_to_bit(&self, block_id: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        block_id.hash(&mut hasher);
        let hash = hasher.finish();

        (hash as usize) % (self.bitmap.len() * 8)
    }

    /// XOR this SPORE with another to find potentially missing blocks
    ///
    /// Returns a bitmap where 1 indicates blocks the other node might have
    /// that we don't have.
    pub fn xor(&self, other: &Spore) -> Vec<u8> {
        let max_len = self.bitmap.len().max(other.bitmap.len());
        let mut result = vec![0u8; max_len];

        for i in 0..max_len {
            let self_byte = self.bitmap.get(i).copied().unwrap_or(0);
            let other_byte = other.bitmap.get(i).copied().unwrap_or(0);
            result[i] = self_byte ^ other_byte;
        }

        result
    }

    /// Identify potentially missing blocks by comparing with another SPORE
    ///
    /// Returns bit indices where the other node has blocks we might not have.
    pub fn find_missing(&self, other: &Spore) -> Vec<usize> {
        let xor_result = self.xor(other);
        let mut missing_bits = Vec::new();

        for (byte_index, &byte) in xor_result.iter().enumerate() {
            if byte == 0 {
                continue;
            }

            for bit_offset in 0..8 {
                if (byte & (1 << bit_offset)) != 0 {
                    // This bit differs - other node might have a block we don't
                    let bit_index = byte_index * 8 + bit_offset;
                    missing_bits.push(bit_index);
                }
            }
        }

        missing_bits
    }

    /// Identify missing block IDs by comparing with another SPORE
    ///
    /// Returns the actual block IDs that the other node has but we don't.
    pub fn find_missing_blocks(&self, other: &Spore) -> Vec<String> {
        let our_blocks: HashSet<&String> = self.block_ids.iter().collect();

        other.block_ids.iter()
            .filter(|block_id| !our_blocks.contains(block_id))
            .cloned()
            .collect()
    }

    /// Get compact byte representation for network transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Reconstruct SPORE from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }

    /// Check if SPORE likely contains a block
    pub fn probably_has(&self, block_id: &str) -> bool {
        if self.bitmap.is_empty() {
            return false;
        }

        let bit_index = self.hash_to_bit(block_id);
        let byte_index = bit_index / 8;
        let bit_offset = bit_index % 8;

        if byte_index >= self.bitmap.len() {
            return false;
        }

        (self.bitmap[byte_index] & (1 << bit_offset)) != 0
    }

    /// Get bitmap size in bytes
    pub fn size(&self) -> usize {
        self.bitmap.len()
    }

    /// Check if SPORE is empty
    pub fn is_empty(&self) -> bool {
        self.bitmap.iter().all(|&b| b == 0)
    }
}

impl Default for Spore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spore_creation() {
        let blocks = vec!["block1".to_string(), "block2".to_string(), "block3".to_string()];
        let spore = Spore::from_blocks(&blocks);

        assert!(!spore.is_empty());
        assert!(spore.bitmap.len() >= 32);
    }

    #[test]
    fn test_spore_xor() {
        let blocks1 = vec!["block1".to_string(), "block2".to_string()];
        let blocks2 = vec!["block2".to_string(), "block3".to_string()];

        let spore1 = Spore::from_blocks(&blocks1);
        let spore2 = Spore::from_blocks(&blocks2);

        let xor_result = spore1.xor(&spore2);

        // XOR should show differences
        assert!(xor_result.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_spore_find_missing() {
        let blocks1 = vec!["block1".to_string(), "block2".to_string()];
        let blocks2 = vec!["block1".to_string(), "block2".to_string(), "block3".to_string()];

        let spore1 = Spore::from_blocks(&blocks1);
        let spore2 = Spore::from_blocks(&blocks2);

        let missing = spore1.find_missing(&spore2);

        // spore1 should detect that spore2 has additional blocks
        assert!(!missing.is_empty());
    }

    #[test]
    fn test_spore_probably_has() {
        let blocks = vec!["block1".to_string(), "block2".to_string()];
        let spore = Spore::from_blocks(&blocks);

        // Should probably have blocks we added
        assert!(spore.probably_has("block1"));
        assert!(spore.probably_has("block2"));
    }

    #[test]
    fn test_spore_serialization() {
        let blocks = vec!["block1".to_string(), "block2".to_string()];
        let spore = Spore::from_blocks(&blocks);

        let bytes = spore.to_bytes();
        let restored = Spore::from_bytes(&bytes).unwrap();

        assert_eq!(spore.bitmap, restored.bitmap);
    }

    #[test]
    fn test_empty_spore() {
        let spore = Spore::new();
        assert!(spore.is_empty());
        assert_eq!(spore.size(), 0);
    }

    #[test]
    fn test_identical_spores_no_diff() {
        let blocks = vec!["block1".to_string(), "block2".to_string()];
        let spore1 = Spore::from_blocks(&blocks);
        let spore2 = Spore::from_blocks(&blocks);

        let missing = spore1.find_missing(&spore2);

        // Identical SPOREs should have no differences
        assert!(missing.is_empty());
    }
}
