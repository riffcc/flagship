<template>
  <v-card
    class="d-flex flex-column"
    style="width: 90vw; height: 85vh;"
  >
    <v-card-title class="d-flex justify-space-between align-center pa-4">
      <span>Network Mesh Topology</span>
    </v-card-title>

    <!-- Stats Bar -->
    <v-card-subtitle v-if="networkMap" class="px-4 pb-2">
      <v-row dense class="text-caption">
        <v-col>
          <v-icon start size="small">$server</v-icon>
          {{ networkMap.stats.total_peers }} peers
        </v-col>
        <v-col>
          <v-icon start size="small" color="primary">$hexagon-multiple</v-icon>
          {{ networkMap.stats.server_nodes }} servers
        </v-col>
        <v-col>
          <v-icon start size="small" color="secondary">$web</v-icon>
          {{ networkMap.stats.browser_peers }} browsers
        </v-col>
        <v-col>
          <v-icon start size="small">$vector-polyline</v-icon>
          {{ networkMap.stats.mesh_edges }} edges
        </v-col>
        <v-col>
          <v-icon start size="small">$percent</v-icon>
          {{ networkMap.stats.occupancy_percent.toFixed(1) }}% occupancy
        </v-col>
      </v-row>
    </v-card-subtitle>

    <v-card-text class="flex-grow-1 pa-0" style="position: relative;">
      <!-- Loading State -->
      <v-overlay
        v-if="loading"
        contained
        model-value
        class="align-center justify-center"
      >
        <v-progress-circular
          indeterminate
          size="64"
          color="primary"
        ></v-progress-circular>
        <div class="mt-4">Loading network map...</div>
      </v-overlay>

      <!-- WebGL Not Supported State -->
      <v-overlay
        v-else-if="!webglSupported"
        contained
        model-value
        class="align-center justify-center"
      >
        <v-alert
          type="warning"
          variant="tonal"
          max-width="400"
        >
          <div class="text-h6 mb-2">3D Visualization Unavailable</div>
          <div>Your browser does not support WebGL, which is required for the 3D network visualization.</div>
          <div class="mt-2 text-caption">Try using a modern browser like Chrome, Firefox, or Edge with hardware acceleration enabled.</div>
        </v-alert>
      </v-overlay>

      <!-- Error State -->
      <v-overlay
        v-else-if="error"
        contained
        model-value
        class="align-center justify-center"
      >
        <v-alert
          type="error"
          variant="tonal"
          max-width="400"
        >
          {{ error.message }}
        </v-alert>
      </v-overlay>

      <!-- 3D Force Graph Container -->
      <div
        v-show="webglSupported && !loading && !error"
        ref="graphContainer"
        style="width: 100%; height: 100%;"
      ></div>

      <!-- Latency Legend (floating bottom-right) -->
      <v-card
        v-show="webglSupported && !loading && !error"
        class="latency-legend"
        elevation="4"
      >
        <v-card-title class="text-caption pa-2 pb-1">
          Bidirectional Latency
        </v-card-title>
        <v-card-text class="pa-2 pt-0">
          <div class="legend-description text-caption mb-2">
            Each connection shows two parallel beams (upstream/downstream)
          </div>
          <div class="legend-gradient"></div>
          <div class="legend-labels">
            <span class="text-caption">0ms</span>
            <span class="text-caption">150ms</span>
            <span class="text-caption">300ms</span>
            <span class="text-caption">1000ms+</span>
          </div>
          <div class="legend-colors mt-1">
            <div class="color-label">
              <div class="color-box" style="background: #00ff00;"></div>
              <span class="text-caption">Good</span>
            </div>
            <div class="color-label">
              <div class="color-box rainbow-gradient"></div>
              <span class="text-caption">Medium</span>
            </div>
            <div class="color-label">
              <div class="color-box" style="background: #ff0000;"></div>
              <span class="text-caption">High</span>
            </div>
            <div class="color-label">
              <div class="color-box" style="background: #880088;"></div>
              <span class="text-caption">Dead</span>
            </div>
          </div>
        </v-card-text>
      </v-card>
    </v-card-text>

    <v-card-actions class="pa-2">
      <v-btn
        size="small"
        variant="text"
        prepend-icon="$refresh"
        @click="refresh"
      >
        Refresh
      </v-btn>
      <v-spacer></v-spacer>
      <v-chip size="x-small" variant="text">
        <v-icon start size="small" color="primary">$hexagon</v-icon>
        Server
      </v-chip>
      <v-chip size="x-small" variant="text">
        <v-icon start size="small" color="secondary">$circle-small</v-icon>
        Browser
      </v-chip>
      <v-chip size="x-small" variant="text">
        <v-icon start size="small" color="success">$circle-small</v-icon>
        DHT
      </v-chip>
    </v-card-actions>
  </v-card>
