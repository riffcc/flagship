/**
 * Split Beam Renderer for Citadel Mesh Visualization
 *
 * Renders bidirectional connections as split beams using Canvas2D,
 * showing upstream and downstream latency separately.
 *
 * Ported from citadel-viz Rust implementation.
 */

import type {
  MeshNode,
  MeshConnection,
  MeshUpdate,
  BeamGeometry,
  SplitBeamPair,
  SlotCoordinate,
} from '/@/types/mesh';
import { latencyToColor, rgbaToCss } from '/@/types/mesh';

/**
 * Configuration for split beam rendering
 */
export interface SplitBeamConfig {
  /** Canvas width */
  width: number;
  /** Canvas height */
  height: number;
  /** Beam width as fraction of connection length */
  beamWidthFraction: number;
  /** Separation between beams as fraction of connection length */
  separationFraction: number;
  /** Number of animated particles per beam */
  particleCount: number;
  /** Particle animation speed */
  particleSpeed: number;
  /** Show node labels */
  showLabels: boolean;
  /** Background color */
  backgroundColor: string;
}

/**
 * Default configuration
 */
export const DEFAULT_CONFIG: SplitBeamConfig = {
  width: 1920,
  height: 1080,
  beamWidthFraction: 0.02,
  separationFraction: 0.05,
  particleCount: 5,
  particleSpeed: 0.5,
  showLabels: true,
  backgroundColor: '#1a1a1a',
};

/**
 * 3D vector math utilities
 */
class Vec3 {
  constructor(public x: number, public y: number, public z: number) {}

  static from(coord: SlotCoordinate): Vec3 {
    return new Vec3(coord.x, coord.y, coord.z);
  }

  static subtract(a: Vec3, b: Vec3): Vec3 {
    return new Vec3(a.x - b.x, a.y - b.y, a.z - b.z);
  }

  static add(a: Vec3, b: Vec3): Vec3 {
    return new Vec3(a.x + b.x, a.y + b.y, a.z + b.z);
  }

  static scale(v: Vec3, s: number): Vec3 {
    return new Vec3(v.x * s, v.y * s, v.z * s);
  }

  static cross(a: Vec3, b: Vec3): Vec3 {
    return new Vec3(
      a.y * b.z - a.z * b.y,
      a.z * b.x - a.x * b.z,
      a.x * b.y - a.y * b.x,
    );
  }

  length(): number {
    return Math.sqrt(this.x * this.x + this.y * this.y + this.z * this.z);
  }

  normalize(): Vec3 {
    const len = this.length();
    if (len > 0.0001) {
      return new Vec3(this.x / len, this.y / len, this.z / len);
    }
    return new Vec3(0, 0, 0);
  }

  toArray(): [number, number, number] {
    return [this.x, this.y, this.z];
  }
}

/**
 * Split Beam Renderer using Canvas2D
 */
