// @ts-nocheck
// Network Router
// Intelligent routing between different network adapters with fallback strategy

import { BaseNetworkAdapter } from './adapters/base';
import { NetworkConfig } from './config';
import { NetworkTelemetry } from './telemetry';

export class NetworkRouter {
  private adapters: Map<string, BaseNetworkAdapter<any>>;
  private config: NetworkConfig;
  private telemetry: NetworkTelemetry;

  constructor(config: NetworkConfig) {
    this.config = config;
    this.adapters = new Map();
    this.telemetry = new NetworkTelemetry();
  }

  registerAdapter(name: string, adapter: BaseNetworkAdapter<any>): void {
    this.adapters.set(name, adapter);
  }

  async initialize(): Promise<void> {
    const initPromises = [];

    for (const [name, adapter] of this.adapters) {
      if (this.isAdapterEnabled(name)) {
        console.log(`Initializing ${name} adapter...`);
        initPromises.push(
          adapter.initialize(this.getAdapterConfig(name)).catch(error => {
            console.error(`Failed to initialize ${name} adapter:`, error);
            return null;
          })
        );
      } else {
        console.log(`${name} adapter disabled by configuration`);
      }
    }

    await Promise.all(initPromises);
    console.log('Network router initialization complete');
  }

  async getReleases(options?: any): Promise<any[]> {
    return this.routeRequest('getReleases', [options]);
  }

  async getRelease(id: string): Promise<any> {
    return this.routeRequest('getRelease', [id]);
  }

  async createRelease(data: any): Promise<any> {
    return this.routeRequest('createRelease', [data]);
  }

  async updateRelease(id: string, data: any): Promise<any> {
    return this.routeRequest('updateRelease', [id, data]);
  }

  async deleteRelease(id: string): Promise<void> {
    return this.routeRequest('deleteRelease', [id]);
  }

  private async routeRequest(method: string, args: any[]): Promise<any> {
    const { order, timeout, retryCount } = this.config.fallback;
    let lastError: Error | undefined;
    let attempt = 0;

    for (const adapterName of order) {
      const adapter = this.adapters.get(adapterName);

      if (!adapter || !this.isAdapterEnabled(adapterName)) {
        console.log(`Skipping ${adapterName} - disabled or not available`);
        continue;
      }

      attempt++;
      console.log(`Attempt ${attempt}: Trying ${adapterName} adapter for ${method}`);

      try {
        const startTime = Date.now();

        // Apply timeout to the request
        let result: any;
        if (timeout > 0) {
          const timeoutPromise = new Promise((_, reject) =>
            setTimeout(() => reject(new Error(`${adapterName} timeout after ${timeout}ms`)), timeout)
          );

          result = await Promise.race([
            (adapter as any)[method](...args),
            timeoutPromise
          ]);
        } else {
          result = await (adapter as any)[method](...args);
        }

        const duration = Date.now() - startTime;

        this.telemetry.recordRequest({
          adapter: adapterName,
          method,
          success: true,
          duration,
          timestamp: Date.now()
        });

        console.log(`Success: ${adapterName} completed ${method} in ${duration}ms`);
        return result;
      } catch (error) {
        lastError = error as Error;
        const duration = Date.now() - startTime;

        this.telemetry.recordRequest({
          adapter: adapterName,
          method,
          success: false,
          duration,
          error: error instanceof Error ? error.message : String(error),
          timestamp: Date.now()
        });

        console.warn(`Failed: ${adapterName} ${method} - ${error instanceof Error ? error.message : error}`);

        // Continue to next adapter if we haven't exceeded retry count
        if (attempt >= retryCount) {
          break;
        }
      }
    }

    throw lastError || new Error('All network adapters failed');
  }

  private isAdapterEnabled(adapterName: string): boolean {
    switch (adapterName) {
      case 'citadel': return this.config.citadel.enabled;
      case 'peerbit': return this.config.peerbit.enabled;
      case 'http': return true; // HTTP always available as fallback
      default: return false;
    }
  }

  private getAdapterConfig(adapterName: string): any {
    return this.config[adapterName as keyof NetworkConfig];
  }

  getTelemetry(): NetworkTelemetry {
    return this.telemetry;
  }

  async getNetworkStats(): Promise<Record<string, any>> {
    const stats: Record<string, any> = {};

    for (const [name, adapter] of this.adapters) {
      if (this.isAdapterEnabled(name)) {
        try {
          stats[name] = await adapter.getNetworkStats();
        } catch (error) {
          console.error(`Failed to get stats from ${name} adapter:`, error);
          stats[name] = { error: error instanceof Error ? error.message : String(error) };
        }
      }
    }

    return stats;
  }

  async checkHealth(): Promise<Record<string, any>> {
    const health: Record<string, any> = {};

    for (const [name, adapter] of this.adapters) {
      if (this.isAdapterEnabled(name)) {
        try {
          health[name] = await adapter.checkHealth();
        } catch (error) {
          console.error(`Failed to check health of ${name} adapter:`, error);
          health[name] = {
            isHealthy: false,
            error: error instanceof Error ? error.message : String(error)
          };
        }
      }
    }

    return health;
  }
}
