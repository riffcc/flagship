# Phase 1: LOCAL Search MVP

## Core Principle
**ZERO SERVER-SIDE SEARCH** - Everything happens in the browser using P2P data.

## Architecture

```
┌──────────────────────────────────────────┐
│         Browser (Flagship)               │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  Search UI Component               │ │
│  │  (search bar, results display)     │ │
│  └────────────┬───────────────────────┘ │
│               │ user query              │
│               ▼                         │
│  ┌────────────────────────────────────┐ │
│  │  Client-Side Search Engine         │ │
│  │  - MiniSearch/Lunr.js index        │ │
│  │  - Fuzzy matching                  │ │
│  │  - Field weights (title > desc)    │ │
│  │  - Result ranking                  │ │
│  └────────────┬───────────────────────┘ │
│               │ search results          │
│               ▼                         │
│  ┌────────────────────────────────────┐ │
│  │  Local Index (IndexedDB)           │ │
│  │  - All catalog metadata            │ │
│  │  - Synced from lens-node           │ │
│  │  - Offline-capable                 │ │
│  └────────────────────────────────────┘ │
│               ▲                         │
└───────────────┼─────────────────────────┘
                │ P2P sync
                │
         ┌──────┴──────┐
         │  Lens Node  │
         │  (Local P2P)│
         └─────────────┘
```

## Implementation Steps

### 1. Search Library Selection
**MiniSearch** - lightweight, fast, full-text search
- 6.1KB gzipped
- Fuzzy search out of the box
- Field boosting
- Auto-suggestions
- Zero dependencies

### 2. Index Fields
```typescript
interface SearchableContent {
  id: string;
  title: string;         // Boost: 3x
  artist?: string;       // Boost: 2x
  description?: string;  // Boost: 1x
  category: string;      // Exact match
  tags?: string[];       // Boost: 1.5x
  year?: number;
  type: 'music' | 'movie' | 'tv' | 'other';
}
```

### 3. Search Features (Phase 1 MVP)
- [x] **Instant search** - Results as you type
- [x] **Fuzzy matching** - Handles typos
- [x] **Category filtering** - Music, Movies, TV Shows
- [x] **Offline support** - Works without network
- [ ] Auto-suggestions (Phase 2)
- [ ] Recent searches (Phase 2)
- [ ] Advanced filters (Phase 3)

### 4. UI Components
```
packages/renderer/src/components/search/
├── SearchBar.vue          # Main search input
├── SearchResults.vue      # Results display
├── SearchFilters.vue      # Category/type filters
└── useSearch.ts           # Composable with search logic
```

### 5. Data Flow
1. **Index Build**: When lens-node data loads, build MiniSearch index
2. **User Types**: Debounced search (300ms) against local index
3. **Results**: Display top 20 results with highlight
4. **Click**: Navigate to content page (existing routes)

## Technical Implementation

### Step 1: Install MiniSearch
```bash
cd packages/renderer
pnpm add minisearch
```

### Step 2: Create Search Composable
```typescript
// src/composables/useLocalSearch.ts
import MiniSearch from 'minisearch';
import { ref, computed } from 'vue';

export function useLocalSearch() {
  const searchIndex = new MiniSearch({
    fields: ['title', 'artist', 'description', 'tags'],
    storeFields: ['id', 'title', 'artist', 'category', 'type'],
    searchOptions: {
      boost: { title: 3, artist: 2, tags: 1.5 },
      fuzzy: 0.2,
      prefix: true
    }
  });

  function indexContent(content: SearchableContent[]) {
    searchIndex.addAll(content);
  }

  function search(query: string, filters?: SearchFilters) {
    return searchIndex.search(query, {
      filter: (result) => {
        if (filters?.category && result.category !== filters.category) {
          return false;
        }
        return true;
      }
    });
  }

  return { indexContent, search };
}
```

### Step 3: SearchBar Component
```vue
<template>
  <v-text-field
    v-model="query"
    label="Search music, movies, TV shows..."
    prepend-inner-icon="mdi-magnify"
    clearable
    @update:model-value="onSearch"
  />
  <SearchResults v-if="results.length" :results="results" />
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useDebounceFn } from '@vueuse/core';
import { useLocalSearch } from '/@/composables/useLocalSearch';

const { search } = useLocalSearch();
const query = ref('');
const results = ref([]);

const onSearch = useDebounceFn((value: string) => {
  if (!value || value.length < 2) {
    results.value = [];
    return;
  }
  results.value = search(value).slice(0, 20);
}, 300);
</script>
```

## Performance Targets
- **Index build**: < 500ms for 10,000 items
- **Search latency**: < 50ms for typical queries
- **Memory usage**: < 10MB for index
- **Offline**: 100% functional without network

## Success Criteria
✅ User can search entire catalog from browser
✅ Results appear instantly (< 100ms perceived)
✅ Works offline from cached data
✅ No server-side dependencies
✅ Fuzzy search handles typos

## Future Enhancements (Phase 2+)
- Voice search
- Natural language queries ("80s rock music")
- Semantic search (ML embeddings)
- Collaborative filtering ("Users who liked X also searched for Y")
- Search analytics (local only, privacy-preserving)
