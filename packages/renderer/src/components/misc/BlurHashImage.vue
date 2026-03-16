<template>
  <div
    class="blurhash-image"
    :style="containerStyle"
  >
    <img
      v-if="placeholderSrc"
      :src="placeholderSrc"
      alt=""
      aria-hidden="true"
      class="blurhash-image__placeholder"
      :class="{ 'blurhash-image__placeholder--hidden': imageLoaded }"
    >
    <img
      v-if="src"
      :src="src"
      :alt="alt"
      class="blurhash-image__image"
      :class="{ 'blurhash-image__image--loaded': imageLoaded }"
      :style="imageStyle"
      @load="imageLoaded = true"
      @error="imageLoaded = true"
    >
    <div
      v-if="gradient"
      class="blurhash-image__gradient"
      :style="{ background: gradient }"
    />
    <div class="blurhash-image__content">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { decodeBlurHashToDataUrl } from '/@/utils/blurhash';

const props = withDefaults(defineProps<{
  src?: string | null;
  blurhash?: string | null;
  alt?: string;
  aspectRatio?: string | number;
  cover?: boolean;
  gradient?: string;
  position?: string;
  width?: string | number;
  height?: string | number;
}>(), {
  src: null,
  blurhash: null,
  alt: '',
  aspectRatio: undefined,
  cover: false,
  gradient: undefined,
  position: 'center center',
  width: '100%',
  height: undefined,
});

const imageLoaded = ref(false);

watch(() => props.src, () => {
  imageLoaded.value = false;
});

const placeholderSrc = computed(() => {
  if (!props.blurhash) {
    return null;
  }

  return decodeBlurHashToDataUrl(props.blurhash, 32, 32);
});

const containerStyle = computed(() => ({
  width: typeof props.width === 'number' ? `${props.width}px` : props.width,
  height: typeof props.height === 'number' ? `${props.height}px` : props.height,
  aspectRatio: props.aspectRatio ? String(props.aspectRatio) : undefined,
}));

const imageStyle = computed(() => ({
  objectFit: props.cover ? 'cover' as const : 'contain' as const,
  objectPosition: props.position,
}));
</script>

<style scoped>
.blurhash-image {
  overflow: hidden;
  position: relative;
}

.blurhash-image__placeholder,
.blurhash-image__image,
.blurhash-image__gradient,
.blurhash-image__content {
  inset: 0;
  position: absolute;
}

.blurhash-image__placeholder,
.blurhash-image__image {
  height: 100%;
  width: 100%;
}

.blurhash-image__placeholder {
  filter: blur(18px);
  transform: scale(1.08);
  transition: opacity 220ms ease;
}

.blurhash-image__placeholder--hidden {
  opacity: 0;
}

.blurhash-image__image {
  opacity: 0;
  transition: opacity 220ms ease;
}

.blurhash-image__image--loaded {
  opacity: 1;
}

.blurhash-image__gradient {
  z-index: 1;
  pointer-events: none;
}

.blurhash-image__content {
  z-index: 2;
}
</style>
