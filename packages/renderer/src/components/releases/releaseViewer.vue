<template>
  <v-dialog
    v-model="dialog"
    max-width="800"
  >
    <template #activator="{props: activatorProps}">
      <slot
        name="activator"
        v-bind="{props: activatorProps}"
      ></slot>
    </template>
    <v-card>
      <v-card-item>
        <v-card-title class="d-flex">
          {{ release.release.contentName }}
          <v-spacer />
          <v-btn
            icon="mdi-close"
            @click="dialog = false"
          />
        </v-card-title>
        <v-divider />
      </v-card-item>
      <v-card-text>
        <audio-viewer
          v-if="release.release.category === 'audio'"
          :file="release.release.file"
        />
        <video-viewer
          v-else-if="release.release.category === 'movie'"
          :file="release.release.file"
        />
      </v-card-text>
    </v-card>
  </v-dialog>
</template>
<script setup lang="ts">
import type {types as orbiterTypes} from '@riffcc/orbiter';
import {ref} from 'vue';

import AudioViewer from './audioViewer.vue';
import VideoViewer from './videoViewer.vue';

export interface ReleaseProps {
  release: orbiterTypes.ReleaseWithId;
}

defineProps<ReleaseProps>();

// Potentially eventually useful: Reference for media file type conversions https://ffmpegwasm.netlify.app/

// Navigation
const dialog = ref(false);
</script>
