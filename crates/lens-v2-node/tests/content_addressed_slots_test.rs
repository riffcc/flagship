//! Integration tests for Content Addressed Slots (Phase 1)
//!
//! Tests the foundation of latency-based slot trumping:
//! - SlotId generation and stability
//! - SlotOwnership with permanent IDs
//! - Map endpoint exposing slot_ids
//! - Trump challenge logic

use lens_node::peer_registry::{SlotOwnership, peer_id_to_slot, calculate_mesh_dimensions};
use lens_node::slot_identity::{SlotId, TrumpChallenge, LatencyProof, SwapPhase, SlotSwapCoordination};
use citadel_core::topology::{SlotCoordinate, MeshConfig};

#[test]
fn test_slot_id_is_permanent_for_coordinate() {
    let coord = SlotCoordinate::new(5, 10, 2);

    // Generate slot ID multiple times
    let id1 = SlotId::from_coordinate(coord);
    let id2 = SlotId::from_coordinate(coord);
    let id3 = SlotId::from_coordinate(coord);

    // All should be identical
    assert_eq!(id1, id2);
    assert_eq!(id2, id3);
    assert_eq!(id1.to_hex(), id2.to_hex());
}

#[test]
fn test_slot_id_unique_per_coordinate() {
    let coord1 = SlotCoordinate::new(5, 10, 2);
    let coord2 = SlotCoordinate::new(5, 11, 2); // Different y
    let coord3 = SlotCoordinate::new(6, 10, 2); // Different x

    let id1 = SlotId::from_coordinate(coord1);
    let id2 = SlotId::from_coordinate(coord2);
    let id3 = SlotId::from_coordinate(coord3);

    // All should be different
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn test_slot_ownership_includes_slot_id() {
    let coord = SlotCoordinate::new(3, 7, 1);
    let ownership = SlotOwnership::new(
        "peer-123".to_string(),
        coord,
        Some("ws://localhost:5000/api/v1/relay/ws".to_string())
    );

    // SlotId should be automatically generated
    let expected_slot_id = SlotId::from_coordinate(coord);
    assert_eq!(ownership.slot_id, expected_slot_id);
    assert_eq!(ownership.slot, coord);
}

#[test]
fn test_slot_id_stable_across_mesh_resize() {
    // Original mesh: 6×6×1 = 36 slots
    let config1 = MeshConfig::new(6, 6, 1);
    let coord = SlotCoordinate::new(2, 3, 0);
    let slot_id_small = SlotId::from_coordinate(coord);

    // Mesh grows to 10×10×1 = 100 slots
    let config2 = MeshConfig::new(10, 10, 1);
    // Coordinate stays the same, so slot_id should too
    let slot_id_large = SlotId::from_coordinate(coord);

    assert_eq!(slot_id_small, slot_id_large,
        "SlotId must remain stable when mesh dimensions change");
}

// NOTE: We don't use XOR distance! We use greedy routing on hex toroid.
// Routing tests are in Citadel's routing module.

#[test]
fn test_trump_challenge_meets_threshold() {
    let target_slot = SlotCoordinate::new(5, 5, 2);

    let challenge = TrumpChallenge {
        challenger_peer_id: "peer-fast".to_string(),
        target_slot_id: SlotId::from_coordinate(target_slot),
        target_coordinate: target_slot,
        current_occupant_peer_id: "peer-slow".to_string(),
        latency_proofs: vec![], // Will add proofs in separate test
        challenger_avg_latency_ms: 50,  // 50% better!
        occupant_avg_latency_ms: 100,
        proposed_reassignment: SlotId::from_coordinate(SlotCoordinate::new(6, 5, 2)),
        proposed_coordinate: SlotCoordinate::new(6, 5, 2),
        timestamp: 0,
        challenger_signature: vec![],
    };

    assert!(challenge.meets_threshold(),
        "50% improvement should meet 20% threshold");
    assert_eq!(challenge.improvement_percentage(), 50.0);
}

#[test]
fn test_trump_challenge_fails_below_threshold() {
    let target_slot = SlotCoordinate::new(5, 5, 2);

    let challenge = TrumpChallenge {
        challenger_peer_id: "peer-barely-faster".to_string(),
        target_slot_id: SlotId::from_coordinate(target_slot),
        target_coordinate: target_slot,
        current_occupant_peer_id: "peer-current".to_string(),
        latency_proofs: vec![],
        challenger_avg_latency_ms: 95,  // Only 5% better
        occupant_avg_latency_ms: 100,
        proposed_reassignment: SlotId::from_coordinate(SlotCoordinate::new(6, 5, 2)),
        proposed_coordinate: SlotCoordinate::new(6, 5, 2),
        timestamp: 0,
        challenger_signature: vec![],
    };

    assert!(!challenge.meets_threshold(),
        "5% improvement should not meet 20% threshold");
}

#[test]
fn test_trump_challenge_exact_threshold() {
    let target_slot = SlotCoordinate::new(5, 5, 2);

    let challenge = TrumpChallenge {
        challenger_peer_id: "peer-exactly-20".to_string(),
        target_slot_id: SlotId::from_coordinate(target_slot),
        target_coordinate: target_slot,
        current_occupant_peer_id: "peer-current".to_string(),
        latency_proofs: vec![],
        challenger_avg_latency_ms: 80,  // Exactly 20% better
        occupant_avg_latency_ms: 100,
        proposed_reassignment: SlotId::from_coordinate(SlotCoordinate::new(6, 5, 2)),
        proposed_coordinate: SlotCoordinate::new(6, 5, 2),
        timestamp: 0,
        challenger_signature: vec![],
    };

    // At exactly 20% threshold, should still meet it (< not <=)
    assert!(!challenge.meets_threshold(),
        "Exactly at threshold should NOT meet (needs to be strictly better)");
}

#[test]
fn test_latency_proof_average() {
    let proofs = vec![
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(0, 0, 0)),
            neighbor_peer_id: "peer-1".to_string(),
            rtt_ms: 10,
            timestamp: 100,
            nonce: 1,
            neighbor_signature: vec![],
        },
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(1, 0, 0)),
            neighbor_peer_id: "peer-2".to_string(),
            rtt_ms: 20,
            timestamp: 100,
            nonce: 2,
            neighbor_signature: vec![],
        },
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(2, 0, 0)),
            neighbor_peer_id: "peer-3".to_string(),
            rtt_ms: 30,
            timestamp: 100,
            nonce: 3,
            neighbor_signature: vec![],
        },
    ];

    let avg = LatencyProof::average_latency(&proofs);
    assert_eq!(avg, 20, "Average of 10, 20, 30 should be 20");
}

