import { test, expect } from '@playwright/test';

test('Cache performance measurement', async ({ page }) => {
  console.log('🚀 First load - populating cache...');
  
  // First load - populate cache
  const firstLoadLogs: string[] = [];
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('[') || text.includes('ms')) {
      firstLoadLogs.push(text);
    }
  });

  await page.goto('http://localhost:5175');
  await page.waitForSelector('text=/Featured Test Movie/', { timeout: 10000 });
  
  // Extract first load time
  const firstLoadTotal = firstLoadLogs.find(log => log.includes('[App] Total initialization'));
  console.log('First load:', firstLoadTotal || 'Not found');
  
  // Wait a bit to ensure cache is populated
  await page.waitForTimeout(1000);
  
  // SECOND LOAD - from cache
  console.log('\n🚀 Second load - using cache...');
  
  const secondLoadLogs: string[] = [];
  page.removeAllListeners('console');
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('[') || text.includes('ms') || text.includes('cache')) {
      secondLoadLogs.push(text);
    }
  });
  
  // Reload the page
  await page.reload();
  await page.waitForSelector('text=/Featured Test Movie/', { timeout: 10000 });
  
  console.log('\n📊 Cache Performance Metrics:');
  console.log('==============================');
  
  // Look for cache hits
  const cacheHits = secondLoadLogs.filter(log => 
    log.includes('cache') || 
    log.includes('Cache') ||
    log.includes('memory')
  );
  
  if (cacheHits.length > 0) {
    console.log('\n💾 Cache Hits:');
    cacheHits.forEach(hit => console.log(hit));
  }
  
  // Extract timing information
  const timings = secondLoadLogs.filter(log => 
    log.includes('ms') && (
      log.includes('[Preload]') ||
      log.includes('[App]') ||
      log.includes('[Main]') ||
      log.includes('[Site]') ||
      log.includes('[LensSDK]') ||
      log.includes('[Federation]')
    )
  );
  
  console.log('\n⏱️ Second Load Timings:');
  timings.forEach(timing => console.log(timing));
  
  // Extract total time
  const secondLoadTotal = secondLoadLogs.find(log => log.includes('[App] Total initialization'));
  
  console.log('\n🎯 Comparison:');
  console.log('==============');
  console.log('First load: ', firstLoadTotal || 'Not measured');
  console.log('Second load:', secondLoadTotal || 'Not measured');
  
  // Take screenshot
  await page.screenshot({ path: 'cache-performance.png' });
  
  console.log('\n✅ Cache test complete!');
});