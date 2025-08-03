<template>
  <v-hover
    :open-delay="150"
    :close-delay="150"
  >
    <template #default="{isHovering, props: propsTemplate}">
      <div
        v-bind="propsTemplate"
        ref="containerRef"
        :class="floating ? 'floating-container' : 'position-relative w-100'"
        :data-navigable="true"
        @dblclick="navigateBack"
        @mousedown="startDrag"
      >
        <v-btn
          v-if="isHovering && !floating"
          density="comfortable"
          :icon="'$arrow-left'"
          class="position-absolute top-0 left-0 mt-3 ml-3"
          :style="{zIndex: 1000}"
          @click="canBack ? router.back() : router.push('/')"
        ></v-btn>
        
        <!-- Floating video controls -->
        <template v-if="floating">
          <!-- Close button -->
          <v-btn
            v-if="isHovering"
            density="comfortable"
            icon="$close"
            class="position-absolute top-0 right-0 mt-2 mr-2"
            :style="{zIndex: 1000}"
            size="small"
            @click="closeFloatingVideo()"
          ></v-btn>
          
          <!-- Now Playing bar -->
          <v-sheet
            v-if="isHovering"
            color="rgba(0, 0, 0, 0.8)"
            class="position-absolute top-0 w-100 d-flex align-center px-2"
            height="32"
            :style="{zIndex: 999}"
          >
            <span class="text-caption text-white text-truncate flex-grow-1">
              Now Playing: {{ props.releaseName || floatingVideoRelease?.name || 'Video' }}
            </span>
            <v-btn
              v-if="props.releaseId || floatingVideoRelease?.id"
              icon="$chevron-right"
              density="compact"
              size="x-small"
              variant="text"
              @click="navigateToRelease"
            ></v-btn>
          </v-sheet>
          
          <!-- Resize handles -->
          <template v-if="floating">
            <div
              class="resize-handle resize-handle-nw"
              @mousedown="startResize('nw', $event)"
            ></div>
            <div
              class="resize-handle resize-handle-ne"
              @mousedown="startResize('ne', $event)"
            ></div>
            <div
              class="resize-handle resize-handle-sw"
              @mousedown="startResize('sw', $event)"
            ></div>
            <div
              class="resize-handle resize-handle-se"
              @mousedown="startResize('se', $event)"
            ></div>
          </template>
          
          <!-- Center play arrow -->
          <v-btn
            v-if="isHovering && !isPlaying"
            icon="$arrow-right-circle"
            size="x-large"
            variant="tonal"
            class="position-absolute"
            :style="{
              top: '50%',
              left: '50%',
              transform: 'translate(-50%, -50%)',
              zIndex: 1000
            }"
            @click="navigateToRelease"
          ></v-btn>
        </template>
        <video
          ref="videoPlayerRef"
          autoplay
          :style="{
            maxHeight: floating ? '100%' : `${displayHeight - 64}px`,
            width: '100%',
            height: '100%',
            objectFit: floating ? 'cover' : 'contain',
            position: 'relative',
            top: 0,
            left: 0,
            backgroundColor: '#0a0a0a',
          }"
          :src="parseUrlOrCid(props.contentCid)"
          :controls="false"
          crossorigin="anonymous"
          @click="togglePlay"
          @loadeddata="onVideoLoaded"
          @canplay="canPlay"
          @progress="updateProgress"
          @error="onVideoError"
          @loadedmetadata="onLoadedMetadata"
        ></video>

        <!-- Codec error overlay -->
        <v-sheet
          v-if="codecError"
          color="rgba(0, 0, 0, 0.9)"
          class="position-absolute d-flex align-center justify-center"
          :style="{
            top: floating ? '32px' : 0,
            left: 0,
            right: 0,
            bottom: 0,
            zIndex: 998
          }"
        >
          <div class="text-center pa-6">
            <v-icon
              size="64"
              color="warning"
              class="mb-4"
            >mdi-alert-circle-outline</v-icon>
            <h3 class="text-h6 mb-2">Unable to play this video</h3>
            <p class="text-body-2 text-medium-emphasis">
              We don't appear to support playing this video in your browser.<br>
              Stay tuned, we'll hopefully fix it soon.
            </p>
            <p class="text-caption text-medium-emphasis mt-4">
              {{ codecErrorDetails }}
            </p>
          </div>
        </v-sheet>

        <!-- Loading overlay -->
        <v-sheet
          v-if="isLoading && !codecError"
          :color="floating ? '#0a0a0a' : 'transparent'"
          class="position-absolute d-flex align-center justify-center"
          :style="{
            top: floating ? '32px' : '0',
            left: 0,
            right: 0,
            bottom: 0,
            zIndex: 998
          }"
        >
          <v-progress-circular
            indeterminate
            size="48"
            width="3"
            color="primary"
          ></v-progress-circular>
        </v-sheet>

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
            :data-navigable="true"
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
import {computed, onBeforeUnmount, onMounted, watch, ref} from 'vue';
import {useRouter, useRoute} from 'vue-router';
import {useDisplay} from 'vuetify';
import {useAudioAlbum} from '../../composables/audioAlbum';
import {useFloatingVideo} from '/@/composables/floatingVideo';
import {usePlaybackController} from '/@/composables/playbackController';
import {usePlayerVolume} from '/@/composables/playerVolume';
import { parseUrlOrCid } from '/@/utils';

