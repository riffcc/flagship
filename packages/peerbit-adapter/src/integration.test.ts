import type { Peerbit} from './peerbit';
import { createPeerbit } from './peerbit';
import type { Release } from './types';
import * as consts from './consts';


describe('Peerbit Integration with Flagship', () => {
  let peerbit: Peerbit;
  const mockSiteId = 'test-site-id';

  beforeEach(async () => {
    const { peerbit: instance } = await createPeerbit({ 
      siteId: mockSiteId,
    });
    peerbit = instance;
  });

  afterEach(async () => {
    await peerbit.close();
  });

  describe('Release management integration', () => {
    test('Add and retrieve releases', async () => {
      const testRelease: Release = {
        [consts.RELEASES_NAME_COLUMN]: 'Integration Test Release',
        [consts.RELEASES_FILE_COLUMN]: 'test-file-cid',
        [consts.RELEASES_AUTHOR_COLUMN]: 'Integration Test Author',
        [consts.RELEASES_CATEGORY_COLUMN]: 'music',
        [consts.RELEASES_COVER_COLUMN]: 'test-cover-cid',
      };

      await peerbit.addRelease(testRelease);

      const mockCallback = jest.fn();
      
      await peerbit.listenForReleases({ f: mockCallback });
      
      expect(mockCallback).toHaveBeenCalled();
      
      const releases = mockCallback.mock.calls[0][0];
      
      expect(releases.length).toBeGreaterThan(0);
      expect(releases.some(r => 
        r.release.contentName === testRelease.contentName &&
        r.release.file === testRelease.file &&
        r.release.author === testRelease.author,
      )).toBe(true);
    });
  });

  describe('Content category integration', () => {
    test('Add and retrieve content categories', async () => {
      const categoryData = {
        [consts.CONTENT_CATEGORIES_CATEGORY_ID]: 'integration-test-category',
        [consts.CONTENT_CATEGORIES_DISPLAY_NAME]: 'Integration Test Category',
        [consts.CONTENT_CATEGORIES_FEATURED]: true,
        [consts.CONTENT_CATEGORIES_METADATA_SCHEMA]: {
          testField: {
            type: 'string',
            description: 'Test field for integration testing',
          },
        },
      };

      await peerbit.addCategory(categoryData);

      const mockCallback = jest.fn();
      
      await peerbit.listenForContentCategories({ f: mockCallback });
      
      expect(mockCallback).toHaveBeenCalled();
      
      const categories = mockCallback.mock.calls[0][0];
      
      expect(categories.length).toBeGreaterThan(0);
      expect(categories.some(c => 
        c.contentCategory.categoryId === 'integration-test-category' &&
        c.contentCategory.displayName === 'Integration Test Category',
      )).toBe(true);
    });
  });

  describe('Site management integration', () => {
    test('Trust and retrieve trusted sites', async () => {
      const testSiteId = 'integration-test-site-id';
      await peerbit.trustSite({ siteName: 'Integration Test Site', siteId: testSiteId });

      const mockCallback = jest.fn();
      
      await peerbit.followTrustedSites({ f: mockCallback });
      
      expect(mockCallback).toHaveBeenCalled();
      
      const trustedSites = mockCallback.mock.calls[0][0];
      
      expect(trustedSites.length).toBeGreaterThan(0);
      expect(trustedSites.some(s => s.data.siteId === testSiteId)).toBe(true);
    });
  });

  describe('Profile management integration', () => {
    test('Change name and retrieve profile', async () => {
      const testName = 'Integration Test User';
      const testLanguage = 'en';
      await peerbit.changeName({ name: testName, language: testLanguage });

      const mockCallback = jest.fn();
      
      await peerbit.listenForNameChange(mockCallback, { accountId: await peerbit.getAccountId() });
      
      expect(mockCallback).toHaveBeenCalled();
      
      const names = mockCallback.mock.calls[0][0];
      
      expect(names).toHaveProperty(testLanguage, testName);
    });
  });

  describe('Moderation integration', () => {
    test('Block and retrieve blocked releases', async () => {
      const testCid = 'integration-test-blocked-cid';
      await peerbit.blockRelease({ cid: testCid });

      const mockCallback = jest.fn();
      
      await peerbit.followBlockedReleases({ f: mockCallback });
      
      expect(mockCallback).toHaveBeenCalled();
      
      const blockedReleases = mockCallback.mock.calls[0][0];
      
      expect(blockedReleases.length).toBeGreaterThan(0);
      expect(blockedReleases.some(b => b.data.contentCID === testCid)).toBe(true);
    });
  });

  describe('Featured releases integration', () => {
    test('Feature and retrieve featured releases', async () => {
      const testRelease: Release = {
        [consts.RELEASES_NAME_COLUMN]: 'Featured Integration Test Release',
        [consts.RELEASES_FILE_COLUMN]: 'featured-test-file-cid',
        [consts.RELEASES_AUTHOR_COLUMN]: 'Featured Integration Test Author',
        [consts.RELEASES_CATEGORY_COLUMN]: 'music',
        [consts.RELEASES_COVER_COLUMN]: 'featured-test-cover-cid',
      };

      await peerbit.addRelease(testRelease);
      const releaseId = 'test-release-id';

      await peerbit.featureRelease({ 
        releaseId, 
        siteId: mockSiteId, 
      });

      const mockCallback = jest.fn();
      
      await peerbit.listenForSiteFeaturedReleases({ f: mockCallback, siteId: mockSiteId });
      
      expect(mockCallback).toHaveBeenCalled();
      
      const featuredReleases = mockCallback.mock.calls[0][0];
      
      expect(featuredReleases.length).toBeGreaterThan(0);
      expect(featuredReleases.some(f => f.data.releaseId === releaseId)).toBe(true);
    });
  });
});
