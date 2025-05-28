import { test, expect } from '@playwright/test';

test('wait for site to load and capture releases', async ({ page }) => {
  const logs: any[] = [];
  
  // Comprehensive console capture
  page.on('console', async msg => {
    const text = msg.text();
    logs.push({ type: msg.type(), text });
    
    // Try to capture objects
    if (msg.type() === 'log' && msg.args().length > 0) {
      for (const arg of msg.args()) {
        try {
          const value = await arg.jsonValue();
          if (value && typeof value === 'object') {
            logs.push({ type: 'object', value });
          }
        } catch (e) {}
      }
    }
  });

  console.log('Navigating to app...');
  await page.goto('http://localhost:5175', { 
    waitUntil: 'domcontentloaded',
    timeout: 30000, 
  });

  // Wait for the Vue app to mount
  await page.waitForFunction(() => {
    return !!(window as any).__VUE__;
  }, { timeout: 10000 });

  console.log('Vue app detected, waiting for site to load...');

  // Wait for any loading indicators to disappear
  try {
    await page.waitForSelector('.v-progress-circular', { state: 'hidden', timeout: 5000 });
  } catch (e) {
    // No loader found or already hidden
  }

  // Wait a bit more for data to load
  await page.waitForTimeout(5000);

  // Check what's on the page
  const pageContent = await page.evaluate(() => {
    return {
      bodyText: document.body.innerText.substring(0, 500),
      hasCards: document.querySelectorAll('.v-card').length,
      hasLinks: document.querySelectorAll('a').length,
      localStorage: Object.fromEntries(
        Object.keys(localStorage).map(k => [k, localStorage.getItem(k)]),
      ),
    };
  });

  console.log('Page content:', JSON.stringify(pageContent, null, 2));

  // Look for site data in localStorage
  const siteData = pageContent.localStorage.site ? JSON.parse(pageContent.localStorage.site) : null;
  if (siteData) {
    console.log('\nFound site in localStorage:', siteData.id);
    
    // Navigate directly to the site
    await page.goto(`http://localhost:5175/#/sites/${siteData.id}`, {
      waitUntil: 'networkidle',
    });
    
    console.log('Navigated to site page, waiting for releases...');
    await page.waitForTimeout(10000);
  }

  // Filter and display relevant logs
  console.log('\n=== RELEVANT CONSOLE LOGS ===');
  const releaseKeywords = ['release', 'Release', 'site', 'Site', 'content', 'CID', 'federated'];
  
  logs.forEach((log, i) => {
    if (log.type === 'object' && log.value) {
      // Check if object might be a release
      if (log.value.contentCID || log.value.federatedFrom || 
          (log.value.name && log.value.id && log.value.created)) {
        console.log(`\n[${i}] OBJECT - Possible Release:`);
        console.log(JSON.stringify(log.value, null, 2));
      }
    } else if (log.text && releaseKeywords.some(kw => log.text.includes(kw))) {
      console.log(`[${i}] ${log.type.toUpperCase()}: ${log.text}`);
    }
  });

  // Try to extract releases from the current page
  const releases = await page.evaluate(() => {
    // Look for release data in various places
    const allReleases: any[] = [];
    
    // Check all v-cards for release data
    document.querySelectorAll('.v-card').forEach(card => {
      const text = card.textContent || '';
      if (text && !text.includes('No releases')) {
        allReleases.push({
          cardText: text.substring(0, 200),
          hasImage: !!card.querySelector('img'),
          links: Array.from(card.querySelectorAll('a')).map(a => a.href),
        });
      }
    });
    
    return allReleases;
  });

  if (releases.length > 0) {
    console.log('\n=== FOUND RELEASE CARDS ===');
    releases.forEach((r, i) => {
      console.log(`\nRelease Card ${i + 1}:`);
      console.log(JSON.stringify(r, null, 2));
    });
  }

  // Take final screenshot
  await page.screenshot({ path: 'site-with-releases.png', fullPage: true });
  console.log('\nScreenshot saved: site-with-releases.png');
  
  console.log('\nTest complete.');
});