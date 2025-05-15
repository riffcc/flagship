import type { App } from 'vue';
import { Peerbit } from 'peerbit';
// import { Documents } from '@peerbit/document'; // No longer needed directly here
import { TrustedNetwork } from '@peerbit/trusted-network';
// import { Program } from '@peerbit/program'; // No longer needed directly here
// import { variant, field, option } from '@dao-xyz/borsh'; // No longer needed directly here
import { Release, Site, Account, AccountType } from './schema'; // Import Site, Account, AccountType

// Utilities
import { hrtime } from '@peerbit/time';
import { logger as peerbitLogger } from '@peerbit/logger';

console.log('[Peerbit Plugin File] Loaded');

// MyDocumentsProgram is removed as Site program will handle releases and users

export default {
  install: async (app: App) => {
    console.log('[Peerbit Plugin Install] Starting');
    const siteIdFromEnv = import.meta.env.VITE_SITE_ID as string;
    const bootstrappersRaw = import.meta.env.VITE_BOOTSTRAPPERS as string | undefined;

    if (!siteIdFromEnv) {
      console.warn('[Peerbit Plugin Install] VITE_SITE_ID is missing. Using a default or allowing Site program to handle.');
      // The Site program constructor in schema.ts expects a siteId, 
      // but we can default it there or pass a generic one if not critical for anonymous browsing.
      // For now, we will pass 'default_site_id' if VITE_SITE_ID is not present.
    }
    const siteId = siteIdFromEnv || 'default_site_id'; // Ensure siteId has a value for Peerbit dir and Site program

    const peerbitClient = await Peerbit.create({
      directory: `./.peerbit/${siteId}`,
    });
    console.log('[Peerbit Plugin Install] Peerbit.create finished.');

    if (bootstrappersRaw) {
      const bootstrappers = bootstrappersRaw.split(',').map((b) => b.trim());
      console.log('[Peerbit Plugin Install] Dialing bootstrappers:', bootstrappers);
      for (const bootstrapper of bootstrappers) {
        try {
          await peerbitClient.dial(bootstrapper);
          console.log(`[Peerbit Plugin Install] Dialed ${bootstrapper} successfully`);
        } catch (dialError) {
          console.error(`[Peerbit Plugin Install] Error dialing bootstrapper ${bootstrapper}:`, dialError);
        }
      }
    } else {
      console.log('[Peerbit Plugin Install] No bootstrappers defined.');
    }

    console.log('[Peerbit Plugin Install] Attempting to open SiteProgram...');
    const siteProgramInstance = new Site(siteId); // Pass siteId to Site program constructor
    const openedSiteProgram = await peerbitClient.open(siteProgramInstance);
    
    const releaseStore = openedSiteProgram.releases;
    const usersStore = openedSiteProgram.users;

    console.log(`[Peerbit Plugin Install] SiteProgram opened. Release store address: ${releaseStore.address?.toString()}, Users store address: ${usersStore.address?.toString()}`);

    if (!releaseStore || releaseStore.closed) {
      console.error('[Peerbit Plugin Install] CRITICAL: Release store from SiteProgram is not open or undefined!');
    } else {
      console.log('[Peerbit Plugin Install] Release store (from SiteProgram) seems to be open.');
      // Existing Release put/get test can remain, using 'releaseStore'
      try {
        console.log('[Peerbit Plugin Install] Attempting to put a new Release (via SiteProgram)...');
        const newRelease = new Release(
          'RiP!: A Remix Manifesto',
          'movie',
          'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
          'Qmb3eeESRoX5L6NhTYLEtFFUS1FZgqe1e7hdBk2f57DUGh',
          JSON.stringify({
            classification: 'PG',
            description: 'Join filmmaker Brett Gaylor and mashup artist Girl Talk as they explore copyright and content creation in the digital age. In the process they dissect the media landscape of the 21st century and shatter the wall between users and producers.',
            duration: '1h 26m',
            author: 'Brett Gaylor',
            cover: 'QmcD4R3Qj8jBWY73H9LQWESgonNB1AMN3of23ubjDhJVSm'
          })
        );
        await releaseStore.put(newRelease);
        console.log(`[Peerbit Plugin Install] Successfully put Release: ${newRelease.id} - ${newRelease.name}`);

        const retrievedRelease = await releaseStore.index.get(newRelease.id);
        if (retrievedRelease) {
          console.log(`[Peerbit Plugin Install] Successfully retrieved Release: ${retrievedRelease.id} - ${retrievedRelease.name}`);
          const replacer = (key: string, value: any) => typeof value === 'bigint' ? value.toString() : value;
          console.log('[Peerbit Plugin Install] Retrieved Release Data:', JSON.stringify(retrievedRelease, replacer, 2));
        } else {
          console.error(`[Peerbit Plugin Install] Failed to retrieve Release by ID: ${newRelease.id}`);
        }
      } catch (error) {
        console.error('[Peerbit Plugin Install] Error during Release put/get test:', error);
      }
    }

    if (!usersStore || usersStore.closed) {
      console.error('[Peerbit Plugin Install] CRITICAL: Users store from SiteProgram is not open or undefined!');
    } else {
      console.log('[Peerbit Plugin Install] Users store (from SiteProgram) seems to be open.');
      // TODO: Add a test for putting/getting an Account if desired
      // Example: 
      // try {
      //   const newAccount = new Account(peerbitClient.identity.publicKey, 'Test User', AccountType.USER);
      //   await usersStore.put(newAccount);
      //   const retrievedAccount = await usersStore.index.get(newAccount.id);
      //   if(retrievedAccount) console.log('Successfully put and got test account:', retrievedAccount.name);
      // } catch (e) { console.error('Error testing user store:', e); }
    }

    console.log('[Peerbit Plugin Install] Attempting to open TrustedNetwork...');
    const trustedNetworkInstance = new TrustedNetwork({
      rootTrust: peerbitClient.identity.publicKey
    });
    const network = await peerbitClient.open(trustedNetworkInstance, {});
    console.log(`[Peerbit Plugin Install] TrustedNetwork opened. Network address: ${network.address?.toString()}`);

    console.log('[Peerbit Plugin Install] Initializing utilities...');
    const timeUtility = hrtime();
    const logInstance = peerbitLogger({ module: 'peerbit-plugin' });

    console.log('[Peerbit Plugin Install] Providing Peerbit, SiteProgram, stores, and utilities to Vue app...');
    app.config.globalProperties.$peerbit = peerbitClient;
    app.config.globalProperties.$site = openedSiteProgram; // Provide the main Site program
    app.config.globalProperties.$releasesStore = releaseStore; // Provide the releases store from Site
    app.config.globalProperties.$usersStore = usersStore; // Provide the users store from Site
    // For backwards compatibility, if some components still use $program or $documents
    app.config.globalProperties.$program = openedSiteProgram; 
    app.config.globalProperties.$documents = releaseStore; 
    app.config.globalProperties.$peerbitReleaseStore = releaseStore; 

    app.config.globalProperties.$network = network;
    app.config.globalProperties.$time = timeUtility;
    app.config.globalProperties.$logger = logInstance;

    app.provide('peerbit', peerbitClient);
    app.provide('site', openedSiteProgram); // Provide the main Site program
    app.provide('releasesStore', releaseStore);
    app.provide('usersStore', usersStore);
    // For backwards compatibility
    app.provide('program', openedSiteProgram);
    app.provide('documents', releaseStore);
    app.provide('peerbitReleaseStore', releaseStore);

    app.provide('network', network);
    app.provide('time', timeUtility);
    app.provide('logger', logInstance);
    console.log('[Peerbit Plugin Install] Peerbit, SiteProgram, stores, and utilities provided. Install function ending.');
  },
};
