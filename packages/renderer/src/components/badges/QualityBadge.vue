<template>
  <div
    v-if="displayText"
    :class="['quality-badge', { 'quality-badge--clickable': clickable }]"
    :style="badgeStyle"
    @click="handleClick"
  >
    {{ displayText }}
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { AudioQuality } from '/@/types/badges';

const props = withDefaults(defineProps<{
  quality?: AudioQuality | null;
  clickable?: boolean;
  playerMode?: boolean; // Desaturated when in player
}>(), {
  clickable: false,
  playerMode: false,
});

const emit = defineEmits<{
  click: [];
}>();

/**
 * Get display text based on quality metadata
 * Only returns text if quality is known
 */
const displayText = computed(() => {
  if (!props.quality) return null;

  const { format, bitrate, bitDepth, sampleRate } = props.quality;

  // 24-bit FLAC
  if (format === 'flac' && bitDepth === 24) {
    if (sampleRate && sampleRate > 48000) {
      return `24-bit/${Math.round(sampleRate / 1000)}kHz FLAC`;
    }
    return '24-bit FLAC';
  }

  // Regular FLAC
  if (format === 'flac') {
    return 'FLAC';
  }

  // MP3 with bitrate
  if (format === 'mp3' && bitrate) {
    return `MP3 ${bitrate}`;
  }

  // AAC with bitrate
  if (format === 'aac' && bitrate) {
    return `AAC ${bitrate}`;
  }

  // Opus
  if (format === 'opus') {
    return bitrate ? `Opus ${bitrate}` : 'Opus';
  }

  // Generic format
  if (format && format !== 'other') {
    return format.toUpperCase();
  }

  // Fallback: if we have any quality info but can't determine format
  if (bitrate) {
    return `${bitrate}kbps`;
  }

  // No meaningful quality info
  return null;
});

/**
 * Get border color based on quality tier
 */
const borderColor = computed(() => {
  if (!props.quality) return '#666';

  const { format, bitDepth, bitrate } = props.quality;

  // Highest quality: 24-bit FLAC - Purple/Violet
  if (format === 'flac' && bitDepth === 24) {
    return '#9333ea'; // purple-600
  }

  // High quality: FLAC - Golden yellow
  if (format === 'flac') {
    return '#eab308'; // yellow-500
  }

  // High bitrate MP3/AAC: 320kbps - Orange
  if (bitrate && bitrate >= 320) {
    return '#f97316'; // orange-500
  }

  // Good quality: 256kbps+ - Blue
  if (bitrate && bitrate >= 256) {
    return '#3b82f6'; // blue-500
  }

  // Medium quality: 192-255kbps - Cyan
  if (bitrate && bitrate >= 192) {
    return '#06b6d4'; // cyan-500
  }

  // Lower quality - Grey
  return '#6b7280'; // gray-500
});

/**
 * Badge styling
 */
const badgeStyle = computed(() => {
  const color = borderColor.value;

  if (props.playerMode) {
    // Desaturated in player mode
    return {
      border: `1px solid ${color}`,
      color: color,
      filter: 'saturate(0.3)',
    };
  }

  return {
    border: `1px solid ${color}`,
    color: color,
  };
});

function handleClick() {
  if (props.clickable) {
    emit('click');
  }
}
</script>

<style scoped>
.quality-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 4px;
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  border-radius: 2px;
  background: transparent;
  line-height: 1;
  opacity: 0.8;
  transition: opacity 0.2s ease, filter 0.2s ease, brightness 0.15s ease;
}

.quality-badge--clickable {
  cursor: pointer;
}

.quality-badge--clickable:hover {
  opacity: 1;
  filter: saturate(1) !important; /* Full color on hover */
}

.quality-badge--clickable:active {
  filter: saturate(1) brightness(1.3) !important; /* Brighten on click */
}
</style>
