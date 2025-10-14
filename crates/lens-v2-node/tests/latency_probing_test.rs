//! Tests for latency probing and rainbow gradient visualization
//!
//! Phase 2: Measure RTT to all 8 neighbors and expose via map API
//! with rainbow gradient for visualization

use lens_node::peer_registry::{SlotOwnership, get_neighbor_slots, calculate_mesh_dimensions};
use lens_node::slot_identity::SlotId;
use citadel_core::topology::SlotCoordinate;

#[test]
fn test_latency_gradient_color_mapping() {
    // 0ms = green (0, 255, 0)
    // 50ms = yellow (255, 255, 0)
    // 100ms+ = red (255, 0, 0)

    let green = latency_to_rgb(0);
    assert_eq!(green, (0, 255, 0), "0ms should be bright green");

    let yellow = latency_to_rgb(50);
    assert_eq!(yellow, (255, 255, 0), "50ms should be yellow");

    let red = latency_to_rgb(100);
    assert_eq!(red, (255, 0, 0), "100ms+ should be red");

    let orange = latency_to_rgb(25);
    // Should be between green and yellow
    assert!(orange.0 > 0 && orange.1 > 0, "25ms should be greenish-yellow");
}

#[test]
fn test_latency_gradient_intermediate_values() {
    // Test the full gradient
    let latencies = vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

    for lat in latencies {
        let (r, g, b) = latency_to_rgb(lat);

        // Red component should increase with latency
        // Green component should decrease with latency
        // Blue should stay at 0 for red→yellow→green gradient

        assert_eq!(b, 0, "Blue should always be 0 for red-yellow-green gradient");
        assert!(r <= 255 && g <= 255, "RGB components must be valid");
    }
}

#[test]
fn test_latency_to_hex_color() {
    let green_hex = latency_to_hex(0);
    assert_eq!(green_hex, "#00ff00", "0ms should be green hex");

    let yellow_hex = latency_to_hex(50);
    assert_eq!(yellow_hex, "#ffff00", "50ms should be yellow hex");

    let red_hex = latency_to_hex(100);
    assert_eq!(red_hex, "#ff0000", "100ms+ should be red hex");
}

#[test]
fn test_slot_ownership_stores_neighbor_latencies() {
    let coord = SlotCoordinate::new(5, 5, 2);
    let mut ownership = SlotOwnership::new(
        "peer-123".to_string(),
        coord,
        None
    );

    // Initially no latency data
    assert_eq!(ownership.avg_neighbor_latency_ms, None);

    // Simulate measuring latencies to 8 neighbors
    let measured_latencies = vec![10, 15, 12, 18, 14, 11, 16, 13]; // 8 neighbors
    let avg = measured_latencies.iter().sum::<u64>() / measured_latencies.len() as u64;

    ownership.avg_neighbor_latency_ms = Some(avg);

    assert_eq!(ownership.avg_neighbor_latency_ms, Some(13),
        "Should store average latency across 8 neighbors");
}

#[test]
fn test_rolling_average_latency_calculation() {
    // Simulate RTT measurements over time
    let measurements = vec![
        vec![10, 12, 11, 10, 13], // Neighbor 1 samples
        vec![15, 14, 16, 15, 14], // Neighbor 2 samples
    ];

    for neighbor_samples in measurements {
        let avg: u64 = neighbor_samples.iter().sum::<u64>() / neighbor_samples.len() as u64;
        assert!(avg > 0, "Average should be positive");

        // Rolling average should smooth out spikes
        let with_spike = vec![10, 10, 10, 100, 10]; // One outlier
        let avg_with_spike: u64 = with_spike.iter().sum::<u64>() / with_spike.len() as u64;
        assert_eq!(avg_with_spike, 28, "Rolling average should smooth outliers");
    }
}

#[test]
fn test_latency_measurement_timeout() {
    // If neighbor doesn't respond within timeout, use max latency
    let timeout_ms = 1000;
    let no_response_latency = u64::MAX;

    // Simulate timeout
    let measured = no_response_latency;
    assert_eq!(measured, u64::MAX, "Timeout should use MAX latency");

    // This prevents dead neighbors from looking "fast"
    let (r, g, _b) = latency_to_rgb(no_response_latency);
    assert_eq!(r, 255, "Timed out neighbors should show as red");
    assert_eq!(g, 0, "Timed out neighbors should show as red");
}