#[test]
fn test_latency_proof_empty_returns_max() {
    let proofs = vec![];
    let avg = LatencyProof::average_latency(&proofs);
    assert_eq!(avg, u64::MAX, "Empty proofs should return MAX");
}

#[test]
fn test_trump_challenge_requires_8_proofs() {
    let target_slot = SlotCoordinate::new(5, 5, 2);

    // Only 6 proofs (not enough!)
    let incomplete_proofs = vec![
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(0, 0, 0)),
            neighbor_peer_id: "peer-1".to_string(),
            rtt_ms: 10,
            timestamp: 100,
            nonce: 1,
            neighbor_signature: vec![],
        },
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(1, 0, 0)),
            neighbor_peer_id: "peer-2".to_string(),
            rtt_ms: 10,
            timestamp: 100,
            nonce: 2,
            neighbor_signature: vec![],
        },
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(2, 0, 0)),
            neighbor_peer_id: "peer-3".to_string(),
            rtt_ms: 10,
            timestamp: 100,
            nonce: 3,
            neighbor_signature: vec![],
        },
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(3, 0, 0)),
            neighbor_peer_id: "peer-4".to_string(),
            rtt_ms: 10,
            timestamp: 100,
            nonce: 4,
            neighbor_signature: vec![],
        },
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(4, 0, 0)),
            neighbor_peer_id: "peer-5".to_string(),
            rtt_ms: 10,
            timestamp: 100,
            nonce: 5,
            neighbor_signature: vec![],
        },
        LatencyProof {
            neighbor_slot_id: SlotId::from_coordinate(SlotCoordinate::new(5, 0, 0)),
            neighbor_peer_id: "peer-6".to_string(),
            rtt_ms: 10,
            timestamp: 100,
            nonce: 6,
            neighbor_signature: vec![],
        },
    ];

    let challenge = TrumpChallenge {
        challenger_peer_id: "peer-challenger".to_string(),
        target_slot_id: SlotId::from_coordinate(target_slot),
        target_coordinate: target_slot,
        current_occupant_peer_id: "peer-current".to_string(),
        latency_proofs: incomplete_proofs,
        challenger_avg_latency_ms: 50,
        occupant_avg_latency_ms: 100,
        proposed_reassignment: SlotId::from_coordinate(SlotCoordinate::new(6, 5, 2)),
        proposed_coordinate: SlotCoordinate::new(6, 5, 2),
        timestamp: 0,
        challenger_signature: vec![],
    };

    assert!(!challenge.verify_proofs(),
        "Challenge must have exactly 8 latency proofs (one per neighbor)");
}

