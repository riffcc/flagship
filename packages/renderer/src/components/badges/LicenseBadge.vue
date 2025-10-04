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
          :src="ccLogoSvg"
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
          :src="ccLogoSvg"
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
 * Creative Commons logo as data URI (original white)
 */
const ccLogoSvg = "data:image/svg+xml,%3c?xml%20version=%271.0%27%20encoding=%27UTF-8%27?%3e%3c!--%20Generator:%20Adobe%20Illustrator%2013.0.2,%20SVG%20Export%20Plug-In%20.%20SVG%20Version:%206.00%20Build%2014948)%20--%3e%3c!DOCTYPE%20svg%20PUBLIC%20%27-//W3C//DTD%20SVG%201.0//EN%27%20%27http://www.w3.org/TR/2001/REC-SVG-20010904/DTD/svg10.dtd%27%3e%3csvg%20version=%271.0%27%20id=%27Layer_1%27%20xmlns=%27http://www.w3.org/2000/svg%27%20xmlns:xlink=%27http://www.w3.org/1999/xlink%27%20x=%270px%27%20y=%270px%27%20width=%2764px%27%20height=%2764px%27%20viewBox=%275.5%20-3.5%2064%2064%27%20enable-background=%27new%205.5%20-3.5%2064%2064%27%20xml:space=%27preserve%27%3e%3cg%3e%3ccircle%20fill=%27%23FFFFFF%27%20cx=%2737.785%27%20cy=%2728.501%27%20r=%2728.836%27/%3e%3cpath%20d=%27M37.441-3.5c8.951,0,16.572,3.125,22.857,9.372c3.008,3.009,5.295,6.448,6.857,10.314%20c1.561,3.867,2.344,7.971,2.344,12.314c0,4.381-0.773,8.486-2.314,12.313c-1.543,3.828-3.82,7.21-6.828,10.143%20c-3.123,3.085-6.666,5.448-10.629,7.086c-3.961,1.638-8.057,2.457-12.285,2.457s-8.276-0.808-12.143-2.429%20c-3.866-1.618-7.333-3.961-10.4-7.027c-3.067-3.066-5.4-6.524-7-10.372S5.5,32.767,5.5,28.5c0-4.229,0.809-8.295,2.428-12.2%20c1.619-3.905,3.972-7.4,7.057-10.486C21.08-0.394,28.565-3.5,37.441-3.5z%20M37.557,2.272c-7.314,0-13.467,2.553-18.458,7.657%20c-2.515,2.553-4.448,5.419-5.8,8.6c-1.354,3.181-2.029,6.505-2.029,9.972c0,3.429,0.675,6.734,2.029,9.913%20c1.353,3.183,3.285,6.021,5.8,8.516c2.514,2.496,5.351,4.399,8.515,5.715c3.161,1.314,6.476,1.971,9.943,1.971%20c3.428,0,6.75-0.665,9.973-1.999c3.219-1.335,6.121-3.257,8.713-5.771c4.99-4.876,7.484-10.99,7.484-18.344%20c0-3.543-0.648-6.895-1.943-10.057c-1.293-3.162-3.18-5.98-5.654-8.458C50.984,4.844,44.795,2.272,37.557,2.272z%20M37.156,23.187%20l-4.287,2.229c-0.458-0.951-1.019-1.619-1.685-2c-0.667-0.38-1.286-0.571-1.858-0.571c-2.856,0-4.286,1.885-4.286,5.657%20c0,1.714,0.362,3.084,1.085,4.113c0.724,1.029,1.791,1.544,3.201,1.544c1.867,0,3.181-0.915,3.944-2.743l3.942,2%20c-0.838,1.563-2,2.791-3.486,3.686c-1.484,0.896-3.123,1.343-4.914,1.343c-2.857,0-5.163-0.875-6.915-2.629%20c-1.752-1.752-2.628-4.19-2.628-7.313c0-3.048,0.886-5.466,2.657-7.257c1.771-1.79,4.009-2.686,6.715-2.686%20C32.604,18.558,35.441,20.101,37.156,23.187z%20M55.613,23.187l-4.229,2.229c-0.457-0.951-1.02-1.619-1.686-2%20c-0.668-0.38-1.307-0.571-1.914-0.571c-2.857,0-4.287,1.885-4.287,5.657c0,1.714,0.363,3.084,1.086,4.113%20c0.723,1.029,1.789,1.544,3.201,1.544c1.865,0,3.18-0.915,3.941-2.743l4,2c-0.875,1.563-2.057,2.791-3.541,3.686%20c-1.486,0.896-3.105,1.343-4.857,1.343c-2.896,0-5.209-0.875-6.941-2.629c-1.736-1.752-2.602-4.19-2.602-7.313%20c0-3.048,0.885-5.466,2.658-7.257c1.77-1.79,4.008-2.686,6.713-2.686C51.117,18.558,53.938,20.101,55.613,23.187z%27/%3e%3c/g%3e%3c/svg%3e";

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
  opacity: 0.9;
}

.license-tooltip {
  max-width: 300px;
}
</style>
