import { test } from 'vitest';
import { chromium } from 'playwright';

test('Detailed Performance Breakdown', async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  
  const timings: Record<string, number> = {};
  const startTime = Date.now();
  
  // Track performance marks
  await page.evaluateOnNewDocument(() => {
    window.performanceMarks = {};
    
    // Override performance.mark to capture all marks
    const originalMark = performance.mark.bind(performance);
    performance.mark = (name: string) => {
      window.performanceMarks[name] = performance.now();
      return originalMark(name);
    };
  });
  
  // Track console logs with timestamps
  page.on('console', msg => {
    const elapsed = Date.now() - startTime;
    if (msg.text().includes('Connecting to bootstrappers') || 
        msg.text().includes('Connected to') ||
        msg.text().includes('Debug')) {
      console.log(`[${elapsed}ms] ${msg.text()}`);
    }
  });
  
  try {
    // Start navigation
    const navStart = Date.now();
    await page.goto('http://localhost:5175');
    timings['navigation'] = Date.now() - navStart;
    
    // Wait for Vue app mount
    const appMountStart = Date.now();
    await page.waitForSelector('#app', { state: 'visible' });
    timings['app_mount'] = Date.now() - appMountStart;
    
    // Check for loading spinner
    const spinnerCheck = Date.now();
    const hasSpinner = await page.$('.v-progress-circular') !== null;
    timings['spinner_check'] = Date.now() - spinnerCheck;
    
    if (hasSpinner) {
      console.log('Loading spinner detected, waiting for it to disappear...');
      const spinnerWaitStart = Date.now();
      await page.waitForSelector('.v-progress-circular', { state: 'hidden', timeout: 10000 });
      timings['spinner_wait'] = Date.now() - spinnerWaitStart;
    }
    
    // Wait for content
    const contentWaitStart = Date.now();
    await page.waitForFunction(() => {
      const app = document.querySelector('#app');
      const text = app?.textContent || '';
      return text.includes('Featured') || text.includes('No featured content');
    });
    timings['content_wait'] = Date.now() - contentWaitStart;
    
    timings['total'] = Date.now() - startTime;
    
    // Get performance entries from the page
    const perfData = await page.evaluate(() => ({
      marks: window.performanceMarks || {},
      resources: performance.getEntriesByType('resource').map(r => ({
        name: r.name.split('/').pop(),
        duration: Math.round(r.duration),
      })).filter(r => r.duration > 100), // Only show resources taking >100ms
    }));
    
    console.log('\n⏱️  PERFORMANCE BREAKDOWN:');
    console.log('Navigation:', timings.navigation + 'ms');
    console.log('App Mount:', timings.app_mount + 'ms');
    console.log('Spinner Check:', timings.spinner_check + 'ms');
    if (timings.spinner_wait) {
      console.log('Waiting for spinner:', timings.spinner_wait + 'ms');
    }
    console.log('Content Wait:', timings.content_wait + 'ms');
    console.log('TOTAL:', timings.total + 'ms');
    
    if (perfData.resources.length > 0) {
      console.log('\n📦 Slow Resources (>100ms):');
      perfData.resources.forEach(r => console.log(`  ${r.name}: ${r.duration}ms`));
    }
    
    if (Object.keys(perfData.marks).length > 0) {
      console.log('\n🏁 Performance Marks:');
      Object.entries(perfData.marks).forEach(([name, time]) => 
        console.log(`  ${name}: ${Math.round(time as number)}ms`));
    }
    
  } finally {
    await browser.close();
  }
});