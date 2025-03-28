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
import {consts, type types as orbiterTypes} from '@riffcc/orbiter';
import { suivre as follow } from '@constl/vue';
import {cid} from 'is-ipfs';
import {computed, onMounted, ref} from 'vue';
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

const contentCategories = follow(orbiter.listenForContentCategories.bind(orbiter));
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
      categoryMetadataData = JSON.parse(targetItem.contentCategory.metadataSchema);
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
</script>
