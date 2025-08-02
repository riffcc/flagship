<template>
  <v-form
    ref="formRef"
    :disabled="addReleaseMutation.isPending.value"
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
      v-model="releaseItem.categoryId"
      :items="contentCategoriesItems"
      :rules="[rules.required]"
      label="Category"
    />
    <v-text-field
      v-model="releaseItem.thumbnailCID"
      label="Thumbnail CID (Optional)"
      :rules="[rules.isValidCid]"
    />
    <v-dialog
      v-model="openAdvanced"
      width="auto"
    >
      <template #activator="{props: activatorProps}">
        <v-btn
          v-bind="activatorProps"
          :disabled="!Boolean(selectedContentCategory)"
          rounded="0"
          text="Advanced"
          variant="outlined"
          class="mb-4"
          block
        ></v-btn>
      </template>
      <v-sheet
        v-if="selectedContentCategory && releaseItem.metadata"
        width="480px"
        max-height="620px"
        class="pa-8 ma-auto"
      >
        <p class="text-subtitle mb-6 text-center">
          Please fill out any extra information about the content that might be useful.
        </p>
        <div
          v-for="[fieldName, fieldConfig] in Object.entries(selectedContentCategory)"
          :key="fieldName"
        >
          <v-select
            v-if="fieldConfig.options"
            :items="fieldConfig.options"
            :label="formatFieldLabel(fieldName)"
            :model-value="String((releaseItem.metadata && releaseItem.metadata[fieldName]) || '')"
            @update:model-value="(v) => handleChangeMetadataField(fieldName, v)"
          />
          <v-text-field
            v-else
            :label="formatFieldLabel(fieldName)"
            :model-value="String((releaseItem.metadata && releaseItem.metadata[fieldName]) || '')"
            :type="fieldConfig.type || 'text'"
            @update:model-value="(v) => handleChangeMetadataField(fieldName, v)"
          >
            <template #append-inner>
              <v-tooltip
                location="top"
                :text="fieldConfig.description || ''"
              >
                <template #activator="{props: tooltipProps}">
                  <v-icon
                    size="small"
                    v-bind="tooltipProps"
                    color="grey-lighten-1"
                    icon="$help-circle-outline"
                  ></v-icon>
                </template>
              </v-tooltip>
            </template>
          </v-text-field>
        </div>
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
      :disabled="!readyToSave || addReleaseMutation.isPending.value"
      :loading="addReleaseMutation.isPending.value"
    />
  </v-form>
</template>

<script setup lang="ts">
import {cid} from 'is-ipfs';
import {computed, onMounted, ref, watch} from 'vue';
import type { ReleaseItem } from '/@/types';
import type { ContentCategoryMetadataField, ReleaseData } from '@riffcc/lens-sdk';
import { useAddReleaseMutation, useEditReleaseMutation, useContentCategoriesQuery } from '/@/plugins/lensService/hooks';

const props = defineProps<{
  initialData?: ReleaseItem;
  mode?: 'create' | 'edit';
}>();

const emit = defineEmits<{
  (e: 'submit', data: ReleaseData): void;
  (e: 'update:success', message: string): void;
  (e: 'update:error', message: string): void;
}>();

const {
  data: contentCategories,
} = useContentCategoriesQuery();


const formRef = ref();
const openAdvanced = ref<boolean>();

const releaseItem = ref<Partial<ReleaseItem>>({});

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidCid: (v: string) => !v || cid(v) || 'Please enter a valid CID.',
};

const addReleaseMutation = useAddReleaseMutation({
  onSuccess: () => {
    emit('update:success', 'Release added successfully!');
    clearForm();
  },
  onError: (e) => {
    console.error('Error on adding release:', e);
    emit('update:error', `Error on adding release: ${e.message.slice(0, 200)}`);
  },
});

const editReleaseMutation = useEditReleaseMutation({
  onSuccess: () => {
    emit('update:success', 'Release edited successfully!');
    clearForm();
  },
  onError: (e) => {
    console.error('Error in editing release:', e);
    emit('update:error', `Error on editing release: ${e.message.slice(0, 200)}`);
  },
});


const contentCategoriesItems = computed(() => contentCategories.value?.map(item => ({
  id: item.id,
  value: item.id,
  title: item.displayName,
})));

