import { test, expect } from '@playwright/test';

test('Performance measurement with optimizations', async ({ page }) => {
  // Listen for console logs
  const logs: string[] = [];
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('[') || text.includes('ms')) {
      logs.push(text);
    }
  });

  // Navigate and wait for initialization
  console.log('🚀 Navigating to localhost:5175...');
  await page.goto('http://localhost:5175');
  
  // Wait for the featured content to appear
  await page.waitForSelector('text=/Featured Test Movie/', { timeout: 10000 });
  
  console.log('\n📊 Performance Metrics:');
  console.log('========================');
  
  // Extract timing information
  const timings = logs.filter(log => 
    log.includes('ms') && (
      log.includes('[Preload]') ||
      log.includes('[App]') ||
      log.includes('[Main]') ||
      log.includes('[Site]') ||
      log.includes('[LensSDK]') ||
      log.includes('[Federation]')
    )
  );
  
  timings.forEach(timing => console.log(timing));
  
  // Check if federation data was cached
  const cacheHits = logs.filter(log => log.includes('cache'));
  if (cacheHits.length > 0) {
    console.log('\n💾 Cache Performance:');
    console.log('====================');
    cacheHits.forEach(hit => console.log(hit));
  }
  
  // Take screenshot
  await page.screenshot({ path: 'performance-test.png' });
  
  console.log('\n✅ Test complete!');
});