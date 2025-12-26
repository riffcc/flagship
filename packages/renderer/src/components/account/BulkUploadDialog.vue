<template>
  <v-dialog
    v-model="dialogOpen"
    max-width="900px"
    persistent
  >
    <v-card>
      <v-card-title class="text-h5 pa-4 d-flex align-center">
        <v-icon class="mr-2">mdi-folder-multiple</v-icon>
        Bulk Upload
        <v-chip
          class="ml-2"
          size="small"
          color="primary"
        >
          {{ parsedAlbums.length }} {{ contentTerminology.plural }}
        </v-chip>
      </v-card-title>

      <v-card-text class="px-4 pb-4">
        <!-- Step indicator -->
        <div class="step-indicator mb-4">
          <div :class="['step', { active: step === 1, completed: step > 1 }]">
            <span class="step-number">1</span>
            <span class="step-label">Parse</span>
          </div>
          <div class="step-divider" />
          <div :class="['step', { active: step === 2, completed: step > 2 }]">
            <span class="step-number">2</span>
            <span class="step-label">Review</span>
          </div>
          <div class="step-divider" />
          <div :class="['step', { active: step === 3 }]">
            <span class="step-number">3</span>
            <span class="step-label">Upload</span>
          </div>
        </div>

        <!-- Step 1: Parsing -->
        <div v-if="step === 1">
          <div class="parsing-progress">
            <v-progress-linear
              :model-value="parseProgress"
              color="primary"
              height="8"
              rounded
            />
            <div class="text-center mt-2 text-caption">
              Parsing metadata... {{ parsedCount }}/{{ totalFolders }}
            </div>
            <div
              v-if="currentParsingFolder"
              class="text-center mt-1 text-caption text-grey"
            >
              {{ currentParsingFolder }}
            </div>
          </div>
        </div>

        <!-- Step 2: Review Albums -->
        <div v-if="step === 2">
          <v-text-field
            v-model="searchQuery"
            :placeholder="`Filter ${contentTerminology.plural}...`"
            prepend-inner-icon="mdi-magnify"
            density="compact"
            hide-details
            class="mb-4"
            clearable
          />

          <div class="albums-table-wrapper">
            <v-table
              density="compact"
              class="albums-table"
            >
              <thead>
                <tr>
                  <th style="width: 32px;">
                    <v-checkbox
                      v-model="selectAll"
                      hide-details
                      density="compact"
                      @change="toggleSelectAll"
                    />
                  </th>
                  <th>Artist</th>
                  <th>Album</th>
                  <th style="width: 80px;">Year</th>
                  <th style="width: 100px;">Tracks</th>
                  <th style="width: 120px;">Category</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="album in filteredAlbums"
                  :key="album.id"
                  :class="{ 'album-row--selected': album.selected }"
                >
                  <td>
                    <v-checkbox
                      v-model="album.selected"
                      hide-details
                      density="compact"
                    />
                  </td>
                  <td>
                    <v-text-field
                      v-model="album.artist"
                      variant="plain"
                      hide-details
                      density="compact"
                      placeholder="Unknown Artist"
                    />
                  </td>
                  <td>
                    <div class="album-cell">
                      <v-text-field
                        v-model="album.album"
                        variant="plain"
                        hide-details
                        density="compact"
                        :placeholder="album.folderName"
                        class="flex-grow-1"
                      />
                      <div class="album-cover-cell ml-2">
                        <img
                          v-if="album.coverArtUrl"
                          :src="album.coverArtUrl"
                          class="album-cover-thumb"
                          alt=""
                        />
                        <v-icon
                          v-else
                          size="24"
                          color="grey-darken-1"
                        >mdi-album</v-icon>
                      </div>
                    </div>
                  </td>
                  <td>
                    <v-text-field
                      v-model.number="album.year"
                      variant="plain"
                      hide-details
                      density="compact"
                      type="number"
                      placeholder="----"
                    />
                  </td>
                  <td class="text-caption text-grey">
                    {{ getRelevantFileCount(album) }} {{ contentTerminology.trackLabel }}
                  </td>
                  <td>
                    <v-select
                      v-model="album.categoryId"
                      :items="categoryItems"
                      variant="plain"
                      hide-details
                      density="compact"
                    />
                  </td>
                </tr>
              </tbody>
            </v-table>
          </div>

          <div class="d-flex justify-space-between align-center mt-3">
            <div class="text-caption text-grey">
              {{ selectedCount }} of {{ parsedAlbums.length }} {{ contentTerminology.plural }} selected
            </div>
            <v-btn
              size="small"
              variant="text"
              color="primary"
              @click="autoDetectCategories"
            >
              Auto-detect categories
            </v-btn>
          </div>
        </div>

        <!-- Step 3: Upload Progress -->
        <div v-if="step === 3">
          <div class="overall-progress mb-4">
            <div class="d-flex justify-space-between align-center mb-1">
              <span class="text-caption text-grey">Overall Progress</span>
              <span class="text-caption">
                {{ overallProgress }}%
                <span
                  v-if="isUploading && uploadSpeed > 0"
                  class="text-grey ml-2"
                >
                  • {{ formatSpeed(uploadSpeed) }}
                </span>
              </span>
            </div>
            <v-progress-linear
              :model-value="overallProgress"
              color="primary"
              height="6"
              rounded
            />
          </div>

          <div class="upload-list">
            <div
              v-for="album in selectedAlbums"
              :key="album.id"
              class="upload-item"
              :class="[`status-${album.uploadStatus}`]"
            >
              <div class="upload-item-header">
                <div class="upload-cover-status">
                  <!-- Show cover art only after upload completes (the UX flourish) -->
                  <div
                    v-if="album.uploadStatus === 'complete' || album.uploadStatus === 'skipped' || album.uploadStatus === 'error'"
                    class="upload-cover-wrapper"
                  >
                    <img
                      v-if="album.coverArtUrl"
                      :src="album.coverArtUrl"
                      class="upload-cover-thumb"
                      alt=""
                    />
                    <v-icon
                      v-else
                      size="28"
                      color="grey-darken-1"
                    >mdi-album</v-icon>
                  </div>
                  <!-- Show status icon only during pending/uploading -->
                  <div
                    v-else
                    class="upload-status-icon-wrapper"
                  >
                    <v-icon
                      v-if="album.uploadStatus === 'pending'"
                      size="24"
                      color="grey"
                    >mdi-clock-outline</v-icon>
                    <v-progress-circular
                      v-else-if="album.uploadStatus === 'uploading'"
                      :size="24"
                      :width="3"
                      indeterminate
                      color="primary"
                    />
                  </div>
                  <!-- Badge overlay for completed states -->
                  <div
                    v-if="album.uploadStatus === 'complete' || album.uploadStatus === 'skipped' || album.uploadStatus === 'error'"
                    class="upload-status-badge"
                    :class="`status-${album.uploadStatus}`"
                  >
                    <v-icon
                      v-if="album.uploadStatus === 'complete'"
                      size="12"
                      color="white"
                    >mdi-check</v-icon>
                    <v-icon
                      v-else-if="album.uploadStatus === 'skipped'"
                      size="12"
                      color="white"
                    >mdi-skip-next</v-icon>
                    <v-icon
                      v-else-if="album.uploadStatus === 'error'"
                      size="12"
                      color="white"
                    >mdi-alert-circle</v-icon>
                  </div>
                </div>
                <div class="upload-item-info">
                  <div class="upload-item-title">
                    {{ album.artist || 'Unknown' }} - {{ album.album || album.folderName }}
                  </div>
                  <div class="upload-item-meta text-caption text-grey">
                    {{ getRelevantFileCount(album) }} {{ contentTerminology.trackLabel }}
                  </div>
                </div>
                <span class="upload-item-progress">
                  {{ album.uploadProgress || 0 }}%
                </span>
              </div>
              <v-progress-linear
                v-if="album.uploadStatus === 'uploading' || album.uploadStatus === 'complete' || album.uploadStatus === 'skipped'"
                :model-value="album.uploadProgress || 0"
                :color="album.uploadStatus === 'complete' ? 'success' : album.uploadStatus === 'skipped' ? 'warning' : 'primary'"
                height="3"
                rounded
                class="mt-1"
              />
              <div
                v-if="album.uploadError"
                class="text-caption mt-1"
                :class="album.uploadStatus === 'skipped' ? 'text-warning' : 'text-error'"
              >
                {{ album.uploadError }}
              </div>
            </div>
          </div>

          <div
            v-if="uploadComplete"
            class="mt-4"
          >
            <v-alert
              type="success"
              class="mb-2"
            >
              Upload complete! {{ successCount }} {{ successCount === 1 ? contentTerminology.singular : contentTerminology.plural }} uploaded.
              <span v-if="skippedCount > 0">
                {{ skippedCount }} already existed.
              </span>
            </v-alert>
          </div>
        </div>
      </v-card-text>

      <v-card-actions class="px-4 pb-4">
        <v-btn
          v-if="step > 1 && !isUploading && !uploadComplete"
          variant="text"
          @click="step--"
        >
          Back
        </v-btn>
        <v-spacer />
        <v-btn
          variant="text"
          :disabled="isUploading && !uploadComplete"
          @click="closeDialog"
        >
          {{ uploadComplete ? 'Done' : 'Cancel' }}
        </v-btn>
        <v-btn
          v-if="step === 2"
          color="primary"
          :disabled="selectedCount === 0"
          @click="startBulkUpload"
        >
          Upload {{ selectedCount }} {{ selectedCount === 1 ? contentTerminology.singular : contentTerminology.plural }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useContentCategoriesQuery, useBulkAddReleasesMutation } from '/@/plugins/lensService/hooks';
