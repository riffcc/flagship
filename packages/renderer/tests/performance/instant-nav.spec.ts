import { test, expect } from '@playwright/test';

test.describe('Instant Navigation Performance', () => {
  test('measures navigation from category to release page', async ({ page }) => {
    // Go to music category (endless scroll)
    await page.goto('/music');

    // Wait for releases to load
    await page.waitForSelector('[data-navigable="true"]');

    // Get the first album tile
    const firstAlbum = page.locator('[data-navigable="true"]').first();
    await expect(firstAlbum).toBeVisible();

    // Measure navigation time
    const startTime = performance.now();

    // Click the album
    await firstAlbum.click();

    // Wait for track list to appear (not loading spinner)
    await page.waitForSelector('.v-list-item', { timeout: 5000 });

    const endTime = performance.now();
    const navigationTime = endTime - startTime;

    console.log(`\n🚀 NAVIGATION TIME: ${navigationTime.toFixed(2)}ms\n`);

    // Assert it's under 300ms (faster than human perception)
    expect(navigationTime).toBeLessThan(300);
  });

  test('measures time to first track visible', async ({ page }) => {
    await page.goto('/music');
    await page.waitForSelector('[data-navigable="true"]');

    const firstAlbum = page.locator('[data-navigable="true"]').first();

    // Start measuring from click
    const navigationPromise = page.waitForURL(/\/release\//);
    const trackVisiblePromise = page.waitForSelector('.v-list-item');

    const clickTime = Date.now();
    await firstAlbum.click();

    await Promise.all([navigationPromise, trackVisiblePromise]);
    const trackVisibleTime = Date.now();

    const timeToTrack = trackVisibleTime - clickTime;
    console.log(`\n🎵 TIME TO FIRST TRACK: ${timeToTrack}ms\n`);

    // Should be instant - under 200ms
    expect(timeToTrack).toBeLessThan(200);
  });

  test('measures multiple navigations', async ({ page }) => {
    const times: number[] = [];

    for (let i = 0; i < 5; i++) {
      await page.goto('/music');
      await page.waitForSelector('[data-navigable="true"]');

      const albums = page.locator('[data-navigable="true"]');
      const album = albums.nth(i);

      const start = Date.now();
      await album.click();
      await page.waitForSelector('.v-list-item');
      const end = Date.now();

      times.push(end - start);

      // Go back for next iteration
      await page.goBack();
    }

    const avg = times.reduce((a, b) => a + b, 0) / times.length;
    const min = Math.min(...times);
    const max = Math.max(...times);

    console.log(`\n📊 NAVIGATION PERFORMANCE (5 runs):`);
    console.log(`   Average: ${avg.toFixed(2)}ms`);
    console.log(`   Min: ${min}ms`);
    console.log(`   Max: ${max}ms`);
    console.log(`   All times: ${times.join('ms, ')}ms\n`);

    // Average should be under 250ms
    expect(avg).toBeLessThan(250);
  });
});
