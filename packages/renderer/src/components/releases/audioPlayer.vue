<template>
  <v-sheet
    position="sticky"
    location="bottom right"
    class="w-100 border rounded-t-xl mx-auto"
    :elevation="24"
    height="100px"
    max-width="960px"
  >
    <audio
      ref="audioPlayerRef"
      class="d-none"
      crossorigin="anonymous"
      :src="parseUrlOrCid(activeTrack?.cid)"
      @ended="handleNext"
      @loadeddata="play"
      @canplay="canPlay"
      @progress="updateProgress"
    ></audio>
    <v-btn
      :class="
        smAndDown
          ? 'border position-absolute top-0 right-0 left-0 mx-auto mt-n6'
          : 'border position-absolute top-0 right-0 mt-n2 mr-n2'
      "
      density="comfortable"
      icon="$close"
      size="small"
      @click="close"
    >
    </v-btn>
    <v-container class="fill-height">
      <v-sheet
        color="transparent"
        height="100%"
        max-width="920px"
        class="d-flex align-center w-100"
      >
        <div class="d-flex align-center">
          <v-btn
            :size="xs ? 'small' : 'large'"
            density="comfortable"
            icon="$skip-previous"
            @click="handlePrevious"
          ></v-btn>
          <v-btn
            :size="xs ? 'default' : 'x-large'"
            density="comfortable"
            :icon="isPlaying ? '$pause-circle' : '$play-circle'"
            :loading="isLoading"
            @click="togglePlay"
          >
          </v-btn>
          <v-btn
            :size="xs ? 'small' : 'large'"
            density="comfortable"
            icon="$skip-next"
            @click="handleNext"
          ></v-btn>
        </div>
        <v-sheet
          color="transparent"
          class="flex-1-0 d-flex flex-column px-2 px-md-4"
        >
          <p class="text-subtitle-2">{{ activeTrack?.title }}</p>
          <v-slider
            v-model="progress"
            :max="audioPlayerRef?.duration"
            track-fill-color="primary"
            track-color="grey"
            thumb-color="white"
            :thumb-size="xs ? 14 : 16"
            color="background"
            class="mx-0"
            hide-details
            @update:model-value="seekingTrack"
          ></v-slider>
          <div>
            <span class="text-subtitle-2 float-left">{{ currentTime }}</span>
            <span class="text-subtitle-2 float-right">{{ duration }}</span>
          </div>
        </v-sheet>
        <v-speed-dial
          v-if="xs"
          location="top center"
        >
          <template #activator="{props: speedDialProps}">
            <v-btn
              class="mx-2"
              icon="$dots-vertical"
              density="comfortable"
              size="small"
              v-bind="speedDialProps"
            ></v-btn>
          </template>
          <v-btn
            :icon="volume === 0 ? '$volume-off' : '$volume-high'"
            size="small"
            density="comfortable"
            @click="toggleVolume"
          ></v-btn>
          <v-btn
            icon="$rotate-left"
            :color="repeat ? 'grey-lighten-3' : 'default'"
            size="small"
            density="comfortable"
            @click="toggleRepeat"
          ></v-btn>
          <v-btn
            icon="$shuffle"
            :color="shuffle ? 'grey-lighten-3' : 'default'"
            size="small"
            density="comfortable"
            @click="toggleShuffle"
          ></v-btn>
        </v-speed-dial>
        <div
          v-else
          class="d-flex ga-1"
        >
          <v-btn
            :icon="volume === 0 ? '$volume-off' : '$volume-high'"
            size="small"
            density="comfortable"
            @click="toggleVolume"
          ></v-btn>
          <v-btn
            icon="$rotate-left"
            :color="repeat ? 'grey-lighten-3' : 'default'"
            density="comfortable"
            size="small"
            @click="toggleRepeat"
          ></v-btn>
          <v-btn
            icon="$shuffle"
            :color="shuffle ? 'grey-lighten-3' : 'default'"
            density="comfortable"
            size="small"
            @click="toggleShuffle"
          ></v-btn>
        </div>
      </v-sheet>
    </v-container>
  </v-sheet>
</template>

<script setup lang="ts">
import {onMounted, watch} from 'vue';
import {useDisplay} from 'vuetify';
import {useAudioAlbum} from '/@/composables/audioAlbum';
import {usePlaybackController} from '/@/composables/playbackController';
import {usePlayerVolume} from '/@/composables/playerVolume';
import { parseUrlOrCid } from '/@/utils';

const {xs, smAndDown} = useDisplay();

const {
  playerRef: audioPlayerRef,
  currentTime,
  duration,
  progress,
  isLoading,
  isPlaying,
  seekingTrack,
  togglePlay,
  updateProgress,
  canPlay,
  play,
  pause,
  stop,
} = usePlaybackController<HTMLAudioElement>();

const {
  activeTrack,
  repeat,
  shuffle,
  handlePrevious,
  handleNext,
  handleOnClose,
  toggleRepeat,
  toggleShuffle,
} = useAudioAlbum();

const {volume, toggleVolume} = usePlayerVolume();

watch(volume, v => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.volume = v;
  }
});

const close = () => {
  pause();
  progress.value = 0;
  handleOnClose();
};

const defaultSkipTime = 10;
onMounted(() => {
  if ('mediaSession' in navigator) {
  navigator.mediaSession.setActionHandler('play', play);
  navigator.mediaSession.setActionHandler('pause', pause);
  navigator.mediaSession.setActionHandler('stop', stop);
  navigator.mediaSession.setActionHandler('seekbackward', (details) => {
    const skipTime = details.seekOffset || defaultSkipTime;
    if (audioPlayerRef.value) {
      seekingTrack(Math.max(audioPlayerRef.value.currentTime - skipTime, 0));
    }
  });
  navigator.mediaSession.setActionHandler('seekforward', (details) => {
    const skipTime = details.seekOffset || defaultSkipTime;
    if (audioPlayerRef.value) {
      seekingTrack(Math.min(audioPlayerRef.value.currentTime + skipTime, audioPlayerRef.value.duration));
    }
  });
  navigator.mediaSession.setActionHandler('seekto', (details) => {
    if(details.seekTime) {
      seekingTrack(details.seekTime);
    }
  });
  navigator.mediaSession.setActionHandler('previoustrack', handlePrevious);
  navigator.mediaSession.setActionHandler('nexttrack', handleNext);
}
});
</script>
