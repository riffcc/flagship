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
            <v-autocomplete
              v-model="selectedRelease"
              :items="releases || []"
              :loading="releasesLoading"
              :item-title="(item) => item.name"
              :item-value="(item) => item"
              label="Select Release"
              clearable
              validate-on="input"
              :rules="releaseRules"
              @update:search="searchQuery = $event"
            >
              <template #item="{ props, item }">
                <v-list-item
                  v-bind="props"
                  :title="item.raw.name"
                  :subtitle="`ID: ${item.raw.id.substring(0, 8)}...`"
                />
              </template>
            </v-autocomplete>
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
        lg="7"
      >
        <v-sheet
          class="px-6 py-4 mx-auto h-100"
          max-width="600px"
          min-height="256px"
        >
          <h6 class="text-h6 font-weight-bold mb-4">Featured Releases</h6>
          <v-list 
            v-if="(featuredWithReleases?.length ?? 0) > 0"
            class="featured-list"
          >
            <template
              v-for="(featuredRelease, index) in featuredWithReleases"
              :key="featuredRelease.id"
            >
              <v-list-item
                class="px-0 featured-item"
                :class="{ 'dragging': draggedIndex === index }"
                :draggable="filterActivedFeatured(featuredRelease)"
                @dragstart="onDragStart(index, $event)"
                @dragend="onDragEnd"
                @dragover="onDragOver(index, $event)"
                @drop="onDrop(index, $event)"
                @click="filterActivedFeatured(featuredRelease) ? editFeaturedRelease(featuredRelease) : null"
                :style="{ cursor: filterActivedFeatured(featuredRelease) ? 'pointer' : 'default' }"
              >
                <template #prepend>
                  <v-icon
                    v-if="filterActivedFeatured(featuredRelease)"
                    icon="mdi-drag-vertical"
                    class="drag-handle mr-2"
                    :style="{ cursor: 'grab' }"
                  />
                  <v-sheet
                    width="90"
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
                <template #title>
                  <div class="d-flex flex-column">
                    <span class="text-subtitle-1">{{ featuredRelease.releaseName || 'Loading...' }}</span>
                    <span class="text-caption text-medium-emphasis">
                      {{ formatDateRange(featuredRelease.startTime, featuredRelease.endTime) }}
                    </span>
                  </div>
                </template>
                <template #append>
                  <v-btn
                    :icon="filterActivedFeatured(featuredRelease) ? '$check' : 'mdi-delete'"
                    size="small"
                    density="compact"
                    :color="filterActivedFeatured(featuredRelease) ? 'warning' : 'error'"
                    @click.stop="filterActivedFeatured(featuredRelease) ? (featuredItemIdToEnd = featuredRelease) : confirmRemoveFeaturedRelease(featuredRelease)"
                  >
                  </v-btn>
                </template>
              </v-list-item>
              <v-divider v-if="index < featuredWithReleases.length - 1" />
            </template>
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
    
    <!-- Edit Dialog -->
    <v-dialog
      v-model="showEditDialog"
      width="400"
    >
      <v-card>
        <v-card-title>Edit Featured Release</v-card-title>
        <v-card-text>
          <v-form ref="editFormRef">
            <v-text-field
              v-model="editingFeatured.startTime"
              type="datetime-local"
              :min="minEditDate"
              :max="maxDate"
              :rules="rules"
              label="Start at"
            ></v-text-field>
            <v-text-field
              v-model="editingFeatured.endTime"
              type="datetime-local"
              :min="minEditEndDate"
              :max="maxDate"
              :rules="editEndAtRules"
              label="End at"
            ></v-text-field>
            <v-switch
              v-model="editingFeatured.promoted"
              :color="editingFeatured.promoted ? 'primary' : 'default'"
              label="Promoted"
            ></v-switch>
          </v-form>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn
            text="Cancel"
            @click="showEditDialog = false"
          />
          <v-btn
            color="primary"
            text="Save"
            :loading="isLoading"
            @click="saveEditedFeatured"
          />
        </v-card-actions>
      </v-card>
    </v-dialog>
    
    <confirmation-dialog
      message="Are you sure you want to end this featured release?"
      :dialog-open="Boolean(featuredItemIdToEnd)"
      :on-close="() => featuredItemIdToEnd = null"
      :on-confirm="confirmEndFeaturedRelease"
    ></confirmation-dialog>
    
    <confirmation-dialog
      message="Remove this ended featured release from the list?"
      :dialog-open="Boolean(featuredItemToRemove)"
      :on-close="() => featuredItemToRemove = null"
      :on-confirm="confirmRemoveFeaturedRelease"
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
import type { FeaturedReleaseItem, ReleaseItem } from '/@/types';
import { 
  useAddFeaturedReleaseMutation, 
  useEditFeaturedReleaseMutation, 
  useGetFeaturedReleasesQuery,
  useGetReleasesQuery,
  useGetReleaseQuery,
  useDeleteFeaturedReleaseMutation
} from '/@/plugins/lensService/hooks';

