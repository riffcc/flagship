import { test } from 'vitest';
import { chromium } from 'playwright';

test('Capture Site.open timing', async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  
  const logs: Array<{time: number, text: string}> = [];
  const testStart = Date.now();
  
  // Capture ALL console logs
  page.on('console', msg => {
    logs.push({
      time: Date.now() - testStart,
      text: msg.text(),
    });
  });
  
  try {
    await page.goto('http://localhost:5175');
    
    // Wait for content
    await page.waitForFunction(() => {
      const app = document.querySelector('#app');
      const text = app?.textContent || '';
      return text.includes('Featured') || text.includes('No featured content') || text.includes('Movies');
    }, { timeout: 5000 });
    
    const totalTime = Date.now() - testStart;
    
    console.log(`\n⏱️  TOTAL LOAD TIME: ${totalTime}ms\n`);
    
    // Print all logs with Site.open timing
    console.log('📋 Full Console Timeline:');
    logs.forEach(log => {
      console.log(`[${log.time}ms] ${log.text}`);
    });
    
  } finally {
    await browser.close();
  }
});