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
import { useRouter } from 'vue-router';
import albumViewer from '/@/components/releases/albumViewer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import { watch } from 'vue';
import { ID_PROPERTY, RELEASE_CATEGORY_ID_PROPERTY, RELEASE_CONTENT_CID_PROPERTY, RELEASE_METADATA_PROPERTY, RELEASE_NAME_PROPERTY, RELEASE_THUMBNAIL_CID_PROPERTY, type AnyObject } from '@riffcc/lens-sdk';
import type { ReleaseItem } from '../types';
import { useLensService } from '../plugins/lensService/utils';
import { useQuery } from '@tanstack/vue-query';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();

const { lensService } = useLensService();
const {
  data: targetRelease,
  isLoading,
} = useQuery<ReleaseItem<AnyObject> | undefined>({
  queryKey: ['release', props.id],
  queryFn: async () => {
    const r = await lensService.getRelease(props.id);
    if (r) {
      return {
        [ID_PROPERTY]: r.id,
        [RELEASE_NAME_PROPERTY]: r.name,
        [RELEASE_CATEGORY_ID_PROPERTY]: r.categoryId,
        [RELEASE_CONTENT_CID_PROPERTY]: r.contentCID,
        [RELEASE_THUMBNAIL_CID_PROPERTY]: r.thumbnailCID,
        [RELEASE_METADATA_PROPERTY]: r.metadata ? JSON.parse(r.metadata) : undefined,
      };
    } else {
      return undefined;
    }
  },
});

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
