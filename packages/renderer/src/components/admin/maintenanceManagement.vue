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

    <v-card class="mt-6">
      <v-card-title>Cleanup Empty Structures</v-card-title>
      <v-card-text>
        <p class="mb-4">Remove empty structures (TV series, seasons, artists, albums) that have no associated content.</p>
        <v-btn
          color="warning"
          prepend-icon="$delete"
          :loading="isCleaningUp"
          @click="cleanupEmptyStructures"
        >
          Cleanup Empty Structures
        </v-btn>
        <v-alert
          v-if="cleanupResults"
          :type="cleanupResults.error ? 'error' : 'success'"
          class="mt-4"
        >
          {{ cleanupResults.message }}
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
import { useQueryClient } from '@tanstack/vue-query';
import { useGetReleasesQuery, useGetFeaturedReleasesQuery, useAddReleaseMutation, useEditReleaseMutation, useWasmP2pDeleteReleaseMutation, useWasmP2pDeleteFeaturedReleaseMutation, useAddFeaturedReleaseMutation, useEditFeaturedReleaseMutation, useContentCategoriesQuery, useGetStructuresQuery, useDeleteStructureMutation, useBulkDeleteAllReleasesMutation } from '/@/plugins/lensService/hooks';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import { useIdentity } from '/@/composables/useIdentity';
import type { ReleaseItem } from '/@/types';

const queryClient = useQueryClient();

const isExporting = ref(false);
const isImporting = ref(false);
const isCleaningUp = ref(false);
const importMode = ref<'upsert' | 'replace'>('upsert');
const importFile = ref<File | null>(null);
const confirmDialog = ref(false);
const cleanupResults = ref<{ message: string; error: boolean } | null>(null);

const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();
const { publicKey } = useIdentity();

// Queries
const { data: releases } = useGetReleasesQuery();
const { data: featuredReleases } = useGetFeaturedReleasesQuery();
const { data: contentCategories } = useContentCategoriesQuery();

// Mutations
const addReleaseMutation = useAddReleaseMutation({
  onError: (e) => console.error('Failed to add release:', e),
});

const editReleaseMutation = useEditReleaseMutation({
  onError: (e) => console.error('Failed to edit release:', e),
});

const deleteReleaseMutation = useWasmP2pDeleteReleaseMutation({
  onError: (e) => console.error('Failed to delete release:', e),
});

const bulkDeleteAllReleasesMutation = useBulkDeleteAllReleasesMutation({
  onError: (e) => console.error('Failed to bulk delete releases:', e),
});

const addFeaturedReleaseMutation = useAddFeaturedReleaseMutation({
  onError: (e) => console.error('Failed to add featured release:', e),
});

const editFeaturedReleaseMutation = useEditFeaturedReleaseMutation({
  onError: (e) => console.error('Failed to edit featured release:', e),
});

