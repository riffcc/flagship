<template>
  <v-hover
    v-slot="{props: hoveringProps, isHovering}"
    open-delay="150"
    close-delay="150"
  >
    <v-sheet
      v-bind="hoveringProps"
      :class="cursorPointer ? 'cursor-pointer mx-auto' : 'mx-auto'"
      color="transparent"
      :height="cardHeight"
      :width="cardWidth"
      :style="showDefederation && sourceSite ? `border: 1px solid ${getSiteColor(sourceSite)};` : ''"
      @click="onClick"
    >
      <template v-if="isOverlapping">
        <v-img
          :src="parseUrlOrCid(props.item.thumbnailCID) ?? '/no-image-icon.png'"
          width="100%"
          cover
          aspect-ratio="1"
          :gradient="cardBackgroundGradient"
        >
          <p class="ml-4 mt-2 text-subtitle-1">
            {{ cardTitle }}
          </p>
          <p
            v-if="cardSubtitle"
            class="ml-4 text-subtitle-2"
          >
            {{ cardSubtitle }}
          </p>
          <template v-if="isHovering">
            <v-icon
              v-if="item.categoryId === 'music'"
              size="4.5rem"
              icon="mdi-play"
              color="primary"
              class="position-absolute top-0 left-0 right-0 bottom-0 ma-auto"
            ></v-icon>
            <div
              v-else-if="item.categoryId === 'tvShow'"
              class="position-absolute top-0 bottom-0 right-0 d-flex flex-column justify-center mr-2 ga-1"
            >
              <v-btn
                size="small"
                color="grey-lighten-3"
                density="comfortable"
                icon="mdi-share-variant"
              ></v-btn>
              <v-btn
                size="small"
                color="grey-lighten-3"
                density="comfortable"
                icon="mdi-heart"
              ></v-btn>
              <v-btn
                size="small"
                color="grey-lighten-3"
                density="comfortable"
                icon="mdi-plus"
              ></v-btn>
            </div>
          </template>
          <!-- Actions slot content (e.g., TV show buttons, play button) -->
          <template
            v-if="item.categoryId === 'tvShow'"
          >
            <v-btn
              color="primary"
              rounded="0"
              prepend-icon="mdi-play"
              size="small"
              class="position-absolute bottom-0 rigth-0 text-none ml-4 mb-10"
              text="Play now"
              @click="router.push(`/release/${item.id}`)"
            ></v-btn>
          </template>
        </v-img>
      </template>
      <template v-else>
        <v-img
          :src="parseUrlOrCid(props.item.thumbnailCID) ?? '/no-image-icon.png'"
          width="100%"
          cover
          aspect-ratio="1"
        >
          <slot
            v-if="isHovering"
            name="hovering"
          ></slot>
        </v-img>
        <p class="text-caption text-sm-subtitle-1 text-center mt-1">
          {{ cardTitle }}
        </p>
        <p
          v-if="cardSubtitle"
          class="text-caption text-sm-subtitle-1 text-center text-medium-emphasis"
        >
          {{ cardSubtitle }}
        </p>
      </template>
    </v-sheet>
  </v-hover>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useDisplay } from 'vuetify';
import { useShowDefederation } from '/@/composables/showDefed';
import { useSiteColors } from '/@/composables/siteColors';
import { type ReleaseItem } from '/@/types';
import { parseUrlOrCid } from '/@/utils';
import { useRouter } from 'vue-router';
import type { AnyObject } from '@riffcc/lens-sdk';


const { showDefederation } = useShowDefederation();
const { getSiteColor } = useSiteColors();
const { xs } = useDisplay();
const router = useRouter();

const props = defineProps<{
  item: ReleaseItem<AnyObject>;
  cursorPointer?: boolean;
  sourceSite?: string;
  onClick?: () => void;
}>();

const cardWidth = computed(() => {
  const categoryId = props.item.categoryId;
  if (categoryId === 'music') {
    return xs.value ? '10.5rem' : '15rem';
  }
  if (categoryId === 'tvShow') {
    return '17rem';
  }
  return xs.value ? '10.5rem' : '12rem';
});

const cardHeight = computed(() => {
  if (props.item.categoryId === 'tvShow') {
    return '10rem';
  }
  return undefined;
});

const cardTitle = computed(() => {
  const categoryId = props.item.categoryId;
  if (categoryId === 'music') {
    return props.item.name;
  }
  if (categoryId === 'tvShow') {
    return props.item.name;
  }
  if (categoryId === 'movie') {
    return props.item.name;
  }
  return props.item.metadata?.['author'] ?? '';
});

const cardSubtitle = computed(() => {
  const categoryId = props.item.categoryId;
  if (categoryId === 'music') {
    return props.item.metadata?.['author'] ?? '';
  }
  if (categoryId === 'tvShow') {
    return props.item.metadata?.['seasons'] ? `${props.item.metadata['seasons']} Seasons` : undefined;
  }
  // Default
  if (categoryId === 'movie') {
    return props.item.metadata?.['releaseYear'] ? `(${props.item.metadata['releaseYear']})` : undefined;
  }
  return props.item.name;
});

const isOverlapping = computed(() => {
  return props.item.categoryId === 'tvShow';
});

const cardBackgroundGradient = computed(() => {
  return props.item.categoryId === 'tvShow' ? 'to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)' : undefined;
});


</script>