import type { AddInput } from '@riffcc/citadel-sdk';
import { uploadDirectory } from '/@/composables/useArchivist';
import { useIdentity } from '/@/composables/useIdentity';
import {
  parseAlbumFolder,
  detectCategoryFromGenre,
  groupFilesByFolder,
  countRelevantFiles,
  filterJunkFiles,
  type ParsedAlbumMetadata,
  type ParsedTrack,
} from '/@/composables/useMetadataParser';

interface Props {
  modelValue: boolean;
  files: File[];
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
  (e: 'upload:success'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// Identity
const { publicKey } = useIdentity();

// Queries
const { data: contentCategories } = useContentCategoriesQuery();

const bulkAddReleasesMutation = useBulkAddReleasesMutation({
  onSuccess: () => {
    emit('upload:success');
  },
});

// Dialog state
const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

// Step state
const step = ref(1);

// Parsing state
const parseProgress = ref(0);
const parsedCount = ref(0);
const totalFolders = ref(0);
const currentParsingFolder = ref('');

// Album data - extends parsed metadata with upload state
interface AlbumEntry {
  id: string;
  artist: string | null;
  album: string | null;
  year: number | null;
  genre: string | null;
  label: string | null;
  tracks: ParsedTrack[];
  files: File[]; // Original files for upload
  coverArt: Blob | null;
  coverArtUrl: string | null; // Object URL for display
  folderName: string;
  selected: boolean;
  categoryId: string;
  uploadStatus: 'pending' | 'uploading' | 'complete' | 'error' | 'skipped';
  uploadProgress: number;
  uploadError?: string;
  contentCID?: string;
}

const parsedAlbums = ref<AlbumEntry[]>([]);

// Review state
const searchQuery = ref('');
const selectAll = ref(true);

// Upload state
const isUploading = ref(false);
const uploadComplete = ref(false);
const currentUploadIndex = ref(0);

// Speed tracking
const uploadSpeed = ref(0); // bytes per second
const lastSpeedCheck = ref({ time: 0, bytes: 0 });
const speedHistory = ref<number[]>([]); // Rolling average
const totalBytesUploaded = ref(0);

// Computed
const categoryItems = computed(() => {
  if (!contentCategories.value) return [{ title: 'Music', value: 'music' }];
  return contentCategories.value.map(cat => ({
    value: cat.id,
    title: cat.displayName || cat.name,
  }));
});

const defaultCategoryId = computed(() => {
  // Default to music if available, otherwise first category
  const musicCat = contentCategories.value?.find(c => c.id === 'music');
  return musicCat?.id || contentCategories.value?.[0]?.id || 'music';
});

const filteredAlbums = computed(() => {
  if (!searchQuery.value) return parsedAlbums.value;
  const q = searchQuery.value.toLowerCase();
  return parsedAlbums.value.filter(a =>
    a.artist?.toLowerCase().includes(q) ||
    a.album?.toLowerCase().includes(q) ||
    a.folderName.toLowerCase().includes(q)
  );
});

const selectedAlbums = computed(() =>
  parsedAlbums.value.filter(a => a.selected)
);

const selectedCount = computed(() => selectedAlbums.value.length);

const overallProgress = computed(() => {
  if (selectedAlbums.value.length === 0) return 0;
  const total = selectedAlbums.value.reduce((sum, a) => sum + (a.uploadProgress || 0), 0);
  return Math.round(total / selectedAlbums.value.length);
});

const successCount = computed(() =>
  selectedAlbums.value.filter(a => a.uploadStatus === 'complete').length
);

const skippedCount = computed(() =>
  selectedAlbums.value.filter(a => a.uploadStatus === 'skipped').length
);

// Adaptive terminology based on content type
const contentTerminology = computed(() => {
  const categories = new Set(parsedAlbums.value.map(a => a.categoryId));

  // If all same category, use specific terms
  if (categories.size === 1) {
    const cat = [...categories][0];
    if (cat === 'music' || cat?.includes('music')) {
      return { singular: 'album', plural: 'albums', trackLabel: 'tracks' };
    }
    if (cat === 'audiobooks' || cat?.includes('audiobook')) {
      return { singular: 'audiobook', plural: 'audiobooks', trackLabel: 'chapters' };
    }
    if (cat === 'movies' || cat?.includes('movie')) {
      return { singular: 'movie', plural: 'movies', trackLabel: 'files' };
    }
    if (cat === 'tv-shows' || cat?.includes('tv')) {
      return { singular: 'episode', plural: 'episodes', trackLabel: 'files' };
    }
    if (cat === 'podcasts' || cat?.includes('podcast')) {
      return { singular: 'podcast', plural: 'podcasts', trackLabel: 'episodes' };
    }
  }

  // Mixed or unknown - use generic terms
  return { singular: 'item', plural: 'items', trackLabel: 'files' };
});

// Methods
function getRelevantFileCount(album: AlbumEntry): number {
  return countRelevantFiles(album.files, album.categoryId);
}

function formatSpeed(bytesPerSecond: number): string {
  if (bytesPerSecond === 0) return '0 KiB/s';
  const k = 1024;
  if (bytesPerSecond < k) {
    return `${Math.round(bytesPerSecond)} B/s`;
  } else if (bytesPerSecond < k * k) {
    return `${(bytesPerSecond / k).toFixed(1)} KiB/s`;
  } else if (bytesPerSecond < k * k * k) {
    return `${(bytesPerSecond / (k * k)).toFixed(1)} MiB/s`;
  } else {
    return `${(bytesPerSecond / (k * k * k)).toFixed(2)} GiB/s`;
  }
}

function updateSpeed(loadedBytes: number) {
  const now = Date.now();
  const timeDelta = (now - lastSpeedCheck.value.time) / 1000; // seconds

  if (lastSpeedCheck.value.time === 0 || timeDelta < 0.2) {
    // First update or too soon
    if (lastSpeedCheck.value.time === 0) {
      lastSpeedCheck.value = { time: now, bytes: loadedBytes };
    }
    return;
  }

  const bytesDelta = loadedBytes - lastSpeedCheck.value.bytes;
  const instantSpeed = bytesDelta / timeDelta;

  // Rolling average of last 5 samples for smoothing
  speedHistory.value.push(instantSpeed);
  if (speedHistory.value.length > 5) {
    speedHistory.value.shift();
  }

  const avgSpeed = speedHistory.value.reduce((a, b) => a + b, 0) / speedHistory.value.length;
  uploadSpeed.value = Math.max(0, avgSpeed);

  lastSpeedCheck.value = { time: now, bytes: loadedBytes };
}

function toggleSelectAll() {
  parsedAlbums.value.forEach(a => {
    a.selected = selectAll.value;
  });
}

function autoDetectCategories() {
  parsedAlbums.value.forEach(album => {
    if (album.genre) {
      const detected = detectCategoryFromGenre(album.genre);
      album.categoryId = detected || defaultCategoryId.value;
    }
  });
}

async function parseFiles() {
  const groupedFiles = groupFilesByFolder(props.files);
  totalFolders.value = groupedFiles.size;
  parsedCount.value = 0;

  const defaultCategory = defaultCategoryId.value;

  // Parse each folder
  const results: AlbumEntry[] = [];
  let index = 0;

  for (const [folderName, rawFiles] of groupedFiles) {
    currentParsingFolder.value = folderName;

    // Filter out junk files (.DS_Store, .AppleDouble, etc.)
    const files = filterJunkFiles(rawFiles);

    try {
      const parsed = await parseAlbumFolder(files, folderName);
      // Create object URL for cover art display
      const coverArtUrl = parsed.coverArt ? URL.createObjectURL(parsed.coverArt) : null;
      results.push({
        id: `album-${index++}`,
        artist: parsed.artist,
        album: parsed.album,
        year: parsed.year,
        genre: parsed.genre,
        label: parsed.label,
        tracks: parsed.tracks,
        files: files,
        coverArt: parsed.coverArt,
        coverArtUrl,
        folderName: parsed.folderName,
        selected: true,
        categoryId: detectCategoryFromGenre(parsed.genre || '') || defaultCategory,
        uploadStatus: 'pending',
        uploadProgress: 0,
      });
    } catch (err) {
      console.error(`Failed to parse folder ${folderName}:`, err);
      // Still add the folder with minimal info
      results.push({
        id: `album-${index++}`,
        artist: null,
        album: null,
        year: null,
        genre: null,
        label: null,
        tracks: [],
        files: files,
        coverArt: null,
        coverArtUrl: null,
        folderName,
        selected: true,
        categoryId: defaultCategory,
        uploadStatus: 'pending',
        uploadProgress: 0,
      });
    }

    parsedCount.value++;
    parseProgress.value = Math.round((parsedCount.value / totalFolders.value) * 100);
  }

  parsedAlbums.value = results;
  currentParsingFolder.value = '';

  // Move to review step
  step.value = 2;
}

async function startBulkUpload() {
  step.value = 3;
  isUploading.value = true;
  currentUploadIndex.value = 0;

  // Reset speed tracking
  uploadSpeed.value = 0;
  lastSpeedCheck.value = { time: 0, bytes: 0 };
  speedHistory.value = [];
  totalBytesUploaded.value = 0;

  const albumsToUpload = selectedAlbums.value;
  let cumulativeBytes = 0;

  // Upload each album sequentially
  for (const album of albumsToUpload) {
    album.uploadStatus = 'uploading';
    album.uploadProgress = 0;

    const albumTotalBytes = album.files.reduce((sum, f) => sum + f.size, 0);
    const albumStartBytes = cumulativeBytes;

    try {
      // Use the original files stored in the album entry
      const files = album.files;

      // Upload to Archivist
      const uploadResult = await uploadDirectory(files, {
        publicKey: publicKey.value,
        concurrency: 4,
        onProgress: (progress) => {
          album.uploadProgress = progress.percent;
          // Update speed tracking with cumulative bytes
          const currentAlbumBytes = (progress.percent / 100) * albumTotalBytes;
          totalBytesUploaded.value = albumStartBytes + currentAlbumBytes;
          updateSpeed(totalBytesUploaded.value);
        },
      });

      if (!uploadResult.success || !uploadResult.cid) {
        throw new Error(uploadResult.error || 'Upload failed');
      }

      album.contentCID = uploadResult.cid;
      album.uploadProgress = 100;
      album.uploadStatus = 'complete';
      cumulativeBytes += albumTotalBytes;

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Upload failed';
      console.error(`Failed to upload album ${album.folderName}:`, err);

      // Check for duplicate CID error
      if (errorMessage.toLowerCase().includes('already') ||
          errorMessage.toLowerCase().includes('duplicate') ||
          errorMessage.toLowerCase().includes('exists')) {
        album.uploadStatus = 'skipped';
        album.uploadError = 'Already uploaded';
        album.uploadProgress = 100;
        cumulativeBytes += albumTotalBytes;
        // Continue to next album - don't treat as error
      } else {
        album.uploadStatus = 'error';
        album.uploadError = errorMessage;
      }
    }

    currentUploadIndex.value++;
  }

  // Now create all releases via bulk mutation (only for newly uploaded, not skipped)
  const releasesToCreate: AddInput[] = albumsToUpload
    .filter(a => a.uploadStatus === 'complete' && a.contentCID)
    .map(album => ({
      name: album.album || album.folderName,
      categoryId: album.categoryId,
      contentCID: album.contentCID!,
      metadata: {
        artist: album.artist,
        year: album.year,
        genre: album.genre,
        label: album.label,
        trackCount: getRelevantFileCount(album),
      },
    }));

  if (releasesToCreate.length > 0) {
    try {
      await bulkAddReleasesMutation.mutateAsync(releasesToCreate);
    } catch (err) {
      console.error('Failed to create releases:', err);
    }
  }

  isUploading.value = false;
  uploadComplete.value = true;
}

function closeDialog() {
  dialogOpen.value = false;
}

function resetState() {
  // Clean up cover art object URLs before clearing
  parsedAlbums.value.forEach(album => {
    if (album.coverArtUrl) {
      URL.revokeObjectURL(album.coverArtUrl);
    }
  });

  step.value = 1;
  parseProgress.value = 0;
  parsedCount.value = 0;
  totalFolders.value = 0;
  currentParsingFolder.value = '';
  parsedAlbums.value = [];
  searchQuery.value = '';
  selectAll.value = true;
  isUploading.value = false;
  uploadComplete.value = false;
  currentUploadIndex.value = 0;

  // Reset speed tracking
  uploadSpeed.value = 0;
  lastSpeedCheck.value = { time: 0, bytes: 0 };
  speedHistory.value = [];
  totalBytesUploaded.value = 0;
}

// Start parsing when dialog opens
watch(dialogOpen, (open) => {
  if (open && props.files.length > 0) {
    resetState();
    parseFiles();
  }
});

// Reset when dialog closes
watch(dialogOpen, (open) => {
  if (!open) {
    setTimeout(resetState, 300);
  }
});
</script>

<style scoped>
.step-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.step {
  display: flex;
  align-items: center;
  gap: 6px;
  opacity: 0.5;
}

.step.active {
  opacity: 1;
}

.step.completed {
  opacity: 0.8;
}

.step-number {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
}

.step.active .step-number {
  background: rgba(138, 43, 226, 0.8);
}

.step.completed .step-number {
  background: rgba(138, 43, 226, 0.5);
}

.step-label {
  font-size: 12px;
}

.step-divider {
  width: 24px;
  height: 1px;
  background: rgba(255, 255, 255, 0.2);
}

.parsing-progress {
  padding: 32px;
  text-align: center;
}

.albums-table-wrapper {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
}

.albums-table {
  font-size: 13px;
}

.album-row--selected {
  background: rgba(138, 43, 226, 0.05);
}

.album-cell {
  display: flex;
  align-items: center;
}

.album-cover-cell {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
  overflow: hidden;
}

.album-cover-thumb {
  width: 32px;
  height: 32px;
  object-fit: cover;
}

.upload-list {
  max-height: 350px;
  overflow-y: auto;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.2);
}

.upload-item {
  padding: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.upload-item:last-child {
  border-bottom: none;
}

.upload-item.status-uploading {
  background: rgba(138, 43, 226, 0.05);
}

.upload-item.status-complete {
  opacity: 0.7;
}

.upload-item.status-error {
  background: rgba(255, 82, 82, 0.1);
}

.upload-item.status-skipped {
  background: rgba(255, 193, 7, 0.08);
  opacity: 0.8;
}

.upload-item-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.upload-cover-status {
  position: relative;
  flex-shrink: 0;
  width: 40px;
  height: 40px;
}

.upload-cover-wrapper {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 4px;
  overflow: hidden;
}

.upload-status-icon-wrapper {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.upload-cover-thumb {
  width: 40px;
  height: 40px;
  object-fit: cover;
}

.upload-status-badge {
  position: absolute;
  bottom: -4px;
  right: -4px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(100, 100, 100, 0.9);
  border: 2px solid rgba(30, 30, 30, 1);
}

.upload-status-badge.status-pending {
  background: rgba(100, 100, 100, 0.9);
}

.upload-status-badge.status-uploading {
  background: rgba(138, 43, 226, 0.9);
}

.upload-status-badge.status-complete {
  background: rgba(76, 175, 80, 0.95);
}

.upload-status-badge.status-skipped {
  background: rgba(255, 193, 7, 0.95);
}

.upload-status-badge.status-error {
  background: rgba(244, 67, 54, 0.95);
}

.upload-item-info {
  flex: 1;
  min-width: 0;
}

.upload-item-title {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.upload-item-meta {
  margin-top: 2px;
}

.upload-item-progress {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  font-family: monospace;
}

.overall-progress {
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
}
</style>
