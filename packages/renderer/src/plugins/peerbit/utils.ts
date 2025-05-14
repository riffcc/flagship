import { inject } from 'vue';
import { ref } from 'vue';
import type { Peerbit } from 'peerbit';

/**
 * Provides access to the peerbit instance
 */
export function usePeerbit() {
  const peerbit = inject<Peerbit>('peerbit');
  const program = inject('program');
  const documents = inject('documents');
  const network = inject('network');
  
  return {
    peerbit,
    program,
    documents,
    network
  };
}

/**
 * Compatibility function for peerbit that replaces the orbiter functionality
 */
export function useOrbiter() {
  const peerbit = inject<Peerbit>('peerbit');
  
  // Create a simple placeholder for orbiter
  const orbiter = ref({
    libp2p: peerbit,
    listenForReleases: null,
    listenForSiteFeaturedReleases: null
  });
  
  return { orbiter: orbiter.value };
} 