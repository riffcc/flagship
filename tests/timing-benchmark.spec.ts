import { test } from 'vitest';
import { chromium } from 'playwright';

test('Timing Analysis - Where is the 3 seconds?', async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  
  const timings: Record<string, number> = {};
  let lastMilestone = Date.now();
  
  const mark = (name: string) => {
    const now = Date.now();
    timings[name] = now - lastMilestone;
    lastMilestone = now;
  };
  
  // Track important console logs
  const logs: Array<{time: number, text: string}> = [];
  const testStart = Date.now();
  
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('bootstrap') || text.includes('Connected') || 
        text.includes('Debug') || text.includes('stage')) {
      logs.push({
        time: Date.now() - testStart,
        text: text.substring(0, 100),
      });
    }
  });
  
  try {
    // Initial navigation
    await page.goto('http://localhost:5175');
    mark('1_initial_load');
    
    // Wait for Vue app
    await page.waitForSelector('#app');
    mark('2_app_visible');
    
    // Check what's in the app initially
    let appContent = await page.textContent('#app');
    console.log('Initial app content:', appContent?.substring(0, 50) + '...');
    
    // Wait for loading to complete (either content or "no content" message)
    const waitStart = Date.now();
    let iterations = 0;
    let foundContent = false;
    
    while (Date.now() - waitStart < 5000 && !foundContent) {
      iterations++;
      appContent = await page.textContent('#app');
      
      if (appContent?.includes('Featured') || 
          appContent?.includes('No featured content') ||
          appContent?.includes('Movies')) {
        foundContent = true;
        break;
      }
      
      // Check for spinner
      const hasSpinner = await page.$('.v-progress-circular') !== null;
      if (iterations === 1 && hasSpinner) {
        console.log('Loading spinner present');
      }
      
      await page.waitForTimeout(100);
    }
    
    mark('3_content_appears');
    
    console.log(`Content found after ${iterations} checks (${iterations * 100}ms)`);
    console.log('Final content preview:', appContent?.substring(0, 100) + '...');
    
    // Print timing breakdown
    console.log('\n⏱️  TIMING BREAKDOWN:');
    let total = 0;
    Object.entries(timings).forEach(([name, time]) => {
      console.log(`${name}: ${time}ms`);
      total += time;
    });
    console.log(`TOTAL: ${total}ms`);
    
    // Print console logs timeline
    if (logs.length > 0) {
      console.log('\n📋 Console Timeline:');
      logs.forEach(log => {
        console.log(`[${log.time}ms] ${log.text}`);
      });
    }
    
    // Check network activity
    const metrics = await page.evaluate(() => {
      const perf = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      return {
        domContentLoaded: Math.round(perf.domContentLoadedEventEnd - perf.fetchStart),
        loadComplete: Math.round(perf.loadEventEnd - perf.fetchStart),
      };
    });
    
    console.log('\n🌐 Page Load Metrics:');
    console.log(`DOM Content Loaded: ${metrics.domContentLoaded}ms`);
    console.log(`Load Event Complete: ${metrics.loadComplete}ms`);
    
  } finally {
    await browser.close();
  }
});