<template>
  <v-container>
    <v-row justify="center">
      <v-col
        cols="12"
        md="6"
        lg="5"
      >
        <v-sheet
          class="px-6 py-4 mx-auto"
          max-width="448px"
        >
          <h6 class="text-h6 font-weight-bold mb-4">New Featured Release</h6>
          <v-form
            ref="formRef"
            validate-on="input lazy"
            @submit.prevent="handleOnSubmit"
          >
            <v-text-field
              v-model="newFeaturedRelease.releaseId"
              label="Release ID"
              validate-on="input"
              :rules="rules"
            ></v-text-field>
            <v-text-field
              v-model="newFeaturedRelease.startAt"
              type="datetime-local"
              :min="minDate"
              :max="maxDate"
              :rules="rules"
              label="Start at"
            ></v-text-field>
            <v-text-field
              v-model="newFeaturedRelease.endAt"
              :disabled="!newFeaturedRelease.startAt"
              type="datetime-local"
              :min="minEndDate"
              :max="maxDate"
              :rules="endAtRules"
              label="End at"
            ></v-text-field>
            <v-btn
              color="primary"
              type="submit"
              text="Create"
              :loading="isLoading"
              :disabled="isLoading || !readyToSave"
              block
            >
            </v-btn>
          </v-form>
        </v-sheet>
      </v-col>
      <v-col
        cols="12"
        md="6"
        lg="5"
      >
        <v-sheet
          class="px-6 py-4 mx-auto h-100"
          max-width="448px"
          min-height="256px"
        >
          <h6 class="text-h6 font-weight-bold mb-4">Featured Releases</h6>
          <v-list v-if="featuredReleases.length > 0">
            <v-list-item
              v-for="featuredRelease, i in featuredReleases"
              :key="i"
              class="px-0"
              :title="featuredRelease.id"
            >
              <template #title="{title}">
                <p class="text-subtitle-2 mx-2">{{ title }}</p>
              </template>
              <template #prepend>
                <v-chip
                  v-if="filterActivedFeature(featuredRelease)"
                  color="green"
                  size="small"
                >
                  Active
                </v-chip>
                <v-chip
                  v-else
                  color="red"
                  size="small"
                >
                  Ended
                </v-chip>
              </template>
              <template #append>
                <v-btn
                  icon="mdi-check"
                  size="small"
                  density="comfortable"
                  :disabled="!filterActivedFeature(featuredRelease)"
                  :color="filterActivedFeature(featuredRelease) ? 'blue' : 'default'"
                  @click="confirmEndFeaturedReleaseDialog = true"
                >
                </v-btn>
              </template>
              <confirmation-dialog
                message="Are you sure you want to end this featured?"
                :dialog-open="confirmEndFeaturedReleaseDialog"
                @close="confirmEndFeaturedReleaseDialog = false"
                @confirm="() => confirmEndFeaturedRelease(featuredRelease.id)"
              ></confirmation-dialog>
            </v-list-item>
          </v-list>
          <div
            v-else
            class="d-flex h-75"
          >
            <span class="ma-auto text-body-2 text-medium-emphasis">No Featured Releases found.</span>
          </div>
        </v-sheet>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import {computed, onMounted, ref, watch, type Ref} from 'vue';
import { suivre as follow } from '@constl/vue';
import {useStaticReleases} from '/@/composables/staticReleases';
import { useStaticStatus } from '/@/composables/staticStatus';
import { useOrbiter } from '/@/plugins/orbiter/utils';
import type { FeaturedReleaseItem } from '/@/@types/release';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import { filterActivedFeature } from '/@/utils';
type FeaturedReleaseData = {
  releaseId: string | null;
  startAt: string | null;
  endAt: string | null;
}

const { staticStatus } = useStaticStatus();
const { staticReleases, staticFeaturedReleases } = useStaticReleases();
const { orbiter } = useOrbiter();
const orbiterFeaturedReleases = follow(orbiter.listenForSiteFeaturedReleases.bind(orbiter));

const featuredReleases = computed<FeaturedReleaseItem[]>(() => {
  if (staticStatus.value === 'static') return staticFeaturedReleases.value;
  else {
    return (orbiterFeaturedReleases.value || []).map((fr): FeaturedReleaseItem => {
      return {
        id: fr.id,
        releaseId: fr.featured.releaseId,
        startTime: fr.featured.startTime,
        endTime: fr.featured.endTime,
      };
    });
  }
});

