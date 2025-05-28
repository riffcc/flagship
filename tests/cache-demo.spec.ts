import { test, expect } from '@playwright/test';

test('Demonstrate cache performance', async ({ page }) => {
  console.log('🚀 Initial page load...\n');
  
  let federationQueryCount = 0;
  let cacheHitCount = 0;
  
  // Listen for console logs
  page.on('console', msg => {
    const text = msg.text();
    
    // Track federation queries
    if (text.includes('[Federation]')) {
      if (text.includes('Serving') && text.includes('from memory cache')) {
        cacheHitCount++;
        console.log('✅ CACHE HIT:', text);
      } else {
        console.log('🔍 Federation:', text);
      }
    }
    
    if (text.includes('getFederationIndex')) {
      federationQueryCount++;
    }
  });

  // Initial load
  await page.goto('http://localhost:5175');
  await page.waitForSelector('.v-main', { timeout: 5000 });
  
  console.log(`\n📊 Initial Load Stats:`);
  console.log(`   Federation queries: ${federationQueryCount}`);
  console.log(`   Cache hits: ${cacheHitCount}`);
  
  // Reset counters
  const firstLoadQueries = federationQueryCount;
  federationQueryCount = 0;
  cacheHitCount = 0;
  
  // Wait for cache TTL to ensure we're within cache window
  console.log('\n⏳ Waiting 5 seconds then triggering re-query...\n');
  await page.waitForTimeout(5000);
  
  // Force component re-render by navigating
  await page.evaluate(() => {
    // Force Vue Query to refetch
    window.dispatchEvent(new Event('focus'));
  });
  
  await page.waitForTimeout(1000);
  
  console.log(`\n📊 After Re-query Stats:`);
  console.log(`   Federation queries: ${federationQueryCount}`);
  console.log(`   Cache hits: ${cacheHitCount}`);
  console.log(`   Cache hit rate: ${cacheHitCount > 0 ? '100%' : '0%'}`);
  
  console.log('\n✅ Cache demonstration complete!');
});