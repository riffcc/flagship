<template>
  <div class="search-bar-container">
    <v-text-field
      v-model="query"
      label="Search music, movies, TV shows..."
      prepend-inner-icon="mdi-magnify"
      clearable
      variant="outlined"
      density="comfortable"
      hide-details
      @update:model-value="onSearch"
      @keydown.enter="onEnter"
      @focus="showResults = true"
      @blur="onBlur"
    >
      <template #append-inner>
        <v-chip v-if="isIndexReady && indexedCount > 0" size="x-small" variant="text">
          {{ indexedCount }} items
        </v-chip>
      </template>
    </v-text-field>

    <!-- Search Results Dropdown -->
    <SearchResults
      v-if="showResults && results.length > 0"
      :results="results"
      :query="query"
      @select="onSelectResult"
      @close="showResults = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useDebounceFn } from '@vueuse/core';
import { useRouter } from 'vue-router';
import { useLocalSearch } from '/@/composables/useLocalSearch';
import SearchResults from './SearchResults.vue';

const router = useRouter();
const { search, isIndexReady, indexedCount } = useLocalSearch();

const query = ref('');
const results = ref<any[]>([]);
const showResults = ref(false);

// Debounced search function
const onSearch = useDebounceFn((value: string) => {
  if (!value || value.length < 2) {
    results.value = [];
    return;
  }

  const searchResults = search(value);
  results.value = searchResults;
  showResults.value = searchResults.length > 0;
}, 300);

// Handle Enter key
function onEnter() {
  if (results.value.length > 0) {
    // Navigate to first result
    onSelectResult(results.value[0]);
  }
}

// Handle result selection
function onSelectResult(result: any) {
  showResults.value = false;
  query.value = '';

  // Navigate to content page based on type
  const route = getRouteForContent(result);
  if (route) {
    router.push(route);
  }
}

// Get route for content item
function getRouteForContent(content: any): string | null {
  if (!content.id) return null;

  // Map content type to route
  switch (content.type) {
    case 'artist':
      return `/artists/${content.id}`;
    case 'music':
      return `/release/${content.id}`;
    case 'movie':
    case 'tv':
      return `/release/${content.id}`;
    default:
      return `/release/${content.id}`;
  }
}

// Handle blur with delay to allow click on results
function onBlur() {
  setTimeout(() => {
    showResults.value = false;
  }, 200);
}

// Clear results when query is cleared
watch(query, (newValue) => {
  if (!newValue) {
    results.value = [];
    showResults.value = false;
  }
});
</script>

<style scoped>
.search-bar-container {
  position: relative;
  width: 100%;
  max-width: 600px;
}
</style>