const newFeaturedRelease: Ref<FeaturedReleaseData> = ref({
  releaseId: null,
  startAt: null,
  endAt: null,
});

const formRef = ref();
const isLoading = ref(false);

const rules = [
  (value: string) => Boolean(value) || 'Required field.',
];

const endAtRules = [
  (value: string | null) => !newFeaturedRelease.value.startAt || Boolean(value) || 'End date is required if start date is set.',
  (value: string | null) => {
      if (!value || !newFeaturedRelease.value.startAt) return true;
      return new Date(value) > new Date(newFeaturedRelease.value.startAt) || 'End date must be after start date.';
  },
];

const minDate = ref<string | null>(null);
const maxDate = ref<string | null>(null);
const minEndDate = ref<string | null>(null);

const confirmEndFeaturedReleaseDialog = ref(false);

onMounted(() => {
  const now = new Date();
  const max = new Date(now);
  max.setFullYear(now.getFullYear() + 1);
  minDate.value = now.toISOString().substring(0, 16);
  maxDate.value = max.toISOString().substring(0, 16);
});

watch(() => newFeaturedRelease.value.startAt, newStartAt => {
  minEndDate.value = null;
  if (newStartAt) {
    const newDate = new Date(newStartAt);
    newDate.setDate(newDate.getDate());
    newDate.setMinutes(newDate.getMinutes() + 10);
    minEndDate.value = newDate.toISOString().substring(0, 16);

    if (newFeaturedRelease.value.endAt && new Date(newFeaturedRelease.value.endAt) < newDate) {
      newFeaturedRelease.value.endAt = null;
    }
  } else {
    newFeaturedRelease.value.endAt = null;
  }
});


const resetForm = () => {
  newFeaturedRelease.value = {
    releaseId: null,
    startAt: null,
    endAt: null,
  };
  formRef.value?.resetValidation();
  formRef.value?.reset();
};

const readyToSave = computed(() => {
  if (
    newFeaturedRelease.value.releaseId &&
    newFeaturedRelease.value.startAt &&
    newFeaturedRelease.value.endAt &&
    formRef.value?.isValid
  ) {
    const startTime = new Date(newFeaturedRelease.value.startAt).toISOString();
    const endTime = new Date(newFeaturedRelease.value.endAt).toISOString();

    return {
      releaseId: newFeaturedRelease.value.releaseId,
      startTime,
      endTime,
    };
  }
  return undefined;
});

const handleOnSubmit = async () => {
  if (!readyToSave.value) {
    return;
  }

  isLoading.value = true;
  console.log('Creating new featured release with data:', readyToSave.value);

  try {
    if (staticStatus.value === 'static') {
      await new Promise(resolve => setTimeout(resolve, 500)); // Simulate network delay
      const targetRelease = staticReleases.value.find(r => r.id === readyToSave.value?.releaseId);
      if (targetRelease && targetRelease.id && readyToSave.value) {
        staticFeaturedReleases.value.push({
          id: `featured-${Date.now()}-${Math.random().toString(16).substring(2, 8)}`,
          releaseId: targetRelease.id,
          startTime: readyToSave.value.startTime,
          endTime: readyToSave.value.endTime,
        });
        console.log('Featured release added successfully to static list.');
        resetForm(); // Reset form on success
      } else {
        console.error('Target release not found.');
      }
    } else {
      await orbiter.featureRelease({
        cid: readyToSave.value.releaseId,
        startTime: readyToSave.value.startTime,
        endTime: readyToSave.value.endTime,
      });
    }
  } catch (error) {
    console.error('Error creating featured release:', error);
  } finally {
    isLoading.value = false;
  }
};


const confirmEndFeaturedRelease = async (id: string) => {
  confirmEndFeaturedReleaseDialog.value = false;

  if (staticStatus.value === 'static') {
    const index = staticFeaturedReleases.value.findIndex(fr => fr.id === id);
    if (index !== -1) {
        staticFeaturedReleases.value[index] = {
            ...staticFeaturedReleases.value[index],
            endTime: new Date().toISOString(),
        };
        console.log(`Static featured release ${id} ended.`);
    } else {
        console.warn(`Static featured release ${id} not found to end.`);
    }
  } else {
    try {
      await orbiter.editFeaturedRelease({
        elementId: id,
        featuredRelease: {
          endTime: new Date().toISOString(),
        },
      });
      console.log(`Featured release ${id} ended successfully.`);
    } catch (error) {
      console.error(`Error on ending featured release ${id}:`, error);
    }
  }
};

</script>