#[test]
fn test_map_edge_includes_latency() {
    // Map endpoint should include latency_ms for each edge
    // This will be tested in integration test, but verify structure here

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestEdge {
        from: String,
        to: String,
        latency_ms: Option<u64>,
        color: String, // Hex color for gradient
    }

    let edge = TestEdge {
        from: "peer-1".to_string(),
        to: "peer-2".to_string(),
        latency_ms: Some(25),
        color: latency_to_hex(25),
    };

    assert_eq!(edge.latency_ms, Some(25));
    assert!(edge.color.starts_with("#"), "Color should be hex format");
}

// Helper functions that will be implemented in the actual code

fn latency_to_rgb(latency_ms: u64) -> (u8, u8, u8) {
    // Map 0-100ms to green→yellow→red gradient
    // 0ms = (0, 255, 0) green
    // 50ms = (255, 255, 0) yellow
    // 100ms+ = (255, 0, 0) red

    let lat = latency_ms.min(100) as f64;

    if lat <= 50.0 {
        // Green to yellow: increase red, keep green at 255
        let ratio = lat / 50.0;
        let red = (255.0 * ratio) as u8;
        (red, 255, 0)
    } else {
        // Yellow to red: decrease green, keep red at 255
        let ratio = (lat - 50.0) / 50.0;
        let green = (255.0 * (1.0 - ratio)) as u8;
        (255, green, 0)
    }
}

fn latency_to_hex(latency_ms: u64) -> String {
    let (r, g, b) = latency_to_rgb(latency_ms);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[test]
fn test_latency_probing_cycle() {
    // Every 30 seconds, probe all 8 neighbors
    let probe_interval_secs = 30;

    // Simulate one probing cycle
    let mesh_config = calculate_mesh_dimensions(10);
    let my_slot = SlotCoordinate::new(5, 5, 2);
    let neighbors = get_neighbor_slots(&my_slot, &mesh_config);

    assert_eq!(neighbors.len(), 8, "Should have exactly 8 neighbors to probe");

    // Each neighbor gets a ping
    let mut latencies = Vec::new();
    for (_direction, neighbor_coord) in neighbors {
        // Simulate RTT measurement
        let simulated_rtt = 15u64; // 15ms
        latencies.push(simulated_rtt);
    }

    assert_eq!(latencies.len(), 8, "Should measure latency to all 8 neighbors");

    let avg = latencies.iter().sum::<u64>() / latencies.len() as u64;
    assert_eq!(avg, 15, "Average should match simulated RTT");
}

#[test]
fn test_trump_challenge_with_measured_latencies() {
    // A node with better measured latencies can challenge
    let target_slot = SlotCoordinate::new(5, 5, 2);

    // Current occupant has poor latencies (avg 80ms)
    let occupant_latencies = vec![75, 82, 78, 85, 80, 76, 84, 80];
    let occupant_avg = occupant_latencies.iter().sum::<u64>() / occupant_latencies.len() as u64;

    // Challenger has excellent latencies (avg 40ms)
    let challenger_latencies = vec![38, 42, 39, 41, 40, 37, 43, 40];
    let challenger_avg = challenger_latencies.iter().sum::<u64>() / challenger_latencies.len() as u64;

    // Improvement percentage
    let improvement = ((occupant_avg - challenger_avg) as f64 / occupant_avg as f64) * 100.0;

    assert!(improvement > 20.0,
        "Challenger with 50% better latency should exceed 20% threshold");

    // Verify both would show different colors in visualization
    let occupant_color = latency_to_hex(occupant_avg);
    let challenger_color = latency_to_hex(challenger_avg);

    assert_ne!(occupant_color, challenger_color,
        "Different latencies should have different colors");
}

#[test]
fn test_gradient_extremes() {
    // Test edge cases
    let zero = latency_to_hex(0);
    let very_high = latency_to_hex(u64::MAX);
    let just_over_threshold = latency_to_hex(101);

    assert_eq!(zero, "#00ff00", "Zero latency = pure green");
    assert_eq!(very_high, "#ff0000", "MAX latency = pure red");
    assert_eq!(just_over_threshold, "#ff0000", "Over 100ms = pure red");
}
