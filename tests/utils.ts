import type {Browser, ElectronApplication, Page} from 'playwright';
import {chromium, _electron as electron, firefox, webkit} from 'playwright';
import electronExecutablePath from 'electron';

import {dossiers} from '@constl/utils-tests';
import path, {dirname} from 'path';
import {fileURLToPath} from 'url';


export const surÉlectron = async (): Promise<{
  appli: ElectronApplication;
  page: Page;
  fermer: () => Promise<void>;
}> => {
  // Utiliser un dossier temporaire pour le compte Constellation dans les tests
  const {dossier, fEffacer} = await dossiers.dossierTempo();
  let appli: ElectronApplication;

  try {
    // Inclure {...process.env} est nécessaire pour les tests sur Linux
    const entryPoint = path.join(process.cwd(), 'packages', 'main', 'dist', 'index.cjs');
    appli = await electron.launch({
      args: [entryPoint],
      env: {...process.env, DOSSIER_CONSTL: dossier},
    });
  } catch (error) {
    console.error('Electron launch failed:', error);
    // Re-throw the error to fail the test setup clearly
    throw new Error(`Electron launch failed: ${error.message}`);
  }

  const page = await appli.firstWindow();

  const fermer = async () => {
    try {
      await appli.close();
    } finally {
      fEffacer?.();
    }
  };

  return {appli, page, fermer};
};

export const surNavig = async ({
  typeNavigateur,
}: {
  typeNavigateur: 'webkit' | 'chromium' | 'firefox';
}): Promise<{
  page: Page;
  fermer: () => Promise<void>;
}> => {
  let navigateur: Browser;
  switch (typeNavigateur) {
    case 'chromium':
      navigateur = await chromium.launch({
        args: ['--disable-web-security'],
        //        headless: false,
      });
      break;
    case 'firefox':
      navigateur = await firefox.launch();
      break;
    case 'webkit':
      navigateur = await webkit.launch({
        args: ['--disable-web-security'],
      });
      break;
    default:
      throw new Error(typeNavigateur);
  }

  const page = await navigateur.newPage();
  const __dirname = dirname(fileURLToPath(import.meta.url));
  const fichierHtml = path.join(
    __dirname,
    '..',
    'packages',
    'renderer',
    'dist',
    'web',
    'index.html',
  );

  await page.goto(`file://${fichierHtml}`);

  const fermer = async () => {
    await page.close();
    await navigateur.close();
  };

  return {page, fermer};
};
