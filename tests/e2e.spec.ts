import type {Page, Browser} from 'playwright';
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
  let page: Page;

  beforeAll(async () => {
    const browserType = (environnement || 'chromium') as 'firefox' | 'chromium' | 'webkit';
    if (['firefox', 'chromium', 'webkit'].includes(browserType)) {
      ({page, browser} = await onBrowser({
        typeNavigateur: browserType,
      }));
    } else {
      throw new Error(`Unsupported test environment: ${environnement}. Must be 'firefox', 'chromium', or 'webkit'.`);
    }
  });

  afterAll(async () => {
    if (browser) {
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
      await page.waitForFunction(() => (window as any).peerbitAPI !== undefined, {timeout: 10000});
    } catch (e) {
      throw new Error('window.peerbitAPI did not become available within 10 seconds.');
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
});

