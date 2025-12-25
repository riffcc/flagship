<template>
  <div
    class="archivist-badge"
    @mouseenter="handleMouseEnter"
    @mouseleave="showTooltip = false"
  >
    <img
      :src="iconSrc"
      alt="Stored on Archivist"
      class="archivist-icon"
      :class="{ 'archivist-icon--loading': loading }"
    />

    <Teleport to="body">
      <Transition name="tooltip">
        <div
          v-if="showTooltip"
          class="archivist-tooltip"
          :style="tooltipPosition"
        >
          <div class="tooltip-header">
            <img :src="iconSrc" class="tooltip-header-icon" />
            <span>Archivist Storage</span>
          </div>

          <div v-if="loading" class="tooltip-loading">
            <v-progress-circular size="16" width="2" indeterminate />
            <span>Checking availability...</span>
          </div>

          <div v-else-if="nodeInfo.length > 0" class="node-list">
            <div
              v-for="node in nodeInfo"
              :key="node.url"
              :class="['node-item', { 'node-item--alive': node.alive }]"
            >
              <span class="node-status">{{ node.alive ? '●' : '○' }}</span>
              <span class="node-url">{{ formatNodeUrl(node.url) }}</span>
              <span class="node-status-text">
                {{ node.alive ? 'Available' : 'Unavailable' }}
              </span>
            </div>
          </div>

          <div v-else class="tooltip-empty">
            No node information available
          </div>

          <div v-if="lastCheck" class="tooltip-footer">
            Last verified: {{ formatTime(lastCheck) }}
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { getNodesForCid, type ArchivistNodeInfo } from '/@/composables/useArchivist';

const ARCHIVIST_ICON = 'https://docs.archivist.storage/logos/archivist-terminal.svg';

const props = defineProps<{
  cid: string;
}>();

const showTooltip = ref(false);
const loading = ref(false);
const nodeInfo = ref<ArchivistNodeInfo[]>([]);
const lastCheck = ref<Date | null>(null);
const tooltipX = ref(0);
const tooltipY = ref(0);

const iconSrc = computed(() => ARCHIVIST_ICON);

const tooltipPosition = computed(() => ({
  left: `${tooltipX.value}px`,
  top: `${tooltipY.value}px`,
}));

async function handleMouseEnter(event: MouseEvent) {
  // Position tooltip near cursor
  tooltipX.value = event.clientX + 12;
  tooltipY.value = event.clientY + 12;

  showTooltip.value = true;

  // Fetch node info if we don't have recent data
  if (!lastCheck.value || Date.now() - lastCheck.value.getTime() > 30000) {
    loading.value = true;
    try {
      nodeInfo.value = await getNodesForCid(props.cid);
      lastCheck.value = new Date();
    } finally {
      loading.value = false;
    }
  }
}

function formatNodeUrl(url: string): string {
  try {
    const parsed = new URL(url);
    return parsed.hostname + (parsed.port ? `:${parsed.port}` : '');
  } catch {
    return url;
  }
}

function formatTime(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  if (diff < 60000) {
    return 'Just now';
  } else if (diff < 3600000) {
    const mins = Math.floor(diff / 60000);
    return `${mins}m ago`;
  } else {
    return date.toLocaleTimeString();
  }
}
</script>

<style scoped>
.archivist-badge {
  display: inline-flex;
  align-items: center;
  cursor: pointer;
  position: relative;
}

.archivist-icon {
  width: 16px;
  height: 16px;
  opacity: 0.7;
  transition: opacity 0.2s ease, filter 0.2s ease;
  filter: brightness(0.9);
}

.archivist-icon:hover {
  opacity: 1;
  filter: brightness(1.1) drop-shadow(0 0 4px rgba(138, 43, 226, 0.5));
}

.archivist-icon--loading {
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.7; }
  50% { opacity: 1; }
}

.archivist-tooltip {
  position: fixed;
  background: rgba(10, 10, 10, 0.95);
  border: 1px solid rgba(138, 43, 226, 0.4);
  border-radius: 8px;
  padding: 12px;
  min-width: 220px;
  max-width: 300px;
  z-index: 10000;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5),
              0 0 20px rgba(138, 43, 226, 0.15);
  backdrop-filter: blur(8px);
}

.tooltip-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  color: #fff;
  font-weight: 600;
  font-size: 13px;
}

.tooltip-header-icon {
  width: 18px;
  height: 18px;
}

.tooltip-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
}

.node-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.node-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
}

.node-item--alive {
  color: rgba(255, 255, 255, 0.9);
}

.node-status {
  font-size: 10px;
}

.node-item--alive .node-status {
  color: #4ade80; /* green-400 */
}

.node-item:not(.node-item--alive) .node-status {
  color: #f87171; /* red-400 */
}

.node-url {
  flex: 1;
  font-family: monospace;
  font-size: 11px;
}

.node-status-text {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.node-item--alive .node-status-text {
  color: #4ade80;
}

.node-item:not(.node-item--alive) .node-status-text {
  color: #f87171;
}

.tooltip-empty {
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  font-style: italic;
}

.tooltip-footer {
  margin-top: 10px;
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.4);
  font-size: 10px;
}

/* Transition */
.tooltip-enter-active,
.tooltip-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
