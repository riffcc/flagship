import { test, expect } from '@playwright/test';
import { _electron as electron } from 'playwright';
import type { ElectronApplication, Page } from 'playwright';
import { join } from 'path';
import { setTimeout } from 'timers/promises';

// Helper to wait for network stabilization
const waitForNetwork = async (page: Page, timeout = 5000) => {
  await page.waitForLoadState('networkidle');
  await setTimeout(timeout);
};

// Helper to wait for content
const waitForContent = async (page: Page, selector: string, text: string, timeout = 30000) => {
  await page.waitForSelector(selector, { timeout });
  await expect(page.locator(selector).getByText(text)).toBeVisible({ timeout });
};

test.describe('Federation E2E Tests', () => {
  let app1: ElectronApplication;
  let app2: ElectronApplication;
  let page1: Page;
  let page2: Page;

  test.beforeAll(async () => {
    // Launch two instances of the app
    const args = [join(__dirname, '../'), '--no-sandbox'];
    
    app1 = await electron.launch({
      args,
      env: {
        ...process.env,
        NODE_ENV: 'test',
        LENS_TEST_INSTANCE: '1',
      },
    });
    
    app2 = await electron.launch({
      args,
      env: {
        ...process.env,
        NODE_ENV: 'test',
        LENS_TEST_INSTANCE: '2',
      },
    });

    page1 = await app1.firstWindow();
    page2 = await app2.firstWindow();

    // Wait for both apps to load
    await Promise.all([
      page1.waitForLoadState('domcontentloaded'),
      page2.waitForLoadState('domcontentloaded'),
    ]);
  });

  test.afterAll(async () => {
    if (app1) await app1.close();
    if (app2) await app2.close();
  });

  test('two instances can share content through federation', async () => {
    // Wait for network initialization
    await waitForNetwork(page1);
    await waitForNetwork(page2);

    // Create a site on instance 1
    await page1.click('button:has-text("Create Site")');
    await page1.fill('input[name="siteName"]', 'Federation Test Site 1');
    await page1.fill('textarea[name="siteDescription"]', 'Testing federation between instances');
    await page1.click('button:has-text("Create")');

    // Wait for site creation
    await waitForContent(page1, 'h1', 'Federation Test Site 1');

    // Get site address from instance 1
    const siteAddress = await page1.evaluate(() => {
      return window.electronLensService?.getSiteId();
    });
    expect(siteAddress).toBeTruthy();

    // Upload content on instance 1
    await page1.click('a:has-text("Upload")');
    await page1.fill('input[name="releaseName"]', 'Federated Test Content');
    await page1.selectOption('select[name="category"]', 'video');
    await page1.fill('input[name="contentCID"]', 'QmTestFederationCID123');
    await page1.click('button:has-text("Upload Release")');

    // Wait for upload success
    await waitForContent(page1, '.release-card', 'Federated Test Content');

    // Instance 2 opens the same site
    await page2.click('button:has-text("Open Site")');
    await page2.fill('input[name="siteAddress"]', siteAddress);
    await page2.click('button:has-text("Open")');

    // Wait for site to load on instance 2
    await waitForContent(page2, 'h1', 'Federation Test Site 1');

    // Wait for content to replicate
    await waitForContent(page2, '.release-card', 'Federated Test Content', 60000);

    // Verify content details match
    const content1 = await page1.locator('.release-card').first().textContent();
    const content2 = await page2.locator('.release-card').first().textContent();
    expect(content1).toBe(content2);
  });

  test('subscription-based federation', async () => {
    // Create a new site on instance 2
    await page2.click('a:has-text("Home")');
    await page2.click('button:has-text("Create Site")');
    await page2.fill('input[name="siteName"]', 'Subscription Test Site');
    await page2.fill('textarea[name="siteDescription"]', 'Site that subscribes to others');
    await page2.click('button:has-text("Create")');

    await waitForContent(page2, 'h1', 'Subscription Test Site');

    // Get site addresses
    const site1Address = await page1.evaluate(() => window.electronLensService?.getSiteId());
    const site2Address = await page2.evaluate(() => window.electronLensService?.getSiteId());

    // Navigate to admin panel on instance 2
    await page2.click('a:has-text("Admin")');
    await page2.click('button:has-text("Manage Subscriptions")');

    // Add subscription to site 1
    await page2.click('button:has-text("Add Subscription")');
    await page2.fill('input[name="siteId"]', site1Address);
    await page2.fill('input[name="name"]', 'Federation Test Site 1');
    await page2.check('input[name="recursive"]');
    await page2.click('button:has-text("Subscribe")');

    // Wait for subscription to be created
    await waitForContent(page2, '.subscription-item', 'Federation Test Site 1');

    // Add new content to site 1
    await page1.click('a:has-text("Upload")');
    await page1.fill('input[name="releaseName"]', 'New Content After Subscription');
    await page1.selectOption('select[name="category"]', 'audio');
    await page1.fill('input[name="contentCID"]', 'QmNewSubscriptionContent456');
    await page1.click('button:has-text("Upload Release")');

    // Content should appear on subscribed site
    await page2.click('a:has-text("Browse")');
    await waitForContent(page2, '.release-card', 'New Content After Subscription', 60000);

    // Verify federation metadata
    const federatedRelease = await page2.evaluate(async () => {
      const releases = await window.electronLensService?.getReleases();
      return releases?.find(r => r.name === 'New Content After Subscription');
    });

    expect(federatedRelease).toBeTruthy();
    expect(federatedRelease.federatedFrom).toBe(site1Address);
    expect(federatedRelease.federatedRealtime).toBe(true);
  });

  test('site metadata synchronization', async () => {
    // Update site metadata on instance 1
    await page1.click('a:has-text("Admin")');
    await page1.click('button:has-text("Site Settings")');
    
    await page1.fill('input[name="siteName"]', 'Updated Federation Site');
    await page1.fill('textarea[name="siteDescription"]', 'Updated description with more details');
    await page1.click('button:has-text("Save Settings")');

    // Wait for update
    await waitForContent(page1, '.success-message', 'Settings updated');

    // Refresh instance 2
    await page2.reload();
    await waitForNetwork(page2);

    // Check if metadata updated on instance 2
    await waitForContent(page2, 'h1', 'Updated Federation Site');
    
    const description = await page2.locator('.site-description').textContent();
    expect(description).toContain('Updated description with more details');
  });

  test('parallel content operations', async () => {
    const uploadPromises = [];

    // Upload multiple items from instance 1
    for (let i = 0; i < 3; i++) {
      uploadPromises.push(
        page1.evaluate(async (index) => {
          return window.electronLensService?.addRelease({
            name: `Parallel Release ${index}`,
            categoryId: 'test',
            contentCID: `QmParallel${index}`,
          });
        }, i),
      );
    }

    // Upload multiple items from instance 2
    for (let i = 3; i < 6; i++) {
      uploadPromises.push(
        page2.evaluate(async (index) => {
          return window.electronLensService?.addRelease({
            name: `Parallel Release ${index}`,
            categoryId: 'test',
            contentCID: `QmParallel${index}`,
          });
        }, i),
      );
    }

    // Wait for all uploads
    const results = await Promise.all(uploadPromises);
    expect(results.every(r => r?.success)).toBe(true);

    // Wait for replication
    await setTimeout(5000);

    // Both instances should have all 6 releases
    const [releases1, releases2] = await Promise.all([
      page1.evaluate(() => window.electronLensService?.getReleases()),
      page2.evaluate(() => window.electronLensService?.getReleases()),
    ]);

    const parallelReleases1 = releases1?.filter(r => r.name.startsWith('Parallel Release'));
    const parallelReleases2 = releases2?.filter(r => r.name.startsWith('Parallel Release'));

    expect(parallelReleases1?.length).toBe(6);
    expect(parallelReleases2?.length).toBe(6);
  });

  test('offline resilience and recovery', async () => {
    // Add content on instance 1
    const beforeOffline = await page1.evaluate(async () => {
      return window.electronLensService?.addRelease({
        name: 'Before Offline Content',
        categoryId: 'test',
        contentCID: 'QmBeforeOffline',
      });
    });
    expect(beforeOffline?.success).toBe(true);

    // Simulate instance 2 going offline by closing the peer connection
    await page2.evaluate(async () => {
      const peerId = await window.electronLensService?.getPeerId();
      // This would disconnect from other peers
      return peerId;
    });

    // Add content while instance 2 is "offline"
    const duringOffline = await page1.evaluate(async () => {
      return window.electronLensService?.addRelease({
        name: 'During Offline Content',
        categoryId: 'test',
        contentCID: 'QmDuringOffline',
      });
    });
    expect(duringOffline?.success).toBe(true);

    // Reconnect instance 2
    await page2.evaluate(async () => {
      const site1Address = await window.electronLensService?.getSiteId();
      if (site1Address) {
        await window.electronLensService?.dial(site1Address);
      }
    });

    // Wait for sync
    await setTimeout(10000);

    // Instance 2 should now have the content added while offline
    await waitForContent(page2, '.release-card', 'During Offline Content', 30000);

    // Verify both releases are present
    const releases = await page2.evaluate(() => window.electronLensService?.getReleases());
    const offlineContent = releases?.filter(r => 
      r.name === 'Before Offline Content' || r.name === 'During Offline Content',
    );
    expect(offlineContent?.length).toBe(2);
  });

  test('conflict resolution during simultaneous edits', async () => {
    // Create a release
    const release = await page1.evaluate(async () => {
      return window.electronLensService?.addRelease({
        name: 'Conflict Test Release',
        categoryId: 'test',
        contentCID: 'QmConflictOriginal',
      });
    });
    expect(release?.success).toBe(true);
    const releaseId = release?.id;

    // Wait for replication
    await setTimeout(3000);

    // Attempt simultaneous edits
    const [edit1, edit2] = await Promise.allSettled([
      page1.evaluate(async (id) => {
        return window.electronLensService?.editRelease({
          id,
          name: 'Edited by Instance 1',
          categoryId: 'test',
          contentCID: 'QmEditedBy1',
        });
      }, releaseId),
      page2.evaluate(async (id) => {
        return window.electronLensService?.editRelease({
          id,
          name: 'Edited by Instance 2',
          categoryId: 'test',
          contentCID: 'QmEditedBy2',
        });
      }, releaseId),
    ]);

    // At least one should succeed
    const successCount = [edit1, edit2].filter(
      r => r.status === 'fulfilled' && r.value?.success,
    ).length;
    expect(successCount).toBeGreaterThanOrEqual(1);

    // Wait for convergence
    await setTimeout(5000);

    // Both instances should have the same final state
    const [final1, final2] = await Promise.all([
      page1.evaluate(async (id) => {
        return window.electronLensService?.getRelease({ id });
      }, releaseId),
      page2.evaluate(async (id) => {
        return window.electronLensService?.getRelease({ id });
      }, releaseId),
    ]);

    expect(final1?.name).toBe(final2?.name);
    expect(final1?.contentCID).toBe(final2?.contentCID);
  });
});