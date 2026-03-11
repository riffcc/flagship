// @ts-nocheck
// Peerbit Network Adapter (DISABLED)
// WASM P2P has been removed - this adapter is a stub that always fails
// All network operations should go through HTTP or Citadel adapters

import { BaseNetworkAdapter, NetworkStats, NetworkHealth } from './base';
import { Release } from '../../lensService/httpLensService';

export class PeerbitAdapter implements BaseNetworkAdapter<Release> {
  readonly name = 'peerbit';
  readonly type = 'p2p';

  async initialize(_config: any): Promise<void> {
    console.warn('PeerbitAdapter is disabled - WASM P2P has been removed');
    throw new Error('PeerbitAdapter is disabled');
  }

  async disconnect(): Promise<void> {
    // No-op
  }

  async getReleases(_options?: any): Promise<Release[]> {
    throw new Error('PeerbitAdapter is disabled');
  }

  async getRelease(_id: string): Promise<Release> {
    throw new Error('PeerbitAdapter is disabled');
  }

  async createRelease(_data: any): Promise<Release> {
    throw new Error('PeerbitAdapter is disabled');
  }

  async updateRelease(_id: string, _data: any): Promise<Release> {
    throw new Error('PeerbitAdapter is disabled');
  }

  async deleteRelease(_id: string): Promise<void> {
    throw new Error('PeerbitAdapter is disabled');
  }

  getPeerId(): string {
    return 'disabled';
  }

  async getConnectedPeers(): Promise<string[]> {
    return [];
  }

  async getNetworkStats(): Promise<NetworkStats> {
    return {
      peerCount: 0,
      latency: 0,
      uptime: 0,
      dataSource: 'peerbit'
    };
  }

  async checkHealth(): Promise<NetworkHealth> {
    return {
      isHealthy: false,
      responseTime: 0,
      error: new Error('PeerbitAdapter is disabled')
    };
  }
}
