<template>
  <canvas ref="canvas" class="renderer-container"></canvas>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'

const props = defineProps({
  meshWidth: Number,
  meshHeight: Number,
  meshDepth: Number,
  hexSize: Number,
  layerSpacing: Number,
  activeTransmissions: Array,
})

const emit = defineEmits(['fps-update'])

const container = ref(null)
let scene, camera, renderer, controls
let hexGroup, particleSystem
let slots = []
let animationId = null
let lastFrame = 0

onMounted(() => {
  initThreeJS()
  generateMesh()
  animate()
})

onBeforeUnmount(() => {
  if (animationId) cancelAnimationFrame(animationId)
  if (renderer) renderer.dispose()
  if (controls) controls.dispose()
})

watch(() => [props.hexSize, props.layerSpacing], () => {
  regenerateMesh()
})

function initThreeJS() {
  // Scene
  scene = new THREE.Scene()
  scene.background = new THREE.Color(0x0a0a0a)

  // Camera
  camera = new THREE.PerspectiveCamera(
    75,
    container.value.clientWidth / container.value.clientHeight,
    0.1,
    10000
  )
  camera.position.set(100, 100, 100)

  // Renderer with WebGL2
  renderer = new THREE.WebGLRenderer({
    antialias: true,
    powerPreference: 'high-performance',
  })
  renderer.setSize(container.value.clientWidth, container.value.clientHeight)
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
  container.value.appendChild(renderer.domElement)

  // Controls
  controls = new OrbitControls(camera, renderer.domElement)
  controls.enableDamping = true
  controls.dampingFactor = 0.05
  controls.maxDistance = 1000
  controls.minDistance = 10

  // Lights
  const ambientLight = new THREE.AmbientLight(0x404040, 1)
  scene.add(ambientLight)

  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.5)
  directionalLight.position.set(50, 100, 50)
  scene.add(directionalLight)

  // Resize handler
  window.addEventListener('resize', onWindowResize)
}

function onWindowResize() {
  camera.aspect = container.value.clientWidth / container.value.clientHeight
  camera.updateProjectionMatrix()
  renderer.setSize(container.value.clientWidth, container.value.clientHeight)
}

function hexToWorld(x, y, z) {
  const hexWidth = props.hexSize * Math.sqrt(3)
  const hexHeight = props.hexSize * 2

  const worldX = hexWidth * (x + 0.5 * (y % 2))
  const worldY = z * props.layerSpacing
  const worldZ = hexHeight * 0.75 * y

  return new THREE.Vector3(worldX, worldY, worldZ)
}

function createHexagonGeometry(size) {
  const shape = new THREE.Shape()
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 3) * i - Math.PI / 6
    const x = size * Math.cos(angle)
    const y = size * Math.sin(angle)
    if (i === 0) {
      shape.moveTo(x, y)
    } else {
      shape.lineTo(x, y)
    }
  }
  shape.closePath()

  const geometry = new THREE.ShapeGeometry(shape)
  geometry.rotateX(-Math.PI / 2)
  return geometry
}

function generateMesh() {
  // Clear existing
  if (hexGroup) scene.remove(hexGroup)
  hexGroup = new THREE.Group()
  slots = []

  // Use simple circular sprites (billboards) - always face camera, super cheap to render
  const canvas = document.createElement('canvas')
  canvas.width = 64
  canvas.height = 64
  const ctx = canvas.getContext('2d')

  // Draw filled circle
  ctx.fillStyle = '#8a2be2'
  ctx.globalAlpha = 0.6
  ctx.beginPath()
  ctx.arc(32, 32, 28, 0, Math.PI * 2)
  ctx.fill()

  // Draw edge ring
  ctx.strokeStyle = '#00ff88'
  ctx.globalAlpha = 0.8
  ctx.lineWidth = 3
  ctx.beginPath()
  ctx.arc(32, 32, 28, 0, Math.PI * 2)
  ctx.stroke()

  const texture = new THREE.CanvasTexture(canvas)
  const spriteMaterial = new THREE.SpriteMaterial({
    map: texture,
    transparent: true,
    sizeAttenuation: true
  })

  for (let z = 0; z < props.meshDepth; z++) {
    for (let y = 0; y < props.meshHeight; y++) {
      for (let x = 0; x < props.meshWidth; x++) {
        const pos = hexToWorld(x, y, z)

        // Create sprite (always faces camera, cheap to render)
        const sprite = new THREE.Sprite(spriteMaterial.clone())
        sprite.position.copy(pos)
        sprite.scale.set(props.hexSize * 2, props.hexSize * 2, 1)
        hexGroup.add(sprite)

        slots.push({ x, y, z, position: pos })
      }
    }
  }

  scene.add(hexGroup)

  // Center camera on mesh
  const gridWidth = props.hexSize * Math.sqrt(3) * props.meshWidth
  const gridDepth = props.hexSize * 2 * 0.75 * props.meshHeight
  const centerX = gridWidth / 2
  const centerZ = gridDepth / 2
  const centerY = (props.meshDepth * props.layerSpacing) / 2

  hexGroup.position.set(-centerX, -centerY, -centerZ)
}

function regenerateMesh() {
  generateMesh()
}

function createRandomTransmission() {
  if (slots.length === 0) return null

  const fromSlot = slots[Math.floor(Math.random() * slots.length)]
  const neighbors = getNeighbors(fromSlot.x, fromSlot.y, fromSlot.z)
  if (neighbors.length === 0) return null

  const toSlot = neighbors[Math.floor(Math.random() * neighbors.length)]

  const colors = ['#00ff00', '#00ffff', '#ffff00', '#ff00ff']
  return {
    from: fromSlot,
    to: toSlot,
    color: colors[Math.floor(Math.random() * colors.length)],
  }
}

function getNeighbors(x, y, z) {
  const neighbors = []
  const hexOffsets = [
    [1, 0], [-1, 0],
    [0, -1], [0, 1],
    [(y % 2 === 0) ? -1 : 1, -1],
    [(y % 2 === 0) ? -1 : 1, 1],
  ]

  for (const [dx, dy] of hexOffsets) {
    const nx = (x + dx + props.meshWidth) % props.meshWidth
    const ny = (y + dy + props.meshHeight) % props.meshHeight
    const slot = slots.find(s => s.x === nx && s.y === ny && s.z === z)
    if (slot) neighbors.push(slot)
  }

  // Vertical neighbors
  const upSlot = slots.find(s => s.x === x && s.y === y && s.z === (z + 1) % props.meshDepth)
  const downSlot = slots.find(s => s.x === x && s.y === y && s.z === (z - 1 + props.meshDepth) % props.meshDepth)
  if (upSlot) neighbors.push(upSlot)
  if (downSlot) neighbors.push(downSlot)

  return neighbors
}

function animate() {
  animationId = requestAnimationFrame(animate)

  const now = performance.now()
  const deltaTime = now - lastFrame
  lastFrame = now
  const fps = Math.round(1000 / deltaTime)
  emit('fps-update', fps)

  // Update controls
  if (controls) controls.update()

  // Render
  if (renderer && scene && camera) {
    renderer.render(scene, camera)
  }
}

defineExpose({
  regenerateMesh,
  createRandomTransmission,
})
</script>

<style scoped>
.renderer-container {
  width: 100%;
  height: 100%;
}
</style>
