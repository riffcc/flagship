<template>
  <v-sheet
    position="sticky"
    location="bottom right"
    class="w-auto border rounded-t-xl mx-3"
    :elevation="24"
    height="100px"
  >
    <audio
      ref="audioPlayerRef"
      class="d-none"
      crossorigin="anonymous"
      :src="`https://${IPFS_GATEWAY}/ipfs/${selectedAudio.cid}`"
      @loadeddata="play"
      @ended="handleNext"
    ></audio>
    <v-btn
      class="border position-absolute top-0 right-0 mt-n3 mr-n3"
      position="absolute"
      icon
      size="x-small"
      @click="props.onCloseCallback"
    >
      <v-icon
        icon="fas fa-close"
        size="x-small"
      ></v-icon>
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
            :size="xs ? 'x-small' : 'small'"
            icon="fas fa-backward-step"
            @click="props.handlePrevious"
          ></v-btn>
          <v-btn
            :size="xs ? 'small' : 'medium'"
            :icon="isPlaying ? 'fa fa-pause' : 'fas fa-play'"
            @click="togglePlay"
          >
          </v-btn>
          <v-btn
            :size="xs ? 'x-small' : 'small'"
            icon="fas fa-forward-step"
            @click="props.handleNext"
          ></v-btn>
        </div>
        <v-sheet
          color="transparent"
          class="flex-1-0 d-flex flex-column ml-4"
        >
          <p class="text-subtitle-2">{{ selectedAudio.name }}</p>
          <v-slider
            v-model="progress"
            :max="audioPlayerRef?.duration"
            track-fill-color="primary-darken-1"
            track-color="grey"
            thumb-color="white"
            :thumb-size="xs ? 14 : 18"
            color="background"
            class="mx-0"
            hide-details
            @update:model-value="seekingTrack"
          ></v-slider>
          <div>
            <span class="text-subtitle-2">{{ currentTime }}</span>
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
              icon="fas fa-bars"
              size="x-small"
              v-bind="speedDialProps"
            ></v-btn>
          </template>
          <v-btn
            :icon="volume === 0 ? 'fas fa-volume-off' : 'fas fa-volume-high'"
            :variant="volume !== 0 ? 'tonal' : 'plain'"
            size="x-small"
            color="#F2F2F2"
            @click="toggleVolume"
          ></v-btn>
          <v-btn
            icon="fas fa-rotate-left"
            :variant="props.repeat ? 'plain' : 'tonal'"
            size="x-small"
            color="#F2F2F2"
            @click="props.toggleRepeat"
          ></v-btn>
          <v-btn
            icon="fas fa-shuffle"
            :variant="props.shuffle ? 'plain' : 'tonal'"
            size="x-small"
            color="#F2F2F2"
            @click="props.toggleShuffle"
          ></v-btn>
        </v-speed-dial>
        <div
          v-else
          class="d-flex ml-6"
        >
          <v-btn
            :icon="volume === 0 ? 'fas fa-volume-off' : 'fas fa-volume-high'"
            :variant="volume !== 0 ? 'tonal' : 'flat'"
            size="x-small"
            @click="toggleVolume"
          ></v-btn>
          <v-btn
            class="mx-1"
            icon="fas fa-rotate-left"
            :variant="props.repeat ? 'tonal' : 'flat'"
            size="x-small"
            @click="props.toggleRepeat"
          ></v-btn>
          <v-btn
            class="mx-1"
            icon="fas fa-shuffle"
            :variant="props.shuffle ? 'tonal' : 'flat'"
            size="x-small"
            @click="props.toggleShuffle"
          ></v-btn>
        </div>
      </v-sheet>
    </v-container>
  </v-sheet>
</template>

<script setup lang="ts">
import {onUnmounted, ref, watch, onMounted} from 'vue';
import {IPFS_GATEWAY} from '/@/constants/ipfs';
import {formatTime} from '/@/utils';
import {useDisplay} from 'vuetify';

const props = defineProps<{
  selectedAudio: {
    cid: string;
    name: string;
  };
  onCloseCallback: () => void;
  handleNext: () => void;
  handlePrevious: () => void;
  repeat: boolean;
  toggleRepeat: () => void;
  shuffle: boolean;
  toggleShuffle: () => void;
}>();

const audioPlayerRef = ref<HTMLAudioElement>();
const isPlaying = ref(false);
const progress = ref(0);
const isLoading = ref(true);
const volume = ref(1);
const currentTime = ref('00:00');
const duration = ref('00:00');

const {xs} = useDisplay();

watch(volume, v => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.volume = v;
  }
});

const seekingTrack = (v: number) => {
  if (audioPlayerRef.value) {
    isLoading.value = true;
    pause();
    audioPlayerRef.value.currentTime = v;
  }
};

const togglePlay = () => (isPlaying.value ? pause() : play());

const toggleVolume = (): void => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.volume > 0 ? mute() : unmute();
  }
};

const mute = (): void => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.volume = 0;
    volume.value = 0;
  }
};

const unmute = (): void => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.volume = 1;
    volume.value = 1;
  }
};

const pause = () => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.pause();
    isPlaying.value = false;
  }
};

const play = () => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.play();
    isPlaying.value = true;
  }
};

const updateProgress = () => {
  if (audioPlayerRef.value) {
    currentTime.value = formatTime(audioPlayerRef.value.currentTime);
    duration.value = formatTime(audioPlayerRef.value.duration);

    progress.value = audioPlayerRef.value.currentTime;
    if (isPlaying.value) {
      requestAnimationFrame(updateProgress);
    }
  }
};

const canPlay = () => {
  isLoading.value = false;
  if (audioPlayerRef.value && audioPlayerRef.value.currentTime > 0) {
    audioPlayerRef.value?.play();
    isPlaying.value = true;
  }
};
// const onClosePlayer = () => {
//   pause()
//   isPlaying.value = false;
//   progress.value = 0
//   props.onCloseCallback()
// }

onMounted(() => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.addEventListener('progress', updateProgress);
    audioPlayerRef.value.addEventListener('canplay', canPlay);
  }
  if ('mediaSession' in window.navigator) {
    window.navigator.mediaSession.setActionHandler('play', () => {
      play();
    });
    window.navigator.mediaSession.setActionHandler('pause', () => {
      pause();
    });
    window.navigator.mediaSession.setActionHandler('previoustrack', () => {
      props.handlePrevious();
    });
    window.navigator.mediaSession.setActionHandler('nexttrack', () => {
      props.handleNext();
    });
  }
});

onUnmounted(() => {
  if (audioPlayerRef.value) {
    audioPlayerRef.value.removeEventListener('progress', updateProgress);
    audioPlayerRef.value.removeEventListener('canplay', canPlay);
  }
});
</script>
