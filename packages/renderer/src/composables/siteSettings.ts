import { type Ref, ref } from 'vue';


export interface ContentCategory {
  id: string;
  featured?: boolean;
}

// Define categories more explicitly
const contentCategories: Ref<ContentCategory[]> = ref([
  { id: 'music', featured: true },
  { id: 'movie', featured: true }, // Keep movie
  { id: 'tvShow', featured: true }, // Keep tvShow
  { id: 'video', featured: true }, // Keep generic video for now
  { id: 'audiobook' },
  { id: 'game' },
  { id: 'book'},
  { id: 'video'},
  { id: 'other'},
]);

export const useSiteSettings = () => {
  return {
    contentCategories,
  };
};
