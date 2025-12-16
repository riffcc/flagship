<template>
  <v-card
    class="d-flex flex-column"
    style="width: 90vw; height: 85vh;"
  >
    <v-card-title class="d-flex justify-space-between align-center pa-4">
      <span>Citadel Mesh - Split Beam Visualization</span>
      <div class="d-flex gap-2">
        <v-btn
          :icon="showLabels ? '$label' : '$label-off'"
          variant="text"
          @click="toggleLabels"
          :title="showLabels ? 'Hide labels' : 'Show labels'"
        ></v-btn>
        <v-btn
          :icon="isDark ? '$weather-sunny' : '$weather-night'"
          variant="text"
          @click="toggleTheme"
        ></v-btn>
        <v-btn
          icon="$refresh"
          variant="text"
          @click="reconnect"
          :loading="connectionState === 'connecting' || connectionState === 'reconnecting'"
        ></v-btn>
      </div>
    </v-card-title>

    <!-- Stats Bar -->
    <v-card-subtitle v-if="meshData" class="px-4 pb-2">
      <v-row dense class="text-caption">
        <v-col>
          <v-icon start size="small">$server-network</v-icon>
          {{ meshData.node_count }} nodes
        </v-col>
        <v-col>
          <v-icon start size="small">$vector-polyline</v-icon>
          {{ meshData.connection_count }} connections
        </v-col>
        <v-col>
          <v-icon start size="small">$speedometer</v-icon>
          {{ meshData.avg_latency.toFixed(1) }}ms avg
        </v-col>
        <v-col>
          <v-icon
            start
            size="small"
            :color="connectionStateColor"
          >
            {{ connectionStateIcon }}
          </v-icon>
          {{ connectionState }}
        </v-col>
      </v-row>
    </v-card-subtitle>

    <v-card-text class="flex-grow-1 pa-0" style="position: relative;">
      <!-- Loading/Error States -->
      <v-overlay
        v-if="connectionState === 'connecting' || connectionState === 'reconnecting'"
        contained
        model-value
        class="align-center justify-center"
      >
        <v-progress-circular
          indeterminate
          size="64"
          color="primary"
        ></v-progress-circular>
        <div class="mt-4">
          {{ connectionState === 'connecting' ? 'Connecting to mesh...' : 'Reconnecting...' }}
        </div>
      </v-overlay>

      <v-overlay
        v-else-if="connectionState === 'error' || connectionState === 'disconnected'"
        contained
        model-value
        class="align-center justify-center"
      >
        <v-alert
          type="error"
          variant="tonal"
          max-width="400"
        >
          <div class="d-flex flex-column align-center">
            <div class="mb-2">Failed to connect to mesh visualization backend</div>
            <v-btn
              color="primary"
              variant="elevated"
              @click="reconnect"
            >
              Retry Connection
            </v-btn>
          </div>
        </v-alert>
      </v-overlay>

      <!-- Canvas Container -->
      <div
        v-show="connectionState === 'connected'"
        ref="canvasContainer"
        style="width: 100%; height: 100%;"
      >
        <canvas ref="canvas"></canvas>
      </div>

      <!-- Split Beam Legend (floating bottom-right) -->
      <v-card
        v-show="connectionState === 'connected'"
        class="split-beam-legend"
        elevation="4"
      >
        <v-card-title class="text-caption pa-2 pb-1">
          Bidirectional Latency
        </v-card-title>
        <v-card-text class="pa-2 pt-0">
          <div class="legend-description text-caption mb-2">
            Each connection shows two beams:
          </div>
          <div class="beam-types mb-2">
            <div class="beam-type-item">
              <div class="beam-indicator" style="border-left: 3px solid #00ff00;"></div>
              <span class="text-caption">Upstream (A → B)</span>
            </div>
            <div class="beam-type-item">
              <div class="beam-indicator" style="border-left: 3px solid #00ff00;"></div>
              <span class="text-caption">Downstream (B → A)</span>
            </div>
          </div>
          <div class="legend-gradient"></div>
          <div class="legend-labels">
            <span class="text-caption">0ms</span>
            <span class="text-caption">200ms</span>
            <span class="text-caption">500ms</span>
            <span class="text-caption">1000ms+</span>
          </div>
          <div class="legend-colors mt-2">
            <div class="color-label">
              <div class="color-box" style="background: #00ff00;"></div>
              <span class="text-caption">Excellent (&lt;50ms)</span>
            </div>
            <div class="color-label">
              <div class="color-box" style="background: #ffff00;"></div>
              <span class="text-caption">Good (50-200ms)</span>
            </div>
            <div class="color-label">
              <div class="color-box" style="background: #ff9900;"></div>
              <span class="text-caption">Poor (200-500ms)</span>
            </div>
            <div class="color-label">
              <div class="color-box" style="background: #ff0000;"></div>
              <span class="text-caption">Bad (&gt;500ms)</span>
            </div>
          </div>
        </v-card-text>
      </v-card>
    </v-card-text>

    <v-card-actions class="pa-2">
      <v-chip size="x-small" variant="text">
        <v-icon start size="small">$alpha-c-circle</v-icon>
        Citadel DHT
      </v-chip>
      <v-chip size="x-small" variant="text">
        <v-icon start size="small">$ray-start-arrow</v-icon>
        Split Beams
      </v-chip>
      <v-spacer></v-spacer>
      <v-chip size="x-small" variant="text" class="text-caption">
        Canvas2D Renderer
      </v-chip>
    </v-card-actions>
  </v-card>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed, nextTick } from 'vue';
