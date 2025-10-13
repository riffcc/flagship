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
  relayUrl: string;
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
  if (window.__RUNTIME_CONFIG__?.apiUrl) {
    return window.__RUNTIME_CONFIG__.apiUrl;
  }

  // Fall back to build-time environment variable (development)
  return import.meta.env.VITE_API_URL || 'http://127.0.0.1:5002/api/v1';
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
  return `${baseUrl}/api/v1/relay/ws`;
}

/**
 * Get full runtime configuration object
 */
export function getRuntimeConfig(): RuntimeConfig {
  return {
    apiUrl: getApiUrl(),
    relayUrl: getRelayUrl(),
  };
}

/**
 * Check if runtime config is loaded (true = Docker, false = development)
 */
export function isRuntimeConfigLoaded(): boolean {
  return !!window.__RUNTIME_CONFIG__;
}
