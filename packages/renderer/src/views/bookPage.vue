<template>
  <v-container fluid class="pb-16">
    <!-- Loading state -->
    <v-sheet
      v-if="isLoading"
      color="transparent"
      class="d-flex w-100 fill-height align-center justify-center"
    >
      <v-progress-circular
        indeterminate
        color="primary"
        size="64"
      ></v-progress-circular>
    </v-sheet>

    <!-- Book not found -->
    <v-sheet
      v-else-if="!book && !isLoading"
      color="transparent"
      class="d-flex flex-column mx-auto mt-8"
      max-width="20rem"
    >
      <v-icon
        size="64"
        class="mb-4 text-center"
        color="grey"
      >$book-open-page-variant</v-icon>
      <p class="text-h6 text-center mb-2">Book not found</p>
      <p class="text-body-2 text-center text-grey mb-4">
        The book you're looking for doesn't exist or has been removed.
      </p>
      <v-btn
        color="primary"
        @click="router.push('/books')"
      >
        Browse Books
      </v-btn>
    </v-sheet>

    <!-- Book content -->
    <template v-else-if="book">
      <!-- Book header -->
      <v-row class="mb-6">
        <v-col cols="12" md="4" lg="3">
          <v-img
            :src="parseUrlOrCid(book.thumbnailCID)"
            aspect-ratio="0.67"
            cover
            rounded="lg"
            class="elevation-8"
          >
            <template #placeholder>
              <v-sheet
                color="grey-darken-3"
                class="d-flex align-center justify-center fill-height"
              >
                <v-icon size="64" color="grey">$book-open-page-variant</v-icon>
              </v-sheet>
            </template>
          </v-img>
        </v-col>

        <v-col cols="12" md="8" lg="9">
          <p class="text-overline text-grey mb-1">Book</p>
          <h1 class="text-h3 text-sm-h2 font-weight-bold mb-3">{{ book.name }}</h1>

          <div class="d-flex align-center ga-3 mb-4">
            <p v-if="book.metadata?.author" class="text-h6 mb-0">
              by {{ book.metadata.author }}
            </p>
          </div>

          <div class="d-flex align-center ga-3 mb-4 flex-wrap">
            <v-chip v-if="book.metadata?.genre" size="small" color="primary">
              {{ book.metadata.genre }}
            </v-chip>
            <v-chip v-if="book.metadata?.publishedYear" size="small" color="secondary">
              {{ book.metadata.publishedYear }}
            </v-chip>
            <v-chip v-if="book.metadata?.pages" size="small" color="accent">
              {{ book.metadata.pages }} pages
            </v-chip>
            <v-chip v-if="book.metadata?.language" size="small">
              {{ book.metadata.language }}
            </v-chip>
          </div>

          <p v-if="book.metadata?.description" class="text-body-1 mb-4">
            {{ book.metadata.description }}
          </p>

          <v-btn
            color="primary"
            size="large"
            prepend-icon="$book-open-variant"
            class="mt-2"
            @click="router.push(`/read/${book.id}`)"
          >
            Read Book
          </v-btn>
        </v-col>
      </v-row>

      <v-divider class="my-6" />

      <!-- Publication details -->
      <div v-if="book.metadata?.publisher || book.metadata?.isbn">
        <h2 class="text-h5 font-weight-bold mb-4">Publication Details</h2>

        <v-list bg-color="transparent">
          <v-list-item v-if="book.metadata?.publisher">
            <v-list-item-title class="text-body-2 text-grey">Publisher</v-list-item-title>
            <v-list-item-subtitle class="text-body-1">{{ book.metadata.publisher }}</v-list-item-subtitle>
          </v-list-item>

          <v-list-item v-if="book.metadata?.isbn">
            <v-list-item-title class="text-body-2 text-grey">ISBN</v-list-item-title>
            <v-list-item-subtitle class="text-body-1">{{ book.metadata.isbn }}</v-list-item-subtitle>
          </v-list-item>

          <v-list-item v-if="book.metadata?.edition">
            <v-list-item-title class="text-body-2 text-grey">Edition</v-list-item-title>
            <v-list-item-subtitle class="text-body-1">{{ book.metadata.edition }}</v-list-item-subtitle>
          </v-list-item>
        </v-list>

        <v-divider class="my-6" />
      </div>

      <!-- Table of Contents -->
      <div v-if="tableOfContents.length > 0">
        <h2 class="text-h5 font-weight-bold mb-4">Table of Contents</h2>

        <v-list bg-color="transparent">
          <v-list-item
            v-for="(chapter, index) in tableOfContents"
            :key="index"
            class="chapter-item px-2 rounded"
          >
            <template #prepend>
              <span class="text-body-2 text-grey mr-4" style="min-width: 2rem;">
                {{ (index + 1).toString().padStart(2, '0') }}
              </span>
            </template>

            <v-list-item-title class="text-body-1">
              {{ chapter.title }}
            </v-list-item-title>

            <v-list-item-subtitle v-if="chapter.page">
              Page {{ chapter.page }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>
      </div>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import {
  useGetReleaseQuery,
} from '/@/plugins/lensService/hooks';
import { parseUrlOrCid } from '/@/utils';

interface Chapter {
  title: string;
  page?: number;
}

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

// Fetch the book release
const { data: book, isLoading } = useGetReleaseQuery(props.id);

// Parse table of contents from metadata if available
const tableOfContents = computed<Chapter[]>(() => {
  if (!book.value?.metadata?.tableOfContents) return [];

  try {
    const parsed = JSON.parse(book.value.metadata.tableOfContents);
    return Array.isArray(parsed) ? parsed : [];
  } catch (error) {
    console.error('Failed to parse tableOfContents:', error);
    return [];
  }
});
</script>

<style scoped>
.chapter-item {
  transition: background-color 0.2s ease;
}

.chapter-item:hover {
  background-color: rgba(var(--v-theme-surface-variant), 0.3);
}
</style>
