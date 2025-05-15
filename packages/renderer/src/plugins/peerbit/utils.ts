import { inject } from 'vue';
import type { IPeerbitService } from '/@/lib/types';

/**
 * Provides access to the peerbit service instance
 */
export function usePeerbitService() {
  const peerbitService = inject<IPeerbitService | undefined>('peerbitService');
  if (!peerbitService) {
    throw new Error('Fail to initialize the Peerbit Service');
  }
  return {
    peerbitService,
  };
}


// Define a more representative type for the orbiter-like structure if needed
// For now, we'll make it more directly reflective of Peerbit's capabilities
// that AccountPage.vue seems to expect.

// Define an interface for the Orbiter-like adapter
export interface OrbiterAdapter {
  getAccountId: () => Promise<string | undefined>;
  getPeerId: () => Promise<string | undefined>;
  listenForNameChange: () => Record<string, string>;
  followIsModerator: () => Promise<boolean>;
  followCanUpload: () => Promise<boolean>;
  constellation: {
    réseau: {
      suivreConnexionsPostesSFIP?: () => unknown[];
      suivreConnexionsDispositifs?: () => unknown[];
      suivreConnexionsMembres?: () => unknown[];
    };
  };
}

/**
 * Compatibility function for peerbit that attempts to provide
 * an interface similar to what AccountPage.vue expects from "orbiter".
 */
export function useOrbiter(): { orbiter: OrbiterAdapter } {
  const { peerbitService } = usePeerbitService();

  return { orbiter: {
    getAccountId: async () => peerbitService.getPublicKey(),
    getPeerId: async () => peerbitService.getPeerId(),
    followIsModerator: async () => {
      return false;
    },
    followCanUpload: async () => {
      return false;
    },
    listenForNameChange: () => {
      console.warn('orbiter.listenForNameChange is not fully implemented');
      return {};
    },
    constellation: {
      réseau: {
        suivreConnexionsPostesSFIP: () => { /* Placeholder */ return []; },
        suivreConnexionsDispositifs: () => { /* Placeholder */ return []; },
        suivreConnexionsMembres: () => { /* Placeholder */ return []; },
      },
    },
  } };
}
