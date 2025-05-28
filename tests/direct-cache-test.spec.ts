import { test, expect } from '@playwright/test';

test('Direct cache test', async ({ page }) => {
  // Enable verbose console logging
  page.on('console', msg => {
    if (msg.type() === 'log') {
      console.log(msg.text());
    }
  });

  console.log('🚀 Loading page and monitoring cache...\n');
  
  await page.goto('http://localhost:5175');
  
  // Wait for initial load
  await page.waitForSelector('.v-main');
  await page.waitForTimeout(2000);
  
  console.log('\n🔄 Triggering manual refetch to test cache...\n');
  
  // Execute cache test in the browser context
  const cacheTest = await page.evaluate(async () => {
    // Get the lens service from window (Vue app provides it)
    const app = document.querySelector('#app').__vue_app__;
    const lensService = app.config.globalProperties.$lensService;
    
    console.log('📊 Testing federation cache directly...');
    
    // First query - should hit the service
    console.time('First query');
    const result1 = await lensService.getFederationIndexFeatured(50);
    console.timeEnd('First query');
    console.log(`First query returned ${result1.length} items`);
    
    // Second query immediately - should hit cache
    console.time('Second query (should be cached)');
    const result2 = await lensService.getFederationIndexFeatured(50);
    console.timeEnd('Second query (should be cached)');
    console.log(`Second query returned ${result2.length} items`);
    
    // Different limit - won't hit cache
    console.time('Third query (different params)');
    const result3 = await lensService.getFederationIndexFeatured(10);
    console.timeEnd('Third query (different params)');
    console.log(`Third query returned ${result3.length} items`);
    
    return {
      firstQueryItems: result1.length,
      secondQueryItems: result2.length,
      thirdQueryItems: result3.length
    };
  });
  
  console.log('\n📊 Cache Test Results:', cacheTest);
  console.log('\n✅ Direct cache test complete!');
});