const selectedContentCategory = computed(() => {
  if (!contentCategories.value || !releaseItem.value.categoryId) {
    console.log('Advanced button disabled: no categories or categoryId', {
      hasCategories: !!contentCategories.value,
      categoryId: releaseItem.value.categoryId
    });
    return null;
  }
  
  const targetItem = contentCategories.value.find(item => item.id === releaseItem.value.categoryId);
  if (!targetItem || !targetItem.metadataSchema) {
    console.log('Advanced button disabled: no matching category or metadataSchema', {
      targetItem,
      categoryId: releaseItem.value.categoryId
    });
    return null;
  }
  
  // metadataSchema should already be parsed by the query hook
  // If it's still a string, parse it
  if (typeof targetItem.metadataSchema === 'string') {
    try {
      const parsedSchema = JSON.parse(targetItem.metadataSchema);
      return parsedSchema;
    } catch (e) {
      console.error('Failed to parse metadata schema:', e, targetItem.metadataSchema);
      return null;
    }
  }
  
  return targetItem.metadataSchema;
});

const handleChangeMetadataField = (fieldName: string, value: string | null) => {
  if (!releaseItem.value.metadata) {
    releaseItem.value.metadata = {};
  }
  
  // Only update fields that are defined in the schema
  if (selectedContentCategory.value && fieldName in selectedContentCategory.value) {
    releaseItem.value.metadata = {
      ...releaseItem.value.metadata,
      [fieldName]: value || '',
    };
  }
};

onMounted(() => {
  if(props.initialData) {
    releaseItem.value = {
      ...releaseItem.value,
      ...props.initialData,
      metadata: props.initialData.metadata || {},
    };
  }
});

// Ensure metadata is preserved when switching categories
watch(() => releaseItem.value.categoryId, () => {
  if (!releaseItem.value.metadata) {
    releaseItem.value.metadata = {};
  }
});

const readyToSave = computed(() => {
  if (
    releaseItem.value.name &&
    releaseItem.value.contentCID &&
    releaseItem.value.categoryId &&
    formRef.value.isValid
  ) {
    return releaseItem.value;
  }
  return undefined;
});

const handleOnSubmit = () => {
  if (!readyToSave.value) return;

  const data = readyToSave.value;

  if (props.mode === 'edit' && data.id) {
    editReleaseMutation.mutate({
    id: data.id,
    name: data.name!,
    categoryId: data.categoryId!,
    contentCID: data.contentCID!,
    thumbnailCID: data.thumbnailCID,
    metadata: data.metadata,
    siteAddress: data.siteAddress!,
    postedBy: data.postedBy!,
  });
  } else {
    addReleaseMutation.mutate({
    name: data.name!,
    categoryId: data.categoryId!,
    contentCID: data.contentCID!,
    thumbnailCID: data.thumbnailCID,
    metadata: data.metadata,
  });
  }
};

const fieldLabelMap: Record<string, string> = {
  // Music fields
  description: 'Description',
  totalSongs: 'Total Songs',
  totalDuration: 'Total Duration',
  genres: 'Genres',
  tags: 'Tags',
  musicBrainzID: 'MusicBrainz ID',
  albumTitle: 'Album Title',
  releaseYear: 'Release Year',
  releaseType: 'Release Type',
  fileFormat: 'File Format',
  bitrate: 'Bitrate',
  mediaFormat: 'Media Format',
  
  // Video fields
  title: 'Title',
  duration: 'Duration',
  resolution: 'Resolution',
  format: 'Format',
  uploader: 'Uploader',
  uploadDate: 'Upload Date',
  sourceUrl: 'Source URL',
  
  // Movie fields
  TMDBID: 'TMDB ID',
  IMDBID: 'IMDB ID',
  classification: 'Classification',
  
  // TV Show fields
  seasons: 'Seasons',
  totalEpisodes: 'Total Episodes',
  firstAiredYear: 'First Aired Year',
  status: 'Status',
  network: 'Network',
  averageEpisodeDuration: 'Average Episode Duration',
  
  // Common field
  cover: 'Cover Image CID',
  author: 'Author',
};

const formatFieldLabel = (fieldName: string): string => {
  return fieldLabelMap[fieldName] || fieldName;
};

const clearForm = () => {
  releaseItem.value = {
    id: '',
    name: '',
    contentCID: '',
    categoryId: '',
    metadata: {},
    siteAddress: '',
  };
  formRef.value?.resetValidation();
  formRef.value?.reset();
};
</script>