const props = defineProps<{
  contentCid: string;
  floating?: boolean;
  releaseId?: string;
  releaseName?: string;
}>();

const router = useRouter();
const route = useRoute();
const containerRef = ref<HTMLElement>();

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
const {floatingVideoSource, floatingVideoInitialTime, floatingVideoRelease, closeFloatingVideo} = useFloatingVideo();

// Dragging state
const isDragging = ref(false);
const isResizing = ref(false);
const resizeDirection = ref('');
const dragStart = ref({ x: 0, y: 0 });
const containerPosition = ref({ x: 0, y: 0 });
const containerSize = ref({ width: 384, height: 216 });
const initialSize = ref({ width: 0, height: 0 });
const initialPosition = ref({ x: 0, y: 0 });

// Codec error handling
const codecError = ref(false);
const codecErrorDetails = ref('');

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

const navigateBack = () => {
  router.back();
};

const navigateToRelease = () => {
  const releaseId = props.releaseId || floatingVideoRelease.value?.id || (route.name === 'Release' ? route.params.id as string : undefined);
  if (releaseId) {
    // Close floating video before navigating
    closeFloatingVideo();
    router.push(`/release/${releaseId}`);
  }
};

// Drag handlers
const startDrag = (e: MouseEvent) => {
  if (!props.floating || !containerRef.value || isResizing.value) return;
  
  const rect = containerRef.value.getBoundingClientRect();
  isDragging.value = true;
  dragStart.value = { x: e.clientX - rect.left, y: e.clientY - rect.top };
  containerPosition.value = { x: rect.left, y: rect.top };
  
  document.addEventListener('mousemove', handleDrag);
  document.addEventListener('mouseup', stopDrag);
};

const startResize = (direction: string, e: MouseEvent) => {
  if (!props.floating || !containerRef.value) return;
  
  e.stopPropagation();
  e.preventDefault();
  
  const rect = containerRef.value.getBoundingClientRect();
  isResizing.value = true;
  resizeDirection.value = direction;
  dragStart.value = { x: e.clientX, y: e.clientY };
  initialSize.value = { width: rect.width, height: rect.height };
  initialPosition.value = { x: rect.left, y: rect.top };
  
  document.addEventListener('mousemove', handleResize);
  document.addEventListener('mouseup', stopDrag);
};

const handleDrag = (e: MouseEvent) => {
  if (!containerRef.value || !isDragging.value || isResizing.value) return;
  
  const newX = Math.max(0, Math.min(window.innerWidth - containerRef.value.offsetWidth, e.clientX - dragStart.value.x));
  const newY = Math.max(0, Math.min(window.innerHeight - containerRef.value.offsetHeight, e.clientY - dragStart.value.y));
  
  containerRef.value.style.left = `${newX}px`;
  containerRef.value.style.top = `${newY}px`;
  containerRef.value.style.right = 'auto';
  containerRef.value.style.bottom = 'auto';
};

const handleResize = (e: MouseEvent) => {
  if (!isResizing.value || !containerRef.value) return;
  
  const deltaX = e.clientX - dragStart.value.x;
  const deltaY = e.clientY - dragStart.value.y;
  
  let newWidth = initialSize.value.width;
  let newHeight = initialSize.value.height;
  let newX = initialPosition.value.x;
  let newY = initialPosition.value.y;
  
  // Minimum sizes
  const minWidth = 320;
  const minHeight = 180;
  
  // Handle resizing based on direction
  switch (resizeDirection.value) {
    case 'se': // Southeast (bottom-right)
      newWidth = Math.max(minWidth, initialSize.value.width + deltaX);
      newHeight = Math.max(minHeight, initialSize.value.height + deltaY);
      break;
    case 'sw': // Southwest (bottom-left)
      const swWidth = Math.max(minWidth, initialSize.value.width - deltaX);
      if (swWidth !== initialSize.value.width - deltaX) {
        newX = initialPosition.value.x + (initialSize.value.width - minWidth);
      } else {
        newX = initialPosition.value.x + deltaX;
      }
      newWidth = swWidth;
      newHeight = Math.max(minHeight, initialSize.value.height + deltaY);
      break;
    case 'ne': // Northeast (top-right)
      newWidth = Math.max(minWidth, initialSize.value.width + deltaX);
      const neHeight = Math.max(minHeight, initialSize.value.height - deltaY);
      if (neHeight !== initialSize.value.height - deltaY) {
        newY = initialPosition.value.y + (initialSize.value.height - minHeight);
      } else {
        newY = initialPosition.value.y + deltaY;
      }
      newHeight = neHeight;
      break;
    case 'nw': // Northwest (top-left)
      const nwWidth = Math.max(minWidth, initialSize.value.width - deltaX);
      const nwHeight = Math.max(minHeight, initialSize.value.height - deltaY);
      if (nwWidth !== initialSize.value.width - deltaX) {
        newX = initialPosition.value.x + (initialSize.value.width - minWidth);
      } else {
        newX = initialPosition.value.x + deltaX;
      }
      if (nwHeight !== initialSize.value.height - deltaY) {
        newY = initialPosition.value.y + (initialSize.value.height - minHeight);
      } else {
        newY = initialPosition.value.y + deltaY;
      }
      newWidth = nwWidth;
      newHeight = nwHeight;
      break;
  }
  
  // Constrain to window bounds
  newX = Math.max(0, Math.min(window.innerWidth - newWidth, newX));
  newY = Math.max(0, Math.min(window.innerHeight - newHeight, newY));
  
  // Update styles
  containerRef.value.style.width = `${newWidth}px`;
  containerRef.value.style.height = `${newHeight}px`;
  containerRef.value.style.left = `${newX}px`;
  containerRef.value.style.top = `${newY}px`;
  containerRef.value.style.right = 'auto';
  containerRef.value.style.bottom = 'auto';
};

