<template>
  <v-container
    fluid
    class="pa-0"
  >
    <template v-if="displayRelease">
      <video-player
        v-if="['videos', 'movies', 'tv-shows'].includes(categorySlug)"
        :content-cid="displayRelease.contentCID"
        :release-id="displayRelease.id"
        :release-name="displayRelease.name"
      />
      <!-- Always render albumViewer - pass pre-parsed tracks for instant display -->
      <album-viewer
        v-else
        :key="`release-${props.id}`"
        :release="displayRelease"
      ></album-viewer>
    </template>

    <div
      v-else
      class="d-flex align-center justify-center h-screen"
    >
      <v-sheet
        color="transparent"
        class="d-flex flex-column mx-auto"
        max-width="16rem"
      >
        <template v-if="isLoading">
          <v-progress-circular
            indeterminate
            color="primary"
          ></v-progress-circular>
        </template>
        <template v-else>
          <p class="text-white text-center mb-2">Release not found.</p>
          <v-btn
            color="primary-darken-1"
            @click="router.push('/')"
          >
            Go to Home
          </v-btn>
        </template>
      </v-sheet>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import { watch, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useQueryClient } from '@tanstack/vue-query';
import albumViewer from '/@/components/releases/albumViewer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import { useGetReleaseQuery } from '/@/plugins/lensService/hooks';
import type { ReleaseItem } from '/@/types';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();
const queryClient = useQueryClient();
const { data: targetRelease, isLoading } = useGetReleaseQuery(props.id);

// Try to get release from cached releases list (from category/home pages)
const cachedRelease = computed(() => {
  const releases = queryClient.getQueryData<ReleaseItem[]>(['releases']);
  return releases?.find(r => r.id === props.id) || null;
});

// Read tile data synchronously from history.state for instant render
const tileData = history.state?.name ? {
  id: props.id,
  name: history.state.name,
  thumbnailCID: history.state.thumbnailCID || '',
  contentCID: history.state.contentCID || '',
  categoryId: history.state.categoryId || 'music',
  categorySlug: history.state.categorySlug || 'music',
  metadata: {
    author: history.state.author,
    artistId: history.state.artistId,
    releaseYear: history.state.releaseYear,
    trackCount: history.state.trackCount,
  },
} : null;

// Compute trackCount from trackMetadata if not explicitly set
function getTrackCount(metadata: any): number | undefined {
  if (metadata?.trackCount) return metadata.trackCount;
  if (metadata?.trackMetadata) {
    try {
      const tracks = typeof metadata.trackMetadata === 'string'
        ? JSON.parse(metadata.trackMetadata)
        : metadata.trackMetadata;
      if (Array.isArray(tracks)) return tracks.length;
    } catch (e) { /* ignore */ }
  }
  return undefined;
}

// Parse trackMetadata into track objects
function parseTrackMetadata(metadata: any): Array<{title: string; artist?: string; duration?: string}> | null {
  if (!metadata?.trackMetadata) return null;
  try {
    const tracks = typeof metadata.trackMetadata === 'string'
      ? JSON.parse(metadata.trackMetadata)
      : metadata.trackMetadata;
    return Array.isArray(tracks) ? tracks : null;
  } catch (e) {
    return null;
  }
}

// Use real data when available, fall back to cached release, then tile data
// Priority: fetched single release > cached from releases list > tile data from navigation
// ALWAYS ensure trackCount is populated and tracks are pre-parsed
const displayRelease = computed(() => {
  const release = targetRelease.value || cachedRelease.value || tileData;
  if (!release) return null;

  // Parse metadata if string
  const metadata = typeof release.metadata === 'string'
    ? JSON.parse(release.metadata)
    : release.metadata;

  // Pre-parse tracks for instant rendering
  const tracks = parseTrackMetadata(metadata);
  const trackCount = tracks?.length || getTrackCount(metadata);

  // Return release with guaranteed trackCount AND pre-parsed tracks
  return {
    ...release,
    metadata: {
      ...metadata,
      trackCount,
      // Pre-parsed tracks ready for instant display - no parsing needed downstream
      _parsedTracks: tracks,
    }
  };
});

const categorySlug = computed(() => {
  return targetRelease.value?.categorySlug || cachedRelease.value?.categorySlug || tileData?.categorySlug || 'music';
});

watch(targetRelease, (r) => {
  if (r) {
    if ('mediaSession' in navigator) {
      try {
        // Check both 'artist' and 'author' fields for compatibility
        const artistName = r.metadata?.artist || r.metadata?.author;
        navigator.mediaSession.metadata = new MediaMetadata({
          title: r.name,
          artist: artistName as string | undefined,
          album: r.metadata?.albumName as string || '',
          artwork: r.thumbnailCID ? [
            {
              src: r.thumbnailCID,
            },
          ] : undefined,
        });
      } catch (error) {
        console.error('Failed to set MediaMetadata:', error);
      }
    }
  } else {
    if ('mediaSession' in navigator) {
      navigator.mediaSession.metadata = null;
    }
  }
}, { immediate: true });

</script>
