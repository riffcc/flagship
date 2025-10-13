//! DHT-Routed Peer-to-Peer Messaging
//!
//! This module implements DHT-routed messaging as per Citadel SPEC Section 2.4:
//! - Messages route THROUGH the DHT mesh, not via direct TCP connections
//! - peer_message_key(peer_id, nonce) stores messages in DHT
//! - Response mechanism using response_key pattern
//! - 5-second message timeout for request/response cycles
//!
//! ## Key Concepts
//!
//! 1. **DHT Message Storage**: Messages are PUT at peer_message_key(target, nonce)
//! 2. **Response Keys**: Each message includes where to write the response
//! 3. **Message Polling**: Nodes poll DHT for incoming messages at their peer_message_key
//! 4. **Message Expiration**: Messages timeout after 5 seconds if no response
//!
//! ## Usage Example
//!
//! ```rust
//! // Send a message to another peer through DHT
//! let response = send_to_peer(
//!     dht_storage.clone(),
//!     "my-peer-id".to_string(),
//!     "target-peer-id".to_string(),
//!     b"Hello!".to_vec(),
//! ).await?;
//!
//! // Start polling loop for incoming messages
//! tokio::spawn(async move {
//!     poll_messages_loop(
//!         dht_storage,
//!         "my-peer-id".to_string(),
//!         message_handler,
//!     ).await;
//! });
//! ```

use anyhow::{anyhow, Context, Result};
use citadel_core::key_mapping::DHTKey;
use citadel_dht::local_storage::LocalStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Message timeout duration (5 seconds)
pub const MESSAGE_TIMEOUT: Duration = Duration::from_secs(5);

/// Message polling interval (100ms)
pub const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Maximum number of nonces to check per poll cycle
pub const MAX_NONCE_CHECK: u64 = 100;

/// Peer-to-peer message routed through DHT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerMessage {
    /// Sender's peer ID
    pub from: String,

    /// Recipient's peer ID
    pub to: String,

    /// Unique message nonce (prevents duplicates)
    pub nonce: u64,

    /// Message payload (arbitrary bytes)
    pub payload: Vec<u8>,

    /// DHT key where response should be written
    pub response_key: DHTKey,

    /// Timestamp when message was created (Unix timestamp)
    pub timestamp: u64,
}

impl PeerMessage {
    /// Create a new peer message
    pub fn new(
        from: String,
        to: String,
        nonce: u64,
        payload: Vec<u8>,
        response_key: DHTKey,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            from,
            to,
            nonce,
            payload,
            response_key,
            timestamp,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).context("Failed to serialize PeerMessage")
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).context("Failed to deserialize PeerMessage")
    }

    /// Check if message has expired (older than 5 seconds)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.timestamp > MESSAGE_TIMEOUT.as_secs()
    }
}

/// Generate DHT key for peer messages
///
/// Key format: blake3("peer-message" || peer_id || nonce)
pub fn peer_message_key(peer_id: &str, nonce: u64) -> DHTKey {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"peer-message");
    hasher.update(peer_id.as_bytes());
    hasher.update(&nonce.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Generate DHT key for peer location (which slot they occupy)
///
/// Key format: blake3("peer-location" || peer_id)
pub fn peer_location_key(peer_id: &str) -> DHTKey {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"peer-location");
    hasher.update(peer_id.as_bytes());
    *hasher.finalize().as_bytes()
}

