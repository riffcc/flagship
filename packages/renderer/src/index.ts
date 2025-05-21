import { createApp, type App as VueApp } from 'vue';
import { VueQueryPlugin } from '@tanstack/vue-query';
import App from './App.vue';
import routerPlugin from './plugins/router';
import vuetifyPlugin from './plugins/vuetify';
import lensServicePlugin from './plugins/lensService';
import { loadFonts } from './plugins/webfontloader';

const app: VueApp = createApp(App);

const installPlugins = () => {
  loadFonts();
  app.use(routerPlugin);
  app.use(vuetifyPlugin);
  app.use(VueQueryPlugin);
  app.use(lensServicePlugin);
};

installPlugins();
app.mount('#app');
