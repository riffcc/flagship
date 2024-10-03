<template>
  <div
    class="d-flex flex-column align-center justify-center bg-black"
  >
    <v-hover
      :open-delay="150"
      :close-delay="150"
    >
      <template #default="{ isHovering, props }">
        <v-container
          v-bind="props"
          id="video-player"
          class="position-relative"
        >
          <v-btn
            v-if="isHovering"
            size="small"
            icon="fa:fas fa-arrow-left"
            class="position-absolute top-0 left-0 mt-2 ml-2"
            @click="$router.push('/')"
          ></v-btn>

          <video
            ref="videoPlayerRef"
            autoplay
            class="w-100"
            :src="propsComponent.videoSource"
            width="100%"
            height="100%"
            :controls="false"
            crossorigin="anonymous"
            @loadeddata="onLoad"
            @click="togglePlay"
          ></video>

          <v-progress-circular
            v-if="isLoading"
            indeterminate
            size="32"
            class="position-absolute top-0 right-0 bottom-0 left-0 ma-auto"
          ></v-progress-circular>

          <div
            v-if="isHovering"
            class="position-absolute left-0 bottom-0 w-100"
          >
            <v-slider
              v-model="progress"
              track-fill-color="primary"
              thumb-color="white"
              thumb-size="16px"
              class="pt-4"
              :max="videoPlayerRef?.duration"
              @update:model-value="seekingTrack"
            >
              <template #prepend>
                <v-btn
                  icon
                  size="small"
                  @click="togglePlay"
                >
                  <v-icon
                    v-if="isPlaying"
                    icon="fa:fas fa-pause"
                  >
                  </v-icon>
                  <v-icon
                    v-else
                    icon="fa:fas fa-play"
                  >
                  </v-icon>
                </v-btn>
              </template>

              <template #append>
                <v-btn
                  icon
                  size="small"
                  @click="toggleVolume"
                >
                  <v-icon
                    v-if="videoPlayerRef && !(videoPlayerRef.volume > 0)"
                    icon="fa:fas fa-volume-off"
                  ></v-icon>
                  <v-icon
                    v-else
                    icon="fa:fas fa-volume-high"
                  ></v-icon>
                </v-btn>

                <v-btn
                  icon
                  size="small"
                  @click="toggleFullscreen"
                >
                  <v-icon icon="fa:fas fa-expand"></v-icon>
                </v-btn>
              </template>
            </v-slider>
          </div>
        </v-container>
      </template>
    </v-hover>
  </div>
</template>

<script setup lang="ts">
import type { Ref } from 'vue';
import { onMounted, onBeforeUnmount, ref, watch } from 'vue';

const propsComponent = defineProps<{
  videoSource: string;
}>();

const videoPlayerRef: Ref<HTMLVideoElement | null> = ref(null);
const isPlaying = ref(false);
const isLoading = ref(true);
const videoDuration = ref(0);
const progress = ref(0);


const seekingTrack = (e: number): void => {
  console.log('seeking', e);
  if (!videoPlayerRef.value) return;
  isLoading.value = true;
  pause();
  // const targetPercent = parseFloat((e.offsetX / (e.target as HTMLElement).parentElement!.offsetWidth).toFixed(4));
  // const targetCurrentTime = videoPlayerRef.value.duration * targetPercent;
  videoPlayerRef.value.currentTime = e;
};


const togglePlay = (): void => {
  isPlaying.value ? pause() : play();
};

const toggleVolume = (): void => {
  videoPlayerRef.value && !(videoPlayerRef.value.volume > 0) ? unmute() : mute();
};

const onLoad = (e: Event): void => {
  console.log(e);
  // play();
};

const pause = (): void => {
  if (!videoPlayerRef.value) return;
  videoPlayerRef.value.pause();
  isPlaying.value = false;
};


const mute = (): void => {
  if (!videoPlayerRef.value) return;
  console.log('mute');
  videoPlayerRef.value.volume = 0;
};

const unmute = (): void => {
  if (!videoPlayerRef.value) return;
  console.log('unmute');
  videoPlayerRef.value.volume = 1;
};

const play = (): void => {
  if (!videoPlayerRef.value) return;
  videoPlayerRef.value.play();
  isPlaying.value = true;
};

const updateProgress = (): void => {
  if (!videoPlayerRef.value) return;
  progress.value = videoPlayerRef.value.currentTime;
  if (isPlaying.value) {
    requestAnimationFrame(updateProgress);
  }
};

const toggleFullscreen = (): void => {
  if (!videoPlayerRef.value) return;
  videoPlayerRef.value.requestFullscreen();
};

watch(
  () => propsComponent.videoSource,
  () => {
    if (videoPlayerRef.value) {
      videoPlayerRef.value.load();
      videoPlayerRef.value.addEventListener('loadedmetadata', () => {
        videoDuration.value = videoPlayerRef.value!.duration;
      });
    }
  },
);

const canPlay = () => {
  isLoading.value = false;
  if (videoPlayerRef.value && videoPlayerRef.value.currentTime > 0) {
    videoPlayerRef.value?.play();
    isPlaying.value = true;
  }
};
onMounted((): void => {
  if (videoPlayerRef.value) {
    videoPlayerRef.value.addEventListener('progress', updateProgress);
    videoPlayerRef.value.addEventListener('canplay', canPlay);

  }
});

onBeforeUnmount((): void => {
  if (!videoPlayerRef.value) return;
  videoPlayerRef.value.removeEventListener('progress', updateProgress);
  videoPlayerRef.value.removeEventListener('canplay', canPlay);

});
</script>
