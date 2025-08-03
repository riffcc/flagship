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

    <!-- Series not found -->
    <v-sheet
      v-else-if="!series && !isLoading"
      color="transparent"
      class="d-flex flex-column mx-auto mt-8"
      max-width="20rem"
    >
      <v-icon
        size="64"
        class="mb-4 text-center"
        color="grey"
      >$television-off</v-icon>
      <p class="text-h6 text-center mb-2">Series not found</p>
      <p class="text-body-2 text-center text-grey mb-4">
        The series you're looking for doesn't exist or has been removed.
      </p>
      <v-btn
        color="primary"
        @click="router.push('/featured/tv-shows')"
      >
        Browse TV Shows
      </v-btn>
    </v-sheet>

    <!-- Series content -->
    <template v-else-if="series">
      <!-- Series header -->
      <v-row class="mb-6">
        <v-col cols="12" md="3">
          <v-img
            :src="parseUrlOrCid(series.thumbnailCID || episodes[0]?.thumbnailCID)"
            aspect-ratio="0.75"
            cover
            rounded="lg"
            class="elevation-4"
          >
            <template #placeholder>
              <v-sheet
                color="grey-darken-3"
                class="d-flex align-center justify-center fill-height"
              >
                <v-icon size="64" color="grey">$television</v-icon>
              </v-sheet>
            </template>
          </v-img>
        </v-col>
        
        <v-col cols="12" md="9">
          <h1 class="text-h3 text-sm-h2 font-weight-bold mb-2">{{ series.name }}</h1>
          
          <div class="d-flex align-center ga-4 mb-4">
            <v-chip size="small" color="primary">
              {{ totalSeasons }} {{ totalSeasons === 1 ? 'Season' : 'Seasons' }}
            </v-chip>
            <v-chip size="small" color="secondary">
              {{ episodes.length }} {{ episodes.length === 1 ? 'Episode' : 'Episodes' }}
            </v-chip>
          </div>

          <p v-if="series.description" class="text-body-1 mb-4">
            {{ series.description }}
          </p>

          <!-- Play first episode button -->
          <v-btn
            v-if="firstEpisode"
            color="primary"
            size="large"
            prepend-icon="$play"
            @click="router.push(`/release/${firstEpisode.id}`)"
          >
            Play First Episode
          </v-btn>
        </v-col>
      </v-row>

      <v-divider class="my-6" />

      <!-- Season selector -->
      <v-tabs
        v-if="seasons.length > 0"
        v-model="selectedSeasonTab"
        class="mb-6"
        color="primary"
      >
        <v-tab
          v-for="season in seasons"
          :key="season.id"
          :value="season.id"
        >
          {{ season.name }}
        </v-tab>
      </v-tabs>

      <!-- Episodes grid -->
      <div v-if="currentSeasonEpisodes.length > 0">
        <h2 class="text-h5 font-weight-bold mb-4">
          {{ selectedSeason?.name || 'Episodes' }}
        </h2>
        
        <v-row>
          <v-col
            v-for="episode in currentSeasonEpisodes"
            :key="episode.id"
            cols="12"
            sm="6"
            md="4"
            lg="3"
          >
            <v-card
              class="episode-card"
              rounded="lg"
              @click="router.push(`/release/${episode.id}`)"
            >
              <v-img
                :src="parseUrlOrCid(episode.thumbnailCID)"
                aspect-ratio="1.78"
                cover
              >
                <template #placeholder>
                  <v-sheet
                    color="grey-darken-3"
                    class="d-flex align-center justify-center fill-height"
                  >
                    <v-icon size="48" color="grey">$television-play</v-icon>
                  </v-sheet>
                </template>
              </v-img>
              
              <v-card-text>
                <p class="text-caption text-grey mb-1">
                  Episode {{ episode.metadata?.episodeNumber || '?' }}
                </p>
                <p class="text-body-2 font-weight-medium">
                  {{ episode.name }}
                </p>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </div>

      <!-- No episodes message -->
      <v-sheet
        v-else
        color="transparent"
        class="d-flex flex-column mx-auto mt-8"
        max-width="20rem"
      >
        <p class="text-body-1 text-center text-grey">
          No episodes available for this season.
        </p>
      </v-sheet>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { 
  useGetStructureQuery, 
  useGetStructuresQuery,
  useGetReleasesQuery 
} from '/@/plugins/lensService/hooks';
import { parseUrlOrCid } from '/@/utils';
import type { ReleaseItem } from '/@/types';

