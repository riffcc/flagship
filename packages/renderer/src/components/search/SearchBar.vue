<template>
  <div class="search-bar-container">
    <v-text-field
      v-model="query"
      label="Search music, movies, TV shows..."
      prepend-inner-icon="$magnify"
      variant="outlined"
      density="comfortable"
      hide-details
      @update:model-value="onSearch"
      @keydown="onKeydown"
      @focus="showResults = true"
      @blur="onBlur"
    >
      <template #append-inner>
        <v-btn
          v-if="query"
          icon="$close"
          size="x-small"
          variant="text"
          @click="clearQuery"
        />
        <v-chip v-if="isIndexReady && contentCount > 0" size="x-small" variant="text">
          {{ contentCount }} items
        </v-chip>
      </template>
    </v-text-field>

    <!-- Search Results Dropdown -->
    <SearchResults
      v-if="showResults && results.length > 0"
      :results="results"
      :query="query"
      :focused-index="focusedIndex"
      @select="onSelectResult"
      @close="showResults = false"
    />

    <!-- No Results Message -->
    <v-card
      v-else-if="showResults && query.length >= 2 && results.length === 0"
      class="search-no-results"
      elevation="8"
    >
      <v-card-text class="text-center text-grey">
        No results for "{{ query }}"
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useDebounceFn } from '@vueuse/core';
import { useRouter } from 'vue-router';
import { useLocalSearch } from '/@/composables/useLocalSearch';
import SearchResults from './SearchResults.vue';

const router = useRouter();
const { search, isIndexReady, contentCount } = useLocalSearch();

const query = ref('');
const results = ref<any[]>([]);
const showResults = ref(false);
const focusedIndex = ref(-1);

// Debounced search function (150ms like QuickSearch)
const onSearch = useDebounceFn((value: string) => {
  if (!value || value.length < 2) {
    results.value = [];
    showResults.value = false;
    return;
  }

  const searchResults = search(value);
  console.log('[SearchBar] Search for:', value, '-> results:', searchResults.length);
  results.value = searchResults;
  showResults.value = true;
}, 150);

// Handle keyboard navigation
function onKeydown(event: KeyboardEvent) {
  const total = results.value.length;

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      if (total > 0) {
        focusedIndex.value = Math.min(total - 1, focusedIndex.value + 1);
      }
      break;
    case 'ArrowUp':
      event.preventDefault();
      if (total > 0) {
        focusedIndex.value = Math.max(-1, focusedIndex.value - 1);
      }
      break;
    case 'Enter':
      event.preventDefault();
      if (focusedIndex.value >= 0 && results.value[focusedIndex.value]) {
        onSelectResult(results.value[focusedIndex.value]);
      } else if (results.value.length > 0) {
        onSelectResult(results.value[0]);
      }
      break;
    case 'Escape':
      event.preventDefault();
      showResults.value = false;
      focusedIndex.value = -1;
      break;
  }
}

// Clear search query
function clearQuery() {
  query.value = '';
  results.value = [];
  showResults.value = false;
  focusedIndex.value = -1;
}

// Handle result selection
function onSelectResult(result: any) {
  showResults.value = false;
  query.value = '';

  const route = getRouteForContent(result);
  if (route) {
    router.push(route);
  }
}

// Get route for content item
function getRouteForContent(content: any): string | null {
  if (!content.id) return null;

  switch (content.type) {
    case 'artist':
      return `/artist/${content.id}`;
    case 'music':
    case 'movie':
    case 'tv':
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

.search-no-results {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 8px;
  z-index: 9999;
}

</style>
