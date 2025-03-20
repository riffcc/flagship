<template>
  <v-form
    ref="formRef"
    :disabled="isLoading"
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
      :rules="[rules.required, rules.isValidCid]"
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
      :rules="[rules.isValidCid]"
    />
    <v-text-field
      v-model="releaseItem.cover"
      label="Cover Image CID (Optional)"
      :rules="[rules.isValidCid]"
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
            placeholder="Values separated by comma"
          >
            <template #append-inner>
              <v-tooltip location="top">
                <template #activator="{props: tagsTooltipProps}">
                  <v-icon
                    size="small"
                    v-bind="tagsTooltipProps"
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
                <template #activator="{props: musicBrainzIDTooltipProps}">
                  <v-icon
                    size="small"
                    v-bind="musicBrainzIDTooltipProps"
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
</template>

<script setup lang="ts">
import {consts, type types as orbiterTypes} from '@riffcc/orbiter';
import {cid} from 'is-ipfs';
import {computed, ref} from 'vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import type { ReleaseItem, PartialReleaseItem } from '/@/@types/release';

const props = defineProps<{
  initialData?: PartialReleaseItem;
  mode?: 'create' | 'edit';
}>();

const emit = defineEmits<{
  (e: 'submit', data: unknown): void;
  (e: 'update:success', message: string): void;
  (e: 'update:error', message: string): void;
}>();

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
  isValidCid: (v: string) => !v || cid(v) || 'Please enter a valid CID.',
};
const isLoading = ref(false);

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
    const data = readyToSave.value;
    const release = {
      [consts.RELEASES_AUTHOR_COLUMN]: data.author,
      [consts.RELEASES_CATEGORY_COLUMN]: data.category,
      [consts.RELEASES_FILE_COLUMN]: data.contentCID,
      [consts.RELEASES_METADATA_COLUMN]: JSON.stringify(data.metadata),
      [consts.RELEASES_NAME_COLUMN]: data.name,
      [consts.RELEASES_THUMBNAIL_COLUMN]: data.thumbnail,
      [consts.RELEASES_COVER_COLUMN]: data.cover,
    };
    if (props.mode === 'edit' && props.initialData?.id) {
      await orbiter.editRelease({
        releaseId: props.initialData.id,
        release,
      });
    } else {
      await orbiter.addRelease(release);
    }
    emit('submit', data);
    emit('update:success', 'Release saved successfully!');
    clearForm();
  } catch (error) {
    console.error('Error saving release:', error);
    emit('update:error', 'Error saving release. Please try again later.');
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
