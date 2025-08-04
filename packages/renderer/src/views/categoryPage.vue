<template>
  <v-container
    fluid
    class="pb-16 px-3"
  >
    <template v-if="props.showAll">
      <!-- Show all releases with infinite scroll -->
      <p class="text-h6 text-sm-h5 font-weight-bold mb-4">{{ pageCategory?.displayName }}</p>
      <infinite-release-list
        :category-slug="props.category"
        @release-click="handleItemClick"
      />
    </template>

    <template v-else>
      <!-- Show only featured releases -->
      <v-sheet
        v-if="isLoading"
        color="transparent"
        class="d-flex w-100 fill-height align-center justify-center"
      >
        <v-progress-circular
          indeterminate
          color="primary"
        ></v-progress-circular>
      </v-sheet>

      <template v-else-if="featuredReleasesInCategory.length > 0 && pageCategory">
        <content-section :title="pageCategory.displayName">
          <v-col
            v-for="item in featuredReleasesInCategory"
            :key="item.id"
          >
            <content-card
              :item="item"
              cursor-pointer
              @click="handleItemClick(item)"
            />
          </v-col>
        </content-section>
      </template>

      <v-sheet
        v-else
        color="transparent"
        class="d-flex flex-column mx-auto"
        max-width="16rem"
      >
        <p class="text-white text-center mb-2">No featured content in this category yet.</p>
      </v-sheet>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import ContentSection from '/@/components/home/contentSection.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import InfiniteReleaseList from '/@/components/misc/infiniteReleaseList.vue';
import { useContentCategoriesQuery, useGetReleasesQuery, useGetFeaturedReleasesQuery, useGetStructuresQuery } from '/@/plugins/lensService/hooks';
import { filterActivedFeatured } from '/@/utils';
import type { ReleaseItem } from '/@/types';

const props = defineProps<{
  category: string
  showAll?: boolean
}>();
const router = useRouter();

const { data: contentCategories } = useContentCategoriesQuery();
const { data: releases, isLoading: isReleasesLoading } = useGetReleasesQuery();
const { data: featuredReleases, isLoading: isFeaturedLoading } = useGetFeaturedReleasesQuery();

const pageCategory = computed(() => {
  const slug = props.category; // This is the slug from the URL like "tv-shows"
  
  // Find category by categoryId (slug) - API now merges duplicates
  return contentCategories.value?.find((cat) => cat.categoryId === slug);
});

// Fetch structures for TV shows
const isTVCategory = computed(() => {
  // Check if this is the TV category page
  if (props.category === 'tv-shows') return true;
  // Also check by display name in case of corrupted category data
  const category = pageCategory.value;
  return category?.displayName === 'TV Shows';
});
const { data: structures, isLoading: isStructuresLoading, error: structuresError } = useGetStructuresQuery({
  enabled: isTVCategory,
  // Retry less aggressively if service isn't ready
  retry: 1,
  retryDelay: 1000
});

const isLoading = computed(() => isReleasesLoading.value || isFeaturedLoading.value || (isTVCategory.value && isStructuresLoading.value));

// Get featured releases that are active and in this category
const featuredReleasesInCategory = computed<ReleaseItem[]>(() => {
  if (!releases.value || !featuredReleases.value) return [];

  // Get active featured release IDs
  const activeFeaturedReleaseIds = featuredReleases.value
    .filter(filterActivedFeatured)
    .map(fr => fr.releaseId);

  // Filter releases that are both featured and in this category
  // Look up each release's category to check if it matches our target slug
  const targetSlug = pageCategory.value?.categoryId; // e.g., 'music', 'tv-shows'
  const categoryReleases = releases.value.filter(r => {
    if (!r.id || !activeFeaturedReleaseIds.includes(r.id)) return false;
    
    // Look up this release's category
    const releaseCategory = contentCategories.value?.find(c => c.id === r.categoryId);
    return releaseCategory?.categoryId === targetSlug;
  });

  // For TV shows, group by series and return series tiles
  if (isTVCategory.value) {
    const seriesMap = new Map<string, any>();
    
    // Get all series structures if available
    const seriesStructures = structures.value ? 
      structures.value.filter((s: any) => s.type === 'series') : [];
    
    console.log('TV Category - Processing releases:', categoryReleases.length, 'Series structures:', seriesStructures.length, 'Error loading structures:', structuresError.value);
    
    // For each episode, find its series
    for (const release of categoryReleases) {
      const seriesId = release.metadata?.seriesId;
      
      if (seriesId) {
        // Episode has a series ID from the upload form
        const series = seriesStructures.find((s: any) => s.id === seriesId);
        
        if (!seriesMap.has(seriesId)) {
          // Create entry for the series
          if (series) {
            // We have the series structure
            seriesMap.set(seriesId, {
              id: series.id,
              name: series.name,
              categoryId: pageCategory.value?.id,
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
            // Series structure doesn't exist yet, but we know it's part of a series
            // This shouldn't happen if upload form works correctly, but handle it anyway
            seriesMap.set(seriesId, {
              id: seriesId,
              name: `Series ${seriesId.substring(0, 8)}...`, // Fallback name
              categoryId: pageCategory.value?.id,
              thumbnailCID: release.thumbnailCID,
              contentCID: release.contentCID,
              description: `TV Series`,
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
        
        // Sort episodes by season and episode number
        seriesMap.get(seriesId).metadata.episodes.sort((a: any, b: any) => {
          const aSeason = a.metadata?.seasonNumber || 0;
          const bSeason = b.metadata?.seasonNumber || 0;
          const aEpisode = a.metadata?.episodeNumber || 0;
          const bEpisode = b.metadata?.episodeNumber || 0;
          
          if (aSeason !== bSeason) return aSeason - bSeason;
          return aEpisode - bEpisode;
        });
      } else {
        // Episode without series metadata - show as standalone
        // This is likely old content or improperly uploaded
        seriesMap.set(release.id, {
          ...release,
          metadata: {
            ...release.metadata,
            isStandaloneEpisode: true
          }
        });
      }
    }
    
    console.log('Series map created:', seriesMap.size, 'series/episodes');
    return Array.from(seriesMap.values());
  }

  return categoryReleases;
});

// Handle clicking on items - navigate to series or release page
const handleItemClick = (item: ReleaseItem) => {
  if (item.metadata?.isSeries) {
    // Navigate to the series view page
    router.push(`/series/${item.id}`);
  } else {
    router.push(`/release/${item.id}`);
  }
};

</script>
