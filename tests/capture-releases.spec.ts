import { test } from '@playwright/test';

test('capture release console output', async ({ page }) => {
  // Enable verbose console output
  page.on('console', msg => {
    if (msg.type() === 'log') {
      console.log(msg.text());
    }
  });

  page.on('pageerror', err => {
    console.error('Page error:', err);
  });

  // Go to the app
  await page.goto('http://localhost:5175', { waitUntil: 'domcontentloaded' });
  
  // Wait for initialization
  console.log('\n=== Waiting for app initialization ===');
  await page.waitForTimeout(15000);
  
  // Navigate to releases page to trigger any additional loading
  console.log('\n=== Navigating to releases ===');
  await page.click('a[href="/releases"]');
  await page.waitForTimeout(5000);
  
  console.log('\n=== Test complete ===');
});