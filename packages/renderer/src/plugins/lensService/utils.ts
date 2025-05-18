import { inject } from 'vue';
import type { ILensService } from '@riffcc/lens-sdk';

export function useLensService() {
  const lensService = inject<ILensService>('lensService');
  if (!lensService) {
    throw new Error('Lens Service plugin not initialized.');
  }
  return { lensService };
}
