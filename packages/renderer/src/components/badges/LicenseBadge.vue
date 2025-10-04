<template>
  <v-tooltip v-if="license" location="bottom">
    <template #activator="{ props: tooltipProps }">
      <v-chip
        v-bind="tooltipProps"
        :size="size"
        variant="tonal"
        color="green"
        :class="['license-badge', { 'license-badge--linkable': linkable }]"
        :href="linkable ? licenseUrl : undefined"
        :target="linkable ? '_blank' : undefined"
        :rel="linkable ? 'noopener noreferrer' : undefined"
        @click="handleClick"
      >
        <v-icon icon="mdi-creative-commons" size="12" class="mr-1" />
        {{ displayText }}
      </v-chip>
    </template>
    <div class="license-tooltip">
      <div class="font-weight-bold mb-1">{{ fullLicenseName }}</div>
      <div class="text-caption">{{ license.attribution || 'Click to view license' }}</div>
    </div>
  </v-tooltip>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { LicenseInfo } from '/@/types/badges';

const props = withDefaults(defineProps<{
  license?: LicenseInfo | null;
  size?: 'x-small' | 'small' | 'default' | 'large';
  linkable?: boolean;
}>(), {
  size: 'x-small',
  linkable: true,
});

const emit = defineEmits<{
  click: [];
}>();

/**
 * Get short display text for the license
 */
const displayText = computed(() => {
  if (!props.license) return null;

  const version = props.license.version || '4.0';
  return `${props.license.type.toUpperCase()} ${version}`;
});

/**
 * Get full license name
 */
const fullLicenseName = computed(() => {
  if (!props.license) return '';

  const names: Record<string, string> = {
    'cc0': 'Public Domain (CC0)',
    'cc-by': 'Attribution',
    'cc-by-sa': 'Attribution-ShareAlike',
    'cc-by-nd': 'Attribution-NoDerivatives',
    'cc-by-nc': 'Attribution-NonCommercial',
    'cc-by-nc-sa': 'Attribution-NonCommercial-ShareAlike',
    'cc-by-nc-nd': 'Attribution-NonCommercial-NoDerivatives',
  };

  const name = names[props.license.type] || props.license.type;
  const version = props.license.version || '4.0';

  return `Creative Commons ${name} ${version}`;
});

/**
 * Get license URL
 */
const licenseUrl = computed(() => {
  if (!props.license) return null;

  // Use custom URL if provided
  if (props.license.url) {
    return props.license.url;
  }

  // Generate CC license URL
  const version = props.license.version || '4.0';
  const licenseSlug = props.license.type === 'cc0' ? 'zero' : props.license.type.replace('cc-', '');

  return `https://creativecommons.org/licenses/${licenseSlug}/${version}/`;
});

function handleClick() {
  if (!props.linkable) {
    emit('click');
  }
}
</script>

<style scoped>
.license-badge {
  font-size: 10px;
  font-weight: 500;
  opacity: 0.85;
  transition: opacity 0.2s ease;
}

.license-badge--linkable {
  cursor: pointer;
  text-decoration: none;
}

.license-badge--linkable:hover {
  opacity: 1;
}

.license-badge :deep(.v-chip__content) {
  padding-inline: 6px;
}

.license-tooltip {
  max-width: 300px;
}
</style>
