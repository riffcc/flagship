<template>
  <div
    class="d-flex flex-column align-center justify-center bg-black"
  >
    <v-hover
      :open-delay="150"
      :close-delay="150"
    >
      <template #default="{ isHovering, props: propsTemplate }">
        <v-container
          v-bind="propsTemplate"
          id="video-player"
          class="position-relative"
        >
          <v-btn
            v-if="isHovering"
            size="small"
            icon="fa:fas fa-arrow-left"
            class="position-absolute top-0 left-0 mt-2 ml-2"
            :style="{ zIndex: 1000 }"
            @click="router.back()"
          ></v-btn>

          <video
            ref="videoPlayerRef"
            autoplay
            class="w-100 h-100"
            :src="`https://${IPFS_GATEWAY}/ipfs/${props.contentCid}`"
            :controls="false"
            crossorigin="anonymous"
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
import { onMounted, onBeforeUnmount, ref } from 'vue';
import { useRouter } from 'vue-router';
import { IPFS_GATEWAY } from '/@/constants/ipfs';

const props = defineProps<{
  contentCid: string;
}>();

const router = useRouter();
const videoPlayerRef: Ref<HTMLVideoElement | null> = ref(null);
const isPlaying = ref(false);
const isLoading = ref(true);
const progress = ref(0);


const seekingTrack = (e: number): void => {
  console.log('seeking', e);
  if (!videoPlayerRef.value) return;
  isLoading.value = true;
  pause();
  videoPlayerRef.value.currentTime = e;
};


const togglePlay = (): void => {
  isPlaying.value ? pause() : play();
};

const toggleVolume = (): void => {
  videoPlayerRef.value && !(videoPlayerRef.value.volume > 0) ? unmute() : mute();
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
