<template>
  <v-container
    fluid
    class="fill-height pb-16 px-3"
  >
    <template v-if="props.showAll">
      <!-- Show all releases with infinite scroll -->
      <p class="text-h6 text-sm-h5 font-weight-bold mb-4">{{ pageCategory?.displayName }}</p>
      <infinite-release-list
        :category-filter="pageCategory?.id"
        @release-click="(release) => router.push(`/release/${release.id}`)"
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
              @click="router.push(`/release/${item.id}`)"
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
import { useContentCategoriesQuery, useGetReleasesQuery, useGetFeaturedReleasesQuery } from '/@/plugins/lensService/hooks';
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

const isLoading = computed(() => isReleasesLoading.value || isFeaturedLoading.value);

const pageCategory = computed(() => {
  const categoryId = props.category;
  const category = contentCategories.value?.find((cat) => cat.categoryId === categoryId);
  return category;
});

// Get featured releases that are active and in this category
const featuredReleasesInCategory = computed<ReleaseItem[]>(() => {
  if (!releases.value || !featuredReleases.value) return [];

  // Get active featured release IDs
  const activeFeaturedReleaseIds = featuredReleases.value
    .filter(filterActivedFeatured)
    .map(fr => fr.releaseId);

  // Filter releases that are both featured and in this category
  return releases.value.filter(r =>
    r.id &&
    activeFeaturedReleaseIds.includes(r.id) &&
    r.categoryId === pageCategory.value?.id,
  );
});

</script>
