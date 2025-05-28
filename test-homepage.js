import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();
  
  // Enable console logging
  page.on('console', msg => {
    const text = msg.text();
    if (text.includes('federation') || text.includes('Featured') || text.includes('error') || text.includes('Error')) {
      console.log('PAGE LOG:', text);
    }
  });
  
  // Navigate to the homepage
  await page.goto('http://localhost:5175/');
  
  // Wait for the page to load
  await page.waitForTimeout(15000);
  
  // Check what's actually visible on the page
  const loadingVisible = await page.isVisible('.v-progress-circular').catch(() => false);
  const noContentVisible = await page.isVisible('text=No content here').catch(() => false);
  const noFeaturedVisible = await page.isVisible('text=No featured content yet').catch(() => false);
  
  console.log('UI State:', {
    loading: loadingVisible,
    noContent: noContentVisible,
    noFeatured: noFeaturedVisible,
  });
  
  // Try to execute federation queries directly in the browser
  const federationResults = await page.evaluate(async () => {
    try {
      // Access the lens service from Vue app
      const app = window.__app;
      if (!app) return { error: 'No Vue app found' };
      
      const lensService = app.config.globalProperties.$lensService;
      if (!lensService) return { error: 'No lens service found' };
      
      // Try calling federation methods directly
      const featured = await lensService.getFederationIndexFeatured(10);
      const recent = await lensService.getFederationIndexRecent(10);
      
      return {
        featuredCount: featured?.length || 0,
        recentCount: recent?.length || 0,
        featuredSample: featured?.[0] || null,
        recentSample: recent?.[0] || null,
      };
    } catch (error) {
      return { error: error.message };
    }
  });
  
  console.log('Direct federation query results:', federationResults);
  
  // Wait a bit more to see console logs
  await page.waitForTimeout(5000);
  
  await browser.close();
})();