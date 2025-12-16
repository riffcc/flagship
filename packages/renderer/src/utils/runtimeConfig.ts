/**
 * Runtime Configuration Utility
 *
 * Loads configuration from runtime-injected config.js (Docker) or
 * falls back to build-time VITE_* environment variables (development).
 *
 * This enables dynamic configuration in Docker containers without rebuilding.
 */

interface RuntimeConfig {
  apiUrl: string;
  lensNode?: string;  // Alternative to apiUrl (Docker prebuilt uses this)
  relayUrl: string;
  networkMode?: 'peerbit-only' | 'citadel-only' | 'hybrid' | 'auto';
  usePeerbit?: boolean;
  useCitadel?: boolean;
  peerbitRelayUrl?: string;
  peerbitBootstrappers?: string;
  peerbitSiteAddress?: string;
  citadelBootstrapNodes?: string;
  citadelDhtPath?: string;
  fallbackOrder?: string;
  fallbackTimeout?: number;
}

declare global {
  interface Window {
    __RUNTIME_CONFIG__?: RuntimeConfig;
  }
}

/**
 * Get the API URL from runtime config or build-time env
 */
export function getApiUrl(): string {
  // Try runtime config first (injected by Docker)
  // Support both apiUrl and lensNode field names for compatibility
  if (window.__RUNTIME_CONFIG__?.apiUrl) {
    return window.__RUNTIME_CONFIG__.apiUrl;
  }
  if (window.__RUNTIME_CONFIG__?.lensNode) {
    return window.__RUNTIME_CONFIG__.lensNode;
  }

  // Fall back to build-time environment variables (development)
  // Support both VITE_API_URL and VITE_LENS_NODE for backwards compatibility
  // Default: api.global.riff.cc for production, override with VITE_API_URL for local dev
  return import.meta.env.VITE_API_URL || import.meta.env.VITE_LENS_NODE || 'https://api.global.riff.cc/api/v1';
}

/**
 * Get the Relay WebSocket URL from runtime config or build-time env
 */
export function getRelayUrl(): string {
  // Try runtime config first (injected by Docker)
  if (window.__RUNTIME_CONFIG__?.relayUrl) {
    return window.__RUNTIME_CONFIG__.relayUrl;
  }

  // Fall back to build-time environment variable (development)
  const relayUrl = import.meta.env.VITE_RELAY_URL;
  if (relayUrl) {
    return relayUrl;
  }

  // Construct from API URL as fallback
  const apiUrl = getApiUrl();
  const wsUrl = apiUrl.replace('http://', 'ws://').replace('https://', 'wss://');
  const baseUrl = wsUrl.replace(/\/api\/v1\/?$/, '');
  return `${baseUrl}/ws`;
}

/**
 * Get network configuration from runtime config or build-time env
 */
export function getNetworkConfig(): {
  mode: 'peerbit-only' | 'citadel-only' | 'hybrid' | 'auto';
  peerbit: {
    enabled: boolean;
    relayUrl: string;
    bootstrappers: string[];
    siteAddress: string;
  };
  citadel: {
    enabled: boolean;
    bootstrapNodes: string[];
    dhtStoragePath: string;
  };
  fallback: {
    order: ('citadel' | 'peerbit' | 'http')[];
    timeout: number;
  };
} {
  // Try runtime config first (injected by Docker)
  if (window.__RUNTIME_CONFIG__) {
    const runtimeConfig = window.__RUNTIME_CONFIG__;

    return {
      mode: runtimeConfig.networkMode || 'hybrid',
      peerbit: {
        enabled: runtimeConfig.usePeerbit !== false,
        relayUrl: runtimeConfig.peerbitRelayUrl || getRelayUrl(),
        bootstrappers: runtimeConfig.peerbitBootstrappers ? runtimeConfig.peerbitBootstrappers.split(',') : [],
        siteAddress: runtimeConfig.peerbitSiteAddress || ''
      },
      citadel: {
        enabled: runtimeConfig.useCitadel !== false,
        bootstrapNodes: runtimeConfig.citadelBootstrapNodes ? runtimeConfig.citadelBootstrapNodes.split(',') : [],
        dhtStoragePath: runtimeConfig.citadelDhtPath || '.citadel-dht'
      },
      fallback: {
        order: runtimeConfig.fallbackOrder ? runtimeConfig.fallbackOrder.split(',') as ('citadel' | 'peerbit' | 'http')[] : ['citadel', 'peerbit', 'http'],
        timeout: runtimeConfig.fallbackTimeout || 2000
      }
    };
  }

  // Fall back to build-time environment variables (development)
  return {
    mode: (import.meta.env.VITE_NETWORK_MODE as any) || 'hybrid',
    peerbit: {
      enabled: import.meta.env.VITE_USE_PEERBIT !== 'false',
      relayUrl: import.meta.env.VITE_RELAY_URL || getRelayUrl(),
      bootstrappers: import.meta.env.VITE_BOOTSTRAPPERS ? import.meta.env.VITE_BOOTSTRAPPERS.split(',') : [],
      siteAddress: import.meta.env.VITE_SITE_ADDRESS || ''
    },
    citadel: {
      enabled: import.meta.env.VITE_USE_CITADEL !== 'false',
      bootstrapNodes: import.meta.env.VITE_CITADEL_BOOTSTRAP_NODES ? import.meta.env.VITE_CITADEL_BOOTSTRAP_NODES.split(',') : [],
      dhtStoragePath: import.meta.env.VITE_CITADEL_DHT_PATH || '.citadel-dht'
    },
    fallback: {
      order: import.meta.env.VITE_FALLBACK_ORDER ? import.meta.env.VITE_FALLBACK_ORDER.split(',') as ('citadel' | 'peerbit' | 'http')[] : ['citadel', 'peerbit', 'http'],
      timeout: import.meta.env.VITE_FALLBACK_TIMEOUT ? parseInt(import.meta.env.VITE_FALLBACK_TIMEOUT) : 2000
    }
  };
}

/**
 * Check if runtime config is loaded (true = Docker, false = development)
 */
export function isRuntimeConfigLoaded(): boolean {
  return !!window.__RUNTIME_CONFIG__;
}
