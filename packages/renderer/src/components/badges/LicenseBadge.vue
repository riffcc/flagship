<template>
  <v-tooltip v-if="license" location="bottom" content-class="license-tooltip-content">
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
      <div class="font-weight-bold mb-2">{{ fullLicenseName }}</div>

      <!-- You can -->
      <div class="license-section">
        <div class="license-section-title text-success">You can:</div>
        <ul class="license-list">
          <li v-for="right in licenseRights.can" :key="right">{{ right }}</li>
        </ul>
      </div>

      <!-- Under the following terms -->
      <div v-if="licenseRights.terms.length > 0" class="license-section mt-2">
        <div class="license-section-title text-warning">Under the following terms:</div>
        <ul class="license-list">
          <li v-for="term in licenseRights.terms" :key="term.name">
            <strong>{{ term.name }}</strong> — {{ term.description }}
          </li>
        </ul>
      </div>

      <!-- You cannot -->
      <div v-if="licenseRights.cannot.length > 0" class="license-section mt-2">
        <div class="license-section-title text-error">You cannot:</div>
        <ul class="license-list">
          <li v-for="restriction in licenseRights.cannot" :key="restriction">{{ restriction }}</li>
        </ul>
      </div>

      <div class="text-caption mt-2 text-grey">Click to view full license</div>
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
 * Get short display text for the license (without CC- prefix since logo shows it)
 * No version shown - just the license type
 */
const displayText = computed(() => {
  if (!props.license) return null;

  // Remove 'cc-' prefix since the logo already shows CC
  const type = props.license.type === 'cc0' ? '0' : props.license.type.replace('cc-', '').toUpperCase();
  return type;
});

/**
 * Get full license name for tooltip
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

  // Don't show version if unknown
  if (!props.license.version || props.license.version === 'unknown') {
    return `Creative Commons ${name}`;
  }

  return `Creative Commons ${name} ${props.license.version}`;
});

/**
 * License terms/restrictions for each CC license type
 */
interface LicenseTerm {
  name: string;
  description: string;
}

interface LicenseRightsInfo {
  can: string[];
  terms: LicenseTerm[];
  cannot: string[];
}

const licenseRights = computed((): LicenseRightsInfo => {
  if (!props.license) {
    return { can: [], terms: [], cannot: [] };
  }

  const type = props.license.type;

  // Base rights all CC licenses grant
  const shareRight = 'Share — copy and redistribute the material in any medium or format';
  const adaptRight = 'Adapt — remix, transform, and build upon the material';
  const anyPurposeRight = 'For any purpose, even commercially';

  // Common terms
  const attributionTerm: LicenseTerm = {
    name: 'Attribution',
    description: 'You must give appropriate credit, provide a link to the license, and indicate if changes were made.',
  };
  const shareAlikeTerm: LicenseTerm = {
    name: 'ShareAlike',
    description: 'If you remix, transform, or build upon the material, you must distribute your contributions under the same license.',
  };
  const nonCommercialTerm: LicenseTerm = {
    name: 'NonCommercial',
    description: 'You may not use the material for commercial purposes.',
  };
  const noDerivativesTerm: LicenseTerm = {
    name: 'NoDerivatives',
    description: 'If you remix, transform, or build upon the material, you may not distribute the modified material.',
  };

  switch (type) {
    case 'cc0':
      return {
        can: [
          shareRight,
          adaptRight,
          anyPurposeRight,
        ],
        terms: [],
        cannot: [],
      };

    case 'cc-by':
      return {
        can: [
          shareRight,
          adaptRight,
          anyPurposeRight,
        ],
        terms: [attributionTerm],
        cannot: [],
      };

    case 'cc-by-sa':
      return {
        can: [
          shareRight,
          adaptRight,
          anyPurposeRight,
        ],
        terms: [attributionTerm, shareAlikeTerm],
        cannot: [],
      };

    case 'cc-by-nd':
      return {
        can: [
          shareRight,
          anyPurposeRight,
        ],
        terms: [attributionTerm, noDerivativesTerm],
        cannot: [
          'Distribute modified versions of the material',
        ],
      };

    case 'cc-by-nc':
      return {
        can: [
          shareRight,
          adaptRight,
        ],
        terms: [attributionTerm, nonCommercialTerm],
        cannot: [
          'Use the material for commercial purposes',
        ],
      };

    case 'cc-by-nc-sa':
      return {
        can: [
          shareRight,
          adaptRight,
        ],
        terms: [attributionTerm, nonCommercialTerm, shareAlikeTerm],
        cannot: [
          'Use the material for commercial purposes',
        ],
      };

    case 'cc-by-nc-nd':
      return {
        can: [
          shareRight,
        ],
        terms: [attributionTerm, nonCommercialTerm, noDerivativesTerm],
        cannot: [
          'Use the material for commercial purposes',
          'Distribute modified versions of the material',
        ],
      };

    default:
      return {
        can: [shareRight],
        terms: [],
        cannot: [],
      };
  }
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

  // Generate CC license URL (use 4.0 if version is unknown or not set)
  const version = (!props.license.version || props.license.version === 'unknown') ? '4.0' : props.license.version;
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
  padding: 4px 4px 3px 4px;
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  border-radius: 2px;
  border: 1px solid #22c55e; /* green-500 */
  color: #22c55e;
  background: transparent;
  line-height: 1;
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
  transform: translateY(-1px);
}

.license-tooltip {
  max-width: 400px;
  padding: 4px 0;
}

.license-section {
  margin-bottom: 4px;
}

.license-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.license-list {
  margin: 0;
  padding-left: 16px;
  font-size: 12px;
  line-height: 1.4;
}

.license-list li {
  margin-bottom: 2px;
}

.license-list li:last-child {
  margin-bottom: 0;
}
</style>

<style>
/* Non-scoped styles for tooltip portal content */
.license-tooltip-content {
  background-color: #1a1a1a !important;
  color: #fff !important;
  max-width: 420px !important;
}

.license-tooltip-content .font-weight-bold {
  color: #fff !important;
}

.license-tooltip-content .text-caption {
  color: rgba(255, 255, 255, 0.6) !important;
}

.license-tooltip-content .text-success {
  color: #4ade80 !important;
}

.license-tooltip-content .text-warning {
  color: #fbbf24 !important;
}

.license-tooltip-content .text-error {
  color: #f87171 !important;
}

.license-tooltip-content .text-grey {
  color: rgba(255, 255, 255, 0.5) !important;
}

.license-tooltip-content strong {
  color: #fff !important;
}
</style>
