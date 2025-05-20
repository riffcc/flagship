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
        <p class="text-white text-center mb-2">No featured content yet. It will appear once some content is marked as featured</p>
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
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useQuery } from '@tanstack/vue-query';
import type { AnyObject, ContentCategoryData, ContentCategoryMetadata} from '@riffcc/lens-sdk';
import {
  ID_PROPERTY,
  RELEASE_CATEGORY_ID_PROPERTY,
  RELEASE_CONTENT_CID_PROPERTY,
  RELEASE_METADATA_PROPERTY,
  RELEASE_NAME_PROPERTY,
  RELEASE_THUMBNAIL_CID_PROPERTY,
} from '@riffcc/lens-sdk';
import ContentSection from '/@/components/home/contentSection.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import FeaturedSlider from '/@/components/home/featuredSlider.vue';
import type { ReleaseItem, FeaturedReleaseItem } from '/@/types';
import { useLensService } from '/@/plugins/lensService/utils';
import { DEFAULT_CONTENT_CATEGORIES } from '/@/constants/contentCategories';
import { useStaticStatus } from '/@/composables/staticStatus';
import { useStaticReleases } from '/@/composables/staticReleases';
import { filterActivedFeatured, filterPromotedFeatured } from '../utils';

const router = useRouter();
const { staticStatus } = useStaticStatus();
const { staticReleases, staticFeaturedReleases } = useStaticReleases();
const { lensService } = useLensService();
const {
  data: releases,
  isLoading: isReleasesLoading,
  isFetched: isReleasesFetched,
} = useQuery<ReleaseItem<AnyObject>[]>({
  queryKey: ['releases'],
  queryFn: async () => {
    if (staticStatus.value) {
      return staticReleases.value;
    } else {
      const result = await lensService.getLatestReleases();
      return result.map(r => ({
        [ID_PROPERTY]: r.id,
        [RELEASE_NAME_PROPERTY]: r.name,
        [RELEASE_CATEGORY_ID_PROPERTY]: r.categoryId,
        [RELEASE_CONTENT_CID_PROPERTY]: r.contentCID,
        [RELEASE_THUMBNAIL_CID_PROPERTY]: r.thumbnailCID,
        [RELEASE_METADATA_PROPERTY]: r.metadata ? JSON.parse(r.metadata) : undefined,
      }));;
    }
  },
});

const {
  data: featuredReleases,
  isLoading: isFeaturedReleasesLoading,
  isFetched: isFeaturedReleasesFetched,
} = useQuery<FeaturedReleaseItem[]>({
  queryKey: ['featuredReleases'],
  queryFn: async () => {
    if (staticStatus.value) {
      return staticFeaturedReleases.value;
    } else {
      // Not implemented
      return [];
      // const result = await lensService.getLatestFeaturedReleases();
      // return result.map(r => ({
      //   [ID_PROPERTY]: r.id,
      //   [RELEASE_NAME_PROPERTY]: r.name,
      //   [RELEASE_CATEGORY_ID_PROPERTY]: r.categoryId,
      //   [RELEASE_CONTENT_CID_PROPERTY]: r.contentCID,
      //   [RELEASE_THUMBNAIL_CID_PROPERTY]: r.thumbnailCID,
      //   [RELEASE_METADATA_PROPERTY]: r.metadata ? JSON.parse(r.metadata) : undefined,
      // }));;
    }
  },
});


const {
  data: contentCategories,
} = useQuery({
  queryKey: ['contentCategories'],
  queryFn: async () => {
    // const result = await lensService.getContentCategories();
    return await new Promise<ContentCategoryData<ContentCategoryMetadata>[]>(r => r(DEFAULT_CONTENT_CATEGORIES));
  },
});

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
        navigationPath: `/featured/${categoryId}`, // Generic path, adjust if specific paths needed
      };
    })
    .filter(section => section.items.length > 0);
});


const isLoading = computed(() => isReleasesLoading || isFeaturedReleasesLoading);
const noFeaturedContent = computed(() => isReleasesFetched && isFeaturedReleasesFetched && promotedFeaturedReleases.value.length === 0 && activeSections.value.length === 0);
const noContent = computed(() => isReleasesFetched && isFeaturedReleasesFetched && releases.value?.length === 0 && featuredReleases.value?.length === 0);

</script>

