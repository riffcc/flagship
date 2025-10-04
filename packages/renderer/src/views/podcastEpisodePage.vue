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

    <!-- Episode not found -->
    <v-sheet
      v-else-if="!episode && !isLoading"
      color="transparent"
      class="d-flex flex-column mx-auto mt-8"
      max-width="20rem"
    >
      <v-icon
        size="64"
        class="mb-4 text-center"
        color="grey"
      >mdi-microphone</v-icon>
      <p class="text-h6 text-center mb-2">Episode not found</p>
      <p class="text-body-2 text-center text-grey mb-4">
        The episode you're looking for doesn't exist or has been removed.
      </p>
      <v-btn
        color="primary"
        @click="router.push('/audiobooks')"
      >
        Browse Podcasts
      </v-btn>
    </v-sheet>

    <!-- Episode content -->
    <template v-else-if="episode">
      <!-- Episode header -->
      <v-row class="mb-6">
        <v-col cols="12" md="4" lg="3">
          <v-img
            :src="parseUrlOrCid(episode.thumbnailCID)"
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
                <v-icon size="64" color="grey">mdi-microphone</v-icon>
              </v-sheet>
            </template>
          </v-img>
        </v-col>

        <v-col cols="12" md="8" lg="9">
          <p class="text-overline text-grey mb-1">Podcast Episode</p>
          <h1 class="text-h3 text-sm-h2 font-weight-bold mb-3">{{ episode.name }}</h1>

          <div class="d-flex align-center ga-3 mb-4">
            <v-avatar
              v-if="podcast"
              :image="parseUrlOrCid(podcast.thumbnailCID)"
              size="32"
              class="cursor-pointer"
              @click="router.push(`/podcast/${episode.metadata?.podcastId}`)"
            >
              <template #placeholder>
                <v-icon size="20">mdi-podcast</v-icon>
              </template>
            </v-avatar>
            <a
              v-if="episode.metadata?.podcastId"
              @click.prevent="router.push(`/podcast/${episode.metadata.podcastId}`)"
              class="podcast-link text-h6"
              style="cursor: pointer; color: rgb(var(--v-theme-primary)); text-decoration: none;"
            >{{ podcast?.name || 'Unknown Podcast' }}</a>
            <span v-else class="text-h6">{{ podcast?.name || 'Unknown Podcast' }}</span>
          </div>

          <div class="d-flex align-center ga-3 mb-4">
            <v-chip v-if="episode.metadata?.episodeNumber" size="small" color="secondary">
              Episode {{ episode.metadata.episodeNumber }}
            </v-chip>
            <v-chip v-if="episode.metadata?.releaseDate" size="small" color="accent">
              {{ episode.metadata.releaseDate }}
            </v-chip>
            <v-chip v-if="episode.metadata?.duration" size="small">
              {{ episode.metadata.duration }}
            </v-chip>
          </div>

          <p v-if="episode.metadata?.description" class="text-body-1 mb-4">
            {{ episode.metadata.description }}
          </p>

          <v-btn
            color="primary"
            size="large"
            prepend-icon="mdi-play-circle"
            class="mt-2"
            @click="router.push(`/release/${episode.id}`)"
          >
            Play Episode
          </v-btn>
        </v-col>
      </v-row>

      <v-divider class="my-6" />

      <!-- Show notes (if available) -->
      <div v-if="episode.metadata?.showNotes">
        <h2 class="text-h5 font-weight-bold mb-4">Show Notes</h2>
        <div class="text-body-1 show-notes">
          {{ episode.metadata.showNotes }}
        </div>
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

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

// Fetch the episode release
const { data: episode, isLoading } = useGetReleaseQuery(props.id);

// Fetch the podcast if we have a podcastId
const { data: podcast } = useGetReleaseQuery(
  computed(() => episode.value?.metadata?.podcastId || ''),
  {
    enabled: computed(() => !!episode.value?.metadata?.podcastId)
  }
);
</script>

<style scoped>
.podcast-link:hover {
  text-decoration: underline;
}

.show-notes {
  white-space: pre-wrap;
  line-height: 1.7;
}
</style>
