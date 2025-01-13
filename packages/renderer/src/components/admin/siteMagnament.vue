<template>
  <v-container fill-height>
    <v-sheet
      class="mx-auto py-4 px-6"
      max-width="384px"
    >
      <h6 class="text-h6 font-weight-bold mb-4">Edit Site Info</h6>
      <v-file-input
        v-model="file"
        accept="image/*"
        label="Site Image"
        prepend-icon=""
      >
        <template #prepend-inner>
          <v-sheet class="my-1 mr-1">
            <v-img
              v-if="siteImage || fileBlobUrl"
              width="120px"
              height="120px"
              cover
              :src="fileBlobUrl ? fileBlobUrl : `https://${IPFS_GATEWAY}/ipfs/${siteImage}`"
            ></v-img>
            <v-sheet
              v-else
              width="120px"
              height="120px"
              class="d-flex"
              border
            >
              <span class="ma-auto text-caption text-medium-emphasis">No image.</span>
            </v-sheet>
          </v-sheet>
        </template>
      </v-file-input>
      <v-text-field
        v-model="siteName"
        label="Site Name"
      ></v-text-field>
      <v-textarea
        v-model="siteDescription"
        variant="solo-filled"
        label="Site Description"
      ></v-textarea>
      <v-btn
        class="mt-2"
        color="primary"
        block
        @click="handleOnSave"
      >
        Save
      </v-btn>
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import { type Ref, ref, watch } from 'vue';
import { IPFS_GATEWAY } from '/@/constants/ipfs';

const file: Ref<File | null> = ref(null);
const fileBlobUrl: Ref<string | null> = ref(null);

const siteName: Ref<string | null> = ref(null);
const siteDescription: Ref<string | null> = ref(null);
const siteImage: Ref<string | null> = ref(null);

watch(file, (v) => {
  if (!v) {
    fileBlobUrl.value = null;
    return;
  }
  fileBlobUrl.value = URL.createObjectURL(v);
});

function handleOnSave(){};
</script>