const stopDrag = () => {
  isDragging.value = false;
  isResizing.value = false;
  resizeDirection.value = '';
  document.removeEventListener('mousemove', handleDrag);
  document.removeEventListener('mousemove', handleResize);
  document.removeEventListener('mouseup', stopDrag);
};

const defaultSkipTime = 10;

// Video event handlers
const onVideoLoaded = () => {
  // Reset codec error when video loads successfully
  codecError.value = false;
  codecErrorDetails.value = '';
  play();
};

const onLoadedMetadata = () => {
  if (videoPlayerRef.value) {
    // Check if video has valid dimensions
    if (videoPlayerRef.value.videoWidth === 0 || videoPlayerRef.value.videoHeight === 0) {
      codecError.value = true;
      codecErrorDetails.value = 'Video stream appears to be missing or uses an unsupported codec';
    }
  }
};

const onVideoError = (event: Event) => {
  const video = event.target as HTMLVideoElement;
  if (video.error) {
    codecError.value = true;
    
    // Provide user-friendly error messages
    switch (video.error.code) {
      case video.error.MEDIA_ERR_ABORTED:
        codecErrorDetails.value = 'Video loading was aborted';
        break;
      case video.error.MEDIA_ERR_NETWORK:
        codecErrorDetails.value = 'Network error while loading video';
        break;
      case video.error.MEDIA_ERR_DECODE:
        codecErrorDetails.value = 'Video format or codec not supported';
        break;
      case video.error.MEDIA_ERR_SRC_NOT_SUPPORTED:
        codecErrorDetails.value = 'Video format not supported by your browser';
        break;
      default:
        codecErrorDetails.value = 'An unknown error occurred';
    }
  }
};

onMounted((): void => {
  albumFiles.value = [];
  activeTrack.value = undefined;

  if (props.floating) {
    if (floatingVideoInitialTime.value && videoPlayerRef.value) {
      // Wait for video to be ready before seeking
      const seekToTime = floatingVideoInitialTime.value;
      videoPlayerRef.value.addEventListener('loadedmetadata', () => {
        videoPlayerRef.value!.currentTime = seekToTime;
        floatingVideoInitialTime.value = 0; // Reset so we don't seek again
      }, { once: true });
    }
    // If we don't have release info from props but have it stored, keep using the stored info
    if (!props.releaseId && !props.releaseName && floatingVideoRelease.value) {
      // The stored release info will be used for navigation
    }
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
    
    // Store release info - use props if available, otherwise try to get from current route
    const releaseId = props.releaseId || (route.name === 'Release' ? route.params.id as string : undefined);
    const releaseName = props.releaseName || 'Video';
    
    if (releaseId) {
      floatingVideoRelease.value = {
        id: releaseId,
        name: releaseName,
        contentCID: props.contentCid
      };
    }
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
  cursor: move;
  user-select: none;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.1);
  background-color: #0a0a0a;
  overflow: hidden;
}

.floating-container::after {
  content: '';
  position: absolute;
  top: 32px; /* Below the Now Playing bar */
  left: 0;
  width: 20px;
  height: 20px;
  cursor: nwse-resize;
  background: linear-gradient(225deg, transparent 50%, rgba(138, 43, 226, 0.3) 50%);
  z-index: 1001;
}

.floating-container video {
  pointer-events: auto;
}

/* Resize handles */
.resize-handle {
  position: absolute;
  width: 15px;
  height: 15px;
  z-index: 1002;
}

.resize-handle-nw {
  top: 0;
  left: 0;
  cursor: nw-resize;
}

.resize-handle-ne {
  top: 0;
  right: 0;
  cursor: ne-resize;
}

.resize-handle-sw {
  bottom: 0;
  left: 0;
  cursor: sw-resize;
}

.resize-handle-se {
  bottom: 0;
  right: 0;
  cursor: se-resize;
}

/* Optional: Visual hint on hover */
.resize-handle:hover::before {
  content: '';
  position: absolute;
  width: 100%;
  height: 100%;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}
</style>
