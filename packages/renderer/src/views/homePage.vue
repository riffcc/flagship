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
    <template v-if="!noContent">
      <featured-slider
        v-if="(promotedFeaturedReleases?.length || 0) > 0"
      />

      <template
        v-for="section in activeSections"
        :key="section.id"
      >
        <!-- Special TV Show Alert -->
        <v-alert
          v-if="section.categoryId === 'tvShow' && section.items.length > 0"
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
          :pagination="section.categoryId === 'tvShow' && section.items.length > 4"
          @navigate="() => router.push(section.navigationPath)"
        >
          <v-col
            v-for="item in section.items"
            :key="item.id"
          >
            <content-card
              :item="item"
              cursor-pointer
              :source-site="item.sourceSite"
              @click="router.push(`/release/${item.id}`)"
            />
          </v-col>
        </content-section>
      </template>
    </template>
    <v-sheet
      v-else
      color="transparent"
      class="d-flex flex-column mx-auto"
      max-width="16rem"
    >
      <p class="text-white text-center mb-2">No content here. Please upload a release first.</p>
      <v-btn
        color="primary-darken-1"
        @click="router.push('/upload')"
      >
        Go to Upload
      </v-btn>
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import ContentSection from '/@/components/home/contentSection.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import FeaturedSlider from '/@/components/home/featuredSlider.vue';
import { type ReleaseItem, useReleasesStore } from '/@/stores/releases';
import { useContentCategoriesStore, type ContentCategoryItem } from '/@/stores/contentCategories';
import { storeToRefs } from 'pinia';

const router = useRouter();

const releasesStore = useReleasesStore();
const { activedFeaturedReleases, promotedFeaturedReleases, isLoading, noContent } = storeToRefs(releasesStore);

const contentCategoriesStore = useContentCategoriesStore();
const { featuredContentCategories } = storeToRefs(contentCategoriesStore);

function categorizeReleasesByFeaturedCategories(
  releases: ReleaseItem[],
  featuredCats: ContentCategoryItem[],
  limitPerCategory: number = 8,
): Record<string, ReleaseItem[]> {
  const result: Record<string, ReleaseItem[]> = {};
  const addedReleaseIds = new Set<string>();

  featuredCats.forEach(fc => {
    result[fc.contentCategory.categoryId] = [];
  });

  for (const release of releases) {
    if (!release.id || addedReleaseIds.has(release.id)) {
      continue;
    }

    for (const fc of featuredCats) {
      const currentCategoryId = fc.contentCategory.categoryId;
      if (release.category === currentCategoryId) {
        if (result[currentCategoryId].length < limitPerCategory) {
          result[currentCategoryId].push(release);
          addedReleaseIds.add(release.id);
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
  return categorizeReleasesByFeaturedCategories(activedFeaturedReleases.value, featuredContentCategories.value);
});

const activeSections = computed(() => {
  return featuredContentCategories.value
    .map(fc => {
      const categoryId = fc.contentCategory.categoryId;
      const items = categorizedReleases.value[categoryId] || [];
      return {
        id: fc.id,
        categoryId,
        title: categoryId === 'tvShow' ? fc.contentCategory.displayName : `Featured ${fc.contentCategory.displayName}`,
        items: items,
        navigationPath: `/featured/${categoryId}`, // Generic path, adjust if specific paths needed
      };
    })
    .filter(section => section.items.length > 0);
});

</script>

