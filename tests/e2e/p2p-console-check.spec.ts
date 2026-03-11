import { test, expect } from '@playwright/test';

test('P2P console activity check', async ({ page }) => {
  // Collect all console logs
  const logs: Array<{ type: string; text: string }> = [];

  page.on('console', msg => {
    logs.push({
      type: msg.type(),
      text: msg.text(),
    });
  });

  // Navigate and wait for page to settle
  await page.goto('/', { waitUntil: 'networkidle', timeout: 30000 });

  // Wait for app initialization
  await page.waitForTimeout(5000);

  // Print all console logs
  console.log('\n=== ALL CONSOLE LOGS ===');
  logs.forEach(log => {
    console.log(`[${log.type.toUpperCase()}] ${log.text}`);
  });

  // Filter P2P-related logs
  const p2pLogs = logs.filter(log =>
    log.text.includes('[P2P]') ||
    log.text.includes('[App]') ||
    log.text.includes('relay') ||
    log.text.includes('P2P')
  );

  console.log('\n=== P2P-RELATED LOGS ===');
  p2pLogs.forEach(log => {
    console.log(`[${log.type.toUpperCase()}] ${log.text}`);
  });

  // Check for P2P initialization
  const hasP2PInit = logs.some(log =>
    log.text.includes('P2P relay connection') ||
    log.text.includes('[P2P]')
  );

  console.log('\n=== TEST RESULTS ===');
  console.log(`Total logs: ${logs.length}`);
  console.log(`P2P-related logs: ${p2pLogs.length}`);
  console.log(`Has P2P initialization: ${hasP2PInit}`);

  // Take a screenshot
  await page.screenshot({ path: 'test-results/p2p-test-screenshot.png' });

  // Check if P2P status indicator is visible
  const p2pIndicator = page.locator('.p2p-status-indicator');
  const isVisible = await p2pIndicator.isVisible({ timeout: 1000 }).catch(() => false);

  console.log(`P2P indicator visible: ${isVisible}`);

  if (isVisible) {
    const text = await p2pIndicator.textContent();
    console.log(`P2P indicator text: "${text}"`);
  }

  // The test passes if we see P2P activity
  expect(p2pLogs.length).toBeGreaterThan(0);
});
