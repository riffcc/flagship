import type {Browser, Page, ElectronApplication} from 'playwright';
import {afterAll, beforeAll, describe, expect, test} from 'vitest';
import {onBrowser} from './utils';

const environnement = process.env.ENVIRONNEMENT_TESTS;

describe('Test app window', function () {
  let browser: Browser | undefined = undefined;
  let electronApp: ElectronApplication | undefined = undefined;
  let page: Page;

  beforeAll(async () => {
    const testEnvironment = (environnement || 'electron') as 'firefox' | 'chromium' | 'webkit' | 'electron';

    if (testEnvironment === 'electron') {
      const result = await onBrowser({ typeNavigateur: 'electron' });
      page = result.page;
      electronApp = result.electronApp;
    } else if (['firefox', 'chromium', 'webkit'].includes(testEnvironment)) {
      const result = await onBrowser({ typeNavigateur: testEnvironment as 'firefox' | 'chromium' | 'webkit' });
      page = result.page;
      browser = result.browser;
    } else {
      throw new Error(`Unsupported test environment: ${environnement}. Must be 'firefox', 'chromium', 'webkit', or 'electron'.`);
    }

    // Listen for console messages from the page to aid debugging
    page.on('console', msg => {
      const msgType = msg.type().toUpperCase();
      const msgText = msg.text();
      console.log(`E2E BROWSER CONSOLE [${msgType}]: ${msgText}`);
      // Log arguments if any, useful for complex objects or multiple args
      if (msg.args().length > 0) {
        for (let i = 0; i < msg.args().length; ++i) {
          // Attempt to get a string representation of the argument
          msg.args()[i].jsonValue().then(value => {
            console.log(`  ARG ${i}:`, value);
          }).catch(() => {
            // Fallback if jsonValue fails (e.g., for non-serializable objects like functions)
            console.log(`  ARG ${i}: (Could not serialize)`);
          });
        }
      }
    });
  });

  afterAll(async () => {
    if (electronApp) {
      await electronApp.close();
    } else if (browser) {
      await browser.close();
    }
  });

  test('Main window web content', async () => {
    const element = await page.$('#app', {strict: true});
    expect(element, 'Was unable to find the root element').toBeDefined();
    expect((await element!.innerHTML()).trim(), 'Window content was empty').not.equal('');
  });

});

