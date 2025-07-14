import { type App } from 'vue';
import { LensService, ElectronLensService, type ILensService } from '@riffcc/lens-sdk';

export * from './hooks';

export default {
  install: (app: App) => {
    let lensServiceInstance: ILensService | undefined = undefined;

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
      });

    } else {
      console.log('[LensService] Initializing browser LensService...');
      try {
        lensServiceInstance = new LensService();
        console.log('[LensService] Browser LensService initialized successfully');
      } catch (error) {
        console.error('[LensService] Failed to initialize:', error);
        throw error;
      }
    }

    app.provide('lensService', lensServiceInstance);
    app.config.globalProperties.$lensService = lensServiceInstance;

  },
};
