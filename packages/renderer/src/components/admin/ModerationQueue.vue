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
      <div class="queue-controls">
        <!-- View mode toggle -->
        <v-btn-toggle
          v-model="viewMode"
          mandatory
          density="compact"
          variant="outlined"
          class="mr-4"
        >
          <v-btn value="endless" size="small">
            <v-icon size="small" class="mr-1">$gesture-swipe-horizontal</v-icon>
            Endless
          </v-btn>
          <v-btn value="paged" size="small">
            <v-icon size="small" class="mr-1">$format-list-bulleted</v-icon>
            Paged
          </v-btn>
        </v-btn-toggle>
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

    <!-- Endless grid view - reuses InfiniteReleaseList -->
    <infinite-release-list
      v-else-if="viewMode === 'endless'"
      :items="pendingReleasesAsItems"
      @release-click="(item) => openPreviewDialogById(item.id)"
    />

    <!-- Paged data table view - reuses ContentManagement -->
    <content-management
      v-else
      :items="pendingReleasesAsItems"
      :loading="isLoading"
      :headers="moderationHeaders"
      hide-default-actions
    >
      <template #item.thumbnail="{ item }">
        <v-card class="my-2" elevation="2" rounded>
          <v-img
            :src="getThumbnailUrl(item.thumbnailCID)"
            height="64"
            width="64"
            cover
          />
        </v-card>
      </template>
      <template #item.name="{ item }">
        <div>
          <span class="font-weight-medium">{{ item.name }}</span>
          <span
            v-if="item.metadata?.author"
            class="text-caption text-grey d-block"
          >{{ item.metadata.author }}</span>
        </div>
      </template>
      <template #item.createdAt="{ item }">
        <span class="text-caption">{{ formatDate(item.createdAt) }}</span>
      </template>
      <template #actions="{ item }">
        <div class="d-flex">
          <v-btn
            color="info"
            variant="text"
            size="small"
            icon
            @click="openPreviewDialogById(item.id)"
          >
            <v-icon>$eye</v-icon>
            <v-tooltip activator="parent" location="top">Preview</v-tooltip>
          </v-btn>
          <v-btn
            color="success"
            variant="text"
            size="small"
            icon
            :loading="processingIds.has(item.id)"
            :disabled="processingIds.has(item.id)"
            @click="approveRelease(item.id)"
          >
            <v-icon>$check</v-icon>
            <v-tooltip activator="parent" location="top">Approve</v-tooltip>
          </v-btn>
          <v-btn
            color="error"
            variant="text"
            size="small"
            icon
            :loading="processingIds.has(item.id)"
            :disabled="processingIds.has(item.id)"
            @click="openRejectDialogById(item.id)"
          >
            <v-icon>$close</v-icon>
            <v-tooltip activator="parent" location="top">Reject</v-tooltip>
          </v-btn>
        </div>
      </template>
    </content-management>

    <!-- Preview dialog -->
    <v-dialog
      v-model="previewDialogOpen"
      max-width="600"
    >
      <v-card v-if="previewingRelease">
        <v-img
          :src="getThumbnailUrl(previewingRelease.thumbnailCid)"
          :alt="previewingRelease.name"
          height="300"
          cover
        />
        <v-card-title>{{ previewingRelease.name }}</v-card-title>
        <v-card-subtitle v-if="previewingRelease.creator">
          {{ previewingRelease.creator }}
        </v-card-subtitle>
        <v-card-text>
          <v-chip size="small" variant="tonal" class="mr-2">
            {{ previewingRelease.categoryId }}
          </v-chip>
          <v-chip
            size="small"
            color="warning"
            variant="tonal"
          >
            Pending
          </v-chip>
          <p
            v-if="previewingRelease.createdAt"
            class="text-caption mt-2"
          >
            Submitted: {{ formatDate(previewingRelease.createdAt) }}
          </p>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn
            variant="text"
            @click="previewDialogOpen = false"
          >
            Close
          </v-btn>
          <v-btn
            color="error"
            variant="tonal"
            :loading="processingIds.has(previewingRelease.id)"
            @click="openRejectDialog(previewingRelease); previewDialogOpen = false"
          >
            Reject
          </v-btn>
          <v-btn
            color="success"
            variant="flat"
            :loading="processingIds.has(previewingRelease.id)"
            @click="approveRelease(previewingRelease.id); previewDialogOpen = false"
          >
            Approve
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

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
import InfiniteReleaseList from '/@/components/misc/infiniteReleaseList.vue';
import ContentManagement from '/@/components/admin/contentManagement.vue';
import type { ReleaseItem } from '/@/types';

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

// View mode state
const viewMode = ref<'endless' | 'paged'>('endless');

// Custom headers for moderation queue (passed to ContentManagement)
const moderationHeaders = [
  { title: '', key: 'thumbnail', sortable: false, width: '80px' },
  { title: 'Name', key: 'name', sortable: true },
  { title: 'Category', key: 'categoryId', sortable: true },
  { title: 'Submitted', key: 'createdAt', sortable: true },
  { title: 'Actions', key: 'actions', sortable: false, align: 'end' as const },
];

// Local state
const isLoading = ref(true);
const processingIds = ref(new Set<string>());
const rejectDialogOpen = ref(false);
const rejectingRelease = ref<ReleaseInfo | null>(null);
const rejectReason = ref('');
const isRejecting = ref(false);
const previewDialogOpen = ref(false);
const previewingRelease = ref<ReleaseInfo | null>(null);

// Map WebSocket ReleaseInfo to ReleaseItem format for ContentCard compatibility
const pendingReleasesAsItems = computed<ReleaseItem[]>(() => {
  return pendingReleases.value.map(r => ({
    id: r.id,
    name: r.name,
    categoryId: r.categoryId,
    thumbnailCID: r.thumbnailCid ?? undefined,
    contentCID: undefined, // Not available in moderation snapshot
    creator: r.creator ?? undefined,
    status: r.status,
    createdAt: r.createdAt ?? undefined,
    metadata: {
      artist: r.creator ?? undefined, // Standard field for music category
    },
  } as ReleaseItem));
});

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

function openPreviewDialog(release: ReleaseInfo) {
  previewingRelease.value = release;
  previewDialogOpen.value = true;
}

function openRejectDialogById(releaseId: string) {
  const release = pendingReleases.value.find(r => r.id === releaseId);
  if (release) {
    openRejectDialog(release);
  }
}

function openPreviewDialogById(releaseId: string) {
  const release = pendingReleases.value.find(r => r.id === releaseId);
  if (release) {
    openPreviewDialog(release);
  }
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

.queue-controls {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
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

@media (max-width: 600px) {
  .queue-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .queue-controls {
    width: 100%;
    justify-content: space-between;
  }
}
</style>
