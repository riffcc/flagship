// Peerbit Network Adapter
// Wraps the existing WASM P2P service to implement the BaseNetworkAdapter interface

import { WasmP2pService } from '../../lensService/wasmP2pService';
import { BaseNetworkAdapter, NetworkStats, NetworkHealth } from './base';
import { Release } from '../../lensService/httpLensService';

export class PeerbitAdapter implements BaseNetworkAdapter<Release> {
  readonly name = 'peerbit';
  readonly type = 'p2p';
  private wasmService: WasmP2pService;
  private isInitialized = false;

  constructor() {
    this.wasmService = new WasmP2pService();
  }

  async initialize(config: any): Promise<void> {
    if (this.isInitialized) return;

    try {
      await this.wasmService.initialize();
      this.isInitialized = true;
      console.log('PeerbitAdapter initialized successfully');
    } catch (error) {
      console.error('Failed to initialize PeerbitAdapter:', error);
      throw error;
    }
  }

  async disconnect(): Promise<void> {
    if (!this.isInitialized) return;

    try {
      await this.wasmService.disconnect();
      this.isInitialized = false;
      console.log('PeerbitAdapter disconnected');
    } catch (error) {
      console.error('Failed to disconnect PeerbitAdapter:', error);
      throw error;
    }
  }

  async getReleases(options?: any): Promise<Release[]> {
    if (!this.isInitialized) {
      throw new Error('PeerbitAdapter not initialized');
    }

    try {
      // Use the existing WASM service to get releases
      // This will be implemented based on the actual WASM API
      const releases = await this.wasmService.getReleases();
      return releases.map(this.mapWasmReleaseToRelease);
    } catch (error) {
      console.error('Failed to get releases via Peerbit:', error);
      throw error;
    }
  }

  async getRelease(id: string): Promise<Release> {
    if (!this.isInitialized) {
      throw new Error('PeerbitAdapter not initialized');
    }

    try {
      const release = await this.wasmService.getRelease(id);
      return this.mapWasmReleaseToRelease(release);
    } catch (error) {
      console.error(`Failed to get release ${id} via Peerbit:`, error);
      throw error;
    }
  }

  async createRelease(data: any): Promise<Release> {
    if (!this.isInitialized) {
      throw new Error('PeerbitAdapter not initialized');
    }

    try {
      const release = await this.wasmService.createRelease(data);
      return this.mapWasmReleaseToRelease(release);
    } catch (error) {
      console.error('Failed to create release via Peerbit:', error);
      throw error;
    }
  }

  async updateRelease(id: string, data: any): Promise<Release> {
    if (!this.isInitialized) {
      throw new Error('PeerbitAdapter not initialized');
    }

    try {
      const release = await this.wasmService.updateRelease(id, data);
      return this.mapWasmReleaseToRelease(release);
    } catch (error) {
      console.error(`Failed to update release ${id} via Peerbit:`, error);
      throw error;
    }
  }

  async deleteRelease(id: string): Promise<void> {
    if (!this.isInitialized) {
      throw new Error('PeerbitAdapter not initialized');
    }

    try {
      await this.wasmService.deleteRelease(id);
    } catch (error) {
      console.error(`Failed to delete release ${id} via Peerbit:`, error);
      throw error;
    }
  }

  getPeerId(): string {
    return this.wasmService.getPeerId();
  }

  async getConnectedPeers(): Promise<string[]> {
    return this.wasmService.getConnectedPeers();
  }

  async getNetworkStats(): Promise<NetworkStats> {
    const peers = await this.getConnectedPeers();
    return {
      peerCount: peers.length,
      latency: 0, // TODO: Implement latency measurement
      uptime: Date.now() - (this.wasmService.getStartTime() || Date.now()),
      dataSource: 'peerbit'
    };
  }

  async checkHealth(): Promise<NetworkHealth> {
    try {
      const start = Date.now();
      const peers = await this.getConnectedPeers();
      const isHealthy = peers.length > 0;

      return {
        isHealthy,
        responseTime: Date.now() - start,
        error: isHealthy ? undefined : new Error('No connected peers')
      };
    } catch (error) {
      return {
        isHealthy: false,
        responseTime: 0,
        error: error as Error
      };
    }
  }

  private mapWasmReleaseToRelease(wasmRelease: any): Release {
    // Map WASM release format to standard Release format
    return {
      id: wasmRelease.id,
      title: wasmRelease.title,
      description: wasmRelease.description,
      categoryId: wasmRelease.categoryId,
      categorySlug: wasmRelease.categorySlug,
      metadata: wasmRelease.metadata || {},
      createdAt: wasmRelease.createdAt || new Date().toISOString(),
      updatedAt: wasmRelease.updatedAt || new Date().toISOString(),
      // Add other required fields
      ...wasmRelease
    };
  }
}