<template>
  <v-container>
    <v-sheet
      class="px-6 py-4 mx-auto"
      max-width="448px"
    >
      <h6 class="text-h6 font-weight-bold mb-4">New Featured Release</h6>
      <v-form @submit="handleOnSubmit">
        <v-text-field
          v-model="releaseId"
          label="Release ID"
          validate-on="input"
          :rules="rules"
        ></v-text-field>
        <v-text-field
          v-model="startAt"
          type="datetime-local"
          :min="minDate"
          :max="maxDate"
          label="Start at"
        ></v-text-field>
        <v-text-field
          v-model="endAt"
          :disabled="!startAt"
          type="datetime-local"
          :min="minEndDate"
          :max="maxDate"
          label="End at"
        ></v-text-field>
        <v-btn
          color="primary"
          type="submit"
          text="Create"
          block
        >
        </v-btn>
      </v-form>
    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import {onMounted, ref, watch} from 'vue';
import {useStaticReleases} from '/@/composables/staticReleases';
import type { SubmitEventPromise } from 'vuetify';

const { staticReleases, staticFeaturedReleases } = useStaticReleases();
const releaseId = ref<string | null>(null);
const startAt = ref<string | null>(null);
const endAt = ref<string | null>(null);

const rules = [
  (value: string) => Boolean(value) || 'Required field.',
  (value: string) => staticReleases.value.map(r => r.id).includes(value) || 'Release dont exists. Please input an valid id.',
];

const minDate = ref<string | null>(null);
const maxDate = ref<string | null>(null);
const minEndDate = ref<string | null>(null);

onMounted(() => {
  const now = new Date();
  const max = new Date(now);
  max.setFullYear(now.getFullYear() + 1);
  minDate.value = now.toISOString().substring(0, 16);
  maxDate.value = max.toISOString().substring(0, 16);
});

watch(startAt, newStartAt => {
  if (newStartAt) {
    const newDate = new Date(newStartAt);
    newDate.setDate(newDate.getDate() + 1);
    minEndDate.value = newDate.toISOString().substring(0, 16);
  }

});

const resetForm = () => {
  releaseId.value = null;
  startAt.value = null;
  endAt.value = null;
};

const handleOnSubmit = (e: SubmitEventPromise) => {
  e.preventDefault();
  e.then(result => {
    if (result.valid) {
      const targetRelease = staticReleases.value.find(r => r.id === releaseId.value);
      if (targetRelease && targetRelease.id) {
        staticFeaturedReleases.value.push({
          id: (staticFeaturedReleases.value.length + 1).toString(),
          releaseId: targetRelease.id,
          startTime: startAt.value as string,
          endTime: endAt.value as string,
        });
      }
      resetForm();
    }
  });

};
</script>
