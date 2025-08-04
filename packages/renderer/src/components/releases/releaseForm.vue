<template>
  <v-form
    ref="formRef"
    :disabled="addReleaseMutation.isPending.value"
    class="d-flex flex-column ga-2"
    @submit.prevent="handleOnSubmit"
  >
    <v-text-field
      v-model="releaseItem.name"
      :label="isTVCategory ? 'Episode Name' : 'Name'"
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
    
    <!-- TV Show Fields -->
    <template v-if="isTVCategory">
      <v-autocomplete
        v-model="selectedSeriesId"
        :items="seriesItems"
        :loading="structuresQuery.isLoading.value"
        label="TV Series"
        placeholder="Type to search or create new series..."
        :rules="[rules.required]"
        clearable
        @update:search="seriesSearchText = $event"
      >
        <template v-if="shouldShowCreateSeriesOption" #append-item>
          <v-list-item
            @click="createNewSeries"
            class="text-primary"
          >
            <v-list-item-title>
              <v-icon start>mdi-plus</v-icon>
              Create "{{ seriesSearchText }}"
            </v-list-item-title>
          </v-list-item>
        </template>
      </v-autocomplete>
      
      <v-row v-if="selectedSeriesId">
        <v-col cols="6">
          <v-text-field
            v-model.number="seasonNumber"
            label="Season Number"
            type="number"
            :rules="[rules.required, rules.positiveNumber]"
            @update:model-value="handleSeasonChange"
          />
        </v-col>
        <v-col cols="6">
          <v-text-field
            v-model.number="episodeNumber"
            label="Episode Number"
            type="number"
            :rules="[rules.required, rules.positiveNumber]"
          />
        </v-col>
      </v-row>
    </template>
    
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
            v-if="(fieldConfig as any).options"
            :items="(fieldConfig as any).options"
            :label="formatFieldLabel(fieldName)"
            :model-value="String((releaseItem.metadata && releaseItem.metadata[fieldName]) || '')"
            @update:model-value="(v) => handleChangeMetadataField(fieldName, v)"
          />
          <v-text-field
            v-else
            :label="formatFieldLabel(fieldName)"
            :model-value="String((releaseItem.metadata && releaseItem.metadata[fieldName]) || '')"
            :type="(fieldConfig as any).type || 'text'"
            @update:model-value="(v) => handleChangeMetadataField(fieldName, v)"
          >
            <template #append-inner>
              <v-tooltip
                location="top"
                :text="(fieldConfig as any).description || ''"
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
import { 
  useAddReleaseMutation, 
  useEditReleaseMutation, 
  useContentCategoriesQuery,
  useGetStructuresQuery,
  useAddStructureMutation,
  useEditStructureMutation
} from '/@/plugins/lensService/hooks';
// import { StringMatch, StringMatchMethod } from '@peerbit/document';

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

// TV-specific state
const selectedSeriesId = ref<string>('');
const seriesSearchText = ref<string>('');
const seasonNumber = ref<number>(1);
const episodeNumber = ref<number>(1);
const selectedSeasonId = ref<string>('');

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
  isValidCid: (v: string) => !v || cid(v) || 'Please enter a valid CID.',
  positiveNumber: (v: number) => v > 0 || 'Must be a positive number.',
};

// Check if TV category is selected (moved before structuresQuery to avoid circular dependency)
const isTVCategory = computed(() => {
  const category = contentCategories.value?.find(c => c.id === releaseItem.value.categoryId);
  // Check both categoryId and displayName for TV shows (handle corrupted data)
  const isTV = category?.categoryId === 'tv-shows' || category?.displayName === 'TV Shows';
  return isTV;
});

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


// Fetch structures for TV shows (both series and seasons)
const structuresQuery = useGetStructuresQuery({
  searchOptions: {
    // Fetch all structures, we'll filter them in computed properties
    fetch: 1000,
  },
  enabled: computed(() => {
    const enabled = isTVCategory.value;
    console.log('Structures query enabled:', enabled, 'isTVCategory:', isTVCategory.value);
    return enabled;
  }),
});

// Watch for structure query errors
watch(() => structuresQuery.error.value, (error) => {
  if (error) {
    console.error('Failed to fetch structures:', error);
  }
});

const addStructureMutation = useAddStructureMutation({
  onSuccess: (response) => {
    console.log('Structure created successfully:', response);
  },
  onError: (e) => {
    console.error('Error creating structure:', e);
  },
});