import { useTheme } from '/@/composables/useTheme';
import { SplitBeamRenderer } from '/@/lib/split-beam-renderer';
import { MeshClient, ConnectionState } from '/@/lib/mesh-client';
import type { MeshUpdate } from '/@/types/mesh';

const { isDark, toggleTheme } = useTheme();

const canvas = ref<HTMLCanvasElement | null>(null);
const canvasContainer = ref<HTMLDivElement | null>(null);
const meshData = ref<MeshUpdate | null>(null);
const connectionState = ref<ConnectionState>(ConnectionState.DISCONNECTED);
const showLabels = ref(true);

let renderer: SplitBeamRenderer | null = null;
let meshClient: MeshClient | null = null;

// Connection state indicators
const connectionStateColor = computed(() => {
  switch (connectionState.value) {
    case ConnectionState.CONNECTED:
      return 'success';
    case ConnectionState.CONNECTING:
    case ConnectionState.RECONNECTING:
      return 'warning';
    case ConnectionState.ERROR:
    case ConnectionState.DISCONNECTED:
      return 'error';
    default:
      return 'grey';
  }
});

const connectionStateIcon = computed(() => {
  switch (connectionState.value) {
    case ConnectionState.CONNECTED:
      return '$check-circle';
    case ConnectionState.CONNECTING:
    case ConnectionState.RECONNECTING:
      return '$sync';
    case ConnectionState.ERROR:
      return '$alert-circle';
    case ConnectionState.DISCONNECTED:
      return '$close-circle';
    default:
      return '$help-circle';
  }
});

/**
 * Initialize renderer
 */
const initializeRenderer = async () => {
  if (!canvas.value || !canvasContainer.value) {
    console.warn('[P2PMesh] Canvas or container not available');
    return;
  }

  // Clean up existing renderer
  if (renderer) {
    renderer.destroy();
    renderer = null;
  }

  // Get container dimensions
  const width = canvasContainer.value.clientWidth;
  const height = canvasContainer.value.clientHeight;

  console.log('[P2PMesh] Initializing renderer with dimensions:', { width, height });

  // Create renderer
  renderer = new SplitBeamRenderer(canvas.value, {
    width,
    height,
    showLabels: showLabels.value,
    backgroundColor: isDark.value ? '#1a1a1a' : '#f5f5f5',
  });

  // If we have mesh data, update renderer
  if (meshData.value) {
    renderer.updateMesh(meshData.value);
  }

  // Start render loop
  renderer.startRenderLoop();
};

