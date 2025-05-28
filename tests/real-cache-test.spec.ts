import { test, expect } from '@playwright/test';

test('Real cache performance within session', async ({ page }) => {
  console.log('🚀 Loading page and testing cache...');
  
  // Listen for cache messages
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('cache') || text.includes('Cache') || text.includes('memory')) {
      console.log('💾', text);
    }
  });

  // Navigate to the page
  await page.goto('http://localhost:5175');
  await page.waitForTimeout(2000);
  
  // Now navigate to a different page and back to trigger cache
  console.log('\n🔄 Navigating away and back to test cache...');
  
  // Go to admin page
  await page.click('text=Admin');
  await page.waitForTimeout(500);
  
  // Go back to home - this should use cache!
  await page.click('text=Home');
  await page.waitForTimeout(1000);
  
  // Navigate away and back again
  console.log('\n🔄 Second navigation to test established cache...');
  await page.click('text=Admin');
  await page.waitForTimeout(500);
  await page.click('text=Home');
  await page.waitForTimeout(1000);
  
  // Force a re-query to test cache
  console.log('\n🔄 Testing direct query cache...');
  await page.evaluate(() => {
    // This will trigger the federation queries again
    window.location.hash = '#refresh';
    window.location.hash = '';
  });
  
  await page.waitForTimeout(1000);
  
  console.log('\n✅ Cache test complete! Check console for cache hits.');
});