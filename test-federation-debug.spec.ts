import { test, expect } from '@playwright/test';

test('Debug federation index issues', async ({ page }) => {
  // Listen to all console messages
  page.on('console', msg => {
    const type = msg.type();
    const text = msg.text();
    
    // Color code by type
    if (type === 'error') {
      console.error('🔴 ERROR:', text);
    } else if (type === 'warning') {
      console.warn('🟡 WARN:', text);
    } else if (text.includes('[Site]') || text.includes('contentCID')) {
      console.log('🟢 SITE:', text);
    } else if (text.includes('Federation')) {
      console.log('🔵 FED:', text);
    } else {
      console.log('⚪', text);
    }
  });

  // Also capture detailed errors
  page.on('pageerror', error => {
    console.error('🔴 PAGE ERROR:', error.message);
  });

  console.log('🚀 Navigating to localhost:5175...');
  await page.goto('http://localhost:5175', { 
    waitUntil: 'networkidle',
    timeout: 30000, 
  });

  console.log('⏳ Waiting for app to initialize...');
  await page.waitForTimeout(5000);

  console.log('📸 Taking screenshot...');
  await page.screenshot({ path: 'federation-debug.png' });

  // Try to navigate to federation stats
  console.log('🔍 Navigating to federation stats...');
  await page.goto('http://localhost:5175/#/federation-stats');
  await page.waitForTimeout(2000);

  // Check if we can see the stats button
  const statsButton = page.locator('button:has-text("Refresh Stats")');
  if (await statsButton.isVisible()) {
    console.log('📊 Clicking refresh stats...');
    await statsButton.click();
    await page.waitForTimeout(2000);
  }

  console.log('✅ Test complete. Check console output above for issues.');
});