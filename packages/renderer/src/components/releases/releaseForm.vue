<template>
  <v-form
    ref="formRef"
    validate-on="input lazy"
    class="d-flex flex-column ga-2"
    @submit.prevent="handleOnSubmit"
  >
    <v-text-field
      v-model="releaseItem.name"
      label="Name"
      :rules="[rules.required]"
    />
    <v-text-field
      v-model="releaseItem.contentCID"
      label="Content CID"
      :rules="[rules.isValidCid]"
    />
    <v-select
      v-model="releaseItem.category"
      :items="consts.CONTENT_CATEGORIES"
      :rules="[rules.required]"
      label="Category"
    />
    <v-text-field
      v-model="releaseItem.author"
      label="Author"
      :rules="[rules.required]"
    />
    <v-text-field
      v-model="releaseItem.thumbnail"
      label="Thumbnail CID (Optional)"
    />
    <v-text-field
      v-model="releaseItem.cover"
      label="Cover Image CID (Optional)"
    />
    <v-dialog
      v-model="openAdvanced"
      width="auto"
    >
      <template #activator="{props: activatorProps}">
        <v-btn
          v-bind="activatorProps"
          rounded="0"
          text="Advanced"
          variant="outlined"
          class="mb-4"
          block
        ></v-btn>
      </template>
      <v-sheet
        width="480px"
        max-height="620px"
        class="pa-8 ma-auto"
      >
        <p class="text-subtitle mb-6 text-center">
          Please fill out any extra information about the content that might be useful.
        </p>
        <v-text-field
          v-model="releaseItem.metadata.description"
          label="Description"
        />
        <v-select
          v-model="releaseItem.metadata.license"
          :items="licenseTypes"
          label="License"
        />
        <template v-if="releaseItem.category === 'music'">
          <v-text-field
            v-model="(releaseItem.metadata as orbiterTypes.MusicReleaseMetadata).tags"
            label="Tags"
            placeholder="Values sepatared by comma"
          >
            <template #append-inner>
              <v-tooltip location="top">
                <template #activator="{props}">
                  <v-icon
                    size="small"
                    v-bind="props"
                    color="grey-lighten-1"
                    icon="mdi-help-circle-outline"
                  ></v-icon>
                </template>
                <span>Any tags you feel are appropriate for the media - such as rock, country, or
                  pop.</span>
              </v-tooltip>
            </template>
          </v-text-field>
          <v-text-field
            v-model="(releaseItem.metadata as orbiterTypes.MusicReleaseMetadata).musicBrainzID"
            label="MusicBrainz ID"
          >
            <template #append-inner>
              <v-tooltip location="top">
                <template #activator="{props}">
                  <v-icon
                    size="small"
                    v-bind="props"
                    color="grey-lighten-1"
                    icon="mdi-help-circle-outline"
                  ></v-icon>
                </template>
                <span>If the content has an entry on MusicBrainz, enter it here to pre-fill the rest of
                  this form.</span>
              </v-tooltip>
            </template>
          </v-text-field>
          <v-text-field
            v-model="(releaseItem.metadata as orbiterTypes.MusicReleaseMetadata).albumTitle"
            label="Album Title"
          />
          <v-text-field
            v-model="(releaseItem.metadata as orbiterTypes.MusicReleaseMetadata).releaseYear"
            label="Release Year"
          />
          <v-select
            v-model="(releaseItem.metadata as orbiterTypes.MusicReleaseMetadata).releaseType"
            :items="musicReleaseTypes"
            label="Release Type"
          />
          <v-select
            v-model="(releaseItem.metadata as orbiterTypes.MusicReleaseMetadata).fileFormat"
            :items="musicFileFormats"
            label="Format"
          />
          <v-text-field
            v-model="(releaseItem.metadata as orbiterTypes.MusicReleaseMetadata).bitrate"
            label="Bitrate"
          />
          <v-select
            v-model="(releaseItem.metadata as orbiterTypes.MusicReleaseMetadata).mediaFormat"
            :items="musicMediaFormats"
            label="Media"
          />
        </template>
        <template v-else-if="releaseItem.category === 'movie'">
          <v-text-field
            v-model="(releaseItem.metadata as orbiterTypes.MovieReleaseMetadata).posterCID"
            label="Poster CID"
          />
          <v-text-field
            v-model="(releaseItem.metadata as orbiterTypes.MovieReleaseMetadata).TMDBID"
            label="TMDB ID"
          />
          <v-text-field
            v-model="(releaseItem.metadata as orbiterTypes.MovieReleaseMetadata).IMDBID"
            label="IMDB ID"
          />
          <v-select
            v-model="(releaseItem.metadata as orbiterTypes.MovieReleaseMetadata).releaseType"
            :items="movieReleaseTypes"
            label="Media"
          />
        </template>
        <v-btn
          rounded="0"
          text="Save"
          color="primary"
          block
          @click="openAdvanced = false"
        />
      </v-sheet>
    </v-dialog>
    <v-btn
      rounded="0"
      color="primary"
      type="submit"
      block
      text="Submit"
      :disabled="!readyToSave"
      :is-loading="isLoading"
    />
  </v-form>
  <v-alert
    v-if="resultDetails"
    class="mt-2"
    :text="resultDetails.message"
    :type="resultDetails.variant"
  />
