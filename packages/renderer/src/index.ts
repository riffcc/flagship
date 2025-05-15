import { createApp, type App as VueApp } from 'vue';
import App from './App.vue';
import routeur from './plugins/router';
import vuetify from './plugins/vuetify';
import {pinia} from './plugins/pinia';
import peerbitPlugin from './plugins/peerbit';
import { useReleasesStore } from './stores/releases';
import type { IPeerbitService } from '/@/lib/types';
import { Release } from '/@/lib/schema';
// import { registerPlugins } from './plugins/inscription/common'; // Removed

async function runPeerbitPostMountTasks(peerbitServiceInstance: IPeerbitService | undefined) {
  if (!peerbitServiceInstance) {
    console.error('[AppInit] Peerbit service instance not available for post-mount tasks.');
    return;
  }
  // Run Release put/get test asynchronously
  try {
    console.log('[AppInit] Starting async Release put/get test (post-mount)...');
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
    console.log(`[AppInit] Async Test (post-mount): Successfully put Release: ${newRelease.id} - ${newRelease.name}`);
    console.log(`[AppInit] Async Test (post-mount): Entry hash: ${result}`);

    const retrievedRelease = await peerbitServiceInstance.getRelease(newRelease.id);
    if (retrievedRelease) {
      console.log(`[AppInit] Async Test (post-mount): Successfully retrieved Release: ${retrievedRelease.id} - ${retrievedRelease.name}`);
      const replacer = (_key: string, value: unknown) =>
        typeof value === 'bigint' ? value.toString() : value;
      console.log('[AppInit] Async Test (post-mount): Retrieved Release Data:', JSON.stringify(retrievedRelease, replacer, 2));
    } else {
      console.error(`[AppInit] Async Test (post-mount): Failed to retrieve Release by ID: ${newRelease.id}`);
    }
  } catch (error) {
    console.error('[AppInit] Error during async Release put/get test (post-mount):', error);
  }
}

async function initializeAndMountApp() {
  const app: VueApp = createApp(App);

  // registerPlugins(app); // Removed
  app.use(routeur);
  app.use(vuetify);
  app.use(pinia);

  let peerbitServiceInstance: IPeerbitService | undefined;

  // Manually invoke and await the plugin's install method
  if (peerbitPlugin && typeof peerbitPlugin.install === 'function') {
    try {
      console.log('[AppInit] Manually invoking and awaiting Peerbit plugin install method...');
      // Cast to any to acknowledge the dynamic return type, then to IPeerbitService
      peerbitServiceInstance = await peerbitPlugin.install(app) as IPeerbitService;
      console.log('[AppInit] Peerbit plugin install method completed.');

      // Pre-fetch releases before mounting to reduce flash
      const releasesStore = useReleasesStore();
      await releasesStore.fetchReleasesFromPeerbit(); // Ensure fetch completes

    } catch (error) {
      console.error('[AppInit] Error during manual Peerbit plugin install or initial fetch:', error);
    }
  } else {
    console.error('[AppInit] Peerbit plugin or its install method is not defined correctly.');
  }

  app.mount('#app');
  console.log('[AppInit] Vue App mounted.');

  // Run post-mount tasks
  runPeerbitPostMountTasks(peerbitServiceInstance);
}

initializeAndMountApp();
