import { test, expect } from '@playwright/test';

test('inspect releases on the site page', async ({ page }) => {
  // Enable verbose console logging
  page.on('console', msg => {
    if (msg.type() === 'log') {
      console.log('CONSOLE:', msg.text());
    }
  });

  // Monitor network requests
  page.on('request', request => {
    if (request.url().includes('release') || request.url().includes('Release')) {
      console.log('REQUEST:', request.method(), request.url());
    }
  });

  page.on('response', response => {
    if (response.url().includes('release') || response.url().includes('Release')) {
      console.log('RESPONSE:', response.status(), response.url());
    }
  });

  // Navigate to the app
  console.log('Navigating to app...');
  await page.goto('http://localhost:5175', { 
    waitUntil: 'networkidle',
    timeout: 30000, 
  });

  // Wait for initial load
  await page.waitForTimeout(3000);

  // Take a screenshot of the initial page
  await page.screenshot({ path: 'initial-page.png', fullPage: true });
  console.log('Screenshot saved: initial-page.png');

  // Look for any site links or cards
  console.log('\n=== LOOKING FOR SITE ELEMENTS ===');
  const siteLinks = await page.locator('a').all();
  for (const link of siteLinks) {
    const href = await link.getAttribute('href');
    const text = await link.textContent();
    if (href && href.includes('site')) {
      console.log(`Found site link: ${href} - Text: ${text}`);
    }
  }

  // Look for site cards or any clickable elements
  const clickableElements = await page.locator('[role="button"], .v-card, .site-card, [data-test*="site"]').all();
  console.log(`Found ${clickableElements.length} potentially clickable elements`);

  // Try to find the site by text
  const siteTexts = ['wings test site', 'test site', 'wings', 'site'];
  let clicked = false;
  
  for (const text of siteTexts) {
    try {
      const element = page.locator(`text="${text}"`).first();
      if (await element.isVisible({ timeout: 1000 })) {
        console.log(`Clicking on element with text: ${text}`);
        await element.click();
        clicked = true;
        break;
      }
    } catch (e) {
      // Continue trying other texts
    }
  }

  if (!clicked) {
    // Try clicking the first card-like element
    if (clickableElements.length > 0) {
      console.log('Clicking first clickable element...');
      await clickableElements[0].click();
    }
  }

  // Wait for navigation
  await page.waitForTimeout(5000);

  // Take screenshot after navigation
  await page.screenshot({ path: 'after-navigation.png', fullPage: true });
  console.log('Screenshot saved: after-navigation.png');

  // Look for release elements
  console.log('\n=== LOOKING FOR RELEASE ELEMENTS ===');
  const releaseSelectors = [
    '.release',
    '[data-release]',
    '.v-card',
    '.release-card',
    '[class*="release"]',
    'article',
    '.content-item',
  ];

  for (const selector of releaseSelectors) {
    const elements = await page.locator(selector).all();
    if (elements.length > 0) {
      console.log(`Found ${elements.length} elements matching ${selector}`);
      
      // Inspect first few elements
      for (let i = 0; i < Math.min(3, elements.length); i++) {
        const text = await elements[i].textContent();
        console.log(`  Element ${i + 1}: ${text?.substring(0, 100)}...`);
      }
    }
  }

  // Try to extract data from page context
  console.log('\n=== EXTRACTING PAGE DATA ===');
  const pageData = await page.evaluate(() => {
    const data: any = {
      url: window.location.href,
      title: document.title,
      localStorageKeys: Object.keys(localStorage),
      sessionStorageKeys: Object.keys(sessionStorage),
    };

    // Look for any global variables that might contain releases
    const globalVars = Object.keys(window);
    const releaseRelated = globalVars.filter(key => 
      key.toLowerCase().includes('release') || 
      key.toLowerCase().includes('site') ||
      key.toLowerCase().includes('lens'),
    );
    data.releaseRelatedGlobals = releaseRelated;

    // Check for Vue app
    if ((window as any).__VUE__) {
      data.hasVue = true;
    }

    return data;
  });

  console.log('Page data:', JSON.stringify(pageData, null, 2));

  // Try to interact with the page to load releases
  console.log('\n=== TRYING TO TRIGGER RELEASE LOADING ===');
  
  // Scroll to trigger lazy loading
  await page.evaluate(() => {
    window.scrollTo(0, document.body.scrollHeight);
  });
  await page.waitForTimeout(2000);

  // Look for any "Browse" or "Releases" buttons
  const browseButtons = await page.locator('button:has-text("Browse"), button:has-text("Releases"), a:has-text("Browse"), a:has-text("Releases")').all();
  if (browseButtons.length > 0) {
    console.log(`Found ${browseButtons.length} browse/releases buttons`);
    await browseButtons[0].click();
    await page.waitForTimeout(3000);
  }

  // Final screenshot
  await page.screenshot({ path: 'final-page.png', fullPage: true });
  console.log('Screenshot saved: final-page.png');

  // Wait to capture any delayed console logs
  await page.waitForTimeout(5000);
  
  console.log('\nTest complete.');
});