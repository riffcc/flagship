// Network Service Composable
// Provides access to the UnifiedNetworkService in Vue components

import { inject, onMounted, ref } from 'vue';
import { UnifiedNetworkService } from '../plugins/network';

export function useNetworkService() {
  const networkService = inject<UnifiedNetworkService>('networkService');

  if (!networkService) {
    throw new Error('useNetworkService() is called without provider. Make sure to install the UnifiedNetworkService plugin.');
  }

  const isInitialized = ref(false);
  const isLoading = ref(false);
  const error = ref<Error | null>(null);

  onMounted(async () => {
    try {
      if (!networkService.getConfig().peerbit.enabled && !networkService.getConfig().citadel.enabled) {
        // If both P2P networks are disabled, just use HTTP
        isInitialized.value = true;
        return;
      }

      isLoading.value = true;
      await networkService.initialize();
      isInitialized.value = true;
    } catch (err) {
      error.value = err as Error;
      console.error('Failed to initialize network service:', err);
    } finally {
      isLoading.value = false;
    }
  });

  return {
    networkService,
    isInitialized,
    isLoading,
    error,

    // Helper methods
    async getReleases(options?: any) {
      return networkService.getReleases(options);
    },

    async getRelease(id: string) {
      return networkService.getRelease(id);
    },

    async getNetworkStats() {
      return networkService.getNetworkStats();
    },

    async checkHealth() {
      return networkService.checkHealth();
    }
  };
}