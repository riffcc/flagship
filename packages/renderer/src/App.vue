<template>
  <v-app>
    <div class="app-header-border"></div>
    <gamepad-nav-bar v-if="gamepadState.connected" />
    <app-bar v-else />
    <v-main min-height="100vh" class="mt-12">
      <router-view />
    </v-main>
    <audio-player v-if="activeTrack"></audio-player>
    <video-player
      v-if="floatingVideoSource"
      floating
      :content-cid="floatingVideoSource"
      :release-id="floatingVideoRelease?.id"
      :release-name="floatingVideoRelease?.name"
    ></video-player>
    <app-footer />
    <gamepad-hints :gamepad-state="gamepadState" />
    <div
      v-show="showCursor"
      id="gamepad-cursor"
      class="gamepad-cursor"
    ></div>

    <!-- DHT Debug Overlay (activated by typing "magicmagicmagic") -->
    <v-overlay
      v-model="showDHTDebug"
      class="align-center justify-center"
      scrim="rgba(0, 0, 0, 0.8)"
    >
      <v-card
        max-width="800"
        max-height="80vh"
        class="overflow-auto"
      >
        <v-card-title class="d-flex justify-space-between align-center">
          <span>DHT & Sync Debug Console</span>
          <v-btn
            icon="$close"
            variant="text"
            @click="showDHTDebug = false"
          ></v-btn>
        </v-card-title>
        <v-card-text>
          <v-row dense>
            <v-col cols="12">
              <v-card variant="outlined">
                <v-card-subtitle>Content Status</v-card-subtitle>
                <v-card-text>
                  <div><strong>Total Releases:</strong> {{ releases?.length || 0 }}</div>
                  <div><strong>Featured Releases:</strong> {{ featuredReleases?.length || 0 }}</div>
                </v-card-text>
              </v-card>
            </v-col>

            <v-col cols="12">
              <v-alert
                type="info"
                variant="tonal"
                density="compact"
              >
                <div class="text-caption">
                  <strong>Encryption:</strong> DHT values are encrypted in enterprise mode. Normal mode shares data across all nodes.
                </div>
              </v-alert>
            </v-col>
          </v-row>
        </v-card-text>
      </v-card>
    </v-overlay>

    <!-- Network Map Overlay (activated by typing "batmanbatmanbatman") -->
    <v-overlay
      v-model="showNetworkMap"
      class="align-center justify-center"
      scrim="rgba(0, 0, 0, 0.9)"
    >
      <network-map-graph @close="showNetworkMap = false" />
    </v-overlay>

    <start-menu v-model="showStartMenu" />
  </v-app>
</template>

<script setup lang="ts">
import { onKeyStroke } from '@vueuse/core';
import { ref, watchEffect, onMounted, defineAsyncComponent } from 'vue';

import appFooter from '/@/components/layout/appFooter.vue';
import appBar from '/@/components/layout/appBar.vue';
import GamepadNavBar from '/@/components/layout/gamepadNavBar.vue';
import audioPlayer from '/@/components/releases/audioPlayer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import GamepadHints from '/@/components/gamepad/gamepadHints.vue';
import StartMenu from '/@/components/misc/startMenu.vue';

// Lazy load NetworkMapGraph to avoid WebGPU errors from 3d-force-graph on browsers without support
const NetworkMapGraph = defineAsyncComponent(() => import('/@/components/misc/networkMapGraph.vue'));

import { useAudioAlbum } from '/@/composables/audioAlbum';
import { useFloatingVideo } from '/@/composables/floatingVideo';
import { useShowDefederation } from '/@/composables/showDefed';
import { useLensInitialization } from '/@/composables/lensInitialization';
import { useGamepad } from '/@/composables/useGamepad';
import { useGamepadNavigation } from '/@/composables/useGamepadNavigation';
import { useGetReleasesQuery, useGetFeaturedReleasesQuery, useContentCategoriesQuery } from './plugins/lensService';
import { useGlobalPlayback } from '/@/composables/globalPlayback';
import { useInputMethod } from '/@/composables/useInputMethod';
import { useLocalSearch } from '/@/composables/useLocalSearch';
import { useIdentity } from '/@/composables/useIdentity';

const { showDefederation, showDHTDebug } = useShowDefederation();
const showNetworkMap = ref(false);
const { activeTrack } = useAudioAlbum();
const { floatingVideoSource, floatingVideoRelease } = useFloatingVideo();

const { isLensReady, initLensService } = useLensInitialization();
const { gamepadState, onButtonPress } = useGamepad();
const { showCursor } = useGamepadNavigation();
const { currentInputMethod } = useInputMethod();
const { initialize: initializeIdentity } = useIdentity();

