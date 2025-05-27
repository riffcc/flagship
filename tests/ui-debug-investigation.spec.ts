import type {Browser, Page, ElectronApplication} from 'playwright';
import {afterAll, beforeAll, describe, expect, test} from 'vitest';
import {onBrowser} from './utils';

describe('UI Debug Investigation - Detailed Logging', function () {
  let browser: Browser | undefined = undefined;
  const electronApp: ElectronApplication | undefined = undefined;
  let page: Page;

  beforeAll(async () => {
    const result = await onBrowser({ typeNavigateur: 'chromium' });
    page = result.page;
    browser = result.browser;
  });

  afterAll(async () => {
    if (electronApp) {
      await electronApp.close();
    } else if (browser) {
      await browser.close();
    }
  });

  test('Debug Vue component logging', async () => {
    console.log('🔍 Starting UI Debug Investigation');
    
    // Capture console logs
    const consoleLogs: string[] = [];
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('[HomePage Debug]')) {
        consoleLogs.push(`${msg.type()}: ${text}`);
        console.log(`🐛 ${msg.type()}: ${text}`);
      }
    });

  // Start timing
  const startTime = Date.now();
  
  console.log('🌐 Navigating to homepage...');
  await page.goto('http://localhost:5175', { 
    waitUntil: 'domcontentloaded', 
    timeout: 30000, 
  });

  // Wait for initial render
  await page.waitForSelector('#app', { timeout: 5000 });
  console.log('📱 App element found');

  // Wait a bit for Vue to initialize and logs to start
  await page.waitForTimeout(2000);

  // Check loading states
  const isLoadingVisible = await page.isVisible('[data-testid="loading"]');
  console.log(`🔄 Loading spinner visible: ${isLoadingVisible}`);

  // Wait for content or timeout
  console.log('⏳ Waiting for content to load...');
  
  let finalState = 'unknown';
  try {
    // Wait for either content or no-content message
    await Promise.race([
      page.waitForSelector('[data-testid="featured-content"]', { timeout: 25000 }),
      page.waitForSelector('[data-testid="no-featured-content"]', { timeout: 25000 }),
    ]);
    
    const hasFeaturedContent = await page.isVisible('[data-testid="featured-content"]');
    const hasNoContentMessage = await page.isVisible('[data-testid="no-featured-content"]');
    const isStillLoading = await page.isVisible('[data-testid="loading"]');
    
    if (hasFeaturedContent) {
      finalState = 'featured-content-visible';
    } else if (hasNoContentMessage) {
      finalState = 'no-content-message-visible';
    } else if (isStillLoading) {
      finalState = 'stuck-in-loading';
    }
    
    console.log(`🎯 Final state: ${finalState}`);
    console.log(`🔄 Loading visible: ${isStillLoading}`);
    console.log(`✅ Featured content visible: ${hasFeaturedContent}`);
    console.log(`❌ No content message visible: ${hasNoContentMessage}`);
    
  } catch (error) {
    console.log(`⚠️ Timeout waiting for content: ${error}`);
    finalState = 'timeout';
  }

  const totalTime = Date.now() - startTime;
  console.log(`⏱️ Total time: ${totalTime}ms`);

  // Summary of debug logs
  console.log(`📊 Debug logs captured: ${consoleLogs.length}`);
  
  if (consoleLogs.length === 0) {
    console.log('❌ NO DEBUG LOGS CAPTURED - Vue component may not be logging');
  } else {
    console.log('✅ Debug logs found:');
    consoleLogs.forEach(log => console.log(`  ${log}`));
  }

    // Always pass to see the results
    expect(true).toBe(true);
  });
});