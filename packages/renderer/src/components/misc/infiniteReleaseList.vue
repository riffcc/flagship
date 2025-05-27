<template>
  <div ref="scrollContainer" class="infinite-release-list">
    <v-sheet
      v-if="isLoading"
      class="d-flex justify-center py-8"
      color="transparent"
    >
      <v-progress-circular
        indeterminate
        color="primary"
        size="64"
      ></v-progress-circular>
    </v-sheet>
    
    <template v-else>
      <div class="releases-wrapper">
        <v-row dense justify="center">
          <v-col
            v-for="item in visibleReleases"
            :key="item.id"
            cols="auto"
          >
            <content-card
              :item="item"
              cursor-pointer
              :source-site="(item.metadata?.['sourceSite'] as string | undefined)"
              @click="$emit('release-click', item)"
            />
          </v-col>
          
          <!-- Lightweight placeholder tiles -->
          <v-col
            v-for="n in placeholderCount"
            :key="`placeholder-${n}`"
            cols="auto"
          >
            <div style="width: 15rem; height: 1px;"></div>
          </v-col>
        </v-row>
      </div>
      
      <v-sheet
        v-if="hasMore"
        v-intersect="onIntersect"
        height="100"
        class="d-flex align-center justify-center"
        color="transparent"
      >
        <!-- Invisible trigger for loading more content -->
      </v-sheet>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import type { ReleaseItem, AnyObject } from '/@/types';
import ContentCard from './contentCard.vue';
import { useGetReleasesQuery } from '/@/plugins/lensService/hooks';
import type { SearchOptions } from '@riffcc/lens-sdk';

const props = defineProps<{
  categoryFilter?: string;
  searchOptions?: SearchOptions;
  pageSize?: number;
}>();

const emit = defineEmits<{
  'release-click': [release: ReleaseItem<AnyObject>];
}>();

// Number of items to show per "page"
const PAGE_SIZE = props.pageSize || 60; // Show many items to fill ultrawide screens
const currentPage = ref(1);
const windowWidth = ref(window.innerWidth);

// Fetch releases with the configured batch size (100)
const { data: releases, isLoading } = useGetReleasesQuery({
  searchOptions: props.searchOptions,
});

// Track window resize
const updateWidth = () => {
  windowWidth.value = window.innerWidth;
};

onMounted(() => {
  window.addEventListener('resize', updateWidth);
});

onUnmounted(() => {
  window.removeEventListener('resize', updateWidth);
});

// Filter releases by category if needed
const filteredReleases = computed(() => {
  if (!releases.value) return [];
  
  if (props.categoryFilter) {
    return releases.value.filter(release => release.categoryId === props.categoryFilter);
  }
  
  return releases.value;
});

// Calculate visible releases based on current page
const visibleReleases = computed(() => {
  const endIndex = currentPage.value * PAGE_SIZE;
  return filteredReleases.value.slice(0, endIndex);
});

const totalCount = computed(() => filteredReleases.value.length);
const visibleCount = computed(() => visibleReleases.value.length);
const hasMore = computed(() => visibleCount.value < totalCount.value);

// Calculate placeholders for even rows
const placeholderCount = computed(() => {
  if (visibleCount.value === 0) return 0;
  
  // Use reactive window width
  const containerPadding = 32; // 1rem * 2
  const containerWidth = windowWidth.value - containerPadding;
  const cardWidth = 240; // 15rem for music cards
  const gap = 12; // Vuetify dense gap
  
  // Calculate how many cards fit in a row
  const itemsPerRow = Math.floor((containerWidth + gap) / (cardWidth + gap));
  
  // Calculate how many items are in the last row
  const itemsInLastRow = visibleCount.value % itemsPerRow;
  
  // If last row is incomplete, add placeholders
  return itemsInLastRow === 0 ? 0 : itemsPerRow - itemsInLastRow;
});


const loadMore = () => {
  // Instantly load more content
  currentPage.value++;
};

// Intersection observer for auto-loading more content
const onIntersect = (isIntersecting: boolean) => {
  if (isIntersecting && hasMore.value) {
    loadMore();
  }
};
</script>

<style scoped>
.infinite-release-list {
  width: 100%;
}

.releases-wrapper {
  margin: 0 auto;
  padding: 0 1rem;
}

/* Disable all transitions for instant appearance */
.infinite-release-list * {
  transition: none !important;
}

/* Ensure v-col doesn't have any fade-in effects */
.infinite-release-list .v-col {
  animation: none !important;
}
</style>