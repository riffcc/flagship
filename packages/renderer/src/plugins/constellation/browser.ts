import type {App} from 'vue';

import {mandataire} from '@constl/ipa';
import {RIFFCC_PROTOCOL} from '/@/utils';

const DOMAINS = process.env.VITE_DOMAINS?.split(',').map(d=>d.trim());


export default {
  install: (app: App) => {
    const client = mandataire.générerMandataireProc({
      protocoles: [RIFFCC_PROTOCOL], 
      
      // Add custom domains (only for peer reachable on this domain)
      domaines: DOMAINS,
      
      // Add custom peers (other peers to connect to automatically)
      pairsParDéfaut: process.env.VITE_BOOTSTRAPPERS?.split(',').map(b=>b.trim()),
    });
    app.config.globalProperties.$constl = client;
    app.provide('constl', client);
  },
};
