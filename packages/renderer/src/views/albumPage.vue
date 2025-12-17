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

    <!-- Album not found -->
    <v-sheet
      v-else-if="!album && !isLoading"
      color="transparent"
      class="d-flex flex-column mx-auto mt-8"
      max-width="20rem"
    >
      <v-icon
        size="64"
        class="mb-4 text-center"
        color="grey"
      >$album</v-icon>
      <p class="text-h6 text-center mb-2">Album not found</p>
      <p class="text-body-2 text-center text-grey mb-4">
        The album you're looking for doesn't exist or has been removed.
      </p>
      <v-btn
        color="primary"
        @click="router.push('/music')"
      >
        Browse Music
      </v-btn>
    </v-sheet>

    <!-- Album content -->
    <template v-else-if="album">
      <!-- Album header -->
      <v-row class="mb-6">
        <v-col cols="12" md="4" lg="3">
          <!-- Flippable Album Cover (easter egg - only works if back cover exists) -->
          <div
            class="album-cover-container"
            :class="{ 'is-flipped': isFlipped }"
            @mousedown="album.metadata?.backCoverCID ? startDrag($event) : null"
            @touchstart="album.metadata?.backCoverCID ? startDrag($event) : null"
          >
            <div class="album-cover-flipper">
              <!-- Front Cover -->
              <div class="album-cover-face album-cover-front">
                <v-img
                  :src="parseUrlOrCid(album.thumbnailCID)"
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
                      <v-icon size="64" color="grey">$album</v-icon>
                    </v-sheet>
                  </template>
                </v-img>
              </div>

              <!-- Back Cover (if available) -->
              <div class="album-cover-face album-cover-back">
                <v-img
                  v-if="album.metadata?.backCoverCID"
                  :src="parseUrlOrCid(album.metadata.backCoverCID)"
                  aspect-ratio="1"
                  cover
                  rounded="lg"
                  class="elevation-8"
                />
                <v-sheet
                  v-else
                  color="grey-darken-2"
                  class="d-flex align-center justify-center fill-height"
                  rounded="lg"
                >
                  <div class="text-center pa-4">
                    <v-icon size="48" color="grey-lighten-1" class="mb-2">$album</v-icon>
                    <p class="text-caption text-grey-lighten-1">Back Cover</p>
                  </div>
                </v-sheet>
              </div>
            </div>
          </div>
        </v-col>

        <v-col cols="12" md="8" lg="9">
          <p class="text-overline text-grey mb-1">Album</p>
          <h1 class="text-h3 text-sm-h2 font-weight-bold mb-3">{{ album.name }}</h1>

          <div class="d-flex align-center ga-3 mb-4">
            <v-avatar
              v-if="artist"
              :image="parseUrlOrCid(artist.thumbnailCID)"
              size="32"
              class="cursor-pointer"
              @click="router.push(`/artist/${album.metadata?.artistId}`)"
            >
              <template #placeholder>
                <v-icon size="20">$account-music</v-icon>
              </template>
            </v-avatar>
            <a
              v-if="album.metadata?.artistId"
              @click.prevent="router.push(`/artist/${album.metadata.artistId}`)"
              class="artist-link text-h6"
              style="cursor: pointer; color: rgb(var(--v-theme-primary)); text-decoration: none;"
            >{{ album.metadata?.author || 'Unknown Artist' }}</a>
            <span v-else class="text-h6">{{ album.metadata?.author || 'Unknown Artist' }}</span>
          </div>

          <div class="d-flex align-center ga-3 mb-4">
            <v-chip v-if="album.metadata?.releaseYear" size="small" color="secondary">
              {{ album.metadata.releaseYear }}
            </v-chip>
            <v-chip v-if="trackList.length > 0" size="small" color="accent">
              {{ trackList.length }} {{ trackList.length === 1 ? 'Track' : 'Tracks' }}
            </v-chip>
          </div>

          <p v-if="album.metadata?.description" class="text-body-1 mb-4">
            {{ album.metadata.description }}
          </p>

          <v-btn
            color="primary"
            size="large"
            prepend-icon="$play-circle"
            class="mt-2"
            @click="router.push(`/release/${album.id}`)"
          >
            Play Album
          </v-btn>
        </v-col>
      </v-row>

      <v-divider class="my-6" />

      <!-- Artwork Gallery (for albums with track artwork) -->
      <div v-if="hasTrackArtwork" class="mb-8">
        <h2 class="text-h5 font-weight-bold mb-4">Artwork Gallery</h2>
        <v-row>
          <v-col
            v-for="(artwork, index) in trackArtworkArray"
            :key="index"
            cols="6"
            sm="4"
            md="3"
            lg="2"
          >
            <v-card
              class="artwork-card"
              rounded="lg"
              elevation="4"
              @click="openArtworkDialog(index)"
            >
              <v-img
                :src="artwork"
                aspect-ratio="1"
                cover
              >
                <template #placeholder>
                  <v-sheet
                    color="grey-darken-3"
                    class="d-flex align-center justify-center fill-height"
                  >
                    <v-icon size="48" color="grey">$image</v-icon>
                  </v-sheet>
                </template>
              </v-img>
              <v-card-text class="pa-2 text-center">
                <p class="text-caption">{{ trackList[index]?.title || `Track ${index + 1}` }}</p>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </div>

      <!-- Artwork Dialog (full size view) -->
      <v-dialog v-model="artworkDialog" max-width="90vw">
        <v-card>
          <v-card-text class="pa-0">
            <v-img
              :src="trackArtworkArray[selectedArtworkIndex]"
              contain
              max-height="85vh"
            />
          </v-card-text>
          <v-card-text class="text-center">
            <p class="text-h6">{{ trackList[selectedArtworkIndex]?.title || `Track ${selectedArtworkIndex + 1}` }}</p>
            <p class="text-body-2 text-grey">{{ album?.name }} - {{ album?.metadata?.author }}</p>
          </v-card-text>
          <v-card-actions class="justify-center">
            <v-btn
              icon="$chevron-left"
              @click="previousArtwork"
              :disabled="selectedArtworkIndex === 0"
            />
            <v-btn
              color="primary"
              @click="artworkDialog = false"
            >
              Close
            </v-btn>
            <v-btn
              icon="$chevron-right"
              @click="nextArtwork"
              :disabled="selectedArtworkIndex === trackArtworkArray.length - 1"
            />
          </v-card-actions>
        </v-card>
      </v-dialog>

      <v-divider v-if="hasTrackArtwork" class="my-6" />

      <!-- Track list -->
      <div v-if="trackList.length > 0">
        <h2 class="text-h5 font-weight-bold mb-4">Track List</h2>

        <v-list bg-color="transparent">
          <v-list-item
            v-for="(track, index) in trackList"
            :key="index"
            class="track-item px-2 rounded"
          >
            <template #prepend>
              <span class="text-body-2 text-grey mr-4" style="min-width: 2rem;">
                {{ (index + 1).toString().padStart(2, '0') }}
              </span>
            </template>

            <v-list-item-title class="text-body-1">
              {{ track.title }}
            </v-list-item-title>

            <v-list-item-subtitle v-if="track.artist && track.artist !== album.metadata?.author">
              {{ track.artist }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>
      </div>

      <!-- No tracks message -->
      <v-sheet
        v-else
        color="transparent"
        class="d-flex flex-column mx-auto mt-8"
        max-width="20rem"
      >
        <p class="text-body-1 text-center text-grey">
          No track information available for this album.
        </p>
      </v-sheet>
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import {
  useGetReleaseQuery,
} from '/@/plugins/lensService/hooks';
import { parseUrlOrCid } from '/@/utils';

interface Track {
  title: string;
  artist?: string;
}

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

// Fetch the album release
const { data: album, isLoading } = useGetReleaseQuery(props.id);

// Fetch the artist if we have an artistId
const { data: artist } = useGetReleaseQuery(
  computed(() => album.value?.metadata?.artistId || ''),
  {
    enabled: computed(() => !!album.value?.metadata?.artistId)
  }
);

// Parse track list from metadata
const trackList = computed<Track[]>(() => {
  if (!album.value?.metadata?.trackMetadata) return [];

  try {
    const parsed = JSON.parse(album.value.metadata.trackMetadata);
    return Array.isArray(parsed) ? parsed : [];
  } catch (error) {
    console.error('Failed to parse trackMetadata:', error);
    return [];
  }
});

// Get track artwork array from metadata (parsed through parseUrlOrCid)
const trackArtworkArray = computed(() => {
  if (!album.value?.metadata?.trackArtwork) return [];

  try {
    const artwork = typeof album.value.metadata.trackArtwork === 'string'
      ? JSON.parse(album.value.metadata.trackArtwork)
      : album.value.metadata.trackArtwork;

    if (Array.isArray(artwork)) {
      // Filter out empty strings and parse through parseUrlOrCid
      return artwork
        .filter((url: string) => url && url.trim() !== '')
        .map((url: string) => parseUrlOrCid(url));
    }
    // If it's an object, convert to array
    if (typeof artwork === 'object') {
      return Object.values(artwork)
        .filter((url: any) => url && url.trim() !== '')
        .map((url: any) => parseUrlOrCid(url));
    }
  } catch (e) {
    console.error('Failed to parse trackArtwork:', e);
  }

  return [];
});

const hasTrackArtwork = computed(() => trackArtworkArray.value.length > 0);

// Artwork dialog state
const artworkDialog = ref(false);
const selectedArtworkIndex = ref(0);

function openArtworkDialog(index: number) {
  selectedArtworkIndex.value = index;
  artworkDialog.value = true;
}

function nextArtwork() {
  if (selectedArtworkIndex.value < trackArtworkArray.value.length - 1) {
    selectedArtworkIndex.value++;
  }
}

function previousArtwork() {
  if (selectedArtworkIndex.value > 0) {
    selectedArtworkIndex.value--;
  }
}

// Album cover flip interaction
const isFlipped = ref(false);
let dragStartX = 0;
let isDragging = false;

function startDrag(event: MouseEvent | TouchEvent) {
  isDragging = true;
  dragStartX = 'touches' in event ? event.touches[0].clientX : event.clientX;

  const handleMove = (e: MouseEvent | TouchEvent) => {
    if (!isDragging) return;

    const currentX = 'touches' in e ? e.touches[0].clientX : e.clientX;
    const deltaX = currentX - dragStartX;

    // Flip threshold: 50px drag distance
    if (Math.abs(deltaX) > 50) {
      isFlipped.value = deltaX < 0; // Drag left to flip, drag right to unflip
      stopDrag();
    }
  };

  const stopDrag = () => {
    isDragging = false;
    document.removeEventListener('mousemove', handleMove);
    document.removeEventListener('mouseup', stopDrag);
    document.removeEventListener('touchmove', handleMove);
    document.removeEventListener('touchend', stopDrag);
  };

  document.addEventListener('mousemove', handleMove);
  document.addEventListener('mouseup', stopDrag);
  document.addEventListener('touchmove', handleMove);
  document.addEventListener('touchend', stopDrag);
}
</script>

<style scoped>
.track-item {
  transition: background-color 0.2s ease;
}

.track-item:hover {
  background-color: rgba(var(--v-theme-surface-variant), 0.3);
}

.artist-link:hover {
  text-decoration: underline;
}

.artwork-card {
  cursor: pointer;
  transition: all 0.3s ease;
}

.artwork-card:hover {
  transform: translateY(-8px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.4) !important;
}

/* Flippable Album Cover */
.album-cover-container {
  perspective: 1000px;
  cursor: grab;
  user-select: none;
  position: relative;
}

.album-cover-container:active {
  cursor: grabbing;
}

.album-cover-flipper {
  position: relative;
  width: 100%;
  height: 100%;
  transition: transform 0.6s;
  transform-style: preserve-3d;
}

.album-cover-container.is-flipped .album-cover-flipper {
  transform: rotateY(180deg);
}

.album-cover-face {
  position: absolute;
  width: 100%;
  height: 100%;
  backface-visibility: hidden;
  -webkit-backface-visibility: hidden;
}

.album-cover-front {
  position: relative;
}

.album-cover-back {
  transform: rotateY(180deg);
}

</style>