const showStartMenu = ref(false);

const MAGIC_KEY = 'magicmagic';
const DHT_DEBUG_KEY = 'magicmagicmagic';  // Extended key for DHT debug view
const NETWORK_MAP_KEY = 'batmanbatmanbatman';  // Network map visualization

const yetToType = ref(MAGIC_KEY);

onKeyStroke(e => {
  if (!yetToType.value.length) {
    // Reset for next time
    yetToType.value = MAGIC_KEY;
    return;
  }
  if (e.key === yetToType.value[0]) {
    yetToType.value = yetToType.value.slice(1);
  } else {
    yetToType.value = MAGIC_KEY;
  }
});

watchEffect(() => {
  if (!yetToType.value.length) {
    // Check which magic key was completed
    // If they typed the full DHT debug key, show DHT debug
    // Otherwise toggle defederation view
    if (MAGIC_KEY === 'magicmagic' && !DHT_DEBUG_KEY.startsWith('magicmagic')) {
      // Original behavior - toggle defederation
      showDefederation.value = !showDefederation.value;
    } else {
      // For now, typing magicmagic toggles defederation
      // Later we can implement magicmagicmagic separately
      showDefederation.value = !showDefederation.value;
    }
  }
});

// Separate watcher for DHT debug key (magicmagicmagic)
const dhtDebugYetToType = ref(DHT_DEBUG_KEY);

onKeyStroke(e => {
  if (!dhtDebugYetToType.value.length) {
    // Reset for next time
    dhtDebugYetToType.value = DHT_DEBUG_KEY;
    return;
  }
  if (e.key === dhtDebugYetToType.value[0]) {
    dhtDebugYetToType.value = dhtDebugYetToType.value.slice(1);
  } else {
    dhtDebugYetToType.value = DHT_DEBUG_KEY;
  }
});

watchEffect(() => {
  if (!dhtDebugYetToType.value.length) {
    // Toggle DHT debug view when extended magic key is fully typed
    showDHTDebug.value = !showDHTDebug.value;
  }
});

// Separate watcher for Network Map key (batmanbatmanbatman)
const networkMapYetToType = ref(NETWORK_MAP_KEY);

onKeyStroke(e => {
  if (!networkMapYetToType.value.length) {
    // Reset for next time
    networkMapYetToType.value = NETWORK_MAP_KEY;
    return;
  }
  if (e.key === networkMapYetToType.value[0]) {
    networkMapYetToType.value = networkMapYetToType.value.slice(1);
  } else {
    networkMapYetToType.value = NETWORK_MAP_KEY;
  }
});

watchEffect(() => {
  if (!networkMapYetToType.value.length) {
    // Toggle Network Map view when batman key is fully typed
    showNetworkMap.value = !showNetworkMap.value;
  }
});

onMounted(async () => {
  // Initialize ed25519 identity first
  try {
    await initializeIdentity();
    console.log('[App] Identity initialized');
  } catch (error) {
    console.error('[App] Failed to initialize identity:', error);
  }

  initLensService();

  // Setup gamepad controls
  onButtonPress('start', () => {
    showStartMenu.value = true;
  });

  // Setup L3/R3 for play/pause
  const { globalTogglePlay } = useGlobalPlayback();
  onButtonPress('leftStickButton', () => {
    globalTogglePlay();
  });
  onButtonPress('rightStickButton', () => {
    globalTogglePlay();
  });
});


const { data: releases } = useGetReleasesQuery({
  enabled: isLensReady,
});

const {  data: featuredReleases } = useGetFeaturedReleasesQuery({
  enabled: isLensReady,
});

const { data: contentCategories } = useContentCategoriesQuery({
  enabled: isLensReady,
});

// Setup search indexing
const { indexContent } = useLocalSearch();

// Index catalog data when releases load
// Structure types (artists, series, etc.) are excluded from the count
const STRUCTURE_TYPES = ['artist', 'series', 'season', 'author', 'collection'];

