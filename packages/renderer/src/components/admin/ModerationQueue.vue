<template>
  <div class="moderation-queue">
    <!-- Header with stats -->
    <div class="queue-header">
      <div class="queue-title">
        <h2>Moderation Queue</h2>
        <v-chip
          v-if="stats.pending > 0"
          color="warning"
          size="small"
          class="ml-2"
        >
          {{ stats.pending }} pending
        </v-chip>
        <v-chip
          v-if="connected"
          color="success"
          size="x-small"
          variant="outlined"
          class="ml-2"
        >
          <v-icon size="x-small" class="mr-1">$broadcast</v-icon>
          Live
        </v-chip>
      </div>
      <div class="queue-stats">
        <span class="stat">
          <v-icon size="small" color="success">$check-circle</v-icon>
          {{ stats.approved }} approved
        </span>
        <span class="stat">
          <v-icon size="small" color="error">$close-circle</v-icon>
          {{ stats.rejected }} rejected
        </span>
      </div>
    </div>

    <!-- Connection status -->
    <v-alert
      v-if="!connected && canConnect"
      type="info"
      variant="tonal"
      density="compact"
      class="mb-4"
    >
      <template #prepend>
        <v-progress-circular
          size="16"
          width="2"
          indeterminate
        />
      </template>
      Connecting to moderation stream...
    </v-alert>

    <v-alert
      v-if="error"
      type="error"
      variant="tonal"
      density="compact"
      class="mb-4"
      closable
      @click:close="error = null"
    >
      {{ error.message }}
    </v-alert>

    <!-- Empty state -->
    <div
      v-if="pendingReleases.length === 0 && !isLoading"
      class="empty-state"
    >
      <v-icon size="64" color="grey-lighten-1">$check-decagram</v-icon>
      <p class="text-h6 mt-4">All caught up!</p>
      <p class="text-caption text-grey">No pending releases to review.</p>
    </div>

    <!-- Loading state -->
    <div
      v-else-if="isLoading"
      class="loading-state"
    >
      <v-progress-circular
        indeterminate
        color="primary"
      />
      <p class="text-caption mt-2">Loading pending releases...</p>
    </div>

    <!-- Pending releases list -->
    <div
      v-else
      class="releases-list"
    >
      <v-card
        v-for="release in pendingReleases"
        :key="release.id"
        class="release-card"
        variant="outlined"
      >
        <div class="release-content">
          <!-- Thumbnail -->
          <div class="release-thumbnail">
            <v-img
              v-if="release.thumbnailCid"
              :src="getThumbnailUrl(release.thumbnailCid)"
              :alt="release.name"
              cover
              class="thumbnail-img"
            />
            <v-icon
              v-else
              size="32"
              color="grey"
            >
              $image-off
            </v-icon>
          </div>

          <!-- Info -->
          <div class="release-info">
            <div class="release-name">{{ release.name }}</div>
            <div class="release-meta text-caption text-grey">
              <span v-if="release.creator">{{ release.creator }}</span>
              <span v-if="release.createdAt"> · {{ formatDate(release.createdAt) }}</span>
            </div>
            <v-chip
              size="x-small"
              variant="tonal"
              class="mt-1"
            >
              {{ release.categoryId }}
            </v-chip>
          </div>

          <!-- Actions -->
          <div class="release-actions">
            <v-btn
              color="success"
              variant="tonal"
              size="small"
              :loading="processingIds.has(release.id)"
              :disabled="processingIds.has(release.id)"
              @click="approveRelease(release.id)"
            >
              <v-icon start>$check</v-icon>
              Approve
            </v-btn>
            <v-btn
              color="error"
              variant="tonal"
              size="small"
              :loading="processingIds.has(release.id)"
              :disabled="processingIds.has(release.id)"
              @click="openRejectDialog(release)"
            >
              <v-icon start>$close</v-icon>
              Reject
            </v-btn>
          </div>
        </div>
      </v-card>
    </div>

    <!-- Reject dialog -->
    <v-dialog
      v-model="rejectDialogOpen"
      max-width="400"
    >
      <v-card>
        <v-card-title>Reject Release</v-card-title>
        <v-card-text>
          <p class="mb-4">
            Are you sure you want to reject "{{ rejectingRelease?.name }}"?
          </p>
          <v-textarea
            v-model="rejectReason"
            label="Reason (optional)"
            rows="3"
            variant="outlined"
            hide-details
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn
            variant="text"
            @click="rejectDialogOpen = false"
          >
            Cancel
          </v-btn>
          <v-btn
            color="error"
            :loading="isRejecting"
            @click="confirmReject"
          >
            Reject
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useAdminWebSocket, type ReleaseInfo } from '/@/composables/useAdminWebSocket';
import { useApproveReleaseMutation, useRejectReleaseMutation } from '/@/plugins/lensService/hooks';
import { parseUrlOrCid } from '/@/utils';

