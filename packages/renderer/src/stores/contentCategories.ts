import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { useStaticStatus } from '../composables/staticStatus';

// Define a placeholder or empty default for ContentCategoryItem if needed, or adjust usage
export interface ContentCategoryMetadataField { 
  // Define structure based on what `DEFAULT_CONTENT_CATEGORIES` used to provide, or keep minimal
  name: string;
  type: string; // e.g., 'text', 'number', 'boolean'
  required?: boolean;
  // Add other relevant fields if known
}

export interface ContentCategory {
  name: string;
  metadataSchema: ContentCategoryMetadataField[];
  featured?: boolean;
  // Add other relevant fields if known
}

export interface ContentCategoryItem {
  id: string;
  contentCategory: ContentCategory;
}

// Placeholder for default categories if the static fallback is to be maintained minimally
const DEFAULT_CONTENT_CATEGORIES: ContentCategory[] = [
  // Example structure, adjust as needed or leave empty if not critical for build
  // { name: 'Default Category 1', metadataSchema: [{name: 'title', type: 'text'}], featured: true }, 
];

export const useContentCategoriesStore = defineStore('contentCategories', () => {
  const { staticStatus } = useStaticStatus();

  const contentCategories = computed<ContentCategoryItem[]>(() => {
    // Always return static/empty or a minimal default to avoid Orbiter dependency
    // This ensures the build passes. Actual data fetching needs to be reimplemented with Peerbit.
    if (staticStatus.value === 'static') {
        return DEFAULT_CONTENT_CATEGORIES.map((dcc, i) => ({
            id: i.toString(),
            contentCategory: dcc,
        }));
    }
    return []; // Return empty array if not in static mode or as a general placeholder

  });

  const featuredContentCategories = computed(() => {
    return contentCategories.value.filter(cc => cc.contentCategory.featured);
  });

  return {
    contentCategories,
    featuredContentCategories,
  };
});
