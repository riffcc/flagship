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
      console.log('[Peerbit Plugin Install] Dialing bootstrappers:', bootstrappers);
      for (const bootstrapper of bootstrappers) {
        try {
          await peerbitServiceInstance.dial(bootstrapper);
          console.log(`[Peerbit Plugin Install] Dialed ${bootstrapper} successfully`);
        } catch (dialError) {
          console.error(`[Peerbit Plugin Install] Error dialing bootstrapper ${bootstrapper}:`, dialError);
        }
      }
    } else {
      console.log('[Peerbit Plugin Install] No bootstrappers defined.');
    }
    try {
      console.log('[Peerbit Plugin Install] Attempting to put a new Release...');
      const newRelease = new Release({
        name: 'RiP!: A Remix Manifesto',
        categoryId: 'movie',
        contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
        thumbnailCID: 'Qmb3eeESRoX5L6NhTYLEtFFUS1FZgqe1e7hdBk2f57DUGh',
        metadata: JSON.stringify({
          classification: 'PG',
          description: 'Join filmmaker Brett Gaylor and mashup artist Girl Talk as they explore copyright and content creation in the digital age. In the process they dissect the media landscape of the 21st century and shatter the wall between users and producers.',
          duration: '1h 26m',
          author: 'Brett Gaylor',
          cover: 'QmcD4R3Qj8jBWY73H9LQWESgonNB1AMN3of23ubjDhJVSm',
        }),
      });

      const result = await peerbitServiceInstance.addRelease(newRelease);
      console.log(`[Peerbit Plugin Install] Successfully put Release: ${newRelease.id} - ${newRelease.name}`);
      console.log(`[Peerbit Plugin Install] Entry hash: ${result}`);

      // --- Test: Get the Release document ---
      const retrievedRelease = await peerbitServiceInstance.getRelease(newRelease.id);
      if (retrievedRelease) {
        console.log(`[Peerbit Plugin Install] Successfully retrieved Release: ${retrievedRelease.id} - ${retrievedRelease.name}`);
        const replacer = (_key: string, value: unknown) =>
          typeof value === 'bigint' ? value.toString() : value;
        console.log('[Peerbit Plugin Install] Retrieved Release Data:', JSON.stringify(retrievedRelease, replacer, 2));
      } else {
        console.error(`[Peerbit Plugin Install] Failed to retrieve Release by ID: ${newRelease.id}`);
      }
    } catch (error) {
      console.error('[Peerbit Plugin Install] Error during Release put/get test:', error);
    }
    // --- End Test ---

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
  },
};
