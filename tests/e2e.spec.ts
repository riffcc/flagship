import type {Browser, Page, ElectronApplication} from 'playwright';
import {afterAll, beforeAll, describe, expect, test} from 'vitest';
import {onBrowser} from './utils';

// Define the type for the release data to be sent, matching preload's ReleaseDataType
type ReleaseDataTypeForTest = {
  name: string;
  file: string; // CID
  author: string;
  category: string;
  thumbnail?: string;
  cover?: string;
  metadata?: Record<string, unknown> | string;
};

// Define the expected response structure from the IPC call, matching preload's AddReleaseResponseType
type AddReleaseResponseTypeForTest = {
  success: boolean;
  message?: string;
  error?: string;
};

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

  test('Upload a CID to Peerbit via peerbitAPI', async () => {
    const testCID = 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q';
    const releaseData: ReleaseDataTypeForTest = {
      name: 'Test Release E2E',
      file: testCID,
      author: 'E2E Test',
      category: 'Test Category',
      metadata: {info: 'E2E test upload'},
    };

    // Ensure the page is fully loaded and peerbitAPI is available
    try {
      await page.waitForFunction(() => {
        // This console.log runs in the browser context, captured by page.on('console')
        console.log('E2E polling: typeof window.peerbitAPI =', typeof (window as any).peerbitAPI);
        return (window as any).peerbitAPI !== undefined;
      }, {timeout: 15000}); // Slightly increased timeout for debugging
      console.log('E2E Test: window.peerbitAPI became available.');
    } catch (e) {
      const apiType = await page.evaluate(() => typeof (window as any).peerbitAPI);
      // Attempt to get more info about what is on the window object
      const windowKeys = await page.evaluate(() => Object.keys(window));
      console.error(`E2E Test Error: window.peerbitAPI (last known type: ${apiType}) did not become available. Window keys: ${windowKeys.join(', ')}`);
      throw new Error(`window.peerbitAPI did not become available within 15 seconds. Last known type: ${apiType}. Original error: ${e}`);
    }

    const result = await page.evaluate(
      async data => {
        // @ts-expect-error peerbitAPI is exposed by preload and should be on window
        return window.peerbitAPI.addRelease(data);
      },
      releaseData as any, // Cast to any if type mismatch with evaluate's expectations
    );

    expect(result, 'API call result should be defined').toBeDefined();
    const typedResult = result as AddReleaseResponseTypeForTest; // Cast for type safety

    expect(typedResult.success, `Upload failed: ${typedResult.error || typedResult.message}`).toBe(
      true,
    );
    expect(typedResult.message).toContain('Release "Test Release E2E" added successfully.');
  });

  test('Ping __verySimpleAPI', async () => {
    // Ensure the page is fully loaded and the minimal API is available
    try {
      await page.waitForFunction(() => {
        // This console.log runs in the browser context, captured by page.on('console')
        console.log('E2E polling: typeof window.__verySimpleAPI =', typeof (window as any).__verySimpleAPI);
        return (window as any).__verySimpleAPI !== undefined && typeof (window as any).__verySimpleAPI.ping === 'function';
      }, {timeout: 15000}); 
      console.log('E2E Test: window.__verySimpleAPI became available.');
    } catch (e) {
      const apiType = await page.evaluate(() => typeof (window as any).__verySimpleAPI);
      const windowKeys = await page.evaluate(() => Object.keys(window));
      console.error(`E2E Test Error: window.__verySimpleAPI (last known type: ${apiType}) did not become available. Window keys: ${windowKeys.join(', ')}`);
      throw new Error(`window.__verySimpleAPI did not become available within 15 seconds. Last known type: ${apiType}. Original error: ${e}`);
    }

    const result = await page.evaluate(async () => {
        // @ts-expect-error __verySimpleAPI is exposed by preload and should be on window
        return (window as any).__verySimpleAPI.ping();
      },
    );

    expect(result, 'API call result should be defined').toBeDefined();
    expect(result).toBe('pong'); // Check if the ping was successful
    console.log('E2E Test: __verySimpleAPI.ping() returned pong successfully!');

  });
});