/**
 * Initialize mesh client
 */
const initializeMeshClient = () => {
  if (meshClient) {
    meshClient.destroy();
  }

  // Create mesh client
  // Note: URL should be configured based on environment
  const wsUrl = import.meta.env.VITE_MESH_WS_URL || 'ws://localhost:5000/api/v1/mesh/stream';

  meshClient = new MeshClient({
    url: wsUrl,
    autoReconnect: true,
    reconnectDelay: 5000,
    maxReconnectAttempts: 0, // Infinite retries
  });

  // Set up event handlers
  meshClient.on({
    onUpdate: (update: MeshUpdate) => {
      console.log('[P2PMesh] Received mesh update:', update);
      meshData.value = update;

      // Update renderer with new data
      if (renderer) {
        renderer.updateMesh(update);
      }
    },
    onConnect: () => {
      console.log('[P2PMesh] Connected to mesh stream');
    },
    onDisconnect: () => {
      console.log('[P2PMesh] Disconnected from mesh stream');
    },
    onError: (error: Error) => {
      console.error('[P2PMesh] Mesh client error:', error);
    },
    onStateChange: (state: ConnectionState) => {
      connectionState.value = state;
    },
  });

  // Connect
  meshClient.connect();
};

/**
 * Toggle labels
 */
const toggleLabels = () => {
  showLabels.value = !showLabels.value;
  if (renderer) {
    renderer.updateConfig({ showLabels: showLabels.value });
  }
};

/**
 * Reconnect to mesh
 */
const reconnect = () => {
  if (meshClient) {
    meshClient.disconnect();
    meshClient.connect();
  } else {
    initializeMeshClient();
  }
};

/**
 * Handle window resize
 */
const handleResize = () => {
  if (canvasContainer.value && renderer) {
    const width = canvasContainer.value.clientWidth;
    const height = canvasContainer.value.clientHeight;
    renderer.updateConfig({ width, height });
  }
};

// Initialize on mount
onMounted(async () => {
  await nextTick();
  initializeRenderer();
  initializeMeshClient();

  // Listen for window resize
  window.addEventListener('resize', handleResize);
});

// Update renderer when theme changes
watch(isDark, async () => {
  if (renderer) {
    renderer.updateConfig({
      backgroundColor: isDark.value ? '#1a1a1a' : '#f5f5f5',
    });
  }
});

// Cleanup on unmount
onUnmounted(() => {
  window.removeEventListener('resize', handleResize);

  if (meshClient) {
    meshClient.destroy();
    meshClient = null;
  }

  if (renderer) {
    renderer.destroy();
    renderer = null;
  }
});
</script>

<style scoped>
/* Split Beam Legend - floating bottom-right */
.split-beam-legend {
  position: absolute;
  bottom: 16px;
  right: 16px;
  min-width: 250px;
  backdrop-filter: blur(10px);
  background: rgba(255, 255, 255, 0.95) !important;
}

:deep(.v-theme--dark) .split-beam-legend {
  background: rgba(30, 30, 30, 0.95) !important;
}

/* Beam type indicators */
.beam-types {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.beam-type-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.beam-indicator {
  width: 20px;
  height: 3px;
  border-radius: 2px;
}

/* Gradient bar showing full color spectrum */
.legend-gradient {
  height: 20px;
  border-radius: 4px;
  background: linear-gradient(to right,
    #00ff00 0%,    /* Green at 0ms */
    #ffff00 25%,   /* Yellow */
    #ff9900 50%,   /* Orange */
    #ff0000 75%,   /* Red at 500ms */
    #cc0000 100%   /* Dark red at 1000ms+ */
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

/* Canvas styling */
canvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
