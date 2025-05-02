<template>
  <v-container
    fluid
    class="pa-0"
  >
    <template v-if="targetRelease">
      <video-player
        v-if="['video', 'movie'].includes(targetRelease.category)"
        :content-cid="targetRelease.contentCID"
      />
      <album-viewer
        v-else-if="['audio', 'music'].includes(targetRelease.category)"
        :content-cid="targetRelease.contentCID"
        :title="targetRelease.name"
        :thumbnail="targetRelease.thumbnail"
        :author="targetRelease.author"
        :description="(targetRelease.metadata['description'] as string | undefined)"
        :release-year="(targetRelease.metadata['releaseYear'] as string | number | undefined)"
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
import { computed, watch } from 'vue';
import { useReleasesStore } from '../stores/releases';
import { storeToRefs } from 'pinia';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();
const releasesStore = useReleasesStore();
const { releases, isLoading } = storeToRefs(releasesStore);

const targetRelease = computed(() => {
  return releases.value.find(r => r.id === props.id);
});

watch(targetRelease, (r) => {
  if (r) {
    if ('mediaSession' in navigator) {
      navigator.mediaSession.metadata = new MediaMetadata({
        title: r.name,
        artwork: r.thumbnail ? [
          {
            src: r.thumbnail,
            type: 'image/png',
          },
        ] : undefined,
      });
    }
  }
});

</script>
