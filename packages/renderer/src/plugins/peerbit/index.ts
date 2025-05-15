import type { App } from 'vue';
import { Peerbit } from 'peerbit';
import { Documents } from '@peerbit/document';
import { TrustedNetwork } from '@peerbit/trusted-network';
import { Program } from '@peerbit/program';
import { variant, field, option } from '@dao-xyz/borsh';
import { Release } from './schema'; // Import Release schema

// Utilities
import { hrtime } from '@peerbit/time';
import { logger as peerbitLogger } from '@peerbit/logger';

console.log('[Peerbit Plugin File] Loaded');

// MyDocumentsProgram now uses Release schema
@variant("releases_program") // Changed variant name for clarity
class MyDocumentsProgram extends Program {
  @field({ type: Documents })
  store: Documents<Release>; // Store is now Documents<Release>

  constructor() {
    super();
    // Initialize with a specific type argument for Release
    this.store = new Documents<Release>();
  }

  async open(args?: any): Promise<void> {
    await this.store.open({
      type: Release, // Use Release schema type
      // No `index` field here, aligning with flagship's Site.releases.open() and to avoid linter error.
      // Default indexing (e.g., by document ID) will apply, or other options can be added if needed.
      // Example options from flagship that could be relevant if defaults aren't enough:
      // idProperty: RELEASE_ID_PROPERTY, (If 'id' is not the default primary key for queries)
      // canPerform: async () => true, (For access control)
      // replicate: true or specific replication options from ReplicationOptions
    });
  }
}

export default {
  install: async (app: App) => {
    console.log('[Peerbit Plugin Install] Starting');
    const siteId = import.meta.env.VITE_SITE_ID as string;
    const bootstrappersRaw = import.meta.env.VITE_BOOTSTRAPPERS as string | undefined;

    if (!siteId) {
      console.error('[Peerbit Plugin Install] VITE_SITE_ID is missing.');
    }

    const peerbitClient = await Peerbit.create({
      directory: `./.peerbit/${siteId || 'default_site_id'}`,
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

    console.log('[Peerbit Plugin Install] Attempting to open MyDocumentsProgram (for Releases)...');
    const myDocumentsProgramInstance = new MyDocumentsProgram();
    const openedReleaseProgram = await peerbitClient.open(myDocumentsProgramInstance);
    const releaseStore = openedReleaseProgram.store;

    console.log(`[Peerbit Plugin Install] MyDocumentsProgram opened. Release store address: ${releaseStore.address?.toString()}`);
    if (!releaseStore || releaseStore.closed) {
      console.error('[Peerbit Plugin Install] CRITICAL: Release store is not open or undefined!');
      // throw new Error("Failed to open release store"); // Option to halt if critical
    } else {
      console.log('[Peerbit Plugin Install] Release store seems to be open.');

      // --- Test: Put a Release document ---
      try {
        console.log('[Peerbit Plugin Install] Attempting to put a new Release...');
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

        // --- Test: Get the Release document ---
        const retrievedRelease = await releaseStore.index.get(newRelease.id);
        if (retrievedRelease) {
          console.log(`[Peerbit Plugin Install] Successfully retrieved Release: ${retrievedRelease.id} - ${retrievedRelease.name}`);
          // Custom JSON.stringify replacer to handle BigInt
          const replacer = (key: string, value: any) => 
            typeof value === 'bigint' ? value.toString() : value;
          console.log('[Peerbit Plugin Install] Retrieved Release Data:', JSON.stringify(retrievedRelease, replacer, 2));
        } else {
          console.error(`[Peerbit Plugin Install] Failed to retrieve Release by ID: ${newRelease.id}`);
        }
      } catch (error) {
        console.error('[Peerbit Plugin Install] Error during Release put/get test:', error);
      }
      // --- End Test ---
    }

    console.log('[Peerbit Plugin Install] Attempting to open TrustedNetwork...');
    const trustedNetworkInstance = new TrustedNetwork({
      rootTrust: peerbitClient.identity.publicKey
    });
    const network = await peerbitClient.open(trustedNetworkInstance, {}); // Pass empty options for now
    console.log(`[Peerbit Plugin Install] TrustedNetwork opened. Network address: ${network.address?.toString()}`);

    console.log('[Peerbit Plugin Install] Initializing utilities...');
    const timeUtility = hrtime();
    const logInstance = peerbitLogger({ module: 'peerbit-plugin' });

    console.log('[Peerbit Plugin Install] Providing Peerbit, programs, and utilities to Vue app...');
    app.config.globalProperties.$peerbit = peerbitClient;
    app.config.globalProperties.$program = openedReleaseProgram; // Provide the releases program
    app.config.globalProperties.$documents = releaseStore; // Provide the actual release store
    app.config.globalProperties.$peerbitReleaseStore = releaseStore; // Specifically provide the release store
    app.config.globalProperties.$network = network;
    app.config.globalProperties.$time = timeUtility;
    app.config.globalProperties.$logger = logInstance;

    app.provide('peerbit', peerbitClient);
    app.provide('program', openedReleaseProgram);
    app.provide('documents', releaseStore);
    app.provide('peerbitReleaseStore', releaseStore); // Specifically provide the release store
    app.provide('network', network);
    app.provide('time', timeUtility);
    app.provide('logger', logInstance);
    console.log('[Peerbit Plugin Install] Peerbit, programs, and utilities provided. Install function ending.');
  },
};
