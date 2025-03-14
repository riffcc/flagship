<template>
  <v-container>
    <v-sheet
      class="px-6 py-4 mx-auto"
      max-width="448px"
    >
      <v-list-item class="px-0 mb-4">
        <template
          v-if="showDefederation"
          #prepend
        >
          <v-menu>
            <template #activator="{ props }">
              <v-btn
                v-bind="props"
                icon="mdi-circle"
                variant="text"
                density="compact"
                size="x-small"
                class="mr-2"
                :color="getSiteColor(orbiter.siteId)"
              />
            </template>
            <v-color-picker
              v-model="selectedColors[orbiter.siteId]"
              @update:model-value="saveColor(orbiter.siteId, $event)"
            />
          </v-menu>
        </template>

        <p class="text-body">Site ID: {{ `${orbiter.siteId.slice(0, 17)}...${orbiter.siteId.slice(-10)}` }}</p>
        <template #append>
          <v-btn
            icon="mdi-clipboard-multiple-outline"
            variant="text"
            density="comfortable"
            size="small"
            @click="copyText(orbiter.siteId)"
          ></v-btn>
        </template>
      </v-list-item>

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
        color="primary"
        text="Save"
        block
        @click="handleOnSave"
      ></v-btn>
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import { type Ref, ref, watch } from 'vue';
import { IPFS_GATEWAY } from '/@/constants/ipfs';
import { useOrbiter } from '/@/plugins/orbiter/utils';
import { useSiteColors } from '/@/composables/siteColors';
import { copyText } from '/@/utils';
import { useShowDefederation } from '/@/composables/showDefed';

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
const {orbiter} = useOrbiter();
const {getSiteColor, saveColor, selectedColors} = useSiteColors();
const {showDefederation} = useShowDefederation();
</script>
