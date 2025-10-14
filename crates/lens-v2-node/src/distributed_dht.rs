//! Distributed DHT Operations with Mesh Routing
//!
//! This module implements TRUE distributed DHT where:
//! - Each node stores ONLY keys that hash to its slot
//! - PUT/GET operations route through the mesh to the correct slot
//! - Uses greedy routing from Citadel spec

use anyhow::{anyhow, Result};
use citadel_core::topology::{MeshConfig, SlotCoordinate};
use citadel_core::routing::greedy_direction;
use citadel_core::key_mapping::key_to_slot;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::dht_state::DhtState;

/// DHT operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DHTOperation {
    /// Put a key-value pair
    Put { key: [u8; 32], value: Vec<u8> },
    /// Get a value by key
    Get { key: [u8; 32] },
}

/// DHT request message (routes through mesh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DHTRequest {
    /// Request ID for matching responses
    pub request_id: u64,

    /// Originating peer ID
    pub from_peer: String,

    /// Target slot that should handle this operation
    pub target_slot: SlotCoordinate,

    /// Current slot (for hop counting)
    pub current_slot: SlotCoordinate,

    /// Operation to perform
    pub operation: DHTOperation,

    /// Hop count (for debugging)
    pub hops: u32,
}

/// DHT response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DHTResponse {
    /// Request ID this responds to
    pub request_id: u64,

    /// Peer that handled the request
    pub from_peer: String,

    /// Result of the operation
    pub result: DHTResult,

    /// Number of hops taken
    pub hops: u32,
}

/// DHT operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DHTResult {
    /// PUT succeeded
    PutOk,
    /// GET succeeded with value
    GetOk(Vec<u8>),
    /// Key not found
    NotFound,
    /// Operation failed
    Error(String),
}

/// Distributed DHT client
pub struct DistributedDHT {
    /// This node's peer ID
    my_peer_id: String,

    /// This node's slot coordinate
    my_slot: SlotCoordinate,

    /// Mesh configuration
    mesh_config: MeshConfig,

    /// Local DHT storage (stores ONLY keys that hash to my slot!)
    local_storage: Arc<Mutex<DhtState>>,

    /// Request sender (WebSocket relay)
    request_sender: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<DHTRequest>>>>,
}

