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
          v-for="[categoryId, {type, description, options}] in Object.entries(selectedContentCategory)"
          :key="categoryId"
        >
          <v-select
            v-if="options"
            :items="options"
            :label="categoryId"
            :model-value="String((releaseItem.metadata && releaseItem.metadata[categoryId]) || '')"
            @update:model-value="(v) => handleChangeMetadataField(categoryId, v)"
          />
          <v-text-field
            v-else
            :label="categoryId"
            :model-value="String((releaseItem.metadata && releaseItem.metadata[categoryId]) || '')"
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
import type { ContentCategoryMetadata, ReleaseData, AnyObject } from '@riffcc/lens-sdk';
import { useAddReleaseMutation, useEditReleaseMutation, useContentCategoriesQuery } from '/@/plugins/lensService/hooks';

const props = defineProps<{
  initialData?: ReleaseItem<AnyObject>;
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

const releaseItem = ref<ReleaseItem<AnyObject>>({
  id: '',
  name: '',
  contentCID: '',
  categoryId: '',
  metadata: {},
});

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
  let categoryMetadataData: ContentCategoryMetadata | undefined = undefined;
  if (contentCategories.value) {
    const targetItem = contentCategories.value.find(item => item.id === releaseItem.value.categoryId);
    if (targetItem) {
      categoryMetadataData = targetItem.metadataSchema;
    }
  }
  return categoryMetadataData;
});

const handleChangeMetadataField = (categoryId: string, value: string | null) => {
  if (!releaseItem.value.metadata) {
    releaseItem.value.metadata = {};
  }
  releaseItem.value.metadata = {
    ...releaseItem.value.metadata,
    [categoryId]: value || '',
  };
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
  const releaseDataPayload: ReleaseData<AnyObject> = {
    name: data.name,
    categoryId: data.categoryId,
    contentCID: data.contentCID,
    thumbnailCID: data.thumbnailCID,
    metadata: data.metadata,
  };

  if (props.mode === 'edit' && data.id) {
    editReleaseMutation.mutate({ ...releaseDataPayload, id: data.id });
  } else {
    addReleaseMutation.mutate(releaseDataPayload);
  }
};

const clearForm = () => {
  releaseItem.value = {
    id: '',
    name: '',
    contentCID: '',
    categoryId: '',
    metadata: {},
  };
  formRef.value?.resetValidation();
  formRef.value?.reset();
};
</script>
