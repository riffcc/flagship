//! P2P Heartbeat Protocol for WebRTC Mesh
//!
//! This module implements a heartbeat system for tracking alive peers in the mesh.
//! Each node broadcasts heartbeats through WebRTC data channels to its mesh neighbors.

use citadel_core::topology::SlotCoordinate;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Heartbeat message sent between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Peer ID of the sender
    pub peer_id: String,

    /// Slot coordinate in the mesh
    pub slot: SlotCoordinate,

    /// Timestamp (Unix seconds)
    pub timestamp: u64,

    /// Node capabilities
    pub capabilities: Vec<String>,

    /// Average latency to 8 neighbors (milliseconds, if known)
    pub avg_neighbor_latency_ms: Option<u64>,

    /// Protocol version
    pub protocol_version: String,
}

impl Heartbeat {
    /// Create a new heartbeat message
    pub fn new(
        peer_id: String,
        slot: SlotCoordinate,
        capabilities: Vec<String>,
        avg_neighbor_latency_ms: Option<u64>,
    ) -> Self {
        Self {
            peer_id,
            slot,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            capabilities,
            avg_neighbor_latency_ms,
            protocol_version: "0.8.36".to_string(),
        }
    }

    /// Convert heartbeat to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse heartbeat from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Check if heartbeat is stale (older than 30 seconds)
    pub fn is_stale(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        (now - self.timestamp) > 30
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_creation() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let heartbeat = Heartbeat::new(
            "peer-123".to_string(),
            slot,
            vec!["webrtc".to_string(), "dht".to_string()],
            Some(25),
        );

        assert_eq!(heartbeat.peer_id, "peer-123");
        assert_eq!(heartbeat.slot, slot);
        assert_eq!(heartbeat.capabilities.len(), 2);
        assert_eq!(heartbeat.avg_neighbor_latency_ms, Some(25));
        assert!(!heartbeat.is_stale());
    }

    #[test]
    fn test_heartbeat_json_roundtrip() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let heartbeat = Heartbeat::new(
            "peer-123".to_string(),
            slot,
            vec!["webrtc".to_string()],
            None,
        );

        let json = heartbeat.to_json().unwrap();
        let parsed = Heartbeat::from_json(&json).unwrap();

        assert_eq!(parsed.peer_id, heartbeat.peer_id);
        assert_eq!(parsed.slot, heartbeat.slot);
        assert_eq!(parsed.capabilities, heartbeat.capabilities);
    }

    #[test]
    fn test_heartbeat_staleness() {
        let slot = SlotCoordinate::new(5, 10, 2);
        let mut heartbeat = Heartbeat::new(
            "peer-123".to_string(),
            slot,
            vec!["webrtc".to_string()],
            None,
        );

        // Fresh heartbeat should not be stale
        assert!(!heartbeat.is_stale());

        // Simulate old timestamp (60 seconds ago)
        heartbeat.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 60;

        assert!(heartbeat.is_stale(), "Old heartbeat should be stale");
    }
}
