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
              @click="router.push(`/release/${item.id}`)"
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
import { useContentCategoriesQuery, useGetFeaturedReleasesQuery, useGetReleasesQuery } from '/@/plugins/lensService/hooks';
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
    .map(featuredCategory => {
      // Use the category's ID (UUID) to match releases, not the slug
      const items = releasesByCategory.get(featuredCategory.id) || [];
      return {
        id: featuredCategory.categoryId,
        title: featuredCategory.categoryId === 'tv-shows' ? featuredCategory.displayName : `Featured ${featuredCategory.displayName}`,
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
</script>
