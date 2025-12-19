/**
 * Citadel SDK
 * Lightweight TypeScript SDK for the Citadel API
 *
 * Replaces @riffcc/lens-sdk with a minimal HTTP-only implementation
 * No peerbit, no sqlite3, no WASM - just clean HTTP calls
 */

// Export all types
export * from './types';

// Export client
export { CitadelService, LensService, type ILensService, type CitadelClientConfig } from './client';

// Default export for Vue plugin compatibility
import { CitadelService } from './client';
import type { App } from 'vue';

export default {
  install: (app: App, config?: { baseUrl?: string }) => {
    const service = new CitadelService(config?.baseUrl || '/api/v1');
    app.provide('lensService', service);
    app.config.globalProperties.$lensService = service;
  },
};
