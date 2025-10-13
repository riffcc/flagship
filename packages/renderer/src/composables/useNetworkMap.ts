import { ref, Ref } from 'vue';
import { getApiUrl } from '../utils/runtimeConfig';

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
    x: number;
    y: number;
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

  return {
    networkMap,
    loading,
    error,
    fetchNetworkMap,
  };
}
