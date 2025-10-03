import MiniSearch from 'minisearch';
import { ref, computed } from 'vue';

export interface SearchableContent {
  id: string;
  title: string;
  artist?: string;
  description?: string;
  category: string;
  tags?: string[];
  year?: number;
  type: 'music' | 'movie' | 'tv' | 'other';
}

export interface SearchFilters {
  category?: string;
  type?: 'music' | 'movie' | 'tv' | 'other';
  year?: number;
}

export interface SearchResult extends SearchableContent {
  score: number;
  match: Record<string, string[]>;
}

let searchIndex: MiniSearch<SearchableContent> | null = null;
const isIndexReady = ref(false);
const indexedCount = ref(0);

export function useLocalSearch() {
  // Initialize index if not already created
  if (!searchIndex) {
    searchIndex = new MiniSearch({
      fields: ['title', 'artist', 'description', 'tags'],
      storeFields: ['id', 'title', 'artist', 'category', 'type', 'year', 'tags'],
      searchOptions: {
        boost: { title: 3, artist: 2, tags: 1.5 },
        fuzzy: 0.2,
        prefix: true,
        combineWith: 'AND'
      }
    });
  }

  /**
   * Index content for searching
   * @param content Array of searchable content items
   */
  function indexContent(content: SearchableContent[]) {
    if (!searchIndex) return;

    try {
      // Remove all existing documents
      searchIndex.removeAll();

      // Add new content
      searchIndex.addAll(content);

      indexedCount.value = content.length;
      isIndexReady.value = true;

      console.log(`[LocalSearch] Indexed ${content.length} items`);
    } catch (error) {
      console.error('[LocalSearch] Error indexing content:', error);
      isIndexReady.value = false;
    }
  }

  /**
   * Add single item to index
   * @param item Searchable content item
   */
  function indexItem(item: SearchableContent) {
    if (!searchIndex) return;

    try {
      searchIndex.add(item);
      indexedCount.value++;
      isIndexReady.value = true;
    } catch (error) {
      console.error('[LocalSearch] Error indexing item:', error);
    }
  }

  /**
   * Update existing item in index
   * @param item Updated searchable content item
   */
  function updateItem(item: SearchableContent) {
    if (!searchIndex) return;

    try {
      searchIndex.replace(item);
    } catch (error) {
      console.error('[LocalSearch] Error updating item:', error);
    }
  }

  /**
   * Remove item from index
   * @param id Item ID to remove
   */
  function removeItem(id: string) {
    if (!searchIndex) return;

    try {
      searchIndex.discard(id);
      indexedCount.value--;
    } catch (error) {
      console.error('[LocalSearch] Error removing item:', error);
    }
  }

  /**
   * Search the index
   * @param query Search query string
   * @param filters Optional filters to apply
   * @returns Array of search results
   */
  function search(query: string, filters?: SearchFilters): SearchResult[] {
    if (!searchIndex || !isIndexReady.value) {
      return [];
    }

    if (!query || query.trim().length < 2) {
      return [];
    }

    try {
      const results = searchIndex.search(query, {
        filter: (result) => {
          // Apply category filter
          if (filters?.category && result.category !== filters.category) {
            return false;
          }

          // Apply type filter
          if (filters?.type && result.type !== filters.type) {
            return false;
          }

          // Apply year filter
          if (filters?.year && result.year !== filters.year) {
            return false;
          }

          return true;
        }
      });

      return results.slice(0, 50) as unknown as SearchResult[]; // Limit to top 50 results
    } catch (error) {
      console.error('[LocalSearch] Error searching:', error);
      return [];
    }
  }

  /**
   * Get search suggestions based on partial query
   * @param query Partial search query
   * @returns Array of suggested terms
   */
  function getSuggestions(query: string): string[] {
    if (!searchIndex || !isIndexReady.value || !query || query.length < 2) {
      return [];
    }

    try {
      const results = searchIndex.autoSuggest(query, {
        boost: { title: 3, artist: 2 },
        fuzzy: 0.2
      });

      return results.map(result => result.suggestion).slice(0, 10);
    } catch (error) {
      console.error('[LocalSearch] Error getting suggestions:', error);
      return [];
    }
  }

  /**
   * Clear the entire search index
   */
  function clearIndex() {
    if (!searchIndex) return;

    searchIndex.removeAll();
    indexedCount.value = 0;
    isIndexReady.value = false;
  }

  return {
    // State
    isIndexReady: computed(() => isIndexReady.value),
    indexedCount: computed(() => indexedCount.value),

    // Methods
    indexContent,
    indexItem,
    updateItem,
    removeItem,
    search,
    getSuggestions,
    clearIndex
  };
}
