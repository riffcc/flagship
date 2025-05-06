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
            @submit.prevent="handleOnSubmit"
          >
            <v-text-field
              v-model="newFeaturedRelease.releaseId"
              label="Release ID"
              validate-on="input"
              :rules="rules"
            ></v-text-field>
            <v-text-field
              v-model="newFeaturedRelease.startTime"
              type="datetime-local"
              :min="minDate"
              :max="maxDate"
              :rules="rules"
              label="Start at"
            ></v-text-field>
            <v-text-field
              v-model="newFeaturedRelease.endTime"
              :disabled="!newFeaturedRelease.startTime"
              type="datetime-local"
              :min="minEndDate"
              :max="maxDate"
              :rules="endAtRules"
              label="End at"
            ></v-text-field>
            <v-switch
              v-model="newFeaturedRelease.promoted"
              :color="newFeaturedRelease.promoted ? 'primary' : 'default'"
              label="Promoted"
            ></v-switch>
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
          <v-list v-if="unfilteredFeaturedReleases.length > 0">
            <v-list-item
              v-for="featuredRelease in unfilteredFeaturedReleases"
              :key="featuredRelease.id"
              class="px-0"
              :title="featuredRelease.releaseId"
            >
              <template #title="{title}">
                <p class="text-subtitle-2 text-center">{{ title }}</p>
              </template>
              <template #prepend>
                <v-sheet
                  width="80"
                  class="d-flex justify-center"
                >
                  <template v-if="filterActivedFeatured(featuredRelease)">
                    <v-chip
                      v-if="filterPromotedFeatured(featuredRelease)"
                      color="yellow"
                      size="small"
                      class="w-100 d-flex justify-center"
                    >
                      Promoted
                    </v-chip>
                    <v-chip
                      v-else
                      color="green"
                      size="small"
                      class="w-100 d-flex justify-center"
                    >
                      Active
                    </v-chip>
                  </template>
                  <v-chip
                    v-else
                    color="red"
                    size="small"
                    class="w-100 d-flex justify-center"
                  >
                    Ended
                  </v-chip>
                </v-sheet>
              </template>
              <template #append>
                <v-btn
                  icon="mdi-check"
                  size="small"
                  density="compact"
                  :disabled="!filterActivedFeatured(featuredRelease)"
                  :color="filterActivedFeatured(featuredRelease) ? 'blue' : 'default'"
                  @click="featuredItemIdToEnd = featuredRelease.id"
                >
                </v-btn>
              </template>
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
    <confirmation-dialog
      message="Are you sure you want to end this featured?"
      :dialog-open="Boolean(featuredItemIdToEnd)"
      :on-close="() => featuredItemIdToEnd = null"
      :on-confirm="confirmEndFeaturedRelease"
    ></confirmation-dialog>
  </v-container>
</template>

<script setup lang="ts">
import {computed, onMounted, ref, watch, type Ref} from 'vue';
import { useStaticStatus } from '/@/composables/staticStatus';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import { filterActivedFeatured, filterPromotedFeatured, useReleasesStore, type PartialFeaturedReleaseItem } from '/@/stores/releases';
import { storeToRefs } from 'pinia';
import { useOrbiter } from '/@/plugins/orbiter/utils';
import { useStaticReleases } from '/@/composables/staticReleases';

const { orbiter } = useOrbiter();
const { staticStatus } = useStaticStatus();
const {staticFeaturedReleases} = useStaticReleases();
const releasesStore = useReleasesStore();
const {releases, unfilteredFeaturedReleases} = storeToRefs(releasesStore);

const props = defineProps<{
  initialFeatureData: PartialFeaturedReleaseItem | null;
}>();

const emit = defineEmits<{
  'initial-data-consumed': []
}>();

const newFeaturedRelease: Ref<PartialFeaturedReleaseItem> = ref({
  releaseId: undefined,
  startAt: undefined,
  endAt: undefined,
  promoted: undefined,
});

