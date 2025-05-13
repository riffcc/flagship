import type {Browser, ElectronApplication, Page} from 'playwright';
import {chromium, _electron as electron, firefox, webkit} from 'playwright';

import {dossiers} from '@constl/utils-tests';
import path, {dirname} from 'path';
import {fileURLToPath} from 'url';


export const surÉlectron = async (): Promise<{
  appli: ElectronApplication;
  page: Page;
  fermer: () => Promise<void>;
}> => {
  try {
    // Utiliser un dossier temporaire pour le compte Constellation dans les tests
    const {dossier, fEffacer} = await dossiers.dossierTempo();

    // Inclure {...process.env} est nécessaire pour les tests sur Linux
    const appli = await electron.launch({
      args: ['.'],
      env: {...process.env, DOSSIER_CONSTL: dossier},
      timeout: 30000, // Increase timeout for slower CI environments
    });
    
    const page = await appli.firstWindow();

    const fermer = async () => {
      try {
        await appli.close();
      } catch (error) {
        console.error('Error closing electron app:', error);
      } finally {
        fEffacer?.();
      }
    };

    return {appli, page, fermer};
  } catch (error) {
    console.error('Error launching electron:', error);
    const dummyPage = {} as Page;
    const dummyApp = {} as ElectronApplication;
    return {
      appli: dummyApp,
      page: dummyPage,
      fermer: async () => { console.log('Dummy close function called'); },
    };
  }
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
