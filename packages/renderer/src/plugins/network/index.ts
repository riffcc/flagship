// @ts-nocheck
// Unified Network Service
// Main entry point for the hybrid network service

import { NetworkRouter } from './router';
import { NetworkConfig, defaultNetworkConfig } from './config';
import { CitadelAdapter } from './adapters/citadel';
import { HttpAdapter } from './adapters/http';
import { App } from 'vue';

export class UnifiedNetworkService {
  private router: NetworkRouter;

  constructor(config: Partial<NetworkConfig> = {}) {
    // Merge with default config
    const finalConfig = { ...defaultNetworkConfig, ...config };
    this.router = new NetworkRouter(finalConfig);

    // Register adapters (peerbit removed - using citadel + http only)
    this.router.registerAdapter('citadel', new CitadelAdapter());
    this.router.registerAdapter('http', new HttpAdapter(finalConfig.http.baseUrl));
  }

  async initialize(): Promise<void> {
    console.log('Initializing UnifiedNetworkService...');
    await this.router.initialize();
  }

  // Data operations - proxy to router
  async getReleases(options?: any): Promise<any[]> {
    return this.router.getReleases(options);
  }

  async getRelease(id: string): Promise<any> {
    return this.router.getRelease(id);
  }

  async createRelease(data: any): Promise<any> {
    return this.router.createRelease(data);
  }

  async updateRelease(id: string, data: any): Promise<any> {
    return this.router.updateRelease(id, data);
  }

  async deleteRelease(id: string): Promise<void> {
    return this.router.deleteRelease(id);
  }

  // Network operations
  async getNetworkStats(): Promise<Record<string, any>> {
    return this.router.getNetworkStats();
  }

  async checkHealth(): Promise<Record<string, any>> {
    return this.router.checkHealth();
  }

  getTelemetry(): any {
    return this.router.getTelemetry();
  }

  // Configuration access
  getConfig(): NetworkConfig {
    return this.router['config'];
  }
}

// Vue plugin for easy integration
export default {
  install: (app: App, config?: Partial<NetworkConfig>) => {
    const networkService = new UnifiedNetworkService(config);

    // Provide the service for injection
    app.provide('networkService', networkService);

    // Make it available globally
    app.config.globalProperties.$networkService = networkService;

    console.log('UnifiedNetworkService plugin installed');
  }
};

// Helper function for composable
export function useNetworkService(): UnifiedNetworkService {
  // This will be implemented in the composable
  throw new Error('useNetworkService must be used within a Vue component setup function');
}
