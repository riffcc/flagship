<template>
  <div class="control-panel">
    <h3>🔷 WebGL Hexagonal Grid</h3>

    <div class="control-section">
      <h4>Performance</h4>
      <p class="stat-highlight">
        Rendering {{ totalSlots.toLocaleString() }} slots @ {{ fps }} FPS
      </p>
      <p class="hint">Orbit controls • Scroll to zoom</p>
    </div>

    <div class="control-section">
      <h4>Visualization</h4>
      <div class="slider-group">
        <label>Hex Size: {{ hexSize }}</label>
        <input
          type="range"
          :value="hexSize"
          @input="$emit('update:hex-size', Number($event.target.value))"
          min="1"
          max="5"
          step="0.5"
        />
      </div>
      <div class="slider-group">
        <label>Layer Spacing: {{ layerSpacing }}</label>
        <input
          type="range"
          :value="layerSpacing"
          @input="$emit('update:layer-spacing', Number($event.target.value))"
          min="3"
          max="20"
          step="1"
        />
      </div>
    </div>

    <div class="control-section">
      <h4>Traffic Simulation</h4>
      <div class="slider-group">
        <label>Active Transmissions: {{ activeTransmissions }}</label>
        <input
          type="range"
          :value="maxActiveTransmissions"
          @input="$emit('update:max-active-transmissions', Number($event.target.value))"
          min="10"
          max="500"
          step="10"
        />
      </div>
      <div class="slider-group">
        <label>Link Latency: {{ linkLatencyMs }}ms</label>
        <input
          type="range"
          :value="linkLatencyMs"
          @input="$emit('update:link-latency-ms', Number($event.target.value))"
          min="10"
          max="200"
          step="10"
        />
      </div>
    </div>

    <div v-if="actionStatus" class="action-status">
      {{ actionStatus }}
    </div>
  </div>
</template>

<script setup>
defineProps({
  totalSlots: Number,
  fps: Number,
  hexSize: Number,
  layerSpacing: Number,
  activeTransmissions: Number,
  maxActiveTransmissions: Number,
  linkLatencyMs: Number,
  actionStatus: String,
})

defineEmits([
  'update:hex-size',
  'update:layer-spacing',
  'update:max-active-transmissions',
  'update:link-latency-ms',
])
</script>

<style scoped>
.control-panel {
  position: absolute;
  top: 1rem;
  left: 1rem;
  background: rgba(26, 26, 26, 0.95);
  padding: 1rem;
  border-radius: 8px;
  border: 1px solid #00ff88;
  max-width: 350px;
  font-size: 0.85rem;
}

h3 {
  color: #00ff88;
  margin-bottom: 0.5rem;
}

.control-section {
  margin: 1rem 0;
  padding: 0.5rem 0;
  border-top: 1px solid #333;
}

.control-section:first-child {
  border-top: none;
}

h4 {
  color: #00cc6a;
  font-size: 0.9rem;
  margin-bottom: 0.5rem;
}

.stat-highlight {
  color: #00ff88;
  margin: 0.5rem 0;
}

.hint {
  color: #888;
  font-size: 0.75rem;
  margin: 0;
}

.slider-group {
  padding: 0.5rem 0;
}

.slider-group label {
  display: block;
  margin-bottom: 0.3rem;
  color: #ccc;
}

input[type="range"] {
  width: 100%;
}

.action-status {
  background: #1a1a1a;
  padding: 0.5rem;
  border-radius: 4px;
  margin-top: 0.5rem;
  color: #00ff88;
  font-weight: bold;
}
</style>
