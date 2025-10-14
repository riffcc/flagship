//! Latency Measurement and Rainbow Gradient Visualization
//!
//! Measures RTT to all 8 neighbors and maps to rainbow gradient for visualization.
//! - 0ms = bright green (#00ff00)
//! - 50ms = yellow (#ffff00)
//! - 100ms+ = red (#ff0000)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Latency measurement for a neighbor connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    /// Neighbor peer ID
    pub peer_id: String,

    /// Most recent RTT in milliseconds
    pub rtt_ms: u64,

    /// Rolling average RTT (last 10 samples)
    pub avg_rtt_ms: u64,

    /// Timestamp of last measurement
    pub last_measured: u64,

    /// Sample count for rolling average
    samples: Vec<u64>,
}

impl LatencyMeasurement {
    /// Create a new latency measurement
    pub fn new(peer_id: String, initial_rtt_ms: u64) -> Self {
        Self {
            peer_id,
            rtt_ms: initial_rtt_ms,
            avg_rtt_ms: initial_rtt_ms,
            last_measured: current_timestamp(),
            samples: vec![initial_rtt_ms],
        }
    }

    /// Add a new RTT sample and update rolling average
    pub fn add_sample(&mut self, rtt_ms: u64) {
        self.rtt_ms = rtt_ms;
        self.samples.push(rtt_ms);

        // Keep only last 10 samples for rolling average
        if self.samples.len() > 10 {
            self.samples.remove(0);
        }

        // Recalculate average
        self.avg_rtt_ms = self.samples.iter().sum::<u64>() / self.samples.len() as u64;
        self.last_measured = current_timestamp();
    }

    /// Check if measurement is stale (>60 seconds old)
    pub fn is_stale(&self) -> bool {
        let now = current_timestamp();
        (now - self.last_measured) > 60
    }
}

/// Latency tracker for all neighbor connections
#[derive(Debug, Clone)]
pub struct LatencyTracker {
    /// Map of peer_id to latency measurement
    measurements: HashMap<String, LatencyMeasurement>,
}

impl LatencyTracker {
    /// Create a new latency tracker
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
        }
    }

    /// Record a latency measurement for a peer
    pub fn record(&mut self, peer_id: String, rtt_ms: u64) {
        match self.measurements.get_mut(&peer_id) {
            Some(measurement) => measurement.add_sample(rtt_ms),
            None => {
                self.measurements.insert(peer_id.clone(), LatencyMeasurement::new(peer_id, rtt_ms));
            }
        }
    }

    /// Get latency for a specific peer
    pub fn get(&self, peer_id: &str) -> Option<&LatencyMeasurement> {
        self.measurements.get(peer_id)
    }

    /// Get average latency across all neighbors
    pub fn average_latency(&self) -> Option<u64> {
        if self.measurements.is_empty() {
            return None;
        }

        let sum: u64 = self.measurements.values().map(|m| m.avg_rtt_ms).sum();
        Some(sum / self.measurements.len() as u64)
    }

    /// Clean up stale measurements
    pub fn remove_stale(&mut self) {
        self.measurements.retain(|_, m| !m.is_stale());
    }
}

/// Convert latency (milliseconds) to RGB color for rainbow gradient
///
/// Gradient mapping:
/// - 0ms = (0, 255, 0) bright green
/// - 50ms = (255, 255, 0) yellow
/// - 100ms+ = (255, 0, 0) red
///
/// This creates a smooth green → yellow → red gradient that makes
/// low-latency connections visually obvious (green) and high-latency
/// connections stand out (red).
pub fn latency_to_rgb(latency_ms: u64) -> (u8, u8, u8) {
    // Clamp to 0-100ms range
    let lat = latency_ms.min(100) as f64;

    if lat <= 50.0 {
        // Green to yellow: increase red component, keep green at 255
        let ratio = lat / 50.0;
        let red = (255.0 * ratio) as u8;
        (red, 255, 0)
    } else {
        // Yellow to red: decrease green component, keep red at 255
        let ratio = (lat - 50.0) / 50.0;
        let green = (255.0 * (1.0 - ratio)) as u8;
        (255, green, 0)
    }
}

/// Convert latency to hex color string
///
/// # Examples
/// ```
/// # use lens_node::latency::latency_to_hex;
/// assert_eq!(latency_to_hex(0), "#00ff00");  // Green
/// assert_eq!(latency_to_hex(50), "#ffff00"); // Yellow
/// assert_eq!(latency_to_hex(100), "#ff0000"); // Red
/// ```
pub fn latency_to_hex(latency_ms: u64) -> String {
    let (r, g, b) = latency_to_rgb(latency_ms);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_to_rgb_green() {
        let (r, g, b) = latency_to_rgb(0);
        assert_eq!((r, g, b), (0, 255, 0));
    }

    #[test]
    fn test_latency_to_rgb_yellow() {
        let (r, g, b) = latency_to_rgb(50);
        assert_eq!((r, g, b), (255, 255, 0));
    }

    #[test]
    fn test_latency_to_rgb_red() {
        let (r, g, b) = latency_to_rgb(100);
        assert_eq!((r, g, b), (255, 0, 0));
    }

    #[test]
    fn test_latency_to_hex() {
        assert_eq!(latency_to_hex(0), "#00ff00");
        assert_eq!(latency_to_hex(50), "#ffff00");
        assert_eq!(latency_to_hex(100), "#ff0000");
    }

    #[test]
    fn test_latency_measurement_rolling_average() {
        let mut measurement = LatencyMeasurement::new("peer-1".to_string(), 10);
        assert_eq!(measurement.avg_rtt_ms, 10);

        measurement.add_sample(20);
        assert_eq!(measurement.avg_rtt_ms, 15); // (10 + 20) / 2

        measurement.add_sample(30);
        assert_eq!(measurement.avg_rtt_ms, 20); // (10 + 20 + 30) / 3
    }

    #[test]
    fn test_latency_tracker() {
        let mut tracker = LatencyTracker::new();

        tracker.record("peer-1".to_string(), 10);
        tracker.record("peer-2".to_string(), 20);

        assert_eq!(tracker.get("peer-1").unwrap().rtt_ms, 10);
        assert_eq!(tracker.get("peer-2").unwrap().rtt_ms, 20);

        let avg = tracker.average_latency().unwrap();
        assert_eq!(avg, 15); // (10 + 20) / 2
    }

    #[test]
    fn test_gradient_intermediate() {
        // Test smooth gradient
        let (r25, g25, _) = latency_to_rgb(25);
        assert!(r25 > 0 && r25 < 255, "25ms should be between green and yellow");
        assert_eq!(g25, 255, "Green should still be maxed at 25ms");

        let (r75, g75, _) = latency_to_rgb(75);
        assert_eq!(r75, 255, "Red should be maxed at 75ms");
        assert!(g75 > 0 && g75 < 255, "75ms should be between yellow and red");
    }
}