#[test]
fn test_swap_coordination_phases() {
    let swap = SlotSwapCoordination {
        swap_id: "swap-123".to_string(),
        phase: SwapPhase::PreConnect,
        node_a_peer_id: "peer-1".to_string(),
        node_a_old_slot: SlotId::from_coordinate(SlotCoordinate::new(0, 0, 0)),
        node_a_new_slot: SlotId::from_coordinate(SlotCoordinate::new(5, 5, 2)),
        node_b_peer_id: "peer-2".to_string(),
        node_b_old_slot: SlotId::from_coordinate(SlotCoordinate::new(5, 5, 2)),
        node_b_new_slot: SlotId::from_coordinate(SlotCoordinate::new(0, 0, 0)),
        timestamp: 0,
    };

    assert_eq!(swap.phase, SwapPhase::PreConnect);

    // Test all phases can be created
    let phases = vec![
        SwapPhase::PreConnect,
        SwapPhase::AnnounceNew,
        SwapPhase::DropOld,
        SwapPhase::Verify,
        SwapPhase::Complete,
        SwapPhase::Failed("test failure".to_string()),
    ];

    assert_eq!(phases.len(), 6, "Should have 6 swap phases");
}

#[test]
fn test_slot_id_hex_roundtrip() {
    let coord = SlotCoordinate::new(7, 3, 1);
    let original = SlotId::from_coordinate(coord);

    // Convert to hex and back
    let hex = original.to_hex();
    let parsed = SlotId::from_hex(&hex).expect("Should parse valid hex");

    assert_eq!(original, parsed, "Hex roundtrip should preserve SlotId");
    assert_eq!(hex.len(), 64, "Hex should be 64 chars (32 bytes * 2)");
}

#[test]
fn test_slot_id_short_hex() {
    let coord = SlotCoordinate::new(7, 3, 1);
    let id = SlotId::from_coordinate(coord);

    let short = id.to_short_hex();
    assert_eq!(short.len(), 16, "Short hex should be 16 chars (8 bytes * 2)");

    let full = id.to_hex();
    assert!(full.starts_with(&short), "Full hex should start with short hex");
}

#[test]
fn test_slot_ownership_latency_field() {
    let coord = SlotCoordinate::new(5, 7, 2);
    let mut ownership = SlotOwnership::new(
        "peer-123".to_string(),
        coord,
        None
    );

    // Initially no latency data
    assert_eq!(ownership.avg_neighbor_latency_ms, None);

    // Update with measured latency
    ownership.avg_neighbor_latency_ms = Some(45);
    assert_eq!(ownership.avg_neighbor_latency_ms, Some(45));
}

#[test]
fn test_multiple_peers_same_mesh_different_slot_ids() {
    let config = calculate_mesh_dimensions(10);

    let peers = vec![
        "peer-1",
        "peer-2",
        "peer-3",
        "peer-4",
        "peer-5",
    ];

    let mut slot_ids = std::collections::HashSet::new();

    for peer_id in peers {
        let slot = peer_id_to_slot(peer_id, &config);
        let slot_id = SlotId::from_coordinate(slot);

        // Each peer should get a unique slot ID
        assert!(slot_ids.insert(slot_id),
            "Each peer should get a unique slot ID");
    }
}

// Routing logic uses Citadel's greedy_direction() on hex toroid topology
// Tests for that are in citadel-core/src/routing.rs

#[test]
fn test_slot_id_display_format() {
    let coord = SlotCoordinate::new(5, 7, 2);
    let id = SlotId::from_coordinate(coord);

    let display = format!("{}", id);
    assert!(display.starts_with("slot-"),
        "Display format should start with 'slot-'");

    let debug = format!("{:?}", id);
    assert!(debug.starts_with("SlotId("),
        "Debug format should start with 'SlotId('");
}
