<template>
  <v-card
    class="d-flex flex-column"
    style="width: 90vw; height: 85vh;"
  >
    <v-card-title class="d-flex justify-space-between align-center pa-4">
      <span>Network Mesh Topology</span>
      <div class="d-flex gap-2">
        <v-btn
          :icon="isDark ? 'mdi-weather-sunny' : 'mdi-weather-night'"
          variant="text"
          @click="toggleTheme"
        ></v-btn>
        <v-btn
          icon="mdi-close"
          variant="text"
          @click="$emit('close')"
        ></v-btn>
      </div>
    </v-card-title>

    <!-- Stats Bar -->
    <v-card-subtitle v-if="networkMap" class="px-4 pb-2">
      <v-row dense class="text-caption">
        <v-col>
          <v-icon start size="small">mdi-server</v-icon>
          {{ networkMap.stats.total_peers }} peers
        </v-col>
        <v-col>
          <v-icon start size="small" color="primary">mdi-hexagon-multiple</v-icon>
          {{ networkMap.stats.server_nodes }} servers
        </v-col>
        <v-col>
          <v-icon start size="small" color="secondary">mdi-web</v-icon>
          {{ networkMap.stats.browser_peers }} browsers
        </v-col>
        <v-col>
          <v-icon start size="small">mdi-vector-polyline</v-icon>
          {{ networkMap.stats.mesh_edges }} edges
        </v-col>
        <v-col>
          <v-icon start size="small">mdi-percent</v-icon>
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
        v-show="!loading && !error"
        ref="graphContainer"
        style="width: 100%; height: 100%;"
      ></div>
    </v-card-text>

    <v-card-actions class="pa-2">
      <v-btn
        size="small"
        variant="text"
        prepend-icon="mdi-refresh"
        @click="refresh"
      >
        Refresh
      </v-btn>
      <v-spacer></v-spacer>
      <v-chip size="x-small" variant="text">
        <v-icon start size="small" color="primary">mdi-hexagon</v-icon>
        Server
      </v-chip>
      <v-chip size="x-small" variant="text">
        <v-icon start size="small" color="secondary">mdi-circle-small</v-icon>
        Browser
      </v-chip>
      <v-chip size="x-small" variant="text">
        <v-icon start size="small" color="success">mdi-circle-small</v-icon>
        DHT
      </v-chip>
    </v-card-actions>
  </v-card>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick, onBeforeUnmount, computed } from 'vue';
import ForceGraph3D from '3d-force-graph';
import SpriteText from 'three-spritetext';
import { useNetworkMap } from '/@/composables/useNetworkMap';
import { useTheme } from '/@/composables/useTheme';

const emit = defineEmits<{
  close: [];
}>();

const { networkMap, loading, error, fetchNetworkMap, connectWebSocket, disconnectWebSocket } = useNetworkMap();
const { isDark, currentTheme, toggleTheme } = useTheme();
const graphContainer = ref<HTMLDivElement | null>(null);

let graphInstance: any = null;

// Theme-aware colors
const serverNodeColor = computed(() => currentTheme.value.colors['server-node']);
const browserNodeColor = computed(() => currentTheme.value.colors['browser-node']);
const backgroundColor = computed(() => currentTheme.value.colors.background);

const initializeGraph = () => {
  if (!graphContainer.value || !networkMap.value) return;

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
        color: edge.color,
        is_browser_connection: isBrowserConnection,
      };
    }),
  };

  // Initialize 3D force graph
  graphInstance = ForceGraph3D()(graphContainer.value)
    .graphData(graphData)
    .backgroundColor(backgroundColor.value)

    // Node styling
    .nodeLabel(node => {
      const n = node as any;
      return `${n.name}<br/>Slot: (${n.slot.x}, ${n.slot.y}, ${n.slot.z})<br/>Type: ${n.peer_type}<br/>Capabilities: ${n.capabilities.join(', ')}`;
    })
    .nodeColor(node => {
      const n = node as any;
      // Theme-aware colors: Server nodes use primary color, Browser nodes use theme-specific color
      return n.peer_type === 'server' ? serverNodeColor.value : browserNodeColor.value;
    })
    .nodeVal(node => {
      const n = node as any;
      // Server nodes bigger than browser nodes
      return n.peer_type === 'server' ? 8 : 3;
    })
    .nodeThreeObject(node => {
      const n = node as any;
      const sprite = new SpriteText(n.name);
      sprite.color = n.peer_type === 'server' ? serverNodeColor.value : browserNodeColor.value;
      sprite.textHeight = n.peer_type === 'server' ? 8 : 6;
      sprite.position.y = n.peer_type === 'server' ? 20 : 15;
      return sprite;
    })
    .nodeThreeObjectExtend(true)  // Extend default sphere with our text sprite

    // Link styling
    .linkColor(link => {
      const l = link as any;
      // Use rainbow gradient color from backend (green=low latency, red=high latency)
      // If no color specified, fall back to connection type colors
      if (l.color) {
        return l.color;
      }
      // Fallback: Neighbor connections: purple, Relay connections: orange
      return l.type === 'neighbor' ? 'rgba(138, 43, 226, 0.4)' : 'rgba(255, 152, 0, 0.6)';
    })
    .linkWidth(link => {
      const l = link as any;
      return l.type === 'neighbor' ? 2 : 2;  // Slightly thicker for visibility
    })
    .linkOpacity(link => {
      const l = link as any;
      // Browser connections slightly more transparent
      return l.is_browser_connection ? 0.6 : 0.8;
    })
    .linkLineDash(link => {
      const l = link as any;
      // Dashed lines for browser-to-server connections
      return l.is_browser_connection ? [5, 5] : null;
    })
    .linkDashScale(1)
    .linkDashGap(2)
    .linkLabel(link => {
      const l = link as any;
      if (l.latency_ms !== null && l.latency_ms !== undefined) {
        return `${l.type} connection<br/>Latency: ${l.latency_ms}ms`;
      }
      return `${l.type} connection`;
    })
    .linkDirectionalParticles(link => {
      const l = link as any;
      // Show particles on relay connections
      return l.type === 'relay' ? 2 : 0;
    })
    .linkDirectionalParticleWidth(2)
    .linkDirectionalParticleSpeed(0.005)

    // Controls
    .enableNodeDrag(true)
    .enableNavigationControls(true)
    .showNavInfo(false)

    // Camera
    .cameraPosition({ z: 400 })

    // Interaction
    .onNodeHover(node => {
      if (graphContainer.value) {
        graphContainer.value.style.cursor = node ? 'pointer' : 'default';
      }
    })
    .onNodeClick(node => {
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
  // Connect to WebSocket for real-time updates
  connectWebSocket();

  // Fetch initial network map
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
  // Disconnect WebSocket
  disconnectWebSocket();

  // Clean up graph instance
  if (graphInstance) {
    graphInstance._destructor();
    graphInstance = null;
  }
});
</script>

<style scoped>
/* Ensure container takes full space */
</style>
