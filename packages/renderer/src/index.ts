import { createApp, type App as VueApp } from 'vue';
import App from './App.vue';
import routeur from './plugins/router';
import vuetify from './plugins/vuetify';
import {pinia} from './plugins/pinia';
import lensService from './plugins/lensService';

  const app: VueApp = createApp(App);

  app.use(routeur);
  app.use(vuetify);
  app.use(pinia);
  app.use(lensService);

  app.mount('#app');
