// Base Network Adapter Interface
// Defines the common interface that all network adapters must implement

export interface BaseNetworkAdapter<T> {
  readonly name: string;
  readonly type: string;
  readonly isInitialized: boolean;

  initialize(config: any): Promise<void>;
  disconnect(): Promise<void>;

  // Data operations
  getReleases(options?: any): Promise<T[]>;
  getRelease(id: string): Promise<T>;
  createRelease(data: any): Promise<T>;
  updateRelease(id: string, data: any): Promise<T>;
  deleteRelease(id: string): Promise<void>;

  // Network operations
  getPeerId(): string;
  getConnectedPeers(): Promise<string[]>;
  getNetworkStats(): Promise<NetworkStats>;

  // Health check
  checkHealth(): Promise<NetworkHealth>;
}

export interface NetworkStats {
  peerCount: number;
  latency: number;
  uptime: number;
  dataSource: 'citadel' | 'peerbit' | 'http';
}

export interface NetworkHealth {
  isHealthy: boolean;
  responseTime: number;
  error?: Error;
}