<template>
  <v-dialog
    v-model="dialog"
    max-width="500px"
  >
    <v-card
      :loading="loading"
      :disabled="loading"
    >
      <template #title> Download track </template>

      <template #subtitle> {{ track?.title }} - {{ track?.album }} </template>

      <template #text>
        <v-list>
          <v-header>Select a quality:</v-header>
          <v-list-item>
            <template #title>Original</template>

            <template #append>
              {{ track?.size }}
              <v-btn
                variant="text"
                icon
                class="ml-3"
                @click="downloadTrack()"
              >
                <v-icon
                  size="25px"
                  icon="mdi-download"
                />
              </v-btn>
            </template>
          </v-list-item>
        </v-list>
      </template>
    </v-card>
  </v-dialog>
</template>

<script lang="ts">
import {defineComponent} from 'vue';
import type {AudioTrack} from '../../composables/audioAlbum';
import {IPFS_GATEWAY} from '/@/constants/ipfs';

export default defineComponent({
  data() {
    return {
      dialog: false,
      loading: false,
      track: undefined as AudioTrack | undefined,
    };
  },
  methods: {
    setTrack(track: AudioTrack) {
      this.track = track;
      this.dialog = true;
    },

    //TODO: Move this into a generic utils file
    async downloadTrack() {
      try {
        this.loading = true;
        const url = `https://${IPFS_GATEWAY}/ipfs/${this.track?.cid}`;
        const response = await fetch(url);

        if (!response.ok) {
          throw new Error(`Error downloading a file: ${response.statusText}`);
        }

        const blob = await response.blob();
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = `${this.track?.title} - ${this.track?.album}`;

        link.click();

        URL.revokeObjectURL(link.href);
      } catch (e) {
        console.log(e);
      } finally {
        this.loading = false;
      }
    },
  },
});
</script>
