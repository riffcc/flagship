//! Block Codec
//!
//! Handles serialization and deserialization between Release data and P2P blocks.

use anyhow::{Result, Context};
use lens_v2_p2p::network::BlockData;
use lens_v2_p2p::BlockMeta;
use serde::{Deserialize, Serialize};

use crate::routes::releases::Release;

/// Block envelope wrapping release data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEnvelope {
    /// Block metadata
    pub meta: BlockMeta,

    /// Release data (can be multiple releases in a block)
    pub releases: Vec<Release>,

    /// Featured release IDs (for featured list changes)
    pub featured: Vec<String>,

    /// Block signature (future: cryptographic proof)
    pub signature: Option<String>,
}

impl BlockEnvelope {
    /// Create a new block envelope from releases
    pub fn new(releases: Vec<Release>, height: u64) -> Self {
        let block_id = Self::compute_block_id(&releases, height);

        Self {
            meta: BlockMeta {
                id: block_id.clone(),
                height,
                prev: None, // Set by caller
                timestamp: chrono::Utc::now().timestamp() as u64,
            },
            releases,
            featured: Vec::new(),
            signature: None,
        }
    }

    /// Create envelope from a single release
    pub fn from_release(release: Release, height: u64) -> Self {
        Self::new(vec![release], height)
    }

    /// Compute deterministic block ID from contents
    fn compute_block_id(releases: &[Release], height: u64) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash height
        height.hash(&mut hasher);

        // Hash all release IDs
        for release in releases {
            release.id.hash(&mut hasher);
            release.created_at.hash(&mut hasher);
        }

        format!("block-{:x}", hasher.finish())
    }

    /// Set the previous block hash
    pub fn with_prev(mut self, prev: String) -> Self {
        self.meta.prev = Some(prev);
        self
    }

    /// Add featured releases to this block
    pub fn with_featured(mut self, featured: Vec<String>) -> Self {
        self.featured = featured;
        self
    }

    /// Serialize to BlockData for P2P transfer
    pub fn to_block_data(&self) -> Result<BlockData> {
        let data = serde_json::to_vec(self)
            .context("Failed to serialize block envelope")?;

        Ok(BlockData {
            id: self.meta.id.clone(),
            height: self.meta.height,
            data,
            prev: self.meta.prev.clone(),
            timestamp: self.meta.timestamp,
        })
    }

    /// Deserialize from BlockData received from P2P
    pub fn from_block_data(block: &BlockData) -> Result<Self> {
        serde_json::from_slice(&block.data)
            .context("Failed to deserialize block envelope")
    }
}

/// Block codec for converting between database format and P2P blocks
pub struct BlockCodec;

impl BlockCodec {
    /// Create a block from a release (single-release block)
    pub fn encode_release(release: Release, height: u64, prev: Option<String>) -> Result<BlockData> {
        let mut envelope = BlockEnvelope::from_release(release, height);
        if let Some(prev_hash) = prev {
            envelope = envelope.with_prev(prev_hash);
        }
        envelope.to_block_data()
    }

    /// Create a block from multiple releases (batch block)
    pub fn encode_releases(releases: Vec<Release>, height: u64, prev: Option<String>) -> Result<BlockData> {
        let mut envelope = BlockEnvelope::new(releases, height);
        if let Some(prev_hash) = prev {
            envelope = envelope.with_prev(prev_hash);
        }
        envelope.to_block_data()
    }

    /// Extract releases from a received block
    pub fn decode_releases(block: &BlockData) -> Result<Vec<Release>> {
        let envelope = BlockEnvelope::from_block_data(block)?;
        Ok(envelope.releases)
    }

    /// Extract metadata from a block
    pub fn decode_meta(block: &BlockData) -> Result<BlockMeta> {
        let envelope = BlockEnvelope::from_block_data(block)?;
        Ok(envelope.meta)
    }

