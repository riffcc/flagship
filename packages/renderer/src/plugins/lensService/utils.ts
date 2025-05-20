import { inject } from 'vue';
import type { LensService } from '@riffcc/lens-sdk';

export function useLensService() {
  const lensService = inject<LensService>('lensService');
  if (!lensService) {
    throw new Error('Lens Service plugin not initialized.');
  }
  return { lensService };
}