watchEffect(() => {
  if (releases.value && releases.value.length > 0) {
    // Filter out structure-type releases (artists, series, etc.)
    const contentReleases = releases.value.filter(release =>
      !STRUCTURE_TYPES.includes(release.metadata?.type as string)
    );

    const searchableContent = contentReleases.map(release => ({
      id: release.id,
      title: release.name,
      artist: release.metadata?.artist as string | undefined,
      description: release.metadata?.description as string | undefined,
      category: release.categoryId || 'other',
      tags: (release.metadata?.tags as string[]) || [],
      year: release.metadata?.year as number | undefined,
      type: (release.categoryId === 'music' ? 'music' :
             release.categoryId === 'movies' ? 'movie' :
             release.categoryId === 'tv-shows' ? 'tv' :
             'other') as 'music' | 'movie' | 'tv' | 'artist' | 'other',
      thumbnailCID: release.thumbnailCID,
    }));

    indexContent(searchableContent);
    console.log(`[App] Indexed ${searchableContent.length} content releases for search (${releases.value.length - searchableContent.length} structure releases excluded)`);
  }
});

// UI now loads immediately - no spinner needed
// Loading states are shown inline within components
</script>

<style>
.app-header-border {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg,
    rgba(98, 28, 166, 0.3) 0%,
    rgba(98, 28, 166, 0.6) 15%,
    rgba(98, 28, 166, 0.9) 30%,
    rgba(98, 28, 166, 1) 40%,
    rgba(98, 28, 166, 0.9) 50%,
    rgba(98, 28, 166, 0.6) 65%,
    rgba(98, 28, 166, 0.3) 80%,
    rgba(98, 28, 166, 0.1) 95%,
    transparent 100%
  );
  background-size: 150% 100%;
  z-index: 1000;
  animation: gentleFlow 20s ease-in-out infinite;
  filter: blur(0.5px);
}

@keyframes gentleFlow {
  0% {
    background-position: 0% 50%;
    opacity: 0.6;
  }
  25% {
    background-position: 50% 50%;
    opacity: 0.8;
  }
  50% {
    background-position: 100% 50%;
    opacity: 0.7;
  }
  75% {
    background-position: 50% 50%;
    opacity: 0.9;
  }
  100% {
    background-position: 0% 50%;
    opacity: 0.6;
  }
}

/* Add a subtle glow that follows the animation */
.app-header-border::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 20px;
  background: linear-gradient(180deg,
    rgba(138, 43, 226, 0.2) 0%,
    rgba(138, 43, 226, 0.1) 30%,
    transparent 100%
  );
  animation: gentleGlow 30s ease-in-out infinite;
  pointer-events: none;
}

@keyframes gentleGlow {
  0%, 100% {
    opacity: 0.3;
  }
  50% {
    opacity: 0.5;
  }
}

.gamepad-cursor {
  position: fixed;
  width: 24px;
  height: 24px;
  border: 3px solid #8a2be2;
  border-radius: 50%;
  background: rgba(138, 43, 226, 0.3);
  pointer-events: none;
  z-index: 10000;
  transform: translate(-50%, -50%);
  transition: opacity 0.2s;
  box-shadow: 0 0 20px rgba(138, 43, 226, 0.8);
}

/* Global gamepad focus styles */
.gamepad-focused {
  position: relative !important;
  z-index: 10 !important;
  transition: all 0.2s ease !important;
}

/* Hide hover effects when using gamepad */
body.input-gamepad *:hover {
  background-color: inherit !important;
}

/* Only show hover effects when using mouse */
body.input-mouse *:hover {
  transition: all 0.2s ease;
}

/* Only show gamepad focus when using gamepad */
body.input-gamepad .gamepad-focused {
  box-shadow: 0 0 0 3px rgba(138, 43, 226, 0.8), 0 0 20px rgba(138, 43, 226, 0.5) !important;
}

/* Hide gamepad focus when using mouse */
body.input-mouse .gamepad-focused {
  box-shadow: none !important;
  outline: none !important;
}

/* Ensure pure black backgrounds */
.v-application {
  background: #000000 !important;
}

.v-main {
  background: #000000 !important;
}

/* Remove any gray backgrounds */
.v-sheet {
  background: transparent !important;
}

/* Content cards should have subtle borders for OLED */
.content-card {
  border: 1px solid rgba(255, 255, 255, 0.05);
  transition: all 0.3s ease;
}

.content-card:hover {
  border-color: rgba(138, 43, 226, 0.3);
}

/* Prevent focus on non-interactive elements */
div:not([data-navigable]):not(.content-card):focus,
v-sheet:focus,
v-container:focus {
  outline: none !important;
  box-shadow: none !important;
  background-color: transparent !important;
}

/* Prevent text selection with gamepad */
body.input-gamepad * {
  user-select: none;
}

/* P2P Status Indicator */
.p2p-status-indicator {
  position: fixed !important;
  bottom: 16px;
  right: 16px;
  z-index: 4999;
  opacity: 0.9;
  transition: opacity 0.3s ease;
}

.p2p-status-indicator:hover {
  opacity: 1;
}
</style>
