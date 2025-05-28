<template>
  <v-container
    fluid
    class="pa-0"
  >
    <template v-if="targetRelease || federationEntry">
      <video-player
        v-if="isVideo"
        :content-cid="contentCid"
      />
      <album-viewer
        v-else-if="isAudio"
        :release="targetRelease || federationEntryAsRelease"
      ></album-viewer>
    </template>
    <div
      v-else
      class="d-flex align-center justify-center h-screen"
    >
      <v-sheet
        color="transparent"
        class="d-flex flex-column mx-auto"
        max-width="16rem"
      >
        <template v-if="isLoading">
          <v-progress-circular
            indeterminate
            color="primary"
          ></v-progress-circular>
        </template>
        <template v-else>
          <p class="text-white text-center mb-2">Release not found.</p>
          <v-btn
            color="primary-darken-1"
            @click="router.push('/')"
          >
            Go to Home
          </v-btn>
        </template>
      </v-sheet>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import { computed, watch, ref } from 'vue';
import { useRouter } from 'vue-router';
import albumViewer from '/@/components/releases/albumViewer.vue';
import videoPlayer from '/@/components/releases/videoPlayer.vue';
import { useGetReleaseQuery, useLensService } from '/@/plugins/lensService/hooks';
import { federationEntryToRelease } from '/@/utils/federationIndex';
import type { IndexableFederationEntry } from '@riffcc/lens-sdk';
import type { ReleaseItem, AnyObject } from '/@/types';

const props = defineProps<{
  id: string;
}>();

const router = useRouter();
const { lensService } = useLensService();

// Check if this is a federation index ID (format: sourceSiteId:contentCid)
const isFederationId = computed(() => props.id.includes(':'));

// Try to get as a regular release first
const { data: targetRelease, isLoading: isReleaseLoading } = useGetReleaseQuery(props, {
  enabled: !isFederationId.value, // Only query if not a federation ID
});

// For federation IDs, we need to extract the content CID and possibly fetch metadata
const federationEntry = ref<IndexableFederationEntry | null>(null);
const isFederationLoading = ref(false);

// Parse federation ID
const parsedFederationId = computed(() => {
  if (!isFederationId.value) return null;
  const [sourceSiteId, contentCid] = props.id.split(':');
  return { sourceSiteId, contentCid };
});

// Get content CID for player
const contentCid = computed(() => {
  if (targetRelease.value) {
    return targetRelease.value.contentCID;
  }
  if (parsedFederationId.value) {
    return parsedFederationId.value.contentCid;
  }
  return null;
});

// Convert federation entry to release format if needed
const federationEntryAsRelease = computed<ReleaseItem<AnyObject> | null>(() => {
  if (!federationEntry.value) return null;
  return federationEntryToRelease(federationEntry.value);
});

// Determine content type
const isVideo = computed(() => {
  if (targetRelease.value) {
    return ['video', 'movie'].includes(targetRelease.value.categoryId);
  }
  if (federationEntry.value) {
    return ['video', 'movie'].includes(federationEntry.value.categoryId) || 
           federationEntry.value.contentType === 'video';
  }
  return false;
});

const isAudio = computed(() => {
  if (targetRelease.value) {
    return ['audio', 'music'].includes(targetRelease.value.categoryId);
  }
  if (federationEntry.value) {
    return ['audio', 'music'].includes(federationEntry.value.categoryId) || 
           federationEntry.value.contentType === 'audio';
  }
  return false;
});

const isLoading = computed(() => {
  return isReleaseLoading.value || isFederationLoading.value;
});

// If it's a federation ID, try to get minimal metadata from the federation index
watch(isFederationId, async (isFedId) => {
  if (isFedId && parsedFederationId.value) {
    isFederationLoading.value = true;
    try {
      // Try to find this entry in the federation index
      const results = await lensService.complexFederationIndexQuery({
        sourceSiteId: parsedFederationId.value.sourceSiteId,
        limit: 1000, // Search through recent entries
      });
      
      // Find the specific entry
      const entry = results.find(e => 
        e.contentCid === parsedFederationId.value!.contentCid &&
        e.sourceSiteId === parsedFederationId.value!.sourceSiteId,
      );
      
      if (entry) {
        federationEntry.value = entry;
      } else {
        // Create a minimal entry if not found
        federationEntry.value = {
          id: props.id,
          contentCid: parsedFederationId.value.contentCid,
          title: 'Federated Content',
          sourceSiteId: parsedFederationId.value.sourceSiteId,
          sourceSiteName: 'Unknown Site',
          contentType: 'video', // Default to video
          categoryId: 'video',
          timestamp: Date.now(),
          tags: [],
        } as IndexableFederationEntry;
      }
    } catch (error) {
      console.error('Failed to fetch federation entry metadata:', error);
      // Create minimal entry on error
      federationEntry.value = {
        id: props.id,
        contentCid: parsedFederationId.value.contentCid,
        title: 'Federated Content',
        sourceSiteId: parsedFederationId.value.sourceSiteId,
        sourceSiteName: 'Unknown Site',
        contentType: 'video',
        categoryId: 'video',
        timestamp: Date.now(),
        tags: [],
      } as IndexableFederationEntry;
    } finally {
      isFederationLoading.value = false;
    }
  }
}, { immediate: true });

// Update media session metadata
watch([targetRelease, federationEntry], ([release, fedEntry]) => {
  const activeContent = release || (fedEntry ? federationEntryAsRelease.value : null);
  
  if (activeContent && 'mediaSession' in navigator) {
    try {
      navigator.mediaSession.metadata = new MediaMetadata({
        title: activeContent.name,
        artist: activeContent.metadata?.['author'] as string | undefined,
        album: activeContent.metadata?.albumName as string || '',
        artwork: activeContent.thumbnailCID ? [
          {
            src: activeContent.thumbnailCID,
          },
        ] : undefined,
      });
    } catch (error) {
      console.error('Failed to set MediaMetadata:', error);
    }
  } else if ('mediaSession' in navigator) {
    navigator.mediaSession.metadata = null;
  }
}, { immediate: true });
</script>