const editStructureMutation = useEditStructureMutation({
  onSuccess: (response) => {
    console.log('Structure updated successfully:', response);
  },
  onError: (e) => {
    console.error('Error updating structure:', e);
  },
});

const contentCategoriesItems = computed(() => {
  if (!contentCategories.value) return [];
  
  // Only show categories created by our own site (not federated ones)
  const ourSiteAddress = import.meta.env.VITE_SITE_ADDRESS;
  
  return contentCategories.value
    .filter(item => item.siteAddress === ourSiteAddress) // Only our site's categories
    .map(item => ({
      id: item.id,
      value: item.id,
      title: item.displayName,
    }));
});

// Get list of series for autocomplete
const seriesItems = computed(() => {
  console.log('Computing series items, structures query state:', {
    isLoading: structuresQuery.isLoading.value,
    isError: structuresQuery.isError.value,
    error: structuresQuery.error.value,
    dataLength: structuresQuery.data.value?.length
  });
  
  if (!structuresQuery.data.value) {
    console.log('No structures data available');
    return [];
  }
  
  console.log('All structures:', structuresQuery.data.value);
  
  const series = structuresQuery.data.value
    .filter((s: any) => s.type === 'series')
    .map((s: any) => ({
      value: s.id,
      title: s.name,
    }));
  console.log('Filtered series for autocomplete:', series);
  return series;
});

// Check if we should show the "Create new series" option
const shouldShowCreateSeriesOption = computed(() => {
  if (!seriesSearchText.value || seriesSearchText.value.length < 2) return false;
  // Check if series already exists
  const exists = structuresQuery.data.value?.some(
    (s: any) => s.name.toLowerCase() === seriesSearchText.value.toLowerCase()
  );
  return !exists;
});

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
    
    // If editing a TV episode, initialize the TV-specific fields
    if (props.initialData.metadata?.seriesId) {
      selectedSeriesId.value = props.initialData.metadata.seriesId as string;
      seasonNumber.value = (props.initialData.metadata.seasonNumber as number) || 1;
      episodeNumber.value = (props.initialData.metadata.episodeNumber as number) || 1;
      selectedSeasonId.value = (props.initialData.metadata.seasonId as string) || '';
      
      console.log('Initialized TV episode fields:', {
        seriesId: selectedSeriesId.value,
        seasonNumber: seasonNumber.value,
        episodeNumber: episodeNumber.value,
        seasonId: selectedSeasonId.value
      });
    }
  }
});

// Ensure metadata is preserved when switching categories
watch(() => releaseItem.value.categoryId, () => {
  if (!releaseItem.value.metadata) {
    releaseItem.value.metadata = {};
  }
});

