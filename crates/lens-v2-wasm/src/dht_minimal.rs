//! Minimal DHT implementation for WASM
//!
//! Ultra-lightweight DHT client that can read/write directly from the browser
//! without requiring WebSocket connections.

use wasm_bindgen::prelude::*;
use web_sys::window;

/// Minimal DHT node for browser
#[wasm_bindgen]
pub struct DHTClient {
    /// API endpoint for DHT operations
    api_url: String,

    /// Our peer ID
    peer_id: String,
}

#[wasm_bindgen]
impl DHTClient {
    /// Create a new DHT client
    #[wasm_bindgen(constructor)]
    pub fn new(api_url: String) -> Self {
        // Generate random peer ID
        let peer_id = format!("browser-{}", js_sys::Math::random());

        Self {
            api_url,
            peer_id,
        }
    }

    /// PUT a value into the DHT
    /// Uses fetch API for direct HTTP access (no WebSocket!)
    pub async fn put(&self, key: String, value: String) -> Result<(), JsValue> {
        let window = window().ok_or_else(|| JsValue::from_str("No window"))?;
        let url = format!("{}/dht/put", self.api_url);

        // Create PUT request
        let opts = web_sys::RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(web_sys::RequestMode::Cors);

        // Body with key and value
        let body = js_sys::JSON::stringify(&js_sys::Object::from(serde_wasm_bindgen::to_value(
            &serde_json::json!({
                "key": key,
                "value": value,
            })
        )?))?;
        opts.set_body(&body);

        let request = web_sys::Request::new_with_str_and_init(&url, &opts)?;
        request.headers().set("Content-Type", "application/json")?;

        let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;

        if !resp.ok() {
            return Err(JsValue::from_str(&format!("DHT PUT failed: {}", resp.status())));
        }

        Ok(())
    }

    /// GET a value from the DHT
    /// Uses fetch API for direct HTTP access (no WebSocket!)
    pub async fn get(&self, key: String) -> Result<Option<String>, JsValue> {
        let window = window().ok_or_else(|| JsValue::from_str("No window"))?;
        let url = format!("{}/dht/get?key={}", self.api_url, key);

        let opts = web_sys::RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(web_sys::RequestMode::Cors);

        let request = web_sys::Request::new_with_str_and_init(&url, &opts)?;

        let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;

        if resp.status() == 404 {
            return Ok(None);
        }

        if !resp.ok() {
            return Err(JsValue::from_str(&format!("DHT GET failed: {}", resp.status())));
        }

        let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
        let result: serde_json::Value = serde_wasm_bindgen::from_value(json)?;

        if let Some(value) = result.get("value").and_then(|v| v.as_str()) {
            Ok(Some(value.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Get peer ID
    pub fn peer_id(&self) -> String {
        self.peer_id.clone()
    }
}

/// Generate Blake3 hash of a key for DHT storage
/// Uses Blake3 for fast, correct, deterministic hashing
#[wasm_bindgen]
pub fn hash_key(prefix: &str, id: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prefix.as_bytes());
    hasher.update(id.as_bytes());
    let hash = hasher.finalize();

    // Return hex string of hash
    hex::encode(hash.as_bytes())
}
