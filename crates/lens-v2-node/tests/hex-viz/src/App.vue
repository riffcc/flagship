<template>
  <div id="app">
    <Header
      :mesh-width="meshWidth"
      :mesh-height="meshHeight"
      :mesh-depth="meshDepth"
      :total-slots="totalSlots"
      :fps="fps"
      @update-dimensions="updateDimensions"
      @generate="regenerateMesh"
      @toggle-scenario="toggleScenario"
      :scenario-active="scenarioMode"
    />

    <div class="main-container">
      <HexRenderer
        ref="renderer"
        :mesh-width="meshWidth"
        :mesh-height="meshHeight"
        :mesh-depth="meshDepth"
        :hex-size="hexSize"
        :layer-spacing="layerSpacing"
        :active-transmissions="activeTransmissions"
        @fps-update="fps = $event"
      />

      <ControlPanel
        :total-slots="totalSlots"
        :fps="fps"
        :hex-size="hexSize"
        :layer-spacing="layerSpacing"
        :active-transmissions="activeTransmissions.length"
        :max-active-transmissions="maxActiveTransmissions"
        :link-latency-ms="linkLatencyMs"
        :action-status="actionStatus"
        @update:hex-size="hexSize = $event"
        @update:layer-spacing="layerSpacing = $event"
        @update:max-active-transmissions="maxActiveTransmissions = $event"
        @update:link-latency-ms="linkLatencyMs = $event"
      />

      <Toast :message="toastMessage" />
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import Header from './components/Header.vue'
import HexRenderer from './components/HexRenderer.vue'
import ControlPanel from './components/ControlPanel.vue'
import Toast from './components/Toast.vue'

// Mesh configuration
const meshWidth = ref(32)
const meshHeight = ref(32)
const meshDepth = ref(8)
const totalSlots = computed(() => meshWidth.value * meshHeight.value * meshDepth.value)

// Rendering settings
const hexSize = ref(2)
const layerSpacing = ref(5)
const fps = ref(0)

// Traffic simulation
const activeTransmissions = ref([])
const maxActiveTransmissions = ref(100)
const linkLatencyMs = ref(50)
const scenarioMode = ref(false)
const scenarioInterval = ref(null)

// UI
const actionStatus = ref(null)
const toastMessage = ref(null)

// Refs
const renderer = ref(null)

function updateDimensions({ width, height, depth }) {
  meshWidth.value = width
  meshHeight.value = height
  meshDepth.value = depth
}

function regenerateMesh() {
  if (renderer.value) {
    renderer.value.regenerateMesh()
  }
  actionStatus.value = `✅ Generated ${totalSlots.value.toLocaleString()} hexagonal slots`
  setTimeout(() => { actionStatus.value = null }, 3000)
}

function toggleScenario() {
  scenarioMode.value = !scenarioMode.value

  if (scenarioMode.value) {
    startScenario()
    actionStatus.value = '🎬 Scenario Mode: ACTIVE'
  } else {
    stopScenario()
    actionStatus.value = '⏸️ Scenario Mode: PAUSED'
    setTimeout(() => { actionStatus.value = null }, 2000)
  }
}

function startScenario() {
  if (!renderer.value) return

  scenarioInterval.value = setInterval(() => {
    const numTransmissions = Math.min(
      5,
      maxActiveTransmissions.value - activeTransmissions.value.length
    )

    for (let i = 0; i < numTransmissions; i++) {
      if (renderer.value) {
        const transmission = renderer.value.createRandomTransmission()
        if (transmission) {
          transmission.progress = 0
          transmission.latency = linkLatencyMs.value
          activeTransmissions.value.push(transmission)
        }
      }
    }
  }, 100)
}

function stopScenario() {
  if (scenarioInterval.value) {
    clearInterval(scenarioInterval.value)
    scenarioInterval.value = null
  }
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  background: #0a0a0a;
  color: #ffffff;
  overflow: hidden;
}

#app {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.main-container {
  flex: 1;
  position: relative;
  overflow: hidden;
}
</style>
