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
        <!-- Clickable filter stats -->
        <div class="queue-stats">
          <button
            class="stat stat--clickable"
            :class="{ 'stat--active': statusFilter === 'pending' }"
            @click="toggleFilter('pending')"
          >
            <v-icon size="small" color="warning">$clock-outline</v-icon>
            {{ stats.pending }} pending
          </button>
          <button
            class="stat stat--clickable"
            :class="{ 'stat--active': statusFilter === 'approved' }"
            @click="toggleFilter('approved')"
          >
            <v-icon size="small" color="success">$check-circle</v-icon>
            {{ stats.approved }} approved
          </button>
          <button
            class="stat stat--clickable"
            :class="{ 'stat--active': statusFilter === 'rejected' }"
            @click="toggleFilter('rejected')"
          >
            <v-icon size="small" color="error">$close-circle</v-icon>
            {{ stats.rejected }} rejected
          </button>
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

    <!-- Selection toolbar -->
    <v-slide-y-transition>
      <div
        v-if="selectedIds.size > 0"
        class="selection-toolbar"
      >
        <div class="selection-info">
          <v-btn
            icon
            variant="text"
            size="small"
            @click="clearSelection"
          >
            <v-icon>$close</v-icon>
          </v-btn>
          <span class="ml-2">{{ selectedIds.size }} selected</span>
          <v-btn
            variant="text"
            size="small"
            class="ml-2"
            @click="selectAll"
          >
            Select all ({{ filteredReleases.length }})
          </v-btn>
        </div>
        <div class="selection-actions">
          <v-btn
            color="success"
            variant="tonal"
            size="small"
            :loading="isBulkProcessing"
            @click="bulkApprove"
          >
            <v-icon class="mr-1">$check</v-icon>
            Approve selected
          </v-btn>
          <v-btn
            color="error"
            variant="tonal"
            size="small"
            class="ml-2"
            :loading="isBulkProcessing"
            @click="bulkRejectDialog = true"
          >
            <v-icon class="mr-1">$close</v-icon>
            Reject selected
          </v-btn>
          <!-- Delete button with confirmation -->
          <v-btn
            :color="deleteConfirmState === 'idle' ? 'grey' : (deleteConfirmState === 'confirm-block' ? 'error' : 'warning')"
            :variant="deleteConfirmState === 'idle' ? 'tonal' : 'flat'"
            size="small"
            class="ml-2"
            :loading="isBulkProcessing"
            @mousedown="startDeleteHold"
            @mouseup="endDeleteHold"
            @mouseleave="endDeleteHold"
            @touchstart.passive="startDeleteHold"
            @touchend="endDeleteHold"
            @touchcancel="endDeleteHold"
            @click="handleDeleteClick"
          >
            <v-icon class="mr-1">$delete</v-icon>
            <template v-if="deleteConfirmState === 'idle'">
              Delete selected
            </template>
            <template v-else-if="deleteConfirmState === 'confirm'">
              Click to confirm
            </template>
            <template v-else-if="deleteConfirmState === 'confirm-block'">
              DELETE + BLOCK
            </template>
          </v-btn>
          <!-- Delete hold progress indicator -->
          <v-progress-linear
            v-if="deleteHoldProgress > 0"
            :model-value="deleteHoldProgress"
            color="error"
            height="3"
            class="delete-progress"
          />
        </div>
      </div>
    </v-slide-y-transition>

    <!-- Filter indicator -->
    <v-chip
      v-if="statusFilter !== 'all'"
      closable
      class="mb-4"
      @click:close="statusFilter = 'all'"
    >
      Showing: {{ statusFilter }}
    </v-chip>

    <!-- Empty state -->
    <div
      v-if="filteredReleases.length === 0 && !isLoading"
      class="empty-state"
    >
      <v-icon size="64" color="grey-lighten-1">$check-decagram</v-icon>
      <p class="text-h6 mt-4">
        {{ statusFilter === 'all' ? 'All caught up!' : `No ${statusFilter} releases` }}
      </p>
      <p class="text-caption text-grey">
        {{ statusFilter === 'all' ? 'No pending releases to review.' : `No releases with status "${statusFilter}".` }}
      </p>
      <v-btn
        v-if="statusFilter !== 'all'"
        variant="text"
        class="mt-2"
        @click="statusFilter = 'all'"
      >
        Show all releases
      </v-btn>
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
      <p class="text-caption mt-2">Loading releases...</p>
    </div>

    <!-- Endless grid view with selection support -->
    <div
      v-else-if="viewMode === 'endless'"
      class="selectable-grid"
      :class="{ 'selection-mode': selectionMode }"
    >
      <div
        v-for="item in filteredReleasesAsItems"
        :key="item.id"
        class="selectable-item"
        :class="{
          'selectable-item--selected': selectedIds.has(item.id),
        }"
        @click="handleItemClick(item.id, $event)"
        @mousedown="startLongPress(item.id)"
        @mouseup="cancelLongPress"
        @mouseleave="cancelLongPress"
        @touchstart.passive="startLongPress(item.id)"
        @touchend="cancelLongPress"
        @touchcancel="cancelLongPress"
      >
        <!-- Selection checkbox overlay -->
        <div
          v-if="selectionMode"
          class="selection-checkbox"
        >
          <v-checkbox-btn
            :model-value="selectedIds.has(item.id)"
            color="primary"
            @click.stop="toggleSelection(item.id)"
          />
        </div>
        <content-card
          :item="item"
          @click.stop="handleItemClick(item.id, $event)"
        />
      </div>
    </div>

    <!-- Paged data table view - reuses ContentManagement -->
    <content-management
      v-else
      :items="filteredReleasesAsItems"
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

    <!-- Preview dialog with album viewer embedded in moderation chrome -->
    <v-dialog
      v-model="previewDialogOpen"
      max-width="1000"
      scrollable
    >
      <v-card style="max-height: 90vh; overflow-y: auto;">
        <!-- Loading state -->
        <div v-if="isLoadingPreview" class="d-flex align-center justify-center pa-8" style="min-height: 400px;">
          <v-progress-circular indeterminate size="64" />
        </div>
        <!-- Album viewer with moderation controls -->
        <AlbumViewer
          v-else-if="fullPreviewRelease"
          :release="fullPreviewRelease as ReleaseItem"
          :moderation-mode="true"
          :moderation-loading="previewingReleaseId ? processingIds.has(previewingReleaseId) : false"
          @close="previewDialogOpen = false"
          @approve="handlePreviewApprove"
          @reject="handlePreviewReject"
          @delete="handlePreviewDelete"
        />
        <!-- Fallback if release not found -->
        <div v-else class="d-flex flex-column align-center justify-center pa-8" style="min-height: 200px;">
          <p class="text-h6 mb-4">Release not found</p>
          <v-btn @click="previewDialogOpen = false">Close</v-btn>
        </div>
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

    <!-- Bulk reject dialog -->
    <v-dialog
      v-model="bulkRejectDialog"
      max-width="400"
    >
      <v-card>
        <v-card-title>Reject {{ selectedIds.size }} Releases</v-card-title>
        <v-card-text>
          <p class="mb-4">
            Are you sure you want to reject {{ selectedIds.size }} selected releases?
          </p>
          <v-textarea
            v-model="bulkRejectReason"
            label="Reason (optional, applies to all)"
            rows="3"
            variant="outlined"
            hide-details
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn
            variant="text"
            @click="bulkRejectDialog = false"
          >
            Cancel
          </v-btn>
          <v-btn
            color="error"
            :loading="isBulkProcessing"
            @click="confirmBulkReject"
          >
            Reject All
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, reactive } from 'vue';
import { useAdminWebSocket, type ReleaseInfo } from '/@/composables/useAdminWebSocket';
import { useApproveReleaseMutation, useRejectReleaseMutation, useDeleteReleaseMutation, usePendingReleasesQuery } from '/@/plugins/lensService/hooks';
import { parseUrlOrCid } from '/@/utils';
import ContentManagement from '/@/components/admin/contentManagement.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import AlbumViewer from '/@/components/releases/albumViewer.vue';
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
const deleteMutation = useDeleteReleaseMutation();

