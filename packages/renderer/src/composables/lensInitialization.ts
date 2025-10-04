import { ref } from 'vue';
import { RIFFCC_PEERBIT_BOOTSTRAPPERS } from '../constants/config';

// This state will be shared across the entire application
const isLensReady = ref(false);

const USE_PEERBIT = import.meta.env.VITE_USE_PEERBIT === 'true';

const initLensService = async () => {
  // Check if we should use legacy Peerbit
  if (!USE_PEERBIT) {
    console.log('[Lens] Peerbit disabled, using Lens V2 SDK identity system');
    isLensReady.value = true;
    return;
  }

  // Prevent re-initialization
  if (isLensReady.value) return;

  const { useLensService } = await import('/@/plugins/lensService/hooks');
  const { lensService } = useLensService();
  const siteAddress = import.meta.env.VITE_SITE_ADDRESS;
  const lensNode = import.meta.env.VITE_LENS_NODE;
  const viteBootstrappers = import.meta.env.VITE_BOOTSTRAPPERS;

  // --- Environment Variable Checks (Good, no changes needed) ---
  if (!siteAddress) {
    throw new Error('VITE_SITE_ADDRESS env var missing...', { cause: 'MISSING_CONFIG' });
  }
  if (!lensNode) {
    throw new Error('VITE_LENS_NODE env var missing...', { cause: 'MISSING_CONFIG' });
  }

  try {
    console.log('Starting background Lens Service initialization...');
    await lensService.init('.lens-node');

    // --- REFINED DIALING LOGIC ---

    // 1. Construct a clean, unified list of all addresses to dial.
    //    - `lensNode` can be a comma-separated string of addresses.
    //    - `viteBootstrappers` can be a comma-separated string of addresses.
    //    - `RIFFCC_PEERBIT_BOOTSTRAPPERS` is correctly spread if it's an array.
    const lensNodesList = lensNode
      ? lensNode.split(',').map((addr: string) => addr.trim())
      : [];

    const bootstrappersList = viteBootstrappers
      ? viteBootstrappers.split(',').map((addr: string) => addr.trim())
      : [];

    const allAddressesToDial = [
      ...lensNodesList,
      ...bootstrappersList,
      ...RIFFCC_PEERBIT_BOOTSTRAPPERS,
    ].filter(Boolean); // .filter(Boolean) removes any empty or null strings.

    console.log('Attempting to dial bootstrap peers:', allAddressesToDial);

    // 2. Dial all peers in parallel.
    const dialResults = await Promise.allSettled(
      allAddressesToDial.map(b => lensService.peerbit?.dial(b.trim())),
    );

    // 3. (IMPROVEMENT) Log the results of the dialing attempts for easy debugging.
    let successfulDials = 0;
    dialResults.forEach((result, index) => {
      const address = allAddressesToDial[index];
      if (result.status === 'fulfilled') {
        successfulDials++;
        console.log(`✅ Successfully dialed: ${address}`);
      } else {
        console.warn(`❌ Failed to dial: ${address}`, result.reason.message || result.reason);
      }
    });
    console.log(`Finished dialing: ${successfulDials}/${allAddressesToDial.length} peers connected.`);


    // --- Continue with initialization ---
    await lensService.openSite(siteAddress, { federate: false });
    isLensReady.value = true;
    console.log('✅ Lens Service is ready in the background.');

  } catch (error) {
    console.error('❌ Lens Service background initialization failed:', error);
  }
};

export function useLensInitialization() {
  return {
    isLensReady,
    initLensService,
  };
}
