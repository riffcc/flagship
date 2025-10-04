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

    <!-- Artist not found -->
    <v-sheet
      v-else-if="!artist && !isLoading"
      color="transparent"
      class="d-flex flex-column mx-auto mt-8"
      max-width="20rem"
    >
      <v-icon
        size="64"
        class="mb-4 text-center"
        color="grey"
      >mdi-account-music</v-icon>
      <p class="text-h6 text-center mb-2">Artist not found</p>
      <p class="text-body-2 text-center text-grey mb-4">
        The artist you're looking for doesn't exist or has been removed.
      </p>
      <v-btn
        color="primary"
        @click="router.push('/music')"
      >
        Browse Music
      </v-btn>
    </v-sheet>

    <!-- Artist content -->
    <template v-else-if="artist">
      <!-- Artist header -->
      <v-row class="mb-6">
        <v-col cols="12" md="3">
          <v-img
            :src="parseUrlOrCid(artist.thumbnailCID)"
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
                <v-icon size="64" color="grey">mdi-account-music</v-icon>
              </v-sheet>
            </template>
          </v-img>
        </v-col>

        <v-col cols="12" md="9">
          <h1 class="text-h3 text-sm-h2 font-weight-bold mb-2">{{ artist.name }}</h1>

          <div class="d-flex align-center ga-4 mb-4">
            <v-chip v-if="artist.metadata?.genre" size="small" color="primary">
              {{ artist.metadata.genre }}
            </v-chip>
            <v-chip v-if="artist.metadata?.formed" size="small" color="secondary">
              {{ artist.metadata.formed }}
            </v-chip>
            <v-chip size="small" color="accent">
              {{ albums.length }} {{ albums.length === 1 ? 'Album' : 'Albums' }}
            </v-chip>
          </div>

          <p v-if="artist.metadata?.bio" class="text-body-1 mb-4">
            {{ artist.metadata.bio }}
          </p>
        </v-col>
      </v-row>

      <v-divider class="my-6" />

      <!-- Discography -->
      <div v-if="albums.length > 0">
        <h2 class="text-h5 font-weight-bold mb-4">Discography</h2>

        <v-row>
          <v-col
            v-for="album in albums"
            :key="album.id"
            cols="12"
            sm="6"
            md="4"
            lg="3"
          >
            <v-card
              class="album-card"
              rounded="lg"
              @click="router.push(`/album/${album.id}`)"
            >
              <v-img
                :src="parseUrlOrCid(album.thumbnailCID)"
                aspect-ratio="1"
                cover
              >
                <template #placeholder>
                  <v-sheet
                    color="grey-darken-3"
                    class="d-flex align-center justify-center fill-height"
                  >
                    <v-icon size="48" color="grey">mdi-album</v-icon>
                  </v-sheet>
                </template>
              </v-img>

              <v-card-text>
                <p class="text-body-2 font-weight-medium mb-1">
                  {{ album.name }}
                </p>
                <p v-if="album.metadata?.releaseYear" class="text-caption text-grey">
                  {{ album.metadata.releaseYear }}
                </p>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </div>

      <!-- No albums message -->
      <v-sheet
        v-else
        color="transparent"
        class="d-flex flex-column mx-auto mt-8"
        max-width="20rem"
      >
        <p class="text-body-1 text-center text-grey">
          No albums available for this artist.
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

// Fetch the artist release
const { data: artist, isLoading: isArtistLoading } = useGetReleaseQuery(props.id);

// Fetch all releases to get albums
const { data: releases, isLoading: isReleasesLoading } = useGetReleasesQuery({
  searchOptions: { fetch: 1000 }
});

const isLoading = computed(() =>
  isArtistLoading.value || isReleasesLoading.value
);

// Get all albums for this artist
const albums = computed<ReleaseItem[]>(() => {
  if (!releases.value || !props.id) return [];

  return releases.value
    .filter((r: ReleaseItem) => r.metadata?.artistId === props.id)
    .sort((a: ReleaseItem, b: ReleaseItem) => {
      const aYear = parseInt(a.metadata?.releaseYear || '0');
      const bYear = parseInt(b.metadata?.releaseYear || '0');
      return bYear - aYear; // Newest first
    });
});
</script>

<style scoped>
.album-card {
  cursor: pointer;
  transition: all 0.3s ease;
}

.album-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
}
</style>
