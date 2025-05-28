import { test, expect } from '@playwright/test';

test('Federation index stress test - multiple refreshes', async ({ page }) => {
  let hasErrors = false;
  let successfulLoads = 0;
  let corruptionDetected = false;

  // Listen to console for errors
  page.on('console', msg => {
    const text = msg.text();
    if (msg.type() === 'error') {
      console.error('🔴', text);
      if (text.includes('SQLITE_CORRUPT')) {
        corruptionDetected = true;
      }
      hasErrors = true;
    } else if (text.includes('[HomePage Federated]')) {
      console.log('🔵', text);
    } else if (text.includes('[PerSiteFederationIndex]')) {
      console.log('🟢', text);
    }
  });

  console.log('🚀 Initial load...');
  await page.goto('http://localhost:5175', { waitUntil: 'networkidle' });
  await page.waitForTimeout(5000);

  // Check if content is visible
  const contentCards = await page.locator('.v-card').count();
  console.log(`📊 Initial load: Found ${contentCards} content cards`);
  if (contentCards > 0) successfulLoads++;

  // Stress test with multiple refreshes
  for (let i = 1; i <= 5; i++) {
    console.log(`\n🔄 Refresh attempt ${i}/5...`);
    
    // Hard refresh
    await page.reload({ waitUntil: 'networkidle' });
    await page.waitForTimeout(3000);
    
    // Check content again
    const cards = await page.locator('.v-card').count();
    console.log(`📊 After refresh ${i}: Found ${cards} content cards`);
    
    if (cards > 0) {
      successfulLoads++;
      console.log('✅ Content survived refresh!');
    } else {
      console.log('❌ Content lost after refresh');
      
      // Take screenshot for debugging
      await page.screenshot({ path: `federation-fail-refresh-${i}.png` });
      
      // Check for "No featured content" message
      const noContentMsg = await page.locator('text=No featured content').isVisible();
      if (noContentMsg) {
        console.log('⚠️  "No featured content" message is showing');
      }
    }
    
    if (corruptionDetected) {
      console.log('🚨 Database corruption detected! Attempting recovery...');
      
      // Click the corruption alert button if it exists
      const clearButton = page.locator('button:has-text("Clear Storage and Reload")');
      if (await clearButton.isVisible()) {
        console.log('🧹 Clicking clear storage button...');
        await clearButton.click();
        await page.waitForLoadState('load');
        await page.waitForTimeout(5000);
        corruptionDetected = false;
        continue;
      }
    }
  }

  // Test navigation
  console.log('\n🧭 Testing navigation...');
  await page.goto('http://localhost:5175/#/federation-stats');
  await page.waitForTimeout(2000);
  
  const statsButton = page.locator('button:has-text("Refresh Stats")');
  if (await statsButton.isVisible()) {
    await statsButton.click();
    await page.waitForTimeout(1000);
    
    // Check if stats show entries
    const statsText = await page.locator('body').textContent();
    if (statsText?.includes('Total Entries:')) {
      const match = statsText.match(/Total Entries: (\d+)/);
      if (match) {
        console.log(`📈 Federation index contains ${match[1]} entries`);
      }
    }
  }

  // Go back to home
  await page.goto('http://localhost:5175');
  await page.waitForTimeout(3000);
  
  const finalCards = await page.locator('.v-card').count();
  console.log(`\n📊 Final check: ${finalCards} content cards visible`);
  console.log(`✅ Successful loads: ${successfulLoads}/6`);
  console.log(`${hasErrors ? '❌ Errors detected' : '✅ No errors'}`);
  
  expect(successfulLoads).toBeGreaterThan(0);
});