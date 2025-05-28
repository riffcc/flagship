<template>
  <div
    ref="scrollContainer"
    class="infinite-release-list"
  >
    <v-sheet
      v-if="isLoading && currentPage === 1"
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
      <div class="releases-wrapper">
        <v-row
          dense
          justify="center"
        >
          <v-col
            v-for="item in visibleReleases"
            :key="item.id"
            cols="auto"
          >
            <content-card
              :item="item"
              cursor-pointer
              :source-site="(item.metadata?.['sourceSite'] as string | undefined)"
              @click="$emit('release-click', item)"
            />
          </v-col>
          
          <!-- Lightweight placeholder tiles -->
          <v-col
            v-for="n in placeholderCount"
            :key="`placeholder-${n}`"
            cols="auto"
          >
            <div style="width: 15rem; height: 1px;"></div>
          </v-col>
        </v-row>
      </div>
      
      <v-sheet
        v-if="hasMore || isLoadingMore"
        v-intersect="onIntersect"
        height="100"
        class="d-flex align-center justify-center"
        color="transparent"
      >
        <v-progress-circular
          v-if="isLoadingMore"
          indeterminate
          color="primary"
          size="32"
        ></v-progress-circular>
      </v-sheet>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch } from 'vue';
import type { ReleaseItem, AnyObject } from '/@/types';
import ContentCard from './contentCard.vue';
import { useComplexFederationIndexQuery } from '/@/plugins/lensService/hooks';
import { federationEntriesToReleases } from '/@/utils/federationIndex';

const props = defineProps<{
  categoryFilter?: string;
  pageSize?: number;
}>();

const emit = defineEmits<{
  'release-click': [release: ReleaseItem<AnyObject>];
}>();

// Number of items to show/fetch per "page"
const PAGE_SIZE = props.pageSize || 60;
const FETCH_SIZE = 100; // Fetch 100 items at a time from federation index
const currentPage = ref(1);
const windowWidth = ref(window.innerWidth);
const allFetchedReleases = ref<ReleaseItem<AnyObject>[]>([]);
const currentOffset = ref(0);
const isLoadingMore = ref(false);
const hasMoreData = ref(true);

// Initial query for federation index
const { 
  data: federationEntries, 
  isLoading,
  refetch,
} = useComplexFederationIndexQuery({
  categoryId: props.categoryFilter,
  limit: FETCH_SIZE,
  offset: 0,
});

// Convert initial entries to releases
watch(federationEntries, (entries) => {
  if (entries && currentOffset.value === 0) {
    allFetchedReleases.value = federationEntriesToReleases(entries);
    if (entries.length < FETCH_SIZE) {
      hasMoreData.value = false;
    }
  }
}, { immediate: true });

// Track window resize
const updateWidth = () => {
  windowWidth.value = window.innerWidth;
};

onMounted(() => {
  window.addEventListener('resize', updateWidth);
});

onUnmounted(() => {
  window.removeEventListener('resize', updateWidth);
});

// Calculate visible releases based on current page
const visibleReleases = computed(() => {
  const endIndex = currentPage.value * PAGE_SIZE;
  return allFetchedReleases.value.slice(0, endIndex);
});

// Calculate if we need to fetch more data
const needsMoreData = computed(() => {
  const totalVisible = currentPage.value * PAGE_SIZE;
  const totalFetched = allFetchedReleases.value.length;
  return totalVisible >= totalFetched - PAGE_SIZE && hasMoreData.value;
});

// Determine if we have more content to show
const hasMore = computed(() => {
  return visibleReleases.value.length < allFetchedReleases.value.length || hasMoreData.value;
});

// Calculate placeholder count
const placeholderCount = computed(() => {
  const cardWidth = 15 * 16; // 15rem in pixels
  const cardsPerRow = Math.floor(windowWidth.value / cardWidth);
  const visibleCount = visibleReleases.value.length;
  const remainder = visibleCount % cardsPerRow;
  return remainder > 0 ? cardsPerRow - remainder : 0;
});

// Fetch more data from federation index
async function fetchMoreData() {
  if (isLoadingMore.value || !hasMoreData.value) return;
  
  isLoadingMore.value = true;
  currentOffset.value += FETCH_SIZE;
  
  try {
    // Use the complex query to fetch more with offset
    const { lensService } = await import('/@/plugins/lensService/hooks').then(m => ({ lensService: m.useLensService().lensService }));
    
    const moreEntries = await lensService.complexFederationIndexQuery({
      categoryId: props.categoryFilter,
      limit: FETCH_SIZE,
      offset: currentOffset.value,
    });
    
    if (moreEntries.length > 0) {
      const moreReleases = federationEntriesToReleases(moreEntries);
      allFetchedReleases.value = [...allFetchedReleases.value, ...moreReleases];
    }
    
    if (moreEntries.length < FETCH_SIZE) {
      hasMoreData.value = false;
    }
  } catch (error) {
    console.error('Failed to fetch more federation data:', error);
    hasMoreData.value = false;
  } finally {
    isLoadingMore.value = false;
  }
}

// Intersection observer callback
const onIntersect = (isIntersecting: boolean) => {
  if (isIntersecting && hasMore.value && !isLoadingMore.value) {
    if (needsMoreData.value) {
      // Need to fetch more data first
      fetchMoreData().then(() => {
        currentPage.value += 1;
      });
    } else {
      // Just show more of what we already have
      currentPage.value += 1;
    }
  }
};
</script>

<style scoped>
.infinite-release-list {
  width: 100%;
}

.releases-wrapper {
  width: 100%;
}
</style>