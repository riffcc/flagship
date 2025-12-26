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
          <!-- Parsing indicator -->
          <v-alert
            v-if="isParsingMetadata"
            type="info"
            density="compact"
            variant="tonal"
            class="mb-4"
          >
            <template #prepend>
              <v-progress-circular size="16" width="2" indeterminate />
            </template>
            Parsing file metadata...
          </v-alert>

          <!-- Detected codec badges -->
          <div v-if="detectedCodec" class="codec-badges mb-3">
            <v-chip
              v-if="isLossless"
              size="small"
              color="success"
              variant="tonal"
              class="mr-2"
            >
              <v-icon start size="small">mdi-quality-high</v-icon>
              Lossless
            </v-chip>
            <v-chip
              size="small"
              :color="isLossless ? 'success' : 'primary'"
              variant="outlined"
            >
              {{ detectedCodec.toUpperCase() }}
            </v-chip>
          </div>

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
            <v-combobox
              v-model="artistName"
              :items="artistItems"
              item-title="title"
              item-value="title"
              :loading="artistsLoading"
              label="Artist"
              placeholder="Type or select artist..."
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

          <!-- Thumbnail section -->
          <div class="thumbnail-section mb-4">
            <div class="d-flex align-center gap-3">
              <!-- Cover art preview -->
              <div
                v-if="extractedCoverArt"
                class="thumbnail-preview"
              >
                <v-img
                  :src="extractedCoverArt.url"
                  width="80"
                  height="80"
                  cover
                  class="rounded"
                />
                <v-chip
                  size="x-small"
                  color="success"
                  variant="tonal"
                  class="thumbnail-badge"
                >
                  Auto-detected
                </v-chip>
              </div>
              <div
                v-else
                class="thumbnail-placeholder"
              >
                <v-icon size="32" color="grey">mdi-image-off</v-icon>
              </div>
              <div class="flex-grow-1">
                <v-text-field
                  v-model="thumbnailCID"
                  label="Thumbnail CID"
                  :hint="extractedCoverArt ? 'Auto-detected from files. Enter CID to override.' : 'Enter a CID or leave empty to use extracted cover art'"
                  persistent-hint
                  density="compact"
                />
              </div>
            </div>
          </div>

          <!-- Moderation queue option (admin only) -->
          <v-checkbox
            v-if="isAdmin"
            v-model="uploadToModerationQueue"
            density="compact"
            class="mb-2"
          >
            <template #label>
              <div>
                <span class="text-body-2">Do not automatically approve</span>
                <p class="text-caption text-grey mt-n1">
                  Upload releases to the moderation queue instead of directly approving them
                </p>
              </div>
            </template>
          </v-checkbox>
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
        <!-- VIEW RELEASE button - shown after successful upload -->
        <!-- Uses router-link via :to prop for middle-click support -->
        <v-btn
          v-if="uploadComplete && createdReleaseId"
          :to="`/release/${createdReleaseId}`"
          color="primary"
          variant="tonal"
          @click="dialogOpen = false"
        >
          View Release
        </v-btn>
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
        <!-- Advanced options dialog on step 2 -->
        <ReleaseAdvancedOptions
          v-if="step === 2"
          v-model="showAdvanced"
          :category-id="selectedCategoryId"
          :metadata-schema="selectedCategoryMetadataSchema"
          :metadata="advancedMetadata"
          :license-type="licenseType"
          :license-version="licenseVersion"
          :license-jurisdiction="licenseJurisdiction"
          :license-attribution="licenseAttribution"
          :custom-license-url="customLicenseUrl"
          @update:metadata="advancedMetadata = $event"
          @update:license-type="licenseType = $event"
          @update:license-version="licenseVersion = $event"
          @update:license-jurisdiction="licenseJurisdiction = $event"
          @update:license-attribution="licenseAttribution = $event"
          @update:custom-license-url="customLicenseUrl = $event"
        >
          <template #activator="{ props: activatorProps }">
            <v-btn
              v-bind="activatorProps"
              variant="outlined"
            >
              Advanced
            </v-btn>
          </template>
        </ReleaseAdvancedOptions>
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
import { useRouter } from 'vue-router';
import { useAccountStatusQuery, useContentCategoriesQuery, useGetReleasesQuery, useAddReleaseMutation } from '/@/plugins/lensService/hooks';
import { uploadFile, uploadDirectory, type FileUploadState } from '/@/composables/useArchivist';
import { useIdentity } from '/@/composables/useIdentity';
import {
  parseAlbumFolder,
  isAudioFile,
  isVideoFile,
  filterJunkFiles,
  detectCategoryFromGenre,
  type ParsedAlbumMetadata,
} from '/@/composables/useMetadataParser';
import ReleaseAdvancedOptions from '/@/components/releases/ReleaseAdvancedOptions.vue';

