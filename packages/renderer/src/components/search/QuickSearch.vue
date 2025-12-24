<template>
  <Teleport to="body">
    <Transition name="quick-search">
      <div
        v-if="isOpen"
        class="quick-search-overlay"
        @click.self="close"
      >
        <div class="quick-search-container">
          <!-- Search Input -->
          <div class="quick-search-input-wrapper">
            <v-icon
              icon="mdi-magnify"
              size="32"
              class="quick-search-icon"
            />
            <input
              ref="searchInput"
              v-model="query"
              type="text"
              class="quick-search-input"
              placeholder="Search..."
              autofocus
              @keydown="handleInputKeydown"
            />
            <span class="quick-search-hint">
              ESC to close
            </span>
            <v-btn
              icon="mdi-close"
              variant="text"
              size="small"
              class="quick-search-close"
              @click="close"
            />
          </div>

          <!-- Results Grid -->
          <div
            v-if="results.length > 0"
            class="quick-search-results"
          >
            <div class="quick-search-grid">
              <div
                v-for="(result, index) in results"
                :key="result.id"
                class="quick-search-result-card"
                :class="{ focused: focusedIndex === index }"
                data-navigable="true"
                @click="selectResult(result)"
                @mouseenter="focusedIndex = index"
              >
                <v-img
                  :src="getThumbnailUrl(result.thumbnailCID)"
                  aspect-ratio="1"
                  cover
                  class="quick-search-result-image"
                >
                  <template #placeholder>
                    <div class="d-flex align-center justify-center fill-height">
                      <v-icon
                        :icon="getTypeIcon(result.type)"
                        size="48"
                        color="grey"
                      />
                    </div>
                  </template>
                </v-img>
                <div class="quick-search-result-info">
                  <p class="quick-search-result-title">
                    {{ result.title }}
                  </p>
                  <p
                    v-if="result.artist"
                    class="quick-search-result-subtitle"
                  >
                    {{ result.artist }}
                  </p>
                  <v-chip
                    size="x-small"
                    :color="getTypeColor(result.type)"
                    class="mt-1"
                  >
                    {{ result.type }}
                  </v-chip>
                </div>
              </div>
            </div>
          </div>

          <!-- No Results -->
          <div
            v-else-if="query.length >= 2"
            class="quick-search-no-results"
          >
            <v-icon
              icon="mdi-magnify-close"
              size="48"
              color="grey"
            />
            <p>No results found</p>
          </div>

          <!-- Instructions -->
          <div
            v-else
            class="quick-search-instructions"
          >
            <p>Start typing to search</p>
            <div class="quick-search-shortcuts">
              <span><kbd>Arrow keys</kbd> Navigate</span>
              <span><kbd>Enter</kbd> Select</span>
              <span><kbd>Backspace</kbd> Clear/Close</span>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useQuickSearch } from '/@/composables/useQuickSearch';
import { useLocalSearch, type SearchResult } from '/@/composables/useLocalSearch';
import { parseUrlOrCid } from '/@/utils';

const router = useRouter();
const { isOpen, query, close } = useQuickSearch();
const { search } = useLocalSearch();

const searchInput = ref<HTMLInputElement | null>(null);
const focusedIndex = ref(0);

// Debounced search results
const results = ref<SearchResult[]>([]);
let searchTimeout: number | undefined;

watch(query, newQuery => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }

  if (newQuery.length < 2) {
    results.value = [];
    focusedIndex.value = 0;
    return;
  }

  searchTimeout = setTimeout(() => {
    results.value = search(newQuery);
    focusedIndex.value = 0;
  }, 150) as unknown as number;
});

// Focus input when opened
watch(isOpen, async newIsOpen => {
  if (newIsOpen) {
    await nextTick();
    searchInput.value?.focus();
    focusedIndex.value = 0;
  }
});

// Calculate grid columns for navigation
const gridColumns = computed(() => {
  // Matches CSS: minmax(150px, 1fr)
  // Estimate based on typical viewport
  if (typeof window === 'undefined') return 4;
  const containerWidth = Math.min(800, window.innerWidth - 64);
  return Math.floor(containerWidth / 170);
});

