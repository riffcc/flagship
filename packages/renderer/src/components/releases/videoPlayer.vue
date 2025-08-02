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
            maxHeight: floating ? 'calc(100% - 32px)' : `${displayHeight - 64}px`,
            width: '100%',
            height: floating ? 'calc(100% - 32px)' : '100%',
            objectFit: floating ? 'cover' : 'contain',
            position: floating ? 'absolute' : 'relative',
            top: floating ? '32px' : '0',
            left: 0,
            backgroundColor: '#0a0a0a',
          }"
          :src="parseUrlOrCid(props.contentCid)"
          :controls="false"
          crossorigin="anonymous"
          @click="togglePlay"
          @loadeddata="play"
          @canplay="canPlay"
          @progress="updateProgress"
        ></video>

        <!-- Loading overlay -->
        <v-sheet
          v-if="isLoading"
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
const dragStart = ref({ x: 0, y: 0 });
const containerPosition = ref({ x: 0, y: 0 });
const containerSize = ref({ width: 384, height: 216 });

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
  if (!props.floating || !containerRef.value) return;
  
  // Check if clicking on resize handle (top-left corner)
  const rect = containerRef.value.getBoundingClientRect();
  const isNearEdge = (
    e.clientX < rect.left + 20 &&
    e.clientY < rect.top + 52 && // 32px for Now Playing bar + 20px handle
    e.clientY > rect.top + 32  // Below the Now Playing bar
  );
  
  if (isNearEdge) {
    isResizing.value = true;
    dragStart.value = { x: e.clientX, y: e.clientY };
    containerSize.value = { width: rect.width, height: rect.height };
    containerPosition.value = { x: rect.left, y: rect.top };
  } else {
    isDragging.value = true;
    dragStart.value = { x: e.clientX - rect.left, y: e.clientY - rect.top };
    containerPosition.value = { x: rect.left, y: rect.top };
  }
  
  document.addEventListener('mousemove', handleDrag);
  document.addEventListener('mouseup', stopDrag);
};

const handleDrag = (e: MouseEvent) => {
  if (!containerRef.value) return;
  
  if (isDragging.value) {
    const newX = Math.max(0, Math.min(window.innerWidth - containerRef.value.offsetWidth, e.clientX - dragStart.value.x));
    const newY = Math.max(0, Math.min(window.innerHeight - containerRef.value.offsetHeight, e.clientY - dragStart.value.y));
    
    containerRef.value.style.left = `${newX}px`;
    containerRef.value.style.top = `${newY}px`;
    containerRef.value.style.right = 'auto';
    containerRef.value.style.bottom = 'auto';
  } else if (isResizing.value) {
    // For top-left resize, we need to move the position AND change the size
    const deltaX = e.clientX - dragStart.value.x;
    const deltaY = e.clientY - dragStart.value.y;
    
    // Calculate new size (inverse the deltas since we're resizing from top-left)
    const newWidth = Math.max(320, Math.min(window.innerWidth * 0.8, containerSize.value.width - deltaX));
    const newHeight = Math.max(180, Math.min(window.innerHeight * 0.8, containerSize.value.height - deltaY));
    
    // Calculate new position
    const newLeft = Math.max(0, containerPosition.value.x + deltaX);
    const newTop = Math.max(0, containerPosition.value.y + deltaY);
    
    // Only update if within bounds
    if (newLeft + newWidth <= window.innerWidth && newTop + newHeight <= window.innerHeight) {
      containerRef.value.style.width = `${newWidth}px`;
      containerRef.value.style.height = `${newHeight}px`;
      containerRef.value.style.left = `${newLeft}px`;
      containerRef.value.style.top = `${newTop}px`;
      containerRef.value.style.right = 'auto';
      containerRef.value.style.bottom = 'auto';
    }
  }
};

const stopDrag = () => {
  isDragging.value = false;
  isResizing.value = false;
  document.removeEventListener('mousemove', handleDrag);
  document.removeEventListener('mouseup', stopDrag);
};

const defaultSkipTime = 10;
onMounted((): void => {
  albumFiles.value = [];
  activeTrack.value = undefined;

  if (props.floating) {
    if (floatingVideoInitialTime.value) {
      seekingTrack(floatingVideoInitialTime.value);
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
</style>
