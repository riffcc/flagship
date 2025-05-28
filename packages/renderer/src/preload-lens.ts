console.time('[Preload] Total preload time');

// Pre-initialize lens service before Vue mounts
import { lensService } from './plugins/lensService';

// Start initialization immediately (only if not already initialized)
console.time('[Preload] Lens service init');
const lensInitPromise = (async () => {
  try {
    // Check if already initialized by trying to get the peer ID
    await lensService.getPeerId();
    console.log('[Preload] Lens service already initialized');
    console.timeEnd('[Preload] Lens service init');
    return lensService;
  } catch {
    // Not initialized, do it now
    await lensService.init('.lens-node');
    console.timeEnd('[Preload] Lens service init');
    console.log('[Preload] Lens service ready');
    return lensService;
  }
})().catch(err => {
  console.error('[Preload] Failed to initialize lens service:', err);
  throw err;
});

// Export the promise so components can await it
export const lensServiceReady = lensInitPromise;

// Also try to pre-connect to first bootstrapper if available
export const preConnectBootstrapper = async () => {
  const bootstrappers = import.meta.env.VITE_BOOTSTRAPPERS;
  if (bootstrappers) {
    const firstBootstrapper = bootstrappers.split(',')[0].trim();
    if (firstBootstrapper) {
      console.time('[Preload] First bootstrapper pre-connect');
      try {
        await lensService.dial(firstBootstrapper);
        console.log('[Preload] Pre-connected to first bootstrapper');
      } catch (err) {
        console.warn('[Preload] Failed to pre-connect:', err.message);
      }
      console.timeEnd('[Preload] First bootstrapper pre-connect');
    }
  }
};

// Start pre-connection after service is ready
lensInitPromise.then(() => preConnectBootstrapper());

// Register service worker for caching
if ('serviceWorker' in navigator && import.meta.env.PROD) {
  console.time('[Preload] Service worker registration');
  navigator.serviceWorker.register('/sw.js').then((registration) => {
    console.timeEnd('[Preload] Service worker registration');
    console.log('[Preload] Service worker registered:', registration.scope);
    
    // Pre-cache current federation data if available
    registration.active?.postMessage({
      type: 'CACHE_FEDERATION_DATA',
      url: '/federation/featured',
      data: { timestamp: Date.now() }
    });
  }).catch((error) => {
    console.error('[Preload] Service worker registration failed:', error);
  });
}

// Log when everything in preload is done
Promise.all([lensInitPromise]).then(() => {
  console.timeEnd('[Preload] Total preload time');
});