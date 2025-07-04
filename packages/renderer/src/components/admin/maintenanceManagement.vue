<template>
  <v-container>
    <h2 class="text-h5 mb-6">Maintenance</h2>

    <v-card class="mb-6">
      <v-card-title>Export Data</v-card-title>
      <v-card-text>
        <p class="mb-4">Export all releases and featured releases to a JSON file for backup or migration.</p>
        <v-btn
          color="primary"
          prepend-icon="$download"
          :loading="isExporting"
          @click="exportAll"
        >
          Export All
        </v-btn>
      </v-card-text>
    </v-card>

    <v-card>
      <v-card-title>Import Data</v-card-title>
      <v-card-text>
        <p class="mb-4">Import releases and featured releases from a JSON file.</p>

        <v-radio-group
          v-model="importMode"
          class="mb-4"
        >
          <v-radio
            label="Upsert - Add new and update existing releases (recommended)"
            value="upsert"
          ></v-radio>
          <v-radio
            label="Replace All - Delete all existing data and replace with imported data"
            value="replace"
            color="error"
          ></v-radio>
        </v-radio-group>

        <v-file-input
          v-model="importFile"
          label="Select JSON file"
          accept=".json"
          prepend-icon="$file-upload"
          class="mb-4"
        ></v-file-input>

        <v-btn
          color="primary"
          prepend-icon="$upload"
          :disabled="!importFile"
          :loading="isImporting"
          @click="importAll"
        >
          Import
        </v-btn>

        <v-alert
          v-if="importMode === 'replace'"
          type="warning"
          class="mt-4"
        >
          Warning: Replace All will permanently delete all existing releases and featured releases before importing.
        </v-alert>
      </v-card-text>
    </v-card>

    <v-dialog
      v-model="confirmDialog"
      max-width="500"
    >
      <v-card>
        <v-card-title>Confirm Replace All</v-card-title>
        <v-card-text>
          Are you sure you want to delete all existing releases and featured releases? This action cannot be undone.
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn
            text
            @click="confirmDialog = false"
          >
            Cancel
          </v-btn>
          <v-btn
            color="error"
            variant="flat"
            @click="confirmReplaceAll"
          >
            Delete All & Import
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
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
import { ref } from 'vue';
import { useGetReleasesQuery, useGetFeaturedReleasesQuery, useAddReleaseMutation, useEditReleaseMutation, useDeleteReleaseMutation, useAddFeaturedReleaseMutation, useEditFeaturedReleaseMutation, useDeleteFeaturedReleaseMutation } from '/@/plugins/lensService/hooks';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import type { ReleaseItem } from '/@/types';
import type { AnyObject } from '@riffcc/lens-sdk';

const isExporting = ref(false);
const isImporting = ref(false);
const importMode = ref<'upsert' | 'replace'>('upsert');
const importFile = ref<File | null>(null);
const confirmDialog = ref(false);

const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

// Queries
const { data: releases } = useGetReleasesQuery();
const { data: featuredReleases } = useGetFeaturedReleasesQuery();

// Mutations
const addReleaseMutation = useAddReleaseMutation({
  onError: (e) => console.error('Failed to add release:', e),
});

const editReleaseMutation = useEditReleaseMutation({
  onError: (e) => console.error('Failed to edit release:', e),
});

const deleteReleaseMutation = useDeleteReleaseMutation({
  onError: (e) => console.error('Failed to delete release:', e),
});

const addFeaturedReleaseMutation = useAddFeaturedReleaseMutation({
  onError: (e) => console.error('Failed to add featured release:', e),
});

const editFeaturedReleaseMutation = useEditFeaturedReleaseMutation({
  onError: (e) => console.error('Failed to edit featured release:', e),
});

const deleteFeaturedReleaseMutation = useDeleteFeaturedReleaseMutation({
  onError: (e) => console.error('Failed to delete featured release:', e),
});

// Helper to clean data for export (remove __context and handle BigInts)
const cleanForExport = (obj: unknown): unknown => {
  if (obj === null || obj === undefined) return obj;
  if (typeof obj === 'bigint') return obj.toString();
  if (obj instanceof Date) return obj.toISOString();
  if (Array.isArray(obj)) return obj.map(cleanForExport);
  if (typeof obj === 'object' && obj !== null) {
    const cleaned: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      // Skip __context as it contains BigInts and isn't needed for import
      if (key === '__context') continue;
      cleaned[key] = cleanForExport(value);
    }
    return cleaned;
  }
  return obj;
};

