import { test } from '@playwright/test';

test('simple debug output', async ({ page }) => {
  // Capture console output and immediately print it
  page.on('console', msg => {
    const text = msg.text();
    // Print all logs that contain our debug markers
    if (text.includes('RELEASE DETAILS DEBUG') || 
        text.includes('RELEASE_') || 
        text.includes('END RELEASE DETAILS') ||
        text.includes('Total releases found:')) {
      console.log(text);
    }
  });

  await page.goto('http://localhost:5175');
  await page.waitForTimeout(10000);
  
  console.log('=== Debug test complete ===');
});