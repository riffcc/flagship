import { test, expect } from '@playwright/test';

test('debug release details', async ({ page }) => {
  let foundReleaseDetails = false;
  const releaseDetailLogs: string[] = [];
  
  // Capture all console logs
  page.on('console', async msg => {
    const text = msg.text();
    
    // Log everything for debugging
    console.log('Console:', text);
    
    // Track when we find release details
    if (text.includes('[Site] Release details:')) {
      console.log('\n=== FOUND RELEASE DETAILS MARKER ===');
      foundReleaseDetails = true;
    }
    
    // Capture logs after finding the marker
    if (foundReleaseDetails) {
      releaseDetailLogs.push(text);
      
      // If this is a release detail log, try to parse it
      if (text.includes('[Site] Release') && text.includes('{')) {
        try {
          // Extract the JSON part
          const jsonStart = text.indexOf('{');
          if (jsonStart !== -1) {
            const jsonStr = text.substring(jsonStart);
            const releaseData = JSON.parse(jsonStr);
            console.log('\n=== PARSED RELEASE DATA ===');
            console.log('Name:', releaseData.name);
            console.log('ID:', releaseData.id);
            console.log('ContentCID:', releaseData.contentCID);
            console.log('Has ContentCID:', releaseData.hasContentCID);
            console.log('Federated From:', releaseData.federatedFrom);
            console.log('Author:', releaseData.author);
            console.log('===========================\n');
          }
        } catch (e) {
          // If parsing fails, just print the raw log
          console.log('Raw log:', text);
        }
      }
    }
  });

  // Navigate to the app
  await page.goto('http://localhost:5175');
  
  // Wait for the app to initialize and load data
  await page.waitForTimeout(10000);
  
  // Try to trigger a re-render by clicking on the releases link
  const releasesLink = page.locator('text=Releases').first();
  if (await releasesLink.isVisible()) {
    await releasesLink.click();
    await page.waitForTimeout(3000);
  }
  
  // Print summary
  console.log('\n=== SUMMARY ===');
  console.log('Found release details marker:', foundReleaseDetails);
  console.log('Number of logs after marker:', releaseDetailLogs.length);
  
  // Keep browser open for manual inspection
  console.log('\nKeeping browser open for 30 seconds...');
  await page.waitForTimeout(30000);
});