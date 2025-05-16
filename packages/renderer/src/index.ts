import { createApp, type App as VueApp } from 'vue';
import App from './App.vue';
import routeur from './plugins/router';
import vuetify from './plugins/vuetify';
import {pinia} from './plugins/pinia';
import peerbitPlugin from './plugins/peerbit';

function initializeAndMountApp() {
  const app: VueApp = createApp(App);

  app.use(routeur);
  app.use(vuetify);
  app.use(pinia);
  app.use(peerbitPlugin);
  app.mount('#app');
  console.log('[AppInit] Vue App mounted.');
}

initializeAndMountApp();
