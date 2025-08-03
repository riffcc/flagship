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
import { computed, ref, watch } from 'vue';
import type { ReleaseItem } from '/@/types';
import ContentCard from './contentCard.vue';
import { useGetReleasesQuery, useGetStructuresQuery, useContentCategoriesQuery } from '/@/plugins/lensService/hooks';
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

// Get content categories to check if this is a TV category
const { data: contentCategories } = useContentCategoriesQuery();

// Check if this is the TV shows category
const isTVCategory = computed(() => {
  if (!props.categoryFilter || !contentCategories.value) return false;
  const category = contentCategories.value.find(c => c.id === props.categoryFilter);
  return category?.displayName === 'TV Shows' || category?.categoryId === 'tv-shows';
});

// Fetch structures for TV series grouping
const { data: structures, isLoading: isStructuresLoading, error: structuresError } = useGetStructuresQuery({
  enabled: isTVCategory,
  searchOptions: { fetch: 1000 }
});

// Debug structures
watch(structures, (newStructures) => {
  if (isTVCategory.value) {
    console.log('Structures updated:', newStructures?.length || 0, 'structures');
    if (newStructures && newStructures.length > 0) {
      console.log('First structure:', newStructures[0]);
    }
  }
}, { immediate: true });

// Filter releases by category if needed
const filteredReleases = computed(() => {
  if (!releases.value) return [];

  let categoryReleases = releases.value;
  if (props.categoryFilter && contentCategories.value) {
    // Find the category to get all its IDs (from different lenses)
    const category = contentCategories.value.find(c => c.id === props.categoryFilter);
    if (category && category.allIds) {
      // Filter by all IDs from federated categories
      categoryReleases = releases.value.filter(release => 
        category.allIds.includes(release.categoryId)
      );
    } else {
      // Fallback to single ID filter
      categoryReleases = releases.value.filter(release => release.categoryId === props.categoryFilter);
    }
  }

  // If this is a TV category, group episodes by series
  if (isTVCategory.value) {
    const seriesMap = new Map<string, any>();
    const seriesStructures = structures.value ? 
      structures.value.filter((s: any) => s.type === 'series') : [];
    
    for (const release of categoryReleases) {
      const seriesId = release.metadata?.seriesId;
      
      if (seriesId) {
        // Episode belongs to a series
        const series = seriesStructures.find((s: any) => s.id === seriesId);
        
        if (!seriesMap.has(seriesId)) {
          if (series) {
            // Create series tile
            seriesMap.set(seriesId, {
              id: series.id,
              name: series.name,
              categoryId: props.categoryFilter,
              thumbnailCID: series.thumbnailCID || release.thumbnailCID,
              contentCID: release.contentCID,
              description: series.description,
              metadata: {
                isSeries: true,
                episodeCount: 0,
                episodes: [],
                ...series.metadata
              }
            });
          } else {
            // Fallback series tile - try to extract series name from episode
            // Episodes might have the series name in their name like "SeriesName - S01E01 - Episode Title"
            let seriesName = `Series ${seriesId.substring(0, 8)}...`;
            
            // Try to extract series name from episode name if it follows a pattern
            const episodeName = release.name || '';
            // Common patterns: "Series Name - S01E01", "Series Name S01E01", "Series Name 1x01"
            const seriesMatch = episodeName.match(/^(.+?)(?:\s*[-–]\s*)?(?:S\d+E\d+|Season|\d+x\d+|Episode)/i);
            if (seriesMatch) {
              seriesName = seriesMatch[1].trim();
            }
            
            seriesMap.set(seriesId, {
              id: seriesId,
              name: seriesName,
              categoryId: props.categoryFilter,
              thumbnailCID: release.thumbnailCID,
              contentCID: release.contentCID,
              metadata: {
                isSeries: true,
                isPseudoSeries: true,
                episodeCount: 0,
                episodes: []
              }
            });
          }
        }
        
        // Add episode to series
        seriesMap.get(seriesId).metadata.episodeCount++;
        seriesMap.get(seriesId).metadata.episodes.push(release);
      } else {
        // Standalone episode
        seriesMap.set(release.id, {
          ...release,
          metadata: {
            ...release.metadata,
            isStandaloneEpisode: true
          }
        });
      }
    }
    
    return Array.from(seriesMap.values());
  }

  return categoryReleases;
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