// Watch for series selection changes
watch(selectedSeriesId, (newId) => {
  console.log('Selected series ID changed:', newId);
  if (newId) {
    // When a series is selected, trigger season handling
    handleSeasonChange();
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

// Create a new TV series
const createNewSeries = async () => {
  if (!seriesSearchText.value) return;
  
  console.log('Creating new series:', seriesSearchText.value);
  
  try {
    const response = await addStructureMutation.mutateAsync({
      name: seriesSearchText.value,
      type: 'series',
      description: '',
      itemIds: [],
    });
    
    console.log('Series creation response:', response);
    
    if (response.success) {
      // The response might have the ID in different fields
      const newSeriesId = response.id || response.hash;
      if (newSeriesId) {
        selectedSeriesId.value = newSeriesId;
        console.log('Series created with ID:', newSeriesId);
        // Refetch structures to include the new series
        await structuresQuery.refetch();
        
        // Clear the search text
        seriesSearchText.value = '';
      } else {
        console.error('No ID returned from series creation');
      }
    } else {
      console.error('Series creation failed:', response);
    }
  } catch (error) {
    console.error('Failed to create series:', error);
    emit('update:error', `Failed to create series: ${error.message}`);
  }
};

// Handle season change - check if we need to create a new season structure
const handleSeasonChange = async () => {
  if (!selectedSeriesId.value || !seasonNumber.value) return;
  
  console.log('Handling season change:', { seriesId: selectedSeriesId.value, seasonNumber: seasonNumber.value });
  
  // Check if season structure already exists
  const seasonName = `Season ${seasonNumber.value}`;
  const existingSeason = structuresQuery.data.value?.find(
    (s: any) => {
      if (s.type !== 'season' || s.parentId !== selectedSeriesId.value) return false;
      
      // Check by metadata seasonNumber first, then by name
      if (s.metadata) {
        try {
          const meta = typeof s.metadata === 'string' ? JSON.parse(s.metadata) : s.metadata;
          if (meta.seasonNumber === seasonNumber.value) return true;
        } catch (e) {
          // Invalid metadata, fall through to name check
        }
      }
      
      return s.name === seasonName;
    }
  );
  
  if (existingSeason) {
    console.log('Found existing season:', existingSeason);
    selectedSeasonId.value = existingSeason.id;
  } else {
    console.log('Creating new season:', seasonName);
    try {
      // Create new season structure
      const response = await addStructureMutation.mutateAsync({
        name: seasonName,
        type: 'season',
        parentId: selectedSeriesId.value,
        order: seasonNumber.value,
        itemIds: [],
        metadata: JSON.stringify({ seasonNumber: seasonNumber.value }),
      });
      
      console.log('Season creation response:', response);
      
      if (response.success) {
        const newSeasonId = response.id || response.hash;
        if (newSeasonId) {
          selectedSeasonId.value = newSeasonId;
          console.log('Season created with ID:', newSeasonId);
          await structuresQuery.refetch();
        } else {
          console.error('No ID returned from season creation');
        }
      } else {
        console.error('Season creation failed:', response);
      }
    } catch (error) {
      console.error('Failed to create season:', error);
    }
  }
};

const handleOnSubmit = async () => {
  if (!readyToSave.value) return;

  const data = readyToSave.value;
  
  // If this is a TV episode, ensure we have the proper structure hierarchy
  if (isTVCategory.value && selectedSeriesId.value) {
    console.log('Processing TV episode submission:', {
      seriesId: selectedSeriesId.value,
      seasonNumber: seasonNumber.value,
      episodeNumber: episodeNumber.value
    });
    
    // Ensure season structure exists
    await handleSeasonChange();
    
    // Add episode metadata
    if (!data.metadata) data.metadata = {};
    data.metadata.seasonNumber = seasonNumber.value;
    data.metadata.episodeNumber = episodeNumber.value;
    data.metadata.seriesId = selectedSeriesId.value;
    data.metadata.seasonId = selectedSeasonId.value;
    
    console.log('Episode metadata set:', data.metadata);
  }

  if (props.mode === 'edit' && data.id) {
    const response = await editReleaseMutation.mutateAsync({
      id: data.id,
      name: data.name!,
      categoryId: data.categoryId!,
      contentCID: data.contentCID!,
      thumbnailCID: data.thumbnailCID,
      metadata: data.metadata,
      siteAddress: data.siteAddress!,
      postedBy: data.postedBy as any,
    });
    
    // If TV episode and successful, add to season's itemIds
    // Use the original episode ID (data.id) not response.id since we're editing
    if (isTVCategory.value && selectedSeasonId.value && response.success) {
      await updateSeasonWithEpisode(data.id);
    }
  } else {
    const response = await addReleaseMutation.mutateAsync({
      name: data.name!,
      categoryId: data.categoryId!,
      contentCID: data.contentCID!,
      thumbnailCID: data.thumbnailCID,
      metadata: data.metadata,
    });
    
    // If TV episode and successful, add to season's itemIds
    if (isTVCategory.value && selectedSeasonId.value && response.success) {
      await updateSeasonWithEpisode(response.id!);
    }
  }
};

// Add episode to season's itemIds
const updateSeasonWithEpisode = async (episodeId: string) => {
  if (!selectedSeasonId.value) return;
  
  // Fetch current season structure
  const seasons = await structuresQuery.refetch();
  const season = seasons.data.value?.find((s: any) => s.id === selectedSeasonId.value);
  
  if (season) {
    // Check if episode is already in itemIds
    const currentItemIds = season.itemIds || [];
    if (!currentItemIds.includes(episodeId)) {
      console.log('Adding episode to season itemIds:', { seasonId: season.id, episodeId });
      // Update season with new episode
      await editStructureMutation.mutateAsync({
        ...season,
        itemIds: [...currentItemIds, episodeId],
      });
      console.log('Episode added to season successfully');
    } else {
      console.log('Episode already in season itemIds');
    }
  } else {
    console.error('Season not found:', selectedSeasonId.value);
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
