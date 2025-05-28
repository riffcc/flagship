<template>
  <v-container class="fill-height pb-16">
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
              :source-site="(item.metadata?.['sourceSite'] as string | undefined)"
              @click="handleContentClick(item)"
            />
          </v-col>
        </content-section>
      </template>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import { useRouter } from 'vue-router';

import type { AnyObject, ContentCategoryData, ContentCategoryMetadata } from '@riffcc/lens-sdk';
import ContentSection from '/@/components/home/contentSection.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import FeaturedSlider from '/@/components/home/featuredSlider.vue';
import type { ReleaseItem } from '/@/types';
import { 
  useContentCategoriesQuery, 
  useFederationIndexFeaturedQuery,
  useFederationIndexRecentQuery,
  useComplexFederationIndexQuery, 
} from '/@/plugins/lensService/hooks';
import { federationEntriesToReleases, extractFeaturedFromIndex } from '/@/utils/federationIndex';

const router = useRouter();

// Use federation index for featured content
const {
  data: federationFeatured,
  isLoading: isFederationFeaturedLoading,
  isFetched: isFederationFeaturedFetched,
} = useFederationIndexFeaturedQuery({
  limit: 50, // Get top 50 featured items
  staleTime: 1000 * 30, // 30s stale time
});

// Get recent content from federation index
const {
  data: federationRecent,
  isLoading: isFederationRecentLoading,
  isFetched: isFederationRecentFetched,
} = useFederationIndexRecentQuery({
  limit: 200, // Get recent 200 items for categorization
  staleTime: 1000 * 60, // 1 minute stale time
});

// Get content categories for organization
const { data: contentCategories } = useContentCategoriesQuery();

// Convert federation index entries to release format for UI compatibility
const featuredReleases = computed<ReleaseItem<AnyObject>[]>(() => {
  if (!federationFeatured.value) return [];
  return federationEntriesToReleases(federationFeatured.value);
});

const allReleases = computed<ReleaseItem<AnyObject>[]>(() => {
  if (!federationRecent.value) return [];
  return federationEntriesToReleases(federationRecent.value);
});

// Extract promoted featured releases (for the slider)
const promotedFeaturedReleases = computed<ReleaseItem<AnyObject>[]>(() => {
  if (!federationFeatured.value) return [];
  // Take the first 10 as "promoted"
  return federationEntriesToReleases(federationFeatured.value.slice(0, 10));
});

// Categorize releases by featured categories
function categorizeReleasesByFeaturedCategories(
  rels?: ReleaseItem<AnyObject>[],
  featuredCats?: ContentCategoryData<ContentCategoryMetadata>[],
  limitPerCategory: number = 8,
): Record<string, ReleaseItem<AnyObject>[]> {
  const result: Record<string, ReleaseItem<AnyObject>[]> = {};
  if (!rels || !featuredCats) {
    return result;
  }
  const addedReleaseIds = new Set<string>();

  featuredCats.forEach(fc => {
    result[fc.id] = [];
  });

  for (const rel of rels) {
    if (!rel.id || addedReleaseIds.has(rel.id)) {
      continue;
    }

    for (const fc of featuredCats) {
      const currentCategoryId = fc.id;
      if (rel.categoryId === currentCategoryId) {
        if (result[currentCategoryId].length < limitPerCategory) {
          result[currentCategoryId].push(rel);
          addedReleaseIds.add(rel.id);
        }
        break;
      }
    }
  }
  return result;
}

const categorizedReleases = computed(() => {
  return categorizeReleasesByFeaturedCategories(allReleases.value, contentCategories.value);
});

const activeSections = computed<{
  id: string;
  title: string;
  items: ReleaseItem<AnyObject>[];
  navigationPath: string;
}[]>(() => {
  if (!contentCategories.value) return [];
  return contentCategories.value
    .filter(c => c.featured)
    .map(fc => {
      const categoryId = fc.id;
      const items = categorizedReleases.value[categoryId] || [];
      return {
        id: fc.id,
        title: categoryId === 'tvShow' ? fc.displayName : `Featured ${fc.displayName}`,
        items: items,
        navigationPath: `/featured/${categoryId}`,
      };
    })
    .filter(section => section.items.length > 0);
});

// Loading states
const isLoading = computed(() => {
  return isFederationFeaturedLoading.value || isFederationRecentLoading.value;
});

const noFeaturedContent = computed(() => {
  if (isFederationFeaturedLoading.value || isFederationRecentLoading.value) {
    return false;
  }
  return promotedFeaturedReleases.value.length === 0 && activeSections.value.length === 0;
});

const noContent = computed(() => {
  if (isFederationFeaturedLoading.value || isFederationRecentLoading.value) {
    return false;
  }
  return federationFeatured.value?.length === 0 && federationRecent.value?.length === 0;
});

// Handle content click - for federation index entries, we already have the IPFS CID
function handleContentClick(item: ReleaseItem<AnyObject>) {
  // The content player already knows how to handle IPFS CIDs
  // Just navigate to the release page with the ID
  router.push(`/release/${item.id}`);
}

// Debug logging
watch([federationFeatured, federationRecent], () => {
  console.log('[HomePage Federated] Data state changed:', {
    featuredCount: federationFeatured.value?.length || 0,
    recentCount: federationRecent.value?.length || 0,
    timestamp: new Date().toISOString(),
  });
}, { immediate: true });

watch([promotedFeaturedReleases, activeSections], () => {
  console.log('[HomePage Federated] Computed content changed:', {
    promotedFeaturedReleasesCount: promotedFeaturedReleases.value.length,
    activeSectionsCount: activeSections.value.length,
    activeSections: activeSections.value.map(s => ({ id: s.id, title: s.title, itemCount: s.items.length })),
    timestamp: new Date().toISOString(),
  });
}, { immediate: true });
</script>