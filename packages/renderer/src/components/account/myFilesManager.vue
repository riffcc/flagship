<template>
  <v-card>
    <v-card-title class="d-flex align-center">
      <v-icon left>$folder</v-icon>
      My Files
      <v-spacer></v-spacer>
      <v-btn
        icon="$refresh"
        variant="text"
        @click="loadFolder"
        :loading="loading"
      ></v-btn>
    </v-card-title>

    <v-card-text>
      <!-- Breadcrumbs -->
      <v-breadcrumbs v-if="breadcrumbs.length > 0" :items="breadcrumbItems" class="px-0">
        <template #item="{ item }">
          <v-breadcrumbs-item
            :title="item.title"
            @click="navigateToFolder(item.value)"
            :disabled="item.disabled"
            @dragover="!item.disabled ? onDragOver(item.value, $event) : null"
            @dragleave="onDragLeave()"
            @drop="!item.disabled ? onDrop(item.value, $event) : null"
            :class="{ 'bg-blue-lighten-4': dropTarget === item.value }"
          >
            {{ item.title }}
          </v-breadcrumbs-item>
        </template>
      </v-breadcrumbs>

      <!-- Toolbar -->
      <div class="d-flex mb-4">
        <v-btn
          color="primary"
          prepend-icon="$folderPlus"
          @click="showNewFolderDialog = true"
          size="small"
        >
          New Folder
        </v-btn>
        <v-btn
          color="primary"
          prepend-icon="$fileAdd"
          @click="openAddFileDialog"
          size="small"
          class="ml-2"
        >
          Add Upload
        </v-btn>
      </div>

      <!-- Loading -->
      <v-progress-linear v-if="loading" indeterminate></v-progress-linear>

      <!-- Folder/File List -->
      <v-list v-if="!loading">
        <!-- Folders -->
        <v-list-item
          v-for="folder in folders"
          :key="folder.id"
          @click="navigateToFolder(folder.id)"
          prepend-icon="$folder"
          :title="folder.name"
          :subtitle="`Modified ${formatDate(folder.modified_at)}`"
          draggable="true"
          @dragstart="onDragStart('folder', folder.id, $event)"
          @dragover="onDragOver(folder.id, $event)"
          @dragleave="onDragLeave()"
          @drop="onDrop(folder.id, $event)"
          @dragend="onDragEnd()"
          :class="{ 'bg-blue-lighten-5': dropTarget === folder.id }"
        >
          <template #append>
            <v-btn
              icon="$delete"
              variant="text"
              size="small"
              @click.stop="deleteFolder(folder)"
            ></v-btn>
          </template>
        </v-list-item>

        <!-- Files -->
        <v-list-item
          v-for="file in files"
          :key="file.id"
          prepend-icon="$file"
          :title="file.name"
          :subtitle="`${formatBytes(file.size_bytes)} - ${formatDate(file.modified_at)}`"
          draggable="true"
          @dragstart="onDragStart('file', file.id, $event)"
          @dragend="onDragEnd()"
        >
          <template #append>
            <v-btn
              icon="$contentCopy"
              variant="text"
              size="small"
              @click.stop="copyContentId(file)"
              title="Copy Content ID"
            ></v-btn>
            <v-btn
              icon="$delete"
              variant="text"
              size="small"
              @click.stop="deleteFile(file)"
            ></v-btn>
          </template>
        </v-list-item>

        <!-- Empty state -->
        <v-list-item v-if="folders.length === 0 && files.length === 0">
          <v-list-item-title class="text-center text-grey">
            This folder is empty
          </v-list-item-title>
        </v-list-item>
      </v-list>
    </v-card-text>

    <!-- New Folder Dialog -->
    <v-dialog v-model="showNewFolderDialog" max-width="400px">
      <v-card>
        <v-card-title>Create New Folder</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="newFolderName"
            label="Folder Name"
            autofocus
            @keyup.enter="createFolder"
          ></v-text-field>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn @click="showNewFolderDialog = false">Cancel</v-btn>
          <v-btn color="primary" @click="createFolder" :loading="creating">Create</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Add File Dialog -->
    <v-dialog v-model="showAddFileDialog" max-width="600px">
      <v-card>
        <v-card-title>Add Upload to Folder</v-card-title>
        <v-card-text>
          <p class="text-caption mb-4">Select an approved upload to add to this folder:</p>

          <v-progress-linear v-if="loadingUploads" indeterminate class="mb-4"></v-progress-linear>

          <v-list v-if="!loadingUploads && approvedUploads.length > 0" class="mb-4" max-height="400" style="overflow-y: auto;">
            <v-list-item
              v-for="upload in approvedUploads"
              :key="upload.upload_id"
              @click="uploadId = upload.upload_id"
              :active="uploadId === upload.upload_id"
              :title="upload.filename"
              :subtitle="`${formatBytes(upload.size_bytes)} • ${formatDate(upload.timestamp)}`"
            >
              <template #prepend>
                <v-icon>$file</v-icon>
              </template>
            </v-list-item>
          </v-list>

          <v-alert v-if="!loadingUploads && approvedUploads.length === 0" type="info" variant="tonal">
            No approved uploads found. Upload files first from your account menu.
          </v-alert>

          <v-text-field
            v-model="uploadId"
            label="Upload ID"
            hint="Or enter an upload ID manually"
            persistent-hint
            class="mt-4"
          ></v-text-field>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn @click="showAddFileDialog = false">Cancel</v-btn>
          <v-btn color="primary" @click="addFile" :loading="adding" :disabled="!uploadId">Add</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-card>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, computed, onMounted } from 'vue';
