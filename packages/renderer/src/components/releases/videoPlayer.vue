<template>
  <v-hover
    :open-delay="150"
    :close-delay="150"
  >
    <template #default="{isHovering, props: propsTemplate}">
      <div
        v-bind="propsTemplate"
        class="position-relative w-100"
      >
        <v-btn
          v-if="isHovering"
          icon="mdi-arrow-left"
          class="position-absolute top-0 left-0 mt-3 ml-3"
          :style="{zIndex: 1000}"
          @click="canBack ? router.back() : router.push('/')"
        ></v-btn>
        <video
          ref="videoPlayerRef"
          autoplay
          :style="{
            maxHeight: `${displayHeight - 64}px`,
            width: '100%',
            height: '100%',
            objectFit: 'contain',
          }"
          :src="`https://${IPFS_GATEWAY}/ipfs/${props.contentCid}`"
          :controls="false"
          crossorigin="anonymous"
          @click="togglePlay"
          @loadeddata="play"
          @canplay="canPlay"
          @progress="updateProgress"
        ></video>

        <v-progress-circular
          v-if="isLoading"
          indeterminate
          size="32"
          class="position-absolute top-0 right-0 bottom-0 left-0 ma-auto"
        ></v-progress-circular>

        <v-sheet
          v-if="isHovering"
          class="position-absolute bottom-0 mb-2 py-md-2 w-100"
        >
          <v-slider
            v-model="progress"
            track-fill-color="primary"
            thumb-color="white"
            thumb-size="16px"
            hide-details
            :max="videoPlayerRef?.duration"
            @update:model-value="seekingTrack"
          >
            <template #prepend>
              <v-btn
                :icon="isPlaying ? 'mdi-pause' : 'mdi-play'"
                density="comfortable"
                @click="togglePlay"
              ></v-btn>
            </template>

            <template #append>
              <v-btn
                :icon="volume === 0 ? 'mdi-volume-off' : 'mdi-volume-high'"
                density="comfortable"
                @click="toggleVolume"
              ></v-btn>
              <v-btn
                icon="mdi-fullscreen"
                density="comfortable"
                @click="toggleFullscreen"
              ></v-btn>
            </template>
          </v-slider>
        </v-sheet>
      </div>
    </template>
  </v-hover>
</template>

<script setup lang="ts">
import {onMounted, computed, watch} from 'vue';
import {useRouter} from 'vue-router';
import {IPFS_GATEWAY} from '/@/constants/ipfs';
import {useDisplay} from 'vuetify';
import { usePlayerVolume } from '/@/composables/playerVolume';
import { useAudioAlbum } from '../../composables/audioAlbum';
import { usePlaybackController } from '/@/composables/playbackController';

const props = defineProps<{
  contentCid: string;
}>();

const router = useRouter();

const {
  playerRef: videoPlayerRef,
  progress,
  isLoading,
  isPlaying,
  seekingTrack,
  togglePlay,
  updateProgress,
  canPlay,
  play,
} = usePlaybackController();

const { volume, toggleVolume } = usePlayerVolume();
const {albumFiles, activeTrack} = useAudioAlbum();

watch(volume, v => {
  if (videoPlayerRef.value) {
    videoPlayerRef.value.volume = v;
  }
});

const {height: displayHeight} = useDisplay();

const canBack = computed(() => Boolean(window.history.state.back));

const toggleFullscreen = (): void => {
  if (!videoPlayerRef.value) return;
  videoPlayerRef.value.requestFullscreen();
};

onMounted((): void => {
  albumFiles.value = [];
  activeTrack.value = undefined;
});

</script>