impl DistributedDHT {
    /// Create a new distributed DHT client
    pub fn new(
        my_peer_id: String,
        my_slot: SlotCoordinate,
        mesh_config: MeshConfig,
        local_storage: Arc<Mutex<DhtState>>,
    ) -> Self {
        Self {
            my_peer_id,
            my_slot,
            mesh_config,
            local_storage,
            request_sender: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the request sender (connected to relay WebSocket)
    pub async fn set_request_sender(&self, sender: tokio::sync::mpsc::UnboundedSender<DHTRequest>) {
        *self.request_sender.lock().await = Some(sender);
    }

    /// PUT a key-value pair (routes to correct slot)
    pub async fn put(&self, key: [u8; 32], value: Vec<u8>) -> Result<()> {
        // Calculate which slot should store this key
        let target_slot = key_to_slot(&key, &self.mesh_config);

        debug!("📍 DHT PUT: key {} -> slot {:?}", hex::encode(&key[..8]), target_slot);

        // If this key belongs to MY slot, store it locally
        if target_slot == self.my_slot {
            let mut storage = self.local_storage.lock().await;
            storage.insert_raw(key, value);
            info!("✅ DHT PUT: Stored locally at my slot {:?}", self.my_slot);
            return Ok(());
        }

        // Otherwise, route the PUT request to the correct slot
        let request = DHTRequest {
            request_id: rand::random(),
            from_peer: self.my_peer_id.clone(),
            target_slot,
            current_slot: self.my_slot,
            operation: DHTOperation::Put { key, value },
            hops: 0,
        };

        // Send via relay (will route through mesh)
        let sender = self.request_sender.lock().await;
        if let Some(tx) = sender.as_ref() {
            tx.send(request)?;
            info!("📤 DHT PUT: Routed request to slot {:?}", target_slot);
            Ok(())
        } else {
            Err(anyhow!("DHT request sender not initialized"))
        }
    }

    /// GET a value by key (routes to correct slot)
    pub async fn get(&self, key: &[u8; 32]) -> Result<Option<Vec<u8>>> {
        // Calculate which slot stores this key
        let target_slot = key_to_slot(key, &self.mesh_config);

        debug!("🔍 DHT GET: key {} -> slot {:?}", hex::encode(&key[..8]), target_slot);

        // If this key belongs to MY slot, get it locally
        if target_slot == self.my_slot {
            let storage = self.local_storage.lock().await;
            let result = storage.get_raw(key).map(|v| v.to_vec());
            if result.is_some() {
                info!("✅ DHT GET: Found locally at my slot {:?}", self.my_slot);
            } else {
                info!("❌ DHT GET: Not found at my slot {:?}", self.my_slot);
            }
            return Ok(result);
        }

        // Otherwise, route the GET request to the correct slot
        let request = DHTRequest {
            request_id: rand::random(),
            from_peer: self.my_peer_id.clone(),
            target_slot,
            current_slot: self.my_slot,
            operation: DHTOperation::Get { key: *key },
            hops: 0,
        };

        // Send via relay (will route through mesh)
        let sender = self.request_sender.lock().await;
        if let Some(tx) = sender.as_ref() {
            tx.send(request)?;
            info!("📤 DHT GET: Routed request to slot {:?}", target_slot);
            // TODO: Wait for response (needs response channel)
            Ok(None)
        } else {
            Err(anyhow!("DHT request sender not initialized"))
        }
    }

    /// Handle incoming DHT request (as relay node or final destination)
    pub async fn handle_request(&self, mut request: DHTRequest) -> Option<DHTResponse> {
        request.hops += 1;

        debug!(
            "📨 DHT Request: from {} to {:?} (currently at {:?}, hop {})",
            request.from_peer, request.target_slot, request.current_slot, request.hops
        );

        // Check if WE are the target slot
        if request.target_slot == self.my_slot {
            info!("🎯 DHT Request reached target slot {:?}", self.my_slot);

            // Execute the operation
            let result = match &request.operation {
                DHTOperation::Put { key, value } => {
                    let mut storage = self.local_storage.lock().await;
                    storage.insert_raw(*key, value.clone());
                    info!("✅ DHT PUT executed: stored key {}", hex::encode(&key[..8]));
                    DHTResult::PutOk
                }
                DHTOperation::Get { key } => {
                    let storage = self.local_storage.lock().await;
                    if let Some(value) = storage.get_raw(key) {
                        info!("✅ DHT GET executed: found key {}", hex::encode(&key[..8]));
                        DHTResult::GetOk(value.to_vec())
                    } else {
                        info!("❌ DHT GET executed: key {} not found", hex::encode(&key[..8]));
                        DHTResult::NotFound
                    }
                }
            };

            // Return response
            return Some(DHTResponse {
                request_id: request.request_id,
                from_peer: self.my_peer_id.clone(),
                result,
                hops: request.hops,
            });
        }

        // We're NOT the target - route it closer using greedy routing
        let next_direction = greedy_direction(&self.my_slot, &request.target_slot, &self.mesh_config);

        if let Some(direction) = next_direction {
            debug!("➡️  Routing DHT request via direction {:?}", direction);
            request.current_slot = self.my_slot;

            // TODO: Forward to next hop via relay
            // For now, return None (request will be forwarded by relay)
            None
        } else {
            // Routing failed
            warn!("⚠️  DHT routing failed: no path to {:?}", request.target_slot);
            Some(DHTResponse {
                request_id: request.request_id,
                from_peer: self.my_peer_id.clone(),
                result: DHTResult::Error("Routing failed".to_string()),
                hops: request.hops,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_put_get() {
        let mesh_config = MeshConfig::new(5, 5, 2); // 50 slots
        let my_slot = SlotCoordinate::new(2, 3, 1);
        let storage = Arc::new(Mutex::new(DhtState::new()));

        let dht = DistributedDHT::new(
            "test-peer".to_string(),
            my_slot,
            mesh_config,
            storage,
        );

        // Find a key that hashes to our slot
        let mut test_key = [0u8; 32];
        for i in 0..10000 {
            test_key[0] = (i & 0xFF) as u8;
            test_key[1] = ((i >> 8) & 0xFF) as u8;

            if key_to_slot(test_key, &mesh_config) == my_slot {
                // Found one!
                let test_value = b"test value".to_vec();

                // PUT
                dht.put(test_key, test_value.clone()).await.unwrap();

                // GET
                let result = dht.get(&test_key).await.unwrap();
                assert_eq!(result, Some(test_value));

                return;
            }
        }

        panic!("Could not find key that hashes to test slot");
    }

    #[tokio::test]
    async fn test_remote_put_requires_routing() {
        let mesh_config = MeshConfig::new(5, 5, 2);
        let my_slot = SlotCoordinate::new(2, 3, 1);
        let storage = Arc::new(Mutex::new(DhtState::new()));

        let dht = DistributedDHT::new(
            "test-peer".to_string(),
            my_slot,
            mesh_config,
            storage,
        );

        // Find a key that does NOT hash to our slot
        let mut test_key = [0u8; 32];
        for i in 0..10000 {
            test_key[0] = (i & 0xFF) as u8;
            test_key[1] = ((i >> 8) & 0xFF) as u8;

            if key_to_slot(test_key, &mesh_config) != my_slot {
                // This key belongs to a different slot
                let test_value = b"remote value".to_vec();

                // PUT should fail because no request sender configured
                let result = dht.put(test_key, test_value).await;
                assert!(result.is_err());
                assert!(result.unwrap_err().to_string().contains("not initialized"));

                return;
            }
        }

        panic!("All keys hash to our slot (impossible with 50 slots)");
    }
}
