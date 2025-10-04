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

    <!-- Podcast not found -->
    <v-sheet
      v-else-if="!podcast && !isLoading"
      color="transparent"
      class="d-flex flex-column mx-auto mt-8"
      max-width="20rem"
    >
      <v-icon
        size="64"
        class="mb-4 text-center"
        color="grey"
      >mdi-podcast</v-icon>
      <p class="text-h6 text-center mb-2">Podcast not found</p>
      <p class="text-body-2 text-center text-grey mb-4">
        The podcast you're looking for doesn't exist or has been removed.
      </p>
      <v-btn
        color="primary"
        @click="router.push('/audiobooks')"
      >
        Browse Podcasts
      </v-btn>
    </v-sheet>

    <!-- Podcast content -->
    <template v-else-if="podcast">
      <!-- Podcast header -->
      <v-row class="mb-6">
        <v-col cols="12" md="3">
          <v-img
            :src="parseUrlOrCid(podcast.thumbnailCID)"
            aspect-ratio="1"
            cover
            rounded="lg"
            class="elevation-4"
          >
            <template #placeholder>
              <v-sheet
                color="grey-darken-3"
                class="d-flex align-center justify-center fill-height"
              >
                <v-icon size="64" color="grey">mdi-podcast</v-icon>
              </v-sheet>
            </template>
          </v-img>
        </v-col>

        <v-col cols="12" md="9">
          <p class="text-overline text-grey mb-1">Podcast</p>
          <h1 class="text-h3 text-sm-h2 font-weight-bold mb-2">{{ podcast.name }}</h1>

          <div class="d-flex align-center ga-4 mb-4">
            <v-chip v-if="podcast.metadata?.host" size="small" color="primary">
              {{ podcast.metadata.host }}
            </v-chip>
            <v-chip v-if="podcast.metadata?.genre" size="small" color="secondary">
              {{ podcast.metadata.genre }}
            </v-chip>
            <v-chip size="small" color="accent">
              {{ episodes.length }} {{ episodes.length === 1 ? 'Episode' : 'Episodes' }}
            </v-chip>
          </div>

          <p v-if="podcast.metadata?.description" class="text-body-1 mb-4">
            {{ podcast.metadata.description }}
          </p>
        </v-col>
      </v-row>

      <v-divider class="my-6" />

      <!-- Episodes list -->
      <div v-if="episodes.length > 0">
        <h2 class="text-h5 font-weight-bold mb-4">Episodes</h2>

        <v-list bg-color="transparent">
          <v-list-item
            v-for="(episode, index) in episodes"
            :key="episode.id"
            class="episode-item mb-2 rounded"
            @click="router.push(`/podcast-episode/${episode.id}`)"
          >
            <template #prepend>
              <v-avatar
                :image="parseUrlOrCid(episode.thumbnailCID)"
                size="80"
                rounded="lg"
                class="mr-4"
              >
                <template #placeholder>
                  <v-sheet
                    color="grey-darken-3"
                    class="d-flex align-center justify-center fill-height"
                  >
                    <v-icon size="32" color="grey">mdi-microphone</v-icon>
                  </v-sheet>
                </template>
              </v-avatar>
            </template>

            <v-list-item-title class="text-body-1 font-weight-medium">
              {{ episode.name }}
            </v-list-item-title>

            <v-list-item-subtitle v-if="episode.metadata?.episodeNumber" class="mt-1">
              Episode {{ episode.metadata.episodeNumber }}
              <span v-if="episode.metadata?.releaseDate"> • {{ episode.metadata.releaseDate }}</span>
            </v-list-item-subtitle>

            <v-list-item-subtitle v-if="episode.metadata?.description" class="mt-2">
              {{ truncateText(episode.metadata.description, 150) }}
            </v-list-item-subtitle>

            <template #append>
              <v-icon color="grey">mdi-chevron-right</v-icon>
            </template>
          </v-list-item>
        </v-list>
      </div>

      <!-- No episodes message -->
      <v-sheet
        v-else
        color="transparent"
        class="d-flex flex-column mx-auto mt-8"
        max-width="20rem"
      >
        <p class="text-body-1 text-center text-grey">
          No episodes available for this podcast.
        </p>
      </v-sheet>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import {
  useGetReleaseQuery,
  useGetReleasesQuery
} from '/@/plugins/lensService/hooks';
import { parseUrlOrCid } from '/@/utils';
import type { ReleaseItem } from '/@/types';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

// Fetch the podcast release
const { data: podcast, isLoading: isPodcastLoading } = useGetReleaseQuery(props.id);

// Fetch all releases to get episodes
const { data: releases, isLoading: isReleasesLoading } = useGetReleasesQuery({
  searchOptions: { fetch: 1000 }
});

const isLoading = computed(() =>
  isPodcastLoading.value || isReleasesLoading.value
);

// Get all episodes for this podcast, sorted by episode number
const episodes = computed<ReleaseItem[]>(() => {
  if (!releases.value || !props.id) return [];

  return releases.value
    .filter((r: ReleaseItem) => r.metadata?.podcastId === props.id)
    .sort((a: ReleaseItem, b: ReleaseItem) => {
      const aNum = parseInt(a.metadata?.episodeNumber || '0');
      const bNum = parseInt(b.metadata?.episodeNumber || '0');
      return aNum - bNum; // Oldest first
    });
});

// Utility function to truncate text
function truncateText(text: string, maxLength: number): string {
  if (!text) return '';
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength).trim() + '...';
}
</script>

<style scoped>
.episode-item {
  cursor: pointer;
  transition: background-color 0.2s ease;
  border: 1px solid rgba(var(--v-border-color), 0.12);
}

.episode-item:hover {
  background-color: rgba(var(--v-theme-surface-variant), 0.3);
}
</style>
