import { inject } from 'vue';
import type { IPeerbitService } from '/@/lib/types';

/**
 * Provides access to the peerbit service instance
 */
export function usePeerbitService() {
  const peerbitService = inject<IPeerbitService | undefined>('peerbitService');
  // Removed the throw new Error to allow the app to load even if Peerbit service is not immediately available.
  // Consumers must handle the case where peerbitService might be undefined.
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

  if (!peerbitService) {
    console.warn('Peerbit service is not available. Orbiter will use stub implementations.');
    return {
      orbiter: {
        getAccountId: async () => {
          console.warn('Orbiter: getAccountId called but Peerbit service is not available.');
          return undefined;
        },
        getPeerId: async () => {
          console.warn('Orbiter: getPeerId called but Peerbit service is not available.');
          return undefined;
        },
        followIsModerator: async () => {
          console.warn('Orbiter: followIsModerator called but Peerbit service is not available.');
          return false;
        },
        followCanUpload: async () => {
          console.warn('Orbiter: followCanUpload called but Peerbit service is not available.');
          return false;
        },
        listenForNameChange: () => {
          console.warn('Orbiter: listenForNameChange called but Peerbit service is not available. This function is also not fully implemented.');
          return {};
        },
        constellation: {
          réseau: {
            suivreConnexionsPostesSFIP: () => {
              console.warn('Orbiter: suivreConnexionsPostesSFIP called but Peerbit service is not available.');
              return [];
            },
            suivreConnexionsDispositifs: () => {
              console.warn('Orbiter: suivreConnexionsDispositifs called but Peerbit service is not available.');
              return [];
            },
            suivreConnexionsMembres: () => {
              console.warn('Orbiter: suivreConnexionsMembres called but Peerbit service is not available.');
              return [];
            },
          },
        },
      },
    };
  }

  // Peerbit service is available, return the original implementation.
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
