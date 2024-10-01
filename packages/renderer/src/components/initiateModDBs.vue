<template>
  <v-dialog
    v-model="siteNotConfigured"
    persistent
    max-width="800"
  >
    <v-card>
      <v-card-title>
        {{ generatedSiteId ? 'Site is now configured' : 'Site not configured' }}
      </v-card-title>
      <v-card-text v-if="!generatedSiteId">
        Each instance of Orbiter.CC must be compiled with a unique site configuration. Click below
        to configure the site. (This can take a while.)
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
        at the root of your Orbiter.CC project.
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
      </v-card-actions>
      <v-card-actions v-else>
        <v-spacer />
        <v-btn
          color="primary"
          prepend-icon="mdi-download"
          :loading="generatingEnvFile"
          @click="downloadEnvFile"
        >
          Download .env file
        </v-btn>
        <v-btn
          color="primary"
          prepend-icon="mdi-export"
          @click="acceptNewModDb"
        >
          Close and enter site
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import type {VariableIds} from '/@/plugins/orbiter/types';

import {computed, ref} from 'vue';

import {suivre as follow} from '@constl/vue';
import {saveAs} from 'file-saver';
import {useOrbiter} from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();

const siteConfigured = follow(({f}) => orbiter.listenForSiteConfigured({f}));
const siteNotConfigured = computed(() => siteConfigured.value === false);

const generatingDb = ref<boolean>(false);
const generatedSiteId = ref<string>();
const generatedVariableIds = ref<VariableIds>();

const generatingEnvFile = ref<boolean>(false);

const generateDb = async () => {
  generatingDb.value = true;

  const {siteId, variableIds} = await orbiter.setUpSite();

  generatedSiteId.value = siteId;
  generatedVariableIds.value = variableIds;

  generatingDb.value = false;
};

const development = import.meta.env.DEV;

const envFileText = computed(() => {
  const trustedSitesSiteIdVar =
    'VITE_TRUSTED_SITES_SITE_ID_VAR_ID=' + generatedVariableIds.value?.trustedSitesSiteIdVariableId;
  const trustedSitesNameVar =
    'VITE_TRUSTED_SITES_NAME_VAR_ID=' + generatedVariableIds.value?.trustedSitesNameVariableId;

  const releasesFileVar =
    'VITE_RELEASES_FILE_VAR_ID=' + generatedVariableIds.value?.releasesFileVar;
  const releasesCategoryVar =
    'VITE_RELEASES_CATEGORY_VAR_ID=' + generatedVariableIds.value?.releasesCategoryVar;
  const releasesAuthorVar =
    'VITE_RELEASES_AUTHOR_VAR_ID=' + generatedVariableIds.value?.releasesAuthorVar;
  const releasesContentNameVar =
    'VITE_RELEASES_CONTENT_NAME_VAR_ID=' + generatedVariableIds.value?.releasesContentNameVar;
  const releasesMetadataVar =
    'VITE_RELEASES_METADATA_VAR_ID=' + generatedVariableIds.value?.releasesMetadataVar;
  const releasesThumbnailVar =
    'VITE_RELEASES_THUMBNAIL_VAR_ID=' + generatedVariableIds.value?.releasesThumbnailVar;

  const collectionsAuthorVar =
    'VITE_COLLECTIONS_AUTHOR_VAR_ID=' + generatedVariableIds.value?.collectionsAuthorVar;
  const collectionsMetadataVar =
    'VITE_COLLECTIONS_METADATA_VAR_ID=' + generatedVariableIds.value?.collectionsMetadataVar;
  const collectionsNameVar =
    'VITE_COLLECTIONS_NAME_VAR_ID=' + generatedVariableIds.value?.collectionsNameVar;
  const collectionsReleasesVar =
    'VITE_COLLECTIONS_RELEASES_VAR_ID=' + generatedVariableIds.value?.collectionsReleasesVar;
  const collectionsThumbnailVar =
    'VITE_COLLECTIONS_THUMBNAIL_VAR_ID=' + generatedVariableIds.value?.collectionsThumbnailVar;
  const collectionsCategoryVar =
    'VITE_COLLECTIONS_CATEGORY_VAR_ID=' + generatedVariableIds.value?.collectionsCategoryVar;


  const featuredReleasesReleaseIdVar =
  'VITE_FEATURED_RELEASES_RELEASE_ID_VAR_ID=' + generatedVariableIds.value?.featuredReleasesReleaseIdVar;
  const featuredReleasesStartTimeVar =
  'VITE_FEATURED_RELEASES_START_TIME_VAR_ID=' + generatedVariableIds.value?.featuredReleasesStartTimeVar;
  const featuredReleasesEndTimeVar =
  'VITE_FEATURED_RELEASES_END_TIME_VAR_ID=' + generatedVariableIds.value?.featuredReleasesEndTimeVar;

  const siteId = 'VITE_SITE_ID=' + generatedSiteId.value;

  return (
    '# The address below should be regenerated for each Orbiter.CC site. If you are setting up an independent site, erase the value below and run the site in development mode (`pnpm dev`) to automatically regenerate. \n' +
    siteId +
    '\n' +
    '\n' +
    '# These should ideally stay the same for all Orbiter.CC sites for optimal performance. Only change if you know what you are doing.\n' +
    trustedSitesSiteIdVar +
    '\n' +
    trustedSitesNameVar +
    '\n' +
    releasesFileVar +
    '\n' +
    releasesCategoryVar +
    '\n' +
    releasesAuthorVar +
    '\n' +
    releasesContentNameVar +
    '\n' +
    releasesMetadataVar +
    '\n' +
    releasesThumbnailVar +
    '\n' +
    collectionsAuthorVar +
    '\n' +
    collectionsMetadataVar +
    '\n' +
    collectionsNameVar +
    '\n' +
    collectionsReleasesVar +
    '\n' +
    collectionsThumbnailVar +
    '\n' +
    collectionsCategoryVar +
    '\n' +
    featuredReleasesReleaseIdVar +
    '\n' +
    featuredReleasesStartTimeVar +
    '\n' +
    featuredReleasesEndTimeVar +
    '\n'
  );
});

const downloadEnvFile = async () => {
  if (!generatedSiteId.value) return;
  generatingEnvFile.value = true;

  saveAs(envFileText.value, '[erase this bit].env');

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