const props = defineProps<{
  initialFeatureData: Partial<FeaturedReleaseItem> | null;
}>();

const emit = defineEmits<{
  'initial-data-consumed': []
}>();

const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

const { data: featuredReleases } = useGetFeaturedReleasesQuery();
const { data: releases, isLoading: releasesLoading } = useGetReleasesQuery();

// Enhanced featured releases with release names and sorted by order
const featuredWithReleases = computed(() => {
  if (!featuredReleases.value) return [];
  
  const enhanced = featuredReleases.value.map(featured => ({
    ...featured,
    releaseName: releases.value?.find(r => r.id === featured.releaseId)?.name || 'Unknown Release',
    // Ensure order exists for TypeScript, but it might be undefined
    order: (featured as any).order
  }));
  
  // Sort by order field if present, otherwise by created date
  return enhanced.sort((a, b) => {
    // If both have order, sort by order (ascending)
    if (a.order !== undefined && b.order !== undefined) {
      return a.order - b.order;
    }
    // If only one has order, it comes first
    if (a.order !== undefined) return -1;
    if (b.order !== undefined) return 1;
    // Otherwise sort by created date (newest first)
    return new Date(b.created).getTime() - new Date(a.created).getTime();
  });
});

const addFeaturedReleaseMutation = useAddFeaturedReleaseMutation({
  onSuccess: () => {
    openSnackbar('Featured release created successfully.', 'success');
    resetForm();
  },
  onError: (e) => {
    console.error('Error creating featured release:', e);
    openSnackbar(`Error creating featured release: ${e.message.slice(0, 200)}`, 'error');
  },
});

const editFeaturedReleaseMutation = useEditFeaturedReleaseMutation({
  onSuccess: () => {
    openSnackbar('Featured release updated successfully.', 'success');
    showEditDialog.value = false;
  },
  onError: (e) => {
    console.error('Error updating featured release:', e);
    openSnackbar(`Error updating featured release: ${e.message.slice(0, 200)}`, 'error');
  },
});

const deleteFeaturedReleaseMutation = useDeleteFeaturedReleaseMutation({
  onSuccess: () => {
    openSnackbar('Featured release removed successfully.', 'success');
  },
  onError: (e) => {
    console.error('Error removing featured release:', e);
    openSnackbar(`Error removing featured release: ${e.message.slice(0, 200)}`, 'error');
  },
});

const selectedRelease: Ref<ReleaseItem | null> = ref(null);
const newFeaturedRelease: Ref<Partial<FeaturedReleaseItem>> = ref({});
const searchQuery = ref('');

const formRef = ref();
const editFormRef = ref();
const reorderingLoading = ref(false);
const isLoading = computed(() => 
  addFeaturedReleaseMutation.isPending.value || 
  editFeaturedReleaseMutation.isPending.value ||
  deleteFeaturedReleaseMutation.isPending.value ||
  reorderingLoading.value
);

const featuredItemIdToEnd: Ref<FeaturedReleaseItem | null> = ref(null);
const featuredItemToRemove: Ref<FeaturedReleaseItem | null> = ref(null);

// Edit dialog state
const showEditDialog = ref(false);
const editingFeatured: Ref<Partial<FeaturedReleaseItem>> = ref({});

// Drag and drop state
const draggedIndex = ref<number | null>(null);
const dragOverIndex = ref<number | null>(null);

const rules = [
  (value: string) => Boolean(value) || 'Required field.',
];

const releaseRules = [
  (value: ReleaseItem) => Boolean(value) || 'Please select a release.',
];

const endAtRules = [
  (value: string | null) => !newFeaturedRelease.value.startTime || Boolean(value) || 'End date is required if start date is set.',
  (value: string | null) => {
    if (!value || !newFeaturedRelease.value.startTime) return true;
    return new Date(value) > new Date(newFeaturedRelease.value.startTime) || 'End date must be after start date.';
  },
];

const editEndAtRules = [
  (value: string | null) => !editingFeatured.value.startTime || Boolean(value) || 'End date is required if start date is set.',
  (value: string | null) => {
    if (!value || !editingFeatured.value.startTime) return true;
    return new Date(value) > new Date(editingFeatured.value.startTime) || 'End date must be after start date.';
  },
];

const minDate = ref<string | null>(null);
const maxDate = ref<string | null>(null);
const minEndDate = ref<string | null>(null);
const minEditDate = ref<string | null>(null);
const minEditEndDate = ref<string | null>(null);

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
    
    // Find and select the release
    if (newData.releaseId && releases.value) {
      selectedRelease.value = releases.value.find(r => r.id === newData.releaseId) || null;
    }

    emit('initial-data-consumed');
  }
}, { immediate: true });

watch(() => selectedRelease.value, (release) => {
  if (release) {
    newFeaturedRelease.value.releaseId = release.id;
  } else {
    newFeaturedRelease.value.releaseId = undefined;
  }
});

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

watch(() => editingFeatured.value.startTime, newStartAt => {
  minEditEndDate.value = null;
  if (newStartAt) {
    const newDate = new Date(newStartAt);
    newDate.setMinutes(newDate.getMinutes() + 10);
    minEditEndDate.value = newDate.toISOString().substring(0, 16);
  }
});

