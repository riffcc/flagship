use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// UBTS Transaction - unified representation of all state transitions
/// Note: Not exported to JS directly due to wasm-bindgen limitations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UBTSTransaction {
    CreateRelease {
        release: serde_json::Value,
        signature: Option<String>,
    },
    UpdateRelease {
        id: String,
        patch: serde_json::Value,
        signature: Option<String>,
    },
    DeleteRelease {
        id: String,
        signature: Option<String>,
    },
    DeleteWithConsensus {
        delete_id: String,
        deleted_block_ids: Vec<String>,
        reason: String,
        deleted_by: String,
        required_confirmations: usize,
        timestamp: u64,
        signature: Option<String>,
    },
    AuthorizeAdmin {
        public_key: String,
        authorized_by: String,
        timestamp: u64,
        signature: Option<String>,
    },
    SetFeatured {
        release_ids: Vec<String>,
        signature: Option<String>,
    },
    AddFeatured {
        release_ids: Vec<String>,
        signature: Option<String>,
    },
    RemoveFeatured {
        release_ids: Vec<String>,
        signature: Option<String>,
    },
    DeleteFeaturedRelease {
        id: String,
        signature: Option<String>,
    },
}

/// UBTS Block - contains multiple transactions
/// Internal representation, use to_js() to get JsValue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UBTSBlock {
    id: String,
    height: u64,
    prev: Option<String>,
    timestamp: u64,
    transactions: Vec<UBTSTransaction>,
    signature: Option<String>,
}

impl UBTSBlock {
    /// Create a new UBTS block
    pub fn new(height: u64, prev: Option<String>, transactions: Vec<UBTSTransaction>) -> Self {
        let timestamp = js_sys::Date::now() as u64 / 1000;

        // Generate block ID from transactions
        let id = Self::compute_id(&transactions, height, timestamp);

        Self {
            id,
            height,
            prev,
            timestamp,
            transactions,
            signature: None,
        }
    }

    /// Compute block ID from transactions
    fn compute_id(transactions: &[UBTSTransaction], height: u64, timestamp: u64) -> String {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();

        // Hash transactions
        for tx in transactions {
            if let Ok(tx_bytes) = serde_json::to_vec(tx) {
                hasher.update(&tx_bytes);
            }
        }

        // Hash height and timestamp
        hasher.update(height.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());

        let result = hasher.finalize();
        format!("ubts-{}", hex::encode(result))
    }

    /// Convert to JsValue for JavaScript interop
    pub fn to_js(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(self)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create from JsValue
    pub fn from_js(value: JsValue) -> Result<Self, JsValue> {
        serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
