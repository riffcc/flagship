<template>
  <v-dialog
    v-model="dialogOpen"
    max-width="700px"
  >
    <v-card>
      <v-card-title class="text-h5 pa-4">
        Upload Files
      </v-card-title>

      <v-card-text class="px-4 pb-4">
        <!-- Permission check -->
        <v-alert
          v-if="!hasUploadPermission && accountStatus"
          type="warning"
          class="mb-4"
        >
          You don't have permission to upload files. Contact an administrator to get the 'uploader' role.
        </v-alert>

        <!-- Upload form -->
        <v-form
          v-if="hasUploadPermission"
          ref="formRef"
          @submit.prevent="handleUpload"
        >
          <!-- Native file/folder picker buttons -->
          <div class="mb-4">
            <v-btn
              color="primary"
              variant="elevated"
              prepend-icon="$file"
              class="mr-2 mb-2"
              @click="openFilePicker"
              :disabled="uploading"
            >
              Select Files
            </v-btn>
            <v-btn
              color="primary"
              variant="elevated"
              prepend-icon="$folder"
              class="mr-2 mb-2"
              @click="openFolderPicker"
              :disabled="uploading"
            >
              Select Folder
            </v-btn>
            <input
              ref="fileInputRef"
              type="file"
              multiple
              style="display: none"
              @change="handleFileSelection"
            />
            <input
              ref="folderInputRef"
              type="file"
              webkitdirectory
              directory
              multiple
              style="display: none"
              @change="handleFolderSelection"
            />
          </div>

          <!-- Selected files preview -->
          <v-card
            v-if="selectedFiles.length > 0"
            variant="outlined"
            class="mb-4"
          >
            <v-card-text>
              <div class="text-subtitle-2 mb-2">
                Selected: {{ selectedFiles.length }} {{ selectedFiles.length === 1 ? 'file' : 'files' }}
                <span v-if="totalSize > 0" class="text-caption text-grey">
                  ({{ formatBytes(totalSize) }})
                </span>
              </div>
              <v-list density="compact" max-height="200" style="overflow-y: auto;">
                <v-list-item
                  v-for="(file, index) in selectedFiles"
                  :key="index"
                  density="compact"
                >
                  <template #prepend>
                    <v-icon size="small">$file</v-icon>
                  </template>
                  <v-list-item-title class="text-caption">
                    {{ file.webkitRelativePath || file.name }}
                  </v-list-item-title>
                  <v-list-item-subtitle class="text-caption">
                    {{ formatBytes(file.size) }}
                  </v-list-item-subtitle>
                  <template #append>
                    <v-btn
                      icon="$close"
                      size="x-small"
                      variant="text"
                      @click="removeFile(index)"
                      :disabled="uploading"
                    ></v-btn>
                  </template>
                </v-list-item>
              </v-list>
              <v-btn
                v-if="selectedFiles.length > 0"
                size="small"
                variant="text"
                color="error"
                class="mt-2"
                @click="clearAllFiles"
                :disabled="uploading"
              >
                Clear All
              </v-btn>
            </v-card-text>
          </v-card>

          <v-textarea
            v-model="title"
            label="Title/Collection Name (optional)"
            :disabled="uploading"
            rows="2"
            class="mb-4"
            hint="For multiple files, this becomes the collection name"
          ></v-textarea>

          <v-textarea
            v-model="description"
            label="Description (optional)"
            :disabled="uploading"
            rows="3"
            class="mb-4"
          ></v-textarea>

          <!-- Upload status -->
          <v-card
            v-if="uploading"
            variant="outlined"
            class="mb-4"
          >
            <v-card-text>
              <div class="text-subtitle-2 mb-2">
                Uploading {{ uploadProgress.current }} of {{ uploadProgress.total }} files...
              </div>
              <v-progress-linear
                :model-value="(uploadProgress.current / uploadProgress.total) * 100"
                color="primary"
                height="8"
                class="mb-2"
              ></v-progress-linear>
              <div v-if="uploadProgress.currentFile" class="text-caption text-grey">
                Current: {{ uploadProgress.currentFile }}
              </div>
            </v-card-text>
          </v-card>

          <!-- Success messages -->
          <v-card
            v-if="uploadResults.length > 0"
            variant="outlined"
            class="mb-4"
          >
            <v-card-text>
              <div class="text-subtitle-2 mb-2">
                Upload Results ({{ uploadResults.length }} files)
              </div>
              <v-list density="compact" max-height="250" style="overflow-y: auto;">
                <v-list-item
                  v-for="(result, index) in uploadResults"
                  :key="index"
                  density="compact"
                >
                  <template #prepend>
                    <v-icon
                      :color="result.success ? (result.status === 'approved' ? 'success' : 'info') : 'error'"
                      size="small"
                    >
                      {{ result.success ? '$checkCircle' : '$alertCircle' }}
                    </v-icon>
                  </template>
                  <v-list-item-title class="text-caption">
                    {{ result.fileName }}
                  </v-list-item-title>
                  <v-list-item-subtitle class="text-caption">
                    <span v-if="result.success">
                      {{ result.status === 'approved' ? 'Approved' : 'Pending approval' }}
                      <span v-if="result.ipfs_cid"> - CID: {{ result.ipfs_cid.substring(0, 12) }}...</span>
                    </span>
                    <span v-else class="text-error">
                      {{ result.error }}
                    </span>
                  </v-list-item-subtitle>
                </v-list-item>
              </v-list>
            </v-card-text>
          </v-card>

          <!-- Error message -->
          <v-alert
            v-if="uploadError"
            type="error"
            class="mb-4"
            closable
            @click:close="uploadError = null"
          >
            {{ uploadError }}
          </v-alert>

          <v-card-actions class="px-0">
            <v-spacer></v-spacer>
            <v-btn
              color="grey-darken-1"
              variant="text"
              @click="closeDialog"
              :disabled="uploading"
            >
              {{ uploadResults.length > 0 ? 'Close' : 'Cancel' }}
            </v-btn>
            <v-btn
              v-if="uploadResults.length === 0"
              color="primary"
              variant="elevated"
              type="submit"
              :disabled="selectedFiles.length === 0 || uploading"
              :loading="uploading"
            >
              Upload {{ selectedFiles.length > 0 ? `(${selectedFiles.length})` : '' }}
            </v-btn>
          </v-card-actions>
        </v-form>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useAccountStatusQuery, usePublicKeyQuery } from '/@/plugins/lensService/hooks';
