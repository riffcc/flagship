import { inject } from 'vue';
import type { BrowserLensService } from '@riffcc/lens-sdk';

export function useLensService() {
  const lensService = inject<BrowserLensService>('lensService');
  if (!lensService) {
    throw new Error('Lens Service plugin not initialized.');
  }
  return { lensService };
}
