import { ref, Ref, onUnmounted } from 'vue';
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
  latency_stats?: LatencyStats;
  color: string;
}

export interface LatencyStats {
  last_1s_ms: number | null;
  last_60s_ms: number | null;
  last_1h_ms: number | null;
  samples_1s: number;
  samples_60s: number;
  samples_1h: number;
}

export interface NetworkStats {
  total_peers: number;
  server_nodes: number;
  browser_peers: number;
  mesh_edges: number;
  relay_connections: number;
  occupancy_percent: number;
}

// WebSocket event types from citadel-lens
interface MeshEventSnapshot {
  type: 'snapshot';
  self_id: string;
  peers: WsPeerInfo[];
  slots: WsSlotInfo[];
}

interface MeshEventPeerJoined {
  type: 'peer_joined';
  id: string;
  addr: string;
  slot: number | null;
}

interface MeshEventPeerLeft {
  type: 'peer_left';
  id: string;
}

interface MeshEventSlotClaimed {
  type: 'slot_claimed';
  index: number;
  peer_id: string;
  coord: [number, number, number];
}

interface MeshEventHeartbeat {
  type: 'heartbeat';
  timestamp: number;
}

interface MeshEventCvdfNewRound {
  type: 'cvdf_new_round';
  round: number;
  weight: number;
  attestation_count: number;
  spore_ranges: number;
}

type MeshEvent =
  | MeshEventSnapshot
  | MeshEventPeerJoined
  | MeshEventPeerLeft
  | MeshEventSlotClaimed
  | MeshEventHeartbeat
  | MeshEventCvdfNewRound
  | { type: string; [key: string]: unknown };

interface WsPeerInfo {
  id: string;
  addr: string;
  slot: WsSlotInfo | null;
  online: boolean;
}

interface WsSlotInfo {
  index: number;
  peer_id: string;
  coord: [number, number, number];
  confirmations: number;
}

/**
 * Get WebSocket URL for mesh updates
 * Constructs ws(s)://host/api/v1/ws/mesh from the API URL
 */
function getMeshWsUrl(): string {
  const apiUrl = getApiUrl();
  const wsUrl = apiUrl.replace('http://', 'ws://').replace('https://', 'wss://');
  // apiUrl is like "http://localhost:8080/api/v1" or "https://api.global.riff.cc/api/v1"
  // We need "ws://localhost:8080/api/v1/ws/mesh"
  return `${wsUrl}/ws/mesh`;
}

