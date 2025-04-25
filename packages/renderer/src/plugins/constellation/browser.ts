import type {App} from 'vue';

import {mandataire} from 'constl-ipa-fork';
import {RIFFCC_PROTOCOL} from '/@/utils';

export default {
  install: (app: App) => {
    const client = mandataire.générerMandataireProc({protocoles: [RIFFCC_PROTOCOL]});
    app.config.globalProperties.$constl = client;
    app.provide('constl', client);
  },
};
