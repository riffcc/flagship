import { ref, Ref, onUnmounted } from 'vue';
import { getApiUrl, getRelayUrl } from '../utils/runtimeConfig';

export interface NetworkMap {
  mesh_config: {
    width: number;
    height: number;
    depth: number;
    total_slots: number;
  };
  nodes: PeerNode[];
  edges: PeerEdge[];
  stats: NetworkStats;
}

export interface PeerNode {
  id: string;
  label: string;
  slot: {
    /** SPIRAL index (slot number in enumeration order) */
    index: number | null;
    /** Hex axial coordinate q */
    q: number;
    /** Hex axial coordinate r */
    r: number;
    /** Vertical layer z */
    z: number;
  };
  peer_type: 'server' | 'browser';
  last_heartbeat: number;
  capabilities: string[];
  online: boolean;
}

export interface PeerEdge {
  from: string;
  to: string;
  connection_type: 'neighbor' | 'relay';
  latency_ms: number | null;
  color: string;
}

export interface NetworkStats {
  total_peers: number;
  server_nodes: number;
  browser_peers: number;
  mesh_edges: number;
  relay_connections: number;
  occupancy_percent: number;
}

export function useNetworkMap() {
  const networkMap: Ref<NetworkMap | null> = ref(null);
  const loading = ref(false);
  const error: Ref<Error | null> = ref(null);

  let ws: WebSocket | null = null;
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  let isIntentionallyClosed = false;

  const fetchNetworkMap = async () => {
    loading.value = true;
    error.value = null;

    try {
      const apiUrl = getApiUrl();
      const response = await fetch(`${apiUrl}/map`);

      if (!response.ok) {
        throw new Error(`Failed to fetch network map: ${response.statusText}`);
      }

      networkMap.value = await response.json();
    } catch (e) {
      error.value = e as Error;
      console.error('[NetworkMap] Failed to fetch network map:', e);
    } finally {
      loading.value = false;
    }
  };

  const connectWebSocket = () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      return; // Already connected
    }

    const relayUrl = getRelayUrl();
    console.log('[NetworkMap] Connecting to relay WebSocket:', relayUrl);

    ws = new WebSocket(relayUrl);

    ws.onopen = () => {
      console.log('[NetworkMap] WebSocket connected');
      isIntentionallyClosed = false;

      // Send hello message with our peer ID
      const helloMsg = {
        type: 'hello',
        peer_id: `flagship-map-${Date.now()}`,
      };
      ws?.send(JSON.stringify(helloMsg));
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Listen for mesh topology updates
        if (data.type === 'mesh_topology_update') {
          console.log('[NetworkMap] Topology update received:', data.update);

          // Automatically refresh the map when topology changes
          fetchNetworkMap();
        }
      } catch (e) {
        console.error('[NetworkMap] Failed to parse WebSocket message:', e);
      }
    };

    ws.onerror = (event) => {
      console.error('[NetworkMap] WebSocket error:', event);
    };

    ws.onclose = () => {
      console.log('[NetworkMap] WebSocket closed');
      ws = null;

      // Reconnect after 5 seconds unless intentionally closed
      if (!isIntentionallyClosed) {
        console.log('[NetworkMap] Reconnecting in 5 seconds...');
        reconnectTimeout = setTimeout(() => {
          connectWebSocket();
        }, 5000);
      }
    };
  };

  const disconnectWebSocket = () => {
    isIntentionallyClosed = true;

    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }

    if (ws) {
      ws.close();
      ws = null;
    }
  };

  // Cleanup on unmount
  onUnmounted(() => {
    disconnectWebSocket();
  });

  return {
    networkMap,
    loading,
    error,
    fetchNetworkMap,
    connectWebSocket,
    disconnectWebSocket,
  };
}