export function useNetworkMap() {
  const networkMap: Ref<NetworkMap | null> = ref(null);
  const loading = ref(false);
  const initialLoading = ref(true); // Only true until first successful load
  const error: Ref<Error | null> = ref(null);
  const connected = ref(false);
  const lastEvent: Ref<MeshEvent | null> = ref(null);

  let ws: WebSocket | null = null;
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  let isIntentionallyClosed = false;

  /**
   * Fetch network map via HTTP (initial load or fallback)
   */
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
      initialLoading.value = false; // First load complete
    } catch (e) {
      error.value = e as Error;
      console.error('[NetworkMap] Failed to fetch network map:', e);
    } finally {
      loading.value = false;
    }
  };

  /**
   * Convert WebSocket snapshot to NetworkMap format
   */
  const snapshotToNetworkMap = (snapshot: MeshEventSnapshot): NetworkMap => {
    const nodes: PeerNode[] = snapshot.peers.map(peer => ({
      id: peer.id,
      label: peer.id.substring(0, 12) + '...',
      slot: peer.slot ? {
        index: peer.slot.index,
        q: peer.slot.coord[0],
        r: peer.slot.coord[1],
        z: peer.slot.coord[2],
      } : {
        index: null,
        q: 0,
        r: 0,
        z: 0,
      },
      peer_type: 'server' as const,
      last_heartbeat: Date.now(),
      capabilities: ['mesh'],
      online: peer.online,
    }));

    // Create edges between peers that have slots (neighbors)
    const edges: PeerEdge[] = [];
    const peersWithSlots = snapshot.peers.filter(p => p.slot);
    for (let i = 0; i < peersWithSlots.length; i++) {
      for (let j = i + 1; j < peersWithSlots.length; j++) {
        edges.push({
          from: peersWithSlots[i].id,
          to: peersWithSlots[j].id,
          connection_type: 'neighbor',
          latency_ms: null,
          color: '#4CAF50',
        });
      }
    }

    return {
      mesh_config: {
        width: 10,
        height: 10,
        depth: 10,
        total_slots: 1000,
      },
      nodes,
      edges,
      stats: {
        total_peers: snapshot.peers.length,
        server_nodes: snapshot.peers.length,
        browser_peers: 0,
        mesh_edges: edges.length,
        relay_connections: 0,
        occupancy_percent: (snapshot.slots.length / 1000) * 100,
      },
    };
  };

  /**
   * Recalculate edges between peers with slots
   */
  const recalculateEdges = (nodes: PeerNode[]): PeerEdge[] => {
    const edges: PeerEdge[] = [];
    const peersWithSlots = nodes.filter(n => n.slot.index !== null);
    for (let i = 0; i < peersWithSlots.length; i++) {
      for (let j = i + 1; j < peersWithSlots.length; j++) {
        edges.push({
          from: peersWithSlots[i].id,
          to: peersWithSlots[j].id,
          connection_type: 'neighbor',
          latency_ms: null,
          color: '#4CAF50',
        });
      }
    }
    return edges;
  };

  /**
   * Recalculate stats from nodes
   */
  const recalculateStats = (nodes: PeerNode[], edges: PeerEdge[]): NetworkStats => {
    const serverNodes = nodes.filter(n => n.peer_type === 'server').length;
    const browserPeers = nodes.filter(n => n.peer_type === 'browser').length;
    const slotsOccupied = nodes.filter(n => n.slot.index !== null).length;
    return {
      total_peers: nodes.length,
      server_nodes: serverNodes,
      browser_peers: browserPeers,
      mesh_edges: edges.length,
      relay_connections: edges.filter(e => e.connection_type === 'relay').length,
      occupancy_percent: (slotsOccupied / 1000) * 100,
    };
  };

  /**
   * Handle peer_joined event - add new peer to network map
   */
  const handlePeerJoined = (event: MeshEventPeerJoined) => {
    if (!networkMap.value) return;

    // Check if peer already exists
    const existingIndex = networkMap.value.nodes.findIndex(n => n.id === event.id);
    if (existingIndex !== -1) {
      // Peer already exists, just mark as online
      networkMap.value.nodes[existingIndex].online = true;
      networkMap.value.nodes[existingIndex].last_heartbeat = Date.now();
    } else {
      // Add new peer
      const newPeer: PeerNode = {
        id: event.id,
        label: event.id.substring(0, 12) + '...',
        slot: {
          index: event.slot,
          q: 0,
          r: 0,
          z: 0,
        },
        peer_type: 'server',
        last_heartbeat: Date.now(),
        capabilities: ['mesh'],
        online: true,
      };
      networkMap.value.nodes.push(newPeer);
    }

    // Recalculate edges and stats
    const edges = recalculateEdges(networkMap.value.nodes);
    const stats = recalculateStats(networkMap.value.nodes, edges);

    // Trigger reactivity by creating new object
    networkMap.value = {
      ...networkMap.value,
      nodes: [...networkMap.value.nodes],
      edges,
      stats,
    };
  };

  /**
   * Handle peer_left event - remove peer from network map
   */
  const handlePeerLeft = (event: MeshEventPeerLeft) => {
    if (!networkMap.value) return;

    // Remove peer from nodes
    const filteredNodes = networkMap.value.nodes.filter(n => n.id !== event.id);

    // Recalculate edges and stats
    const edges = recalculateEdges(filteredNodes);
    const stats = recalculateStats(filteredNodes, edges);

    // Trigger reactivity by creating new object
    networkMap.value = {
      ...networkMap.value,
      nodes: filteredNodes,
      edges,
      stats,
    };
  };

  /**
   * Handle slot_claimed event - update peer's slot information
   */
  const handleSlotClaimed = (event: MeshEventSlotClaimed) => {
    if (!networkMap.value) return;

    // Find the peer and update their slot
    const peerIndex = networkMap.value.nodes.findIndex(n => n.id === event.peer_id);
    if (peerIndex !== -1) {
      networkMap.value.nodes[peerIndex].slot = {
        index: event.index,
        q: event.coord[0],
        r: event.coord[1],
        z: event.coord[2],
      };
    } else {
      // Peer not found - add them with the slot
      const newPeer: PeerNode = {
        id: event.peer_id,
        label: event.peer_id.substring(0, 12) + '...',
        slot: {
          index: event.index,
          q: event.coord[0],
          r: event.coord[1],
          z: event.coord[2],
        },
        peer_type: 'server',
        last_heartbeat: Date.now(),
        capabilities: ['mesh'],
        online: true,
      };
      networkMap.value.nodes.push(newPeer);
    }

    // Recalculate edges and stats
    const edges = recalculateEdges(networkMap.value.nodes);
    const stats = recalculateStats(networkMap.value.nodes, edges);

    // Trigger reactivity by creating new object
    networkMap.value = {
      ...networkMap.value,
      nodes: [...networkMap.value.nodes],
      edges,
      stats,
    };
  };

  /**
   * Handle incoming WebSocket message
   */
  const handleMeshEvent = (event: MeshEvent) => {
    lastEvent.value = event;

    switch (event.type) {
      case 'snapshot': {
        // Full snapshot - replace network map
        const snapshot = event as MeshEventSnapshot;
        console.log('[NetworkMap] Snapshot received:', snapshot.peers.length, 'peers,', snapshot.slots.length, 'slots');
        networkMap.value = snapshotToNetworkMap(snapshot);
        initialLoading.value = false; // Initial data received
        break;
      }

      case 'peer_joined': {
        const peerJoined = event as MeshEventPeerJoined;
        console.log('[NetworkMap] Peer joined:', peerJoined.id);
        handlePeerJoined(peerJoined);
        break;
      }

      case 'peer_left': {
        const peerLeft = event as MeshEventPeerLeft;
        console.log('[NetworkMap] Peer left:', peerLeft.id);
        handlePeerLeft(peerLeft);
        break;
      }

      case 'slot_claimed': {
        const slotClaimed = event as MeshEventSlotClaimed;
        console.log('[NetworkMap] Slot claimed:', slotClaimed.index, 'by', slotClaimed.peer_id);
        handleSlotClaimed(slotClaimed);
        break;
      }

      case 'heartbeat':
        // Just a keepalive, no action needed
        break;

      case 'cvdf_new_round': {
        const cvdfRound = event as MeshEventCvdfNewRound;
        console.log('[NetworkMap] CVDF round:', cvdfRound.round, 'weight:', cvdfRound.weight);
        break;
      }

      default:
        console.log('[NetworkMap] Unknown event type:', event.type);
    }
  };

  /**
   * Connect to WebSocket for real-time mesh updates
   */
  const connectWebSocket = () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      return; // Already connected
    }

    const wsUrl = getMeshWsUrl();
    console.log('[NetworkMap] Connecting to mesh WebSocket:', wsUrl);

    try {
      ws = new WebSocket(wsUrl);
    } catch (e) {
      console.error('[NetworkMap] Failed to create WebSocket:', e);
      error.value = e as Error;
      return;
    }

    ws.onopen = () => {
      console.log('[NetworkMap] WebSocket connected');
      connected.value = true;
      isIntentionallyClosed = false;
      error.value = null;
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as MeshEvent;
        handleMeshEvent(data);
      } catch (e) {
        console.error('[NetworkMap] Failed to parse WebSocket message:', e);
      }
    };

    ws.onerror = (event) => {
      console.error('[NetworkMap] WebSocket error:', event);
      error.value = new Error('WebSocket connection error');
    };

    ws.onclose = () => {
      console.log('[NetworkMap] WebSocket closed');
      ws = null;
      connected.value = false;

      // Reconnect after 2 seconds unless intentionally closed
      if (!isIntentionallyClosed) {
        console.log('[NetworkMap] Reconnecting in 2 seconds...');
        reconnectTimeout = setTimeout(() => {
          connectWebSocket();
        }, 2000);
      }
    };
  };

  /**
   * Disconnect WebSocket
   */
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

    connected.value = false;
  };

  // Cleanup on unmount
  onUnmounted(() => {
    disconnectWebSocket();
  });

  return {
    networkMap,
    loading,
    initialLoading,
    error,
    connected,
    lastEvent,
    fetchNetworkMap,
    connectWebSocket,
    disconnectWebSocket,
  };
}
