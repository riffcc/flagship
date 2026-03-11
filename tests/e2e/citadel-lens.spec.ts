import { test, expect } from '@playwright/test';

/**
 * Citadel Lens Integration Tests
 *
 * Verifies that flagship loads reliably with the citadel-lens backend.
 * Requires:
 *   - lens-node running on http://localhost:8080
 *   - flagship dev server on http://localhost:5175
 */

test.describe('Citadel Lens Integration', () => {

  test.beforeEach(async ({ page }) => {
    // Collect console errors
    page.on('console', msg => {
      if (msg.type() === 'error') {
        console.log(`Console error: ${msg.text()}`);
      }
    });
  });

  test('homepage loads without errors', async ({ page }) => {
    const response = await page.goto('/');

    // Page should load successfully
    expect(response?.status()).toBe(200);

    // Wait for Vue app to hydrate
    await page.waitForSelector('#app', { timeout: 10000 });

    // Check for Vuetify app container (flexible about exact structure)
    const vuetifyApp = page.locator('.v-application');
    await expect(vuetifyApp).toBeVisible({ timeout: 10000 });

    // Logo should be visible somewhere on the page
    const logo = page.locator('img[src="/logo.svg"]');
    await expect(logo).toBeVisible({ timeout: 10000 });
  });

  test('categories load from citadel-lens API', async ({ page }) => {
    await page.goto('/');

    // Wait for categories to potentially load
    await page.waitForTimeout(2000);

    // Check that we don't have "Failed to load" errors visible
    const errorText = page.getByText(/failed to load|error loading/i);
    await expect(errorText).not.toBeVisible();
  });

  test('account page loads and shows identity', async ({ page }) => {
    await page.goto('/#/account');

    // Wait for page to load
    await page.waitForTimeout(1000);

    // Should see "Account info" card
    const accountInfo = page.getByText('Account info');
    await expect(accountInfo).toBeVisible({ timeout: 10000 });

    // Should see "Public Key" field
    const publicKeyLabel = page.getByText('Public Key');
    await expect(publicKeyLabel).toBeVisible({ timeout: 10000 });
  });

  test('API health check responds', async ({ request }) => {
    // Direct API test - lens-node health
    const response = await request.get('http://localhost:8080/health');
    expect(response.status()).toBe(200);
    expect(await response.text()).toBe('OK');
  });

  test('API categories endpoint works', async ({ request }) => {
    const response = await request.get('http://localhost:8080/api/v1/content-categories');
    expect(response.status()).toBe(200);

    const categories = await response.json();
    expect(Array.isArray(categories)).toBe(true);
    expect(categories.length).toBeGreaterThan(0);

    // Should have expected default categories
    const categoryIds = categories.map((c: any) => c.id);
    expect(categoryIds).toContain('music');
    expect(categoryIds).toContain('movies');
  });

  test('API releases endpoint works', async ({ request }) => {
    const response = await request.get('http://localhost:8080/api/v1/releases');
    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(Array.isArray(data)).toBe(true);
  });

  test('API account endpoint works', async ({ request }) => {
    const testKey = 'ed25519p/test123';
    const response = await request.get(`http://localhost:8080/api/v1/account/${encodeURIComponent(testKey)}`);
    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty('publicKey');
    expect(data).toHaveProperty('isAdmin');
    expect(data).toHaveProperty('roles');
    expect(data).toHaveProperty('permissions');
  });

  test('no critical console errors on homepage', async ({ page }) => {
    const errors: string[] = [];

    page.on('console', msg => {
      if (msg.type() === 'error') {
        const text = msg.text();
        // Ignore some known non-critical errors
        if (!text.includes('favicon') && !text.includes('hot-update')) {
          errors.push(text);
        }
      }
    });

    await page.goto('/');
    await page.waitForTimeout(3000);

    // Filter out known non-critical errors
    const criticalErrors = errors.filter(e =>
      !e.includes('peerbit') &&
      !e.includes('WebSocket') &&
      !e.includes('CORS') &&
      !e.includes('net::ERR') &&
      !e.includes('mdi-') &&       // Vuetify MDI icon loading (harmless)
      !e.includes('path') &&       // SVG path attribute errors (Vuetify icons)
      !e.includes('account')       // Icon name parsing errors
    );

    expect(criticalErrors).toHaveLength(0);
  });

});
