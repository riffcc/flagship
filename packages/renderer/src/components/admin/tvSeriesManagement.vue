<template>
  <v-container fluid>
    <v-sheet class="pa-4">
      <h2 class="text-h6 mb-4">Manage TV Series</h2>

      <!-- Warning if backend methods are missing -->
       <v-alert
        v-if="backendMissing"
        type="warning"
        variant="tonal"
        class="mb-4"
        title="Backend Support Missing"
        text="The backend does not fully support TV Series management (add, edit, or delete functions are missing). Functionality will be limited."
      ></v-alert>

      <!-- Add New Series Button/Dialog (Optional) -->
      <v-dialog v-model="dialogOpen" max-width="600px">
        <template #activator="{ props }">
          <v-btn color="primary" class="mb-4" v-bind="props" :disabled="!canAddSeries && !editingSeries">Add New Series</v-btn>
        </template>
        <v-card>
          <v-card-title>
            <span class="text-h5">{{ editingSeries ? 'Edit Series' : 'Add New Series' }}</span>
          </v-card-title>
          <v-card-text>
            <v-form ref="seriesFormRef">
              <v-text-field
                v-model="currentSeries.name"
                label="Series Name"
                :rules="[rules.required]"
                required
              ></v-text-field>
              <v-textarea
                v-model="currentSeries.description"
                label="Description (Optional)"
                rows="3"
              ></v-textarea>
              <v-text-field
                v-model="currentSeries.thumbnail"
                label="Thumbnail CID (Optional)"
                :rules="[rules.isValidCid]"
              ></v-text-field>
              <v-text-field
                v-model="currentSeries.cover"
                label="Cover Image CID (Optional)"
                :rules="[rules.isValidCid]"
              ></v-text-field>
              <!-- Add other series fields here -->
            </v-form>
          </v-card-text>
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn color="blue darken-1" text @click="closeDialog">Cancel</v-btn>
            <v-btn
              color="blue darken-1"
              text
              @click="saveSeries"
              :loading="isSaving"
              :disabled="editingSeries ? !canEditSeries : !canAddSeries"
            >Save</v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>

      <!-- List of Existing Series -->
      <v-data-table
        :headers="headers"
        :items="tvSeries"
        :loading="isLoading"
        item-value="id"
        class="elevation-1"
      >
        <template #item.actions="{ item }">
           <v-tooltip text="Edit Series" location="top">
            <template #activator="{ props }">
              <v-icon v-bind="props" small class="mr-2" @click="editSeries(item.raw as TvSeries)" :disabled="!canEditSeries">mdi-pencil</v-icon>
            </template>
          </v-tooltip>
           <v-tooltip text="Delete Series" location="top">
             <template #activator="{ props }">
              <v-icon v-bind="props" small @click="confirmDelete(item.raw as TvSeries)" :disabled="!canDeleteSeries">mdi-delete</v-icon>
            </template>
          </v-tooltip>
        </template>
         <template #item.thumbnail="{ item }">
           <v-img :src="parseUrlOrCid(item.raw.thumbnail)" height="40" width="60" aspect-ratio="16/9" cover class="my-1"></v-img>
        </template>
         <template #item.cover="{ item }">
           <v-img :src="parseUrlOrCid(item.raw.cover)" height="40" width="60" aspect-ratio="16/9" cover class="my-1"></v-img>
        </template>
        <template #loading>
          <v-skeleton-loader type="table-row@5"></v-skeleton-loader>
        </template>
         <template #no-data>
           <p class="text-center py-4">No TV Series found.</p>
         </template>
      </v-data-table>

       <!-- Confirmation Dialog -->
      <confirmation-dialog
        ref="confirmationDialogRef"
        @confirm="deleteConfirmed"
      />

    </v-sheet>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useTvSeriesStore } from '/@/stores/tvSeries';
import type { TvSeries, PartialTvSeries } from '/@/@types/release';
import { useOrbiter } from '/@/plugins/orbiter/utils';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import ConfirmationDialog from '/@/components/misc/confimationDialog.vue';
import { cid } from 'is-ipfs';
import { parseUrlOrCid } from '/@/utils';

