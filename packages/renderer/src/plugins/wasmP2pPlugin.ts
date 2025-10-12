import { type App } from 'vue';
import { WasmP2pService } from '/@/plugins/lensService/wasmP2pService';

export default {
  install: (app: App) => {
    // Get relay URL from environment
    const apiUrl = import.meta.env.VITE_RELAY_URL || import.meta.env.VITE_API_URL || 'http://localhost:5002';
    const wsUrl = apiUrl.replace('http://', 'ws://').replace('https://', 'wss://');
    // If it doesn't already have /relay/ws, add it
    const relayUrl = wsUrl.endsWith('/relay/ws') ? wsUrl : `${wsUrl}/relay/ws`;

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
