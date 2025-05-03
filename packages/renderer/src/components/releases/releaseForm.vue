<template>
  <v-form
    ref="formRef"
    :disabled="isLoading"
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
      :items="contentCategoriesItems"
      :rules="[rules.required]"
      label="Category"
    />
    <!-- Metadata is now cleared by the watch on releaseItem.category -->
    <!-- @update:model-value="() => releaseItem.metadata = {}" -->

    <!-- Conditional TV Show Fields -->
    <template v-if="isTvShowCategory">
      <v-select
        v-model="releaseItem.seriesId"
        :items="tvSeriesItems"
        label="TV Series"
        :rules="[rules.required]"
        required
      />
      <v-text-field
        v-model.number="releaseItem.seasonNumber"
        label="Season Number"
        type="number"
        :rules="[rules.required, rules.isNumber, rules.isPositive]"
        required
      />
      <v-text-field
        v-model.number="releaseItem.episodeNumber"
        label="Episode Number"
        type="number"
        :rules="[rules.required, rules.isNumber, rules.isPositive]"
        required
      />
    </template>
    <!-- End Conditional Fields -->

    <v-text-field
      v-model="releaseItem.author"
      label="Author"
      :rules="[rules.required]"
    />
    <v-text-field
      v-model="releaseItem.thumbnail"
      label="Thumbnail CID (Optional)"
      :rules="[rules.isValidCid]"
      clearable
    />
    <v-text-field
      v-model="releaseItem.cover"
      label="Cover Image CID (Optional)"
      :rules="[rules.isValidCid]"
      clearable
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
        v-if="selectedContentCategory"
        width="480px"
        max-height="620px"
        class="pa-8 ma-auto"
      >
        <p class="text-subtitle mb-6 text-center">
          Please fill out any extra information about the content that might be useful.
        </p>
        <div
          v-for="[categoryId, {type, description, options}] in Object.entries(selectedContentCategory)"
          :key="categoryId"
        >
          <v-select
            v-if="options"
            :items="options"
            :label="categoryId"
            :model-value="(releaseItem.metadata[categoryId] as string | null | undefined)"
            @update:model-value="(v) => {
              if (v) handleChangeMetadataField(categoryId, v)
            }"
          />
          <v-text-field
            v-else
            :label="categoryId"
            :model-value="releaseItem.metadata[categoryId]"
            :type="type"
            @update:model-value="(v) => handleChangeMetadataField(categoryId, v)"
          >
            <template #append-inner>
              <v-tooltip
                location="top"
                :text="description"
              >
                <template #activator="{props: tooltipProps}">
                  <v-icon
                    size="small"
                    v-bind="tooltipProps"
                    color="grey-lighten-1"
                    icon="mdi-help-circle-outline"
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
      :disabled="!readyToSave"
      :is-loading="isLoading"
    />
  </v-form>
</template>

<script setup lang="ts">
import { consts, type types as orbiterTypes } from '@riffcc/orbiter';
import { suivre as follow } from '@constl/vue';
import { cid } from 'is-ipfs';
import { computed, onMounted, ref, watch } from 'vue'; // Added watch
import { useOrbiter } from '/@/plugins/orbiter/utils';
import type { ReleaseItem, PartialReleaseItem } from '/@/@types/release';
import { useTvSeriesStore } from '/@/stores/tvSeries'; // Import TV Series store
import { storeToRefs } from 'pinia'; // Import storeToRefs

const props = defineProps<{
  initialData?: PartialReleaseItem;
  mode?: 'create' | 'edit';
}>();

const emit = defineEmits<{
  (e: 'submit', data: unknown): void;
  (e: 'update:success', message: string): void;
  (e: 'update:error', message: string): void;
}>();

const { orbiter } = useOrbiter();
const formRef = ref<any>(null); // Use any for VForm ref type
const openAdvanced = ref<boolean>(false);

// --- TV Series Store ---
const tvSeriesStore = useTvSeriesStore();
const { tvSeries } = storeToRefs(tvSeriesStore);
const tvSeriesItems = computed(() => tvSeries.value.map(s => ({ title: s.name, value: s.id })));
// --- End TV Series Store ---

const releaseItem = ref<PartialReleaseItem>({ // Use PartialReleaseItem for flexibility
  name: '',
  contentCID: '',
  category: '',
  author: '',
  metadata: {},
  seriesId: undefined, // Initialize TV Show fields
  seasonNumber: undefined,
  episodeNumber: undefined,
});

const rules = {
  required: (v: string | number) => v !== null && v !== undefined && v !== '' || 'Required field.', // Updated required rule
  isValidCid: (v: string) => !v || cid(v) || 'Please enter a valid CID.',
  isNumber: (v: number) => !v || !isNaN(Number(v)) || 'Must be a number.', // Added number rule
  isPositive: (v: number) => !v || Number(v) > 0 || 'Must be positive.', // Added positive rule
};
const isLoading = ref(false);

