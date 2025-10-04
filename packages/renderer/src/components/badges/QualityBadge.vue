<template>
  <v-chip
    v-if="displayText"
    :size="size"
    variant="tonal"
    :color="color"
    :class="['quality-badge', { 'quality-badge--clickable': clickable }]"
    @click="handleClick"
  >
    <v-icon v-if="icon" :icon="icon" size="12" class="mr-1" />
    {{ displayText }}
  </v-chip>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { AudioQuality } from '/@/types/badges';

const props = withDefaults(defineProps<{
  quality?: AudioQuality | null;
  size?: 'x-small' | 'small' | 'default' | 'large';
  clickable?: boolean;
  showIcon?: boolean;
}>(), {
  size: 'x-small',
  clickable: false,
  showIcon: true,
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
 * Get color based on quality tier
 */
const color = computed(() => {
  if (!props.quality) return 'default';

  const { format, bitDepth, bitrate } = props.quality;

  // Highest quality: 24-bit FLAC
  if (format === 'flac' && bitDepth === 24) {
    return 'purple';
  }

  // High quality: FLAC or high bitrate
  if (format === 'flac' || (bitrate && bitrate >= 320)) {
    return 'indigo';
  }

  // Good quality: 256kbps+
  if (bitrate && bitrate >= 256) {
    return 'blue';
  }

  // Medium quality: 192-255kbps
  if (bitrate && bitrate >= 192) {
    return 'cyan';
  }

  // Lower quality
  return 'grey';
});

/**
 * Get icon based on format
 */
const icon = computed(() => {
  if (!props.showIcon || !props.quality) return null;

  const { format, bitDepth } = props.quality;

  if (format === 'flac') {
    return bitDepth === 24 ? 'mdi-music-note-plus' : 'mdi-music-note';
  }

  if (format === 'mp3' || format === 'aac') {
    return 'mdi-file-music-outline';
  }

  if (format === 'opus') {
    return 'mdi-audio-input-stereo-minijack';
  }

  return 'mdi-music';
});

function handleClick() {
  if (props.clickable) {
    emit('click');
  }
}
</script>

<style scoped>
.quality-badge {
  font-size: 10px;
  font-weight: 500;
  opacity: 0.85;
  transition: opacity 0.2s ease;
}

.quality-badge--clickable {
  cursor: pointer;
}

.quality-badge--clickable:hover {
  opacity: 1;
}

.quality-badge :deep(.v-chip__content) {
  padding-inline: 6px;
}
</style>
