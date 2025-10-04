<template>
  <v-form
    ref="formRef"
    :disabled="mutation.isPending.value"
    class="d-flex flex-column ga-2"
    @submit.prevent="handleOnSubmit"
  >
    <v-text-field
      v-model="artistData.name"
      label="Artist Name"
      :rules="[rules.required]"
    />

    <v-text-field
      v-model="artistData.thumbnailCID"
      label="Artist Photo/Avatar URL or CID"
    />

    <v-textarea
      v-model="artistData.metadata.bio"
      label="Biography"
      rows="4"
      hint="Tell us about this artist"
    />

    <v-text-field
      v-model="artistData.metadata.genre"
      label="Genre"
      hint="e.g. Indie Folk, Rock, Electronic"
    />

    <v-text-field
      v-model="artistData.metadata.formed"
      label="Formed/Active"
      hint="e.g. 2010s, 2015-2020, 1990"
    />

    <v-combobox
      v-model="selectedRelatedArtists"
      :items="availableArtists"
      :loading="artistsQuery.isLoading.value"
      label="Related Artists"
      multiple
      chips
      closable-chips
      hint="Add related artists (bands, collaborations, side projects)"
    >
      <template v-slot:chip="{ props, item }">
        <v-chip
          v-bind="props"
          :text="item.title"
        ></v-chip>
      </template>
    </v-combobox>

    <v-btn
      rounded="0"
      color="primary"
      type="submit"
      block
      :disabled="!isValid || mutation.isPending.value"
      :loading="mutation.isPending.value"
    >
      {{ mode === 'edit' ? 'Update Artist' : 'Create Artist' }}
    </v-btn>
  </v-form>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useEditReleaseMutation, useAddReleaseMutation, useGetReleasesQuery } from '/@/plugins/lensService/hooks';
import type { ReleaseItem } from '/@/types';

const props = defineProps<{
  initialData?: Partial<ReleaseItem>;
  mode?: 'create' | 'edit';
}>();

const emit = defineEmits<{
  (e: 'update:success', message: string): void;
  (e: 'update:error', message: string): void;
}>();

const formRef = ref();
const editMutation = useEditReleaseMutation();
const addMutation = useAddReleaseMutation();
const mutation = computed(() => props.mode === 'edit' ? editMutation : addMutation);

// Query all releases to find artists for the related artists dropdown
const artistsQuery = useGetReleasesQuery();

// Artist data structure
const artistData = ref<Partial<ReleaseItem>>({
  name: '',
  categoryId: 'music',
  thumbnailCID: '',
  contentCID: '', // Artists don't have real content
  metadata: {
    type: 'artist',
    bio: '',
    genre: '',
    formed: '',
    relatedArtists: []
  }
});

// Related artists selection
const selectedRelatedArtists = ref<any[]>([]);

// Available artists for the dropdown (excluding current artist)
const availableArtists = computed(() => {
  if (!artistsQuery.data.value) return [];

  return artistsQuery.data.value
    .filter((r: any) =>
      r.metadata?.type === 'artist' &&
      r.id !== props.initialData?.id
    )
    .map((r: any) => ({
      title: r.name,
      value: r.id
    }));
});

// Form validation
const rules = {
  required: (value: string) => !!value || 'Required field.'
};

const isValid = computed(() => {
  return !!artistData.value.name;
});

// Watch for changes in related artists selection
watch(selectedRelatedArtists, (newValue) => {
  if (artistData.value.metadata) {
    artistData.value.metadata.relatedArtists = newValue.map((item: any) =>
      typeof item === 'string' ? item : item.value
    );
  }
}, { deep: true });

// Initialize form with existing data
onMounted(() => {
  if (props.initialData) {
    artistData.value = {
      ...props.initialData,
      metadata: {
        type: 'artist',
        bio: props.initialData.metadata?.bio || '',
        genre: props.initialData.metadata?.genre || '',
        formed: props.initialData.metadata?.formed || '',
        relatedArtists: props.initialData.metadata?.relatedArtists || []
      }
    };

    // Initialize selected related artists
    if (props.initialData.metadata?.relatedArtists) {
      const relatedIds = props.initialData.metadata.relatedArtists;
      selectedRelatedArtists.value = relatedIds.map((id: string) => {
        const artist = artistsQuery.data.value?.find((r: any) => r.id === id);
        return artist ? { title: artist.name, value: id } : { title: id, value: id };
      });
    }
  }
});

// Submit handler
async function handleOnSubmit() {
  const { valid } = await formRef.value.validate();

  if (!valid) {
    emit('update:error', 'Please fill in all required fields');
    return;
  }

  try {
    const releaseData = {
      name: artistData.value.name!,
      categoryId: artistData.value.categoryId!,
      contentCID: artistData.value.contentCID || '',
      thumbnailCID: artistData.value.thumbnailCID,
      metadata: artistData.value.metadata
    };

    if (props.mode === 'edit' && props.initialData?.id) {
      await editMutation.mutateAsync({
        id: props.initialData.id,
        data: releaseData
      });
      emit('update:success', 'Artist updated successfully');
    } else {
      await addMutation.mutateAsync(releaseData);
      emit('update:success', 'Artist created successfully');
    }
  } catch (error) {
    console.error('Error saving artist:', error);
    emit('update:error', `Failed to ${props.mode === 'edit' ? 'update' : 'create'} artist`);
  }
}
</script>
