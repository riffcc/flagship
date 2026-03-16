/**
 * Runtime Configuration Utility
 *
 * Loads configuration from runtime-injected config.js (Docker) or
 * falls back to build-time VITE_* environment variables (development).
 *
 * This enables dynamic configuration in Docker containers without rebuilding.
 */

interface RuntimeConfig {
  apiUrl?: string;
  citadelPeers?: string;
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
  if (window.__RUNTIME_CONFIG__?.apiUrl) {
    return window.__RUNTIME_CONFIG__.apiUrl;
  }

  return import.meta.env.VITE_API_URL || '/api/v1';
}

/**
 * Get the Relay WebSocket URL from runtime config or build-time env
 */
export function getRelayUrl(): string {
  const apiUrl = getApiUrl();
  const wsUrl = apiUrl.replace('http://', 'ws://').replace('https://', 'wss://');
  const baseUrl = wsUrl.replace(/\/api\/v1\/?$/, '');
  return `${baseUrl}/ws/mesh`;
}

/**
 * Get network configuration from runtime config or build-time env
 */
export function getNetworkConfig(): {
  citadel: {
    bootstrapNodes: string[];
    dhtStoragePath: string;
  };
} {
  if (window.__RUNTIME_CONFIG__) {
    const runtimeConfig = window.__RUNTIME_CONFIG__;
    const bootstrapNodes = runtimeConfig.citadelPeers
      ? runtimeConfig.citadelPeers.split(',').map(value => value.trim()).filter(Boolean)
      : (runtimeConfig.apiUrl ? [runtimeConfig.apiUrl] : []);

    return {
      citadel: {
        bootstrapNodes,
        dhtStoragePath: '.citadel-dht',
      },
    };
  }

  return {
    citadel: {
      bootstrapNodes: import.meta.env.VITE_CITADEL_BOOTSTRAP_NODES ? import.meta.env.VITE_CITADEL_BOOTSTRAP_NODES.split(',') : [],
      dhtStoragePath: '.citadel-dht',
    },
  };
}

/**
 * Check if runtime config is loaded (true = Docker, false = development)
 */
export function isRuntimeConfigLoaded(): boolean {
  return !!window.__RUNTIME_CONFIG__;
}
