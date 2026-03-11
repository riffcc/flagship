// Types
import type { App } from 'vue';
import vuetify from './vuetify';
import router from './router';
import lensServicePlugin from './lensService';
import { VueQueryPlugin } from '@tanstack/vue-query';
import { queryClient } from './tanstackQuery';
import unifiedNetworkService from './network';

export function registerPlugins (app: App) {
  app
    .use(vuetify)
    .use(router)
    .use(VueQueryPlugin, { queryClient })
    .use(lensServicePlugin)
    .use(unifiedNetworkService);
}
