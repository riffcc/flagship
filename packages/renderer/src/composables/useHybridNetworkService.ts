// Hybrid Network Service Composable
// Provides access to both the new UnifiedNetworkService and the legacy LensService
// for backward compatibility during migration

import { inject } from 'vue';
import { ref } from 'vue';
import { UnifiedNetworkService } from '../plugins/network';
import { useLensService } from './useLensService';

export function useHybridNetworkService() {
  const networkService = inject<UnifiedNetworkService>('networkService');
  const { lensService } = useLensService();
  const isInitialized = ref(true);
  const isLoading = ref(false);
  const error = ref<unknown>(null);

  if (!networkService) {
    throw new Error('useHybridNetworkService() is called without provider. Make sure to install the UnifiedNetworkService plugin.');
  }

  return {
    // New unified network service
    networkService,
    isInitialized,
    isLoading,
    error,

    // Legacy lens service for backward compatibility
    lensService,

    // Helper methods that prefer the new service but fall back to legacy
    async getReleases(options?: any) {
      try {
        return await networkService.getReleases(options);
      } catch (error) {
        console.warn('Network service failed, falling back to legacy lens service:', error);
        return lensService.getReleases();
      }
    },

    async getRelease(id: string) {
      try {
        return await networkService.getRelease(id);
      } catch (error) {
        console.warn('Network service failed, falling back to legacy lens service:', error);
        return lensService.getRelease(id);
      }
    },

    // Network-specific methods
    async getNetworkStats() {
      return networkService.getNetworkStats();
    },

    async checkHealth() {
      return networkService.checkHealth();
    },

    // Configuration access
    getNetworkConfig() {
      return networkService.getConfig();
    }
  };
}
