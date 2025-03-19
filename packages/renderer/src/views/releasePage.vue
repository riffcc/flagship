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
        :description="targetRelease.metadata.description"
        :release-year="(targetRelease.metadata as orbiterTypes.MusicReleaseMetadata).releaseYear"
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
import { useStaticReleases } from '/@/composables/staticReleases';
import { computed, onMounted, ref, watch } from 'vue';
import { useStaticStatus } from '../composables/staticStatus';
import {suivre as follow} from '@constl/vue';
import { useOrbiter } from '/@/plugins/orbiter/utils';
import type { ReleaseItem } from '/@/@types/release';
import type { types as orbiterTypes } from '@riffcc/orbiter';


const props = defineProps<{
  id: string;
}>();
const router = useRouter();
const {staticStatus} = useStaticStatus();
const { staticReleases } = useStaticReleases();
const {orbiter} = useOrbiter();
const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));
const targetRelease = computed(() => {
  let _targetRelease: ReleaseItem | undefined = undefined;
  if (staticStatus.value === 'static') {
    _targetRelease = staticReleases.value.find(r => r.id === props.id);
  } else {
    const otr = orbiterReleases.value?.find(r => r.release.id === props.id);
    if (otr) {
      _targetRelease = {
        id: otr.release.id,
        name: otr.release.release.contentName,
        contentCID: otr.release.release.file,
        category: otr.release.release.category,
        author: otr.release.release.author,
        thumbnail: otr.release.release.thumbnail as string,
        cover: otr.release.release.cover,
        metadata: JSON.parse(otr.release.release.metadata as string),
        sourceSite: otr.site,
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
