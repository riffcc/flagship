<template>
  <v-menu
    v-if="displayText"
    v-model="menuOpen"
    :disabled="!hasMultipleQualities"
    location="bottom"
    :close-on-content-click="true"
  >
    <template #activator="{ props: menuProps }">
      <div
        v-bind="hasMultipleQualities ? menuProps : {}"
        :class="['quality-badge', { 'quality-badge--clickable': hasMultipleQualities || clickable }]"
        :style="badgeStyle"
        @click="handleClick"
      >
        {{ displayText }}
        <span v-if="hasMultipleQualities" class="quality-badge__chevron">&#9662;</span>
      </div>
    </template>

    <v-list density="compact" class="quality-menu">
      <v-list-subheader>Switch Quality</v-list-subheader>
      <v-list-item
        v-for="tier in sortedTiers"
        :key="tier.name"
        :active="tier.name === currentTierName"
        @click="selectQuality(tier.name, tier.cid)"
      >
        <v-list-item-title>{{ formatTierName(tier.name) }}</v-list-item-title>
      </v-list-item>
    </v-list>
  </v-menu>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import type { AudioQuality, QualityLadder } from '/@/types/badges';

const props = withDefaults(defineProps<{
  quality?: AudioQuality | null;
  qualityLadder?: QualityLadder | null;
  clickable?: boolean;
  playerMode?: boolean; // Desaturated when in player
}>(), {
  clickable: false,
  playerMode: false,
});

const emit = defineEmits<{
  click: [];
  'quality-change': [tierName: string, cid: string];
}>();

const menuOpen = ref(false);

/**
 * Get sorted tiers from the quality ladder
 */
const sortedTiers = computed(() => {
  if (!props.qualityLadder) return [];

  const tierPriority: Record<string, number> = {
    'lossless': 100,
    'opus': 95,
    'mp3_320': 85,
    'mp3_v0': 84,
    'mp3_256': 75,
    'ogg': 73,
    'mp3_vbr': 65,
    'mp3_192': 50,
    'aac': 30,
  };

  return Object.entries(props.qualityLadder)
    .map(([name, cid]) => ({ name, cid }))
    .sort((a, b) => (tierPriority[b.name] || 0) - (tierPriority[a.name] || 0));
});

/**
 * Check if we have multiple quality options
 */
const hasMultipleQualities = computed(() => sortedTiers.value.length > 1);

/**
 * Get the current tier name from the quality format
 */
const currentTierName = computed(() => {
  if (!props.quality?.format) return null;

  const format = props.quality.format;
  const bitrate = props.quality.bitrate;

  if (format === 'flac' || format === 'wav') return 'lossless';
  if (format === 'opus') return 'opus';
  if (format === 'vorbis') return 'ogg';
  if (format === 'aac') return 'aac';

  if (format === 'mp3') {
    if (bitrate && bitrate >= 320) return 'mp3_320';
    if (bitrate && bitrate >= 256) return 'mp3_256';
    if (bitrate && bitrate >= 192) return 'mp3_192';
    if (props.quality.codec?.includes('VBR')) return 'mp3_v0';
    return 'mp3_vbr';
  }

  return null;
});

/**
 * Format tier name for display
 */
function formatTierName(tierName: string): string {
  const names: Record<string, string> = {
    'lossless': 'FLAC (Lossless)',
    'opus': 'Opus',
    'mp3_320': 'MP3 320',
    'mp3_v0': 'MP3 V0',
    'mp3_256': 'MP3 256',
    'ogg': 'Ogg Vorbis',
    'mp3_vbr': 'MP3 VBR',
    'mp3_192': 'MP3 192',
    'aac': 'AAC',
  };
  return names[tierName] || tierName.toUpperCase();
}

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

  // Vorbis
  if (format === 'vorbis') {
    return 'Vorbis';
  }

  // WAV
  if (format === 'wav') {
    return 'WAV';
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

  // High quality: FLAC/WAV - Golden yellow
  if (format === 'flac' || format === 'wav') {
    return '#eab308'; // yellow-500
  }

  // Opus - Green (modern efficient codec)
  if (format === 'opus') {
    return '#22c55e'; // green-500
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

  // Vorbis - Teal
  if (format === 'vorbis') {
    return '#14b8a6'; // teal-500
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
  if (props.clickable && !hasMultipleQualities.value) {
    emit('click');
  }
}

function selectQuality(tierName: string, cid: string) {
  emit('quality-change', tierName, cid);
}
</script>

<style scoped>
.quality-badge {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 4px 4px 3px 4px;
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

.quality-badge__chevron {
  font-size: 7px;
  opacity: 0.7;
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

.quality-menu {
  min-width: 140px;
}
</style>
