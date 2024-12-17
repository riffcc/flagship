<template>
  <v-app>
    <app-bar />
    <v-main min-height="100vh">
      <router-view />
    </v-main>
    <audio-player v-if="activeTrack"></audio-player>
    <video-player
      v-if="floatingVideoSource"
      floating
      :content-cid="floatingVideoSource"
    ></video-player>
    <app-footer />
  </v-app>
</template>

<script setup lang="ts">
import {onKeyStroke} from '@vueuse/core';
import {ref, watchEffect} from 'vue';
import audioPlayer from '/@/components/releases/audioPlayer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';

import appBar from '/@/components/layout/appBar.vue';
import appFooter from '/@/components/layout/appFooter.vue';

import {useAudioAlbum} from '/@/composables/audioAlbum';
import {useFloatingVideo} from '/@/composables/floatingVideo';
import {useShowDefederation} from '/@/composables/showDefed';

const {showDefederation} = useShowDefederation();
const {activeTrack} = useAudioAlbum();
const {floatingVideoSource} = useFloatingVideo();

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
</script>
