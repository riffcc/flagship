<template>
  <v-hover
    :open-delay="150"
    :close-delay="150"
  >
    <template #default="{isHovering, props: propsTemplate}">
      <div
        v-bind="propsTemplate"
        :class="floating ? 'floating-container' : 'position-relative w-100'"
      >
        <v-btn
          v-if="isHovering"
          density="comfortable"
          :icon="floating ? '$close' : '$arrow-left'"
          class="position-absolute top-0 left-0 mt-3 ml-3"
          :style="{zIndex: 1000}"
          @click="floating ? closeFloatingVideo() : canBack ? router.back() : router.push('/')"
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
          :src="parseUrlOrCid(props.contentCid)"
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
          class="position-absolute bottom-0 w-100"
        >
          <v-slider
            v-model="progress"
            :class="floating ? '' : 'py-md-2'"
            track-fill-color="primary"
            thumb-color="white"
            thumb-size="16px"
            hide-details
            :max="videoPlayerRef?.duration"
            @update:model-value="seekingTrack"
          >
            <template #prepend>
              <v-btn
                :icon="isPlaying ? '$pause' : '$play'"
                density="comfortable"
                @click="togglePlay"
              ></v-btn>
            </template>

            <template #append>
              <v-sheet
                v-if="!floating"
                color="transparent"
                width="136px"
                class="d-flex justify-center ga-1 text-subtitle-2 pt-1"
              >
                <span>{{ currentTime }}</span>
                <span>/</span>
                <span>{{ duration }}</span>
              </v-sheet>
              <v-btn
                :icon="volume === 0 ? '$volume-off' : '$volume-high'"
                density="comfortable"
                @click="toggleVolume"
              ></v-btn>
              <v-btn
                icon="$fullscreen"
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
import {computed, onBeforeUnmount, onMounted, watch} from 'vue';
import {useRouter} from 'vue-router';
import {useDisplay} from 'vuetify';
import {useAudioAlbum} from '../../composables/audioAlbum';
import {useFloatingVideo} from '/@/composables/floatingVideo';
import {usePlaybackController} from '/@/composables/playbackController';
import {usePlayerVolume} from '/@/composables/playerVolume';
import { parseUrlOrCid } from '/@/utils';

const props = defineProps<{
  contentCid: string;
  floating?: boolean;
}>();

const router = useRouter();

const {
  playerRef: videoPlayerRef,
  progress,
  isLoading,
  isPlaying,
  currentTime,
  duration,
  seekingTrack,
  togglePlay,
  updateProgress,
  canPlay,
  play,
  pause,
  stop,
} = usePlaybackController();

const {volume, toggleVolume} = usePlayerVolume();
const {albumFiles, activeTrack} = useAudioAlbum();
const {floatingVideoSource, floatingVideoInitialTime, closeFloatingVideo} = useFloatingVideo();

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

const defaultSkipTime = 10;
onMounted((): void => {
  albumFiles.value = [];
  activeTrack.value = undefined;

  if (props.floating && floatingVideoInitialTime.value) {
    seekingTrack(floatingVideoInitialTime.value);
  } else {
    closeFloatingVideo();
  }
  if ('mediaSession' in navigator) {
    navigator.mediaSession.setActionHandler('play', play);
    navigator.mediaSession.setActionHandler('pause', pause);
    navigator.mediaSession.setActionHandler('stop', stop);
    navigator.mediaSession.setActionHandler('seekbackward', (details) => {
      const skipTime = details.seekOffset || defaultSkipTime;
      if (videoPlayerRef.value) {
        seekingTrack(Math.max(videoPlayerRef.value.currentTime - skipTime, 0));
      }
    });
    navigator.mediaSession.setActionHandler('seekforward', (details) => {
      const skipTime = details.seekOffset || defaultSkipTime;
      if (videoPlayerRef.value) {
        seekingTrack(Math.min(videoPlayerRef.value.currentTime + skipTime, videoPlayerRef.value.duration));
      }
    });
    navigator.mediaSession.setActionHandler('seekto', (details) => {
      if(details.seekTime) {
        seekingTrack(details.seekTime);
      }
    });
  }
});

onBeforeUnmount(() => {
  if (!props.floating && isPlaying.value && videoPlayerRef.value) {
    floatingVideoSource.value = props.contentCid;
    floatingVideoInitialTime.value = videoPlayerRef.value.currentTime;
  }
});
</script>

<style>
.floating-container {
  position: fixed;
  bottom: 0;
  right: 0;
  width: 384px;
  height: 216px;
  z-index: 5000;
  margin: 0px 8px 8px 0px;
}
</style>
