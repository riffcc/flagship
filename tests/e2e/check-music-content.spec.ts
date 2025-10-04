import { test } from '@playwright/test';

test('Check music category page content', async ({ page }) => {
  console.log('Navigating to music category...');
  await page.goto('http://127.0.0.1:5175/#/featured/music');
  
  // Wait for Vue to load
  await page.waitForTimeout(3000);
  
  // Take screenshot
  await page.screenshot({ path: '/tmp/music-page.png', fullPage: true });
  console.log('Screenshot saved to /tmp/music-page.png');
  
  // Get page text
  const bodyText = await page.textContent('body');
  console.log('Page contains "Music":', bodyText?.includes('Music'));
  console.log('Page contains "No featured":', bodyText?.includes('No featured'));
  
  // Count content cards
  const cards = await page.locator('.content-card').count();
  console.log('Content cards found:', cards);
  
  // Get all text content to see what's actually on the page
  const allText = await page.evaluate(() => {
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    const texts: string[] = [];
    let node;
    while (node = walker.nextNode()) {
      const text = node.textContent?.trim();
      if (text && text.length > 0) texts.push(text);
    }
    return texts.filter(t => t.length > 2).slice(0, 50);
  });
  console.log('Visible text on page:', JSON.stringify(allText.slice(0, 20)));
});

test.setTimeout(30000);
