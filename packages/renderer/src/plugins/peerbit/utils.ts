import { inject, type Ref } from 'vue';
import type { IPeerbitService } from '/@/lib/types';

/**
 * Provides access to the peerbit service instance
 */
export function usePeerbitService(): { peerbitServiceRef: Ref<IPeerbitService | undefined> | undefined } {
  const service = inject<Ref<IPeerbitService | undefined>>('peerbitService');
  return { peerbitServiceRef: service };
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
  const { peerbitServiceRef } = usePeerbitService();

  // Check if the ref itself is undefined or its .value is undefined
  if (!peerbitServiceRef || !peerbitServiceRef.value) {
    console.warn('Peerbit service Ref is not available or not yet initialized. Orbiter will use stub implementations.');
    return {
      orbiter: {
        getAccountId: async () => {
          console.warn('Orbiter stub: getAccountId called because Peerbit service is not available/initialized.');
          return undefined;
        },
        getPeerId: async () => {
          console.warn('Orbiter stub: getPeerId called because Peerbit service is not available/initialized.');
          return undefined;
        },
        listenForNameChange: () => {
          console.warn('Orbiter stub: listenForNameChange called because Peerbit service is not available/initialized.');
          return {}; 
        },
        followIsModerator: async () => false,
        followCanUpload: async () => false,
        constellation: {
          réseau: {
            suivreConnexionsPostesSFIP: () => [],
            suivreConnexionsDispositifs: () => [],
            suivreConnexionsMembres: () => [],
          },
        },
      },
    };
  }

  // At this point, peerbitServiceRef.value is guaranteed to be IPeerbitService
  const serviceInstance = peerbitServiceRef.value; 

  return { orbiter: {
    getAccountId: async () => serviceInstance.getPublicKey(),
    getPeerId: async () => serviceInstance.getPeerId(),
    listenForNameChange: () => {
      console.warn('orbiter.listenForNameChange is not fully implemented using actual Peerbit service yet.');
      return {}; 
    },
    followIsModerator: async () => false,
    followCanUpload: async () => false,
    constellation: {
      réseau: {
        suivreConnexionsPostesSFIP: () => { console.warn("Orbiter: suivreConnexionsPostesSFIP not implemented with Peerbit"); return []; },
        suivreConnexionsDispositifs: () => { console.warn("Orbiter: suivreConnexionsDispositifs not implemented with Peerbit"); return []; },
        suivreConnexionsMembres: () => { console.warn("Orbiter: suivreConnexionsMembres not implemented with Peerbit"); return []; },
      },
    },
  } };
}
