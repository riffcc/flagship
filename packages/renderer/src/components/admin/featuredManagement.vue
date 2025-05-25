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
          <v-list v-if="(featuredReleases?.length ?? 0) > 0">
            <v-list-item
              v-for="featuredRelease in featuredReleases"
              :key="featuredRelease.id"
              class="px-0"
              :title="featuredRelease.releaseId"
            >
              <template #title="{ title }">
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
                  icon="$check"
                  size="small"
                  density="compact"
                  :disabled="!filterActivedFeatured(featuredRelease)"
                  :color="filterActivedFeatured(featuredRelease) ? 'blue' : 'default'"
                  @click="featuredItemIdToEnd = featuredRelease"
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
  <v-snackbar
    v-model="showSnackbar"
    :color="snackbarMessage?.type ?? 'default'"
  >
    {{ snackbarMessage?.text }}
    <template #actions>
      <v-btn
        color="white"
        variant="text"
        @click="closeSnackbar"
      >
        Close
      </v-btn>
    </template>
  </v-snackbar>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch, type Ref } from 'vue';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import confirmationDialog from '/@/components/misc/confimationDialog.vue';
import { filterActivedFeatured, filterPromotedFeatured } from '/@/utils';
import type { FeaturedReleaseItem, PartialFeaturedReleaseItem } from '/@/types';
import { useAddFeaturedReleaseMutation, useEditFeaturedReleaseMutation, useGetFeaturedReleasesQuery } from '/@/plugins/lensService/hooks';
import { FEATURED_END_TIME_PROPERTY } from '@riffcc/lens-sdk';

const props = defineProps<{
  initialFeatureData: PartialFeaturedReleaseItem | null;
}>();

const emit = defineEmits<{
  'initial-data-consumed': []
}>();

const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

const { data: featuredReleases } = useGetFeaturedReleasesQuery();
const addFeaturedReleaseMutation = useAddFeaturedReleaseMutation({
  onSuccess: () => {
    openSnackbar('Featured release created succefully.', 'success');
    resetForm();
  },
  onError: (e) => {
    console.error('Error creating featured release:', e);
    openSnackbar(`Error creating featured release: ${e.message.slice(0, 200)}`, 'error');
  },
});
const editFeaturedReleaseMutation = useEditFeaturedReleaseMutation({
  onSuccess: () => {
    openSnackbar('Featured release ended succefully.', 'success');
    resetForm();
  },
  onError: (e) => {
    console.error('Error ending featured release:', e);
    openSnackbar(`Error ending featured release: ${e.message.slice(0, 200)}`, 'error');
  },
});

const newFeaturedRelease: Ref<PartialFeaturedReleaseItem> = ref({});

const formRef = ref();
const isLoading = computed(() => addFeaturedReleaseMutation.isPending.value || editFeaturedReleaseMutation.isPending.value);

const featuredItemIdToEnd: Ref<FeaturedReleaseItem | null> = ref(null);

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
  const data = newFeaturedRelease.value;
  if (
    data.releaseId &&
    data.startTime &&
    data.endTime &&
    data.promoted !== undefined &&
    formRef.value?.isValid
  ) {
    const startTime = (new Date(data.startTime)).toISOString();
    const endTime = (new Date(data.endTime)).toISOString();

    return {
      releaseId: data.releaseId,
      startTime,
      endTime,
      promoted: data.promoted,
    };
  }
  return undefined;
});

const handleOnSubmit = async () => {
  if (!readyToSave.value) {
    return;
  }
  addFeaturedReleaseMutation.mutate(readyToSave.value);
};


const confirmEndFeaturedRelease = async () => {
  if (!featuredItemIdToEnd.value) return;
  editFeaturedReleaseMutation.mutate({
    ...featuredItemIdToEnd.value,
    [FEATURED_END_TIME_PROPERTY]: (new Date()).toISOString(),
  });
  featuredItemIdToEnd.value = null;
};

</script>
