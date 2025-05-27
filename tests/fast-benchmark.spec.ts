import { test, expect } from 'vitest';
import { chromium } from 'playwright';

const BENCHMARK_RUNS = 5;
const DEV_SERVER_URL = 'http://localhost:5175';

test('Fast Loading Benchmark - Actual Performance', async () => {
  console.log('🚀 Starting Fast Loading Benchmark');
  console.log(`Running ${BENCHMARK_RUNS} iterations`);

  const results: number[] = [];

  for (let i = 1; i <= BENCHMARK_RUNS; i++) {
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    
    const startTime = Date.now();
    
    try {
      // Navigate and wait for content to appear
      await page.goto(DEV_SERVER_URL);
      
      // Wait for either content or "no content" message - whichever comes first
      await page.waitForFunction(() => {
        const app = document.querySelector('#app');
        if (!app) return false;
        
        const text = app.textContent || '';
        // Content is ready when we see actual content OR the "no content" message
        return (
          text.includes('Featured') || 
          text.includes('No featured content') ||
          text.includes('Movies') ||
          text.includes('Music') ||
          document.querySelector('.content-card') !== null ||
          document.querySelector('.v-card') !== null
        );
      }, { timeout: 30000 });
      
      const loadTime = Date.now() - startTime;
      results.push(loadTime);
      console.log(`Run ${i}: ${loadTime}ms`);
      
    } catch (error) {
      console.error(`Run ${i} failed:`, error.message);
      results.push(30000); // Timeout value
    } finally {
      await browser.close();
    }
  }

  // Calculate stats
  const avg = results.reduce((a, b) => a + b, 0) / results.length;
  const min = Math.min(...results);
  const max = Math.max(...results);
  
  console.log('\n📊 RESULTS:');
  console.log(`Average: ${avg.toFixed(0)}ms`);
  console.log(`Min: ${min}ms`);
  console.log(`Max: ${max}ms`);
  
  // Check if we achieved sub-2s loading
  const sub2s = results.filter(t => t < 2000).length;
  console.log(`\nSub-2s loads: ${sub2s}/${results.length} (${(sub2s/results.length*100).toFixed(0)}%)`);
  
  expect(avg).toBeLessThan(3000); // Expect average under 3s
  expect(min).toBeLessThan(2000); // Expect at least one sub-2s load
});