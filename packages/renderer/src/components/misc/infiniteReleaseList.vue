<template>
  <div
    ref="scrollContainer"
    class="infinite-release-list"
  >
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
      <div class="grid-container">
        <div class="releases-grid">
          <content-card
            v-for="item in visibleReleases"
            :key="item.id"
            :item="item"
            cursor-pointer
            @click="$emit('release-click', item)"
          />
        </div>
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
import { computed, ref } from 'vue';
import type { ReleaseItem } from '/@/types';
import ContentCard from './contentCard.vue';
import { useGetReleasesQuery } from '/@/plugins/lensService/hooks';
import type { SearchOptions } from '@riffcc/lens-sdk';

const props = defineProps<{
  categoryFilter?: string;
  searchOptions?: SearchOptions;
  pageSize?: number;
}>();

defineEmits<{
  'release-click': [release: ReleaseItem];
}>();

// Number of items to show per "page"
const PAGE_SIZE = props.pageSize || 60; // Show many items to fill ultrawide screens
const currentPage = ref(1);

// Fetch releases with the configured batch size (100)
const { data: releases, isLoading } = useGetReleasesQuery({
  searchOptions: props.searchOptions,
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

.grid-container {
  display: flex;
  justify-content: center;
  width: 100%;
}

.releases-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(12.5rem, 1fr));
  gap: 0.5rem;
  justify-content: start;
  max-width: 100%;
}

/* Firefox fallback using feature detection */
@supports (-moz-appearance: none) {
  .grid-container {
    /* Simplify container for Firefox */
    display: block;
    width: 100%;
  }
  
  .releases-grid {
    /* Keep same sizing but fix Firefox grid issues */
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(12.5rem, 1fr));
    gap: 0.5rem;
    width: 100%;
    margin: 0 auto;
    justify-content: center;
    /* Firefox-specific grid fixes */
    grid-auto-flow: row;
    align-items: start;
  }
}

/* Disable all transitions for instant appearance */
.infinite-release-list * {
  transition: none !important;
}
</style>
