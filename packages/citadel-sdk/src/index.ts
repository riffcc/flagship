/**
 * Citadel SDK
 * Lightweight TypeScript SDK for the Citadel API
 *
 * HTTP-oriented Citadel client for the web/renderer path.
 * This does not replace the legacy Electron/Peerbit Lens service contract.
 */

// Export all types
export * from './types';

// Export client
export { CitadelService, type ILensService, type CitadelClientConfig } from './client';

// Default export for Vue plugin compatibility
import { CitadelService } from './client';
import type { App } from 'vue';

export default {
  install: (app: App, config?: { baseUrl?: string }) => {
    const service = new CitadelService(config?.baseUrl || '/api/v1');
    app.provide('citadelService', service);
    app.config.globalProperties.$citadelService = service;
  },
};
