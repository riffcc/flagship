import {createHelia} from 'helia';
import {json} from '@helia/json';
import {Peer} from '@peerbit/server'; // Changed Peerbit to Peer
import {DocumentStore} from '@peerbit/document';
import type {Release} from '@riffcc/peerbit-adapter/types';

let peerInstance: Peer | undefined; // Renamed for clarity

export async function startPeerbitNode() {
  if (peerInstance) {
    console.log('Peerbit node already started.');
    return peerInstance;
  }

  try {
    console.log('Starting Helia...');
    const helia = await createHelia();
    const heliaJson = json(helia);
    console.log('Helia started.');

    console.log('Starting Peerbit node...');
    peerInstance = await Peer.create(); // Create Peer instance
    await peerInstance.connect(heliaJson); // Connect Helia instance
    console.log('Peerbit node started. Peer ID:', peerInstance.peerId.toString());

    // Open a document store for releases
    // The type argument `Release` is used to ensure type safety
    const releasesStore = await peerInstance.open(new DocumentStore<Release>({id: 'releases'}));
    console.log('Releases store opened:', releasesStore.address?.toString());

    // Example: Listen for updates (optional, for debugging)
    releasesStore.events.addEventListener('change', event => {
      console.log('Releases store changed:', event.detail);
    });

    return peerInstance;
  } catch (error) {
    console.error('Failed to start Peerbit node:', error);
    throw error;
  }
}

export async function stopPeerbitNode() {
  if (peerInstance) {
    console.log('Stopping Peerbit node...');
    await peerInstance.stop();
    peerInstance = undefined;
    console.log('Peerbit node stopped.');
  }
}

// Ensure Peerbit node is stopped gracefully on app exit
import {app} from 'electron';

// Conditionally attach the 'will-quit' listener
// This prevents errors in test environments where `app.on` might not be defined on a mocked `app` object
if (typeof app.on === 'function') {
  app.on('will-quit', async () => {
    await stopPeerbitNode();
  });
}