</template>

<script setup lang="ts">
// IMPORTANT: Import WebGPU shim FIRST before any Three.js or 3d-force-graph imports
// This provides fallback constants for browsers without WebGPU support (e.g., Firefox)
import { isWebGLSupported } from '/@/lib/webgpu-shim';

import { ref, onMounted, watch, nextTick, onBeforeUnmount, computed } from 'vue';
import ForceGraph3D from '3d-force-graph';
import SpriteText from 'three-spritetext';
import * as THREE from 'three';
import { useNetworkMap } from '/@/composables/useNetworkMap';
import { useTheme } from '/@/composables/useTheme';

// Check WebGL support on component load
const webglSupported = isWebGLSupported();

const emit = defineEmits<{
  close: [];
}>();

const { networkMap, loading, error, fetchNetworkMap } = useNetworkMap();
const { isDark, currentTheme } = useTheme();
const graphContainer = ref<HTMLDivElement | null>(null);

let graphInstance: any = null;

// Theme-aware colors
const serverNodeColor = computed(() => currentTheme.value.colors['server-node']);
const browserNodeColor = computed(() => currentTheme.value.colors['browser-node']);
const backgroundColor = computed(() => currentTheme.value.colors.background);

/**
 * Calculate latency color using pH strip-style gradient
 * 0ms = pure bright green
 * 0-300ms = rainbow gradient (green → yellow → orange → red)
 * 300-1000ms = red to dark red
 * 1000ms+ = black (dead link)
 */
const getLatencyColor = (latencyMs: number | null | undefined): string => {
  if (latencyMs === null || latencyMs === undefined) {
    return 'rgba(128, 128, 128, 0.4)'; // Gray for unknown latency
  }

  if (latencyMs >= 1000) {
    return '#880088'; // Dark magenta for dead links (1000ms+) - visible on both light and dark backgrounds
  }

  if (latencyMs >= 300) {
    // 300-1000ms: Red to dark magenta
    const t = (latencyMs - 300) / 700; // 0 at 300ms, 1 at 1000ms
    const r = Math.floor(255 - (255 - 136) * t); // 255 -> 136 (0x88)
    const b = Math.floor(136 * t); // 0 -> 136 (0x88)
    return `rgb(${r}, 0, ${b})`;
  }

  // 0-300ms: Rainbow gradient (green → yellow → orange → red)
  const t = latencyMs / 300; // 0 at 0ms, 1 at 300ms

  if (t < 0.25) {
    // 0-75ms: Green to yellow-green
    const green = 255;
    const red = Math.floor(255 * (t / 0.25) * 0.5); // 0 to 127
    return `rgb(${red}, ${green}, 0)`;
  } else if (t < 0.5) {
    // 75-150ms: Yellow-green to yellow
    const green = 255;
    const red = Math.floor(127 + 128 * ((t - 0.25) / 0.25)); // 127 to 255
    return `rgb(${red}, ${green}, 0)`;
  } else if (t < 0.75) {
    // 150-225ms: Yellow to orange
    const red = 255;
    const green = Math.floor(255 * (1 - (t - 0.5) / 0.25) * 0.5 + 127); // 255 to 127
    return `rgb(${red}, ${green}, 0)`;
  } else {
    // 225-300ms: Orange to red
    const red = 255;
    const green = Math.floor(127 * (1 - (t - 0.75) / 0.25)); // 127 to 0
    return `rgb(${red}, ${green}, 0)`;
  }
};