import { usePublicKeyQuery } from '/@/plugins/lensService/hooks';
import { API_URL } from '/@/plugins/router';

interface VirtualFolder {
  id: string;
  name: string;
  parent_id?: string;
  owner: string;
  created_at: string;
  modified_at: string;
}

interface VirtualFile {
  id: string;
  name: string;
  folder_id: string;
  upload_id: string;
  ipfs_cid: string;
  size_bytes: number;
  mime_type?: string;
  owner: string;
  created_at: string;
  modified_at: string;
}

interface FolderContents {
  folder: VirtualFolder;
  folders: VirtualFolder[];
  files: VirtualFile[];
  breadcrumbs: VirtualFolder[];
}

const { data: publicKey } = usePublicKeyQuery();

const loading = ref(false);
const currentFolder = ref<VirtualFolder | null>(null);
const folders = ref<VirtualFolder[]>([]);
const files = ref<VirtualFile[]>([]);
const breadcrumbs = ref<VirtualFolder[]>([]);

const showNewFolderDialog = ref(false);
const newFolderName = ref('');
const creating = ref(false);

const showAddFileDialog = ref(false);
const uploadId = ref('');
const adding = ref(false);
const approvedUploads = ref<any[]>([]);
const loadingUploads = ref(false);

const draggedItem = ref<{type: 'folder' | 'file', id: string} | null>(null);
const dropTarget = ref<string | null>(null);

const breadcrumbItems = computed(() => {
  return breadcrumbs.value.map((folder, index) => ({
    title: folder.name,
    value: folder.id,
    disabled: index === breadcrumbs.value.length - 1,
  }));
});

onMounted(() => {
  loadFolder();
});

async function loadFolder(folderId?: string) {
  if (!publicKey.value) return;

  loading.value = true;
  try {
    const url = folderId
      ? `${API_URL}/myfiles/folder/${folderId}`
      : `${API_URL}/myfiles`;

    const response = await fetch(url, {
      headers: {
        'X-Public-Key': publicKey.value,
      },
    });

    if (!response.ok) throw new Error('Failed to load folder');

    const data: FolderContents = await response.json();
    currentFolder.value = data.folder;
    folders.value = data.folders;
    files.value = data.files;
    breadcrumbs.value = data.breadcrumbs;
  } catch (error) {
    console.error('Failed to load folder:', error);
  } finally {
    loading.value = false;
  }
}

function navigateToFolder(folderId: string) {
  loadFolder(folderId);
}

async function createFolder() {
  if (!publicKey.value || !newFolderName.value.trim()) return;

  creating.value = true;
  try {
    const response = await fetch(`${API_URL}/myfiles/folder`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Public-Key': publicKey.value,
      },
      body: JSON.stringify({
        name: newFolderName.value,
        parent_id: currentFolder.value?.id,
      }),
    });

    if (!response.ok) throw new Error('Failed to create folder');

    showNewFolderDialog.value = false;
    newFolderName.value = '';
    loadFolder(currentFolder.value?.id);
  } catch (error) {
    console.error('Failed to create folder:', error);
    alert('Failed to create folder');
  } finally {
    creating.value = false;
  }
}

