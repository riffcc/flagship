import type {ElectronApplication, Page, Browser} from 'playwright';

import {afterAll, beforeAll, describe, expect, test} from 'vitest';

import {onBrowser, onElectron} from './utils';

const environnement = process.env.ENVIRONNEMENT_TESTS;

describe('Test app window', function () {
  let appliÉlectron: ElectronApplication | undefined = undefined;
  let browser: Browser | undefined = undefined;
  let page: Page;

  beforeAll(async () => {
    if (!environnement || environnement === 'électron') {
      ({appli: appliÉlectron, page} = await onElectron());
    } else if (['firefox', 'chromium', 'webkit'].includes(environnement)) {
      ({page, browser} = await onBrowser({
        typeNavigateur: environnement as 'firefox' | 'chromium' | 'webkit',
      }));
    } else {
      throw new Error(environnement);
    }
  });

  afterAll(async () => {
    if (appliÉlectron) {
      await appliÉlectron.close();
    }
    if (browser) {
      await browser.close();
    }
  });

  test('Main window state', async context => {
    if (!appliÉlectron) context.skip();
    const windowState: {isVisible: boolean; isDevToolsOpened: boolean; isCrashed: boolean} =
      await appliÉlectron!.evaluate(async ({BrowserWindow}) => {
        await new Promise<void>(résoudre => {
          const fVérif = () => {
            if (BrowserWindow.getAllWindows().length) {
              clearInterval(intervale);
              résoudre();
            }
          };
          const intervale = setInterval(fVérif, 1000);
          fVérif();
        });
        const mainWindow = BrowserWindow.getAllWindows()[0];

        const getState = () => ({
          isVisible: mainWindow.isVisible(),
          isDevToolsOpened: mainWindow.webContents.isDevToolsOpened(),
          isCrashed: mainWindow.webContents.isCrashed(),
        });

        return new Promise(resolve => {
          if (mainWindow.isVisible()) {
            resolve(getState());
          } else mainWindow.once('ready-to-show', () => setTimeout(() => resolve(getState()), 0));
        });
      });

    expect(windowState.isCrashed, 'The app has crashed').toBeFalsy();
    expect(windowState.isVisible, 'The main window was not visible').toBeTruthy();
    expect(windowState.isDevToolsOpened, 'The DevTools panel was open').toBeFalsy();
  });

  test('Main window web content', async context => {
    if (!appliÉlectron) context.skip();

    const element = await page.$('#app', {strict: true});
    expect(element, 'Was unable to find the root element').toBeDefined();
    expect((await element!.innerHTML()).trim(), 'Window content was empty').not.equal('');
  });
});

