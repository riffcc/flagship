import type {Browser, ElectronApplication, Page} from 'playwright';
import {chromium, _electron as electron, firefox, webkit} from 'playwright';

import path, {dirname} from 'path';
import {fileURLToPath} from 'url';


export const onElectron = async (): Promise<{
  appli: ElectronApplication;
  page: Page;
}> => {
  // Inclure {...process.env} est nécessaire pour les tests sur Linux
  const appli = await electron.launch({
    args: ['.'],
    env: {...process.env},
  });
  const page = await appli.firstWindow();

  return {appli, page};
};

export const onBrowser = async ({
  typeNavigateur,
}: {
  typeNavigateur: 'webkit' | 'chromium' | 'firefox';
}): Promise<{
  page: Page;
  browser: Browser;
}> => {
  let browser: Browser;
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

  const page = await browser.newPage();
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

  return {page, browser};
};
