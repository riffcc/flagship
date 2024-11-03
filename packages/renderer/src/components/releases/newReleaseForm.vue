<template>
  <v-form
    ref="formRef"
    validate-on="input lazy"
    class="d-flex flex-column ga-2"
    @submit.prevent="handleOnSubmit"
  >
    <v-text-field
      v-model="releaseName"
      label="Name"
      :rules="[rules.required]"
    />
    <v-text-field
      v-model="contentCID"
      label="Content CID"
      :rules="[rules.isValidCid]"
    />
    <v-select
      v-model="releaseCategory"
      :items="orbiter.contentCategories"
      :rules="[rules.required]"
      label="Category"
    />
    <v-text-field
      v-model="author"
      label="Author"
      :rules="[rules.required]"
    />
    <v-text-field
      v-model="thumbnailCID"
      label="Thumbnail CID (Optional)"
    />
    <v-text-field
      v-model="coverCID"
      label="Cover image CID"
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
          v-model="releaseMetadata.description"
          label="Description"
        />
        <v-select
          v-model="releaseMetadata.license"
          :items="licenseTypes"
          label="License"
        />
        <template v-if="releaseCategory == 'music'">
          <v-text-field
            v-model="musicReleaseMetadata.tags"
            label="Tags"
            placeholder="Values sepatared by comma"
          >
            <template #append-inner>
              <v-tooltip
                location="top"
              >
                <template #activator="{ props }">
                  <v-icon
                    size="small"
                    v-bind="props"
                    color="grey-lighten-1"
                    icon="mdi-help-circle-outline"
                  ></v-icon>
                </template>
                <span>Any tags you feel are appropriate for the media - such as rock, country, or pop.</span>
              </v-tooltip>
            </template>
          </v-text-field>
          <v-text-field
            v-model="musicReleaseMetadata.musicBrainzID"
            label="MusicBrainz ID"
          >
            <template #append-inner>
              <v-tooltip
                location="top"
              >
                <template #activator="{ props }">
                  <v-icon
                    size="small"
                    v-bind="props"
                    color="grey-lighten-1"
                    icon="mdi-help-circle-outline"
                  ></v-icon>
                </template>
                <span>If the content has an entry on MusicBrainz, enter it here to pre-fill the rest of this form.</span>
              </v-tooltip>
            </template>
          </v-text-field>
          <v-text-field
            v-model="musicReleaseMetadata.albumTitle"
            label="Album Title"
          />
          <v-text-field
            v-model="musicReleaseMetadata.initialReleaseYear"
            label="Initial Release Year"
          />
          <v-select
            v-model="musicReleaseMetadata.releaseType"
            :items="musicReleaseTypes"
            label="Release Type"
          />
          <v-select
            v-model="musicReleaseMetadata.fileFormat"
            :items="musicFileFormats"
            label="Format"
          />
          <v-text-field
            v-model="musicReleaseMetadata.bitrate"
            label="Bitrate"
          />
          <v-select
            v-model="musicReleaseMetadata.mediaFormat"
            :items="musicMediaFormats"
            label="Media"
          />
        </template>
        <template v-else-if="releaseCategory == 'movie'">
          <v-text-field
            v-model="movieReleaseMetadata.posterCID"
            label="Poster CID"
          />
          <v-text-field
            v-model="movieReleaseMetadata.TMDBID"
            label="TMDB ID"
          />
          <v-text-field
            v-model="movieReleaseMetadata.IMDBID"
            label="IMDB ID"
          />
          <v-select
            v-model="movieReleaseMetadata.releaseType"
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
      :loading="loading"
    />
  </v-form>
</template>

<script setup lang="ts">
import {computed, ref} from 'vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import {cid} from 'is-ipfs';
import {
  consts, type types,
} from '@riffcc/orbiter';
// import convert from 'image-file-resize';

const {orbiter} = useOrbiter();
const formRef = ref();
const openAdvanced = ref<boolean>();

const author = ref<string>();
const contentCID = ref<string>();
const releaseCategory = ref<string>();
const releaseName = ref<string>();
const thumbnailCID = ref<string>();
const coverCID = ref<string>(); // TODO - option to autogenerate this from movie files

const releaseMetadata = ref<types.ReleaseMetadata>({});
const musicReleaseMetadata = ref<types.MusicReleaseMetadata>({});
const movieReleaseMetadata = ref<types.MovieReleaseMetadata>({});

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidCid: (v: string) => cid(v) || 'Please enter a valid CID.',
};
const loading = ref(false);
const readyToSave = computed(() => {
  if (
    contentCID.value &&
    author.value &&
    releaseName.value &&
    releaseCategory.value &&
    coverCID.value &&
    formRef.value.isValid
  ) {
    let metadataValue = releaseMetadata.value;
    if (releaseCategory.value == 'movie') {
      metadataValue = {
        ...metadataValue,
        ...movieReleaseMetadata.value,
      };
    } else if (releaseCategory.value == 'music') {
      metadataValue = {
        ...metadataValue,
        ...musicReleaseMetadata.value,
      };
    }

    return {
      contentCIDValue: contentCID.value,
      authorValue: author.value,
      metadataValue,
      releaseNameValue: releaseName.value,
      releaseCategoryValue: releaseCategory.value,
      coverCIDValue: coverCID.value,
    };
  } else return undefined;
});

const handleOnSubmit = async () => {
  if (!readyToSave.value) return;
  loading.value = true;
  console.log('ON SUBMIT');
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
    [consts.RELEASES_STATUS_COLUMN]: 'pending',
    [consts.RELEASES_COVER_COLUMN]: coverCIDValue,
  });
  clearForm();
  loading.value = false;
};

const clearForm = () => {
  author.value = undefined;
  contentCID.value = undefined;
  releaseCategory.value = undefined;
  releaseName.value = undefined;
  thumbnailCID.value = undefined;
  coverCID.value = undefined;

  releaseMetadata.value = {};
  musicReleaseMetadata.value = {};
  movieReleaseMetadata.value = {};
};

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
