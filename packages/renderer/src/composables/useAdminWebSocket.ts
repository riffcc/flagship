/**
 * Admin WebSocket composable for real-time moderation updates
 * Connects to /api/v1/ws/admin with admin authentication
 */

import { ref, Ref, onUnmounted, computed, watch } from 'vue';
import { getApiUrl } from '../utils/runtimeConfig';
import { useIdentity } from './useIdentity';
import { useAccountStatusQuery } from '../plugins/lensService/hooks';

// Admin WebSocket event types from citadel-lens ws_admin.rs
export interface AdminEventConnected {
  type: 'connected';
  admin_pubkey: string;
}

export interface ReleaseInfo {
  id: string;
  name: string;
  categoryId: string;
  creator: string | null;
  thumbnailCid: string | null;
  status: string;
  createdAt: string | null;
}

export interface AdminEventSnapshot {
  type: 'snapshot';
  pending_count: number;
  approved_count: number;
  rejected_count: number;
  pending_releases: ReleaseInfo[];
}

export interface AdminEventReleaseSubmitted {
  type: 'release_submitted';
  release: ReleaseInfo;
}

export interface AdminEventReleaseApproved {
  type: 'release_approved';
  release_id: string;
  moderator: string;
  timestamp: string;
}

export interface AdminEventReleaseRejected {
  type: 'release_rejected';
  release_id: string;
  moderator: string;
  reason: string | null;
  timestamp: string;
}

export interface AdminEventStatsUpdated {
  type: 'stats_updated';
  pending: number;
  approved: number;
  rejected: number;
}

export interface AdminEventHeartbeat {
  type: 'heartbeat';
  timestamp: number;
}

export interface AdminEventError {
  type: 'error';
  message: string;
}

export type AdminEvent =
  | AdminEventConnected
  | AdminEventSnapshot
  | AdminEventReleaseSubmitted
  | AdminEventReleaseApproved
  | AdminEventReleaseRejected
  | AdminEventStatsUpdated
  | AdminEventHeartbeat
  | AdminEventError;

export interface ModerationStats {
  pending: number;
  approved: number;
  rejected: number;
}

/**
 * Get WebSocket URL for admin moderation updates
 * Constructs ws(s)://host/api/v1/ws/admin?pubkey=... from the API URL
 */
function getAdminWsUrl(pubkey: string): string {
  const apiUrl = getApiUrl();
  const wsUrl = apiUrl.replace('http://', 'ws://').replace('https://', 'wss://');
  return `${wsUrl}/ws/admin?pubkey=${encodeURIComponent(pubkey)}`;
}

/**
 * Composable for real-time admin moderation updates via WebSocket
 */
