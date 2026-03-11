import { describe, it, expect, beforeEach } from 'vitest';
import { useLocalSearch, type SearchableContent } from '../../packages/renderer/src/composables/useLocalSearch';

describe('useLocalSearch', () => {
  const mockContent: SearchableContent[] = [
    {
      id: '1',
      title: 'Abbey Road',
      artist: 'The Beatles',
      description: 'Classic album by The Beatles',
      category: 'music',
      tags: ['rock', 'classic'],
      year: 1969,
      type: 'music',
    },
    {
      id: '2',
      title: 'The Dark Side of the Moon',
      artist: 'Pink Floyd',
      description: 'Progressive rock masterpiece',
      category: 'music',
      tags: ['rock', 'progressive'],
      year: 1973,
      type: 'music',
    },
    {
      id: '3',
      title: 'Thriller',
      artist: 'Michael Jackson',
      description: 'Pop music phenomenon',
      category: 'music',
      tags: ['pop', 'classic'],
      year: 1982,
      type: 'music',
    },
    {
      id: '4',
      title: 'Inception',
      description: 'Mind-bending sci-fi movie',
      category: 'movies',
      tags: ['sci-fi', 'thriller'],
      year: 2010,
      type: 'movie',
    },
  ];

  let search: ReturnType<typeof useLocalSearch>;

  beforeEach(() => {
    search = useLocalSearch();
    search.clearIndex();
  });

  describe('indexContent', () => {
    it('should index multiple items successfully', () => {
      search.indexContent(mockContent);
      expect(search.isIndexReady.value).toBe(true);
      expect(search.indexedCount.value).toBe(4);
    });

    it('should replace existing index when indexing new content', () => {
      search.indexContent([mockContent[0]]);
      expect(search.indexedCount.value).toBe(1);

      search.indexContent(mockContent);
      expect(search.indexedCount.value).toBe(4);
    });

    it('should handle empty array', () => {
      search.indexContent([]);
      expect(search.isIndexReady.value).toBe(true);
      expect(search.indexedCount.value).toBe(0);
    });
  });

  describe('search', () => {
    beforeEach(() => {
      search.indexContent(mockContent);
    });

    it('should find exact title matches', () => {
      const results = search.search('Abbey Road');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].title).toBe('Abbey Road');
    });

    it('should find partial matches', () => {
      const results = search.search('Abbey');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].title).toBe('Abbey Road');
    });

    it('should find artist matches', () => {
      const results = search.search('Beatles');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].artist).toBe('The Beatles');
    });

    it('should return empty array for queries less than 2 characters', () => {
      const results = search.search('a');
      expect(results).toEqual([]);
    });

    it('should return empty array for empty query', () => {
      const results = search.search('');
      expect(results).toEqual([]);
    });

    it('should handle queries with no matches', () => {
      const results = search.search('nonexistent query xyz');
      expect(results).toEqual([]);
    });

    it('should support fuzzy matching', () => {
      // "Thriler" (one letter off) should match "Thriller" with fuzzy search enabled
      const results = search.search('Thriler');
      expect(results.length).toBeGreaterThan(0);
    });

    it('should filter by category', () => {
      const results = search.search('music', { category: 'music' });
      expect(results.length).toBeGreaterThan(0);
      results.forEach(result => {
        expect(result.category).toBe('music');
      });
    });

    it('should filter by type', () => {
      const results = search.search('Inception', { type: 'movie' });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].type).toBe('movie');
    });

    it('should filter by year', () => {
      const results = search.search('Abbey', { year: 1969 });
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].year).toBe(1969);
    });

    it('should limit results to 50', () => {
      // Create a large dataset
      const largeDataset: SearchableContent[] = Array.from({ length: 100 }, (_, i) => ({
        id: `item-${i}`,
        title: `Test Item ${i}`,
        category: 'music',
        type: 'music' as const,
      }));

      search.indexContent(largeDataset);
      const results = search.search('Test');
      expect(results.length).toBeLessThanOrEqual(50);
    });
  });

  describe('getSuggestions', () => {
    beforeEach(() => {
      search.indexContent(mockContent);
    });

    it('should return suggestions for partial queries', () => {
      const suggestions = search.getSuggestions('Abb');
      expect(suggestions.length).toBeGreaterThan(0);
    });

    it('should return empty array for queries less than 2 characters', () => {
      const suggestions = search.getSuggestions('a');
      expect(suggestions).toEqual([]);
    });

    it('should return empty array for empty query', () => {
      const suggestions = search.getSuggestions('');
      expect(suggestions).toEqual([]);
    });

    it('should limit suggestions to 10', () => {
      const largeDataset: SearchableContent[] = Array.from({ length: 20 }, (_, i) => ({
        id: `item-${i}`,
        title: `Test Suggestion ${i}`,
        category: 'music',
        type: 'music' as const,
      }));

      search.indexContent(largeDataset);
      const suggestions = search.getSuggestions('Test');
      expect(suggestions.length).toBeLessThanOrEqual(10);
    });
  });

  describe('indexItem', () => {
    beforeEach(() => {
      search.indexContent(mockContent);
    });

    it('should add a single item to the index', () => {
      const initialCount = search.indexedCount.value;

      const newItem: SearchableContent = {
        id: '5',
        title: 'New Album',
        artist: 'New Artist',
        category: 'music',
        type: 'music',
      };

      search.indexItem(newItem);
      expect(search.indexedCount.value).toBe(initialCount + 1);

      const results = search.search('New Album');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].title).toBe('New Album');
    });
  });

  describe('updateItem', () => {
    beforeEach(() => {
      search.indexContent(mockContent);
    });

    it('should update an existing item', () => {
      const updatedItem: SearchableContent = {
        ...mockContent[0],
        title: 'Updated Title',
      };

      search.updateItem(updatedItem);

      const results = search.search('Updated Title');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].title).toBe('Updated Title');
    });
  });

  describe('removeItem', () => {
    beforeEach(() => {
      search.indexContent(mockContent);
    });

    it('should remove an item from the index', () => {
      const initialCount = search.indexedCount.value;

      search.removeItem('1');
      expect(search.indexedCount.value).toBe(initialCount - 1);

      const results = search.search('Abbey Road');
      expect(results.length).toBe(0);
    });
  });

  describe('clearIndex', () => {
    beforeEach(() => {
      search.indexContent(mockContent);
    });

    it('should clear the entire index', () => {
      search.clearIndex();
      expect(search.isIndexReady.value).toBe(false);
      expect(search.indexedCount.value).toBe(0);

      const results = search.search('Beatles');
      expect(results).toEqual([]);
    });
  });

  describe('field boosting', () => {
    beforeEach(() => {
      search.indexContent(mockContent);
    });

    it('should prioritize title matches over other fields', () => {
      const results = search.search('Dark');
      expect(results.length).toBeGreaterThan(0);
      // Title match should be first due to boosting
      expect(results[0].title).toContain('Dark');
    });
  });

  describe('tag searching', () => {
    beforeEach(() => {
      search.indexContent(mockContent);
    });

    it('should find items by tags', () => {
      const results = search.search('progressive');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].tags).toContain('progressive');
    });

    it('should find multiple items with the same tag', () => {
      const results = search.search('classic');
      expect(results.length).toBeGreaterThanOrEqual(2);
    });
  });
});
