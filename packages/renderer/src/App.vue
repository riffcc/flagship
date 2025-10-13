<template>
  <v-app>
    <div class="app-header-border"></div>
    <gamepad-nav-bar />
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
    <!-- P2P Status Indicator -->
    <v-chip
      v-if="connected || peers.length > 0 || directPeersConnected > 0"
      size="small"
      variant="tonal"
      class="p2p-status-indicator"
      :color="directPeersConnected > 0 ? 'success' : (connected ? 'primary' : 'warning')"
    >
      <v-icon start>{{ directPeersConnected > 0 ? 'mdi-lan-connect' : (connected ? 'mdi-cloud-check' : 'mdi-cloud-off') }}</v-icon>
      P2P: {{ peers.length }} relay{{ directPeersConnected > 0 ? ` | ${directPeersConnected} direct` : '' }}
    </v-chip>

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
            icon="mdi-close"
            variant="text"
            @click="showDHTDebug = false"
          ></v-btn>
        </v-card-title>
        <v-card-text>
          <v-row dense>
            <v-col cols="12">
              <v-card variant="outlined">
                <v-card-subtitle>Site Information</v-card-subtitle>
                <v-card-text>
                  <div><strong>Site ID:</strong> {{ myPeerId || 'Not connected' }}</div>
                  <div><strong>Connected Peers (Relay):</strong> {{ peers.length }}</div>
                  <div><strong>Connected Peers (Direct):</strong> {{ directPeersConnected }}</div>
                  <div><strong>Relay Status:</strong> {{ connected ? 'Connected' : 'Disconnected' }}</div>
                </v-card-text>
              </v-card>
            </v-col>

            <v-col cols="12" v-if="peers.length > 0">
              <v-card variant="outlined">
                <v-card-subtitle>Discovered Peers</v-card-subtitle>
                <v-card-text>
                  <v-list density="compact">
                    <v-list-item
                      v-for="peer in peers"
                      :key="peer.peer_id"
                      :title="peer.peer_id"
                      :subtitle="`Score: ${peer.score} | Height: ${peer.latest_height}`"
                    >
                      <template #prepend>
                        <v-icon color="success">mdi-lan</v-icon>
                      </template>
                    </v-list-item>
                  </v-list>
                </v-card-text>
              </v-card>
            </v-col>

            <v-col cols="12">
              <v-card variant="outlined">
                <v-card-subtitle>Sync Status</v-card-subtitle>
                <v-card-text>
                  <div><strong>Local Blocks:</strong> {{ localBlocks.length }}</div>
                  <div><strong>Needed Blocks:</strong> {{ neededBlocks.length }}</div>
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
import { ref, watchEffect, onMounted } from 'vue';

import appFooter from '/@/components/layout/appFooter.vue';
import GamepadNavBar from '/@/components/layout/gamepadNavBar.vue';
import audioPlayer from '/@/components/releases/audioPlayer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import GamepadHints from '/@/components/gamepad/gamepadHints.vue';
import StartMenu from '/@/components/misc/startMenu.vue';
import NetworkMapGraph from '/@/components/misc/networkMapGraph.vue';

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
import { useP2P } from '/@/composables/useP2P';
import { useIdentity } from '/@/composables/useIdentity';

const { showDefederation, showDHTDebug } = useShowDefederation();
const showNetworkMap = ref(false);
const { connected, peers, directPeersConnected, connect, myPeerId, localBlocks, neededBlocks } = useP2P();
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

  // Connect to P2P relay for peer discovery
  try {
    connect();
    console.log('[App] P2P relay connection initiated');
  } catch (error) {
    console.warn('[App] Failed to connect to P2P relay:', error);
  }

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
watchEffect(() => {
  if (releases.value && releases.value.length > 0) {
    const searchableContent = releases.value.map(release => ({
      id: release.id,
      title: release.name,
      artist: release.metadata?.artist as string | undefined,
      description: release.metadata?.description as string | undefined,
      category: release.categoryId || 'other',
      tags: (release.metadata?.tags as string[]) || [],
      year: release.metadata?.year as number | undefined,
      type: (release.metadata?.type === 'artist' ? 'artist' :
             release.categoryId === 'music' ? 'music' :
             release.categoryId === 'movies' ? 'movie' :
             release.categoryId === 'tv-shows' ? 'tv' :
             'other') as 'music' | 'movie' | 'tv' | 'artist' | 'other',
      thumbnailCID: release.thumbnailCID,
    }));

    indexContent(searchableContent);
    console.log(`[App] Indexed ${searchableContent.length} releases for search`);
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
