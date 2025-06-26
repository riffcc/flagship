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
              @click="router.push(`/release/${item.id}`)"
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
import { useContentCategoriesQuery, useGetFeaturedReleasesQuery, useGetReleasesQuery } from '/@/plugins/lensService/hooks';
import { filterActivedFeatured, filterPromotedFeatured } from '../utils';

const router = useRouter();

// Optimize loading: Reduce stale time for faster fallback and eager loading
const {
  data: releases,
  isLoading: isReleasesLoading,
  isFetched: isReleasesFetched,
} = useGetReleasesQuery({
  staleTime: 1000 * 30, // 30s stale time for faster refresh
});

const {
  data: featuredReleases,
  isLoading: isFeaturedReleasesLoading,
  isFetched: isFeaturedReleasesFetched,
} = useGetFeaturedReleasesQuery({
  staleTime: 1000 * 30, // 30s stale time for faster refresh
});



const { data: contentCategories } = useContentCategoriesQuery();

const activedFeaturedReleases = computed<ReleaseItem<AnyObject>[]>(() => {
  if (!releases.value || !featuredReleases.value) return [];
  const activedFeaturedReleasesIds = featuredReleases.value
    .filter(filterActivedFeatured)
    .map(fr => fr.releaseId);
  return releases.value.filter(r => r.id && activedFeaturedReleasesIds.includes(r.id));
});

const promotedFeaturedReleases = computed<ReleaseItem<AnyObject>[]>(() => {
  if (!releases.value || !featuredReleases.value) return [];
  const promotedActivedFeaturedReleasesIds = featuredReleases.value
    .filter(filterActivedFeatured)
    .filter(filterPromotedFeatured)
    .map(fr => fr.releaseId);
  return releases.value.filter(r => r.id && promotedActivedFeaturedReleasesIds.includes(r.id));
});



function categorizeReleasesByFeaturedCategories(
  rels?: ReleaseItem<AnyObject>[],
  featuredCats?: Omit<ContentCategoryData<ContentCategoryMetadata>, 'siteAddress'>[],
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
        // A release is categorized, move to the next release.
        // It won't be added to multiple sections by this logic as release.category is singular.
        break;
      }
    }
  }
  return result;
}

const categorizedReleases = computed(() => {
  return categorizeReleasesByFeaturedCategories(activedFeaturedReleases.value, contentCategories.value);
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


// Progressive loading - show content as each query completes
const isFeaturedLoading = computed(() => {
  return isFeaturedReleasesLoading.value || !isFeaturedReleasesFetched.value;
});

const isReleasesOnlyLoading = computed(() => {
  return isReleasesLoading.value || !isReleasesFetched.value;
});

// Show loading until both queries complete
const isLoading = computed(() => {
  // Show loading if BOTH queries are still loading
  // This prevents showing "no featured content" before releases are loaded
  return isReleasesLoading.value || isFeaturedReleasesLoading.value;
});

const noFeaturedContent = computed(() => {
  // Only show "no featured content" if BOTH queries are done and there's no featured content
  if (isReleasesLoading.value || isFeaturedReleasesLoading.value) {
    return false; // Still loading, don't show "no featured content" yet
  }
  return promotedFeaturedReleases.value.length === 0 && activeSections.value.length === 0;
});

const noContent = computed(() => {
  // Only show "no content" if BOTH queries are done and there's truly no content anywhere
  if (isFeaturedLoading.value || isReleasesOnlyLoading.value) {
    return false; // Still loading something, don't show "no content" yet
  }
  return releases.value?.length === 0 && featuredReleases.value?.length === 0;
});

// Detailed logging to track content availability vs display
watch([releases, featuredReleases, isReleasesFetched, isFeaturedReleasesFetched], () => {
  console.log('[HomePage Debug] Data state changed:', {
    releasesCount: releases.value?.length || 0,
    featuredReleasesCount: featuredReleases.value?.length || 0,
    isReleasesFetched: isReleasesFetched.value,
    isFeaturedReleasesFetched: isFeaturedReleasesFetched.value,
    isReleasesLoading: isReleasesLoading.value,
    isFeaturedReleasesLoading: isFeaturedReleasesLoading.value,
    timestamp: new Date().toISOString(),
  });
}, { immediate: true });

watch([promotedFeaturedReleases, activeSections], () => {
  console.log('[HomePage Debug] Computed content changed:', {
    promotedFeaturedReleasesCount: promotedFeaturedReleases.value.length,
    activeSectionsCount: activeSections.value.length,
    activeSections: activeSections.value.map(s => ({ id: s.id, title: s.title, itemCount: s.items.length })),
    timestamp: new Date().toISOString(),
  });
}, { immediate: true });

watch([isLoading, noContent, noFeaturedContent], () => {
  console.log('[HomePage Debug] UI state changed:', {
    isLoading: isLoading.value,
    noContent: noContent.value,
    noFeaturedContent: noFeaturedContent.value,
    shouldShowContent: !isLoading.value && !noContent.value && !noFeaturedContent.value,
    timestamp: new Date().toISOString(),
  });
}, { immediate: true });


</script>