const deleteFeaturedReleaseMutation = useWasmP2pDeleteFeaturedReleaseMutation({
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
    // Create mapping of category ID to slug
    const categoryIdToSlugMap = new Map<string, string>();
    if (contentCategories.value) {
      contentCategories.value.forEach(cat => {
        categoryIdToSlugMap.set(cat.id, cat.categoryId);
      });
    }

    // Add categorySlug to each release
    const releasesWithSlug = (releases.value || []).map(release => ({
      ...release,
      categorySlug: categoryIdToSlugMap.get(release.categoryId)
    }));

    const cleanedReleases = cleanForExport(releasesWithSlug) as unknown[];
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
  isImporting.value = true;

  try {
    await deleteAllData();
    await performImport();
  } catch (error) {
    console.error('Replace all failed:', error);
    openSnackbar('Replace all failed: ' + (error instanceof Error ? error.message : String(error)), 'error');
    isImporting.value = false;
  }
};

const deleteAllData = async () => {
  try {
    let featuredDeleted = 0;
    let releasesDeleted = 0;

    // Delete all featured releases first (still one-by-one as there's no bulk endpoint yet)
    if (featuredReleases.value && featuredReleases.value.length > 0) {
      openSnackbar(`Deleting ${featuredReleases.value.length} featured releases...`, 'info');
      for (const featured of featuredReleases.value) {
        try {
          const result = await deleteFeaturedReleaseMutation.mutateAsync(featured.id);
          // WASM P2P mutations return {id: "..."} on success
          if (result && result.id) {
            featuredDeleted++;
          }
        } catch (err) {
          // Continue on error
        }
      }
    } else {
      openSnackbar('No featured releases to delete', 'info');
    }

    // Then delete all releases using bulk delete (efficient single UBTS block)
    if (releases.value && releases.value.length > 0) {
      openSnackbar(`Deleting ${releases.value.length} releases in bulk...`, 'info');
      try {
        const result = await bulkDeleteAllReleasesMutation.mutateAsync();
        releasesDeleted = result.deleted;
        openSnackbar(`Bulk delete complete: ${releasesDeleted} releases deleted (transaction: ${result.delete_transaction_id})`, 'success');
      } catch (err) {
        openSnackbar('Bulk delete failed: ' + (err instanceof Error ? err.message : String(err)), 'error');
        throw err;
      }
    } else {
      openSnackbar('No releases to delete', 'info');
    }

    // Wait a bit for queries to update
    await new Promise(resolve => setTimeout(resolve, 1000));

    openSnackbar(`Deleted ${releasesDeleted} releases and ${featuredDeleted} featured releases`, 'success');
  } catch (error) {
    openSnackbar('Failed to delete existing data: ' + (error instanceof Error ? error.message : String(error)), 'error');
    throw error;
  }
};

// Helper function to map category slug to ID
const getCategoryIdFromSlug = (categorySlug: string): string => {
  if (!contentCategories.value) return categorySlug;

  // First check if it's already a valid category ID
  const existingById = contentCategories.value.find(c => c.id === categorySlug);
  if (existingById) return categorySlug;

  // Normalize the input slug
  const normalizedInput = categorySlug.toLowerCase().trim();

  // Try exact match with category slugs
  const exactMatch = contentCategories.value.find(c =>
    c.categoryId?.toLowerCase() === normalizedInput
  );
  if (exactMatch) return exactMatch.id;

  // Try matching by adding/removing 's' for plural/singular
  const withS = normalizedInput.endsWith('s') ? normalizedInput : normalizedInput + 's';
  const withoutS = normalizedInput.endsWith('s') ? normalizedInput.slice(0, -1) : normalizedInput;

  const pluralMatch = contentCategories.value.find(c =>
    c.categoryId?.toLowerCase() === withS || c.categoryId?.toLowerCase() === withoutS
  );
  if (pluralMatch) return pluralMatch.id;

  // Try matching with spaces converted to hyphens and vice versa
  const withHyphens = normalizedInput.replace(/\s+/g, '-');
  const withSpaces = normalizedInput.replace(/-/g, ' ');

  const formattedMatch = contentCategories.value.find(c =>
    c.categoryId?.toLowerCase() === withHyphens ||
    c.categoryId?.toLowerCase() === withSpaces
  );
  if (formattedMatch) return formattedMatch.id;

  // If nothing matches, return the original (will likely fail, but preserves the error)
  return categorySlug;
};

const performImport = async () => {
  if (!importFile.value) return;

  isImporting.value = true;

  try {
    const text = await importFile.value.text();
    const importData = JSON.parse(text);

    if (!importData.version || !importData.releases) {
      throw new Error('Invalid import file format');
    }

    // Use bulk HTTP API import endpoint for efficient importing
    openSnackbar(`Importing ${importData.releases.length} releases via bulk API...`, 'info');

    const { API_URL } = await import('/@/plugins/router');
    const response = await fetch(`${API_URL}/import`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        publicKey: publicKey.value,
        data: importData,
      }),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Import failed: ${response.statusText}`);
    }

    const result = await response.json();

    // Result format: { success: boolean, imported: number, skipped: number, errors: string[] }
    if (result.errors && result.errors.length > 0) {
      console.error('Import errors:', result.errors);
      openSnackbar(
        `Import completed with warnings: ${result.imported} imported, ${result.skipped} skipped. Check console for errors.`,
        'warning'
      );
    } else {
      openSnackbar(
        `Import successful: ${result.imported} releases imported, ${result.skipped} already existed`,
        'success'
      );
    }

    // Wait for backend to sync, then force refetch to refresh UI
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Force immediate refetch of queries (using queryClient from setup)
    await queryClient.refetchQueries({ queryKey: ['releases'] });
    await queryClient.refetchQueries({ queryKey: ['featuredReleases'] });

    importFile.value = null;
  } catch (error) {
    openSnackbar('Import failed: ' + (error instanceof Error ? error.message : String(error)), 'error');
    console.error('Import error:', error);
  } finally {
    isImporting.value = false;
  }
};

// Structures query and mutation for cleanup
const { data: structures } = useGetStructuresQuery();
const deleteStructureMutation = useDeleteStructureMutation();

// Cleanup empty structures
const cleanupEmptyStructures = async () => {
  isCleaningUp.value = true;
  cleanupResults.value = null;

  try {
    if (!structures.value || !releases.value) {
      cleanupResults.value = { message: 'No structures or releases found', error: true };
      return;
    }

    let deletedCount = 0;
    const errors: string[] = [];

    // Find empty series (series with no episodes)
    const tvSeries = structures.value.filter((s: any) => s.type === 'series');
    for (const series of tvSeries) {
      const hasEpisodes = releases.value.some((r: any) =>
        r.metadata?.seriesId === series.id
      );

      if (!hasEpisodes) {
        try {
          await deleteStructureMutation.mutateAsync(series.id);
          deletedCount++;
          console.log(`Deleted empty series: ${series.name}`);
        } catch (error) {
          errors.push(`Failed to delete series ${series.name}: ${error}`);
        }
      }
    }

    // Find empty seasons (seasons with no episodes)
    const seasons = structures.value.filter((s: any) => s.type === 'season');
    for (const season of seasons) {
      const hasEpisodes = releases.value.some((r: any) =>
        r.metadata?.seriesId === season.parentId &&
        r.metadata?.seasonNumber === season.metadata?.seasonNumber
      );

      if (!hasEpisodes) {
        try {
          await deleteStructureMutation.mutateAsync(season.id);
          deletedCount++;
          console.log(`Deleted empty season: ${season.name || `Season ${season.metadata?.seasonNumber}`}`);
        } catch (error) {
          errors.push(`Failed to delete season ${season.name}: ${error}`);
        }
      }
    }

    // Find empty artists (artists with no releases)
    const artists = structures.value.filter((s: any) => s.type === 'artist');
    for (const artist of artists) {
      const hasReleases = releases.value.some((r: any) =>
        r.metadata?.artistId === artist.id || r.metadata?.structureId === artist.id
      );

      if (!hasReleases) {
        try {
          await deleteStructureMutation.mutateAsync(artist.id);
          deletedCount++;
          console.log(`Deleted empty artist: ${artist.name}`);
        } catch (error) {
          errors.push(`Failed to delete artist ${artist.name}: ${error}`);
        }
      }
    }

    // Find empty albums (albums with no tracks)
    const albums = structures.value.filter((s: any) => s.type === 'album');
    for (const album of albums) {
      const hasTracks = releases.value.some((r: any) =>
        r.metadata?.albumId === album.id || r.metadata?.structureId === album.id
      );

      if (!hasTracks) {
        try {
          await deleteStructureMutation.mutateAsync(album.id);
          deletedCount++;
          console.log(`Deleted empty album: ${album.name}`);
        } catch (error) {
          errors.push(`Failed to delete album ${album.name}: ${error}`);
        }
      }
    }

    if (errors.length > 0) {
      cleanupResults.value = {
        message: `Deleted ${deletedCount} empty structures. Errors: ${errors.join(', ')}`,
        error: true
      };
    } else {
      cleanupResults.value = {
        message: `Successfully deleted ${deletedCount} empty structures`,
        error: false
      };
    }
  } catch (error) {
    cleanupResults.value = { message: `Cleanup failed: ${error}`, error: true };
  } finally {
    isCleaningUp.value = false;
  }
};
</script>