// Export functionality
const exportAll = async () => {
  isExporting.value = true;

  try {
    const cleanedReleases = cleanForExport(releases.value || []) as unknown[];
    const cleanedFeaturedReleases = cleanForExport(featuredReleases.value || []) as unknown[];

    const exportData = {
      version: '1.0',
      exportDate: new Date().toISOString(),
      releases: cleanedReleases,
      featuredReleases: cleanedFeaturedReleases,
    };

    const jsonStr = JSON.stringify(exportData, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    a.download = `flagship-export-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    openSnackbar(`Exported ${cleanedReleases.length} releases and ${cleanedFeaturedReleases.length} featured releases`, 'success');
  } catch (error) {
    openSnackbar('Export failed: ' + (error instanceof Error ? error.message : String(error)), 'error');
  } finally {
    isExporting.value = false;
  }
};

// Import functionality
const importAll = async () => {
  if (!importFile.value) return;

  if (importMode.value === 'replace') {
    confirmDialog.value = true;
    return;
  }

  performImport();
};

const confirmReplaceAll = async () => {
  confirmDialog.value = false;
  await deleteAllData();
  await performImport();
};

const deleteAllData = async () => {
  try {
    let featuredDeleted = 0;
    let releasesDeleted = 0;

    // Delete all featured releases first
    if (featuredReleases.value && featuredReleases.value.length > 0) {
      console.log(`Deleting ${featuredReleases.value.length} featured releases...`);
      for (const featured of featuredReleases.value) {
        try {
          const result = await deleteFeaturedReleaseMutation.mutateAsync({ id: featured.id });
          if (result.success) {
            featuredDeleted++;
          } else {
            console.error(`Failed to delete featured release ${featured.id}:`, result.error);
          }
        } catch (err) {
          console.error(`Error deleting featured release ${featured.id}:`, err);
        }
      }
    }

    // Then delete all releases
    if (releases.value && releases.value.length > 0) {
      console.log(`Deleting ${releases.value.length} releases...`);
      for (const release of releases.value) {
        try {
          const result = await deleteReleaseMutation.mutateAsync({ id: release.id });
          if (result.success) {
            releasesDeleted++;
          } else {
            console.error(`Failed to delete release ${release.id}:`, result.error);
          }
        } catch (err) {
          console.error(`Error deleting release ${release.id}:`, err);
        }
      }
    }

    // Wait a bit for queries to update
    await new Promise(resolve => setTimeout(resolve, 1000));

    openSnackbar(`Deleted ${releasesDeleted} releases and ${featuredDeleted} featured releases`, 'success');
  } catch (error) {
    openSnackbar('Failed to delete existing data: ' + (error instanceof Error ? error.message : String(error)), 'error');
    throw error;
  }
};

const performImport = async () => {
  if (!importFile.value) return;

  isImporting.value = true;

  try {
    const text = await importFile.value.text();
    const importData = JSON.parse(text);

    if (!importData.version || !importData.releases || !importData.featuredReleases) {
      throw new Error('Invalid import file format');
    }

    let releasesImported = 0;
    let featuredImported = 0;

    // Import releases
    for (const release of importData.releases) {
      try {
        // Extract the data without the __context
        const releaseData: Omit<ReleaseItem<AnyObject>, 'siteAddress'> = {
          id: release.id,
          name: release.name,
          categoryId: release.categoryId,
          contentCID: release.contentCID,
          thumbnailCID: release.thumbnailCID,
          metadata: release.metadata,
        };

        if (importMode.value === 'upsert') {
          // Check if release exists
          const existing = releases.value?.find(r => r.id === release.id);
          if (existing) {
            // Update existing
            await editReleaseMutation.mutateAsync({
              ...releaseData,
              siteAddress: existing.siteAddress,
            });
            releasesImported++;
          } else {
            // Add new
            await addReleaseMutation.mutateAsync(releaseData);
            releasesImported++;
          }
        } else {
          // Replace mode - just add
          await addReleaseMutation.mutateAsync(releaseData);
          releasesImported++;
        }
      } catch (error) {
        console.error('Failed to import release:', release.id, error);
      }
    }

    // Import featured releases
    for (const featured of importData.featuredReleases) {
      try {
        const featuredData = {
          releaseId: featured.releaseId,
          startTime: featured.startTime,
          endTime: featured.endTime,
          promoted: featured.promoted,
        };

        if (importMode.value === 'upsert') {
          // Check if featured release exists
          const existing = featuredReleases.value?.find(f => f.id === featured.id);
          if (existing) {
            // Update existing
            await editFeaturedReleaseMutation.mutateAsync({
              ...featuredData,
              id: featured.id,
              siteAddress: existing.siteAddress,
            });
            featuredImported++;
          } else {
            // Add new
            await addFeaturedReleaseMutation.mutateAsync(featuredData);
            featuredImported++;
          }
        } else {
          // Replace mode - just add
          await addFeaturedReleaseMutation.mutateAsync(featuredData);
          featuredImported++;
        }
      } catch (error) {
        console.error('Failed to import featured release:', featured.id, error);
      }
    }

    openSnackbar(`Import complete: ${releasesImported} releases and ${featuredImported} featured releases imported`, 'success');
    importFile.value = null;
  } catch (error) {
    openSnackbar('Import failed: ' + (error instanceof Error ? error.message : String(error)), 'error');
  } finally {
    isImporting.value = false;
  }
};
</script>
