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

    <!-- Movie not found -->
    <v-sheet
      v-else-if="!movie && !tileData && !isFetching"
      color="transparent"
      class="d-flex flex-column mx-auto mt-8"
      max-width="20rem"
    >
      <v-icon
        size="64"
        class="mb-4 text-center"
        color="grey"
      >$filmstrip</v-icon>
      <p class="text-h6 text-center mb-2">Movie not found</p>
      <p class="text-body-2 text-center text-grey mb-4">
        The movie you're looking for doesn't exist or has been removed.
      </p>
      <v-btn
        color="primary"
        @click="router.push('/movies')"
      >
        Browse Movies
      </v-btn>
    </v-sheet>

    <!-- Movie content -->
    <template v-else-if="movie">
      <!-- Movie header -->
      <v-row class="mb-6">
        <v-col cols="12" md="4" lg="3">
          <v-img
            :src="parseUrlOrCid(movie.thumbnailCID)"
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
                <v-icon size="64" color="grey">$filmstrip</v-icon>
              </v-sheet>
            </template>
          </v-img>
        </v-col>

        <v-col cols="12" md="8" lg="9">
          <p class="text-overline text-grey mb-1">Movie</p>
          <h1 class="text-h3 text-sm-h2 font-weight-bold mb-3">{{ movie.name }}</h1>

          <div class="d-flex align-center flex-wrap ga-3 mb-4">
            <v-chip v-if="movie.metadata?.releaseYear" size="small" color="secondary">
              {{ movie.metadata.releaseYear }}
            </v-chip>
            <v-chip v-if="duration" size="small" color="accent">
              {{ duration }}
            </v-chip>
            <v-chip v-if="movie.metadata?.rating" size="small" color="warning">
              {{ movie.metadata.rating }}
            </v-chip>
          </div>

          <div v-if="genres.length > 0" class="d-flex flex-wrap ga-2 mb-4">
            <v-chip
              v-for="genre in genres"
              :key="genre"
              size="small"
              variant="outlined"
            >
              {{ genre }}
            </v-chip>
          </div>

          <p v-if="movie.metadata?.description" class="text-body-1 mb-4" style="max-width: 800px;">
            {{ movie.metadata.description }}
          </p>

          <v-btn
            color="primary"
            size="large"
            prepend-icon="$play"
            class="mt-2"
            @click="router.push(`/release/${movie.id}`)"
          >
            Watch Now
          </v-btn>
        </v-col>
      </v-row>

      <v-divider class="my-6" />

      <!-- Cast & Crew (if available) -->
      <div v-if="hasCastOrCrew" class="mb-8">
        <h2 class="text-h5 font-weight-bold mb-4">Cast & Crew</h2>

        <div v-if="movie.metadata?.director" class="mb-3">
          <span class="text-grey">Director:</span>
          <span class="ml-2">{{ movie.metadata.director }}</span>
        </div>

        <div v-if="movie.metadata?.cast" class="mb-3">
          <span class="text-grey">Cast:</span>
          <span class="ml-2">{{ movie.metadata.cast }}</span>
        </div>

        <div v-if="movie.metadata?.writer" class="mb-3">
          <span class="text-grey">Writer:</span>
          <span class="ml-2">{{ movie.metadata.writer }}</span>
        </div>
      </div>

      <!-- Technical Details (if available) -->
      <div v-if="hasTechnicalDetails">
        <h2 class="text-h5 font-weight-bold mb-4">Details</h2>

        <v-row>
          <v-col v-if="movie.metadata?.language" cols="12" sm="6" md="4">
            <span class="text-grey">Language:</span>
            <span class="ml-2">{{ movie.metadata.language }}</span>
          </v-col>

          <v-col v-if="movie.metadata?.country" cols="12" sm="6" md="4">
            <span class="text-grey">Country:</span>
            <span class="ml-2">{{ movie.metadata.country }}</span>
          </v-col>

          <v-col v-if="movie.metadata?.studio" cols="12" sm="6" md="4">
            <span class="text-grey">Studio:</span>
            <span class="ml-2">{{ movie.metadata.studio }}</span>
          </v-col>
        </v-row>
      </div>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useGetReleaseQuery } from '/@/plugins/lensService/hooks';
import { parseUrlOrCid } from '/@/utils';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

// INSTANT: Get tile data from router state (passed from contentCard)
const tileData = ref<{
  name?: string;
  thumbnailCID?: string;
  releaseYear?: string | number;
} | null>(null);

onMounted(() => {
  if (history.state) {
    tileData.value = {
      name: history.state.name,
      thumbnailCID: history.state.thumbnailCID,
      releaseYear: history.state.releaseYear,
    };
  }
});

// Fetch the movie release (lazy - fills in details)
const { data: movie, isLoading: isFetching } = useGetReleaseQuery(props.id);

// Show loading only if we have NO data
const isLoading = computed(() => !tileData.value && !movie.value && isFetching.value);

// Use tile data for instant render, fall back to fetched data
const displayName = computed(() => movie.value?.name || tileData.value?.name);
const displayThumbnail = computed(() => movie.value?.thumbnailCID || tileData.value?.thumbnailCID);
const displayYear = computed(() => movie.value?.metadata?.releaseYear || tileData.value?.releaseYear);

// Format duration from minutes to hours and minutes
const duration = computed(() => {
  const mins = movie.value?.metadata?.duration || movie.value?.metadata?.runtime;
  if (!mins) return null;

  const hours = Math.floor(mins / 60);
  const minutes = mins % 60;

  if (hours === 0) return `${minutes}m`;
  if (minutes === 0) return `${hours}h`;
  return `${hours}h ${minutes}m`;
});

// Parse genres from metadata
const genres = computed(() => {
  const genreData = movie.value?.metadata?.genre || movie.value?.metadata?.genres;
  if (!genreData) return [];

  if (Array.isArray(genreData)) return genreData;
  if (typeof genreData === 'string') {
    return genreData.split(',').map(g => g.trim()).filter(Boolean);
  }
  return [];
});

// Check if we have cast/crew info
const hasCastOrCrew = computed(() => {
  const meta = movie.value?.metadata;
  return meta?.director || meta?.cast || meta?.writer;
});

// Check if we have technical details
const hasTechnicalDetails = computed(() => {
  const meta = movie.value?.metadata;
  return meta?.language || meta?.country || meta?.studio;
});
</script>

<style scoped>
</style>