    /// Extract featured list from a block
    pub fn decode_featured(block: &BlockData) -> Result<Vec<String>> {
        let envelope = BlockEnvelope::from_block_data(block)?;
        Ok(envelope.featured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_release(id: &str) -> Release {
        Release {
            id: id.to_string(),
            name: "Test Release".to_string(),
            category_id: "test-category".to_string(),
            category_slug: "test".to_string(),
            content_cid: "QmTest123".to_string(),
            thumbnail_cid: None,
            metadata: None,
            site_address: "local".to_string(),
            posted_by: "test_user".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            vector_clock: std::collections::HashMap::new(),
            is_tombstone: false,
            tombstone_type: None,
            deleted_at: None,
            deleted_by: None,
        }
    }

    #[test]
    fn test_block_envelope_single_release() {
        let release = make_test_release("release-1");
        let envelope = BlockEnvelope::from_release(release.clone(), 1);

        assert_eq!(envelope.meta.height, 1);
        assert_eq!(envelope.releases.len(), 1);
        assert_eq!(envelope.releases[0].id, "release-1");
    }

    #[test]
    fn test_block_envelope_multiple_releases() {
        let releases = vec![
            make_test_release("release-1"),
            make_test_release("release-2"),
            make_test_release("release-3"),
        ];

        let envelope = BlockEnvelope::new(releases, 5);

        assert_eq!(envelope.meta.height, 5);
        assert_eq!(envelope.releases.len(), 3);
    }

    #[test]
    fn test_block_envelope_with_prev() {
        let release = make_test_release("release-1");
        let envelope = BlockEnvelope::from_release(release, 10)
            .with_prev("block-abc123".to_string());

        assert_eq!(envelope.meta.prev, Some("block-abc123".to_string()));
    }

    #[test]
    fn test_block_envelope_serialization() {
        let release = make_test_release("release-1");
        let envelope = BlockEnvelope::from_release(release, 1);

        // Serialize to BlockData
        let block_data = envelope.to_block_data().unwrap();

        assert_eq!(block_data.height, 1);
        assert!(!block_data.data.is_empty());

        // Deserialize back
        let decoded = BlockEnvelope::from_block_data(&block_data).unwrap();
        assert_eq!(decoded.meta.height, 1);
        assert_eq!(decoded.releases.len(), 1);
        assert_eq!(decoded.releases[0].id, "release-1");
    }

    #[test]
    fn test_codec_encode_single_release() {
        let release = make_test_release("release-1");

        let block = BlockCodec::encode_release(release, 1, None).unwrap();

        assert_eq!(block.height, 1);
        assert!(block.prev.is_none());
    }

    #[test]
    fn test_codec_encode_with_prev() {
        let release = make_test_release("release-1");

        let block = BlockCodec::encode_release(
            release,
            5,
            Some("prev-block-id".to_string())
        ).unwrap();

        assert_eq!(block.height, 5);
        assert_eq!(block.prev, Some("prev-block-id".to_string()));
    }

    #[test]
    fn test_codec_roundtrip() {
        let release = make_test_release("release-test");

        // Encode
        let block = BlockCodec::encode_release(release.clone(), 10, None).unwrap();

        // Decode
        let decoded_releases = BlockCodec::decode_releases(&block).unwrap();

        assert_eq!(decoded_releases.len(), 1);
        assert_eq!(decoded_releases[0].id, release.id);
        assert_eq!(decoded_releases[0].name, release.name);
        assert_eq!(decoded_releases[0].content_cid, release.content_cid);
    }

    #[test]
    fn test_codec_batch_encode() {
        let releases = vec![
            make_test_release("r1"),
            make_test_release("r2"),
            make_test_release("r3"),
        ];

        let block = BlockCodec::encode_releases(releases, 20, None).unwrap();

        let decoded = BlockCodec::decode_releases(&block).unwrap();
        assert_eq!(decoded.len(), 3);
    }

    #[test]
    fn test_deterministic_block_ids() {
        let release1 = make_test_release("r1");
        let release2 = make_test_release("r1"); // Same ID

        let envelope1 = BlockEnvelope::from_release(release1, 1);
        let envelope2 = BlockEnvelope::from_release(release2, 1);

        // Same content at same height should produce same block ID
        assert_eq!(envelope1.meta.id, envelope2.meta.id);
    }
}
