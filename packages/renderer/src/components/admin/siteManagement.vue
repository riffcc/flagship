<template>
  <v-container>
    <v-sheet
      class="px-6 py-4 mx-auto"
      max-width="448px"
    >
      <v-list-item
        v-if="siteAddress && siteAddress.length > 0"
        class="px-0"
        :title="`Site Address: ${siteAddress.slice(0, 17)}...${siteAddress.slice(-10)}`"
      >
        <template
          v-if="showDefederation"
          #prepend
        >
          <v-menu>
            <template #activator="{ props }">
              <v-btn
                v-bind="props"
                icon="$circle"
                variant="text"
                density="comfortable"
                size="x-small"
                class="mr-2"
                :color="getSiteColor(siteAddress)"
              />
            </template>
            <v-color-picker
              v-model="selectedColors[siteAddress]"
              @update:model-value="saveColor(siteAddress, $event)"
            />
          </v-menu>
        </template>
        <template #append>
          <v-tooltip
            text="Copy Site ID"
            location="bottom"
          >
            <template #activator="{ props: tooltipProps }">
              <v-btn
                v-bind="tooltipProps"
                :icon="getIcon(siteAddress)"
                :color="getColor(siteAddress)"
                variant="text"
                density="comfortable"
                size="x-small"
                @click="copy(siteAddress, siteAddress)"
              ></v-btn>
            </template>
          </v-tooltip>
        </template>
      </v-list-item>
      <v-divider class="mt-2"></v-divider>
      <h3 class="mt-4 mb-2">Edit Site Info</h3>
      <v-file-input
        v-model="file"
        accept="image/*"
        label="Image"
        prepend-icon=""
      >
        <template #prepend-inner>
          <v-sheet class="my-1 mr-1">
            <v-img
              v-if="fileBlobUrl || siteImage"
              width="120px"
              height="120px"
              cover
              :src="parseUrlOrCid(fileBlobUrl || siteImage)"
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
        label="Name"
      ></v-text-field>
      <v-textarea
        v-model="siteDescription"
        variant="solo-filled"
        label="Description"
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
import { computed, type Ref, ref, watch } from 'vue';
import { parseUrlOrCid } from '/@/utils';

import { useSiteColors } from '/@/composables/siteColors';
import { useShowDefederation } from '/@/composables/showDefed';
import { useCopyToClipboard } from '/@/composables/copyToClipboard';
import { useLensService } from '/@/plugins/lensService/hooks';

const file: Ref<File | undefined> = ref();
const fileBlobUrl: Ref<string | undefined> = ref();

const siteName: Ref<string | undefined> = ref();
const siteDescription: Ref<string | undefined> = ref();
const siteImage: Ref<string | undefined> = ref();

watch(file, (v) => {
  if (!v) {
    fileBlobUrl.value = undefined;
    return;
  }
  fileBlobUrl.value = URL.createObjectURL(v);
});

function handleOnSave(){};
const {getSiteColor, saveColor, selectedColors} = useSiteColors();
const {showDefederation} = useShowDefederation();
const { lensService } = useLensService();
const siteAddress = computed(() => {
  return lensService?.siteProgram?.address;
});
const { copy, getIcon, getColor } = useCopyToClipboard();
</script>
