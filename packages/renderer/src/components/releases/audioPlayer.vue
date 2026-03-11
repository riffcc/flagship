<template>
  <v-sheet
    position="fixed"
    location="bottom"
    class="w-100 border rounded-t-xl audio-player-sheet"
    color="black"
    :elevation="24"
    height="100px"
    max-width="960px"
    style="left: 50%; transform: translateX(-50%); z-index: 1000;"
    :data-navigable="true"
    @dblclick="navigateToAlbum"
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
          <div class="d-flex align-center ga-1">
            <p class="text-subtitle-2 d-flex align-center ga-1 mb-0">
              <span v-if="activeTrack?.artist">{{ activeTrack.artist }}</span>
              <span
                v-if="activeTrack?.artist && activeTrack?.title"
                :style="{
                  color: 'white',
                  fontWeight: 'bold',
                  fontSize: '1.2em',
                  lineHeight: '1'
                }"
              >
                ›
              </span>
              <span
                v-if="activeTrack?.title"
                class="track-name"
                :style="{
                  color: 'rgba(168, 85, 247, 1)',
                  textShadow: '0 0 10px rgba(168, 85, 247, 0.7)',
                  fontWeight: '500'
                }"
              >
                {{ activeTrack.title }}
              </span>
            </p>
            <QualityBadge
              v-if="albumQuality"
              :quality="albumQuality"
              :quality-ladder="qualityLadder"
              player-mode
              @quality-change="handleQualityChange"
            />
          </div>
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
            :data-navigable="true"
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
          <!-- Volume Button with Flyout -->
          <div class="volume-control" v-click-outside="closeVolumeFlyout">
            <v-btn
              :icon="volume === 0 ? '$volume-off' : '$volume-high'"
              size="small"
              density="comfortable"
              class="volume-btn"
              :class="{ 'volume-active': showVolumeFlyout }"
              @click="handleVolumeClick"
            ></v-btn>
            <Transition name="volume-flyout">
              <div v-if="showVolumeFlyout" class="volume-flyout">
                <v-slider
                  :model-value="volume"
                  :min="0"
                  :max="1"
                  :step="0.01"
                  hide-details
                  thumb-color="white"
                  track-color="grey"
                  track-fill-color="primary"
                  :thumb-size="12"
                  @update:model-value="setVolume"
                />
              </div>
            </Transition>
          </div>
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
import {ref, onMounted, watch, onUnmounted} from 'vue';
import {useDisplay} from 'vuetify';
import {useRouter} from 'vue-router';
import {useAudioAlbum} from '/@/composables/audioAlbum';
import {usePlaybackController} from '/@/composables/playbackController';
import {usePlayerVolume} from '/@/composables/playerVolume';
import {useGlobalPlayback} from '/@/composables/globalPlayback';
import { parseUrlOrCid } from '/@/utils';
import QualityBadge from '/@/components/badges/QualityBadge.vue';

const {xs, smAndDown} = useDisplay();
const router = useRouter();

// Volume flyout state
const showVolumeFlyout = ref(false);

const handleVolumeClick = () => {
  if (showVolumeFlyout.value) {
    // Flyout is open - toggle mute/unmute
    toggleVolume();
  } else {
    // Flyout is closed - open it
    showVolumeFlyout.value = true;
  }
};

const closeVolumeFlyout = () => {
  showVolumeFlyout.value = false;
};

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
  albumQuality,
  qualityLadder,
  albumFiles,
  currentContentCid,
  isSwitchingQuality,
  handlePlay,
  handlePrevious,
  handleNext,
  handleOnClose,
  toggleRepeat,
  toggleShuffle,
} = useAudioAlbum();

const {volume, toggleVolume, setVolume} = usePlayerVolume();

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

const navigateToAlbum = () => {
  router.back();
};

/**
 * Handle quality tier change from the badge dropdown.
 * Switches to the new quality tier by updating the CID and reloading tracks.
 */
async function handleQualityChange(tierName: string, newCid: string) {
  console.log(`[audioPlayer] Switching quality to ${tierName} (CID: ${newCid})`);

  // Don't switch if already on this CID
  if (currentContentCid.value === newCid) {
    console.log('[audioPlayer] Already on this quality tier');
    return;
  }

  // Remember current playback state
  const wasPlaying = activeTrack.value;
  const playingIndex = wasPlaying?.index;
  const currentPosition = audioPlayerRef.value?.currentTime || 0;

  isSwitchingQuality.value = true;

  try {
    // Update the content CID
    currentContentCid.value = newCid;

    // Update audio quality to match the tier
    const tierToQuality: Record<string, { format: string; bitrate?: number; codec?: string }> = {
      'lossless': { format: 'flac', codec: 'FLAC' },
      'opus': { format: 'opus', codec: 'Opus' },
      'mp3_320': { format: 'mp3', bitrate: 320, codec: 'MP3' },
      'mp3_v0': { format: 'mp3', bitrate: 245, codec: 'LAME VBR' },
      'mp3_256': { format: 'mp3', bitrate: 256, codec: 'MP3' },
      'ogg': { format: 'vorbis', codec: 'Vorbis' },
      'mp3_vbr': { format: 'mp3', bitrate: 192, codec: 'LAME VBR' },
      'mp3_192': { format: 'mp3', bitrate: 192, codec: 'MP3' },
      'aac': { format: 'aac', codec: 'AAC' },
    };

    const newQuality = tierToQuality[tierName];
    if (newQuality) {
      albumQuality.value = newQuality as typeof albumQuality.value;
    }

    // Fetch tracks from the new CID - this is handled by the album viewer
    // For now, we just update the current track's CID if playing
    // The albumViewer will handle the full track list refresh

    // If we were playing, the track CID needs to update
    // This is a simplified approach - the album viewer handles full switching
    if (wasPlaying && playingIndex !== undefined && albumFiles.value[playingIndex]) {
      // The album viewer will refresh albumFiles with new CIDs
      // For seamless switching, we'd need to coordinate with albumViewer
      // For now, just log and let the user know quality changed
      console.log(`[audioPlayer] Quality switched to ${tierName} - track will update on next play`);
    }

    console.log(`[audioPlayer] Quality switched to ${tierName}`);
  } catch (error) {
    console.error('[audioPlayer] Failed to switch quality:', error);
  } finally {
    isSwitchingQuality.value = false;
  }
}

const defaultSkipTime = 10;
onMounted(() => {
  // Register global playback handlers
  const { registerPlaybackHandlers } = useGlobalPlayback();
  registerPlaybackHandlers({ play, pause, togglePlay });

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

onUnmounted(() => {
  // Clear global playback handlers when audio player is unmounted
  const { clearPlaybackHandlers } = useGlobalPlayback();
  clearPlaybackHandlers();
});
</script>

<style scoped>
.volume-control {
  position: relative;
}

.volume-btn:hover,
.volume-btn.volume-active {
  color: rgba(138, 43, 226, 0.9) !important;
  text-shadow: 0 0 8px rgba(138, 43, 226, 0.6);
}

.volume-flyout {
  position: absolute;
  bottom: calc(100% - 0.55em);
  left: -1.0em;
  padding: 8px 16px;
  width: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.volume-flyout :deep(.v-slider-thumb) {
  will-change: transform;
}

.volume-flyout-enter-active,
.volume-flyout-leave-active {
  transition: opacity 50ms ease, transform 50ms ease;
}

.volume-flyout-enter-from,
.volume-flyout-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
</style>