const { orbiter } = useOrbiter();
const tvSeriesStore = useTvSeriesStore();
const { tvSeries, isLoading } = storeToRefs(tvSeriesStore);
const { open: openSnackbar } = useSnackbarMessage();

const dialogOpen = ref(false);
const editingSeries = ref<TvSeries | null>(null);
const currentSeries = ref<PartialTvSeries>({});
const seriesFormRef = ref<any>(null); // For form validation
const isSaving = ref(false);
const confirmationDialogRef = ref<InstanceType<typeof ConfirmationDialog> | null>(null);
const seriesToDelete = ref<TvSeries | null>(null);

// Check if Orbiter methods exist
const canAddSeries = computed(() => typeof orbiter.addTvSeries === 'function');
const canEditSeries = computed(() => typeof orbiter.editTvSeries === 'function');
const canDeleteSeries = computed(() => typeof orbiter.deleteTvSeries === 'function');
const backendMissing = computed(() => !canAddSeries.value || !canEditSeries.value || !canDeleteSeries.value);

const rules = {
  required: (v: string) => !!v || 'Required field.',
  isValidCid: (v: string) => !v || cid(v) || 'Please enter a valid CID.',
};

const headers = [
  { title: 'Name', key: 'name', sortable: true },
  { title: 'Description', key: 'description', sortable: false },
  { title: 'Thumbnail', key: 'thumbnail', sortable: false },
  { title: 'Cover', key: 'cover', sortable: false },
  { title: 'Actions', key: 'actions', sortable: false },
];

watch(dialogOpen, (isOpen) => {
  if (!isOpen) {
    editingSeries.value = null;
    currentSeries.value = {};
    seriesFormRef.value?.resetValidation();
  }
});

const editSeries = (series: TvSeries) => {
  editingSeries.value = series;
  currentSeries.value = { ...series }; // Copy to avoid modifying store directly
  dialogOpen.value = true;
};

const closeDialog = () => {
  dialogOpen.value = false;
};

const saveSeries = async () => {
  const { valid } = await seriesFormRef.value?.validate();
  if (!valid) return;

  isSaving.value = true;
  try {
    const seriesData = { ...currentSeries.value };
    // Remove ID if it exists but is empty or null (for creation)
    if (!seriesData.id) delete seriesData.id;

    if (editingSeries.value && editingSeries.value.id) {
      // Edit existing series
      // Ensure ID is not part of the data payload if Orbiter expects it separately
      const payload = { ...seriesData };
      delete payload.id;
      await orbiter.editTvSeries({ seriesId: editingSeries.value.id, series: payload }); // Adjust based on Orbiter API
      openSnackbar('TV Series updated successfully!', 'success');
    } else {
      // Add new series
      await orbiter.addTvSeries(seriesData); // Adjust based on Orbiter API
      openSnackbar('TV Series added successfully!', 'success');
    }
    closeDialog();
  } catch (error) {
    console.error('Error saving TV Series:', error);
    openSnackbar(`Error saving TV Series: ${error.message || 'Unknown error'}`, 'error');
  } finally {
    isSaving.value = false;
  }
};

const confirmDelete = (series: TvSeries) => {
    seriesToDelete.value = series;
    confirmationDialogRef.value?.open(
        'Delete TV Series',
        `Are you sure you want to delete the series "${series.name}"? This action cannot be undone. Note: This does not delete associated episode releases.`,
        { color: 'error', confirmText: 'Delete' }
    );
};

const deleteConfirmed = async () => {
    if (!seriesToDelete.value || !seriesToDelete.value.id) return;

    try {
        await orbiter.deleteTvSeries(seriesToDelete.value.id); // Adjust based on Orbiter API
        openSnackbar('TV Series deleted successfully!', 'success');
        seriesToDelete.value = null;
    } catch (error) {
        console.error('Error deleting TV Series:', error);
        openSnackbar(`Error deleting TV Series: ${error.message || 'Unknown error'}`, 'error');
    }
};

</script>

<style scoped>
/* Add any specific styles if needed */
</style>