// View mode state
const viewMode = ref<'endless' | 'paged'>('endless');

// Filter state
const statusFilter = ref<'all' | 'pending' | 'approved' | 'rejected'>('all');

// Selection state
const selectedIds = reactive(new Set<string>());
const selectionMode = ref(false);
let longPressTimer: ReturnType<typeof setTimeout> | null = null;
const LONG_PRESS_DURATION = 500; // ms

// Bulk action state
const bulkRejectDialog = ref(false);
const bulkRejectReason = ref('');
const isBulkProcessing = ref(false);

// Delete confirmation state
const deleteConfirmState = ref<'idle' | 'confirm' | 'confirm-block'>('idle');
const deleteHoldProgress = ref(0);
let deleteHoldTimer: ReturnType<typeof setInterval> | null = null;
let deleteConfirmTimeout: ReturnType<typeof setTimeout> | null = null;
const DELETE_HOLD_DURATION = 4000; // 4 seconds to enable blocklist
const DELETE_CONFIRM_TIMEOUT = 5000; // 5 seconds before resetting confirm state

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
const previewingReleaseId = ref<string | null>(null);

// Fetch pending releases with full data (includes contentCID for playback)
// This fetches from /releases/pending which returns complete ReleaseItem objects
const { data: pendingReleasesData, isLoading: isLoadingPendingReleases } = usePendingReleasesQuery({
  enabled: computed(() => previewDialogOpen.value),
});

