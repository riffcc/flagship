<template>
  <v-container
    v-if="initLoading || initError"
    class="h-screen"
  >
    <v-sheet
      color="transparent"
      class="d-flex w-100 fill-height align-center justify-center"
    >
      <v-progress-circular
        v-if="initLoading"
        indeterminate
        color="primary"
      ></v-progress-circular>
      <p v-else-if="initError">{{ initError }}</p>
    </v-sheet>
  </v-container>
  <v-app v-else>
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
import { ref, watchEffect, onMounted, watch } from 'vue';

import audioPlayer from '/@/components/releases/audioPlayer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import appBar from '/@/components/layout/appBar.vue';
import appFooter from '/@/components/layout/appFooter.vue';

import { useAudioAlbum } from '/@/composables/audioAlbum';
import { useFloatingVideo } from '/@/composables/floatingVideo';
import { useShowDefederation } from '/@/composables/showDefed';
import { useAccountStatusQuery, useLensService } from '/@/plugins/lensService/hooks';
import {
  AccountType,
  type SiteArgs,
  MEMBER_SITE_ARGS,
  ADMIN_SITE_ARGS,
} from '@riffcc/lens-sdk';

const { showDefederation } = useShowDefederation();
const { activeTrack } = useAudioAlbum();
const { floatingVideoSource } = useFloatingVideo();
const { lensService } = useLensService();
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
const initLoading = ref(true);
const initError = ref<string | null>();
const siteAddress = import.meta.env.VITE_SITE_ADDRESS;

onMounted(async () => {
  try {
    if (!siteAddress) {
      throw new Error(
        'VITE_SITE_ADDRESS env var missing. Please review your .env file.',
        { cause: 'MISSING_CONFIG' },
      );
    }
    await lensService.init('.lens-node');

    const bootstrappers = import.meta.env.VITE_BOOTSTRAPPERS;
    if (bootstrappers) {
      const promises = bootstrappers
        .split(',')
        .map((b) => lensService.dial(b.trim()));
      const result = await Promise.allSettled(promises);
      console.log(result);
    }
    await lensService.openSite(siteAddress);
  } catch (error) {
    if (error instanceof Error) {
      if (error.cause === 'MISSING_CONFIG') {
        initError.value = error.message;
      } else {
        initError.value = error.message.slice(200);
      }
    } else {
      initError.value = JSON.stringify(error).slice(200);
    }
  } finally {
    initLoading.value = false;
  }
});

const { data: accountStatus } = useAccountStatusQuery();

watch(accountStatus, async (newValue, oldValue) => {
  if (!siteAddress) return;
  if (newValue !== oldValue) {
    console.log('accountStatus changed');
    let newSiteArgs: SiteArgs | undefined;
    switch (newValue) {
      case AccountType.MEMBER:
        newSiteArgs = MEMBER_SITE_ARGS;
        break;
      case AccountType.ADMIN:
        newSiteArgs = ADMIN_SITE_ARGS;
        break;
      default:
        newSiteArgs = undefined;
        break;
    }
    try {
      await lensService.closeSite();
      await lensService.openSite(siteAddress, newSiteArgs);
    } catch (e) {
      console.log(`Error on reopened the site with new replication args: ${e}`);
    }
  }
});

</script>
