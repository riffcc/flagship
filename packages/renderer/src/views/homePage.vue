<template>
  <v-container fluid class="fill-height pb-16">
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
    <v-sheet
      v-else-if="noContent || noFeaturedContent"
      color="transparent"
      class="d-flex flex-column mx-auto"
      max-width="16rem"
    >
      <template v-if="noContent">
        <p class="text-white text-center mb-2">No content here. Please upload a release first.</p>
        <v-btn
          color="primary-darken-1"
          @click="router.push('/upload')"
        >
          Go to Upload
        </v-btn>
      </template>
      <template v-else-if="noFeaturedContent">
        <p class="text-white text-center mb-2">
          No featured content yet. It will appear once some content is marked as
          featured
        </p>
      </template>
    </v-sheet>
    <template v-else>
      <featured-slider :promoted-featured-releases="promotedFeaturedReleases" />
      <template
        v-for="section in activeSections"
        :key="section.id"
      >
        <v-alert
          v-if="section.id === 'tvShow' && section.items.length > 0"
          type="info"
          class="mt-8 mb-n8"
          color="black"
          text-color="white"
        >
          Riff.CC: We're still adding UI support for TV shows, but below you can see what TV will look
          like on this platform.
        </v-alert>

        <content-section
          :title="section.title"
          :pagination="section.id === 'tvShow' && section.items.length > 4"
          @navigate="() => router.push(section.navigationPath)"
        >
          <v-col
            v-for="item in section.items"
            :key="item.id"
          >
            <content-card
              :item="item"
              cursor-pointer
              @click="handleItemClick(item)"
            />
          </v-col>
          <!-- Ghost items to maintain alignment -->
          <v-col
            v-for="n in Math.max(0, 8 - section.items.length)"
            :key="`ghost-${section.id}-${n}`"
            style="visibility: hidden;"
          >
            <content-card
              :item="{}"
              cursor-pointer
            />
          </v-col>
        </content-section>
      </template>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';

import ContentSection from '/@/components/home/contentSection.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import FeaturedSlider from '/@/components/home/featuredSlider.vue';
import type { ReleaseItem } from '/@/types';
import { useContentCategoriesQuery, useGetFeaturedReleasesQuery, useGetReleasesQuery, useGetStructuresQuery } from '/@/plugins/lensService/hooks';
import { filterActivedFeatured, filterPromotedFeatured } from '../utils';

const router = useRouter();

const {
  data: releases,
  isLoading: isReleasesLoading,
  isFetched: isReleasesFetched,
} = useGetReleasesQuery();

const {
  data: featuredReleases,
  isLoading: isFeaturedReleasesLoading,
  isFetched: isFeaturedReleasesFetched,
} = useGetFeaturedReleasesQuery();

const { data: contentCategories } = useContentCategoriesQuery();

// Fetch structures for TV series - only load when needed and handle errors gracefully
const { data: structures } = useGetStructuresQuery({
  retry: 1,
  retryDelay: 1000,
  // Only enable if we have releases
  enabled: computed(() => !!releases.value && releases.value.length > 0)
});

const activedFeaturedReleases = computed<ReleaseItem[]>(() => {
  if (!releases.value || !featuredReleases.value) return [];
  const activedFeaturedReleasesIds = featuredReleases.value
    .filter(filterActivedFeatured)
    .map(fr => fr.releaseId);
  return releases.value.filter(r => r.id && activedFeaturedReleasesIds.includes(r.id));
});

const promotedFeaturedReleases = computed<ReleaseItem[]>(() => {
  if (!releases.value || !featuredReleases.value) return [];
  const promotedActivedFeaturedReleasesIds = featuredReleases.value
    .filter(filterActivedFeatured)
    .filter(filterPromotedFeatured)
    .map(fr => fr.releaseId);
  return releases.value.filter(r => r.id && promotedActivedFeaturedReleasesIds.includes(r.id));
});


