import { test, expect } from '@playwright/test';

test('True cache performance demonstration', async ({ page }) => {
  console.log('🚀 Testing cache performance...\n');
  
  const timings: { [key: string]: number[] } = {
    federationQueries: [],
    initialization: []
  };
  
  // Listen for performance metrics
  page.on('console', msg => {
    const text = msg.text();
    
    // Capture cache hits
    if (text.includes('Serving') && text.includes('from memory cache')) {
      console.log('✅ CACHE HIT!', text);
      timings.federationQueries.push(1); // 1ms for cache hit
    }
    
    // Capture federation query times
    if (text.includes('getFederationIndex') && text.includes('returned')) {
      const match = text.match(/(\d+) entries/);
      if (match) {
        console.log('🔍', text);
      }
    }
    
    // Capture initialization time
    if (text.includes('[App] Total initialization')) {
      const match = text.match(/(\d+\.?\d*)/);
      if (match) {
        timings.initialization.push(parseFloat(match[1]));
      }
    }
  });

  // FIRST LOAD - Cold start
  console.log('📊 FIRST LOAD (Cold Start):');
  console.log('===========================');
  const start1 = Date.now();
  await page.goto('http://localhost:5175');
  await page.waitForSelector('.v-main', { timeout: 10000 });
  const load1Time = Date.now() - start1;
  console.log(`Total page load time: ${load1Time}ms\n`);
  
  // Wait 3 seconds (within our 5s stale time)
  await page.waitForTimeout(3000);
  
  // SECOND LOAD - Should hit TanStack Query cache
  console.log('\n📊 SECOND LOAD (TanStack Query Cache):');
  console.log('=====================================');
  const start2 = Date.now();
  await page.goto('http://localhost:5175');
  await page.waitForSelector('.v-main', { timeout: 10000 });
  const load2Time = Date.now() - start2;
  console.log(`Total page load time: ${load2Time}ms`);
  console.log(`Speedup: ${((load1Time - load2Time) / load1Time * 100).toFixed(1)}% faster\n`);
  
  // Wait for stale time to expire
  console.log('⏳ Waiting for stale time to expire (7s)...');
  await page.waitForTimeout(7000);
  
  // THIRD LOAD - Should hit our memory cache
  console.log('\n📊 THIRD LOAD (Memory Cache):');
  console.log('============================');
  const start3 = Date.now();
  await page.goto('http://localhost:5175');
  await page.waitForSelector('.v-main', { timeout: 10000 });
  const load3Time = Date.now() - start3;
  console.log(`Total page load time: ${load3Time}ms`);
  
  // Summary
  console.log('\n🎯 PERFORMANCE SUMMARY:');
  console.log('======================');
  console.log(`1st Load (Cold):        ${load1Time}ms`);
  console.log(`2nd Load (Query Cache): ${load2Time}ms (${((load1Time - load2Time) / load1Time * 100).toFixed(1)}% faster)`);
  console.log(`3rd Load (Memory Cache): ${load3Time}ms (${((load1Time - load3Time) / load1Time * 100).toFixed(1)}% faster)`);
  
  await page.screenshot({ path: 'cache-performance-final.png' });
  console.log('\n✅ Cache test complete!');
});