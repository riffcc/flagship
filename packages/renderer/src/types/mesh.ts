/**
 * Citadel Mesh Visualization Types
 *
 * Types for visualizing the Citadel DHT mesh topology with split beam connections
 * showing bidirectional latency between nodes.
 */

/**
 * SPIRAL slot coordinate in the hex mesh
 * Uses axial hex coordinates (q, r) with vertical layers (z)
 */
export interface SlotCoordinate {
  /** SPIRAL index (slot number in enumeration order) */
  index: number | null;
  /** Hex axial coordinate q */
  q: number;
  /** Hex axial coordinate r */
  r: number;
  /** Vertical layer z (2.5D) */
  z: number;
}

/**
 * A node in the Citadel mesh
 */
export interface MeshNode {
  /** Unique node identifier (peer ID) */
  id: string;
  /** 3D slot position in the mesh */
  slot: SlotCoordinate;
  /** Average latency to this node (milliseconds) */
  latency_ms: number;
  /** Node label for display */
  label?: string;
  /** Whether this node is online */
  online: boolean;
  /** Node capabilities */
  capabilities?: string[];
}

/**
 * Multi-window latency statistics for a connection
 */
export interface LatencyStats {
  /** Average latency over the last 1 second */
  last_1s_ms: number | null;
  /** Average latency over the last 60 seconds */
  last_60s_ms: number | null;
  /** Average latency over the last 1 hour */
  last_1h_ms: number | null;
  /** Number of samples in the 1s window */
  samples_1s: number;
  /** Number of samples in the 60s window */
  samples_60s: number;
  /** Number of samples in the 1h window */
  samples_1h: number;
}

/**
 * A bidirectional connection between two nodes
 * Split into upstream and downstream beams for asymmetric latency visualization
 */
export interface MeshConnection {
  /** Source node ID */
  from: string;
  /** Target node ID */
  to: string;
  /** Upstream latency (from -> to) in milliseconds */
  latency_up_ms: number;
  /** Downstream latency (to -> from) in milliseconds */
  latency_down_ms: number;
  /** Multi-window latency statistics */
  latency_stats?: LatencyStats;
  /** Connection type (neighbor, relay, etc.) */
  connection_type?: 'neighbor' | 'relay' | 'replication';
}

/**
 * Complete mesh topology update
 */
export interface MeshUpdate {
  /** Total number of nodes */
  node_count: number;
  /** Total number of connections */
  connection_count: number;
  /** Average latency across all connections */
  avg_latency: number;
  /** All nodes in the mesh */
  nodes: MeshNode[];
  /** All connections in the mesh */
  connections: MeshConnection[];
  /** Timestamp of this update */
  timestamp: number;
}

/**
 * Mesh statistics
 */
export interface MeshStats {
  /** Total nodes in mesh */
  node_count: number;
  /** Total connections */
  connection_count: number;
  /** Average latency (ms) */
  avg_latency: number;
  /** Minimum latency (ms) */
  min_latency: number;
  /** Maximum latency (ms) */
  max_latency: number;
  /** Mesh occupancy percentage */
  occupancy_percent: number;
}

/**
 * Beam geometry for rendering split beams
 */
export interface BeamGeometry {
  /** Start position [x, y, z] */
  start: [number, number, number];
  /** End position [x, y, z] */
  end: [number, number, number];
  /** Beam color (RGBA) */
  color: [number, number, number, number];
  /** Beam width fraction */
  width: number;
}

/**
 * Split beam pair (upstream + downstream)
 */
export interface SplitBeamPair {
  /** Upstream beam (from -> to) */
  upstream: BeamGeometry;
  /** Downstream beam (to -> from) */
  downstream: BeamGeometry;
  /** Connection metadata */
  connection: MeshConnection;
}

/**
 * Latency color thresholds
 */
export const LatencyThresholds = {
  EXCELLENT: 50,    // Green: < 50ms
  GOOD: 200,        // Yellow: 50-200ms
  POOR: 500,        // Orange: 200-500ms
  VERY_POOR: 1000,  // Red: > 500ms
} as const;

/**
 * Convert latency (ms) to RGBA color array
 *
 * Color coding:
 * - Green: < 50ms (excellent)
 * - Yellow: 50-200ms (good)
 * - Orange: 200-500ms (poor)
 * - Red: > 500ms (very poor)
 */
export function latencyToColor(latencyMs: number): [number, number, number, number] {
  if (latencyMs < LatencyThresholds.EXCELLENT) {
    return [0, 1, 0, 1]; // Green
  } else if (latencyMs < LatencyThresholds.GOOD) {
    return [1, 1, 0, 1]; // Yellow
  } else if (latencyMs < LatencyThresholds.POOR) {
    return [1, 0.6, 0, 1]; // Orange
  } else {
    return [1, 0, 0, 1]; // Red
  }
}

/**
 * Convert RGBA array to CSS color string
 */
export function rgbaToCss(rgba: [number, number, number, number]): string {
  const [r, g, b, a] = rgba;
  return `rgba(${Math.floor(r * 255)}, ${Math.floor(g * 255)}, ${Math.floor(b * 255)}, ${a})`;
}
