<template>
  <v-sheet
    position="sticky"
    location="bottom right"
    class="w-auto border rounded-t-xl mx-auto"
    :elevation="24"
    height="100px"
    max-width="960px"
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
      class="border position-absolute top-0 right-0 mt-n1 mr-n1"
      density="comfortable"
      icon="mdi-close"
      size="x-small"
      @click="props.onCloseCallback"
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
            :size="xs ? 'small' : 'default'"
            density="comfortable"
            icon="mdi-skip-previous"
            @click="props.handlePrevious"
          ></v-btn>
          <v-btn
            :size="xs ? 'default' : 'large'"
            density="comfortable"
            :icon="isPlaying ? 'mdi-pause-circle' : 'mdi-play-circle'"
            @click="togglePlay"
          >
          </v-btn>
          <v-btn
            :size="xs ? 'small' : 'default'"
            density="comfortable"
            icon="mdi-skip-next"
            @click="props.handleNext"
          ></v-btn>
        </div>
        <v-sheet
          color="transparent"
          class="flex-1-0 d-flex flex-column px-2 px-md-4"
        >
          <p class="text-subtitle-2">{{ selectedAudio.name }}</p>
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
              icon="mdi-dots-vertical"
              density="comfortable"
              size="small"
              v-bind="speedDialProps"
            ></v-btn>
          </template>
          <v-btn
            :icon="volume === 0 ? 'mdi-volume-off' : 'mdi-volume-high'"
            size="small"
            density="comfortable"
            color="#F2F2F2"
            @click="toggleVolume"
          ></v-btn>
          <v-btn
            icon="mdi-rotate-left"
            :variant="props.repeat ? 'elevated' : 'outlined'"
            size="small"
            density="comfortable"
            color="#F2F2F2"
            @click="props.toggleRepeat"
          ></v-btn>
          <v-btn
            icon="mdi-shuffle"
            :variant="props.shuffle ? 'elevated' : 'outlined'"
            size="small"
            density="comfortable"
            color="#F2F2F2"
            @click="props.toggleShuffle"
          ></v-btn>
        </v-speed-dial>
        <div
          v-else
          class="d-flex"
        >
          <v-btn
            :icon="volume === 0 ? 'mdi-volume-off' : 'mdi-volume-high'"
            size="small"
            @click="toggleVolume"
          ></v-btn>
          <v-btn
            icon="mdi-rotate-left"
            :variant="props.repeat ? 'tonal' : 'flat'"
            size="small"
            @click="props.toggleRepeat"
          ></v-btn>
          <v-btn
            icon="mdi-shuffle"
            :variant="props.shuffle ? 'tonal' : 'flat'"
            size="small"
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
import { usePlayerVolume } from '/@/composables/playerVolume';
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
const currentTime = ref('00:00');
const duration = ref('00:00');
const { volume, toggleVolume } = usePlayerVolume();
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

const togglePlay = () => isPlaying.value ? pause() : play();


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
    requestAnimationFrame(updateProgress);

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
