import {GestionnaireFenêtres} from '@constl/mandataire-electron-principal';
import {RIFFCC_PROTOCOL} from '/@/consts';

const enDéveloppement = import.meta.env.DEV;

const importationIPA = import('@constl/ipa');
const importationServeur = import('@constl/serveur');


export const gestionnaireFenêtres = new GestionnaireFenêtres({
  enDéveloppement,
  importationIPA,
  importationServeur,
  journal: enDéveloppement ? console.log : undefined,
  opts: {
    protocoles: [RIFFCC_PROTOCOL], 

    // Useful for running tests with a temporary folder
    dossier: process.env.DOSSIER_CONSTL,

    // Add custom domains (only for peer reachable on this domain)
    domaines: process.env.VITE_DOMAINS?.split(',').map(d=>d.trim()),

    // Add custom peers (other peers to connect to automatically)
    pairsParDéfaut: process.env.VITE_BOOTSTRAPPERS?.split(',').map(b=>b.trim()),
  },
});