const initializeGraph = () => {
  if (!webglSupported || !graphContainer.value || !networkMap.value) return;

  // Clear any existing graph
  if (graphInstance) {
    graphInstance._destructor();
    graphInstance = null;
  }

  // Transform network map data to 3d-force-graph format
  const graphData = {
    nodes: networkMap.value.nodes.map(node => ({
      id: node.id,
      name: node.label,
      peer_type: node.peer_type,
      slot: node.slot,
      capabilities: node.capabilities,
      online: node.online,
    })),
    links: networkMap.value.edges.map(edge => {
      // Determine if this is a browser connection (one endpoint is browser)
      const fromNode = networkMap.value!.nodes.find(n => n.id === edge.from);
      const toNode = networkMap.value!.nodes.find(n => n.id === edge.to);
      const isBrowserConnection = fromNode?.peer_type === 'browser' || toNode?.peer_type === 'browser';

      return {
        source: edge.from,
        target: edge.to,
        type: edge.connection_type,
        latency_ms: edge.latency_ms,
        latency_stats: edge.latency_stats,
        color: edge.color,
        is_browser_connection: isBrowserConnection,
      };
    }),
  };

  // Initialize 3D force graph
  graphInstance = new ForceGraph3D(graphContainer.value)
    .graphData(graphData)
    .backgroundColor(backgroundColor.value)

    // Node styling
    .nodeLabel((node: any) => {
      const n = node as any;
      const slotInfo = n.slot.index !== null
        ? `Slot #${n.slot.index} (${n.slot.q}, ${n.slot.r}, ${n.slot.z})`
        : 'Unclaimed';
      return `${n.name}<br/>${slotInfo}<br/>Type: ${n.peer_type}<br/>Capabilities: ${n.capabilities.join(', ')}`;
    })
    .nodeColor((node: any) => {
      const n = node as any;
      // Theme-aware colors: Server nodes use primary color, Browser nodes use theme-specific color
      return n.peer_type === 'server' ? serverNodeColor.value : browserNodeColor.value;
    })
    .nodeVal((node: any) => {
      const n = node as any;
      // Server nodes bigger than browser nodes
      return n.peer_type === 'server' ? 8 : 3;
    })
    .nodeThreeObject((node: any) => {
      const n = node as any;
      const sprite = new SpriteText(n.name);
      sprite.color = n.peer_type === 'server' ? serverNodeColor.value : browserNodeColor.value;
      sprite.textHeight = n.peer_type === 'server' ? 8 : 6;
      // SpriteText extends THREE.Sprite which has position property
      (sprite as THREE.Object3D).position.y = n.peer_type === 'server' ? 20 : 15;
      return sprite;
    })
    .nodeThreeObjectExtend(true)  // Extend default sphere with our text sprite

    // Custom split beam link rendering - two parallel lines per connection
    .linkThreeObject((link: any) => {
      const l = link as any;

      // Get source and target nodes (they'll be populated by the force graph)
      const source = l.source as any;
      const target = l.target as any;

      // For now, use same latency for both directions
      // TODO: Get separate up/down latency from backend when available
      const latencyUp = l.latency_ms || 0;
      const latencyDown = l.latency_ms || 0;

      // Create a group to hold both beams
      const group = new THREE.Group();

      // Calculate direction vector
      const start = new THREE.Vector3(source.x || 0, source.y || 0, source.z || 0);
      const end = new THREE.Vector3(target.x || 0, target.y || 0, target.z || 0);
      const dir = new THREE.Vector3().subVectors(end, start);
      const length = dir.length();

      // Calculate perpendicular offset for split beams
      const up = new THREE.Vector3(0, 1, 0);
      const perp = new THREE.Vector3().crossVectors(dir, up).normalize();

      // Offset distance (5% of connection length, minimum 2 units)
      const offsetDist = Math.max(2, length * 0.05);

      // Upstream beam (left side, A→B)
      const upstreamOffset = perp.clone().multiplyScalar(-offsetDist / 2);
      const upstreamStart = start.clone().add(upstreamOffset);
      const upstreamEnd = end.clone().add(upstreamOffset);

      const upstreamGeometry = new THREE.BufferGeometry().setFromPoints([upstreamStart, upstreamEnd]);
      const upstreamMaterial = new THREE.LineBasicMaterial({
        color: getLatencyColor(latencyUp),
        opacity: l.is_browser_connection ? 0.6 : 0.8,
        transparent: true,
        linewidth: 2,
      });
      const upstreamLine = new THREE.Line(upstreamGeometry, upstreamMaterial);
      group.add(upstreamLine);

      // Downstream beam (right side, B→A)
      const downstreamOffset = perp.clone().multiplyScalar(offsetDist / 2);
      const downstreamStart = start.clone().add(downstreamOffset);
      const downstreamEnd = end.clone().add(downstreamOffset);

      const downstreamGeometry = new THREE.BufferGeometry().setFromPoints([downstreamStart, downstreamEnd]);
      const downstreamMaterial = new THREE.LineBasicMaterial({
        color: getLatencyColor(latencyDown),
        opacity: l.is_browser_connection ? 0.6 : 0.8,
        transparent: true,
        linewidth: 2,
      });
      const downstreamLine = new THREE.Line(downstreamGeometry, downstreamMaterial);
      group.add(downstreamLine);

      // Add directional particles for relay connections
      if (l.type === 'relay') {
        // Create small spheres as particles along the upstream beam
        const particleGeometry = new THREE.SphereGeometry(1, 8, 8);
        const particleMaterial = new THREE.MeshBasicMaterial({
          color: getLatencyColor(latencyUp),
          opacity: 0.8,
          transparent: true,
        });

        for (let i = 0; i < 3; i++) {
          const t = i / 3;
          const particlePos = new THREE.Vector3().lerpVectors(upstreamStart, upstreamEnd, t);
          const particle = new THREE.Mesh(particleGeometry, particleMaterial);
          particle.position.copy(particlePos);
          group.add(particle);
        }
      }

      return group;
    })
    .linkThreeObjectExtend(false) // Replace default link rendering with our custom beams
    .linkLabel((link: any) => {
      const l = link as any;
      const stats = l.latency_stats;

      // Show multi-window latency stats if available
      if (stats && (stats.last_1s_ms !== null || stats.last_60s_ms !== null || stats.last_1h_ms !== null)) {
        const fmt = (v: number | null) => v !== null ? `${v.toFixed(1)}ms` : '—';
        return [
          `<b>${l.type}</b>`,
          `Last 1s: ${fmt(stats.last_1s_ms)} (${stats.samples_1s || 0} samples)`,
          `Last 60s: ${fmt(stats.last_60s_ms)} (${stats.samples_60s || 0} samples)`,
          `Last 1h: ${fmt(stats.last_1h_ms)} (${stats.samples_1h || 0} samples)`,
        ].join('<br/>');
      }

      // Fallback to old single-value latency
      if (l.latency_ms !== null && l.latency_ms !== undefined) {
        return `${l.type} connection<br/>Latency: ${l.latency_ms}ms`;
      }
      return `${l.type} connection`;
    })

    // Controls
    .enableNodeDrag(true)
    .enableNavigationControls(true)
    .showNavInfo(false)

    // Camera
    .cameraPosition({ z: 400 })

    // Interaction
    .onNodeHover((node: any) => {
      if (graphContainer.value) {
        graphContainer.value.style.cursor = node ? 'pointer' : 'default';
      }
    })
    .onNodeClick((node: any) => {
      if (!graphInstance) return;
      const n = node as any;
      // Focus camera on clicked node
      const distance = 150;
      const distRatio = 1 + distance / Math.hypot(n.x, n.y, n.z);

      graphInstance.cameraPosition(
        { x: n.x * distRatio, y: n.y * distRatio, z: n.z * distRatio },
        n,
        1000,
      );
    });

  // Start animation
  graphInstance.d3Force('charge').strength(-120);
  graphInstance.d3Force('link').distance(50);
};

