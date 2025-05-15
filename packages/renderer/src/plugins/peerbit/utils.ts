import { inject, ref, computed, type Ref, type ComputedRef } from 'vue';
import type { Peerbit } from 'peerbit';
import type { Libp2p } from '@libp2p/interface'; // For typing libp2p if needed directly
import type { Documents } from '@peerbit/document';
import { Account, AccountType, Release } from './schema'; // Import Account, AccountType, and Release

// Define a more representative type for the orbiter-like structure if needed
// For now, we'll make it more directly reflective of Peerbit's capabilities
// that AccountPage.vue seems to expect.

// Define an interface for the Orbiter-like adapter
export interface OrbiterAdapter {
  isReady: Ref<boolean>;
  getAccountId: () => Promise<string | undefined>;
  getDeviceId: () => Promise<string | undefined>;
  getPeerId: () => Promise<string | undefined>;
  listenForNameChange: () => Record<string, string>; 
  followIsModerator: () => Promise<boolean>;
  followCanUpload: () => Promise<boolean>;
  getCurrentUserAccount: () => Promise<Account | undefined>;
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
  const site = inject<any | undefined>('site'); // site program
  const releasesStore = inject<Documents<Release> | undefined>('releasesStore');
  const usersStore = inject<Documents<Account> | undefined>('usersStore');
  const network = inject<any | undefined>('network'); 
  
  return {
    peerbit,
    site,
    releasesStore,
    usersStore,
    network
  };
}

/**
 * Compatibility function for peerbit that attempts to provide
 * an interface similar to what AccountPage.vue expects from "orbiter".
 */
export function useOrbiter(): { orbiter: ComputedRef<OrbiterAdapter> } { 
  const peerbitClient = inject<Peerbit | undefined>('peerbit');
  const usersStore = inject<Documents<Account> | undefined>('usersStore');
  
  const isReady = ref(false);

  const _getCurrentUserAccount = async (): Promise<Account | undefined> => {
    // Ensure client, identity, and store are ready before trying to get from index
    if (!peerbitClient || !peerbitClient.identity || !usersStore || usersStore.closed) {
        console.debug('[_getCurrentUserAccount] Conditions not met:', 
            { hasClient: !!peerbitClient, hasIdentity: !!peerbitClient?.identity, hasStore: !!usersStore, storeClosed: usersStore?.closed });
        return undefined;
    }
    try {
      const accountDoc = await usersStore.index.get(peerbitClient.identity.publicKey.bytes);
      return accountDoc;
    } catch (error) {
      console.error("Error fetching current user account from usersStore:", error);
      return undefined;
    }
  };

  const orbiterAdapter: ComputedRef<OrbiterAdapter> = computed(() => {
    // Readiness check: peerbitClient and its identity are available, and usersStore is open
    if (peerbitClient && peerbitClient.identity && usersStore && !usersStore.closed) {
      console.log('[useOrbiter] Peerbit client and users store are ready. Creating full adapter.');
      isReady.value = true;
      return {
        isReady: isReady, 
        getAccountId: async () => peerbitClient.identity.publicKey.toString(), 
        getDeviceId: async () => peerbitClient.identity.publicKey.toString(), 
        getPeerId: async () => peerbitClient.peerId.toString(),
        getCurrentUserAccount: _getCurrentUserAccount,
        followIsModerator: async () => {
          const account = await _getCurrentUserAccount();
          if (account) {
            return account.type === AccountType.MODERATOR || account.type === AccountType.ADMIN;
          }
          return false;
        },
        followCanUpload: async () => {
          const account = await _getCurrentUserAccount();
          if (account) {
            return account.type === AccountType.USER || account.type === AccountType.MODERATOR || account.type === AccountType.ADMIN;
          }
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
            suivreConnexionsMembres: () => { /* Placeholder */ return []; }
          }
        }
      } as OrbiterAdapter; 
    } else {
      console.warn('[useOrbiter] Peerbit client or users store not ready yet. Creating placeholder adapter.',
        { hasClient: !!peerbitClient, hasIdentity: !!peerbitClient?.identity, hasStore: !!usersStore, storeClosed: usersStore?.closed });
      isReady.value = false;
      return {
        isReady: isReady,
        getAccountId: async () => undefined,
        getDeviceId: async () => undefined,
        getPeerId: async () => undefined,
        getCurrentUserAccount: async () => undefined,
        listenForNameChange: () => ({}), 
        followIsModerator: async () => false,  
        followCanUpload: async () => false,    
        constellation: { réseau: {} }, 
      } as OrbiterAdapter;
    }
  });

  return { orbiter: orbiterAdapter }; 
} 