const contentCategories = follow(orbiter.listenForContentCategories.bind(orbiter));
const contentCategoriesItems = computed(() => (contentCategories.value ?? []).map(item => ({
  id: item.id,
  value: item.contentCategory.categoryId,
  title: item.contentCategory.displayName,
})));

const isTvShowCategory = computed(() => releaseItem.value.category === 'tvShow');

// Reset TV fields if category changes away from tvShow
watch(() => releaseItem.value.category, (newCategory) => {
  if (newCategory !== 'tvShow') {
    releaseItem.value.seriesId = undefined;
    releaseItem.value.seasonNumber = undefined;
    releaseItem.value.episodeNumber = undefined;
  }
  // Also clear metadata when category changes
  releaseItem.value.metadata = {};
});


const selectedContentCategory = computed(() => {
  let categoryMetadataData: orbiterTypes.ContentCategoryMetadataField | undefined = undefined;
  if (contentCategories.value) {
    const targetItem = contentCategories.value.find(item => item.contentCategory.categoryId === releaseItem.value.category);
    if (targetItem) {
      try { // Added try-catch for parsing
        categoryMetadataData = JSON.parse(targetItem.contentCategory.metadataSchema);
      } catch (e) {
        console.error("Failed to parse metadata schema:", e);
        categoryMetadataData = undefined; // Ensure it's undefined on error
      }
    }
  }
  return categoryMetadataData;
});

const handleChangeMetadataField = (categoryId: string, value: string | number | null | undefined) => { // Updated value type
  // Ensure metadata is an object
  if (typeof releaseItem.value.metadata !== 'object' || releaseItem.value.metadata === null) {
      releaseItem.value.metadata = {};
  }

  if (value === null || value === undefined || value === '') { // Handle clearing field
    // Create a new object excluding the categoryId
    const newMetadata = { ...releaseItem.value.metadata };
    delete newMetadata[categoryId];
    releaseItem.value.metadata = newMetadata;
  } else {
    releaseItem.value.metadata = {
      ...releaseItem.value.metadata,
      [categoryId]: value,
    };
  };
};

onMounted(() => {
  if (props.initialData) {
    releaseItem.value = {
      ...releaseItem.value, // Keep default undefined for TV fields initially
      ...props.initialData,
      metadata: props.initialData.metadata ? { ...props.initialData.metadata } : {},
    };
  }
});

const readyToSave = computed(() => {
  // Basic validation + form validity
  const basicFieldsValid = releaseItem.value.name &&
                           releaseItem.value.contentCID &&
                           releaseItem.value.category &&
                           releaseItem.value.author;

  // TV Show specific validation
  const tvShowFieldsValid = !isTvShowCategory.value || (
    releaseItem.value.seriesId &&
    releaseItem.value.seasonNumber !== undefined && releaseItem.value.seasonNumber > 0 &&
    releaseItem.value.episodeNumber !== undefined && releaseItem.value.episodeNumber > 0
  );

  // Check VForm validity state
  const formIsValid = formRef.value?.isValid; // Access validity state

  if (basicFieldsValid && tvShowFieldsValid && formIsValid) {
    return releaseItem.value;
  }
  return undefined;
});


const handleOnSubmit = async () => {
  // Trigger validation explicitly
  const { valid } = await formRef.value?.validate();
  if (!valid || !readyToSave.value) return; // Check computed property again after validation

  isLoading.value = true;
  try {
    const data = readyToSave.value; // Use the validated data

    // Base release data
    const release: Record<string, unknown> = { // Changed type to Record
      [consts.RELEASES_AUTHOR_COLUMN]: data.author,
      [consts.RELEASES_CATEGORY_COLUMN]: data.category,
      [consts.RELEASES_FILE_COLUMN]: data.contentCID,
      [consts.RELEASES_METADATA_COLUMN]: JSON.stringify(data.metadata || {}), // Ensure metadata is stringified
      [consts.RELEASES_NAME_COLUMN]: data.name,
      [consts.RELEASES_THUMBNAIL_COLUMN]: data.thumbnail || null, // Send null if empty
      [consts.RELEASES_COVER_COLUMN]: data.cover || null, // Send null if empty
    };

    // Add TV Show specific fields if applicable
    // Assuming Orbiter schema uses these column names
    if (isTvShowCategory.value) {
      release['seriesId'] = data.seriesId;
      release['seasonNumber'] = data.seasonNumber;
      release['episodeNumber'] = data.episodeNumber;
    }

    if (props.mode === 'edit' && props.initialData?.id) {
      await orbiter.editRelease({
        releaseId: props.initialData.id,
        release,
      });
      emit('update:success', 'Release (Episode) updated successfully!'); // Updated message
    } else {
      await orbiter.addRelease(release);
      emit('update:success', 'Release (Episode) added successfully!'); // Updated message
    }
    emit('submit', data); // Emit the local data structure
    clearForm();
  } catch (error) {
    console.error('Error saving release:', error);
    // More specific error message
    emit('update:error', `Error saving release: ${error.message || 'Please try again later.'}`);
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
  formRef.value?.resetValidation();
  formRef.value?.reset();
};
</script>
