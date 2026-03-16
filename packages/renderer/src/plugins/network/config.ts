// Network Configuration Interface
// Defines the configuration structure for the hybrid network service

export interface NetworkConfig {
  // Citadel configuration
  citadel: {
    bootstrapNodes: string[];
    dhtStoragePath: string;
    lazyLoading: boolean;
    cacheTTL: number; // milliseconds
  };

  // HTTP API configuration
  http: {
    baseUrl: string;
    timeout: number;
  };

  // Fallback strategy
  fallback: {
    timeout: number; // ms to wait before fallback
    retryCount: number;
    order: ('citadel' | 'http')[];
  };
}

// Default configuration
export const defaultNetworkConfig: NetworkConfig = {
  citadel: {
    bootstrapNodes: [],
    dhtStoragePath: '.citadel-dht',
    lazyLoading: true,
    cacheTTL: 10000,
  },
  http: {
    baseUrl: '/api/v1',
    timeout: 5000,
  },
  fallback: {
    timeout: 2000,
    retryCount: 2,
    order: ['citadel', 'http'],
  },
};
