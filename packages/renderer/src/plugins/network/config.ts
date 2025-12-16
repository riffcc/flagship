// Network Configuration Interface
// Defines the configuration structure for the hybrid network service

export interface NetworkConfig {
  // Mode selection
  mode: 'peerbit-only' | 'citadel-only' | 'hybrid' | 'auto';

  // Peerbit configuration
  peerbit: {
    enabled: boolean;
    relayUrl: string;
    bootstrappers: string[];
    siteAddress: string;
  };

  // Citadel configuration
  citadel: {
    enabled: boolean;
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
    order: ('citadel' | 'peerbit' | 'http')[];
  };
}

// Default configuration
export const defaultNetworkConfig: NetworkConfig = {
  mode: 'hybrid',
  peerbit: {
    enabled: true,
    relayUrl: 'ws://localhost:5002/ws',
    bootstrappers: [],
    siteAddress: ''
  },
  citadel: {
    enabled: true,
    bootstrapNodes: [],
    dhtStoragePath: '.citadel-dht',
    lazyLoading: true,
    cacheTTL: 10000
  },
  http: {
    baseUrl: '/api/v1',
    timeout: 5000
  },
  fallback: {
    timeout: 2000,
    retryCount: 2,
    order: ['citadel', 'peerbit', 'http']
  }
};