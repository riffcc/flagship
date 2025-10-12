//! UBTS Codec
//!
//! Encoding and decoding UBTS blocks to/from wire format

use anyhow::{Context, Result};
use lens_v2_p2p::network::BlockData;
use crate::ubts::{UBTSBlock, UBTSTransaction};

/// UBTS Codec for encoding/decoding UBTS blocks
pub struct UBTSCodec;

impl UBTSCodec {
    /// Encode a UBTS block to BlockData for P2P transmission
    pub fn encode(block: &UBTSBlock) -> Result<BlockData> {
        // Serialize UBTS block to JSON
        let data = serde_json::to_vec(block)
            .context("Failed to serialize UBTS block")?;

        Ok(BlockData {
            id: block.id.clone(),
            height: block.height,
            data,
            prev: block.prev.clone(),
            timestamp: block.timestamp,
        })
    }

    /// Decode a BlockData into a UBTS block
    pub fn decode(block_data: &BlockData) -> Result<UBTSBlock> {
        // Check if this is a UBTS block (ID starts with "ubts-")
        if !block_data.id.starts_with("ubts-") {
            anyhow::bail!("Not a UBTS block: {}", block_data.id);
        }

        // Deserialize from JSON
        let block: UBTSBlock = serde_json::from_slice(&block_data.data)
            .context("Failed to deserialize UBTS block")?;

        Ok(block)
    }

    /// Check if a BlockData is a UBTS block
    pub fn is_ubts_block(block_data: &BlockData) -> bool {
        block_data.id.starts_with("ubts-")
    }

    /// Extract transactions from a UBTS block
    pub fn extract_transactions(block: &UBTSBlock) -> Vec<UBTSTransaction> {
        block.transactions.clone()
    }

    /// Create a UBTS block from transactions
    pub fn create_block(
        height: u64,
        prev: Option<String>,
        transactions: Vec<UBTSTransaction>,
    ) -> UBTSBlock {
        UBTSBlock::new(height, prev, transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ubts::UBTSTransaction;
    use crate::routes::releases::Release;

    #[test]
    fn test_ubts_encoding_decoding() -> Result<()> {
        let tx = UBTSTransaction::CreateRelease {
            release: Release {
                id: "test-123".to_string(),
                name: "Test Release".to_string(),
                description: "A test release".to_string(),
                version: "1.0.0".to_string(),
                download_url: "https://example.com/release.tar.gz".to_string(),
                category: "apps".to_string(),
                tags: vec!["test".to_string()],
            },
            signature: None,
        };

        let block = UBTSBlock::new(1, None, vec![tx.clone()]);

        // Encode
        let block_data = UBTSCodec::encode(&block)?;

        // Check it's recognized as UBTS
        assert!(UBTSCodec::is_ubts_block(&block_data));

        // Decode
        let decoded_block = UBTSCodec::decode(&block_data)?;

        // Verify
        assert_eq!(decoded_block.id, block.id);
        assert_eq!(decoded_block.height, block.height);
        assert_eq!(decoded_block.transactions.len(), 1);

        Ok(())
    }

    #[test]
    fn test_non_ubts_block() {
        let block_data = BlockData {
            id: "release-abc123".to_string(),
            height: 1,
            data: vec![],
            prev: None,
            timestamp: 0,
        };

        assert!(!UBTSCodec::is_ubts_block(&block_data));
    }
}
