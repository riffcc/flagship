import type { App } from 'vue';
import { Peerbit } from 'peerbit';
// import { TrustedNetwork } from '@peerbit/trusted-network';
import { hrtime } from '@peerbit/time';
import { logger as peerbitLogger } from '@peerbit/logger';

import { Release, Site } from '/@/lib/schema';
import type { IPeerbitService } from '/@/lib/types';
import { DEFAULT_SITE_ID } from '/@/lib/constants';

console.log('[Peerbit Plugin File] Loaded');

let peerbitServiceInstance: IPeerbitService;

export default {
  install: async (app: App) => {
    const bootstrappersRaw = import.meta.env.VITE_BOOTSTRAPPERS as string | undefined;

    if (import.meta.env.IS_ELECTRON) {
      console.log('[Peerbit Plugin Renderer] Running in Electron. Using main process Peerbit via IPC.');
      if (!window.electronPeerbit) {
        throw new Error(
          'Electron Peerbit API (window.electronPeerbit) not found. Ensure preload script is correctly loaded and exposing the API.',
        );
      }
      peerbitServiceInstance = {
        getPublicKey: () => window.electronPeerbit.getPublicKey(),
        getPeerId: () => window.electronPeerbit.getPeerId(),
        dial: (address: string) => window.electronPeerbit.dial(address),
        addRelease: (release: Release) => window.electronPeerbit.addRelease(release),
        getRelease: (id: string) => window.electronPeerbit.getRelease(id),
        getLatestReleases: (size?: number) => window.electronPeerbit.getLatestReleases(size),
      };
    } else {
      console.log('[Peerbit Plugin Install] Starting');
      let siteId = import.meta.env.VITE_SITE_ID as string | undefined;

      if (!siteId) {
        console.error('VITE_SITE_ID is missing in environment variables.');
        console.warn('[Peerbit Plugin Install] VITE_SITE_ID is missing. Using a default or allowing Site program to handle.');
      }
      siteId = DEFAULT_SITE_ID;

      console.log('[Peerbit Plugin Renderer] Running as Web App. Initializing local Peerbit node.');

      const localPeerbitClient = await Peerbit.create({
        directory: `./.peerbit/${siteId}`,
      });
      const siteProgram = await localPeerbitClient.open(new Site(siteId));
      console.log('[Peerbit Plugin Install] Peerbit.create finished.');
      console.log(`[Peerbit Plugin Renderer] Local Peerbit for web initialized. Peer ID: ${localPeerbitClient.peerId.toString()}`);
      peerbitServiceInstance = {
        getPublicKey: () => localPeerbitClient.identity.publicKey.toString(),
        dial: (address: string) => localPeerbitClient.dial(address),
        getPeerId: () => localPeerbitClient.peerId.toString(), // This is fine
        addRelease: (release: Release) => siteProgram.addRelease(release),
        getRelease: (id: string) => siteProgram.getRelease(id),
        getLatestReleases: (size?: number) => siteProgram.getLatestReleases(size),
      };

    }

    if (bootstrappersRaw) {
      const bootstrappers = bootstrappersRaw.split(',').map((b) => b.trim());
      console.log('[Peerbit Plugin Install] Dialing bootstrappers in parallel:', bootstrappers);
      const dialPromises = bootstrappers.map(async (bootstrapper) => {
        try {
          await peerbitServiceInstance.dial(bootstrapper);
          console.log(`[Peerbit Plugin Install] Dialed ${bootstrapper} successfully`);
        } catch (dialError) {
          console.error(`[Peerbit Plugin Install] Error dialing bootstrapper ${bootstrapper}:`, dialError);
        }
      });
      await Promise.allSettled(dialPromises);
      console.log('[Peerbit Plugin Install] Finished dialing bootstrappers.');
    } else {
      console.log('[Peerbit Plugin Install] No bootstrappers defined.');
    }

    // The async Release put/get test has been moved to src/index.ts to run after app mount.

    // console.log('[Peerbit Plugin Install] Attempting to open TrustedNetwork...');
    // const trustedNetworkInstance = new TrustedNetwork({
    //   rootTrust: peerbitClient.identity.publicKey,
    // });
    // const network = await peerbitClient.open(trustedNetworkInstance, {});
    // console.log(`[Peerbit Plugin Install] TrustedNetwork opened. Network address: ${network.address?.toString()}`);

    console.log('[Peerbit Plugin Install] Initializing utilities...');
    const timeUtility = hrtime();
    const logInstance = peerbitLogger({ module: 'peerbit-plugin' });

    console.log('[Peerbit Plugin Install] Providing Peerbit, SiteProgram, stores, and utilities to Vue app...');
    app.config.globalProperties.$peerbitService = peerbitServiceInstance;
    // app.config.globalProperties.$site = openedSiteProgram; // Provide the main Site program
    // app.config.globalProperties.$releasesStore = releaseStore; // Provide the releases store from Site
    // app.config.globalProperties.$usersStore = usersStore; // Provide the users store from Site
    // // For backwards compatibility, if some components still use $program or $documents
    // app.config.globalProperties.$program = openedSiteProgram;
    // app.config.globalProperties.$documents = releaseStore;
    // app.config.globalProperties.$peerbitReleaseStore = releaseStore;

    // app.config.globalProperties.$network = network;
    app.config.globalProperties.$time = timeUtility;
    app.config.globalProperties.$logger = logInstance;

    app.provide('peerbitService', peerbitServiceInstance);
    // app.provide('site', openedSiteProgram); // Provide the main Site program
    // app.provide('releasesStore', releaseStore);
    // app.provide('usersStore', usersStore);
    // // For backwards compatibility
    // app.provide('program', openedSiteProgram);
    // app.provide('documents', releaseStore);
    // app.provide('peerbitReleaseStore', releaseStore);

    // app.provide('network', network);
    app.provide('time', timeUtility);
    app.provide('logger', logInstance);
    console.log('[Peerbit Plugin Install] Peerbit, SiteProgram, stores, and utilities provided. Install function ending.');

    return peerbitServiceInstance; // Return the instance
  },
};
