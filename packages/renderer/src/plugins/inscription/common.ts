/**
 * plugins/index.ts
 *
 * Automatically included in `./src/main.ts`
 */

// Plugins
import vuetify from '../vuetify';
import routeur from '../router';
import {loadFonts} from '../webfontloader';

import orbiter from '../orbiter';

// Types
import type {App} from 'vue';

export function registerPlugins(app: App) {
  loadFonts();
  app.use(routeur);
  app.use(vuetify);
  app.use(orbiter);
}