const resetForm = () => {
  selectedRelease.value = null;
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
    
    // Calculate the next order number
    const maxOrder = featuredReleases.value?.reduce((max, item) => {
      const itemOrder = (item as any).order;
      return itemOrder !== undefined && itemOrder > max ? itemOrder : max;
    }, -1) ?? -1;

    return {
      releaseId: data.releaseId,
      startTime,
      endTime,
      promoted: data.promoted,
      order: maxOrder + 1,
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

const editFeaturedRelease = (featured: FeaturedReleaseItem) => {
  const startDate = new Date(featured.startTime);
  const endDate = new Date(featured.endTime);
  
  // Allow editing past dates for active features
  minEditDate.value = startDate < new Date() ? startDate.toISOString().substring(0, 16) : new Date().toISOString().substring(0, 16);
  
  editingFeatured.value = {
    ...featured,
    startTime: startDate.toISOString().substring(0, 16),
    endTime: endDate.toISOString().substring(0, 16),
  };
  showEditDialog.value = true;
};

const saveEditedFeatured = async () => {
  if (!editFormRef.value?.isValid || !editingFeatured.value.id) return;
  
  const startTime = new Date(editingFeatured.value.startTime!).toISOString();
  const endTime = new Date(editingFeatured.value.endTime!).toISOString();
  
  editFeaturedReleaseMutation.mutate({
    ...editingFeatured.value as FeaturedReleaseItem,
    startTime,
    endTime,
  });
};

const confirmEndFeaturedRelease = async () => {
  if (!featuredItemIdToEnd.value) return;
  editFeaturedReleaseMutation.mutate({
    ...featuredItemIdToEnd.value,
    endTime: (new Date()).toISOString(),
  });
  featuredItemIdToEnd.value = null;
};

const confirmRemoveFeaturedRelease = (featured?: FeaturedReleaseItem) => {
  if (featured) {
    featuredItemToRemove.value = featured;
  } else if (featuredItemToRemove.value) {
    deleteFeaturedReleaseMutation.mutate(featuredItemToRemove.value.id);
    featuredItemToRemove.value = null;
  }
};

const formatDateRange = (startTime: string, endTime: string) => {
  const start = new Date(startTime);
  const end = new Date(endTime);
  const now = new Date();
  
  const formatDate = (date: Date) => {
    return date.toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
    });
  };
  
  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('en-US', { 
      hour: 'numeric', 
      minute: '2-digit',
      hour12: true 
    });
  };
  
  if (start.toDateString() === end.toDateString()) {
    return `${formatDate(start)} • ${formatTime(start)} - ${formatTime(end)}`;
  } else {
    return `${formatDate(start)} ${formatTime(start)} - ${formatDate(end)} ${formatTime(end)}`;
  }
};

// Drag and drop handlers
const onDragStart = (index: number, event: DragEvent) => {
  draggedIndex.value = index;
  event.dataTransfer!.effectAllowed = 'move';
  event.dataTransfer!.setData('text/html', ''); // Firefox requires this
};

const onDragEnd = () => {
  draggedIndex.value = null;
  dragOverIndex.value = null;
};

const onDragOver = (index: number, event: DragEvent) => {
  event.preventDefault();
  event.dataTransfer!.dropEffect = 'move';
  dragOverIndex.value = index;
};

const onDrop = async (dropIndex: number, event: DragEvent) => {
  event.preventDefault();
  
  if (draggedIndex.value === null || draggedIndex.value === dropIndex) {
    return;
  }
  
  const items = [...featuredWithReleases.value];
  const [draggedItem] = items.splice(draggedIndex.value, 1);
  items.splice(dropIndex, 0, draggedItem);
  
  // Update order field for all items
  try {
    reorderingLoading.value = true;
    
    // Update each item with its new order
    const updatePromises = items.map((item, index) => {
      // Only update if order changed or item didn't have order before
      if (item.order !== index || item.order === undefined) {
        return editFeaturedReleaseMutation.mutateAsync({
          ...item,
          order: index
        });
      }
      return Promise.resolve();
    });
    
    await Promise.all(updatePromises);
    openSnackbar('Featured releases reordered successfully!', 'success');
    
  } catch (error) {
    console.error('Error reordering featured releases:', error);
    openSnackbar('Failed to reorder featured releases', 'error');
  } finally {
    reorderingLoading.value = false;
  }
  
  draggedIndex.value = null;
  dragOverIndex.value = null;
};
</script>

<style scoped>
.featured-list {
  max-height: 500px;
  overflow-y: auto;
}

.featured-item {
  transition: all 0.3s ease;
}

.featured-item.dragging {
  opacity: 0.5;
}

.featured-item:hover .drag-handle {
  opacity: 1;
}

.drag-handle {
  opacity: 0.5;
  transition: opacity 0.2s ease;
}

.featured-item[draggable="true"]:hover {
  background-color: rgba(255, 255, 255, 0.05);
}
</style>