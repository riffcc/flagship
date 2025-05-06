<template>
  <v-form
    ref="formRef"
    :disabled="isLoading"
    validate-on="input lazy"
    class="d-flex flex-column ga-2"
    @submit.prevent="handleOnSubmit"
  >
    <v-text-field
      v-model="contentCategory.contentCategory.categoryId"
      label="Category ID"
      :rules="[rules.required]"
    />
    <v-text-field
      v-model="contentCategory.contentCategory.displayName"
      label="Display Name"
      :rules="[rules.required]"
    />
    <v-switch
      v-model="contentCategory.contentCategory.featured"
      :color="contentCategory.contentCategory.featured ? 'primary' : 'default'"
      label="Featured"
    ></v-switch>
    <v-list-item
      title="Metadata Schema"
      class="pa-0"
    >
      <template #append>
        <v-dialog
          v-model="createMetadataFieldDialog"
        >
          <template #activator="{props: createMetadataFieldDialogActivatorProps}">
            <v-btn
              icon="mdi-plus-circle"
              variant="text"
              density="comfortable"
              size="small"
              v-bind="createMetadataFieldDialogActivatorProps"
            ></v-btn>
          </template>
          <v-sheet
            width="480px"
            max-height="620px"
            class="pa-8 ma-auto"
          >
            <metadata-field-form
              @submit="handleSubmitMetadataField"
            />
            <v-btn
              color="blue-darken-1 float-right"
              variant="text"
              @click="createMetadataFieldDialog = false"
            >
              Cancel
            </v-btn>
          </v-sheet>
        </v-dialog>
      </template>
    </v-list-item>
    <v-divider></v-divider>
    <v-list
      v-if="Object.entries(contentCategory.contentCategory.metadataSchema).length > 0"
      max-height="10rem"
    >
      <v-list-item
        v-for="[fieldKey] in Object.entries(contentCategory.contentCategory.metadataSchema)"
        :key="fieldKey"
        :title="fieldKey"
      >
        <template #append>
          <v-btn
            icon="mdi-pencil"
            variant="text"
            density="comfortable"
            size="x-small"
            @click="() => editMetadataField(fieldKey)"
          ></v-btn>
          <v-btn
            icon="mdi-delete"
            variant="text"
            density="comfortable"
            size="x-small"
            @click="() => deleteMetadataField(fieldKey)"
          ></v-btn>
        </template>
      </v-list-item>
    </v-list>
    <v-sheet
      v-else
      class="d-flex justify-center"
      height="4rem"
    >
      <p class="text-subtitle-2 text-medium-emphasis my-auto">No metadata fields.</p>
    </v-sheet>
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
  <v-dialog
    v-model="editMetadataFieldDialog"
    max-width="500px"
  >
    <v-card class="py-3">
      <v-card-title>
        <span class="text-h6 ml-2">Edit Metadata Field</span>
      </v-card-title>

      <v-card-text>
        <metadata-field-form
          :initial-data="editedMetadataField"
          mode="edit"
          @submit="handleSubmitMetadataField"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          color="blue-darken-1"
          variant="text"
          @click="editMetadataFieldDialog = false"
        >
          Cancel
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import type { types as orbiterTypes } from '@riffcc/orbiter';
import {consts } from '@riffcc/orbiter';
import {computed, onMounted, ref} from 'vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import MetadataFieldForm from '/@/components/releases/metadataFieldForm.vue';

const props = defineProps<{
  initialData?: orbiterTypes.ContentCategoryWithId<orbiterTypes.ContentCategoryMetadataField>;
  mode?: 'create' | 'edit';
}>();

const emit = defineEmits<{
  (e: 'submit', data: unknown): void;
  (e: 'update:success', message: string): void;
  (e: 'update:error', message: string): void;
}>();

const {orbiter} = useOrbiter();
const formRef = ref();

const contentCategory = ref<orbiterTypes.ContentCategoryWithId<orbiterTypes.ContentCategoryMetadataField>>({
  id: '',
  contentCategory: {
    categoryId: '',
    displayName: '',
    metadataSchema: {},
  },
});

const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
};
const isLoading = ref(false);
const createMetadataFieldDialog = ref(false);
const editMetadataFieldDialog = ref(false);

const editedMetadataField = ref<orbiterTypes.ContentCategoryMetadataField[string] & { fieldKey: string; }>({
  fieldKey: '',
  description: '',
  type: 'string',
  options: [],
});

function editMetadataField(fieldKey: string) {
  const targetField = Object.entries(contentCategory.value.contentCategory.metadataSchema).find(
    ([k]) => k === fieldKey,
  );
  if(targetField) {
    const [targetFieldKey, targetFieldValue] = targetField;
    editedMetadataField.value = {
      fieldKey: targetFieldKey,
      description: targetFieldValue.description,
      type: targetFieldValue.type,
      options: targetFieldValue.options,
    };
    editMetadataFieldDialog.value = true;
  }
}

function handleSubmitMetadataField(data: orbiterTypes.ContentCategoryMetadataField[string] & { fieldKey: string; }) {
  const metadataField: orbiterTypes.ContentCategoryMetadataField[string] = {
    description: data.description,
    type: data.type,
    options: data.options && data.options.length > 0 ? data.options : undefined,
  };
  contentCategory.value.contentCategory.metadataSchema[data.fieldKey] = metadataField;
  if (createMetadataFieldDialog.value) createMetadataFieldDialog.value = false;
  if (editMetadataFieldDialog.value) editMetadataFieldDialog.value = false;
};

function deleteMetadataField(fieldKey: string) {
  contentCategory.value.contentCategory.metadataSchema = Object.fromEntries(
    Object.entries(contentCategory.value.contentCategory.metadataSchema).filter((field) => field[0] !== fieldKey),
  );
};

onMounted(() => {
  if(props.initialData) {
    contentCategory.value = {
      ...contentCategory.value,
      id: props.initialData.id,
      contentCategory: {
        ...props.initialData.contentCategory,
        metadataSchema: {
          ...props.initialData.contentCategory.metadataSchema,
        },
      },
    };
  }
});

const readyToSave = computed(() => {
  if (
    contentCategory.value.contentCategory.categoryId &&
    contentCategory.value.contentCategory.displayName &&
    contentCategory.value.contentCategory.metadataSchema &&
    formRef.value.isValid
  ) {
    return contentCategory.value;
  }
  return undefined;
});

const handleOnSubmit = async () => {
  if (!readyToSave.value) return;
  isLoading.value = true;
  try {
    const data = readyToSave.value;
    const category = {
      [consts.CONTENT_CATEGORIES_CATEGORY_ID]: data.contentCategory.categoryId,
      [consts.CONTENT_CATEGORIES_DISPLAY_NAME]: data.contentCategory.displayName,
      [consts.CONTENT_CATEGORIES_FEATURED]: data.contentCategory.featured,
      [consts.CONTENT_CATEGORIES_METADATA_SCHEMA]: JSON.stringify(data.contentCategory.metadataSchema),
    };
    if (props.mode === 'edit' && props.initialData?.id) {
      await orbiter.editCategory({
        elementId: props.initialData.id,
        category,
      });
    } else {
      await orbiter.addCategory(category);
    }
    emit('submit', data);
    emit('update:success', 'Category saved successfully!');
    clearForm();
  } catch (error) {
    console.error('Error saving category:', error);
    emit('update:error', 'Error saving category. Please try again later.');
  } finally {
    isLoading.value = false;
  }
};

const clearForm = () => {
  contentCategory.value = {
    id: '',
    contentCategory: {
      categoryId: '',
      displayName: '',
      metadataSchema: {},
    },
  };
};
</script>
