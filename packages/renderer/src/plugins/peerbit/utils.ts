import { inject, ref, computed, type Ref } from 'vue';
import type { Peerbit } from 'peerbit';
import type { Libp2p } from '@libp2p/interface'; // For typing libp2p if needed directly

// Define a more representative type for the orbiter-like structure if needed
// For now, we'll make it more directly reflective of Peerbit's capabilities
// that AccountPage.vue seems to expect.

// Define an interface for the Orbiter-like adapter
export interface OrbiterAdapter {
  isReady: boolean;
  getAccountId: () => Promise<string | undefined>;
  getDeviceId: () => Promise<string | undefined>;
  getPeerId: () => Promise<string | undefined>;
  listenForNameChange: () => Record<string, string>; 
  followIsModerator: () => boolean; 
  followCanUpload: () => boolean;   
  constellation: { 
    réseau: {
      suivreConnexionsPostesSFIP?: () => any[]; 
      suivreConnexionsDispositifs?: () => any[]; 
      suivreConnexionsMembres?: () => any[];     
    };
  };
}

/**
 * Provides access to the peerbit instance
 */
export function usePeerbit() {
  const peerbit = inject<Peerbit | undefined>('peerbit'); // Allow undefined initially
  console.log('[usePeerbit] Injected peerbit:', peerbit);
  const program = inject<any | undefined>('program'); 
  const documents = inject<any | undefined>('documents'); 
  const network = inject<any | undefined>('network'); 
  
  return {
    peerbit,
    program,
    documents,
    network
  };
}

/**
 * Compatibility function for peerbit that attempts to provide
 * an interface similar to what AccountPage.vue expects from "orbiter".
 */
export function useOrbiter(): { orbiter: Ref<OrbiterAdapter> } { 
  const peerbitClient = inject<Peerbit | undefined>('peerbit'); // Allow undefined initially
  console.log('[useOrbiter] Injected peerbitClient:', peerbitClient);

  const orbiter = computed<OrbiterAdapter>(() => {
    if (!peerbitClient) {
      console.warn('[useOrbiter] peerbitClient is not available yet or not injected.');
      return {
        isReady: false,
        getAccountId: async () => undefined,
        getDeviceId: async () => undefined,
        getPeerId: async () => undefined,
        listenForNameChange: () => ({}), 
        followIsModerator: () => false,  
        followCanUpload: () => false,    
        constellation: { réseau: {} }, 
      } as OrbiterAdapter; // Cast to ensure type compliance for the placeholder
    }

    console.log('[useOrbiter] peerbitClient is available, creating orbiter adapter.');
    return {
      isReady: true,
      getAccountId: async () => peerbitClient.identity.publicKey.toString(),
      getDeviceId: async () => peerbitClient.identity.publicKey.toString(),
      getPeerId: async () => peerbitClient.peerId.toString(),
      listenForNameChange: () => { 
        console.warn('orbiter.listenForNameChange is not fully implemented');
        return {}; 
      },
      followIsModerator: () => {
        console.warn('orbiter.followIsModerator is not fully implemented');
        return false;
      },
      followCanUpload: () => {
        console.warn('orbiter.followCanUpload is not fully implemented');
        return false;
      },
      constellation: {
        réseau: {
          suivreConnexionsPostesSFIP: () => {
            console.warn('orbiter.constellation.réseau.suivreConnexionsPostesSFIP is not fully implemented');
            return []; 
          },
          suivreConnexionsDispositifs: () => {
            console.warn('orbiter.constellation.réseau.suivreConnexionsDispositifs is not fully implemented');
            return []; 
          },
          suivreConnexionsMembres: () => {
            console.warn('orbiter.constellation.réseau.suivreConnexionsMembres is not fully implemented');
            return []; 
          }
        }
      }
    } as OrbiterAdapter; // Cast to ensure type compliance for the actual adapter
  });

  return { orbiter };
} 