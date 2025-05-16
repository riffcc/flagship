import { type App, ref, readonly } from 'vue';
import { Peerbit } from 'peerbit';
// import { TrustedNetwork } from '@peerbit/trusted-network';
import { hrtime } from '@peerbit/time';
import { logger as peerbitLogger } from '@peerbit/logger';

import { Release, Site } from '/@/lib/schema';
import type { IPeerbitService } from '/@/lib/types';
import { DEFAULT_SITE_ID } from '/@/lib/constants';

console.log('[Peerbit Plugin File] Loaded');

const peerbitServiceRef = ref<IPeerbitService | undefined>(undefined);

export async function initializePeerbitService() {
  console.log('[Peerbit Service Initialize] Starting Peerbit service initialization...');
  try {
    let serviceInstance: IPeerbitService;
    const bootstrappersRaw = import.meta.env.VITE_BOOTSTRAPPERS as string | undefined;

    if (import.meta.env.IS_ELECTRON) {
      console.log('[Peerbit Plugin Renderer] Running in Electron. Using main process Peerbit via IPC.');
      if (!window.electronPeerbit) {
        throw new Error(
          'Electron Peerbit API (window.electronPeerbit) not found. Ensure preload script is correctly loaded and exposing the API.',
        );
      }
      // Ensure electronIPC is available (typescript will help via global.d.ts)
      if (!window.electronIPC || typeof window.electronIPC.onceMainReady !== 'function') {
         throw new Error('Electron IPC API (window.electronIPC.onceMainReady) not found. Ensure preload script is correctly loaded.');
      }

      // Wait for the main process to be ready
      await new Promise<void>(resolve => {
        console.log('[Peerbit Plugin Renderer] Waiting for main process ready signal...');
        window.electronIPC.onceMainReady(() => {
          console.log('[Peerbit Plugin Renderer] Main process ready signal received.');
          resolve();
        });
      });

      serviceInstance = {
        getPublicKey: () => window.electronPeerbit.getPublicKey(),
        getPeerId: () => window.electronPeerbit.getPeerId(),
        dial: (address: string) => window.electronPeerbit.dial(address),
        addRelease: (release: Release) => window.electronPeerbit.addRelease(release),
        getRelease: (id: string) => window.electronPeerbit.getRelease(id),
        getLatestReleases: (size?: number) => window.electronPeerbit.getLatestReleases(size),
      };
    } else {
      console.log('[Peerbit Service Initialize] Running as Web App. Initializing local Peerbit node.');
      let siteId = import.meta.env.VITE_SITE_ID as string | undefined;

      if (!siteId) {
        console.warn('[Peerbit Service Initialize] VITE_SITE_ID is missing. Using default.');
      }
      siteId = DEFAULT_SITE_ID; // Ensure siteId has a value

      const localPeerbitClient = await Peerbit.create({
        directory: `./.peerbit/${siteId}`,
      });
      const siteProgram = await localPeerbitClient.open(new Site(siteId));
      console.log(`[Peerbit Service Initialize] Local Peerbit for web initialized. Peer ID: ${localPeerbitClient.peerId.toString()}`);
      
      serviceInstance = {
        getPublicKey: () => localPeerbitClient.identity.publicKey.toString(),
        getPeerId: () => localPeerbitClient.peerId.toString(),
        dial: (address: string) => localPeerbitClient.dial(address),
        addRelease: async (releaseDataFromForm: any) => { // Changed from 'release: Release' to 'any'
          // Map data from form to Release constructor properties for web path
          const releaseConstructorProps = {
            name: releaseDataFromForm.name,
            categoryId: releaseDataFromForm.category, // Map 'category' to 'categoryId'
            contentCID: releaseDataFromForm.file,     // Map 'file' to 'contentCID'
            thumbnailCID: releaseDataFromForm.thumbnail, // Map 'thumbnail' to 'thumbnailCID'
            metadata: releaseDataFromForm.metadata ? JSON.stringify(releaseDataFromForm.metadata) : undefined,
          };

          if (!releaseConstructorProps.name || !releaseConstructorProps.categoryId || !releaseConstructorProps.contentCID) {
            console.error('[Peerbit Web] Missing required fields for Release:', releaseConstructorProps);
            throw new Error('Missing required fields (name, categoryId, or contentCID) for Release constructor.');
          }
          
          const releaseInstance = new Release(releaseConstructorProps);
          if (!releaseInstance.id) {
            console.error('[Peerbit Web] Critical: Release instance created without an ID. Data:', releaseConstructorProps);
            throw new Error('Release instance created without an ID. Check Release class constructor.');
          }
          console.log(`[Peerbit Web] Adding release - ID: ${releaseInstance.id}, Name: ${releaseInstance.name}`);
          const resultHash = await siteProgram.addRelease(releaseInstance);
          // Match the return structure expected by the component and now provided by main process
          return { success: true, id: releaseInstance.id, hash: resultHash, message: "Release added successfully (Web)" }; 
        },
        getRelease: (id: string) => siteProgram.getRelease(id),
        getLatestReleases: (size?: number) => siteProgram.getLatestReleases(size),
      };
    }

    peerbitServiceRef.value = serviceInstance; // Make the service instance available reactively
    console.log('[Peerbit Service Initialize] Peerbit service instance created and assigned to ref.');

    if (bootstrappersRaw && peerbitServiceRef.value) {
      const bootstrappers = bootstrappersRaw.split(',').map((b) => b.trim());
      console.log('[Peerbit Service Initialize] Dialing bootstrappers in parallel:', bootstrappers);
      const dialPromises = bootstrappers.map(async (bootstrapper) => {
        try {
          await peerbitServiceRef.value!.dial(bootstrapper); // Use non-null assertion as we've just assigned it
          console.log(`[Peerbit Service Initialize] Dialed ${bootstrapper} successfully`);
        } catch (dialError) {
          console.error(`[Peerbit Service Initialize] Error dialing bootstrapper ${bootstrapper}:`, dialError);
        }
      });
      await Promise.allSettled(dialPromises);
      console.log('[Peerbit Service Initialize] Finished dialing bootstrappers.');
    } else if (!bootstrappersRaw) {
      console.log('[Peerbit Service Initialize] No bootstrappers defined.');
    }

    console.log('[Peerbit Service Initialize] Peerbit service initialization complete.');
  } catch (error) {
    console.error('[Peerbit Service Initialize] Failed to initialize Peerbit Service:', error);
    // Optionally, set peerbitServiceRef.value to a specific error state or leave it undefined
  }
}

export default {
  install: (app: App) => {
    console.log('[Peerbit Plugin Install] Initializing synchronous utilities...');
    const timeUtility = hrtime();
    const logInstance = peerbitLogger({ module: 'peerbit-plugin' });

    console.log('[Peerbit Plugin Install] Providing Peerbit service ref and utilities to Vue app...');
    // Provide the service as a readonly ref to prevent accidental mutation by consumers
    app.provide('peerbitService', readonly(peerbitServiceRef));
    app.provide('time', timeUtility);
    app.provide('logger', logInstance);

    // For legacy options API or direct access (less common with Composition API)
    app.config.globalProperties.$peerbitService = readonly(peerbitServiceRef);
    app.config.globalProperties.$time = timeUtility;
    app.config.globalProperties.$logger = logInstance;

    console.log('[Peerbit Plugin Install] Synchronous part of Peerbit plugin installed.');
    // The initializePeerbitService() function will be called separately, e.g., in App.vue's onMounted hook.
  },
};
