/**
 * P2P Streaming Composable - MediaSource-based streaming from P2P blocks
 *
 * Seamlessly plays media content delivered via P2P block exchange.
 * Uses MediaSource API to enable progressive playback as blocks arrive.
 */

import { ref, computed, watch, onUnmounted } from 'vue'
import { useP2P } from './useP2P'

interface StreamingSession {
  contentId: string
  mediaSource: MediaSource
  sourceBuffer: SourceBuffer | null
  mimeType: string
  blocks: string[] // Ordered list of block IDs for this content
  receivedBlocks: Set<string>
  bufferedUntil: number // Byte offset buffered so far
  isReady: boolean
  url: string | null
}

export function useP2PStreaming() {
  const { getBlock, hasBlock, requestBlocksFromPeer, peerConnections } = useP2P()

  const activeStreams = ref<Map<string, StreamingSession>>(new Map())

  /**
   * Start streaming content from P2P
   * @param contentId - Unique content identifier
   * @param blockIds - Ordered list of block IDs making up this content
   * @param mimeType - MIME type for MediaSource (e.g., 'video/webm; codecs="vp9"')
   */
  const startStream = (contentId: string, blockIds: string[], mimeType: string): string | null => {
    // Check if already streaming
    if (activeStreams.value.has(contentId)) {
      console.warn(`[P2PStream] Already streaming ${contentId}`)
      const session = activeStreams.value.get(contentId)!
      return session.url
    }

    console.log(`[P2PStream] Starting stream for ${contentId} (${blockIds.length} blocks, ${mimeType})`)

    // Create MediaSource
    const mediaSource = new MediaSource()
    const url = URL.createObjectURL(mediaSource)

    const session: StreamingSession = {
      contentId,
      mediaSource,
      sourceBuffer: null,
      mimeType,
      blocks: blockIds,
      receivedBlocks: new Set(),
      bufferedUntil: 0,
      isReady: false,
      url,
    }

    // Handle MediaSource open
    mediaSource.addEventListener('sourceopen', () => {
      console.log(`[P2PStream] MediaSource opened for ${contentId}`)

      try {
        // Create SourceBuffer
        const sourceBuffer = mediaSource.addSourceBuffer(mimeType)
        session.sourceBuffer = sourceBuffer
        session.isReady = true

        // Start requesting blocks from peers
        requestNextBlocks(session)

        // Handle buffer updates
        sourceBuffer.addEventListener('updateend', () => {
          // Request more blocks if needed
          requestNextBlocks(session)
        })
      } catch (e) {
        console.error(`[P2PStream] Failed to create SourceBuffer:`, e)
      }
    })

    activeStreams.value.set(contentId, session)
    return url
  }

  /**
   * Request the next batch of blocks for streaming
   */
  const requestNextBlocks = (session: StreamingSession) => {
    if (!session.sourceBuffer || session.sourceBuffer.updating) {
      return
    }

    // Find blocks we need but haven't received yet
    const neededBlocks: string[] = []
    const maxBlocksPerRequest = 10 // Request in batches

    for (const blockId of session.blocks) {
      if (!session.receivedBlocks.has(blockId)) {
        neededBlocks.push(blockId)

        if (neededBlocks.length >= maxBlocksPerRequest) {
          break
        }
      }
    }

    if (neededBlocks.length === 0) {
      // All blocks received, end stream
      if (session.mediaSource.readyState === 'open') {
        console.log(`[P2PStream] All blocks received for ${session.contentId}, ending stream`)
        session.mediaSource.endOfStream()
      }
      return
    }

    // Check which blocks we already have locally
    const localBlocks: string[] = []
    const remoteBlocks: string[] = []

    for (const blockId of neededBlocks) {
      if (hasBlock(blockId)) {
        localBlocks.push(blockId)
      } else {
        remoteBlocks.push(blockId)
      }
    }

    // Append local blocks immediately
    if (localBlocks.length > 0) {
      appendBlocks(session, localBlocks)
    }

    // Request remote blocks from connected peers
    if (remoteBlocks.length > 0) {
      const peers = peerConnections.value.filter(p => p.connected)
      if (peers.length > 0) {
        // Round-robin across peers
        const peerIndex = Math.floor(Math.random() * peers.length)
        const peer = peers[peerIndex]

        console.log(`[P2PStream] Requesting ${remoteBlocks.length} blocks from ${peer.peer_id}`)
        requestBlocksFromPeer(peer.peer_id, remoteBlocks)
      } else {
        console.warn(`[P2PStream] No connected peers to request blocks from`)
      }
    }
  }

  /**
   * Append blocks to the SourceBuffer
   */
  const appendBlocks = (session: StreamingSession, blockIds: string[]) => {
    if (!session.sourceBuffer || session.sourceBuffer.updating) {
      return
    }

    // Get block data
    const blockData: ArrayBuffer[] = []
    for (const blockId of blockIds) {
      const block = getBlock(blockId)
      if (block) {
        blockData.push(block.data)
        session.receivedBlocks.add(blockId)
      }
    }

    if (blockData.length === 0) {
      return
    }

    // Concatenate block data
    const totalSize = blockData.reduce((sum, buf) => sum + buf.byteLength, 0)
    const combined = new Uint8Array(totalSize)
    let offset = 0

    for (const buf of blockData) {
      combined.set(new Uint8Array(buf), offset)
      offset += buf.byteLength
    }

    console.log(`[P2PStream] Appending ${blockData.length} blocks (${totalSize} bytes) to ${session.contentId}`)

    try {
      session.sourceBuffer.appendBuffer(combined)
      session.bufferedUntil += totalSize
    } catch (e) {
      console.error(`[P2PStream] Failed to append buffer:`, e)
    }
  }

  /**
   * Watch for new blocks arriving and append them to active streams
   */
  watch(
    () => peerConnections.value.length,
    () => {
      // When blocks arrive, check if any active streams need them
      for (const session of activeStreams.value.values()) {
        if (session.isReady) {
          requestNextBlocks(session)
        }
      }
    },
    { deep: true }
  )

  /**
   * Stop streaming and clean up
   */
  const stopStream = (contentId: string) => {
    const session = activeStreams.value.get(contentId)
    if (!session) {
      return
    }

    console.log(`[P2PStream] Stopping stream ${contentId}`)

    // Clean up MediaSource
    if (session.url) {
      URL.revokeObjectURL(session.url)
    }

    if (session.mediaSource.readyState === 'open') {
      session.mediaSource.endOfStream()
    }

    activeStreams.value.delete(contentId)
  }

  /**
   * Get stream URL for a content ID
   */
  const getStreamUrl = (contentId: string): string | null => {
    const session = activeStreams.value.get(contentId)
    return session?.url || null
  }

  /**
   * Check if content is currently streaming
   */
  const isStreaming = (contentId: string): boolean => {
    return activeStreams.value.has(contentId)
  }

  /**
   * Get streaming progress for content
   */
  const getProgress = (contentId: string): number => {
    const session = activeStreams.value.get(contentId)
    if (!session) {
      return 0
    }

    const receivedCount = session.receivedBlocks.size
    const totalCount = session.blocks.length

    return totalCount > 0 ? receivedCount / totalCount : 0
  }

  // Cleanup on unmount
  onUnmounted(() => {
    for (const contentId of activeStreams.value.keys()) {
      stopStream(contentId)
    }
  })

  return {
    // State
    activeStreams: computed(() => Array.from(activeStreams.value.keys())),

    // Methods
    startStream,
    stopStream,
    getStreamUrl,
    isStreaming,
    getProgress,
  }
}
