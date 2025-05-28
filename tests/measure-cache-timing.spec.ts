import { test, expect } from '@playwright/test';

test('Measure actual cache timing', async ({ page }) => {
  console.log('🚀 Measuring cache performance...\n');
  
  // Capture timing from console
  const timings: number[] = [];
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('query:') && text.includes('ms')) {
      const match = text.match(/(\d+\.?\d*)ms/);
      if (match) {
        timings.push(parseFloat(match[1]));
        console.log(text);
      }
    }
  });

  await page.goto('http://localhost:5175');
  
  // Wait for initial load
  await page.waitForSelector('.v-main');
  await page.waitForTimeout(2000);
  
  console.log('\n🔄 Testing cache with direct measurements...\n');
  
  // Execute timing test in browser
  await page.evaluate(async () => {
    console.time('First query: direct service call');
    await window.$lensService?.getFederationIndexFeatured(50);
    console.timeEnd('First query: direct service call');
    
    // Immediate second call - should be faster if cached
    console.time('Second query: potential cache hit');
    await window.$lensService?.getFederationIndexFeatured(50);
    console.timeEnd('Second query: potential cache hit');
    
    // Wait a bit then try again
    await new Promise(resolve => setTimeout(resolve, 100));
    
    console.time('Third query: after 100ms');
    await window.$lensService?.getFederationIndexFeatured(50);
    console.timeEnd('Third query: after 100ms');
  });
  
  await page.waitForTimeout(1000);
  
  if (timings.length >= 2) {
    console.log('\n📊 Performance Analysis:');
    console.log(`First query: ${timings[0]}ms`);
    console.log(`Second query: ${timings[1]}ms`);
    if (timings[1] < timings[0] * 0.5) {
      console.log(`✅ Cache is working! Second query was ${((1 - timings[1]/timings[0]) * 100).toFixed(1)}% faster`);
    }
  }
  
  console.log('\n✅ Cache timing test complete!');
});