</template>

<script setup lang="ts">
import {consts, type types as orbiterTypes} from '@riffcc/orbiter';
import {cid} from 'is-ipfs';
import {computed, ref, watch} from 'vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import type { ReleaseItem, PartialReleaseItem } from '/@/@types/release';

const {orbiter} = useOrbiter();
const formRef = ref();
const openAdvanced = ref<boolean>();

const releaseItem = ref<ReleaseItem>({
  name: '',
  contentCID: '',
  category: '',
  author: '',
  metadata: {},
});

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidCid: (v: string) => cid(v) || 'Please enter a valid CID.',
};
const isLoading = ref(false);
const resultDetails = ref<{
  message: string;
  variant: 'success' | 'error'
} | null>(null);
const readyToSave = computed(() => {
  if (
    releaseItem.value.name &&
    releaseItem.value.contentCID &&
    releaseItem.value.category &&
    releaseItem.value.author &&
    formRef.value.isValid
  ) {
    return releaseItem.value;
  }
  return undefined;
});

const handleOnSubmit = async () => {
  if (!readyToSave.value) return;
  isLoading.value = true;
  try {
    const {
      contentCIDValue,
      authorValue,
      metadataValue,
      releaseNameValue,
      releaseCategoryValue,
      coverCIDValue,
    } = readyToSave.value;
    await orbiter.addRelease({
      [consts.RELEASES_AUTHOR_COLUMN]: authorValue,
      [consts.RELEASES_CATEGORY_COLUMN]: releaseCategoryValue,
      [consts.RELEASES_FILE_COLUMN]: contentCIDValue,
      [consts.RELEASES_METADATA_COLUMN]: JSON.stringify(metadataValue),
      [consts.RELEASES_NAME_COLUMN]: releaseNameValue,
      [consts.RELEASES_THUMBNAIL_COLUMN]: thumbnailCID.value,
      [consts.RELEASES_COVER_COLUMN]: coverCIDValue,
    });
    resultDetails.value = {
      message: 'Release uploaded successfully, yay!',
      variant: 'success',
    };
    clearForm();
  } catch (error) {
    console.log('error uploading release', error);
    resultDetails.value = {
      message: 'Error uploading release. Please try again later.',
      variant: 'error',
    };
  } finally {
    isLoading.value = false;
  }
};

const clearForm = () => {
  releaseItem.value = {
    name: '',
    contentCID: '',
    category: '',
    author: '',
    metadata: {},
  };
};

watch(resultDetails, (v) => {
  if (v) {
    setTimeout(() => {
      resultDetails.value = null;
    }, 10000);
  }
});

const licenseTypes = ['CC BY', 'CC BY-NC', 'CC BY-NC-ND'];

const musicReleaseTypes = [
  'Album',
  'Soundtrack',
  'EP',
  'Anthology',
  'Compilation',
  'Single',
  'Live Album',
  'Remix',
  'Bootleg',
  'Interview',
  'Mixtape',
  'Demo',
  'Concert Recording',
  'DJ Mix',
  'Unknown',
];

const musicFileFormats = ['MP3', 'FLAC', 'AAC', 'AC3', 'DTS'];

const musicMediaFormats = ['CD', 'DVD', 'Vinyl', 'Soundboard', 'SACD', 'DAT', 'WEB', 'Blu-Ray'];

const movieReleaseTypes = [
  'Feature Film',
  'Short Film',
  'Miniseries',
  'Stand-up Comedy',
  'Live Performance',
  'Movie Collection',
];
</script>
