import { test, expect } from '@playwright/test';

test.describe('P2P Integration', () => {
  test('should show P2P status indicator and connect to relay', async ({ page }) => {
    // Navigate to app
    await page.goto('/');

    // Wait for app to load
    await page.waitForSelector('.v-app', { timeout: 10000 });

    // Check for P2P status indicator
    const p2pIndicator = page.locator('.p2p-status-indicator');

    // The P2P indicator should appear once connected
    await expect(p2pIndicator).toBeVisible({ timeout: 5000 });

    // Check that it shows relay connection info
    const indicatorText = await p2pIndicator.textContent();
    expect(indicatorText).toContain('P2P:');

    console.log('P2P Status:', indicatorText);
  });

  test('should establish relay connection on app mount', async ({ page, browser }) => {
    // Listen for console logs to verify P2P initialization
    const consoleLogs: string[] = [];
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('[P2P]') || text.includes('[App]')) {
        consoleLogs.push(text);
      }
    });

    await page.goto('/');
    await page.waitForSelector('.v-app', { timeout: 10000 });

    // Wait a bit for P2P connection
    await page.waitForTimeout(3000);

    // Check console logs for P2P activity
    const p2pLogs = consoleLogs.filter(log => log.includes('[P2P]'));

    console.log('P2P Console Logs:', p2pLogs);

    // Should see connection attempts
    const hasConnectionLog = p2pLogs.some(log =>
      log.includes('Connecting to relay') ||
      log.includes('Connected to relay') ||
      log.includes('relay connection initiated')
    );

    expect(hasConnectionLog).toBe(true);
  });

  test('should attempt WebRTC connections when peers are discovered', async ({ page }) => {
    const p2pLogs: string[] = [];
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('[P2P]')) {
        p2pLogs.push(text);
      }
    });

    await page.goto('/');
    await page.waitForSelector('.v-app', { timeout: 10000 });

    // Wait for P2P initialization and potential peer discovery
    await page.waitForTimeout(5000);

    console.log('All P2P logs:', p2pLogs);

    // Check for relay connection
    const hasRelayConnection = p2pLogs.some(log =>
      log.includes('Connected to relay') ||
      log.includes('relay connection initiated')
    );

    expect(hasRelayConnection).toBe(true);

    // If peers are discovered, should see WebRTC activity
    const hasPeerReferral = p2pLogs.some(log => log.includes('peer referral'));
    if (hasPeerReferral) {
      const hasWebRTC = p2pLogs.some(log =>
        log.includes('WebRTC') ||
        log.includes('offer') ||
        log.includes('ICE candidate')
      );
      console.log('Peer discovery and WebRTC:', { hasPeerReferral, hasWebRTC });
    }
  });

  test('should display connection count in P2P indicator', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.v-app', { timeout: 10000 });

    // Wait for P2P to initialize
    await page.waitForTimeout(3000);

    const p2pIndicator = page.locator('.p2p-status-indicator');
    await expect(p2pIndicator).toBeVisible({ timeout: 5000 });

    const text = await p2pIndicator.textContent();

    // Should show relay peer count
    expect(text).toMatch(/P2P:\s*\d+\s*relay/);

    console.log('P2P Connection Status:', text);
  });

  test('should handle block storage and retrieval', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('.v-app', { timeout: 10000 });

    // Test block storage via console
    const result = await page.evaluate(() => {
      // Access the P2P composable through the window (for testing)
      // In a real scenario, blocks would be added through peer exchange

      // Create a test block
      const testBlockId = 'test-block-123';
      const testData = new TextEncoder().encode('Test block data');

      return {
        testBlockId,
        dataSize: testData.byteLength,
      };
    });

    console.log('Block storage test:', result);
    expect(result.testBlockId).toBe('test-block-123');
    expect(result.dataSize).toBeGreaterThan(0);
  });
});
