import { test, expect } from '@playwright/test';

test('Simple cache performance test', async ({ page }) => {
  console.log('🚀 Testing with cache...');
  
  // Listen for all console logs
  const logs: string[] = [];
  page.on('console', msg => {
    const text = msg.text();
    logs.push(text);
    
    // Log cache-related messages immediately
    if (text.includes('cache') || text.includes('Cache') || text.includes('memory')) {
      console.log('💾', text);
    }
  });

  // Navigate
  await page.goto('http://localhost:5175');
  
  // Wait a bit for everything to load
  await page.waitForTimeout(3000);
  
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
  
  // Look for federation queries
  const federationLogs = logs.filter(log => log.includes('Federation'));
  if (federationLogs.length > 0) {
    console.log('\n🔍 Federation Activity:');
    federationLogs.forEach(log => console.log(log));
  }
  
  // Take screenshot
  await page.screenshot({ path: 'cache-test.png' });
  
  console.log('\n✅ Test complete!');
});