interface Props {
  modelValue: boolean;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
  (e: 'update:success'): void;
  (e: 'bulk-upload', files: File[]): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// Router
const router = useRouter();

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
const artistName = ref(''); // Allow arbitrary artist names
const albumTitle = ref('');

// TV-specific
const selectedSeriesId = ref<string | null>('');
const seasonNumber = ref(1);
const episodeNumber = ref(1);

// Parsed metadata from files
const parsedMetadata = ref<ParsedAlbumMetadata | null>(null);
const isParsingMetadata = ref(false);
const extractedCoverArt = ref<{ blob: Blob; url: string } | null>(null);

// Moderation options (admin only)
const uploadToModerationQueue = ref(false);

// Advanced options
const showAdvanced = ref(false);
const advancedMetadata = ref<Record<string, any>>({});
const licenseType = ref('');
const licenseVersion = ref('4.0');
const licenseJurisdiction = ref('');
const licenseAttribution = ref('');
const customLicenseUrl = ref('');

// Detected codec info
const detectedCodec = ref<string | null>(null);
const isLossless = ref(false);

// Upload state
const isUploading = ref(false);
const uploadComplete = ref(false);
const uploadError = ref<string | null>(null);
const uploadProgress = ref(0);
const currentFileIndex = ref(0);
const currentFileName = ref('');
const contentCID = ref('');
const fileUploadStates = ref<FileUploadState[]>([]);
const createdReleaseId = ref<string | null>(null);

// Speed tracking
const uploadSpeed = ref(0); // bytes per second
const lastSpeedCheck = ref({ time: 0, bytes: 0 });
const speedHistory = ref<number[]>([]); // Rolling average

// Computed
const hasUploadPermission = computed(() => {
  if (!accountStatus.value) return false;
  return accountStatus.value.permissions?.includes('upload') || accountStatus.value.isAdmin;
});

const isAdmin = computed(() => accountStatus.value?.isAdmin ?? false);

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

const selectedCategoryMetadataSchema = computed(() => {
  if (!contentCategories.value || !selectedCategoryId.value) {
    return null;
  }
  const category = contentCategories.value.find(c => c.id === selectedCategoryId.value);
  if (!category?.metadataSchema) {
    return null;
  }
  // Parse if string
  if (typeof category.metadataSchema === 'string') {
    try {
      return JSON.parse(category.metadataSchema);
    } catch {
      return null;
    }
  }
  return category.metadataSchema;
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
  const topLevelFolders: string[] = [];

  // First pass: collect all entries before any async work
  // (DataTransferItemList becomes invalid after the event handler returns)
  const entries: FileSystemEntry[] = [];
  for (const item of Array.from(items)) {
    if (item.kind === 'file') {
      const entry = item.webkitGetAsEntry?.();
      if (entry) {
        entries.push(entry);
        if (entry.isDirectory) {
          topLevelFolders.push(entry.name);
        }
      }
    }
  }

  console.log(`[ipfsUploadDialog] Detected ${topLevelFolders.length} top-level folders:`, topLevelFolders);

  // If multiple top-level folders detected, switch to bulk upload mode
  if (topLevelFolders.length >= 2) {
    // Process all entries to get files before emitting
    for (const entry of entries) {
      if (entry.isDirectory) {
        await traverseDirectory(entry as FileSystemDirectoryEntry, files, entry.name + '/');
      } else {
        const fileEntry = entry as FileSystemFileEntry;
        const file = await new Promise<File>((res) => {
          fileEntry.file((f) => res(f));
        });
        files.push(file);
      }
    }
    console.log(`[ipfsUploadDialog] Bulk upload triggered with ${files.length} files from ${topLevelFolders.length} folders`);
    emit('bulk-upload', files);
    dialogOpen.value = false;
    return;
  }

  // Process entries for single-folder or file upload
  for (const entry of entries) {
    if (entry.isDirectory) {
      await traverseDirectory(entry as FileSystemDirectoryEntry, files, entry.name + '/');
    } else {
      const fileEntry = entry as FileSystemFileEntry;
      const file = await new Promise<File>((res) => {
        fileEntry.file((f) => res(f));
      });
      files.push(file);
    }
  }

  selectedFiles.value = files;

  // Auto-fill title from folder name if not already set
  if (topLevelFolders.length > 0 && !releaseTitle.value) {
    releaseTitle.value = topLevelFolders[0];
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
  parsedMetadata.value = null;
  if (extractedCoverArt.value?.url) {
    URL.revokeObjectURL(extractedCoverArt.value.url);
  }
  extractedCoverArt.value = null;
  detectedCodec.value = null;
  isLossless.value = false;
}

/**
 * Auto-detect category from file types
 */
function detectCategoryFromFiles(files: File[]): string | null {
  const cleanFiles = filterJunkFiles(files);
  let audioCount = 0;
  let videoCount = 0;

  for (const file of cleanFiles) {
    if (isAudioFile(file.name)) audioCount++;
    else if (isVideoFile(file.name)) videoCount++;
  }

  // If mostly video, suggest movies/tv
  if (videoCount > 0 && videoCount >= audioCount) {
    return 'movies';
  }

  // If mostly audio, suggest music (genre detection will refine)
  if (audioCount > 0) {
    return 'music';
  }

  return null;
}

/**
 * Parse metadata from selected files
 */
async function parseSelectedFiles() {
  if (selectedFiles.value.length === 0) return;

  const cleanFiles = filterJunkFiles(selectedFiles.value);
  const audioFiles = cleanFiles.filter(f => isAudioFile(f.name));

  if (audioFiles.length === 0) {
    // Not audio files - just detect category from file types
    const detected = detectCategoryFromFiles(cleanFiles);
    if (detected && !selectedCategoryId.value) {
      // Find matching category
      const cat = contentCategories.value?.find(c =>
        c.id === detected || c.name?.toLowerCase() === detected
      );
      if (cat) {
        selectedCategoryId.value = cat.id;
      }
    }
    return;
  }

  // Parse audio metadata
  isParsingMetadata.value = true;
  try {
    const folderName = (selectedFiles.value[0] as any).webkitRelativePath?.split('/')[0] || 'Upload';
    const metadata = await parseAlbumFolder(cleanFiles, folderName);
    parsedMetadata.value = metadata;

    // Auto-fill fields from metadata (prefer metadata over folder name)
    if (metadata.album) {
      releaseTitle.value = metadata.album;  // Always use album name from metadata
      albumTitle.value = metadata.album;
    }
    if (metadata.artist) {
      artistName.value = metadata.artist;
    }

    // Codec detection
    detectedCodec.value = metadata.codec;
    isLossless.value = metadata.lossless;

    // Auto-detect category from genre
    if (!selectedCategoryId.value) {
      let categoryId = detectCategoryFromGenre(metadata.genre);
      if (!categoryId) {
        categoryId = detectCategoryFromFiles(cleanFiles);
      }
      if (categoryId) {
        const cat = contentCategories.value?.find(c =>
          c.id === categoryId || c.name?.toLowerCase() === categoryId
        );
        if (cat) {
          selectedCategoryId.value = cat.id;
        }
      }
    }

    // Extract cover art
    if (metadata.coverArt && metadata.coverArtMimeType) {
      const url = URL.createObjectURL(metadata.coverArt);
      extractedCoverArt.value = { blob: metadata.coverArt, url };
    }

  } catch (err) {
    console.error('Failed to parse metadata:', err);
  } finally {
    isParsingMetadata.value = false;
  }
}

async function handlePaste(event: ClipboardEvent) {
  // Only handle paste when dialog is open and on step 1
  if (!dialogOpen.value || step.value !== 1) return;

  const items = event.clipboardData?.items;
  if (!items) return;

  const files: File[] = [];
  let topLevelFolderCount = 0;
  let topLevelFolderName: string | null = null;

  for (const item of Array.from(items)) {
    if (item.kind === 'file') {
      const entry = (item as any).webkitGetAsEntry?.();
      if (entry) {
        if (entry.isDirectory) {
          topLevelFolderCount++;
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

    // If multiple top-level folders detected, switch to bulk upload mode
    if (topLevelFolderCount >= 2) {
      emit('bulk-upload', files);
      dialogOpen.value = false;
      return;
    }

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
    const metadata: Record<string, any> = {
      ...advancedMetadata.value, // Include advanced metadata fields
    };
    if (isMusicCategory.value) {
      if (artistName.value) metadata.artist = artistName.value;
      if (albumTitle.value) metadata.albumTitle = albumTitle.value;
    }
    if (isTVCategory.value) {
      if (selectedSeriesId.value) {
        metadata.seriesId = selectedSeriesId.value;
        metadata.seasonNumber = seasonNumber.value;
        metadata.episodeNumber = episodeNumber.value;
      }
    }
    // Add codec info to metadata
    if (detectedCodec.value) {
      metadata.codec = detectedCodec.value;
      metadata.lossless = isLossless.value;
    }
    // Add license to metadata if selected
    if (licenseType.value) {
      if (licenseType.value === 'custom') {
        metadata.license = JSON.stringify({
          type: 'custom',
          url: customLicenseUrl.value,
          ...(licenseAttribution.value ? { attribution: licenseAttribution.value } : {}),
        });
      } else {
        metadata.license = JSON.stringify({
          type: licenseType.value,
          version: licenseVersion.value,
          ...(licenseJurisdiction.value ? { jurisdiction: licenseJurisdiction.value } : {}),
          ...(licenseAttribution.value ? { attribution: licenseAttribution.value } : {}),
        });
      }
    }

    // Add track metadata from parsed ID3/FLAC tags
    console.log('[ipfsUploadDialog] parsedMetadata.value:', parsedMetadata.value);
    console.log('[ipfsUploadDialog] parsedMetadata.value?.tracks:', parsedMetadata.value?.tracks);
    if (parsedMetadata.value?.tracks && parsedMetadata.value.tracks.length > 0) {
      const trackData = parsedMetadata.value.tracks.map(track => ({
        title: track.title || track.fileName.replace(/\.[^.]+$/, ''),
        artist: track.artist || parsedMetadata.value?.artist || null,
        ...(track.duration ? { duration: track.duration } : {}),
        ...(track.trackNumber ? { trackNumber: track.trackNumber } : {}),
      }));
      metadata.trackMetadata = JSON.stringify(trackData);
      console.log('[ipfsUploadDialog] Pre-populated trackMetadata:', trackData);
    } else {
      console.warn('[ipfsUploadDialog] No tracks found in parsedMetadata - trackMetadata will not be set');
    }

    // Upload cover art if extracted and no thumbnail CID provided
    let finalThumbnailCID = thumbnailCID.value;
    if (!finalThumbnailCID && extractedCoverArt.value) {
      try {
        const coverFile = new File(
          [extractedCoverArt.value.blob],
          'cover.jpg',
          { type: extractedCoverArt.value.blob.type || 'image/jpeg' }
        );
        const coverResult = await uploadFile(coverFile, {
          publicKey: publicKey.value,
        });
        if (coverResult.success && coverResult.cid) {
          finalThumbnailCID = coverResult.cid;
        }
      } catch (err) {
        console.warn('Failed to upload cover art:', err);
        // Continue without thumbnail
      }
    }

    // Create the release
    const releaseData = {
      name: releaseTitle.value,
      categoryId: selectedCategoryId.value,
      contentCID: cid, // Use the cid variable directly, not contentCID.value
      thumbnailCID: finalThumbnailCID || undefined,
      metadata,
      // If admin chose to upload to moderation queue, set status to pending
      ...(uploadToModerationQueue.value ? { status: 'pending' as const } : {}),
    };

    console.log('[ipfsUploadDialog] Creating release with data:', releaseData);
    console.log('[ipfsUploadDialog] CID value:', cid);
    console.log('[ipfsUploadDialog] CID type:', typeof cid);
    console.log('[ipfsUploadDialog] contentCID.value:', contentCID.value);

    if (!cid) {
      throw new Error('BUG: CID is empty or undefined before creating release!');
    }

    const result = await addReleaseMutation.mutateAsync(releaseData);
    console.log('[ipfsUploadDialog] Release created:', result);

    // Store the release ID for the VIEW RELEASE button
    // API might return 'hash' or 'id' depending on implementation
    const releaseId = result?.hash || result?.id;
    if (releaseId) {
      createdReleaseId.value = releaseId;
      console.log('[ipfsUploadDialog] Release ID stored:', releaseId);
    } else {
      console.warn('[ipfsUploadDialog] No release ID in response:', result);
    }

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

function navigateToRelease() {
  if (createdReleaseId.value) {
    dialogOpen.value = false;
    router.push(`/release/${createdReleaseId.value}`);
  }
}

function resetForm() {
  step.value = 1;
  selectedFiles.value = [];
  selectedCategoryId.value = '';
  releaseTitle.value = '';
  thumbnailCID.value = '';
  selectedArtistId.value = '';
  artistName.value = '';
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
  createdReleaseId.value = null;
  lastSpeedCheck.value = { time: 0, bytes: 0 };
  speedHistory.value = [];

  // Reset metadata parsing state
  parsedMetadata.value = null;
  isParsingMetadata.value = false;
  if (extractedCoverArt.value?.url) {
    URL.revokeObjectURL(extractedCoverArt.value.url);
  }
  extractedCoverArt.value = null;
  detectedCodec.value = null;
  isLossless.value = false;
  uploadToModerationQueue.value = false;
  showAdvanced.value = false;
  advancedMetadata.value = {};
  licenseType.value = '';
  licenseVersion.value = '4.0';
  licenseJurisdiction.value = '';
  licenseAttribution.value = '';
  customLicenseUrl.value = '';
}

// Reset when dialog closes
watch(dialogOpen, (open) => {
  if (!open) {
    setTimeout(resetForm, 300);
  }
});

// Parse metadata when moving to step 2
watch(step, async (newStep, oldStep) => {
  if (newStep === 2 && oldStep === 1) {
    await parseSelectedFiles();
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

/* Thumbnail section */
.thumbnail-section {
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
}

.thumbnail-preview {
  position: relative;
  flex-shrink: 0;
}

.thumbnail-badge {
  position: absolute;
  bottom: -6px;
  left: 50%;
  transform: translateX(-50%);
  font-size: 9px !important;
}

.thumbnail-placeholder {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  flex-shrink: 0;
}

/* Codec badges */
.codec-badges {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
}
</style>
