import type {Page, Browser} from 'playwright';

import {afterAll, beforeAll, describe, expect, test} from 'vitest';

import {onBrowser} from './utils';

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
});

