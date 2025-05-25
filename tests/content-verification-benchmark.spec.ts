import { test } from 'vitest';
import { chromium } from 'playwright';

test('Verify Actual Content Loading', async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  
  const testStart = Date.now();
  const milestones: Record<string, number> = {};
  
  // Track console logs
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('HomePage Debug')) {
      console.log(`[${Date.now() - testStart}ms] ${text.substring(0, 150)}`);
    }
  });
  
  try {
    await page.goto('http://localhost:5175');
    milestones.navigation = Date.now() - testStart;
    
    // Wait for app
    await page.waitForSelector('#app', { state: 'visible' });
    milestones.appVisible = Date.now() - testStart;
    
    // Wait for loading to finish (spinner gone)
    try {
      await page.waitForSelector('.v-progress-circular', { state: 'hidden', timeout: 10000 });
      milestones.loadingComplete = Date.now() - testStart;
    } catch {
      console.log('No loading spinner detected or it never disappeared');
    }
    
    // Check what content is actually displayed
    const content = await page.evaluate(() => {
      const app = document.querySelector('#app');
      const text = app?.textContent || '';
      
      return {
        fullText: text.substring(0, 500),
        hasNavigation: text.includes('Home') && text.includes('Music'),
        hasNoContentMessage: text.includes('No featured content') || text.includes('No content'),
        hasFeaturedSlider: !!document.querySelector('.featured-slider'),
        hasContentCards: document.querySelectorAll('.content-card, .v-card').length,
        hasImages: document.querySelectorAll('img[src*="ipfs"], img[src*="mock"]').length,
        hasContentSections: document.querySelectorAll('.content-section').length,
      };
    });
    
    milestones.contentChecked = Date.now() - testStart;
    
    console.log('\n📊 TIMING MILESTONES:');
    Object.entries(milestones).forEach(([name, time]) => {
      console.log(`${name}: ${time}ms`);
    });
    
    console.log('\n🔍 CONTENT VERIFICATION:');
    console.log('Has Navigation:', content.hasNavigation);
    console.log('Has "No Content" Message:', content.hasNoContentMessage);
    console.log('Has Featured Slider:', content.hasFeaturedSlider);
    console.log('Number of Content Cards:', content.hasContentCards);
    console.log('Number of Images:', content.hasImages);
    console.log('Number of Content Sections:', content.hasContentSections);
    
    console.log('\n📄 PAGE CONTENT PREVIEW:');
    console.log(content.fullText);
    
    // Take a screenshot
    await page.screenshot({ path: '/tmp/content-verification.png', fullPage: true });
    console.log('\n📸 Screenshot saved to /tmp/content-verification.png');
    
  } finally {
    await browser.close();
  }
});