// WebSocket connection
const {
  pendingReleases,
  stats,
  connected,
  error,
  canConnect,
  connect,
  disconnect,
} = useAdminWebSocket();

// Mutations
const approveMutation = useApproveReleaseMutation();
const rejectMutation = useRejectReleaseMutation();

// Local state
const isLoading = ref(true);
const processingIds = ref(new Set<string>());
const rejectDialogOpen = ref(false);
const rejectingRelease = ref<ReleaseInfo | null>(null);
const rejectReason = ref('');
const isRejecting = ref(false);

// Connect on mount
onMounted(() => {
  if (canConnect.value) {
    connect();
  }
  // Give WebSocket time to connect and receive snapshot
  setTimeout(() => {
    isLoading.value = false;
  }, 1500);
});

// Watch for canConnect changes (e.g., after login)
watch(canConnect, (can) => {
  if (can && !connected.value) {
    connect();
  }
});

onUnmounted(() => {
  disconnect();
});

// Helper functions
function getThumbnailUrl(cid: string): string {
  return parseUrlOrCid(cid) ?? '/no-image-icon.png';
}

function formatDate(dateStr: string | null): string {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  return date.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

async function approveRelease(releaseId: string) {
  processingIds.value.add(releaseId);
  try {
    await approveMutation.mutateAsync(releaseId);
  } catch (err) {
    console.error('Failed to approve release:', err);
  } finally {
    processingIds.value.delete(releaseId);
  }
}

function openRejectDialog(release: ReleaseInfo) {
  rejectingRelease.value = release;
  rejectReason.value = '';
  rejectDialogOpen.value = true;
}

async function confirmReject() {
  if (!rejectingRelease.value) return;

  isRejecting.value = true;
  processingIds.value.add(rejectingRelease.value.id);

  try {
    await rejectMutation.mutateAsync({
      releaseId: rejectingRelease.value.id,
      reason: rejectReason.value || undefined,
    });
    rejectDialogOpen.value = false;
  } catch (err) {
    console.error('Failed to reject release:', err);
  } finally {
    isRejecting.value = false;
    if (rejectingRelease.value) {
      processingIds.value.delete(rejectingRelease.value.id);
    }
  }
}
</script>

<style scoped>
.moderation-queue {
  padding: 16px;
}

.queue-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  flex-wrap: wrap;
  gap: 12px;
}

.queue-title {
  display: flex;
  align-items: center;
}

.queue-title h2 {
  font-size: 1.5rem;
  font-weight: 500;
  margin: 0;
}

.queue-stats {
  display: flex;
  gap: 16px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
}

.empty-state,
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  text-align: center;
}

.releases-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.release-card {
  background: rgba(255, 255, 255, 0.02);
}

.release-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
}

.release-thumbnail {
  width: 64px;
  height: 64px;
  border-radius: 8px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.05);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.thumbnail-img {
  width: 100%;
  height: 100%;
}

.release-info {
  flex: 1;
  min-width: 0;
}

.release-name {
  font-size: 15px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.release-meta {
  margin-top: 2px;
}

.release-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

@media (max-width: 600px) {
  .release-content {
    flex-wrap: wrap;
  }

  .release-actions {
    width: 100%;
    justify-content: flex-end;
    margin-top: 8px;
  }
}
</style>