const route = useRoute();
const router = useRouter();

// Get series ID from route
const seriesId = computed(() => route.params.id as string);

// Fetch the series structure
const { data: series, isLoading: isSeriesLoading } = useGetStructureQuery(seriesId.value);

// Fetch all structures to get seasons
const { data: structures, isLoading: isStructuresLoading } = useGetStructuresQuery({
  searchOptions: { fetch: 1000 }
});

// Fetch all releases to get episodes
const { data: releases, isLoading: isReleasesLoading } = useGetReleasesQuery({
  searchOptions: { fetch: 1000 }
});

const isLoading = computed(() => 
  isSeriesLoading.value || isStructuresLoading.value || isReleasesLoading.value
);

// Get all seasons for this series
const seasons = computed(() => {
  if (!structures.value || !seriesId.value) return [];
  
  return structures.value
    .filter((s: any) => s.type === 'season' && s.parentId === seriesId.value)
    .sort((a: any, b: any) => (a.order || 0) - (b.order || 0));
});

// Get total number of seasons
const totalSeasons = computed(() => seasons.value.length || 1);

// Selected season tab
const selectedSeasonTab = ref<string>('');

// Set initial selected season
watch(seasons, (newSeasons) => {
  if (newSeasons.length > 0 && !selectedSeasonTab.value) {
    selectedSeasonTab.value = newSeasons[0].id;
  }
}, { immediate: true });

// Get selected season
const selectedSeason = computed(() => 
  seasons.value.find((s: any) => s.id === selectedSeasonTab.value)
);

// Get all episodes for this series
const episodes = computed<ReleaseItem[]>(() => {
  if (!releases.value || !seriesId.value) return [];
  
  return releases.value.filter(
    (r: ReleaseItem) => r.metadata?.seriesId === seriesId.value
  );
});

// Get episodes for current season
const currentSeasonEpisodes = computed<ReleaseItem[]>(() => {
  if (!episodes.value) return [];
  
  // If we have seasons, filter by selected season
  if (selectedSeason.value) {
    return episodes.value
      .filter((e: ReleaseItem) => e.metadata?.seasonId === selectedSeason.value.id)
      .sort((a: ReleaseItem, b: ReleaseItem) => {
        const aNum = a.metadata?.episodeNumber || 0;
        const bNum = b.metadata?.episodeNumber || 0;
        return aNum - bNum;
      });
  }
  
  // No seasons, show all episodes
  return episodes.value.sort((a: ReleaseItem, b: ReleaseItem) => {
    const aSeason = a.metadata?.seasonNumber || 1;
    const bSeason = b.metadata?.seasonNumber || 1;
    const aEpisode = a.metadata?.episodeNumber || 0;
    const bEpisode = b.metadata?.episodeNumber || 0;
    
    if (aSeason !== bSeason) return aSeason - bSeason;
    return aEpisode - bEpisode;
  });
});

// Get first episode for play button
const firstEpisode = computed(() => {
  if (episodes.value.length === 0) return null;
  
  // Sort all episodes and return the first
  const sorted = [...episodes.value].sort((a: ReleaseItem, b: ReleaseItem) => {
    const aSeason = a.metadata?.seasonNumber || 1;
    const bSeason = b.metadata?.seasonNumber || 1;
    const aEpisode = a.metadata?.episodeNumber || 0;
    const bEpisode = b.metadata?.episodeNumber || 0;
    
    if (aSeason !== bSeason) return aSeason - bSeason;
    return aEpisode - bEpisode;
  });
  
  return sorted[0];
});
</script>

<style scoped>
.episode-card {
  cursor: pointer;
  transition: all 0.3s ease;
}

.episode-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
}
</style>