export function useAdminWebSocket() {
  const { publicKey } = useIdentity();
  const { data: accountStatus } = useAccountStatusQuery();
  const isAdmin = computed(() => accountStatus.value?.isAdmin ?? false);

  const pendingReleases: Ref<ReleaseInfo[]> = ref([]);
  const stats: Ref<ModerationStats> = ref({ pending: 0, approved: 0, rejected: 0 });
  const connected = ref(false);
  const error: Ref<Error | null> = ref(null);
  const lastEvent: Ref<AdminEvent | null> = ref(null);
  const authenticatedAs: Ref<string | null> = ref(null);

  let ws: WebSocket | null = null;
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  let isIntentionallyClosed = false;

  const canConnect = computed(() => isAdmin.value && publicKey.value);

  /**
   * Handle incoming WebSocket message
   */
  const handleAdminEvent = (event: AdminEvent) => {
    lastEvent.value = event;

    switch (event.type) {
      case 'connected': {
        const connectedEvent = event as AdminEventConnected;
        console.log('[AdminWS] Authenticated as:', connectedEvent.admin_pubkey);
        authenticatedAs.value = connectedEvent.admin_pubkey;
        break;
      }

      case 'snapshot': {
        const snapshot = event as AdminEventSnapshot;
        console.log('[AdminWS] Snapshot received:', snapshot.pending_count, 'pending releases');
        pendingReleases.value = snapshot.pending_releases;
        stats.value = {
          pending: snapshot.pending_count,
          approved: snapshot.approved_count,
          rejected: snapshot.rejected_count,
        };
        break;
      }

      case 'release_submitted': {
        const submitted = event as AdminEventReleaseSubmitted;
        console.log('[AdminWS] New release submitted:', submitted.release.id);
        // Add to pending releases
        pendingReleases.value = [...pendingReleases.value, submitted.release];
        stats.value = { ...stats.value, pending: stats.value.pending + 1 };
        break;
      }

      case 'release_approved': {
        const approved = event as AdminEventReleaseApproved;
        console.log('[AdminWS] Release approved:', approved.release_id);
        // Remove from pending releases
        pendingReleases.value = pendingReleases.value.filter(r => r.id !== approved.release_id);
        stats.value = {
          ...stats.value,
          pending: Math.max(0, stats.value.pending - 1),
          approved: stats.value.approved + 1,
        };
        break;
      }

      case 'release_rejected': {
        const rejected = event as AdminEventReleaseRejected;
        console.log('[AdminWS] Release rejected:', rejected.release_id, 'reason:', rejected.reason);
        // Remove from pending releases
        pendingReleases.value = pendingReleases.value.filter(r => r.id !== rejected.release_id);
        stats.value = {
          ...stats.value,
          pending: Math.max(0, stats.value.pending - 1),
          rejected: stats.value.rejected + 1,
        };
        break;
      }

      case 'stats_updated': {
        const statsEvent = event as AdminEventStatsUpdated;
        console.log('[AdminWS] Stats updated:', statsEvent);
        stats.value = {
          pending: statsEvent.pending,
          approved: statsEvent.approved,
          rejected: statsEvent.rejected,
        };
        break;
      }

      case 'heartbeat':
        // Just a keepalive, no action needed
        break;

      case 'error': {
        const errorEvent = event as AdminEventError;
        console.error('[AdminWS] Server error:', errorEvent.message);
        error.value = new Error(errorEvent.message);
        break;
      }

      default:
        console.log('[AdminWS] Unknown event type:', (event as { type: string }).type);
    }
  };

  /**
   * Connect to WebSocket for real-time admin updates
   */
  const connect = () => {
    if (!canConnect.value) {
      console.warn('[AdminWS] Cannot connect: not an admin or no public key');
      return;
    }

    if (ws && ws.readyState === WebSocket.OPEN) {
      return; // Already connected
    }

    const wsUrl = getAdminWsUrl(publicKey.value!);
    console.log('[AdminWS] Connecting to admin WebSocket');

    try {
      ws = new WebSocket(wsUrl);
    } catch (e) {
      console.error('[AdminWS] Failed to create WebSocket:', e);
      error.value = e as Error;
      return;
    }

    ws.onopen = () => {
      console.log('[AdminWS] WebSocket connected');
      connected.value = true;
      isIntentionallyClosed = false;
      error.value = null;
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as AdminEvent;
        handleAdminEvent(data);
      } catch (e) {
        console.error('[AdminWS] Failed to parse WebSocket message:', e);
      }
    };

    ws.onerror = (event) => {
      console.error('[AdminWS] WebSocket error:', event);
      error.value = new Error('WebSocket connection error');
    };

    ws.onclose = (event) => {
      console.log('[AdminWS] WebSocket closed, code:', event.code);
      ws = null;
      connected.value = false;
      authenticatedAs.value = null;

      // Don't reconnect if auth failed (403)
      if (event.code === 1003 || event.code === 4403) {
        console.warn('[AdminWS] Authentication failed, not reconnecting');
        error.value = new Error('Admin authentication failed');
        return;
      }

      // Reconnect after 2 seconds unless intentionally closed
      if (!isIntentionallyClosed) {
        console.log('[AdminWS] Reconnecting in 2 seconds...');
        reconnectTimeout = setTimeout(() => {
          connect();
        }, 2000);
      }
    };
  };

  /**
   * Disconnect WebSocket
   */
  const disconnect = () => {
    isIntentionallyClosed = true;

    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }

    if (ws) {
      ws.close();
      ws = null;
    }

    connected.value = false;
    authenticatedAs.value = null;
  };

  /**
   * Request a fresh snapshot from the server
   */
  const requestRefresh = () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send('refresh');
    }
  };

  // Cleanup on unmount
  onUnmounted(() => {
    disconnect();
  });

  return {
    // State
    pendingReleases,
    stats,
    connected,
    error,
    lastEvent,
    authenticatedAs,
    canConnect,

    // Actions
    connect,
    disconnect,
    requestRefresh,
  };
}
