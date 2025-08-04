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
  categoryFilter?: string;  // Category ID (hash) to filter by
  categorySlug?: string;    // Category slug (e.g., 'music', 'tv-shows') to filter by
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
const { data: releases, isLoading } = useGetReleasesQuery();

// Get content categories to check if this is a TV category
const { data: contentCategories } = useContentCategoriesQuery();

// Check if this is the TV shows category
const isTVCategory = computed(() => {
  return props.categorySlug === 'tv-shows' || props.categoryFilter === 'tv-shows';
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

// Filter releases client-side if we have a category filter
const filteredReleases = computed(() => {
  if (!releases.value) return [];

  let categoryReleases = releases.value;
  
  // Filter by category slug if specified (for federation support)
  if (props.categorySlug && contentCategories.value) {
    // Find all categories with this slug
    const matchingCategories = contentCategories.value.filter(c => c.categoryId === props.categorySlug);
    if (matchingCategories.length > 0) {
      // Get all category IDs including federated ones
      const allCategoryIds = new Set<string>();
      for (const cat of matchingCategories) {
        allCategoryIds.add(cat.id);
        if (cat.allIds) {
          cat.allIds.forEach(id => allCategoryIds.add(id));
        }
      }
      // Filter releases that match any of these category IDs
      categoryReleases = categoryReleases.filter(r => allCategoryIds.has(r.categoryId));
    }
  } else if (props.categoryFilter) {
    // Direct category ID filter (for non-federated)
    categoryReleases = categoryReleases.filter(r => r.categoryId === props.categoryFilter);
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
