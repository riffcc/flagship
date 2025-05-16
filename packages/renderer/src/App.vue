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
import { onKeyStroke } from '@vueuse/core';
import { ref, watchEffect, onMounted } from 'vue';
import type { IPeerbitService } from '/@/lib/types';
import { Release } from '/@/lib/schema';
import { initializePeerbitService } from '/@/plugins/peerbit';
import { usePeerbitService } from '/@/plugins/peerbit/utils';

import audioPlayer from '/@/components/releases/audioPlayer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import appBar from '/@/components/layout/appBar.vue';
import appFooter from '/@/components/layout/appFooter.vue';

import { useAudioAlbum } from '/@/composables/audioAlbum';
import { useFloatingVideo } from '/@/composables/floatingVideo';
import { useShowDefederation } from '/@/composables/showDefed';

const { showDefederation } = useShowDefederation();
const { activeTrack } = useAudioAlbum();
const { floatingVideoSource } = useFloatingVideo();
const { peerbitService } = usePeerbitService();
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
  try {
    console.log('[App.vue onMounted] Initializing Peerbit service...');
    await initializePeerbitService();
    console.log('[App.vue onMounted] Peerbit service initialized successfully.');
    // At this point, peerbitServiceRef.value should be populated.
    // Stores or components relying on Peerbit can now safely access it,
    // ideally by watching peerbitServiceRef or being triggered by an event/callback.
    async function runPeerbitPostMountTasks(peerbitServiceInstance: IPeerbitService | undefined) {
      if (!peerbitServiceInstance) {
        console.error('[AppInit] Peerbit service instance not available for post-mount tasks.');
        return;
      }
      // Run Release put/get test asynchronously
      try {
        console.log('[AppInit] Starting async Release put/get test (post-mount)...');
        const newRelease = new Release({
          name: 'RiP!: A Remix Manifesto',
          categoryId: 'movie',
          contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
          thumbnailCID: 'Qmb3eeESRoX5L6NhTYLEtFFUS1FZgqe1e7hdBk2f57DUGh',
          metadata: JSON.stringify({
            classification: 'PG',
            description: 'Join filmmaker Brett Gaylor and mashup artist Girl Talk as they explore copyright and content creation in the digital age. In the process they dissect the media landscape of the 21st century and shatter the wall between users and producers.',
            duration: '1h 26m',
            author: 'Brett Gaylor',
            cover: 'QmcD4R3Qj8jBWY73H9LQWESgonNB1AMN3of23ubjDhJVSm',
          }),
        });

        const result = await peerbitServiceInstance.addRelease(newRelease);
        console.log(`[AppInit] Async Test (post-mount): Successfully put Release: ${newRelease.id} - ${newRelease.name}`);
        console.log(`[AppInit] Async Test (post-mount): Entry hash: ${result}`);

        const retrievedRelease = await peerbitServiceInstance.getRelease(newRelease.id);
        if (retrievedRelease) {
          console.log(`[AppInit] Async Test (post-mount): Successfully retrieved Release: ${retrievedRelease.id} - ${retrievedRelease.name}`);
          const replacer = (_key: string, value: unknown) =>
            typeof value === 'bigint' ? value.toString() : value;
          console.log('[AppInit] Async Test (post-mount): Retrieved Release Data:', JSON.stringify(retrievedRelease, replacer, 2));
        } else {
          console.error(`[AppInit] Async Test (post-mount): Failed to retrieve Release by ID: ${newRelease.id}`);
        }
      } catch (error) {
        console.error('[AppInit] Error during async Release put/get test (post-mount):', error);
      }
    }
    await runPeerbitPostMountTasks(peerbitService.value);
  } catch (error) {
    console.error('[App.vue onMounted] Failed to initialize Peerbit service:', error);
  }
});
</script>