const refresh = async () => {
  await fetchNetworkMap();
  await nextTick();
  initializeGraph();
};

// Initialize on mount
onMounted(async () => {
  await fetchNetworkMap();
  await nextTick();
  initializeGraph();
});

// Reinitialize when network map changes
watch(() => networkMap.value, async () => {
  await nextTick();
  initializeGraph();
});

// Reinitialize when theme changes
watch([isDark, serverNodeColor, browserNodeColor, backgroundColor], async () => {
  await nextTick();
  initializeGraph();
});

// Cleanup on unmount
onBeforeUnmount(() => {
  // Clean up graph instance
  if (graphInstance) {
    graphInstance._destructor();
    graphInstance = null;
  }
});
</script>

<style scoped>
/* Latency Legend - floating bottom-right */
.latency-legend {
  position: absolute;
  bottom: 16px;
  right: 16px;
  min-width: 200px;
  backdrop-filter: blur(10px);
  background: rgba(255, 255, 255, 0.9) !important;
}

:deep(.v-theme--dark) .latency-legend {
  background: rgba(30, 30, 30, 0.9) !important;
}

/* Gradient bar showing full color spectrum */
.legend-gradient {
  height: 20px;
  border-radius: 4px;
  background: linear-gradient(to right,
    #00ff00 0%,    /* Pure green at 0ms */
    #7fff00 15%,   /* Yellow-green */
    #ffff00 25%,   /* Yellow */
    #ff7f00 40%,   /* Orange */
    #ff0000 50%,   /* Red at 300ms */
    #bb0044 75%,   /* Red-magenta transition */
    #880088 100%   /* Dark magenta at 1000ms+ (visible on dark backgrounds) */
  );
  margin-bottom: 4px;
  border: 1px solid rgba(0, 0, 0, 0.2);
}

/* Labels below gradient */
.legend-labels {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
}

/* Color boxes with labels */
.legend-colors {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.color-label {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-box {
  width: 16px;
  height: 16px;
  border-radius: 3px;
  border: 1px solid rgba(0, 0, 0, 0.3);
  flex-shrink: 0;
}

.rainbow-gradient {
  background: linear-gradient(to right, #7fff00, #ffff00, #ff7f00);
}
</style>