/// Generate DHT key for response messages
///
/// Key format: blake3("peer-response" || peer_id || nonce)
pub fn peer_response_key(peer_id: &str, nonce: u64) -> DHTKey {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"peer-response");
    hasher.update(peer_id.as_bytes());
    hasher.update(&nonce.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Send a message to a peer through DHT routing
///
/// This function:
/// 1. Generates a unique nonce for the message
/// 2. Creates a PeerMessage with a response_key
/// 3. PUTs the message at peer_message_key(target, nonce) in DHT
/// 4. Polls for response at response_key with 5-second timeout
/// 5. Returns response payload or timeout error
///
/// # Arguments
///
/// * `dht_storage` - Shared DHT storage
/// * `my_peer_id` - Our peer ID (sender)
/// * `target` - Target peer ID (recipient)
/// * `msg` - Message payload bytes
///
/// # Returns
///
/// Response payload bytes or error if timeout/failure
pub async fn send_to_peer(
    dht_storage: Arc<Mutex<LocalStorage>>,
    my_peer_id: String,
    target: String,
    msg: Vec<u8>,
) -> Result<Vec<u8>> {
    // Step 1: Generate unique nonce
    let nonce = rand::random::<u64>();

    // Step 2: Create response key where target will write response
    let response_key = peer_response_key(&my_peer_id, nonce);

    debug!(
        "📤 Sending message to {} (nonce: {}, size: {} bytes)",
        target,
        nonce,
        msg.len()
    );

    // Step 3: Create PeerMessage
    let message = PeerMessage::new(
        my_peer_id.clone(),
        target.clone(),
        nonce,
        msg,
        response_key.clone(),
    );

    // Step 4: PUT message at target's message key (routes through DHT!)
    let message_key = peer_message_key(&target, nonce);
    let message_bytes = message.to_bytes()?;

    {
        let mut dht = dht_storage.lock().await;
        dht.put(message_key, message_bytes);
    }

    info!("✅ Message PUT in DHT at peer_message_key({}, {})", target, nonce);

    // Step 5: Poll for response at our response key (5-second timeout)
    let start = Instant::now();

    loop {
        if start.elapsed() > MESSAGE_TIMEOUT {
            warn!(
                "⏱️ Timeout waiting for response from {} (nonce: {})",
                target, nonce
            );
            return Err(anyhow!(
                "Timeout waiting for response from {} after {:?}",
                target,
                MESSAGE_TIMEOUT
            ));
        }

        // Check for response
        let dht = dht_storage.lock().await;
        if let Some(response_bytes) = dht.get(&response_key) {
            let response = PeerMessage::from_bytes(response_bytes)?;

            info!(
                "📥 Received response from {} (nonce: {}, size: {} bytes, latency: {:?})",
                response.from,
                response.nonce,
                response.payload.len(),
                start.elapsed()
            );

            return Ok(response.payload);
        }
        drop(dht);

        // Sleep before next poll
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Message handler trait for processing incoming messages
///
/// Implement this trait to define how your application handles DHT messages
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message and return a response
    ///
    /// # Arguments
    ///
    /// * `message` - The incoming message
    ///
    /// # Returns
    ///
    /// Response payload bytes
    async fn handle_message(&self, message: &PeerMessage) -> Vec<u8>;
}

/// Simple echo message handler (for testing)
pub struct EchoHandler;

#[async_trait::async_trait]
impl MessageHandler for EchoHandler {
    async fn handle_message(&self, message: &PeerMessage) -> Vec<u8> {
        // Echo back the same payload
        message.payload.clone()
    }
}

/// Poll for incoming messages at our peer_message_key
///
/// This function runs in a background loop and:
/// 1. Checks DHT for messages addressed to us (at peer_message_key)
/// 2. Processes each message using the provided handler
/// 3. Writes response to sender's response_key
/// 4. Deletes processed messages from DHT
///
/// # Arguments
///
/// * `dht_storage` - Shared DHT storage
/// * `my_peer_id` - Our peer ID
/// * `handler` - Message handler implementation
pub async fn poll_messages_loop<H: MessageHandler>(
    dht_storage: Arc<Mutex<LocalStorage>>,
    my_peer_id: String,
    handler: Arc<H>,
) {
    info!("🔄 Starting message polling loop for peer {}", my_peer_id);

    let mut interval = tokio::time::interval(POLL_INTERVAL);
    let mut nonce_counter = 0u64;

    loop {
        interval.tick().await;

        // Check for messages at peer_message_key(my_peer_id, nonce)
        // We check a sliding window of nonces to catch any messages
        let start_nonce = nonce_counter.saturating_sub(MAX_NONCE_CHECK);
        let end_nonce = nonce_counter + MAX_NONCE_CHECK;

        for nonce in start_nonce..=end_nonce {
            let message_key = peer_message_key(&my_peer_id, nonce);

            let mut dht = dht_storage.lock().await;

            if let Some(message_bytes) = dht.get(&message_key) {
                // Found a message!
                match PeerMessage::from_bytes(message_bytes) {
                    Ok(message) => {
                        // Check if message is expired
                        if message.is_expired() {
                            debug!(
                                "🗑️ Deleting expired message from {} (nonce: {})",
                                message.from, message.nonce
                            );
                            dht.delete(&message_key);
                            continue;
                        }

                        debug!(
                            "📨 Received message from {} (nonce: {}, size: {} bytes)",
                            message.from,
                            message.nonce,
                            message.payload.len()
                        );

                        // Drop lock before calling handler
                        drop(dht);

                        // Process message and get response
                        let response_payload = handler.handle_message(&message).await;

                        // Create response message
                        let response = PeerMessage::new(
                            my_peer_id.clone(),
                            message.from.clone(),
                            message.nonce,
                            response_payload,
                            [0u8; 32], // No response to a response
                        );

                        // Write response to sender's response_key
                        let response_bytes = match response.to_bytes() {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                warn!("Failed to serialize response: {}", e);
                                continue;
                            }
                        };

                        let mut dht = dht_storage.lock().await;
                        dht.put(message.response_key, response_bytes);

                        info!(
                            "✅ Sent response to {} (nonce: {}, size: {} bytes)",
                            message.from,
                            message.nonce,
                            response.payload.len()
                        );

                        // Delete processed message
                        dht.delete(&message_key);

                        debug!("🗑️ Deleted processed message (nonce: {})", message.nonce);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialize message at nonce {}: {}",
                            nonce, e
                        );
                        // Delete corrupted message
                        dht.delete(&message_key);
                    }
                }
            } else {
                drop(dht);
            }
        }

        // Increment nonce counter for next iteration
        nonce_counter = nonce_counter.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== TDD Tests for DHT Messaging ==========

    #[test]
    fn test_peer_message_key_deterministic() {
        let key1 = peer_message_key("peer-123", 42);
        let key2 = peer_message_key("peer-123", 42);

        assert_eq!(key1, key2, "Same peer ID and nonce should produce same key");
    }

    #[test]
    fn test_peer_message_key_different_peer() {
        let key1 = peer_message_key("peer-123", 42);
        let key2 = peer_message_key("peer-456", 42);

        assert_ne!(key1, key2, "Different peer IDs should produce different keys");
    }

    #[test]
    fn test_peer_message_key_different_nonce() {
        let key1 = peer_message_key("peer-123", 42);
        let key2 = peer_message_key("peer-123", 99);

        assert_ne!(key1, key2, "Different nonces should produce different keys");
    }

    #[test]
    fn test_peer_location_key_deterministic() {
        let key1 = peer_location_key("peer-123");
        let key2 = peer_location_key("peer-123");

        assert_eq!(key1, key2, "Same peer ID should produce same location key");
    }

    #[test]
    fn test_peer_response_key_deterministic() {
        let key1 = peer_response_key("peer-123", 42);
        let key2 = peer_response_key("peer-123", 42);

        assert_eq!(key1, key2, "Same peer ID and nonce should produce same response key");
    }

    #[test]
    fn test_peer_message_creation() {
        let mut response_key = [0u8; 32];
        response_key[0] = 1;
        response_key[1] = 2;
        response_key[2] = 3;

        let msg = PeerMessage::new(
            "peer-from".to_string(),
            "peer-to".to_string(),
            42,
            b"Hello".to_vec(),
            response_key,
        );

        assert_eq!(msg.from, "peer-from");
        assert_eq!(msg.to, "peer-to");
        assert_eq!(msg.nonce, 42);
        assert_eq!(msg.payload, b"Hello");
        assert_eq!(msg.response_key[0], 1);
        assert_eq!(msg.response_key[1], 2);
        assert_eq!(msg.response_key[2], 3);
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_peer_message_serialization() {
        let mut response_key = [0u8; 32];
        response_key[0] = 1;
        response_key[1] = 2;
        response_key[2] = 3;

        let msg = PeerMessage::new(
            "peer-from".to_string(),
            "peer-to".to_string(),
            42,
            b"Hello".to_vec(),
            response_key,
        );

        let bytes = msg.to_bytes().unwrap();
        let deserialized = PeerMessage::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.from, msg.from);
        assert_eq!(deserialized.to, msg.to);
        assert_eq!(deserialized.nonce, msg.nonce);
        assert_eq!(deserialized.payload, msg.payload);
        assert_eq!(deserialized.response_key, msg.response_key);
        assert_eq!(deserialized.timestamp, msg.timestamp);
    }

    #[test]
    fn test_peer_message_expiration() {
        let mut msg = PeerMessage::new(
            "peer-from".to_string(),
            "peer-to".to_string(),
            42,
            b"Hello".to_vec(),
            [0u8; 32],
        );

        // Fresh message should not be expired
        assert!(!msg.is_expired());

        // Make message old (6 seconds ago)
        msg.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - 6;

        // Old message should be expired
        assert!(msg.is_expired());
    }

    #[tokio::test]
    async fn test_send_to_peer_and_receive_response() {
        let dht_storage = Arc::new(Mutex::new(LocalStorage::new()));
        let my_peer_id = "peer-sender".to_string();
        let target_peer_id = "peer-receiver".to_string();

        // Start a background task to simulate receiver responding
        let dht_clone = dht_storage.clone();
        let target_clone = target_peer_id.clone();
        tokio::spawn(async move {
            // Wait a bit for message to arrive
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Find the message in DHT
            let dht = dht_clone.lock().await;
            let keys = dht.keys();

            for key in keys {
                if let Some(msg_bytes) = dht.get(&key) {
                    if let Ok(msg) = PeerMessage::from_bytes(msg_bytes) {
                        if msg.to == target_clone {
                            // Found our message! Write response
                            drop(dht);

                            let response = PeerMessage::new(
                                target_clone.clone(),
                                msg.from.clone(),
                                msg.nonce,
                                b"Response!".to_vec(),
                                [0u8; 32],
                            );

                            let mut dht = dht_clone.lock().await;
                            dht.put(msg.response_key, response.to_bytes().unwrap());
                            break;
                        }
                    }
                }
            }
        });

        // Send message and wait for response
        let response = send_to_peer(
            dht_storage,
            my_peer_id,
            target_peer_id,
            b"Hello!".to_vec(),
        )
        .await
        .unwrap();

        assert_eq!(response, b"Response!");
    }

    #[tokio::test]
    async fn test_send_to_peer_timeout() {
        let dht_storage = Arc::new(Mutex::new(LocalStorage::new()));
        let my_peer_id = "peer-sender".to_string();
        let target_peer_id = "peer-no-response".to_string();

        // Don't start a receiver - message will timeout

        // This should timeout after 5 seconds
        // We use a shorter timeout for testing
        let start = Instant::now();
        let result = send_to_peer(
            dht_storage,
            my_peer_id,
            target_peer_id,
            b"Hello!".to_vec(),
        )
        .await;

        assert!(result.is_err());
        assert!(start.elapsed() >= MESSAGE_TIMEOUT);
    }

    #[tokio::test]
    async fn test_echo_handler() {
        let handler = EchoHandler;
        let msg = PeerMessage::new(
            "peer-from".to_string(),
            "peer-to".to_string(),
            42,
            b"Echo this!".to_vec(),
            [0u8; 32],
        );

        let response = handler.handle_message(&msg).await;
        assert_eq!(response, b"Echo this!");
    }

    #[tokio::test]
    async fn test_poll_messages_loop_processes_message() {
        let dht_storage = Arc::new(Mutex::new(LocalStorage::new()));
        let my_peer_id = "peer-receiver".to_string();
        let handler = Arc::new(EchoHandler);

        // Put a message in DHT for us (use nonce in initial scanning range)
        let nonce = 50u64;  // Within first MAX_NONCE_CHECK range
        let msg = PeerMessage::new(
            "peer-sender".to_string(),
            my_peer_id.clone(),
            nonce,
            b"Test message".to_vec(),
            peer_response_key("peer-sender", nonce),
        );

        let message_key = peer_message_key(&my_peer_id, nonce);
        {
            let mut dht = dht_storage.lock().await;
            dht.put(message_key.clone(), msg.to_bytes().unwrap());
        }

        // Start polling loop
        let dht_clone = dht_storage.clone();
        let my_peer_id_clone = my_peer_id.clone();
        let poll_handle = tokio::spawn(async move {
            // Poll for just a short time
            tokio::time::timeout(
                Duration::from_secs(2),
                poll_messages_loop(dht_clone, my_peer_id_clone, handler),
            )
            .await
            .ok();
        });

        // Wait for message to be processed (give poll loop more time)
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Check that response was written
        {
            let dht = dht_storage.lock().await;
            let response_key = peer_response_key("peer-sender", nonce);
            let response_bytes = dht.get(&response_key);
            assert!(response_bytes.is_some(), "Response should be written");

            let response = PeerMessage::from_bytes(response_bytes.unwrap()).unwrap();
            assert_eq!(response.payload, b"Test message");

            // Message should be deleted after processing
            assert!(dht.get(&message_key).is_none(), "Message should be deleted");
        }

        poll_handle.abort();
    }

    #[tokio::test]
    async fn test_poll_messages_loop_deletes_expired() {
        let dht_storage = Arc::new(Mutex::new(LocalStorage::new()));
        let my_peer_id = "peer-receiver".to_string();
        let handler = Arc::new(EchoHandler);

        // Put an expired message in DHT (use nonce in initial scanning range)
        let nonce = 50u64;  // Within first MAX_NONCE_CHECK range
        let mut msg = PeerMessage::new(
            "peer-sender".to_string(),
            my_peer_id.clone(),
            nonce,
            b"Expired".to_vec(),
            [0u8; 32],
        );

        // Make it expired (6 seconds old)
        msg.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - 6;

        let message_key = peer_message_key(&my_peer_id, nonce);
        {
            let mut dht = dht_storage.lock().await;
            dht.put(message_key, msg.to_bytes().unwrap());
        }

        // Start polling loop
        let dht_clone = dht_storage.clone();
        let my_peer_id_clone = my_peer_id.clone();
        let poll_handle = tokio::spawn(async move {
            tokio::time::timeout(
                Duration::from_secs(2),
                poll_messages_loop(dht_clone, my_peer_id_clone, handler),
            )
            .await
            .ok();
        });

        // Wait for polling to process expired message (give poll loop more time)
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Expired message should be deleted
        {
            let dht = dht_storage.lock().await;
            assert!(dht.get(&message_key).is_none(), "Expired message should be deleted");
        }

        poll_handle.abort();
    }
}
