import {Orbiter, consts} from '@riffcc/orbiter';
import { créerConstellation } from 'constl-ipa-fork';
import type {App} from 'vue';

export default {
  install: (app: App) => {
    const siteId = import.meta.env.VITE_SITE_ID;
    const bootstrappers = import.meta.env.VITE_BOOTSTRAPPERS as string | undefined;

    let orbiterApp: Orbiter | undefined = undefined;
    if (siteId) {
      const constellation = créerConstellation({
        pairsParDéfaut: bootstrappers ? bootstrappers.split(',').map(b=>b.trim()) : undefined,
        protocoles: [consts.RIFFCC_PROTOCOL],
      });
      orbiterApp = new Orbiter({
        siteId,
        constellation,
      });
    } else {
      throw new Error('VITE_SITE_ID is missing, please check the .env or generate a new one with orb export-config');
    }

    app.config.globalProperties.$orbiter = orbiterApp;
    app.provide('orbiter', orbiterApp);
  },
};

