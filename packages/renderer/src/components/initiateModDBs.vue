<template>
  <v-dialog
    v-model="dialog"
    persistent
    max-width="800"
  >
    <v-card>
      <v-card-title>
        {{ generatedSiteId ? 'Site is now configured' : 'Site not configured' }}
      </v-card-title>
      <v-card-text v-if="!generatedSiteId">
        Each instance of Orbiter must be compiled with a unique site configuration. Click below to
        configure the site. (This can take a while.)
        <v-alert
          v-if="development"
          class="mt-4"
          title="You seem to be in development mode"
          text="This is fine if you're just testing things out. If you're trying to deploy a development server, however, be aware that Orbiter account credentials are linked to the browser and the site domain name. This means that the site will be configured with your localhost address as the root moderator, which is probably not what you want. You should instead deploy the site to your production domain name first, open this page under that domain name (or in the installable Electron app version of your site) and then configure and recompile and deploy the site."
          type="warning"
          variant="tonal"
        />
      </v-card-text>
      <v-card-text v-else>
        Site configured! Be sure to copy the code below into a <code>.env</code> file and place it
        at the root of your Orbiter project.
        <v-textarea
          class="mt-4"
          :value="envFileText"
          :append-inner-icon="textCopied ? 'mdi-check' : 'mdi-content-copy'"
          readonly
          variant="outlined"
          @click:append-inner="copyGeneratedEnvFile"
        />
      </v-card-text>
      <v-card-actions v-if="!generatedSiteId">
        <v-btn
          color="primary mx-auto"
          prepend-icon="mdi-creation"
          variant="outlined"
          :loading="generatingDb"
          @click="generateDb"
        >
          Configure site
        </v-btn>
        <v-btn
          color="primary mx-auto"
          variant="outlined"
          prepend-icon="mdi-exit"
          @click="() => (status = 'static')"
        >
          View in dev (static data) mode
        </v-btn>
      </v-card-actions>
      <v-card-actions v-else>
        <v-spacer />
        <v-btn
          color="primary"
          append-icon="mdi-download"
          :loading="generatingEnvFile"
          @click="downloadEnvFile"
        >
          Download .env file
        </v-btn>
        <v-btn
          color="primary"
          append-icon="mdi-export"
          @click="acceptNewModDb"
        >
          Close and enter site
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import type {types as orbiterTypes} from '@riffcc/orbiter';

import {computed, ref} from 'vue';

import {suivre as follow} from '@constl/vue';
import {saveAs} from 'file-saver';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import {useDevStatus} from '../composables/devStatus';
import {constantCase} from 'change-case';

const {orbiter} = useOrbiter();
const {status} = useDevStatus();

const siteConfigured = follow(({f}) => orbiter.listenForSiteConfigured({f}));
const siteNotConfigured = computed(() => siteConfigured.value === false);

const staticDevMode = computed(() => status.value === 'static');
const dialog = computed(() => siteNotConfigured.value && !staticDevMode.value);

const generatingDb = ref<boolean>(false);
const generatedSiteId = ref<string>();
const generatedVariableIds = ref<orbiterTypes.VariableIds>();

const generatingEnvFile = ref<boolean>(false);

const generateDb = async () => {
  generatingDb.value = true;

  const {siteId, variableIds} = await orbiter.setUpSite();

  // For now, only admins can add content.
  await orbiter.makeSitePrivate();

  generatedSiteId.value = siteId;
  generatedVariableIds.value = variableIds;

  generatingDb.value = false;
};

const development = import.meta.env.DEV;

const envFileText = computed(() => {
  const generatedVariableIdsValue = generatedVariableIds.value;
  if (!generatedVariableIdsValue) throw new Error("This shouldn't happen...");
  const variableIdsList = (Object.keys(generatedVariableIdsValue) as (keyof orbiterTypes.VariableIds)[]).map(
    k => `VITE_${constantCase(k)}_ID=${generatedVariableIdsValue[k]}`,
  );
  const siteId = 'VITE_SITE_ID=' + generatedSiteId.value;

  return (
    '# The address below should be regenerated for each Orbiter site. If you are setting up an independent site, erase the value below and run the site in development mode (`pnpm dev`) to automatically regenerate. \n' +
    siteId +
    '\n' +
    '\n' +
    '# These should ideally stay the same for all Orbiter sites for optimal performance. Only change if you know what you are doing.\n' +
    variableIdsList.join('\n') +
    '\n'
  );
});

const downloadEnvFile = async () => {
  if (!generatedSiteId.value) return;
  generatingEnvFile.value = true;

  saveAs(new Blob([envFileText.value], {type: 'text/plain;charset=utf-8'}), '[erase this bit].env');

  generatingEnvFile.value = false;
};

const textCopied = ref(false);
const copyGeneratedEnvFile = async () => {
  if (navigator.clipboard) {
    await navigator.clipboard.writeText(envFileText.value);
  }
  textCopied.value = true;
};

const acceptNewModDb = async () => {
  if (!generatedSiteId.value || !generatedVariableIds.value) {
    throw new Error('Mod DB and variables not generated.');
  }
};
</script>
