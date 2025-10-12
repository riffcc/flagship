<template>
  <v-container fluid class="pa-4">
    <v-row>
      <v-col cols="12">
        <h1 class="text-h4 mb-4">Upload & File Management</h1>
        <p class="text-body-1 mb-6">
          Upload files, organize them into folders, and create releases from your uploads.
        </p>
      </v-col>
    </v-row>

    <!-- Quick Actions -->
    <v-row class="mb-4">
      <v-col cols="12">
        <v-card>
          <v-card-title>Quick Actions</v-card-title>
          <v-card-text>
            <v-btn
              color="primary"
              size="large"
              prepend-icon="$upload"
              class="mr-2 mb-2"
              @click="showUploadDialog = true"
              :disabled="!hasUploadPermission"
            >
              Upload Files
            </v-btn>
            <v-btn
              color="secondary"
              size="large"
              prepend-icon="$plus"
              class="mr-2 mb-2"
              @click="goToCreateRelease"
              :disabled="!canCreateRelease"
            >
              Create Release
            </v-btn>
            <v-alert
              v-if="!hasUploadPermission"
              type="warning"
              variant="tonal"
              class="mt-4"
            >
              You don't have permission to upload files. Contact an administrator to get upload permissions.
            </v-alert>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- My Files Section -->
    <v-row>
      <v-col cols="12">
        <my-files-manager v-if="hasUploadPermission" />
        <v-card v-else>
          <v-card-text>
            <v-alert type="info" variant="tonal">
              File management requires upload permissions. Contact an administrator.
            </v-alert>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Upload Dialog -->
    <ipfs-upload-dialog
      v-model="showUploadDialog"
      @update:success="handleUploadSuccess"
    />
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
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useAccountStatusQuery } from '/@/plugins/lensService/hooks';
import { useSnackbarMessage } from '/@/composables/snackbarMessage';
import ipfsUploadDialog from '/@/components/account/ipfsUploadDialog.vue';
import myFilesManager from '/@/components/account/myFilesManager.vue';

const router = useRouter();
const { data: accountStatus } = useAccountStatusQuery();
const { snackbarMessage, showSnackbar, openSnackbar, closeSnackbar } = useSnackbarMessage();

const showUploadDialog = ref(false);

const hasUploadPermission = computed(() => {
  if (!accountStatus.value) return false;
  return accountStatus.value.permissions?.includes('upload') || accountStatus.value.isAdmin;
});

const canCreateRelease = computed(() => {
  if (!accountStatus.value) return false;
  return accountStatus.value.permissions?.includes('create_release') || accountStatus.value.isAdmin;
});

function handleUploadSuccess() {
  openSnackbar('Files uploaded successfully!', 'success');
}

function goToCreateRelease() {
  // The existing upload page flow for creating releases
  // In future, could show release form here or navigate to it
  openSnackbar('Navigate to Admin panel to create releases from uploaded files', 'info');
}
</script>
