/**
 * P2P Composable - Peer-to-Peer content loading via relay
 *
 * Connects to lens-v2-node relay for peer discovery and content exchange.
 * Uses WantList protocol for efficient block synchronization.
 */

import { ref, computed, onUnmounted } from 'vue'

interface WantList {
  generation: number
  full: boolean
  have_ranges: Array<{ start: number; end: number }>
  have_blocks: string[]
  tip_hash: string | null
  need_ranges: Array<{ start: number; end: number }>
  need_blocks: string[]
  rollups: Array<{
    id: number
    blocks: string[]
    priority: number
    estimated_size: number
  }>
  max_rollup_size: number
  max_rollup_bytes: number
}

interface PeerReferral {
  type: 'peer_referral'
  peers: Array<{
    peer_id: string
    latest_height: number
    score: number
  }>
}

export function useP2P() {
  const ws = ref<WebSocket | null>(null)
  const connected = ref(false)
  const peers = ref<Array<{ peer_id: string; latest_height: number; score: number }>>([])
  const generation = ref(0)
  const localBlocks = ref<Set<string>>(new Set())
  const neededBlocks = ref<Set<string>>(new Set())

  const relayUrl = computed(() => {
    const apiUrl = import.meta.env.VITE_API_URL || 'http://127.0.0.1:5002'
    const wsUrl = apiUrl.replace('http://', 'ws://').replace('https://', 'wss://')
    return `${wsUrl}/api/v1/relay/ws`
  })

  const connect = () => {
    if (ws.value) {
      console.warn('[P2P] Already connected')
      return
    }

    console.log('[P2P] Connecting to relay:', relayUrl.value)
    ws.value = new WebSocket(relayUrl.value)

    ws.value.onopen = () => {
      console.log('[P2P] Connected to relay')
      connected.value = true

      // Send initial WantList
      sendWantList()
    }

    ws.value.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as PeerReferral
        console.log('[P2P] Received message:', data)

        if (data.type === 'peer_referral') {
          peers.value = data.peers
          console.log(`[P2P] Received ${data.peers.length} peer referrals`)

          // TODO: Establish direct connections to peers
          // For now, just log them
          data.peers.forEach(peer => {
            console.log(`[P2P] Peer: ${peer.peer_id} (height=${peer.latest_height}, score=${peer.score})`)
          })
        }
      } catch (e) {
        console.error('[P2P] Failed to parse message:', e)
      }
    }

    ws.value.onerror = (error) => {
      console.error('[P2P] WebSocket error:', error)
    }

    ws.value.onclose = () => {
      console.log('[P2P] Disconnected from relay')
      connected.value = false
      ws.value = null
    }
  }

  const disconnect = () => {
    if (ws.value) {
      ws.value.close()
      ws.value = null
      connected.value = false
    }
  }

  const sendWantList = () => {
    if (!ws.value || ws.value.readyState !== WebSocket.OPEN) {
      console.warn('[P2P] Cannot send WantList: not connected')
      return
    }

    generation.value++

    const wantlist: WantList = {
      generation: generation.value,
      full: true,
      have_ranges: [],
      have_blocks: Array.from(localBlocks.value),
      tip_hash: null,
      need_ranges: [],
      need_blocks: Array.from(neededBlocks.value),
      rollups: [],
      max_rollup_size: 1000,
      max_rollup_bytes: 100 * 1024 * 1024, // 100 MB
    }

    console.log('[P2P] Sending WantList:', {
      generation: wantlist.generation,
      have: wantlist.have_blocks.length,
      need: wantlist.need_blocks.length,
    })

    ws.value.send(JSON.stringify(wantlist))
  }

  const requestBlock = (blockId: string) => {
    neededBlocks.value.add(blockId)
    console.log(`[P2P] Requesting block: ${blockId}`)
    sendWantList()
  }

  const addLocalBlock = (blockId: string) => {
    localBlocks.value.add(blockId)
    neededBlocks.value.delete(blockId)
    console.log(`[P2P] Added local block: ${blockId}`)
    sendWantList()
  }

  // Cleanup on unmount
  onUnmounted(() => {
    disconnect()
  })

  return {
    // State
    connected,
    peers,
    localBlocks: computed(() => Array.from(localBlocks.value)),
    neededBlocks: computed(() => Array.from(neededBlocks.value)),

    // Methods
    connect,
    disconnect,
    requestBlock,
    addLocalBlock,
    sendWantList,
  }
}
