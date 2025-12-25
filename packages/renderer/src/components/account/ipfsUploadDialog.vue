<template>
  <v-dialog
    v-model="dialogOpen"
    max-width="600px"
    persistent
  >
    <v-card>
      <v-card-title class="text-h5 pa-4 d-flex align-center">
        <img
          src="https://docs.archivist.storage/logos/archivist-terminal.svg"
          alt="Archivist"
          class="mr-2"
          style="width: 24px; height: 24px;"
        />
        Upload
      </v-card-title>

      <v-card-text class="px-4 pb-4">
        <!-- Step indicator -->
        <div class="step-indicator mb-4">
          <div :class="['step', { active: step === 1, completed: step > 1 }]">
            <span class="step-number">1</span>
            <span class="step-label">Select Files</span>
          </div>
          <div class="step-divider" />
          <div :class="['step', { active: step === 2, completed: step > 2 }]">
            <span class="step-number">2</span>
            <span class="step-label">Details</span>
          </div>
          <div class="step-divider" />
          <div :class="['step', { active: step === 3 }]">
            <span class="step-number">3</span>
            <span class="step-label">Upload</span>
          </div>
        </div>

        <!-- Step 1: File Selection -->
        <div v-if="step === 1">
          <!-- Drop zone (clickable + drag-and-drop) -->
          <div
            class="drop-zone"
            :class="{ 'drop-zone--active': isDragging }"
            @drop.prevent="handleDrop"
            @dragover.prevent="isDragging = true"
            @dragleave.prevent="isDragging = false"
            @click="fileInputRef?.click()"
          >
            <v-icon
              size="48"
              color="grey"
            >
              $cloud-upload
            </v-icon>
            <p class="drop-zone-text">
              Drop files or folders here
            </p>
            <p class="drop-zone-hint">
              or click to browse
            </p>
          </div>

          <!-- Hidden file input -->
          <input
            ref="fileInputRef"
            type="file"
            multiple
            style="display: none"
            @change="handleFileInput"
          />

          <!-- Selected files preview -->
          <v-card
            v-if="selectedFiles.length > 0"
            variant="outlined"
            class="mt-4"
          >
            <v-card-text>
              <div class="d-flex justify-space-between align-center mb-2">
                <span class="text-subtitle-2">
                  {{ selectedFiles.length }} file{{ selectedFiles.length !== 1 ? 's' : '' }}
                  <span class="text-caption text-grey ml-1">
                    ({{ formatBytes(totalSize) }})
                  </span>
                </span>
                <v-btn
                  size="small"
                  variant="text"
                  color="error"
                  @click="clearFiles"
                >
                  Clear
                </v-btn>
              </div>
              <v-list
                density="compact"
                max-height="150"
                style="overflow-y: auto;"
              >
                <v-list-item
                  v-for="(file, index) in selectedFiles.slice(0, 10)"
                  :key="index"
                  density="compact"
                >
                  <template #prepend>
                    <v-icon size="small">$file</v-icon>
                  </template>
                  <v-list-item-title class="text-caption">
                    {{ file.webkitRelativePath || file.name }}
                  </v-list-item-title>
                </v-list-item>
                <v-list-item
                  v-if="selectedFiles.length > 10"
                  density="compact"
                >
                  <v-list-item-title class="text-caption text-grey">
                    ...and {{ selectedFiles.length - 10 }} more
                  </v-list-item-title>
                </v-list-item>
              </v-list>
            </v-card-text>
          </v-card>
        </div>

        <!-- Step 2: Category and Metadata -->
        <div v-if="step === 2">
          <v-select
            v-model="selectedCategoryId"
            :items="categoryItems"
            label="Category"
            item-title="title"
            item-value="value"
            :rules="[v => !!v || 'Category is required']"
            class="mb-4"
          />

          <v-text-field
            v-model="releaseTitle"
            :label="isTVCategory ? 'Episode Name' : 'Title'"
            :rules="[v => !!v || 'Title is required']"
            class="mb-2"
          />

          <!-- Music-specific fields -->
          <template v-if="isMusicCategory">
            <v-autocomplete
              v-model="selectedArtistId"
              :items="artistItems"
              :loading="artistsLoading"
              label="Artist"
              placeholder="Search or create..."
              clearable
              class="mb-2"
            />
            <v-text-field
              v-model="albumTitle"
              label="Album/Release Name"
              class="mb-2"
            />
          </template>

          <!-- TV-specific fields -->
          <template v-if="isTVCategory">
            <v-autocomplete
              v-model="selectedSeriesId"
              :items="seriesItems"
              label="TV Series"
              placeholder="Search or create..."
              clearable
              class="mb-2"
            />
            <v-row v-if="selectedSeriesId">
              <v-col cols="6">
                <v-text-field
                  v-model.number="seasonNumber"
                  label="Season"
                  type="number"
                  min="1"
                />
              </v-col>
              <v-col cols="6">
                <v-text-field
                  v-model.number="episodeNumber"
                  label="Episode"
                  type="number"
                  min="1"
                />
              </v-col>
            </v-row>
          </template>

          <!-- Thumbnail (optional) -->
          <v-text-field
            v-model="thumbnailCID"
            label="Thumbnail CID (optional)"
            hint="Leave empty to auto-detect from files"
            persistent-hint
            class="mb-2"
          />
        </div>

        <!-- Step 3: Upload Progress -->
        <div v-if="step === 3">
          <div
            v-if="isUploading || fileUploadStates.length > 0"
            class="upload-progress"
          >
            <!-- Overall progress bar -->
            <div class="overall-progress mb-4">
              <div class="d-flex justify-space-between align-center mb-1">
                <span class="text-caption text-grey">Overall Progress</span>
                <span class="text-caption">{{ uploadProgress }}%</span>
              </div>
              <v-progress-linear
                :model-value="uploadProgress"
                color="primary"
                height="6"
                rounded
              />
            </div>

            <!-- Individual file progress list -->
            <div class="file-progress-list">
              <div
                v-for="fileState in fileUploadStates"
                :key="fileState.id"
                class="file-progress-item"
                :class="[`status-${fileState.status}`]"
              >
                <div class="file-progress-header">
                  <span class="file-status-icon">
                    <v-icon
                      v-if="fileState.status === 'pending'"
                      size="14"
                      color="grey"
                    >$clock-outline</v-icon>
                    <v-progress-circular
                      v-else-if="fileState.status === 'uploading'"
                      :size="14"
                      :width="2"
                      indeterminate
                      color="primary"
                    />
                    <v-icon
                      v-else-if="fileState.status === 'complete'"
                      size="14"
                      color="success"
                    >$check</v-icon>
                    <v-icon
                      v-else-if="fileState.status === 'error'"
                      size="14"
                      color="error"
                    >$alert-circle</v-icon>
                  </span>
                  <span
                    class="file-name"
                    :title="fileState.relativePath"
                  >
                    {{ fileState.fileName }}
                  </span>
                  <span class="file-size">{{ formatBytes(fileState.size) }}</span>
                </div>
                <v-progress-linear
                  v-if="fileState.status === 'uploading' || fileState.status === 'complete'"
                  :model-value="fileState.progress"
                  :color="fileState.status === 'complete' ? 'success' : 'primary'"
                  height="3"
                  rounded
                  class="file-progress-bar"
                />
                <div
                  v-if="fileState.error"
                  class="file-error text-caption text-error"
                >
                  {{ fileState.error }}
                </div>
              </div>
            </div>

            <!-- Upload stats -->
            <div class="upload-stats mt-3 text-caption text-grey">
              <span>
                {{ fileUploadStates.filter(f => f.status === 'complete').length }} / {{ fileUploadStates.length }} files
              </span>
              <span v-if="isUploading"> • Uploading... ({{ formatSpeed(uploadSpeed) }})</span>
            </div>
          </div>

          <div v-if="uploadComplete">
            <v-alert
              type="success"
              class="mb-4"
            >
              Upload complete!
            </v-alert>
            <div
              v-if="contentCID"
              class="cid-result"
            >
              <p class="text-subtitle-2 mb-2">Content CID:</p>
              <div class="cid-display">
                <img
                  src="https://docs.archivist.storage/logos/archivist-terminal.svg"
                  alt=""
                  class="archivist-inline-icon"
                />
                <code>{{ contentCID }}</code>
                <v-btn
                  icon="$contentCopy"
                  size="x-small"
                  variant="text"
                  @click="copyToClipboard(contentCID)"
                />
              </div>
            </div>
          </div>

          <div v-else-if="uploadError">
            <v-alert
              type="error"
              class="mb-4"
            >
              {{ uploadError }}
            </v-alert>
          </div>
        </div>
      </v-card-text>

      <v-card-actions class="px-4 pb-4">
        <v-btn
          v-if="step > 1 && !isUploading && !uploadComplete && !uploadError"
          variant="text"
          @click="step--"
        >
          Back
        </v-btn>
        <v-spacer />
        <v-btn
          variant="text"
          :disabled="isUploading"
          @click="closeDialog"
        >
          {{ uploadComplete ? 'Done' : 'Cancel' }}
        </v-btn>
        <v-btn
          v-if="step === 1"
          color="primary"
          :disabled="selectedFiles.length === 0"
          @click="step = 2"
        >
          Next
        </v-btn>
        <v-btn
          v-if="step === 2"
          color="primary"
          :disabled="!canProceedToUpload"
          @click="startUpload"
        >
          Upload
        </v-btn>
        <v-btn
          v-if="uploadError"
          color="primary"
          @click="startUpload"
        >
          Retry
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useAccountStatusQuery, useContentCategoriesQuery, useGetReleasesQuery, useAddReleaseMutation } from '/@/plugins/lensService/hooks';
import { uploadFile, uploadDirectory, type FileUploadState } from '/@/composables/useArchivist';
import { useIdentity } from '/@/composables/useIdentity';