export class SplitBeamRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private config: SplitBeamConfig;
  private animationTime: number = 0;
  private animationFrameId: number | null = null;
  private meshData: MeshUpdate | null = null;

  constructor(canvas: HTMLCanvasElement, config: Partial<SplitBeamConfig> = {}) {
    this.canvas = canvas;
    this.config = { ...DEFAULT_CONFIG, ...config };

    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Failed to get 2D context from canvas');
    }
    this.ctx = ctx;

    // Set canvas size
    this.canvas.width = this.config.width;
    this.canvas.height = this.config.height;
  }

  /**
   * Update mesh data
   */
  updateMesh(meshData: MeshUpdate): void {
    this.meshData = meshData;
  }

  /**
   * Calculate split beam geometry for a connection
   */
  private calculateSplitBeam(
    connection: MeshConnection,
    nodeMap: Map<string, MeshNode>,
  ): SplitBeamPair | null {
    const fromNode = nodeMap.get(connection.from);
    const toNode = nodeMap.get(connection.to);

    if (!fromNode || !toNode) {
      return null;
    }

    const start = Vec3.from(fromNode.slot);
    const end = Vec3.from(toNode.slot);

    // Calculate direction vector
    const dir = Vec3.subtract(end, start);

    // Calculate perpendicular offset vector using cross product with up vector
    const up = new Vec3(0, 1, 0);
    const perp = Vec3.cross(dir, up).normalize();

    // Calculate beam offsets
    const upstreamOffset = Vec3.scale(perp, -this.config.separationFraction / 2);
    const downstreamOffset = Vec3.scale(perp, this.config.separationFraction / 2);

    // Upstream beam (from -> to, left side)
    const upstreamStart = Vec3.add(start, upstreamOffset);
    const upstreamEnd = Vec3.add(end, upstreamOffset);
    const upstreamColor = latencyToColor(connection.latency_up_ms);

    // Downstream beam (to -> from, right side)
    const downstreamStart = Vec3.add(start, downstreamOffset);
    const downstreamEnd = Vec3.add(end, downstreamOffset);
    const downstreamColor = latencyToColor(connection.latency_down_ms);

    return {
      upstream: {
        start: upstreamStart.toArray(),
        end: upstreamEnd.toArray(),
        color: upstreamColor,
        width: this.config.beamWidthFraction,
      },
      downstream: {
        start: downstreamStart.toArray(),
        end: downstreamEnd.toArray(),
        color: downstreamColor,
        width: this.config.beamWidthFraction,
      },
      connection,
    };
  }

  /**
   * Project 3D coordinates to 2D canvas space
   */
  private project3D(point: [number, number, number]): [number, number] {
    const [x, y, z] = point;

    // Simple isometric projection
    const scale = 20;
    const centerX = this.canvas.width / 2;
    const centerY = this.canvas.height / 2;

    const screenX = centerX + (x - z) * scale * 0.866;
    const screenY = centerY + (y - (x + z) * 0.5) * scale;

    return [screenX, screenY];
  }

  /**
   * Draw a single beam
   */
  private drawBeam(beam: BeamGeometry): void {
    const [startX, startY] = this.project3D(beam.start);
    const [endX, endY] = this.project3D(beam.end);

    // Calculate beam width in pixels
    const dx = endX - startX;
    const dy = endY - startY;
    const length = Math.sqrt(dx * dx + dy * dy);
    const width = Math.max(2, length * beam.width);

    // Draw beam as a line with color based on latency
    this.ctx.strokeStyle = rgbaToCss(beam.color);
    this.ctx.lineWidth = width;
    this.ctx.lineCap = 'round';

    this.ctx.beginPath();
    this.ctx.moveTo(startX, startY);
    this.ctx.lineTo(endX, endY);
    this.ctx.stroke();
  }

  /**
   * Draw animated particles along a beam
   */
  private drawParticles(beam: BeamGeometry): void {
    for (let i = 0; i < this.config.particleCount; i++) {
      // Calculate particle progress with time offset
      const progress = ((i / this.config.particleCount) + this.animationTime * this.config.particleSpeed) % 1.0;

      // Interpolate particle position
      const particlePos: [number, number, number] = [
        beam.start[0] + (beam.end[0] - beam.start[0]) * progress,
        beam.start[1] + (beam.end[1] - beam.start[1]) * progress,
        beam.start[2] + (beam.end[2] - beam.start[2]) * progress,
      ];

      const [px, py] = this.project3D(particlePos);

      // Draw particle as a small circle
      this.ctx.fillStyle = rgbaToCss(beam.color);
      this.ctx.beginPath();
      this.ctx.arc(px, py, 4, 0, Math.PI * 2);
      this.ctx.fill();
    }
  }

  /**
   * Draw a node
   */
  private drawNode(node: MeshNode): void {
    const [x, y] = this.project3D([node.slot.x, node.slot.y, node.slot.z]);

    // Node color based on latency
    const color = latencyToColor(node.latency_ms);
    this.ctx.fillStyle = rgbaToCss(color);

    // Draw node as a circle
    this.ctx.beginPath();
    this.ctx.arc(x, y, 8, 0, Math.PI * 2);
    this.ctx.fill();

    // Draw outline
    this.ctx.strokeStyle = node.online ? '#ffffff' : '#666666';
    this.ctx.lineWidth = 2;
    this.ctx.stroke();

    // Draw label
    if (this.config.showLabels && node.label) {
      this.ctx.fillStyle = '#ffffff';
      this.ctx.font = '12px sans-serif';
      this.ctx.textAlign = 'center';
      this.ctx.fillText(node.label, x, y - 15);
    }
  }

  /**
   * Render a single frame
   */
  private renderFrame(): void {
    if (!this.meshData) {
      return;
    }

    // Clear canvas
    this.ctx.fillStyle = this.config.backgroundColor;
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

    // Build node map for quick lookup
    const nodeMap = new Map<string, MeshNode>();
    for (const node of this.meshData.nodes) {
      nodeMap.set(node.id, node);
    }

    // Draw all connections as split beams
    for (const connection of this.meshData.connections) {
      const splitBeam = this.calculateSplitBeam(connection, nodeMap);
      if (splitBeam) {
        // Draw upstream beam
        this.drawBeam(splitBeam.upstream);
        this.drawParticles(splitBeam.upstream);

        // Draw downstream beam
        this.drawBeam(splitBeam.downstream);
        this.drawParticles(splitBeam.downstream);
      }
    }

    // Draw all nodes on top
    for (const node of this.meshData.nodes) {
      this.drawNode(node);
    }

    // Draw stats overlay
    this.drawStats();
  }

  /**
   * Draw statistics overlay
   */
  private drawStats(): void {
    if (!this.meshData) {
      return;
    }

    const stats = [
      `Nodes: ${this.meshData.node_count}`,
      `Connections: ${this.meshData.connection_count}`,
      `Avg Latency: ${this.meshData.avg_latency.toFixed(1)}ms`,
    ];

    this.ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
    this.ctx.fillRect(10, 10, 220, 80);

    this.ctx.fillStyle = '#ffffff';
    this.ctx.font = '14px monospace';
    this.ctx.textAlign = 'left';

    stats.forEach((stat, i) => {
      this.ctx.fillText(stat, 20, 35 + i * 20);
    });
  }

  /**
   * Start animation loop
   */
  startRenderLoop(): void {
    if (this.animationFrameId !== null) {
      return; // Already running
    }

    const animate = () => {
      this.animationTime += 0.016; // ~60 FPS
      this.renderFrame();
      this.animationFrameId = requestAnimationFrame(animate);
    };

    animate();
  }

  /**
   * Stop animation loop
   */
  stopRenderLoop(): void {
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    this.stopRenderLoop();
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<SplitBeamConfig>): void {
    this.config = { ...this.config, ...config };

    // Update canvas size if changed
    if (config.width !== undefined || config.height !== undefined) {
      this.canvas.width = this.config.width;
      this.canvas.height = this.config.height;
    }
  }
}
