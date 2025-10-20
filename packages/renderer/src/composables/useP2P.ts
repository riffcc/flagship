/**
 * P2P Composable - Peer-to-Peer content loading via relay
 *
 * Connects to lens-v2-node relay for peer discovery and content exchange.
 * Uses WantList protocol for efficient block synchronization.
 * Establishes WebRTC connections for direct P2P communication.
 */

import { ref, computed, onUnmounted } from 'vue'
import { getApiUrl } from '../utils/runtimeConfig'

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
  your_peer_id?: string
  peers: Array<{
    peer_id: string
    latest_height: number
    score: number
  }>
}

interface SignalingMessage {
  type: 'offer' | 'answer' | 'ice_candidate'
  from: string
  to: string
  sdp?: string
  candidate?: string
  sdp_mid?: string | null
  sdp_m_line_index?: number | null
}

interface Block {
  id: string
  data: ArrayBuffer
  height?: number
}

interface RollupRequest {
  rollup_id: number
  blocks: string[]
  priority: number
}

interface RollupResponse {
  rollup_id: number
  blocks: Block[]
}

interface PeerConnection {
  peer_id: string
  pc: RTCPeerConnection
  dataChannel: RTCDataChannel | null
  connected: boolean
}

export function useP2P() {
  const ws = ref<WebSocket | null>(null)
  const connected = ref(false)
  const peers = ref<Array<{ peer_id: string; latest_height: number; score: number }>>([])
  const generation = ref(0)
  const localBlocks = ref<Set<string>>(new Set())
  const neededBlocks = ref<Set<string>>(new Set())

  // Block storage - IndexedDB for persistence
  const blockStore = ref<Map<string, Block>>(new Map())
  const pendingRollups = ref<Map<number, RollupRequest>>(new Map())

  // WebRTC peer connections
  const myPeerId = ref<string>('')
  const peerConnections = ref<Map<string, PeerConnection>>(new Map())
  const directPeersConnected = computed(() =>
    Array.from(peerConnections.value.values()).filter(p => p.connected).length
  )

  const relayUrl = computed(() => {
    const apiUrl = getApiUrl()
    const wsUrl = apiUrl.replace('http://', 'ws://').replace('https://', 'wss://')
    // Remove trailing /api/v1 if present, then add the relay path
    const baseUrl = wsUrl.replace(/\/api\/v1\/?$/, '')
    return `${baseUrl}/ws`
  })

  // ICE servers for NAT traversal (using Google's public STUN server)
  const iceServers = [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
  ]

  // Create WebRTC peer connection
  const createPeerConnection = (peerId: string): PeerConnection => {
    console.log(`[P2P] Creating WebRTC connection to ${peerId}`)

    const pc = new RTCPeerConnection({ iceServers })
    const peerConn: PeerConnection = {
      peer_id: peerId,
      pc,
      dataChannel: null,
      connected: false,
    }

    // Handle ICE candidates
    pc.onicecandidate = (event) => {
      if (event.candidate && ws.value) {
        console.log(`[P2P] Sending ICE candidate to ${peerId}`)
        const msg: SignalingMessage = {
          type: 'ice_candidate',
          from: myPeerId.value,
          to: peerId,
          candidate: JSON.stringify(event.candidate.toJSON()),
          sdp_mid: event.candidate.sdpMid,
          sdp_m_line_index: event.candidate.sdpMLineIndex,
        }
        ws.value.send(JSON.stringify(msg))
      }
    }

    // Handle connection state changes
    pc.onconnectionstatechange = () => {
      console.log(`[P2P] Connection to ${peerId}: ${pc.connectionState}`)
      if (pc.connectionState === 'connected') {
        peerConn.connected = true
        console.log(`[P2P] ✅ Direct P2P connection established with ${peerId}`)
      } else if (pc.connectionState === 'disconnected' || pc.connectionState === 'failed') {
        peerConn.connected = false
        console.log(`[P2P] ❌ Lost connection to ${peerId}`)
      }
    }

    // Handle incoming data channel
    pc.ondatachannel = (event) => {
      console.log(`[P2P] Received data channel from ${peerId}`)
      const channel = event.channel
      peerConn.dataChannel = channel

      channel.onopen = () => {
        console.log(`[P2P] Data channel with ${peerId} opened`)
        peerConn.connected = true
      }

      channel.onmessage = async (msgEvent) => {
        console.log(`[P2P] Received message from ${peerId}`, msgEvent.data)

        // Parse incoming message
        let message: any
        if (msgEvent.data instanceof ArrayBuffer || msgEvent.data instanceof Blob) {
          const buffer = msgEvent.data instanceof Blob
            ? await msgEvent.data.arrayBuffer()
            : msgEvent.data

          try {
            const text = new TextDecoder().decode(buffer)
            message = JSON.parse(text)
          } catch (e) {
            console.error(`[P2P] Failed to parse message from ${peerId}:`, e)
            return
          }
        } else if (typeof msgEvent.data === 'string') {
          try {
            message = JSON.parse(msgEvent.data)
          } catch (e) {
            console.error(`[P2P] Failed to parse message from ${peerId}:`, e)
            return
          }
        }

        // Handle rollup response (blocks being sent to us)
        if (message.rollup_id && message.blocks && Array.isArray(message.blocks)) {
          await handleRollupResponse(message as RollupResponse, peerId)
        }
        // Handle rollup request (peer requesting blocks from us)
        else if (message.rollup_id && message.blocks && Array.isArray(message.blocks) && message.priority !== undefined) {
          await handleRollupRequest(message as RollupRequest, peerId)
        }
      }
    }

    peerConnections.value.set(peerId, peerConn)
    return peerConn
  }

  // Send WebRTC offer to peer
  const sendOffer = async (peerId: string) => {
    let peerConn = peerConnections.value.get(peerId)
    if (!peerConn) {
      peerConn = createPeerConnection(peerId)
    }

    // Create data channel for block exchange
    const dataChannel = peerConn.pc.createDataChannel('blocks')
    peerConn.dataChannel = dataChannel

    dataChannel.onopen = () => {
      console.log(`[P2P] Data channel opened with ${peerId}`)
      peerConn.connected = true
    }

    dataChannel.onmessage = async (event) => {
      console.log(`[P2P] Received message from ${peerId}`, event.data)

      // Parse incoming message
      let message: any
      if (event.data instanceof ArrayBuffer || event.data instanceof Blob) {
        const buffer = event.data instanceof Blob
          ? await event.data.arrayBuffer()
          : event.data

        try {
          const text = new TextDecoder().decode(buffer)
          message = JSON.parse(text)
        } catch (e) {
          console.error(`[P2P] Failed to parse message from ${peerId}:`, e)
          return
        }
      } else if (typeof event.data === 'string') {
        try {
          message = JSON.parse(event.data)
        } catch (e) {
          console.error(`[P2P] Failed to parse message from ${peerId}:`, e)
          return
        }
      }

      // Handle rollup response (blocks being sent to us)
      if (message.rollup_id && message.blocks && Array.isArray(message.blocks)) {
        await handleRollupResponse(message as RollupResponse, peerId)
      }
      // Handle rollup request (peer requesting blocks from us)
      else if (message.rollup_id && message.blocks && Array.isArray(message.blocks) && message.priority !== undefined) {
        await handleRollupRequest(message as RollupRequest, peerId)
      }
    }

    // Create and send offer
    const offer = await peerConn.pc.createOffer()
    await peerConn.pc.setLocalDescription(offer)

    if (ws.value && offer.sdp) {
      const msg: SignalingMessage = {
        type: 'offer',
        from: myPeerId.value,
        to: peerId,
        sdp: offer.sdp,
      }
      console.log(`[P2P] Sending offer to ${peerId}`)
      ws.value.send(JSON.stringify(msg))
    }
  }

  // Handle incoming WebRTC offer
  const handleOffer = async (from: string, sdp: string) => {
    console.log(`[P2P] Received offer from ${from}`)

    let peerConn = peerConnections.value.get(from)
    if (!peerConn) {
      peerConn = createPeerConnection(from)
    }

    await peerConn.pc.setRemoteDescription(new RTCSessionDescription({
      type: 'offer',
      sdp,
    }))

    const answer = await peerConn.pc.createAnswer()
    await peerConn.pc.setLocalDescription(answer)

    if (ws.value && answer.sdp) {
      const msg: SignalingMessage = {
        type: 'answer',
        from: myPeerId.value,
        to: from,
        sdp: answer.sdp,
      }
      console.log(`[P2P] Sending answer to ${from}`)
      ws.value.send(JSON.stringify(msg))
    }
  }

  // Handle incoming WebRTC answer
  const handleAnswer = async (from: string, sdp: string) => {
    console.log(`[P2P] Received answer from ${from}`)

    const peerConn = peerConnections.value.get(from)
    if (peerConn) {
      await peerConn.pc.setRemoteDescription(new RTCSessionDescription({
        type: 'answer',
        sdp,
      }))
    }
  }

  // Handle incoming ICE candidate
  const handleIceCandidate = async (from: string, candidate: string) => {
    console.log(`[P2P] Received ICE candidate from ${from}`)

    const peerConn = peerConnections.value.get(from)
    if (peerConn) {
      try {
        const candidateObj = JSON.parse(candidate)
        await peerConn.pc.addIceCandidate(new RTCIceCandidate(candidateObj))
      } catch (e) {
        console.error(`[P2P] Failed to add ICE candidate:`, e)
      }
    }
  }

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

    ws.value.onmessage = async (event) => {
      try {
        const data = JSON.parse(event.data)
        console.log('[P2P] Received message:', data)

        // Handle peer referrals
        if (data.type === 'peer_referral') {
          // Store our peer ID if provided
          if (data.your_peer_id && !myPeerId.value) {
            myPeerId.value = data.your_peer_id
            console.log(`[P2P] Our peer ID: ${myPeerId.value}`)
          }

          peers.value = data.peers
          console.log(`[P2P] Received ${data.peers.length} peer referrals`)

          // Establish WebRTC connections to discovered peers
          data.peers.forEach(peer => {
            console.log(`[P2P] Peer: ${peer.peer_id} (height=${peer.latest_height}, score=${peer.score})`)

            // Only initiate connection if we don't already have one
            if (!peerConnections.value.has(peer.peer_id)) {
              // Use lexicographic ordering to determine who initiates
              // This prevents both peers from initiating simultaneously
              if (myPeerId.value < peer.peer_id) {
                console.log(`[P2P] Initiating WebRTC connection to ${peer.peer_id}`)
                sendOffer(peer.peer_id).catch(e => {
                  console.error(`[P2P] Failed to send offer to ${peer.peer_id}:`, e)
                })
              } else {
                console.log(`[P2P] Waiting for offer from ${peer.peer_id}`)
              }
            }
          })
        }
        // Handle WebRTC signaling messages
        else if (data.type === 'offer') {
          await handleOffer(data.from, data.sdp)
        }
        else if (data.type === 'answer') {
          await handleAnswer(data.from, data.sdp)
        }
        else if (data.type === 'ice_candidate') {
          await handleIceCandidate(data.from, data.candidate)
        }
      } catch (e) {
        console.error('[P2P] Failed to handle message:', e)
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
    // Close all WebRTC peer connections
    peerConnections.value.forEach((peerConn) => {
      if (peerConn.dataChannel) {
        peerConn.dataChannel.close()
      }
      peerConn.pc.close()
    })
    peerConnections.value.clear()

    // Close WebSocket
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

  const addLocalBlock = (blockId: string, data?: ArrayBuffer) => {
    localBlocks.value.add(blockId)
    neededBlocks.value.delete(blockId)

    if (data) {
      blockStore.value.set(blockId, { id: blockId, data })
    }

    console.log(`[P2P] Added local block: ${blockId}`)
    sendWantList()
  }

  // Handle incoming rollup request (peer wants blocks from us)
  const handleRollupRequest = async (request: RollupRequest, fromPeer: string) => {
    console.log(`[P2P] Received rollup request ${request.rollup_id} from ${fromPeer} for ${request.blocks.length} blocks`)

    const peerConn = peerConnections.value.get(fromPeer)
    if (!peerConn || !peerConn.dataChannel || !peerConn.connected) {
      console.warn(`[P2P] Cannot respond to ${fromPeer}: not connected`)
      return
    }

    // Collect blocks we have
    const blocksToSend: Block[] = []
    for (const blockId of request.blocks) {
      const block = blockStore.value.get(blockId)
      if (block) {
        blocksToSend.push(block)
      }
    }

    if (blocksToSend.length === 0) {
      console.log(`[P2P] No blocks to send for rollup ${request.rollup_id}`)
      return
    }

    // Create rollup response
    const response: RollupResponse = {
      rollup_id: request.rollup_id,
      blocks: blocksToSend,
    }

    // Send response
    const message = JSON.stringify(response)
    console.log(`[P2P] Sending ${blocksToSend.length} blocks to ${fromPeer} for rollup ${request.rollup_id}`)

    try {
      peerConn.dataChannel.send(message)
    } catch (e) {
      console.error(`[P2P] Failed to send rollup response to ${fromPeer}:`, e)
    }
  }

  // Handle incoming rollup response (peer sent us blocks)
  const handleRollupResponse = async (response: RollupResponse, fromPeer: string) => {
    console.log(`[P2P] Received rollup ${response.rollup_id} from ${fromPeer} with ${response.blocks.length} blocks`)

    let storedCount = 0
    for (const block of response.blocks) {
      try {
        // Store block in local cache
        blockStore.value.set(block.id, block)

        // Update local blocks set
        localBlocks.value.add(block.id)
        neededBlocks.value.delete(block.id)

        storedCount++
        console.log(`[P2P] Stored block ${block.id} (${block.data.byteLength} bytes)`)
      } catch (e) {
        console.error(`[P2P] Failed to store block ${block.id}:`, e)
      }
    }

    console.log(`[P2P] Stored ${storedCount}/${response.blocks.length} blocks from rollup ${response.rollup_id}`)

    // Remove from pending rollups
    pendingRollups.value.delete(response.rollup_id)

    // Update WantList
    if (storedCount > 0) {
      sendWantList()
    }
  }

  // Request blocks from a specific peer via rollup
  const requestBlocksFromPeer = async (peerId: string, blockIds: string[]) => {
    const peerConn = peerConnections.value.get(peerId)
    if (!peerConn || !peerConn.dataChannel || !peerConn.connected) {
      console.warn(`[P2P] Cannot request blocks from ${peerId}: not connected`)
      return
    }

    // Filter out blocks we already have
    const needed = blockIds.filter(id => !localBlocks.value.has(id))
    if (needed.length === 0) {
      console.log(`[P2P] No blocks needed from ${peerId}`)
      return
    }

    // Create rollup request
    const rollup: RollupRequest = {
      rollup_id: Math.floor(Math.random() * 0xFFFFFFFF),
      blocks: needed,
      priority: 128,
    }

    // Track pending rollup
    pendingRollups.value.set(rollup.rollup_id, rollup)

    // Send via data channel
    const message = JSON.stringify(rollup)
    console.log(`[P2P] Sending rollup request ${rollup.rollup_id} to ${peerId} (${needed.length} blocks)`)

    try {
      peerConn.dataChannel.send(message)
    } catch (e) {
      console.error(`[P2P] Failed to send rollup request to ${peerId}:`, e)
      pendingRollups.value.delete(rollup.rollup_id)
    }
  }

  // Get a block from local storage
  const getBlock = (blockId: string): Block | undefined => {
    return blockStore.value.get(blockId)
  }

  // Check if we have a block
  const hasBlock = (blockId: string): boolean => {
    return localBlocks.value.has(blockId)
  }

  // Collect and send browser-discovered peers to relay
  const sendPeerAnnouncement = () => {
    if (!ws.value || ws.value.readyState !== WebSocket.OPEN) {
      console.warn('[P2P] Cannot send peer announcement: not connected')
      return
    }

    // Collect connected peers
    const connectedPeers = Array.from(peerConnections.value.values())
      .filter(p => p.connected && p.peer_id !== myPeerId.value)
      .map(p => ({
        peer_id: p.peer_id,
        connected: p.connected,
        connection_quality: p.dataChannel?.readyState === 'open' ? 'good' : 'poor',
      }))

    if (connectedPeers.length === 0) {
      console.log('[P2P] No connected peers to announce')
      return
    }

    // Send browser peer announcement to relay
    const announcement = {
      type: 'browser_peer_announcement',
      peers: connectedPeers,
    }

    console.log(`[P2P] 🌉 Announcing ${connectedPeers.length} discovered peers to relay (helping nodes find each other!)`)
    ws.value.send(JSON.stringify(announcement))
  }

  // Periodically announce discovered peers (every 30 seconds)
  let peerAnnouncementInterval: number | undefined
  const startPeerAnnouncements = () => {
    // Clear existing interval if any
    if (peerAnnouncementInterval) {
      clearInterval(peerAnnouncementInterval)
    }

    // Send initial announcement after 5 seconds (give time for connections to establish)
    setTimeout(() => {
      sendPeerAnnouncement()
    }, 5000)

    // Then announce every 30 seconds
    peerAnnouncementInterval = setInterval(() => {
      sendPeerAnnouncement()
    }, 30000) as unknown as number
  }

  // Start peer announcements when connected
  const originalConnect = connect
  const enhancedConnect = () => {
    originalConnect()
    // Wait for connection to establish, then start announcements
    const checkConnection = setInterval(() => {
      if (connected.value) {
        clearInterval(checkConnection)
        startPeerAnnouncements()
      }
    }, 100)
  }

  // Stop peer announcements on disconnect
  const originalDisconnect = disconnect
  const enhancedDisconnect = () => {
    if (peerAnnouncementInterval) {
      clearInterval(peerAnnouncementInterval)
      peerAnnouncementInterval = undefined
    }
    originalDisconnect()
  }

  // Cleanup on unmount
  onUnmounted(() => {
    enhancedDisconnect()
  })

  return {
    // State
    connected,
    peers,
    localBlocks: computed(() => Array.from(localBlocks.value)),
    neededBlocks: computed(() => Array.from(neededBlocks.value)),
    blockStore: computed(() => blockStore.value),

    // WebRTC State
    myPeerId: computed(() => myPeerId.value),
    directPeersConnected,
    peerConnections: computed(() => Array.from(peerConnections.value.values())),

    // Methods (using enhanced versions with peer announcements)
    connect: enhancedConnect,
    disconnect: enhancedDisconnect,
    requestBlock,
    addLocalBlock,
    sendWantList,

    // BoTG Block Exchange
    requestBlocksFromPeer,
    getBlock,
    hasBlock,

    // Peer Discovery Bridge (browsers help nodes find each other)
    sendPeerAnnouncement,
  }
}
