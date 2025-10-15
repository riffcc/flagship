<template>
  <div class="header">
    <div class="title">🔷 Scalable Hexagonal Toroidal Mesh</div>
    <div class="stats">
      <div class="stat">
        <span class="stat-label">Total Slots:</span>
        <span class="stat-value">{{ totalSlots.toLocaleString() }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Mesh:</span>
        <span class="stat-value">{{ meshWidth }}×{{ meshHeight }}×{{ meshDepth }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">FPS:</span>
        <span class="stat-value">{{ fps }}</span>
      </div>
    </div>
    <div class="controls">
      <label>Width:</label>
      <input type="number" v-model.number="widthInput" min="8" max="128" step="8" />
      <label>Height:</label>
      <input type="number" v-model.number="heightInput" min="8" max="128" step="8" />
      <label>Layers:</label>
      <input type="number" v-model.number="depthInput" min="4" max="16" step="2" />
      <button @click="generate">Generate</button>
      <button @click="$emit('toggle-scenario')">
        {{ scenarioActive ? '⏸️ Pause' : '▶️ Scenario' }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const props = defineProps({
  meshWidth: Number,
  meshHeight: Number,
  meshDepth: Number,
  totalSlots: Number,
  fps: Number,
  scenarioActive: Boolean,
})

const emit = defineEmits(['update-dimensions', 'generate', 'toggle-scenario'])

const widthInput = ref(props.meshWidth)
const heightInput = ref(props.meshHeight)
const depthInput = ref(props.meshDepth)

function generate() {
  emit('update-dimensions', {
    width: widthInput.value,
    height: heightInput.value,
    depth: depthInput.value,
  })
  emit('generate')
}
</script>

<style scoped>
.header {
  background: #1a1a1a;
  padding: 1rem 2rem;
  border-bottom: 2px solid #00ff88;
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 10;
}

.title {
  font-size: 1.5rem;
  font-weight: bold;
  color: #00ff88;
}

.stats {
  display: flex;
  gap: 2rem;
  font-size: 0.9rem;
}

.stat {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.stat-label {
  color: #888;
}

.stat-value {
  color: #00ff88;
  font-weight: bold;
}

.controls {
  display: flex;
  gap: 1rem;
  align-items: center;
}

label {
  color: #ccc;
  font-size: 0.9rem;
}

input[type="number"] {
  background: #2a2a2a;
  border: 1px solid #444;
  color: #fff;
  padding: 0.4rem;
  border-radius: 4px;
  width: 70px;
  text-align: center;
}

input[type="number"]:focus {
  outline: none;
  border-color: #00ff88;
}

button {
  background: #00ff88;
  color: #0a0a0a;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
  transition: all 0.2s;
}

button:hover {
  background: #00cc6a;
  transform: translateY(-1px);
}
</style>
