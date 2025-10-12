import init, { P2pClient, init_panic_hook } from '/@/wasm/lens_v2_wasm.js';

/**
 * WASM-based P2P service for browser participation in the lens-v2 network
 */
export class WasmP2pService {
  private client: P2pClient | null = null;
  private initialized = false;

  constructor(private relayUrl: string = 'ws://localhost:5002/api/v1/relay/ws') {}

  /**
   * Initialize WASM module and connect to relay
   */
  async initialize(): Promise<void> {
    if (this.initialized) return;

    try {
      // Initialize WASM module
      await init();
      init_panic_hook();

      // Create P2P client
      this.client = new P2pClient(this.relayUrl);

      // Register peer discovery callback - auto-connect to discovered browsers
      this.client.on_peer_discovered((peerId: string) => {
        console.log(`🔗 Discovered browser peer: ${peerId}, initiating WebRTC connection...`);
        try {
          if (!this.client) {
            console.error('❌ P2P client is null when trying to connect');
            return;
          }
          this.client.create_peer_connection(peerId);
          console.log(`✅ WebRTC connection initiated to ${peerId}`);
        } catch (error) {
          console.error(`❌ Failed to connect to peer ${peerId}:`, error);
          console.error('Error details:', error);
        }
      });

      // Connect to relay
      this.client.connect();

      this.initialized = true;
      console.log('✅ WASM P2P client initialized and connected to relay');
    } catch (error) {
      console.error('❌ Failed to initialize WASM P2P client:', error);
      throw error;
    }
  }

  /**
   * Delete a release by broadcasting a DeleteRelease transaction via direct P2P
   */
  async deleteRelease(releaseId: string): Promise<void> {
    console.log(`🗑️ deleteRelease called for: ${releaseId}`);

    if (!this.client) {
      const error = new Error('P2P client not initialized. Call initialize() first.');
      console.error('❌', error);
      throw error;
    }

    try {
      // Create DeleteRelease block
      console.log('📦 Creating DeleteRelease block...');
      const block = this.client.create_delete_release_block(releaseId);
      console.log('✅ Block created:', block);

      // Hybrid P2P: Try WebRTC first (browser + Rust nodes), fallback to relay
      console.log('📡 Broadcasting via hybrid P2P (WebRTC + relay fallback)...');
      try {
        await this.client.broadcast_block_direct(block);
        console.log(`✅ DeleteRelease broadcasted via WebRTC for ${releaseId}`);
      } catch (directError) {
        console.log('⚠️ No WebRTC peers available, falling back to relay');
        await this.client.broadcast_block(block);
        console.log(`✅ DeleteRelease broadcasted via relay for ${releaseId}`);
      }
    } catch (error) {
      console.error('❌ Failed to delete release via P2P:', error);
      console.error('Error type:', typeof error);
      console.error('Error details:', error);
      throw error;
    }
  }

  /**
   * Delete a featured release by broadcasting a DeleteFeaturedRelease transaction via direct P2P
   */
  async deleteFeaturedRelease(featuredReleaseId: string): Promise<void> {
    console.log(`🗑️ deleteFeaturedRelease called for: ${featuredReleaseId}`);

    if (!this.client) {
      const error = new Error('P2P client not initialized. Call initialize() first.');
      console.error('❌', error);
      throw error;
    }

    try {
      // Create DeleteFeaturedRelease block
      console.log('📦 Creating DeleteFeaturedRelease block...');
      const block = this.client.create_delete_featured_release_block(featuredReleaseId);
      console.log('✅ Block created:', block);

      // Hybrid P2P: Try WebRTC first, fallback to relay
      console.log('📡 Broadcasting via hybrid P2P (WebRTC + relay fallback)...');
      try {
        await this.client.broadcast_block_direct(block);
        console.log(`✅ DeleteFeaturedRelease broadcasted via WebRTC for ${featuredReleaseId}`);
      } catch (directError) {
        console.log('⚠️ No WebRTC peers available, falling back to relay');
        await this.client.broadcast_block(block);
        console.log(`✅ DeleteFeaturedRelease broadcasted via relay for ${featuredReleaseId}`);
      }
    } catch (error) {
      console.error('❌ Failed to delete featured release via P2P:', error);
      console.error('Error type:', typeof error);
      console.error('Error details:', error);
      throw error;
    }
  }

  /**
   * Connect to a peer via WebRTC DataChannel
   */
  connectToPeer(peerId: string): void {
    if (!this.client) {
      throw new Error('P2P client not initialized. Call initialize() first.');
    }

    this.client.create_peer_connection(peerId);
    console.log(`🔗 Connecting to peer ${peerId} via WebRTC`);
  }

  /**
   * Register callback for received blocks
   */
  onBlockReceived(callback: (block: any) => void): void {
    if (!this.client) {
      throw new Error('P2P client not initialized. Call initialize() first.');
    }

    this.client.on_block_received(callback);
  }

  /**
   * Get peer ID
   */
  getPeerId(): string {
    if (!this.client) {
      throw new Error('P2P client not initialized. Call initialize() first.');
    }

    return this.client.peer_id();
  }

  /**
   * Disconnect from relay
   */
  disconnect(): void {
    if (this.client) {
      this.client.disconnect();
      this.client = null;
      this.initialized = false;
      console.log('🔌 Disconnected from P2P relay');
    }
  }

  /**
   * Check if client is initialized
   */
  isInitialized(): boolean {
    return this.initialized;
  }
}
