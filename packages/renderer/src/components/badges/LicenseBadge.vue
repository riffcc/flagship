<template>
  <v-tooltip v-if="license" location="bottom">
    <template #activator="{ props: tooltipProps }">
      <a
        v-if="linkable"
        :href="licenseUrl"
        target="_blank"
        rel="noopener noreferrer"
        class="license-badge license-badge--linkable"
        v-bind="tooltipProps"
      >
        <img
          src="/cc.svg"
          alt="CC"
          class="cc-icon"
        />
        {{ displayText }}
      </a>
      <div
        v-else
        class="license-badge"
        v-bind="tooltipProps"
      >
        <img
          src="/cc.svg"
          alt="CC"
          class="cc-icon"
        />
        {{ displayText }}
      </div>
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
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 1px 4px;
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  border-radius: 2px;
  border: 1px solid #22c55e; /* green-500 */
  color: #22c55e;
  background: transparent;
  line-height: 1.4;
  opacity: 0.8;
  transition: opacity 0.2s ease;
  text-decoration: none;
}

.license-badge--linkable {
  cursor: pointer;
}

.license-badge--linkable:hover {
  opacity: 1;
}

.cc-icon {
  width: 10px;
  height: 10px;
  filter: invert(100%) sepia(0%) saturate(7438%) hue-rotate(78deg) brightness(109%) contrast(95%);
  opacity: 0.9;
}

.license-tooltip {
  max-width: 300px;
}
</style>
