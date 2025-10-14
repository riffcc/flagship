//! Content Addressed Slots (CAS)
//!
//! Provides permanent, content-addressed identities for mesh slots that are independent
//! of peer identities. This enables:
//! - Stable routing that survives peer churn
//! - Latency-based slot trumping and optimization
//! - Content persistence at mesh locations
//! - Global consensus on network topology

use blake3;
use citadel_core::topology::SlotCoordinate;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A permanent, content-addressed identifier for a mesh slot
///
/// SlotIds are deterministically generated from slot coordinates using Blake3,
/// ensuring they remain stable even when mesh dimensions change or peers rotate.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlotId(pub [u8; 32]); // 256-bit Blake3 hash

impl SlotId {
    /// Create a stable content-addressed ID from slot coordinate
    ///
    /// The same coordinate always produces the same SlotId, making it
    /// a permanent address for that mesh location.
    ///
    /// # Format
    /// `blake3("citadel-slot-v1:{x}:{y}:{z}")`
    pub fn from_coordinate(coord: SlotCoordinate) -> Self {
        let canonical = format!("citadel-slot-v1:{}:{}:{}", coord.x, coord.y, coord.z);
        let hash = blake3::hash(canonical.as_bytes());
        SlotId(*hash.as_bytes())
    }

    // NOTE: We do NOT use XOR distance routing!
    // We have greedy routing on a 2.5D hexagonal toroid, which is BETTER.
    // Citadel's greedy_direction() gives us O(1) next-hop calculation.
    // We can even calculate the ENTIRE path from source to dest (source routing!)

    /// Convert to hex string for display/logging
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Convert to short hex (first 8 bytes) for compact display
    pub fn to_short_hex(&self) -> String {
        hex::encode(&self.0[..8])
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(SlotId(arr))
    }
}

impl fmt::Display for SlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "slot-{}", self.to_short_hex())
    }
}

impl fmt::Debug for SlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SlotId({})", self.to_short_hex())
    }
}

/// Latency proof for a neighbor connection
///
/// Cryptographically signed by the neighbor to prevent gaming the system.
/// Used in trump challenges to prove latency claims.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LatencyProof {
    /// Slot ID of the neighbor
    pub neighbor_slot_id: SlotId,

    /// Peer ID of the neighbor (current occupant)
    pub neighbor_peer_id: String,

    /// Round-trip time in milliseconds
    pub rtt_ms: u64,

    /// Timestamp of measurement (Unix seconds)
    pub timestamp: u64,

    /// Challenge nonce (prevents replay attacks)
    pub nonce: u64,

    /// Ed25519 signature from neighbor confirming the measurement
    /// Signs: blake3(neighbor_slot_id || neighbor_peer_id || rtt_ms || timestamp || nonce)
    pub neighbor_signature: Vec<u8>,
}

impl LatencyProof {
    /// Verify that the latency proof signature is valid
    pub fn verify(&self, neighbor_public_key: &[u8]) -> bool {
        // TODO: Implement Ed25519 signature verification
        // For now, return true (will implement with ed25519-dalek)
        true
    }

    /// Calculate average latency from multiple proofs
    pub fn average_latency(proofs: &[LatencyProof]) -> u64 {
        if proofs.is_empty() {
            return u64::MAX;
        }
        let sum: u64 = proofs.iter().map(|p| p.rtt_ms).sum();
        sum / proofs.len() as u64
    }
}

/// Trump challenge - attempt to claim a slot based on superior latency
///
/// This is like What.CD's trump system, but for mesh topology optimization.
/// A node can challenge the current occupant of a slot if it can prove
/// significantly better latency to all 8 neighbors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrumpChallenge {
    /// Peer ID of the challenger
    pub challenger_peer_id: String,

    /// Slot ID being challenged
    pub target_slot_id: SlotId,

    /// Coordinate of target slot
    pub target_coordinate: SlotCoordinate,

    /// Current occupant of the slot
    pub current_occupant_peer_id: String,

    /// Latency proofs to all 8 neighbors (cryptographically signed)
    pub latency_proofs: Vec<LatencyProof>,

    /// Average challenger latency (calculated from proofs)
    pub challenger_avg_latency_ms: u64,

    /// Current occupant's average latency (for comparison)
    pub occupant_avg_latency_ms: u64,

    /// Proposed reassignment slot for current occupant
    pub proposed_reassignment: SlotId,

    /// Proposed coordinate for current occupant
    pub proposed_coordinate: SlotCoordinate,

    /// Timestamp of challenge
    pub timestamp: u64,

    /// Challenger's signature over the entire challenge
    pub challenger_signature: Vec<u8>,
}

impl TrumpChallenge {
    /// Check if this challenge meets the trump threshold
    ///
    /// Challenger must have at least 20% better latency than current occupant
    /// to prevent frequent unnecessary swaps.
    pub fn meets_threshold(&self) -> bool {
        if self.challenger_avg_latency_ms == 0 || self.occupant_avg_latency_ms == 0 {
            return false;
        }

        // Challenger must be 20% better
        let threshold = (self.occupant_avg_latency_ms as f64 * 0.8) as u64;
        self.challenger_avg_latency_ms < threshold
    }

