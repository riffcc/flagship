<template>
  <v-form
    ref="formRef"
    validate-on="input lazy"
    class="d-flex flex-column ga-2"
    @submit.prevent="handleOnSubmit"
  >
    <v-text-field
      v-model="metadataField.fieldKey"
      label="Field Key"
      :rules="[rules.required]"
    />
    <v-text-field
      v-model="metadataField.description"
      label="Description"
      :rules="[rules.required]"
    />
    <v-select
      v-model="metadataField.type"
      :items="['string', 'number', 'array']"
      label="Type"
    />
    <v-text-field
      v-model="metadataFieldOption"
      label="Options (Optional)"
    >
      <template #append-inner>
        <v-btn
          icon="mdi-plus-circle"
          variant="text"
          density="comfortable"
          size="small"
          @click="handleAddOption"
        ></v-btn>
      </template>
      <template #details>
        <div
          v-if="metadataField.options && metadataField.options.length > 0"
          class="my-2"
        >
          <v-chip
            v-for="option, i in metadataField.options"
            :key="`${option}-${i}`"
            density="compact"
            class="mx-1 my-1"
          >
            {{ option }}
            <template #append>
              <v-btn
                icon="mdi-close"
                variant="text"
                density="compact"
                size="x-small"
                class="ml-1"
                @click="() => handleDeleteOption(option)"
              ></v-btn>
            </template>
          </v-chip>
        </div>
      </template>
    </v-text-field>
    <v-btn
      rounded="0"
      color="primary"
      type="submit"
      block
      text="Save"
      :disabled="!readyToSave"
    />
  </v-form>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref, type Ref } from 'vue';
import type { types as orbiterTypes } from '/@/plugins/peerbit/orbiter-types';

const props = defineProps<{
  initialData?: Partial<orbiterTypes.ContentCategoryMetadataField[string] & { fieldKey: string; }>;
  mode?: 'create' | 'edit';
}>();

const emit = defineEmits<{
  (e: 'submit', data: orbiterTypes.ContentCategoryMetadataField[string] & { fieldKey: string; }): void;
}>();

const formRef = ref();

const metadataField = ref<orbiterTypes.ContentCategoryMetadataField[string] & { fieldKey: string; }>({
  fieldKey: '',
  description: '',
  type: 'string',
});

const metadataFieldOption = ref('');
const rules = {
  required: (v: string) => Boolean(v) || 'Required field.',
};

onMounted(() => {
  if(props.initialData) {
    metadataField.value = {
      ...metadataField.value,
      ...props.initialData,
    };
  }
});

const readyToSave = computed(() => {
  if (
    metadataField.value.fieldKey &&
    metadataField.value.description &&
    metadataField.value.type &&
    formRef.value.isValid
  ) {
    return metadataField.value;
  }
  return undefined;
});

function handleAddOption() {
  if (metadataFieldOption.value !== '') {
    if (!metadataField.value.options) {
    metadataField.value.options = [];
    }
    if (!metadataField.value.options.includes(metadataFieldOption.value)) {
      metadataField.value.options.push(metadataFieldOption.value);
    }
    metadataFieldOption.value = '';
  }
}

function handleDeleteOption(option: string){
  if (metadataField.value.options) {
    metadataField.value.options = metadataField.value.options.filter(o => o !== option);
  }
}

function handleOnSubmit() {
  if (!readyToSave.value) return;
  const data = readyToSave.value;
  emit('submit', data);
  clearForm();
};


function clearForm() {
  metadataField.value.fieldKey = '';
  metadataField.value.description = '';
  metadataField.value.type = 'string';
  metadataField.value.options = undefined;
  metadataFieldOption.value = '';
};
</script>
