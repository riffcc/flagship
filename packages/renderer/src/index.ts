import {createApp} from 'vue';
import App from './App.vue';
import routeur from './plugins/router';
import vuetify from './plugins/vuetify';
import {pinia} from './plugins/pinia';
import peerbitPlugin from './plugins/peerbit';
import { useReleasesStore } from './stores/releases';
// import { registerPlugins } from './plugins/inscription/common'; // Removed

async function initializeAndMountApp() {
  const app = createApp(App);

  // registerPlugins(app); // Removed
  app.use(routeur);
  app.use(vuetify);
  app.use(pinia);

  // Manually invoke and await the plugin's install method
  if (peerbitPlugin && typeof peerbitPlugin.install === 'function') {
    try {
      console.log('[AppInit] Manually invoking and awaiting Peerbit plugin install method...');
      await peerbitPlugin.install(app); // Call install directly
      console.log('[AppInit] Peerbit plugin install method completed.');
      // Pre-fetch releases before mounting to reduce flash
      const releasesStore = useReleasesStore();
      releasesStore.fetchReleasesFromPeerbit();
      // If the plugin also needs to be registered for other Vue functionalities (e.g. global components)
      // and app.use() is idempotent or guarded against re-execution of install logic,
      // you might call it here. For now, assuming install handles all setup including provide.
      // app.use(peerbitPlugin); // Potentially call this if needed after await
    } catch (error) {
      console.error('[AppInit] Error during manual Peerbit plugin install:', error);
    }
  } else {
    console.error('[AppInit] Peerbit plugin or its install method is not defined correctly.');
  }

  app.mount('#app');
  console.log('[AppInit] Vue App mounted.');
}

initializeAndMountApp();
