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
      <v-card-title>{{ newSite ? 'Add trusted site' : 'Edit trusted site' }}</v-card-title>
      <v-card-text>
        <v-text-field
          v-model="siteInfo"
          variant="outlined"
          label="Site info"
        ></v-text-field>
      </v-card-text>
      <v-card-actions>
        <v-btn
          color="primary"
          :loading="saving"
          :disabled="!readyToSave"
          @click="save"
        >
          {{ newSite ? 'Trust site' : 'Save changes' }}
        </v-btn>
        <v-btn @click="clearDialog">Cancel</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import {computed, ref} from 'vue';

import type {tableaux} from '@constl/ipa';
import {watchEffect} from 'vue';
import type {types as orbiterTypes} from '@riffcc/orbiter';
import {useOrbiter} from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();

const props = defineProps<{site?: tableaux.élémentDonnées<orbiterTypes.TrustedSite>}>();

const dialog = ref(false);
const saving = ref(false);

// Allow same component to be used for both creating and editing trusted sites
const newSite = computed(() => !props.site);

const readyToSave = computed(() => {
  if (siteId.value && siteName.value) {
    return {siteId: siteId.value, siteName: siteName.value};
  }
  return undefined;
});

const siteInfo = ref<string>();
const siteId = ref<string>();
const siteName = ref<string>();

watchEffect(() => {
  if (siteInfo.value) {
    const info = JSON.parse(atob(siteInfo.value)) as orbiterTypes.TrustedSite;
    siteId.value = info.siteId;
    siteName.value = info.siteName;
  }
});

const save = async () => {
  const info = readyToSave.value;
  if (!info) return;

  saving.value = true;

  const {siteId: siteIdValue, siteName: siteNameValue} = info;

  if (newSite.value) {
    await orbiter.trustSite({
      siteId: siteIdValue,
      siteName: siteNameValue,
    });
  } else {
    await orbiter.editTrustedSite({
      elementId: props.site!.id,
      site: {siteId: siteIdValue, siteName: siteNameValue},
    });
  }

  saving.value = false;
  clearDialog();
};

const clearDialog = () => {
  dialog.value = false;

  siteInfo.value = undefined;

  saving.value = false;
};
</script>
