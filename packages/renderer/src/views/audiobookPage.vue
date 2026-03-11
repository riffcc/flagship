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

    <!-- Audiobook not found -->
    <v-sheet
      v-else-if="!audiobook && !isLoading"
      color="transparent"
      class="d-flex flex-column mx-auto mt-8"
      max-width="20rem"
    >
      <v-icon
        size="64"
        class="mb-4 text-center"
        color="grey"
      >$book-music</v-icon>
      <p class="text-h6 text-center mb-2">Audiobook not found</p>
      <p class="text-body-2 text-center text-grey mb-4">
        The audiobook you're looking for doesn't exist or has been removed.
      </p>
      <v-btn
        color="primary"
        @click="router.push('/audiobooks')"
      >
        Browse Audiobooks
      </v-btn>
    </v-sheet>

    <!-- Audiobook content -->
    <template v-else-if="audiobook">
      <!-- Audiobook header -->
      <v-row class="mb-6">
        <v-col cols="12" md="4" lg="3">
          <v-img
            :src="parseUrlOrCid(audiobook.thumbnailCID)"
            aspect-ratio="1"
            cover
            rounded="lg"
            class="elevation-8"
          >
            <template #placeholder>
              <v-sheet
                color="grey-darken-3"
                class="d-flex align-center justify-center fill-height"
              >
                <v-icon size="64" color="grey">$book-music</v-icon>
              </v-sheet>
            </template>
          </v-img>
        </v-col>

        <v-col cols="12" md="8" lg="9">
          <p class="text-overline text-grey mb-1">Audiobook</p>
          <h1 class="text-h3 text-sm-h2 font-weight-bold mb-3">{{ audiobook.name }}</h1>

          <div class="d-flex align-center ga-3 mb-4">
            <p v-if="audiobook.metadata?.author" class="text-h6 mb-0">
              by {{ audiobook.metadata.author }}
            </p>
          </div>

          <div class="d-flex align-center ga-3 mb-4">
            <v-chip v-if="audiobook.metadata?.narrator" size="small" color="primary">
              Narrated by {{ audiobook.metadata.narrator }}
            </v-chip>
            <v-chip v-if="audiobook.metadata?.releaseYear" size="small" color="secondary">
              {{ audiobook.metadata.releaseYear }}
            </v-chip>
            <v-chip v-if="audiobook.metadata?.duration" size="small" color="accent">
              {{ audiobook.metadata.duration }}
            </v-chip>
            <v-chip v-if="audiobook.metadata?.genre" size="small">
              {{ audiobook.metadata.genre }}
            </v-chip>
          </div>

          <p v-if="audiobook.metadata?.description" class="text-body-1 mb-4">
            {{ audiobook.metadata.description }}
          </p>

          <v-btn
            color="primary"
            size="large"
            prepend-icon="$play-circle"
            class="mt-2"
            @click="router.push(`/release/${audiobook.id}`)"
          >
            Play Audiobook
          </v-btn>
        </v-col>
      </v-row>

      <v-divider class="my-6" />

      <!-- Additional info -->
      <div v-if="audiobook.metadata?.publisher || audiobook.metadata?.isbn">
        <h2 class="text-h5 font-weight-bold mb-4">Publication Details</h2>

        <v-list bg-color="transparent">
          <v-list-item v-if="audiobook.metadata?.publisher">
            <v-list-item-title class="text-body-2 text-grey">Publisher</v-list-item-title>
            <v-list-item-subtitle class="text-body-1">{{ audiobook.metadata.publisher }}</v-list-item-subtitle>
          </v-list-item>

          <v-list-item v-if="audiobook.metadata?.isbn">
            <v-list-item-title class="text-body-2 text-grey">ISBN</v-list-item-title>
            <v-list-item-subtitle class="text-body-1">{{ audiobook.metadata.isbn }}</v-list-item-subtitle>
          </v-list-item>

          <v-list-item v-if="audiobook.metadata?.language">
            <v-list-item-title class="text-body-2 text-grey">Language</v-list-item-title>
            <v-list-item-subtitle class="text-body-1">{{ audiobook.metadata.language }}</v-list-item-subtitle>
          </v-list-item>
        </v-list>
      </div>

      <!-- Chapters list if available -->
      <div v-if="chapters.length > 0" class="mt-6">
        <h2 class="text-h5 font-weight-bold mb-4">Chapters</h2>

        <v-list bg-color="transparent">
          <v-list-item
            v-for="(chapter, index) in chapters"
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

            <v-list-item-subtitle v-if="chapter.duration">
              {{ chapter.duration }}
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
  duration?: string;
}

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

// Fetch the audiobook release
const { data: audiobook, isLoading } = useGetReleaseQuery(props.id);

// Parse chapters from metadata if available
const chapters = computed<Chapter[]>(() => {
  if (!audiobook.value?.metadata?.chapters) return [];

  try {
    const parsed = JSON.parse(audiobook.value.metadata.chapters);
    return Array.isArray(parsed) ? parsed : [];
  } catch (error) {
    console.error('Failed to parse chapters:', error);
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
