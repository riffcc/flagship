<template>
  <v-container class="fill-height pb-16">
    <v-sheet
      v-if="isLoadingSeries || isLoadingReleases"
      color="transparent"
      class="d-flex w-100 fill-height align-center justify-center"
    >
      <v-progress-circular indeterminate color="primary"></v-progress-circular>
    </v-sheet>
    <v-sheet v-else-if="!series" color="transparent" class="ma-auto text-center">
      <p class="text-h6">TV Series not found.</p>
      <v-btn color="primary" @click="router.back()">Go Back</v-btn>
    </v-sheet>
    <v-row v-else>
      <!-- Series Header -->
      <v-col cols="12">
        <v-card flat color="transparent">
          <div :class="{'d-flex flex-no-wrap justify-space-between': mdAndUp}">
             <v-avatar
                v-if="!mdAndUp"
                class="ma-3"
                size="125"
                rounded="0"
              >
                <v-img :src="parseUrlOrCid(series.cover || series.thumbnail)" cover>
                   <template #placeholder>
                    <v-sheet color="grey-lighten-2" class="fill-height d-flex align-center justify-center">
                      <v-icon>mdi-image</v-icon>
                    </v-sheet>
                  </template>
                </v-img>
              </v-avatar>
            <div>
              <v-card-title class="text-h4">{{ series.name }}</v-card-title>
              <v-card-subtitle v-if="series.sourceSite">From: {{ series.sourceSite }}</v-card-subtitle>
              <v-card-text>
                <p v-if="series.description">{{ series.description }}</p>
                <p v-else class="text-grey">No description available.</p>
              </v-card-text>
            </div>
             <v-avatar
                v-if="mdAndUp"
                class="ma-3"
                size="150"
                rounded="lg"
                elevation="4"
              >
                 <v-img :src="parseUrlOrCid(series.cover || series.thumbnail)" cover>
                   <template #placeholder>
                    <v-sheet color="grey-lighten-2" class="fill-height d-flex align-center justify-center">
                      <v-icon>mdi-image</v-icon>
                    </v-sheet>
                  </template>
                 </v-img>
              </v-avatar>
          </div>
        </v-card>
      </v-col>

      <!-- Seasons and Episodes -->
      <v-col cols="12">
        <v-expansion-panels v-if="seasons.length > 0" variant="accordion">
          <v-expansion-panel
            v-for="season in seasons"
            :key="season.number"
            :title="`Season ${season.number}`"
            elevation="1"
          >
            <v-expansion-panel-text>
              <v-list lines="two" density="compact">
                <v-list-item
                  v-for="episode in season.episodes"
                  :key="episode.id"
                  :title="`Episode ${episode.episodeNumber}: ${episode.name}`"
                  :subtitle="episode.author"
                  @click="playEpisode(episode)"
                >
                  <template #prepend>
                    <v-avatar rounded="0" class="me-4">
                       <v-img :src="parseUrlOrCid(episode.thumbnail)" cover>
                         <template #placeholder>
                          <v-sheet color="grey-lighten-3" class="fill-height d-flex align-center justify-center">
                            <v-icon size="small">mdi-television-play</v-icon>
                          </v-sheet>
                        </template>
                       </v-img>
                    </v-avatar>
                  </template>
                  <template #append>
                    <v-btn icon flat size="small" @click.stop="playEpisode(episode)">
                      <v-icon>mdi-play-circle-outline</v-icon>
                    </v-btn>
                  </template>
                </v-list-item>
              </v-list>
            </v-expansion-panel-text>
          </v-expansion-panel>
        </v-expansion-panels>
        <v-sheet v-else color="transparent" class="text-center pa-4">
          <p>No episodes found for this series.</p>
          <!-- Optional: Link to upload or admin -->
        </v-sheet>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useDisplay } from 'vuetify';
import { storeToRefs } from 'pinia';
import { useTvSeriesStore } from '/@/stores/tvSeries';
import { useReleasesStore } from '/@/stores/releases';
import type { Episode, Season, ReleaseItem } from '/@/@types/release';
import { parseUrlOrCid } from '/@/utils';
import { usePlaybackController } from '/@/composables/playbackController'; // Assuming playback controller exists

const route = useRoute();
const router = useRouter();
const { mdAndUp } = useDisplay();
const { playContent } = usePlaybackController(); // Use playback composable

const seriesId = computed(() => route.params.id as string);

// Stores
const tvSeriesStore = useTvSeriesStore();
const releasesStore = useReleasesStore();

// State Refs
const { getSeriesById } = tvSeriesStore;
const { releases, isLoading: isLoadingReleases } = storeToRefs(releasesStore);
const { isLoading: isLoadingSeries } = storeToRefs(tvSeriesStore); // Use series store loading state

// Computed Properties
const series = getSeriesById(seriesId.value); // Get reactive series data

const seriesEpisodes = computed<Episode[]>(() => {
  return releases.value
    .filter(
      (r): r is Episode => // Type guard
        r.category === 'tvShow' && r.seriesId === seriesId.value && r.seasonNumber !== undefined && r.episodeNumber !== undefined
    )
    .sort((a, b) => { // Sort by season then episode
      if (a.seasonNumber !== b.seasonNumber) {
        return a.seasonNumber - b.seasonNumber;
      }
      return a.episodeNumber - b.episodeNumber;
    });
});

const seasons = computed<Season[]>(() => {
  const seasonMap = new Map<number, Episode[]>();
  seriesEpisodes.value.forEach(episode => {
    const seasonNum = episode.seasonNumber;
    if (!seasonMap.has(seasonNum)) {
      seasonMap.set(seasonNum, []);
    }
    seasonMap.get(seasonNum)?.push(episode);
  });

  // Convert map to sorted array of Season objects
  return Array.from(seasonMap.entries())
    .sort(([numA], [numB]) => numA - numB) // Sort seasons numerically
    .map(([number, episodes]) => ({ number, episodes }));
});

// Methods
const playEpisode = (episode: ReleaseItem) => {
  // Assuming playContent can handle a ReleaseItem
  playContent(episode);
};

// Watch for series data if fetched asynchronously after component mount
watch(seriesId, (newId) => {
  if (!newId) {
    router.push('/tv-shows'); // Redirect if ID is invalid/missing
  }
  // Potentially trigger a fetch if series data isn't available yet,
  // although the store should handle this reactively.
}, { immediate: true });

</script>

<style scoped>
/* Add styles if needed */
</style>