// Find the full release data from the pending releases query
const fullPreviewRelease = computed(() => {
  if (!previewingReleaseId.value || !pendingReleasesData.value) return null;
  return pendingReleasesData.value.find(r => r.id === previewingReleaseId.value) ?? null;
});
const isLoadingPreview = computed(() => isLoadingPendingReleases.value && previewDialogOpen.value);

// Filter releases by status
const filteredReleases = computed(() => {
  if (statusFilter.value === 'all') {
    return pendingReleases.value;
  }
  return pendingReleases.value.filter(r => r.status === statusFilter.value);
});

// Map filtered releases to ReleaseItem format for ContentCard compatibility
const filteredReleasesAsItems = computed<ReleaseItem[]>(() => {
  return filteredReleases.value.map(r => ({
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

// Toggle filter
function toggleFilter(filter: 'pending' | 'approved' | 'rejected') {
  if (statusFilter.value === filter) {
    statusFilter.value = 'all';
  } else {
    statusFilter.value = filter;
  }
  // Clear selection when changing filters
  clearSelection();
}

// Selection functions
function toggleSelection(id: string) {
  if (selectedIds.has(id)) {
    selectedIds.delete(id);
    if (selectedIds.size === 0) {
      selectionMode.value = false;
    }
  } else {
    selectedIds.add(id);
  }
}

function selectAll() {
  filteredReleases.value.forEach(r => selectedIds.add(r.id));
}

function clearSelection() {
  selectedIds.clear();
  selectionMode.value = false;
}

// Track if long-press just completed to ignore the subsequent click
let longPressJustCompleted = false;

function handleItemClick(id: string, _event: MouseEvent) {
  // Ignore click if it's from the end of a long-press
  if (longPressJustCompleted) {
    longPressJustCompleted = false;
    return;
  }

  if (selectionMode.value) {
    toggleSelection(id);
  } else {
    openPreviewDialogById(id);
  }
}

function startLongPress(id: string) {
  cancelLongPress();
  longPressJustCompleted = false;
  longPressTimer = setTimeout(() => {
    // Enter selection mode and select this item
    selectionMode.value = true;
    selectedIds.add(id);
    longPressJustCompleted = true; // Flag to ignore subsequent click
    // Provide haptic feedback if available
    if (navigator.vibrate) {
      navigator.vibrate(50);
    }
  }, LONG_PRESS_DURATION);
}

function cancelLongPress() {
  if (longPressTimer) {
    clearTimeout(longPressTimer);
    longPressTimer = null;
  }
}

// Bulk actions
async function bulkApprove() {
  if (selectedIds.size === 0) return;

  isBulkProcessing.value = true;
  // Use spread with values() for proper iteration over reactive Set
  const ids = [...selectedIds.values()];

  // Clear selection immediately - we're committing to these
  clearSelection();

  // Process all in parallel - the mutations will each trigger their own
  // invalidation, but since we've already cleared selection it's fine
  const results = await Promise.allSettled(
    ids.map(id => approveMutation.mutateAsync(id))
  );

  const failures = results.filter(r => r.status === 'rejected');
  if (failures.length > 0) {
    console.error(`Failed to approve ${failures.length} releases:`, failures);
  }

  isBulkProcessing.value = false;
}

async function confirmBulkReject() {
  if (selectedIds.size === 0) return;

  isBulkProcessing.value = true;
  // Use spread with values() for proper iteration over reactive Set
  const ids = [...selectedIds.values()];
  const reason = bulkRejectReason.value || undefined;

  // Clear selection and close dialog immediately
  clearSelection();
  bulkRejectDialog.value = false;
  bulkRejectReason.value = '';

  // Process all in parallel
  const results = await Promise.allSettled(
    ids.map(id => rejectMutation.mutateAsync({ releaseId: id, reason }))
  );

  const failures = results.filter(r => r.status === 'rejected');
  if (failures.length > 0) {
    console.error(`Failed to reject ${failures.length} releases:`, failures);
  }

  isBulkProcessing.value = false;
}

// Delete hold/click handlers
function startDeleteHold() {
  if (deleteConfirmState.value === 'idle') return;

  // Start the hold timer for blocklist mode
  let elapsed = 0;
  const interval = 50; // Update every 50ms for smooth progress
  deleteHoldTimer = setInterval(() => {
    elapsed += interval;
    deleteHoldProgress.value = (elapsed / DELETE_HOLD_DURATION) * 100;

    if (elapsed >= DELETE_HOLD_DURATION) {
      // Hold complete - enable blocklist mode
      deleteConfirmState.value = 'confirm-block';
      if (navigator.vibrate) {
        navigator.vibrate([100, 50, 100]); // Double vibration
      }
      endDeleteHold();
    }
  }, interval);
}

function endDeleteHold() {
  if (deleteHoldTimer) {
    clearInterval(deleteHoldTimer);
    deleteHoldTimer = null;
  }
  // Keep progress visible if in blocklist mode
  if (deleteConfirmState.value !== 'confirm-block') {
    deleteHoldProgress.value = 0;
  }
}

function resetDeleteState() {
  deleteConfirmState.value = 'idle';
  deleteHoldProgress.value = 0;
  if (deleteConfirmTimeout) {
    clearTimeout(deleteConfirmTimeout);
    deleteConfirmTimeout = null;
  }
}

function handleDeleteClick() {
  if (deleteConfirmState.value === 'idle') {
    // First click - enter confirm mode
    deleteConfirmState.value = 'confirm';
    // Auto-reset after timeout
    deleteConfirmTimeout = setTimeout(() => {
      resetDeleteState();
    }, DELETE_CONFIRM_TIMEOUT);
  } else {
    // Second click - execute delete
    executeDelete(deleteConfirmState.value === 'confirm-block');
    resetDeleteState();
  }
}

async function executeDelete(blocklist: boolean) {
  if (selectedIds.size === 0) return;

  isBulkProcessing.value = true;
  // Use spread with values() for proper iteration over reactive Set
  const ids = [...selectedIds.values()];

  // Clear selection immediately
  clearSelection();

  if (blocklist) {
    // TODO: When blocklist API is available, blocklist these releases
    console.log('Blocklist requested for releases:', ids);
  }

  // Process all in parallel
  const results = await Promise.allSettled(
    ids.map(id => deleteMutation.mutateAsync(id))
  );

  const failures = results.filter(r => r.status === 'rejected');
  if (failures.length > 0) {
    console.error(`Failed to delete ${failures.length} releases:`, failures);
  }

  isBulkProcessing.value = false;
}

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
  cancelLongPress();
  endDeleteHold();
  if (deleteConfirmTimeout) {
    clearTimeout(deleteConfirmTimeout);
  }
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

async function deleteRelease(releaseId: string) {
  processingIds.value.add(releaseId);
  try {
    await deleteMutation.mutateAsync(releaseId);
  } catch (err) {
    console.error('Failed to delete release:', err);
  } finally {
    processingIds.value.delete(releaseId);
  }
}

function openRejectDialog(release: ReleaseInfo) {
  rejectingRelease.value = release;
  rejectReason.value = '';
  rejectDialogOpen.value = true;
}

function openPreviewDialogById(releaseId: string) {
  previewingReleaseId.value = releaseId;
  previewDialogOpen.value = true;
}

function openRejectDialogById(releaseId: string) {
  const release = pendingReleases.value.find(r => r.id === releaseId);
  if (release) {
    openRejectDialog(release);
  }
}

// Handlers for moderation actions from album viewer
function handlePreviewApprove() {
  if (previewingReleaseId.value) {
    approveRelease(previewingReleaseId.value);
    previewDialogOpen.value = false;
  }
}

function handlePreviewReject() {
  if (previewingReleaseId.value) {
    const release = pendingReleases.value.find(r => r.id === previewingReleaseId.value);
    if (release) {
      openRejectDialog(release);
    }
    previewDialogOpen.value = false;
  }
}

function handlePreviewDelete() {
  if (previewingReleaseId.value) {
    deleteRelease(previewingReleaseId.value);
    previewDialogOpen.value = false;
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
  background: none;
  border: none;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.stat--clickable {
  cursor: pointer;
}

.stat--clickable:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.9);
}

.stat--active {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

/* Selection toolbar */
.selection-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: rgba(var(--v-theme-primary), 0.15);
  border-radius: 8px;
  margin-bottom: 16px;
}

.selection-info {
  display: flex;
  align-items: center;
}

.selection-actions {
  display: flex;
  align-items: center;
  position: relative;
}

.delete-progress {
  position: absolute;
  bottom: -4px;
  left: 0;
  right: 0;
  border-radius: 2px;
}

/* Selectable grid */
.selectable-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 16px;
}

.selectable-item {
  position: relative;
  cursor: pointer;
  border-radius: 8px;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.selectable-item:hover {
  transform: translateY(-2px);
}

.selection-mode .selectable-item {
  user-select: none;
}

.selectable-item--selected {
  box-shadow: 0 0 0 3px rgba(var(--v-theme-primary), 0.8);
}

.selection-checkbox {
  position: absolute;
  top: 4px;
  left: 4px;
  z-index: 10;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 4px;
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
