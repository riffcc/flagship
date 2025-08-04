<template>
  <v-form @submit.prevent="handleSubmit">
    <v-select
      v-model="formData.type"
      :items="structureTypes"
      item-title="label"
      item-value="value"
      label="Structure Type"
      :disabled="mode === 'edit'"
      required
    ></v-select>
    
    <v-text-field
      v-model="formData.name"
      label="Name"
      required
    ></v-text-field>
    
    <v-textarea
      v-model="formData.description"
      label="Description"
      rows="3"
    ></v-textarea>
    
    <v-text-field
      v-model="formData.thumbnailCID"
      label="Thumbnail CID"
      placeholder="Enter IPFS CID for thumbnail"
      prepend-icon="$image"
      hint="Upload image to IPFS and paste the CID here"
    ></v-text-field>
    
    <!-- Type-specific fields -->
    <template v-if="formData.type === 'series'">
      <v-divider class="my-4"></v-divider>
      <p class="text-subtitle-2 mb-2">Series Information</p>
      <v-text-field
        v-model.number="formData.metadata.seasons"
        label="Number of Seasons"
        type="number"
        min="1"
      ></v-text-field>
      <v-select
        v-model="formData.metadata.genres"
        :items="tvGenres"
        label="Genres"
        multiple
        chips
        closable-chips
      ></v-select>
      <v-text-field
        v-model="formData.metadata.releaseYear"
        label="Release Year"
        type="number"
        min="1900"
        :max="new Date().getFullYear() + 1"
      ></v-text-field>
    </template>
    
    <template v-else-if="formData.type === 'artist'">
      <v-divider class="my-4"></v-divider>
      <p class="text-subtitle-2 mb-2">Artist Information</p>
      <v-select
        v-model="formData.metadata.genres"
        :items="musicGenres"
        label="Genres"
        multiple
        chips
        closable-chips
      ></v-select>
      <v-text-field
        v-model="formData.metadata.country"
        label="Country"
      ></v-text-field>
      <v-text-field
        v-model="formData.metadata.website"
        label="Website"
        type="url"
      ></v-text-field>
    </template>
    
    <template v-else-if="formData.type === 'season'">
      <v-divider class="my-4"></v-divider>
      <p class="text-subtitle-2 mb-2">Season Information</p>
      <v-autocomplete
        v-model="formData.parentId"
        :items="availableSeries"
        item-title="name"
        item-value="id"
        label="Parent Series"
        required
      ></v-autocomplete>
      <v-text-field
        v-model.number="formData.metadata.seasonNumber"
        label="Season Number"
        type="number"
        min="1"
        required
      ></v-text-field>
      <v-text-field
        v-model.number="formData.metadata.episodeCount"
        label="Number of Episodes"
        type="number"
        min="1"
      ></v-text-field>
    </template>
    
    <template v-else-if="formData.type === 'tag'">
      <v-divider class="my-4"></v-divider>
      <p class="text-subtitle-2 mb-2">Tag Information</p>
      <v-text-field
        v-model="formData.metadata.color"
        label="Tag Color"
        type="color"
      ></v-text-field>
      <v-select
        v-model="formData.metadata.category"
        :items="['genre', 'mood', 'theme', 'style', 'other']"
        label="Tag Category"
      ></v-select>
    </template>
    
    <template v-else-if="formData.type === 'collection'">
      <v-divider class="my-4"></v-divider>
      <p class="text-subtitle-2 mb-2">Collection Information</p>
      <v-select
        v-model="formData.metadata.collectionType"
        :items="['playlist', 'album', 'compilation', 'franchise', 'custom']"
        label="Collection Type"
      ></v-select>
      <v-switch
        v-model="formData.metadata.isPublic"
        label="Public Collection"
      ></v-switch>
      <v-switch
        v-model="formData.metadata.allowContributions"
        label="Allow Contributions"
      ></v-switch>
    </template>
    
    <v-btn
      type="submit"
      color="primary"
      block
      class="mt-4"
      :loading="isSubmitting"
    >
      {{ mode === 'edit' ? 'Update' : 'Create' }} Structure
    </v-btn>
  </v-form>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useAddStructureMutation, useEditStructureMutation, useGetStructuresQuery } from '/@/plugins/lensService/hooks';
