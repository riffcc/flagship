
import { defineStore } from 'pinia';
import { useOrbiter } from '../plugins/orbiter/utils';
import { computed, ref } from 'vue';
import { useStaticStatus } from '../composables/staticStatus';
import type { types as orbiterTypes } from '@riffcc/orbiter';
import { consts } from '@riffcc/orbiter';

export type ContentCategoryItem = orbiterTypes.ContentCategoryWithId<orbiterTypes.ContentCategoryMetadataField>;
export const useContentCategoriesStore = defineStore('contentCategories', () => {
  const { orbiter } = useOrbiter();
  const { staticStatus } = useStaticStatus();
  const orbiterContentCategories = ref<orbiterTypes.ContentCategoryWithId<string>[]>([]);

  if (orbiter && orbiter.listenForContentCategories) {
    orbiter.listenForContentCategories({
      f: (categories: orbiterTypes.ContentCategoryWithId<string>[]) => {
        orbiterContentCategories.value = categories;
      },
    });
  }

  const contentCategories = computed<ContentCategoryItem[]>(() => {
    if (staticStatus.value === 'static' || !((orbiterContentCategories.value?.length || 0) > 0)) return consts.DEFAULT_CONTENT_CATEGORIES.map((dcc, i) => ({
      id: i.toString(),
      contentCategory: dcc,
    }));
    else {
      return (orbiterContentCategories.value || []).map(cc => ({
        id: cc.id,
        contentCategory: {
          ...cc.contentCategory,
          metadataSchema: JSON.parse(cc.contentCategory.metadataSchema),
        },
      })) as ContentCategoryItem[];
    }
  });
  const featuredContentCategories = computed(() => {
    return contentCategories.value.filter(cc => cc.contentCategory.featured);
  });

  return {
    contentCategories,
    featuredContentCategories,
  };
});
