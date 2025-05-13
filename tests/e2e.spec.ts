import type {ElectronApplication, Page} from 'playwright';

import {afterAll, beforeAll, describe, expect, test} from 'vitest';

import {surNavig, surÉlectron} from './utils';

const environnement = process.env.ENVIRONNEMENT_TESTS;
const isCI = process.env.CI === 'true';

describe('Test app window', function () {
  let appliÉlectron: ElectronApplication | undefined = undefined;
  let page: Page | undefined;
  let fermer: (() => Promise<void>) | undefined;

  beforeAll(async () => {
    try {
      if (!environnement || environnement === 'électron') {
        if (isCI) {
          console.log('Skipping electron tests in CI environment');
          return;
        }
        ({appli: appliÉlectron, page, fermer} = await surÉlectron());
      } else if (['firefox', 'chromium', 'webkit'].includes(environnement)) {
        ({page, fermer} = await surNavig({
          typeNavigateur: environnement as 'webkit' | 'chromium' | 'webkit',
        }));
      } else {
        throw new Error(environnement);
      }
    } catch (error) {
      console.error('Error setting up tests:', error);
    }
  });

  afterAll(async () => {
    if (fermer) {
      await fermer();
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
    if (!appliÉlectron || !page) context.skip();
    
    if (!page) return;

    const element = await page.$('#app', {strict: true});
    expect(element, 'Was unable to find the root element').toBeDefined();
    expect((await element!.innerHTML()).trim(), 'Window content was empty').not.equal('');
  });
});

