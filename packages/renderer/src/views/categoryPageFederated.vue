<template>
  <v-container
    fluid
    class="fill-height pb-16 px-3"
  >
    <template v-if="props.showAll">
      <!-- Show all releases with infinite scroll -->
      <p class="text-h6 text-sm-h5 font-weight-bold mb-4">{{ pageTitle }}</p>
      <infinite-release-list-federated
        :category-filter="props.category"
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
      
      <template v-else-if="featuredReleasesInCategory.length > 0">
        <content-section :title="pageTitle">
          <v-col
            v-for="item in featuredReleasesInCategory"
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
import InfiniteReleaseListFederated from '/@/components/misc/infiniteReleaseListFederated.vue';
import { 
  useContentCategoriesQuery, 
  useFederationIndexByCategoryQuery,
  useComplexFederationIndexQuery, 
} from '/@/plugins/lensService/hooks';
import { federationEntriesToReleases } from '/@/utils/federationIndex';
import type { ReleaseItem, AnyObject } from '/@/types';

const props = defineProps<{
  category: string
  showAll?: boolean
}>();
const router = useRouter();

const { data: contentCategories } = useContentCategoriesQuery();

// Get category-specific content from federation index
const { 
  data: categoryEntries, 
  isLoading: isCategoryLoading, 
} = useFederationIndexByCategoryQuery(props.category, {
  limit: props.showAll ? 1000 : 50, // Get more for "show all", less for featured
});

// For featured view, we might want to filter by tags or recent high-quality content
const { 
  data: featuredCategoryEntries, 
  isLoading: isFeaturedLoading, 
} = useComplexFederationIndexQuery({
  categoryId: props.category,
  limit: 20, // Just get top 20 for featured view
  // Could add more filters here like tags: ['featured'] or afterTimestamp for recent
}, {
  enabled: !props.showAll, // Only run this query for featured view
});

const isLoading = computed(() => isCategoryLoading.value || (!props.showAll && isFeaturedLoading.value));

const pageCategory = computed(() => {
  const categoryId = props.category;
  const category = contentCategories.value?.find((cat) => cat.id === categoryId);
  return category;
});

const pageTitle = computed(() => {
  const displayName = pageCategory.value?.displayName ?? props.category;
  return props.showAll ? displayName : `Featured ${displayName}`;
});

// Convert federation entries to release format
const releasesInCategory = computed<ReleaseItem<AnyObject>[]>(() => {
  if (!categoryEntries.value) return [];
  return federationEntriesToReleases(categoryEntries.value);
});

const featuredReleasesInCategory = computed<ReleaseItem<AnyObject>[]>(() => {
  if (props.showAll) {
    // For "show all", just use all category entries
    return releasesInCategory.value;
  } else {
    // For featured view, use the specifically queried featured entries
    if (!featuredCategoryEntries.value) return [];
    return federationEntriesToReleases(featuredCategoryEntries.value);
  }
});
</script>