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
      @update:model-value="() => releaseItem.metadata = {}"
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
import {consts, type types as orbiterTypes} from '/@/plugins/peerbit/orbiter-types';
import {cid} from 'is-ipfs';
import {computed, onMounted, ref} from 'vue';
import {useOrbiter} from '/@/plugins/peerbit/utils';
import type { ReleaseItem, PartialReleaseItem } from '/@/stores/releases';
import { useContentCategoriesStore } from '/@/stores/contentCategories';
import { storeToRefs } from 'pinia';

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
const contentCategoriesStore = useContentCategoriesStore();
const { contentCategories } = storeToRefs(contentCategoriesStore);

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

const contentCategoriesItems = computed(() => (contentCategories.value ?? []).map(item => ({
  id: item.id,
  value: item.contentCategory.categoryId,
  title: item.contentCategory.displayName,
})));

const selectedContentCategory = computed(() => {
  let categoryMetadataData: orbiterTypes.ContentCategoryMetadataField | undefined = undefined;
  if (contentCategories.value) {
    const targetItem = contentCategories.value.find(item => item.contentCategory.categoryId === releaseItem.value.category);
    if (targetItem) {
      categoryMetadataData = targetItem.contentCategory.metadataSchema;
    }
  }
  return categoryMetadataData;
});

const handleChangeMetadataField = (categoryId: string, value: string) => {
  releaseItem.value.metadata = {
    ...releaseItem.value.metadata,
    [categoryId]: value,
  };
};

onMounted(() => {
  if(props.initialData) {
    releaseItem.value = {
      ...releaseItem.value,
      ...props.initialData,
      metadata: props.initialData.metadata ? { ...props.initialData.metadata } : {},
    };
  }
});

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
    const data = readyToSave.value; // Single declaration of data

    // Prepare data for Peerbit (common for add and update)
    // The 'id' field from 'data' (which comes from releaseItem.value) will be used by updateRelease.
    // releaseItem.value.id is populated from props.initialData.id on mount for edit mode.
    const peerbitReleaseData = {
      id: data.id, // Pass existing ID for updates; will be undefined for new releases
      name: data.name,
      file: data.contentCID, // Map contentCID to 'file' as expected by peerbitNode.ts ReleaseType
      author: data.author,
      category: data.category,
      thumbnail: data.thumbnail,
      cover: data.cover,
      metadata: data.metadata, // Send metadata as an object; peerbitNode.ts handles stringification
    };

    if (props.mode === 'edit' && data.id) {
      // Update existing release using Peerbit
      const response = await (window as any).peerbitAPI.updateRelease(data.id, peerbitReleaseData);
      if (response.success) {
        emit('submit', data); // data here includes the id
        emit('update:success', response.message || 'Release updated in Peerbit successfully!');
        // clearForm(); // Decide on UX: clear form or navigate away after edit
      } else {
        console.error('Error updating release in Peerbit:', response.error);
        emit('update:error', response.error || 'Error updating release in Peerbit. Please try again later.');
      }
    } else {
      // Create new release using Peerbit
      // peerbitReleaseData.id will be undefined here, backend generates it.
      const response = await (window as any).peerbitAPI.addRelease(peerbitReleaseData);
      if (response.success) {
        // Update the local data with the ID returned from the backend
        const submittedData = { ...data, id: response.id };
        emit('submit', submittedData);
        emit('update:success', response.message || 'Release saved to Peerbit successfully!');
        clearForm();
      } else {
        console.error('Error saving release to Peerbit:', response.error);
        emit('update:error', response.error || 'Error saving release to Peerbit. Please try again later.');
      }
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error('Error in submission process:', errorMessage);
    emit('update:error', `Submission error: ${errorMessage}`);
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
