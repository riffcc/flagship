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

import appBar from '/@/components/layout/appBar.vue';
import appFooter from '/@/components/layout/appFooter.vue';
import audioPlayer from '/@/components/releases/audioPlayer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';

import { useAudioAlbum } from '/@/composables/audioAlbum';
import { useFloatingVideo } from '/@/composables/floatingVideo';
import { useShowDefederation } from '/@/composables/showDefed';
import {
  useAccountStatusQuery,
  useLensService,
  // useGetFeaturedReleasesQuery,
  // useGetReleasesQuery,
} from '/@/plugins/lensService/hooks';
import {
  AccountType,
  type SiteArgs,
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
// Use partial replication for guests - balance speed vs data availability
const customMemberSiteArgs: SiteArgs = {
  membersArg: {
    replicate: true,
  },
  administratorsArgs: {
    replicate: true,
  },
  releasesArgs: {
    replicate: true,
  },
  featuredReleasesArgs: {
    replicate: true,
  },
  contentCategoriesArgs: {
    replicate: true,
  },
  subscriptionsArgs: {
    replicate: true,
  },
  blockedContentArgs: {
    replicate: true,
  },
};

// Track initialization stages for better UX
const initStage = ref<'init' | 'connecting' | 'opening' | 'ready'>('init');

onMounted(async () => {
  try {
    if (!siteAddress) {
      throw new Error(
        'VITE_SITE_ADDRESS env var missing. Please review your .env file.',
        { cause: 'MISSING_CONFIG' },
      );
    }
    
    // Stage 1: Initialize lens service
    initStage.value = 'init';
    await lensService.init('.lens-node');
    
    // Stage 2: Connect to bootstrappers FIRST (critical for data availability)
    initStage.value = 'connecting';
    const bootstrappers = import.meta.env.VITE_BOOTSTRAPPERS;
    if (bootstrappers) {
      const bootstrapperList = bootstrappers.split(',').map(b => b.trim());
      console.log('Connecting to bootstrappers:', bootstrapperList);
      
      // Try to connect to at least one bootstrapper before opening site
      const dialResults = await Promise.allSettled(
        bootstrapperList.map(b => lensService.dial(b)),
      );
      
      const connected = dialResults.filter(r => r.status === 'fulfilled').length;
      const failed = dialResults.filter(r => r.status === 'rejected');
      
      console.log(`Connected to ${connected}/${bootstrapperList.length} bootstrappers`);
      if (failed.length > 0) {
        console.warn('Failed connections:', failed.map(f => f.reason?.message || f.reason));
      }
      
      // Even if no bootstrappers connect, continue (might have local data)
    }
    
    // Stage 3: Open site AFTER bootstrapper connection attempts
    initStage.value = 'opening';
    await lensService.openSite(siteAddress, customMemberSiteArgs);
    initStage.value = 'ready';
    initLoading.value = false;

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
    initLoading.value = false;
  }
});


// // Prefetch critical data immediately for faster homepage loading
// const prefetchData = () => {
//   // Start featured releases query immediately (will use static fallback if PeerBit fails)
//   useGetFeaturedReleasesQuery({ staleTime: 1000 * 30 });
//   // Start releases query immediately
//   useGetReleasesQuery({ staleTime: 1000 * 30 });
// };

// // Start prefetching as soon as the component mounts
// onMounted(() => {
//   setTimeout(prefetchData, 2000); // Small delay to ensure lens service is ready
// });

const { data: accountStatus } = useAccountStatusQuery();

watch(accountStatus, async (newValue, oldValue) => {
  if (!siteAddress) return;
  if (newValue !== oldValue) {
    console.log('accountStatus changed');
    let newSiteArgs: SiteArgs | undefined;
    switch (newValue) {
      case AccountType.ADMIN:
        newSiteArgs = ADMIN_SITE_ARGS;
        break;
      default:
        newSiteArgs = undefined;
        break;
    }
    if (newSiteArgs) {
      try {
        await lensService.closeSite();
        await lensService.openSite(siteAddress, newSiteArgs);
      } catch (e) {
        console.log(`Error on reopened the site with new replication args: ${e}`);
      }
    }
  }
});

</script>