    /// Verify all latency proofs are valid
    pub fn verify_proofs(&self) -> bool {
        // TODO: Verify each proof's signature with neighbor's public key
        // For now, return true (will implement with public key lookups)
        self.latency_proofs.len() == 8 // Must have proofs for all 8 neighbors
    }

    /// Calculate latency improvement percentage
    pub fn improvement_percentage(&self) -> f64 {
        if self.occupant_avg_latency_ms == 0 {
            return 0.0;
        }
        let diff = self.occupant_avg_latency_ms as f64 - self.challenger_avg_latency_ms as f64;
        (diff / self.occupant_avg_latency_ms as f64) * 100.0
    }
}

/// Slot swap coordination message
///
/// Used to coordinate zero-downtime handoffs between two nodes swapping slots.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotSwapCoordination {
    /// Swap transaction ID (for tracking)
    pub swap_id: String,

    /// Phase of the swap
    pub phase: SwapPhase,

    /// Node A (challenger)
    pub node_a_peer_id: String,
    pub node_a_old_slot: SlotId,
    pub node_a_new_slot: SlotId,

    /// Node B (current occupant)
    pub node_b_peer_id: String,
    pub node_b_old_slot: SlotId,
    pub node_b_new_slot: SlotId,

    /// Timestamp
    pub timestamp: u64,
}

/// Phases of coordinated slot swap
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SwapPhase {
    /// Phase 1: Pre-connect to new neighbors (both nodes)
    PreConnect,

    /// Phase 2: Announce new slot ownership to DHT
    AnnounceNew,

    /// Phase 3: Drop old neighbor connections
    DropOld,

    /// Phase 4: Verify mesh connectivity
    Verify,

    /// Swap completed successfully
    Complete,

    /// Swap failed, rollback
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_id_deterministic() {
        let coord = SlotCoordinate { x: 2, y: 3, z: 0 };
        let id1 = SlotId::from_coordinate(coord);
        let id2 = SlotId::from_coordinate(coord);
        assert_eq!(id1, id2, "Same coordinate should produce same SlotId");
    }

    #[test]
    fn test_slot_id_unique() {
        let coord1 = SlotCoordinate { x: 2, y: 3, z: 0 };
        let coord2 = SlotCoordinate { x: 2, y: 4, z: 0 };
        let id1 = SlotId::from_coordinate(coord1);
        let id2 = SlotId::from_coordinate(coord2);
        assert_ne!(id1, id2, "Different coordinates should produce different SlotIds");
    }

    // XOR distance removed - we use greedy routing on hex toroid instead!

    #[test]
    fn test_hex_roundtrip() {
        let coord = SlotCoordinate { x: 5, y: 7, z: 1 };
        let id = SlotId::from_coordinate(coord);
        let hex = id.to_hex();
        let parsed = SlotId::from_hex(&hex).expect("Should parse hex");
        assert_eq!(id, parsed, "Hex roundtrip should preserve SlotId");
    }

    #[test]
    fn test_trump_threshold() {
        let challenge = TrumpChallenge {
            challenger_peer_id: "peer-1".to_string(),
            target_slot_id: SlotId([0u8; 32]),
            target_coordinate: SlotCoordinate { x: 0, y: 0, z: 0 },
            current_occupant_peer_id: "peer-2".to_string(),
            latency_proofs: vec![],
            challenger_avg_latency_ms: 80,  // 20% better
            occupant_avg_latency_ms: 100,
            proposed_reassignment: SlotId([1u8; 32]),
            proposed_coordinate: SlotCoordinate { x: 1, y: 0, z: 0 },
            timestamp: 0,
            challenger_signature: vec![],
        };

        assert!(
            challenge.meets_threshold(),
            "20% improvement should meet threshold"
        );

        let close_challenge = TrumpChallenge {
            challenger_avg_latency_ms: 85, // Only 15% better
            ..challenge.clone()
        };

        assert!(
            !close_challenge.meets_threshold(),
            "15% improvement should not meet threshold"
        );
    }

    #[test]
    fn test_improvement_percentage() {
        let challenge = TrumpChallenge {
            challenger_peer_id: "peer-1".to_string(),
            target_slot_id: SlotId([0u8; 32]),
            target_coordinate: SlotCoordinate { x: 0, y: 0, z: 0 },
            current_occupant_peer_id: "peer-2".to_string(),
            latency_proofs: vec![],
            challenger_avg_latency_ms: 50,
            occupant_avg_latency_ms: 100,
            proposed_reassignment: SlotId([1u8; 32]),
            proposed_coordinate: SlotCoordinate { x: 1, y: 0, z: 0 },
            timestamp: 0,
            challenger_signature: vec![],
        };

        let improvement = challenge.improvement_percentage();
        assert!((improvement - 50.0).abs() < 0.01, "Should be 50% improvement");
    }
}