import { useIdentity } from '/@/composables/useIdentity';

interface UploadResult {
  success: boolean;
  upload_id?: string;
  status?: 'pending' | 'approved' | 'rejected';
  ipfs_cid?: string;
  message?: string;
  fileName: string;
  error?: string;
}

interface Props {
  modelValue: boolean;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const { data: accountStatus } = useAccountStatusQuery();
const { data: publicKey } = usePublicKeyQuery();
const { sign } = useIdentity();

const formRef = ref();
const fileInputRef = ref<HTMLInputElement>();
const folderInputRef = ref<HTMLInputElement>();
const selectedFiles = ref<File[]>([]);
const title = ref('');
const description = ref('');
const uploading = ref(false);
const uploadResults = ref<UploadResult[]>([]);
const uploadError = ref<string | null>(null);
const uploadProgress = ref({
  current: 0,
  total: 0,
  currentFile: '',
});

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const hasUploadPermission = computed(() => {
  if (!accountStatus.value) return false;
  return accountStatus.value.permissions.includes('upload');
});

const totalSize = computed(() => {
  return selectedFiles.value.reduce((sum, file) => sum + file.size, 0);
});

// Reset form when dialog closes
watch(dialogOpen, (newValue) => {
  if (!newValue) {
    resetForm();
  }
});

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

function openFilePicker() {
  fileInputRef.value?.click();
}

function openFolderPicker() {
  folderInputRef.value?.click();
}

function handleFileSelection(event: Event) {
  const input = event.target as HTMLInputElement;
  if (input.files) {
    selectedFiles.value.push(...Array.from(input.files));
  }
  // Reset input to allow selecting the same file again
  input.value = '';
}

function handleFolderSelection(event: Event) {
  const input = event.target as HTMLInputElement;
  if (input.files) {
    selectedFiles.value.push(...Array.from(input.files));
  }
  // Reset input to allow selecting the same folder again
  input.value = '';
}

function removeFile(index: number) {
  selectedFiles.value.splice(index, 1);
}

function clearAllFiles() {
  selectedFiles.value = [];
}

function resetForm() {
  selectedFiles.value = [];
  title.value = '';
  description.value = '';
  uploadResults.value = [];
  uploadError.value = null;
  uploadProgress.value = { current: 0, total: 0, currentFile: '' };
}

function closeDialog() {
  dialogOpen.value = false;
}

async function uploadSingleFile(file: File, index: number): Promise<UploadResult> {
  try {
    if (!publicKey.value) {
      throw new Error('Public key not available');
    }

    // Create FormData with file and metadata
    const formData = new FormData();
    formData.append('file', file);

    // Build metadata object
    const metadata: Record<string, any> = {};
    if (title.value) metadata.title = title.value;
    if (description.value) metadata.description = description.value;

    // Include folder path if available
    if (file.webkitRelativePath) {
      metadata.path = file.webkitRelativePath;
      metadata.fileName = file.name;
    }

    // Add file index for batch uploads
    if (selectedFiles.value.length > 1) {
      metadata.batchIndex = index + 1;
      metadata.batchTotal = selectedFiles.value.length;
    }

    if (Object.keys(metadata).length > 0) {
      formData.append('metadata', JSON.stringify(metadata));
    }

    // Create signature payload: timestamp + publicKey + fileName + fileSize
    const timestamp = Date.now();
    const signaturePayload = `${timestamp}:${publicKey.value}:${file.name}:${file.size}`;

    // Sign the payload with user's ed25519 key
    const signature = await sign(signaturePayload);

    // POST to external upload service
    const response = await fetch('https://uploads.global.riff.cc/upload', {
      method: 'POST',
      headers: {
        'X-Public-Key': publicKey.value,
        'X-Signature': signature,
        'X-Timestamp': timestamp.toString(),
      },
      body: formData,
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(errorData.error || `Upload failed: ${response.statusText}`);
    }

    const result = await response.json();
    return {
      success: true,
      fileName: file.webkitRelativePath || file.name,
      upload_id: result.upload_id,
      status: result.status,
      ipfs_cid: result.ipfs_cid,
      message: result.message,
    };

  } catch (error) {
    return {
      success: false,
      fileName: file.webkitRelativePath || file.name,
      error: error instanceof Error ? error.message : 'Upload failed',
    };
  }
}

async function handleUpload() {
  if (selectedFiles.value.length === 0) {
    uploadError.value = 'Please select files or a folder to upload';
    return;
  }

  if (!publicKey.value) {
    uploadError.value = 'Public key not available. Please reconnect.';
    return;
  }

  uploading.value = true;
  uploadError.value = null;
  uploadResults.value = [];
  uploadProgress.value = {
    current: 0,
    total: selectedFiles.value.length,
    currentFile: '',
  };

  try {
    // Upload files one by one
    for (let i = 0; i < selectedFiles.value.length; i++) {
      const file = selectedFiles.value[i];
      uploadProgress.value.current = i + 1;
      uploadProgress.value.currentFile = file.webkitRelativePath || file.name;

      const result = await uploadSingleFile(file, i);
      uploadResults.value.push(result);

      // Small delay between uploads to avoid overwhelming the server
      if (i < selectedFiles.value.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }

    // Check if all uploads failed
    const allFailed = uploadResults.value.every(r => !r.success);
    if (allFailed) {
      uploadError.value = 'All uploads failed. Please check your permissions and try again.';
    }

    // Clear selected files after upload (keep results visible)
    selectedFiles.value = [];

  } catch (error) {
    console.error('Upload error:', error);
    uploadError.value = error instanceof Error ? error.message : 'Upload failed. Please try again.';
  } finally {
    uploading.value = false;
    uploadProgress.value.currentFile = '';
  }
}
</script>
