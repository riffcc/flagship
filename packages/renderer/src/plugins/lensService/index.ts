import { type App } from 'vue';
import { LensService, ElectronLensService, type ILensService } from '@riffcc/lens-sdk';

export * from './hooks';

// Create singleton instance for pre-initialization
let lensServiceInstance: ILensService | undefined = undefined;

if (import.meta.env.IS_ELECTRON) {
  // Electron instance will be created when ready
} else {
  // Create browser instance immediately
  lensServiceInstance = new LensService();
}

export const lensService = lensServiceInstance as ILensService;

export default {
  install: (app: App) => {
    if (import.meta.env.IS_ELECTRON) {
      if (!window.electronLensService) {
        throw new Error(
          'Electron Peerbit API (window.electronPeerbit) not found. Ensure preload script is correctly loaded and exposing the API.',
        );
      }
      if (!window.electronIPC || typeof window.electronIPC.onceMainReady !== 'function') {
        throw new Error('Electron IPC API (window.electronIPC.onceMainReady) not found. Ensure preload script is correctly loaded.');
      }

      window.electronIPC.onceMainReady(() => {
        lensServiceInstance = new ElectronLensService();
        // Update the export
        Object.assign(lensService, lensServiceInstance);
      });
    }

    app.provide('lensService', lensServiceInstance || lensService);
    app.config.globalProperties.$lensService = lensServiceInstance || lensService;
  },
};
