<template>
  <v-container
    fluid
    class="pa-0"
  >
    <template v-if="targetRelease">
      <video-player
        v-if="['video', 'movie'].includes(targetRelease.categoryId)"
        :content-cid="targetRelease.contentCID"
      />
      <album-viewer
        v-else-if="['audio', 'music'].includes(targetRelease.categoryId)"
        :release="targetRelease"
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
import { watch } from 'vue';
import { useRouter } from 'vue-router';
import albumViewer from '/@/components/releases/albumViewer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import { useGetReleaseQuery } from '/@/plugins/lensService/hooks';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

const { data: targetRelease, isLoading } = useGetReleaseQuery(props);

watch(targetRelease, (r) => {
  if (r) {
    if ('mediaSession' in navigator) {
      try {
        navigator.mediaSession.metadata = new MediaMetadata({
          title: r.name,
          artist: r.metadata?.['author'] as string | undefined,
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