async function loadApprovedUploads() {
  if (!publicKey.value) return;

  loadingUploads.value = true;
  try {
    const response = await fetch(`${API_URL}/uploads/my-approved`, {
      headers: {
        'X-Public-Key': publicKey.value,
      },
    });

    if (response.ok) {
      const data = await response.json();
      approvedUploads.value = data.uploads || [];
    }
  } catch (error) {
    console.error('Failed to load uploads:', error);
  } finally {
    loadingUploads.value = false;
  }
}

function openAddFileDialog() {
  showAddFileDialog.value = true;
  loadApprovedUploads();
}

async function addFile() {
  if (!publicKey.value || !uploadId.value.trim()) return;

  adding.value = true;
  try {
    const response = await fetch(`${API_URL}/myfiles/file`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Public-Key': publicKey.value,
      },
      body: JSON.stringify({
        upload_id: uploadId.value,
        folder_id: currentFolder.value?.id,
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to add file');
    }

    showAddFileDialog.value = false;
    uploadId.value = '';
    loadFolder(currentFolder.value?.id);
  } catch (error) {
    console.error('Failed to add file:', error);
    alert(error instanceof Error ? error.message : 'Failed to add file');
  } finally {
    adding.value = false;
  }
}

async function deleteFolder(folder: VirtualFolder) {
  if (!publicKey.value) return;
  if (!confirm(`Delete folder "${folder.name}"? It must be empty.`)) return;

  try {
    const response = await fetch(`${API_URL}/myfiles/folder/${folder.id}`, {
      method: 'DELETE',
      headers: {
        'X-Public-Key': publicKey.value,
      },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to delete folder');
    }

    loadFolder(currentFolder.value?.id);
  } catch (error) {
    console.error('Failed to delete folder:', error);
    alert(error instanceof Error ? error.message : 'Failed to delete folder');
  }
}

async function deleteFile(file: VirtualFile) {
  if (!publicKey.value) return;
  if (!confirm(`Delete file "${file.name}"?`)) return;

  try {
    const response = await fetch(`${API_URL}/myfiles/file/${file.id}`, {
      method: 'DELETE',
      headers: {
        'X-Public-Key': publicKey.value,
      },
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Failed to delete file');
    }

    loadFolder(currentFolder.value?.id);
  } catch (error) {
    console.error('Failed to delete file:', error);
    alert(error instanceof Error ? error.message : 'Failed to delete file');
  }
}

// Drag and drop handlers
function onDragStart(type: 'folder' | 'file', id: string, event: DragEvent) {
  draggedItem.value = { type, id };
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
  }
}

function onDragOver(folderId: string, event: DragEvent) {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
  dropTarget.value = folderId;
}

function onDragLeave() {
  dropTarget.value = null;
}

async function onDrop(targetFolderId: string, event: DragEvent) {
  event.preventDefault();
  dropTarget.value = null;

  if (!draggedItem.value || !publicKey.value) return;

  const { type, id } = draggedItem.value;

  try {
    if (type === 'folder') {
      // Move folder
      const response = await fetch(`${API_URL}/myfiles/folder/${id}/move`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Public-Key': publicKey.value,
        },
        body: JSON.stringify({
          new_parent_id: targetFolderId,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to move folder');
      }
    } else {
      // Move file
      const response = await fetch(`${API_URL}/myfiles/file/${id}/move`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-Public-Key': publicKey.value,
        },
        body: JSON.stringify({
          new_folder_id: targetFolderId,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to move file');
      }
    }

    // Reload current folder
    loadFolder(currentFolder.value?.id);
  } catch (error) {
    console.error('Failed to move item:', error);
    alert(error instanceof Error ? error.message : 'Failed to move item');
  } finally {
    draggedItem.value = null;
  }
}

function onDragEnd() {
  draggedItem.value = null;
  dropTarget.value = null;
}

function copyContentId(file: VirtualFile) {
  navigator.clipboard.writeText(file.ipfs_cid);
  alert(`Copied Content ID: ${file.ipfs_cid}`);
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}
</script>