import { v4 as uuidv4 } from 'uuid';

const props = defineProps<{
  initialData?: any;
  mode?: 'create' | 'edit';
  initialType?: string;
  parentContext?: { parentId: string; parentName: string };
}>();

const emit = defineEmits<{
  'update:success': [message: string];
  'update:error': [message: string];
}>();

const createMutation = useAddStructureMutation();
const updateMutation = useEditStructureMutation();
const { data: structures } = useGetStructuresQuery();

const isSubmitting = ref(false);

const formData = ref({
  id: '',
  type: 'series',
  name: '',
  description: '',
  thumbnailCID: '',
  metadata: {
    seasons: 1,
    genres: [] as string[],
    releaseYear: new Date().getFullYear(),
    country: '',
    website: '',
    seasonNumber: 1,
    episodeCount: 0,
    color: '#1976D2',
    category: 'other',
    collectionType: 'custom',
    isPublic: true,
    allowContributions: false,
  },
  parentId: '',
});

const structureTypes = [
  { label: 'Artist', value: 'artist' },
  { label: 'Tag', value: 'tag' },
  { label: 'Collection', value: 'collection' },
];

const tvGenres = [
  'Action', 'Adventure', 'Animation', 'Comedy', 'Crime', 'Documentary',
  'Drama', 'Fantasy', 'Horror', 'Mystery', 'Romance', 'Sci-Fi',
  'Thriller', 'Western', 'Reality', 'Game Show', 'Talk Show',
];

const musicGenres = [
  'Rock', 'Pop', 'Hip Hop', 'R&B', 'Electronic', 'Jazz', 'Classical',
  'Country', 'Blues', 'Metal', 'Punk', 'Folk', 'Reggae', 'Latin',
  'World', 'Experimental', 'Ambient', 'Indie',
];

const availableSeries = computed(() => {
  if (!structures.value) return [];
  return structures.value.filter((s: any) => s.type === 'series');
});

onMounted(() => {
  if (props.initialData) {
    formData.value = {
      ...props.initialData,
      metadata: {
        ...formData.value.metadata,
        ...props.initialData.metadata,
      },
    };
  } else {
    // Set initial type if provided
    if (props.initialType) {
      formData.value.type = props.initialType;
    }
    // Set parent context if provided
    if (props.parentContext) {
      formData.value.parentId = props.parentContext.parentId;
    }
  }
});


async function handleSubmit() {
  isSubmitting.value = true;
  
  try {
    const structureData = {
      ...formData.value,
      id: formData.value.id || uuidv4(),
      // Clean up metadata based on type
      metadata: Object.fromEntries(
        Object.entries(formData.value.metadata).filter(([_, v]) => v !== '' && v !== null)
      ),
    };
    
    if (props.mode === 'edit') {
      // For editing, we need to include the immutable properties
      const editData = {
        ...structureData,
        postedBy: props.initialData.postedBy,
        siteAddress: props.initialData.siteAddress,
      };
      await updateMutation.mutateAsync(editData);
      emit('update:success', 'Structure updated successfully');
    } else {
      await createMutation.mutateAsync(structureData);
      emit('update:success', 'Structure created successfully');
    }
    
    // Reset form if creating
    if (props.mode !== 'edit') {
      formData.value = {
        id: '',
        type: 'series',
        name: '',
        description: '',
        thumbnailCID: '',
        metadata: {
          seasons: 1,
          genres: [],
          releaseYear: new Date().getFullYear(),
          country: '',
          website: '',
          seasonNumber: 1,
          episodeCount: 0,
          color: '#1976D2',
          category: 'other',
          collectionType: 'custom',
          isPublic: true,
          allowContributions: false,
        },
        parentId: '',
      };
    }
  } catch (error) {
    console.error('Error saving structure:', error);
    emit('update:error', 'Failed to save structure');
  } finally {
    isSubmitting.value = false;
  }
}
</script>