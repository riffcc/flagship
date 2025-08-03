<template>
  <v-container
    v-if="!isLensReady && !releases && !featuredReleases && !contentCategories"
    class="h-screen"
  >
    <v-sheet
      color="transparent"
      class="d-flex w-100 fill-height align-center justify-center"
    >
      <v-progress-circular
        indeterminate
        color="primary"
      ></v-progress-circular>
    </v-sheet>
  </v-container>
  <v-app v-else>
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

import { useAudioAlbum } from '/@/composables/audioAlbum';
import { useFloatingVideo } from '/@/composables/floatingVideo';
import { useShowDefederation } from '/@/composables/showDefed';
import { useLensInitialization } from '/@/composables/lensInitialization';
import { useGamepad } from '/@/composables/useGamepad';
import { useGamepadNavigation } from '/@/composables/useGamepadNavigation';
import { useGetReleasesQuery, useGetFeaturedReleasesQuery, useContentCategoriesQuery } from './plugins/lensService';
import { useGlobalPlayback } from '/@/composables/globalPlayback';
import { useInputMethod } from '/@/composables/useInputMethod';

const { showDefederation } = useShowDefederation();
const { activeTrack } = useAudioAlbum();
const { floatingVideoSource, floatingVideoRelease } = useFloatingVideo();

const { isLensReady, initLensService } = useLensInitialization();
const { gamepadState, onButtonPress } = useGamepad();
const { showCursor } = useGamepadNavigation();
const { currentInputMethod } = useInputMethod();

const showStartMenu = ref(false);

const MAGIC_KEY = 'magicmagic';

const yetToType = ref(MAGIC_KEY);

onKeyStroke(e => {
  if (!yetToType.value.length) return;
  if (e.key === yetToType.value[0]) {
    yetToType.value = yetToType.value.slice(1);
  } else {
    yetToType.value = MAGIC_KEY;
  }
});

watchEffect(() => {
  if (!yetToType.value.length) showDefederation.value = true;
});

const CURTAIN_KEY = 'curtain';
const yetToTypeCurtain = ref(CURTAIN_KEY);

onKeyStroke(e => {
  if (!yetToTypeCurtain.value.length) return;
  if (e.key === yetToTypeCurtain.value[0]) {
    yetToTypeCurtain.value = yetToTypeCurtain.value.slice(1);
  } else {
    yetToTypeCurtain.value = CURTAIN_KEY;
  }
});

watchEffect(() => {
  if (!yetToTypeCurtain.value.length) showDefederation.value = false;
});

onMounted(async () => {
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
</style>
