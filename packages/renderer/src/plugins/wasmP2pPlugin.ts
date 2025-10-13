import { type App } from 'vue';
import { WasmP2pService } from '/@/plugins/lensService/wasmP2pService';
import { getRelayUrl } from '/@/utils/runtimeConfig';

export default {
  install: (app: App) => {
    // Get relay URL from runtime config
    const relayUrl = getRelayUrl();

    console.log('[WASM P2P Plugin] Initializing with relay URL:', relayUrl);

    // Create WASM P2P service instance
    const wasmP2pService = new WasmP2pService(relayUrl);

    // Initialize the service
    wasmP2pService.initialize().catch((error) => {
      console.error('[WASM P2P Plugin] Failed to initialize:', error);
    });

    // Provide to all components
    app.provide('wasmP2pService', wasmP2pService);
    app.config.globalProperties.$wasmP2pService = wasmP2pService;

    console.log('[WASM P2P Plugin] Plugin installed and service initialized');
  },
};