const formRef = ref();
const isLoading = ref(false);
const featuredItemIdToEnd = ref<string | null>(null);

const rules = [
  (value: string) => Boolean(value) || 'Required field.',
];

const endAtRules = [
  (value: string | null) => !newFeaturedRelease.value.startTime || Boolean(value) || 'End date is required if start date is set.',
  (value: string | null) => {
      if (!value || !newFeaturedRelease.value.startTime) return true;
      return new Date(value) > new Date(newFeaturedRelease.value.startTime) || 'End date must be after start date.';
  },
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

watch(() => props.initialFeatureData, (newData) => {
  if (newData) {
    newFeaturedRelease.value = {
      ...newData,
    };

    emit('initial-data-consumed');
  }
}, { immediate: true });


watch(() => newFeaturedRelease.value.startTime, newStartAt => {
  minEndDate.value = null;
  if (newStartAt) {
    const newDate = new Date(newStartAt);
    newDate.setDate(newDate.getDate());
    newDate.setMinutes(newDate.getMinutes() + 10);
    minEndDate.value = newDate.toISOString().substring(0, 16);

    if (newFeaturedRelease.value.endTime && new Date(newFeaturedRelease.value.endTime) < newDate) {
      newFeaturedRelease.value.endTime = undefined;
    }
  } else {
    newFeaturedRelease.value.endTime = undefined;
  }
});


const resetForm = () => {
  newFeaturedRelease.value = {
    releaseId: undefined,
    startTime: undefined,
    endTime: undefined,
    promoted: undefined,
  };
  formRef.value?.resetValidation();
  formRef.value?.reset();
};

const readyToSave = computed(() => {
  if (
    newFeaturedRelease.value.releaseId &&
    newFeaturedRelease.value.startTime &&
    newFeaturedRelease.value.endTime &&
    newFeaturedRelease.value.promoted !== undefined &&
    formRef.value?.isValid
  ) {
    const startTime = new Date(newFeaturedRelease.value.startTime).toISOString();
    const endTime = new Date(newFeaturedRelease.value.endTime).toISOString();

    return {
      releaseId: newFeaturedRelease.value.releaseId,
      startTime,
      endTime,
      promoted: newFeaturedRelease.value.promoted,
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
      const targetRelease = releases.value.find(r => r.id === readyToSave.value?.releaseId);
      if (targetRelease && targetRelease.id && readyToSave.value) {
        staticFeaturedReleases.value.push({
          id: `featured-${Date.now()}-${Math.random().toString(16).substring(2, 8)}`,
          releaseId: targetRelease.id,
          startTime: readyToSave.value.startTime,
          endTime: readyToSave.value.endTime,
          promoted: true,
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
        promoted: readyToSave.value.promoted,
      });
    }
  } catch (error) {
    console.error('Error creating featured release:', error);
  } finally {
    isLoading.value = false;
  }
};

const confirmEndFeaturedRelease = async () => {
  if (!featuredItemIdToEnd.value) return;
  const endTime = (new Date()).toISOString();
  if (staticStatus.value === 'static') {
    const index = staticFeaturedReleases.value.findIndex(fr => fr.id === featuredItemIdToEnd.value);
    if (index !== -1) {
        staticFeaturedReleases.value[index] = {
            ...staticFeaturedReleases.value[index],
            endTime,
        };
        console.log(`Static featured release ${id} ended.`);
    } else {
        console.warn(`Static featured release ${id} not found to end.`);
    }
  } else {
    try {
      await orbiter.editFeaturedRelease({
        elementId: featuredItemIdToEnd.value,
        featuredRelease: {
          endTime,
          promoted: false,
        },
      });
      console.log(`Featured release ${id} ended successfully.`);
    } catch (error) {
      console.error(`Error on ending featured release ${id}:`, error);
    }
  }
  featuredItemIdToEnd.value = null;
};

</script>
