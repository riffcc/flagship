import { test } from 'vitest';
import { chromium } from 'playwright';

test('Fresh Load Performance', async () => {
  const browser = await chromium.launch({ headless: false }); // Not headless to ensure fresh load
  const context = await browser.newContext({
    // Disable cache
    bypassCSP: true,
    ignoreHTTPSErrors: true,
    extraHTTPHeaders: {
      'Cache-Control': 'no-cache',
    },
  });
  const page = await context.newPage();
  
  const logs: Array<{time: number, text: string}> = [];
  const testStart = Date.now();
  
  // Capture ALL console logs
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('Site.open') || text.includes('bootstrap') || text.includes('Connected')) {
      logs.push({
        time: Date.now() - testStart,
        text: text,
      });
    }
  });
  
  try {
    // Force fresh load
    await page.goto('http://localhost:5175', { waitUntil: 'domcontentloaded' });
    
    // Wait for content
    await page.waitForFunction(() => {
      const app = document.querySelector('#app');
      const text = app?.textContent || '';
      return text.includes('Featured') || text.includes('No featured content') || text.includes('Movies');
    }, { timeout: 10000 });
    
    const totalTime = Date.now() - testStart;
    
    console.log(`\n⏱️  TOTAL LOAD TIME: ${totalTime}ms`);
    
    if (totalTime < 2000) {
      console.log('🎉 SUB-2s LOAD ACHIEVED!');
    }
    
    // Print Site.open logs
    console.log('\n📋 Key Timeline Events:');
    logs.forEach(log => {
      console.log(`[${log.time}ms] ${log.text}`);
    });
    
  } finally {
    await browser.close();
  }
});