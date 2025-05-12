<template>
  <v-container class="fill-height pb-16">
    <v-sheet
      v-if="isLoading"
      color="transparent"
      class="d-flex w-100 fill-height align-center justify-center"
    >
      <v-progress-circular
        indeterminate
        color="primary"
      ></v-progress-circular>
    </v-sheet>
    <template v-else-if="filteredReleases.length > 0">
      <content-section
        :title="pageCategory?.contentCategory.displayName ?? ''"
      >
        <v-col
          v-for="item in filteredReleases"
          :key="item.id"
        >
          <content-card
            :item="item"
            cursor-pointer
            :source-site="item.sourceSite"
            @click="router.push(`/release/${item.id}`)"
          />
        </v-col>
      </content-section>
    </template>
    <v-sheet
      v-else
      color="transparent"
      class="d-flex flex-column mx-auto"
      max-width="16rem"
    >
      <p class="text-white text-center mb-2">No content found in this category.</p>
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useReleasesStore } from '/@/stores/releases';
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import contentSection from '/@/components/home/contentSection.vue';
import contentCard from '/@/components/misc/contentCard.vue';
import { useContentCategoriesStore } from '/@/stores/contentCategories';

const props = defineProps<{
  category: string
}>();
const router = useRouter();
const releasesStore = useReleasesStore();
const contentCategoriesStore = useContentCategoriesStore();

const { releases, isLoading } = storeToRefs(releasesStore);
const { contentCategories } = storeToRefs(contentCategoriesStore);

const filteredReleases = computed(() => {
  return releases.value.filter((release) => {
    return release.category === props.category;
  });
});

const pageCategory = computed(() => {
  const categoryId = props.category;
  const category = contentCategories.value.find((cat) => cat.contentCategory.categoryId === categoryId);
  return category;
});

</script>
