import { type Ref, ref } from 'vue';


export interface ContentCategory {
  id: string;
  featured?: boolean;
}

const contentCategories: Ref<ContentCategory[]> = ref([
  { id: 'music', featured: true},
  { id: 'movie', featured: true},
  { id: 'tvShow', featured: true},
  { id: 'audiobook'},
  { id: 'game'},
  { id: 'book'},
  { id: 'video'},
  { id: 'other'},
]);

export const useSiteSettings = () => {
  return {
    contentCategories,
  };
};
