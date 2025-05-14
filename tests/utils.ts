import type {Browser, Page, ElectronApplication} from 'playwright';
import {_electron as electron, chromium, firefox, webkit} from 'playwright';

import path, {dirname} from 'path';
import {fileURLToPath} from 'url';

export const onBrowser = async ({
  typeNavigateur,
}: {
  typeNavigateur: 'webkit' | 'chromium' | 'firefox' | 'electron';
}): Promise<{
  page: Page;
  browser?: Browser;
  electronApp?: ElectronApplication;
}> => {
  if (typeNavigateur === 'electron') {
    console.log('[E2E Utils] Launching Electron app (simplified options)...');
    const __dirname = dirname(fileURLToPath(import.meta.url));
    const projectRoot = path.join(__dirname, '..'); // Assumes utils.ts is in tests/ subdir of project root
    console.log(`[E2E Utils] Electron project root for CWD: ${projectRoot}`);
    
    const electronApp = await electron.launch({ 
        args: ['.'], // Tells Electron to start using the main script from package.json in the CWD
        cwd: projectRoot, // Set the Current Working Directory to the project root
        // env: { 
        //     ...process.env, 
        //     ELECTRON_ENABLE_LOGGING: 'true',
        //     ELECTRON_DEBUG_NOTIFICATIONS: 'true' // May show native notifications for some errors
        // }
    });
    console.log('[E2E Utils] Electron app launched. Waiting for first window...');

    // Temporarily commented out stdio listeners
    // electronApp.process().stdout?.on('data', (data) => {
    //   console.log(`[Electron App STDOUT]: ${data.toString()}`);
    // });
    // electronApp.process().stderr?.on('data', (data) => {
    //   console.error(`[Electron App STDERR]: ${data.toString()}`);
    // });

    const page = await electronApp.firstWindow();
    console.log('[E2E Utils] First window obtained.');
    return {page, electronApp};
  }

  let browser: Browser;
  switch (typeNavigateur) {
    case 'chromium':
      browser = await chromium.launch({
        args: ['--disable-web-security'],
      });
      break;
    case 'firefox':
      browser = await firefox.launch();
      break;
    case 'webkit':
      browser = await webkit.launch({
        args: ['--disable-web-security'],
      });
      break;
    default:
      throw new Error(`Unsupported browser type: ${typeNavigateur}`);
  }

  const page = await browser.newPage();
  const __dirname_browser = dirname(fileURLToPath(import.meta.url));
  const fichierHtml = path.join(
    __dirname_browser,
    '..',
    'packages',
    'renderer',
    'dist',
    'index.html',
  );

  await page.goto(`file://${fichierHtml}`);

  return {page, browser};
};