interface Props {
  modelValue: boolean;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
  (e: 'update:success'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// Identity
const { publicKey } = useIdentity();

// Queries
const { data: accountStatus } = useAccountStatusQuery();
const { data: contentCategories } = useContentCategoriesQuery();
const artistsQuery = useGetReleasesQuery();

const addReleaseMutation = useAddReleaseMutation({
  onSuccess: () => {
    emit('update:success');
  },
});

// Dialog state
const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

// Step state
const step = ref(1);

// File selection state
const fileInputRef = ref<HTMLInputElement>();
const selectedFiles = ref<File[]>([]);
const isDragging = ref(false);

// Metadata state
const selectedCategoryId = ref('');
const releaseTitle = ref('');
const thumbnailCID = ref('');

// Music-specific
const selectedArtistId = ref('');
const albumTitle = ref('');

// TV-specific
const selectedSeriesId = ref('');
const seasonNumber = ref(1);
const episodeNumber = ref(1);

// Upload state
const isUploading = ref(false);
const uploadComplete = ref(false);
const uploadError = ref<string | null>(null);
const uploadProgress = ref(0);
const currentFileIndex = ref(0);
const currentFileName = ref('');
const contentCID = ref('');
const fileUploadStates = ref<FileUploadState[]>([]);

// Speed tracking
const uploadSpeed = ref(0); // bytes per second
const lastSpeedCheck = ref({ time: 0, bytes: 0 });
const speedHistory = ref<number[]>([]); // Rolling average

// Computed
const hasUploadPermission = computed(() => {
  if (!accountStatus.value) return false;
  return accountStatus.value.permissions?.includes('upload') || accountStatus.value.isAdmin;
});

const totalSize = computed(() => {
  return selectedFiles.value.reduce((sum, file) => sum + file.size, 0);
});

const categoryItems = computed(() => {
  if (!contentCategories.value) return [];
  return contentCategories.value.map(cat => ({
    value: cat.id,
    title: cat.displayName || cat.name,
  }));
});

const selectedCategory = computed(() => {
  if (!contentCategories.value || !selectedCategoryId.value) return null;
  return contentCategories.value.find(c => c.id === selectedCategoryId.value);
});

const isMusicCategory = computed(() => {
  const cat = selectedCategory.value;
  return cat?.categoryId === 'music' || cat?.displayName === 'Music';
});

const isTVCategory = computed(() => {
  const cat = selectedCategory.value;
  return cat?.categoryId === 'tv-shows' || cat?.displayName === 'TV Shows';
});

const artistItems = computed(() => {
  if (!artistsQuery.data.value) return [];
  return artistsQuery.data.value
    .filter((r: any) => r.metadata?.type === 'artist')
    .map((r: any) => ({
      value: r.id,
      title: r.name,
    }));
});

const artistsLoading = computed(() => artistsQuery.isLoading.value);

const seriesItems = computed(() => {
  // TODO: Fetch from structures query
  return [];
});

const canProceedToUpload = computed(() => {
  return selectedCategoryId.value && releaseTitle.value;
});

// Methods
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

function formatSpeed(bytesPerSecond: number): string {
  if (bytesPerSecond === 0) return '0 KiB/s';
  const k = 1024;
  if (bytesPerSecond < k) {
    return bytesPerSecond.toFixed(0) + ' B/s';
  } else if (bytesPerSecond < k * k) {
    return (bytesPerSecond / k).toFixed(1) + ' KiB/s';
  } else if (bytesPerSecond < k * k * k) {
    return (bytesPerSecond / (k * k)).toFixed(2) + ' MiB/s';
  } else {
    return (bytesPerSecond / (k * k * k)).toFixed(2) + ' GiB/s';
  }
}

function updateUploadSpeed(totalLoaded: number) {
  const now = Date.now();
  const last = lastSpeedCheck.value;

  // Initialize on first call
  if (last.time === 0) {
    lastSpeedCheck.value = { time: now, bytes: totalLoaded };
    return;
  }

  const timeDelta = (now - last.time) / 1000; // seconds
  const bytesDelta = totalLoaded - last.bytes;

  // Update every 200ms minimum to avoid jitter
  if (timeDelta >= 0.2) {
    const instantSpeed = bytesDelta / timeDelta;

    // Rolling average of last 5 samples for smoothing
    speedHistory.value.push(instantSpeed);
    if (speedHistory.value.length > 5) {
      speedHistory.value.shift();
    }

    const avgSpeed = speedHistory.value.reduce((a, b) => a + b, 0) / speedHistory.value.length;
    uploadSpeed.value = Math.max(0, avgSpeed);

    lastSpeedCheck.value = { time: now, bytes: totalLoaded };
  }
}

function handleFileInput(event: Event) {
  const input = event.target as HTMLInputElement;
  if (input.files) {
    selectedFiles.value = Array.from(input.files);
  }
  input.value = '';
}

async function handleDrop(event: DragEvent) {
  isDragging.value = false;

  const items = event.dataTransfer?.items;
  if (!items) return;

  const files: File[] = [];
  let topLevelFolderName: string | null = null;

  for (const item of Array.from(items)) {
    if (item.kind === 'file') {
      const entry = item.webkitGetAsEntry?.();
      if (entry) {
        if (entry.isDirectory) {
          // Capture the top-level folder name for auto-filling title
          if (!topLevelFolderName) {
            topLevelFolderName = entry.name;
          }
          await traverseDirectory(entry as FileSystemDirectoryEntry, files, entry.name + '/');
        } else {
          const file = item.getAsFile();
          if (file) files.push(file);
        }
      } else {
        const file = item.getAsFile();
        if (file) files.push(file);
      }
    }
  }

  selectedFiles.value = files;

  // Auto-fill title from folder name if not already set
  if (topLevelFolderName && !releaseTitle.value) {
    releaseTitle.value = topLevelFolderName;
  }
}

async function traverseDirectory(
  directory: FileSystemDirectoryEntry,
  files: File[],
  path = '',
): Promise<void> {
  return new Promise((resolve) => {
    const reader = directory.createReader();

    const readEntries = () => {
      reader.readEntries(async (entries) => {
        if (entries.length === 0) {
          resolve();
          return;
        }

        for (const entry of entries) {
          if (entry.isDirectory) {
            await traverseDirectory(
              entry as FileSystemDirectoryEntry,
              files,
              path + entry.name + '/',
            );
          } else {
            const fileEntry = entry as FileSystemFileEntry;
            const file = await new Promise<File>((res) => {
              fileEntry.file((f) => {
                // Preserve the relative path
                Object.defineProperty(f, 'webkitRelativePath', {
                  value: path + f.name,
                  writable: false,
                });
                res(f);
              });
            });
            files.push(file);
          }
        }

        // Continue reading if there are more entries
        readEntries();
      });
    };

    readEntries();
  });
}

function clearFiles() {
  selectedFiles.value = [];
}

async function handlePaste(event: ClipboardEvent) {
  // Only handle paste when dialog is open and on step 1
  if (!dialogOpen.value || step.value !== 1) return;

  const items = event.clipboardData?.items;
  if (!items) return;

  const files: File[] = [];
  let topLevelFolderName: string | null = null;

  for (const item of Array.from(items)) {
    if (item.kind === 'file') {
      const entry = (item as any).webkitGetAsEntry?.();
      if (entry) {
        if (entry.isDirectory) {
          if (!topLevelFolderName) {
            topLevelFolderName = entry.name;
          }
          await traverseDirectory(entry as FileSystemDirectoryEntry, files, entry.name + '/');
        } else {
          const file = item.getAsFile();
          if (file) files.push(file);
        }
      } else {
        const file = item.getAsFile();
        if (file) files.push(file);
      }
    }
  }

  if (files.length > 0) {
    event.preventDefault();
    selectedFiles.value = files;

    if (topLevelFolderName && !releaseTitle.value) {
      releaseTitle.value = topLevelFolderName;
    }
  }
}

// Attach paste listener when dialog is open
onMounted(() => {
  document.addEventListener('paste', handlePaste);
});

onUnmounted(() => {
  document.removeEventListener('paste', handlePaste);
});

async function startUpload() {
  step.value = 3;
  isUploading.value = true;
  uploadError.value = null;
  uploadProgress.value = 0;
  currentFileIndex.value = 0;
  fileUploadStates.value = [];

  // Reset speed tracking
  uploadSpeed.value = 0;
  lastSpeedCheck.value = { time: 0, bytes: 0 };
  speedHistory.value = [];

  try {
    let cid: string;

    if (selectedFiles.value.length === 1) {
      // Single file: upload directly (show as single-file progress)
      const file = selectedFiles.value[0];
      currentFileName.value = file.name;

      // Initialize single file state for UI consistency
      fileUploadStates.value = [{
        id: 'single-file',
        fileName: file.name,
        relativePath: file.name,
        size: file.size,
        status: 'uploading',
        progress: 0,
        loaded: 0,
      }];

      const result = await uploadFile(file, {
        publicKey: publicKey.value,
        onProgress: (progress) => {
          uploadProgress.value = progress.percent;
          fileUploadStates.value = [{
            ...fileUploadStates.value[0],
            progress: progress.percent,
            loaded: progress.loaded,
          }];
          updateUploadSpeed(progress.loaded);
        },
      });

      if (!result.success || !result.cid) {
        throw new Error(result.error || 'Upload failed');
      }

      fileUploadStates.value = [{
        ...fileUploadStates.value[0],
        status: 'complete',
        progress: 100,
        loaded: file.size,
        cid: result.cid,
      }];

      cid = result.cid;
    } else {
      // Multiple files: upload in parallel with individual progress tracking
      const result = await uploadDirectory(selectedFiles.value, {
        publicKey: publicKey.value,
        concurrency: 4,  // Upload 4 files at a time
        onProgress: (progress) => {
          uploadProgress.value = progress.percent;
          updateUploadSpeed(progress.loaded);
        },
        onFileStates: (states) => {
          fileUploadStates.value = states;
        },
      });

      if (!result.success || !result.cid) {
        throw new Error(result.error || 'Upload failed');
      }
      cid = result.cid;
    }

    uploadProgress.value = 100;
    contentCID.value = cid;

    // Build metadata
    const metadata: Record<string, any> = {};
    if (isMusicCategory.value) {
      if (selectedArtistId.value) metadata.artistId = selectedArtistId.value;
      if (albumTitle.value) metadata.albumTitle = albumTitle.value;
    }
    if (isTVCategory.value) {
      if (selectedSeriesId.value) {
        metadata.seriesId = selectedSeriesId.value;
        metadata.seasonNumber = seasonNumber.value;
        metadata.episodeNumber = episodeNumber.value;
      }
    }

    // Create the release
    await addReleaseMutation.mutateAsync({
      name: releaseTitle.value,
      categoryId: selectedCategoryId.value,
      contentCID: contentCID.value,
      thumbnailCID: thumbnailCID.value || undefined,
      metadata,
    });

    uploadComplete.value = true;
    isUploading.value = false;

  } catch (error) {
    console.error('Upload error:', error);
    uploadError.value = error instanceof Error ? error.message : 'Upload failed';
    isUploading.value = false;
  }
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

function closeDialog() {
  dialogOpen.value = false;
}

function resetForm() {
  step.value = 1;
  selectedFiles.value = [];
  selectedCategoryId.value = '';
  releaseTitle.value = '';
  thumbnailCID.value = '';
  selectedArtistId.value = '';
  albumTitle.value = '';
  selectedSeriesId.value = '';
  seasonNumber.value = 1;
  episodeNumber.value = 1;
  isUploading.value = false;
  uploadComplete.value = false;
  uploadError.value = null;
  uploadProgress.value = 0;
  contentCID.value = '';
  fileUploadStates.value = [];
  uploadSpeed.value = 0;
  lastSpeedCheck.value = { time: 0, bytes: 0 };
  speedHistory.value = [];
}

// Reset when dialog closes
watch(dialogOpen, (open) => {
  if (!open) {
    setTimeout(resetForm, 300);
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

.drop-zone {
  border: 2px dashed rgba(255, 255, 255, 0.2);
  border-radius: 8px;
  padding: 48px 24px;
  text-align: center;
  transition: all 0.2s ease;
  cursor: pointer;
}

.drop-zone:hover {
  border-color: rgba(255, 255, 255, 0.4);
  background: rgba(255, 255, 255, 0.02);
}

.drop-zone--active {
  border-color: rgba(138, 43, 226, 0.8);
  background: rgba(138, 43, 226, 0.1);
}

.drop-zone-text {
  margin-top: 12px;
  font-size: 16px;
  color: rgba(255, 255, 255, 0.8);
}

.drop-zone-hint {
  margin-top: 4px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
}

.cid-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 4px;
  font-family: monospace;
}

.cid-display code {
  flex: 1;
  font-size: 12px;
  word-break: break-all;
}

.archivist-inline-icon {
  width: 16px;
  height: 16px;
  opacity: 0.8;
}

/* File progress list styles */
.file-progress-list {
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.2);
}

.file-progress-item {
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  transition: background-color 0.15s ease;
}

.file-progress-item:last-child {
  border-bottom: none;
}

.file-progress-item:hover {
  background: rgba(255, 255, 255, 0.02);
}

.file-progress-item.status-uploading {
  background: rgba(138, 43, 226, 0.05);
}

.file-progress-item.status-complete {
  opacity: 0.7;
}

.file-progress-item.status-error {
  background: rgba(255, 82, 82, 0.1);
}

.file-progress-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.file-status-icon {
  flex-shrink: 0;
  width: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.file-name {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: rgba(255, 255, 255, 0.9);
}

.file-size {
  flex-shrink: 0;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
  font-family: monospace;
}

.file-progress-bar {
  margin-top: 4px;
}

.file-error {
  margin-top: 4px;
  font-size: 11px;
}

.upload-stats {
  text-align: center;
  padding: 8px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
}

.overall-progress {
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
}
</style>
