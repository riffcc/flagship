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
        :author="targetRelease.metadata?.author"
        :description="targetRelease.metadata?.description"
        :release-year="targetRelease.metadata?.releaseYear"
      ></album-viewer>
    </template>
    <div
      v-else
      class="d-flex flex-column align-center justify-center h-screen"
    >
      <p class="mb-2">Release not found.</p>
      <v-btn
        color="primary"
        @click="router.push('/')"
      >
        Go Home
      </v-btn>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import albumViewer from '/@/components/releases/albumViewer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import { useStaticReleases, type ItemContent } from '/@/composables/staticReleases';
import { onBeforeMount, type Ref, ref } from 'vue';

const props = defineProps<{
  id: string;
}>();
const router = useRouter();
const { staticReleases } = useStaticReleases();
const targetRelease: Ref<ItemContent | null> = ref(null);

onBeforeMount(() => {
  const _targetRelease = staticReleases.value.find(r => r.id === props.id);
  if (_targetRelease) {
    targetRelease.value = _targetRelease;
    if ('mediaSession' in navigator) {
      navigator.mediaSession.metadata = new MediaMetadata({
        title: _targetRelease.name,
        artwork: _targetRelease.thumbnail ? [
          {
            src: _targetRelease.thumbnail,
            type: 'image/png',
          },
        ] : undefined,
      });
  }
  }
});
</script>
