import { test, expect } from '@playwright/test';

test.describe('Basic Smoke Test', () => {

  test('should load the homepage and display the main layout', async ({ page }) => {
    await page.goto('/');

    // 1. Look for the <header> rendered by <v-app-bar>. This appears immediately.
    const appBar = page.locator('header.v-app-bar');
    await expect(appBar).toBeVisible();

    // 2. Check for the logo inside the app bar.
    const logo = appBar.locator('img[src="/logo.svg"]');
    await expect(logo).toBeVisible();

    // 3. Look for the <main> tag rendered by <v-main>. This is a better selector.
    // This confirms the core layout of the app is present.
    const mainElement = page.locator('main.v-main');
    await expect(mainElement).toBeVisible();
  });
});