function handleInputKeydown(event: KeyboardEvent) {
  const cols = gridColumns.value;
  const total = results.value.length;

  switch (event.key) {
    case 'Escape':
      event.preventDefault();
      close();
      break;

    case 'Backspace':
      if (query.value.length === 0) {
        event.preventDefault();
        close();
      }
      break;

    case 'ArrowUp':
      event.preventDefault();
      if (total > 0) {
        focusedIndex.value = Math.max(0, focusedIndex.value - cols);
      }
      break;

    case 'ArrowDown':
      event.preventDefault();
      if (total > 0) {
        focusedIndex.value = Math.min(total - 1, focusedIndex.value + cols);
      }
      break;

    case 'ArrowLeft':
      event.preventDefault();
      if (total > 0) {
        focusedIndex.value = Math.max(0, focusedIndex.value - 1);
      }
      break;

    case 'ArrowRight':
      event.preventDefault();
      if (total > 0) {
        focusedIndex.value = Math.min(total - 1, focusedIndex.value + 1);
      }
      break;

    case 'Enter':
      event.preventDefault();
      if (results.value.length > 0 && focusedIndex.value >= 0) {
        selectResult(results.value[focusedIndex.value]);
      }
      break;
  }
}

function selectResult(result: SearchResult) {
  close();

  // Navigate based on type
  if (result.type === 'artist') {
    router.push(`/artist/${result.id}`);
  } else {
    router.push(`/release/${result.id}`);
  }
}

function getThumbnailUrl(cid?: string): string {
  return parseUrlOrCid(cid) ?? '/no-image-icon.png';
}

function getTypeIcon(type: string): string {
  switch (type) {
    case 'music':
      return 'mdi-music';
    case 'movie':
      return 'mdi-movie';
    case 'tv':
      return 'mdi-television';
    case 'artist':
      return 'mdi-account-music';
    default:
      return 'mdi-file';
  }
}

function getTypeColor(type: string): string {
  switch (type) {
    case 'music':
      return 'purple';
    case 'movie':
      return 'blue';
    case 'tv':
      return 'green';
    case 'artist':
      return 'pink';
    default:
      return 'grey';
  }
}
</script>

<style scoped>
.quick-search-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(10px);
  z-index: 9999;
  display: flex;
  justify-content: center;
  padding-top: 10vh;
}

.quick-search-container {
  width: 100%;
  max-width: 800px;
  padding: 0 16px;
}

.quick-search-input-wrapper {
  display: flex;
  align-items: center;
  gap: 16px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(138, 43, 226, 0.5);
  border-radius: 12px;
  padding: 16px 24px;
}

.quick-search-icon {
  color: rgba(138, 43, 226, 0.8);
}

.quick-search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: white;
  font-size: 24px;
  font-weight: 300;
}

.quick-search-input::placeholder {
  color: rgba(255, 255, 255, 0.5);
}

.quick-search-hint {
  color: rgba(255, 255, 255, 0.4);
  font-size: 12px;
}

.quick-search-close {
  color: rgba(255, 255, 255, 0.5);
  margin-left: 8px;
}

.quick-search-close:hover {
  color: rgba(255, 255, 255, 0.9);
}

.quick-search-results {
  margin-top: 24px;
  max-height: 60vh;
  overflow-y: auto;
}

.quick-search-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 16px;
}

.quick-search-result-card {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s ease;
}

.quick-search-result-card:hover,
.quick-search-result-card.focused {
  background: rgba(138, 43, 226, 0.2);
  transform: scale(1.02);
  box-shadow: 0 0 0 2px rgba(138, 43, 226, 0.8), 0 0 20px rgba(138, 43, 226, 0.4);
}

.quick-search-result-image {
  aspect-ratio: 1;
}

.quick-search-result-info {
  padding: 8px;
}

.quick-search-result-title {
  font-size: 14px;
  font-weight: 500;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.quick-search-result-subtitle {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.quick-search-no-results,
.quick-search-instructions {
  text-align: center;
  color: rgba(255, 255, 255, 0.5);
  padding: 48px 0;
}

.quick-search-shortcuts {
  margin-top: 16px;
  display: flex;
  gap: 24px;
  justify-content: center;
  flex-wrap: wrap;
}

.quick-search-shortcuts span {
  font-size: 12px;
}

.quick-search-shortcuts kbd {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  padding: 2px 6px;
  margin-right: 4px;
}

/* Transition */
.quick-search-enter-active,
.quick-search-leave-active {
  transition: opacity 0.2s ease;
}

.quick-search-enter-from,
.quick-search-leave-to {
  opacity: 0;
}

/* Scrollbar styling */
.quick-search-results::-webkit-scrollbar {
  width: 8px;
}

.quick-search-results::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
}

.quick-search-results::-webkit-scrollbar-thumb {
  background: rgba(138, 43, 226, 0.5);
  border-radius: 4px;
}

.quick-search-results::-webkit-scrollbar-thumb:hover {
  background: rgba(138, 43, 226, 0.7);
}
</style>
