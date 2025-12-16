// HTTP Network Adapter
// Refactored HTTP adapter that implements the BaseNetworkAdapter interface

import { HttpLensService, Release } from '../../lensService/httpLensService';
import { BaseNetworkAdapter, NetworkStats, NetworkHealth } from './base';

export class HttpAdapter implements BaseNetworkAdapter<Release> {
  readonly name = 'http';
  readonly type = 'rest';
  private service: HttpLensService;
  private isInitialized = true;

  constructor(baseUrl: string = '/api/v1') {
    this.service = new HttpLensService(baseUrl);
  }

  async initialize(): Promise<void> {
    // No initialization needed for HTTP
  }

  async disconnect(): Promise<void> {
    // No disconnection needed for HTTP
  }

  async getReleases(options?: any): Promise<Release[]> {
    try {
      return this.service.getReleases();
    } catch (error) {
      console.error('Failed to get releases via HTTP:', error);
      throw error;
    }
  }

  async getRelease(id: string): Promise<Release> {
    try {
      return this.service.getRelease(id);
    } catch (error) {
      console.error(`Failed to get release ${id} via HTTP:`, error);
      throw error;
    }
  }

  async createRelease(data: any): Promise<Release> {
    try {
      return this.service.createRelease(data);
    } catch (error) {
      console.error('Failed to create release via HTTP:', error);
      throw error;
    }
  }

  async updateRelease(id: string, data: any): Promise<Release> {
    try {
      return this.service.updateRelease(id, data);
    } catch (error) {
      console.error(`Failed to update release ${id} via HTTP:`, error);
      throw error;
    }
  }

  async deleteRelease(id: string): Promise<void> {
    try {
      await this.service.deleteRelease(id);
    } catch (error) {
      console.error(`Failed to delete release ${id} via HTTP:`, error);
      throw error;
    }
  }

  getPeerId(): string {
    // HTTP doesn't have a peer ID, return a constant
    return 'http-client';
  }

  async getConnectedPeers(): Promise<string[]> {
    // HTTP doesn't have peers, return empty array
    return [];
  }

  async getNetworkStats(): Promise<NetworkStats> {
    return {
      peerCount: 0,
      latency: 0,
      uptime: Date.now() - (this.service.getStartTime() || Date.now()),
      dataSource: 'http'
    };
  }

  async checkHealth(): Promise<NetworkHealth> {
    try {
      const start = Date.now();
      const response = await fetch(`${this.service.baseUrl}/health`);

      return {
        isHealthy: response.ok,
        responseTime: Date.now() - start
      };
    } catch (error) {
      return {
        isHealthy: false,
        responseTime: 0,
        error: error as Error
      };
    }
  }
}