const activeSections = computed(() => {
  const limitPerCategory = 8;
  if (!contentCategories.value || !activedFeaturedReleases.value) return [];

  const releasesByCategory = new Map<string, ReleaseItem[]>();
  for (const release of activedFeaturedReleases.value) {
    if (!release.categoryId) continue;
    if (!releasesByCategory.has(release.categoryId)) {
      releasesByCategory.set(release.categoryId, []);
    }
    releasesByCategory.get(release.categoryId)!.push(release);
  }

  return contentCategories.value
    .filter(category => category.featured)
    .sort((a, b) => {
      // Put TV Shows last
      if (a.displayName === 'TV Shows') return 1;
      if (b.displayName === 'TV Shows') return -1;
      return 0;
    })
    .map(featuredCategory => {
      // Collect releases from all federated category IDs
      let items: ReleaseItem[] = [];
      if (featuredCategory.allIds) {
        // Merged category - get releases from all IDs
        for (const catId of featuredCategory.allIds) {
          items.push(...(releasesByCategory.get(catId) || []));
        }
      } else {
        // Single category (backward compatibility)
        items = releasesByCategory.get(featuredCategory.id) || [];
      }
      
      // For TV shows, group episodes by series
      // Check both categoryId and displayName for TV category (handle corrupted data)
      if (featuredCategory.categoryId === 'tv-shows' || featuredCategory.displayName === 'TV Shows') {
        const seriesMap = new Map<string, any>();
        const seriesStructures = structures.value ? 
          structures.value.filter((s: any) => s.type === 'series') : [];
        
        for (const release of items) {
          const seriesId = release.metadata?.seriesId;
          
          if (seriesId) {
            // Episode has a series ID
            const series = seriesStructures.find((s: any) => s.id === seriesId);
            
            if (!seriesMap.has(seriesId)) {
              if (series) {
                seriesMap.set(seriesId, {
                  id: series.id,
                  name: series.name,
                  categoryId: featuredCategory.id,
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
                // Series structure missing
                seriesMap.set(seriesId, {
                  id: seriesId,
                  name: `Series ${seriesId.substring(0, 8)}...`,
                  categoryId: featuredCategory.id,
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
            
            seriesMap.get(seriesId).metadata.episodeCount++;
            seriesMap.get(seriesId).metadata.episodes.push(release);
            
            // Sort episodes
            seriesMap.get(seriesId).metadata.episodes.sort((a: any, b: any) => {
              const aSeason = a.metadata?.seasonNumber || 0;
              const bSeason = b.metadata?.seasonNumber || 0;
              const aEpisode = a.metadata?.episodeNumber || 0;
              const bEpisode = b.metadata?.episodeNumber || 0;
              
              if (aSeason !== bSeason) return aSeason - bSeason;
              return aEpisode - bEpisode;
            });
          } else {
            // Standalone episode without series
            seriesMap.set(release.id, {
              ...release,
              metadata: {
                ...release.metadata,
                isStandaloneEpisode: true
              }
            });
          }
        }
        
        items = Array.from(seriesMap.values());
      }
      
      return {
        id: featuredCategory.categoryId,
        title: featuredCategory.categoryId === 'tv-shows' ? 'Featured TV' : `Featured ${featuredCategory.displayName}`,
        items: items.slice(0, limitPerCategory),
        navigationPath: `/featured/${featuredCategory.categoryId}`,
      };
    })
    .filter(section => section.items.length > 0);
});

const isLoading = computed(() => {
  return isReleasesLoading.value || isFeaturedReleasesLoading.value;
});

const noFeaturedContent = computed(() => {
  if (!isReleasesFetched.value || !isFeaturedReleasesFetched.value) {
    return false;
  }
  return promotedFeaturedReleases.value.length === 0 && activeSections.value.length === 0;
});

const noContent = computed(() => {
  if (!isReleasesFetched.value || !isFeaturedReleasesFetched.value) {
    return false;
  }
  return releases.value?.length === 0 && featuredReleases.value?.length === 0;
});

// Handle clicking on items - navigate to series or release page
const handleItemClick = (item: any) => {
  if (item.metadata?.isSeries) {
    // Navigate to the series view page
    router.push(`/series/${item.id}`);
  } else {
    router.push(`/release/${item.id}`);
  }
};
</script>
