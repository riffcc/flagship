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
      <template v-if="isLoading">
        <v-progress-circular
          indeterminate
          size="36"
        ></v-progress-circular>
      </template>
      <template v-else>
        <p class="mb-2">Release not found.</p>
        <v-btn
          color="primary"
          @click="router.push('/')"
        >
          Go Home
        </v-btn>
      </template>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import albumViewer from '/@/components/releases/albumViewer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import type { ItemMetadata, ItemStatus, ItemContent} from '/@/composables/staticReleases';
import { useStaticReleases } from '/@/composables/staticReleases';
import { computed, onMounted, ref, watch } from 'vue';
import { useDevStatus } from '/@/composables/devStatus';
import {suivre as follow} from '@constl/vue';
import { useOrbiter } from '/@/plugins/orbiter/utils';


const props = defineProps<{
  id: string;
}>();
const router = useRouter();
const {status} = useDevStatus();
const { staticReleases } = useStaticReleases();
const {orbiter} = useOrbiter();
const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));
const targetRelease = computed(() => {
  let _targetRelease: ItemContent | undefined = undefined;
  if (status.value === 'static') {
    _targetRelease = staticReleases.value.find(r => r.id === props.id);
  } else {
    const otr = orbiterReleases.value?.find(r => r.release.id === props.id);
    if (otr) {
      _targetRelease = {
        id: otr.release.id,
        category: otr.release.release.category,
        contentCID: otr.release.release.file,
        name: otr.release.release.contentName,
        metadata: JSON.parse(otr.release.release.metadata as string) as ItemMetadata,
        thumbnail: otr.release.release.thumbnail as string,
        sourceSite: otr.site,
        status: otr.release.release.status as ItemStatus,
        cover: otr.release.release.cover,
      };
    }
  }
  return _targetRelease;
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

const isLoading = ref(true);

onMounted(() => {
  setTimeout(() => {
    isLoading.value = false;
  }, 6000);
});
</script>
