import type { Peerbit} from './peerbit';
import { createPeerbit } from './peerbit';
import type { Release, Collection } from './types';
import * as consts from './consts';

jest.mock('@peerbit/peer', () => ({
  Peer: jest.fn().mockImplementation(() => ({
    start: jest.fn().mockResolvedValue(undefined),
    stop: jest.fn().mockResolvedValue(undefined),
    id: 'mock-peer-id',
  })),
}));

jest.mock('@peerbit/pubsub', () => ({
  Pubsub: jest.fn().mockImplementation(() => ({
    subscribe: jest.fn().mockResolvedValue(() => Promise.resolve()),
    publish: jest.fn().mockResolvedValue(undefined),
  })),
}));

jest.mock('@peerbit/document', () => {
  const mockDocumentCollection = {
    put: jest.fn().mockResolvedValue('test-id'),
    get: jest.fn().mockResolvedValue({ id: 'test-id', value: {}, creator: 'test-creator' }),
    getAll: jest.fn().mockResolvedValue([
      { id: 'test-id-1', value: { title: 'Test 1' }, creator: 'test-creator-1' },
      { id: 'test-id-2', value: { title: 'Test 2' }, creator: 'test-creator-2' },
    ]),
    delete: jest.fn().mockResolvedValue(undefined),
    update: jest.fn().mockResolvedValue(undefined),
  };

  return {
    Document: {
      createDocumentCollection: jest.fn().mockResolvedValue(mockDocumentCollection),
    },
  };
});

describe('Peerbit', () => {
  let peerbit: Peerbit;
  const mockSiteId = 'test-site-id';

  beforeEach(async () => {
    jest.clearAllMocks();
    const { peerbit: instance } = await createPeerbit({ siteId: mockSiteId });
    peerbit = instance;
  });

  afterEach(async () => {
    await peerbit.close();
  });

  describe('Basic functionality', () => {
    test('getSiteId returns the correct site ID', () => {
      expect(peerbit.getSiteId()).toBe(mockSiteId);
    });

    test('getPeerId returns the peer ID', () => {
      expect(peerbit.getPeerId()).toBe('mock-peer-id');
    });
  });

  describe('Release management', () => {
    const mockRelease: Release = {
      [consts.RELEASES_NAME_COLUMN]: 'Test Release',
      [consts.RELEASES_FILE_COLUMN]: 'test-file-cid',
      [consts.RELEASES_AUTHOR_COLUMN]: 'Test Author',
      [consts.RELEASES_CATEGORY_COLUMN]: 'music',
      [consts.RELEASES_COVER_COLUMN]: 'test-cover-cid',
    };

    test('addRelease adds a release', async () => {
      await expect(peerbit.addRelease(mockRelease)).resolves.not.toThrow();
    });

    test('removeRelease removes a release', async () => {
      await expect(peerbit.removeRelease('test-id')).resolves.not.toThrow();
    });

    test('editRelease updates a release', async () => {
      const updatedRelease = { ...mockRelease, [consts.RELEASES_NAME_COLUMN]: 'Updated Release' };
      await expect(peerbit.editRelease({ releaseId: 'test-id', release: updatedRelease })).resolves.not.toThrow();
    });

    test('listenForReleases returns releases and sets up subscription', async () => {
      const mockCallback = jest.fn();
      await peerbit.listenForReleases({ f: mockCallback });
      
      expect(mockCallback).toHaveBeenCalledWith(expect.arrayContaining([
        expect.objectContaining({
          release: expect.objectContaining({
            id: 'test-id-1',
          }),
          contributor: 'test-creator-1',
          site: mockSiteId,
        }),
        expect.objectContaining({
          release: expect.objectContaining({
            id: 'test-id-2',
          }),
          contributor: 'test-creator-2',
          site: mockSiteId,
        }),
      ]));
    });
  });

  describe('Collection management', () => {
    const mockCollection: Collection = {
      [consts.COLLECTIONS_NAME_COLUMN]: 'Test Collection',
      [consts.COLLECTIONS_CATEGORY_COLUMN]: 'music',
      [consts.COLLECTIONS_RELEASES_COLUMN]: 'release1,release2',
    };

    test('addCollection adds a collection', async () => {
      await expect(peerbit.addCollection(mockCollection)).resolves.not.toThrow();
    });

    test('removeCollection removes a collection', async () => {
      await expect(peerbit.removeCollection('test-id')).resolves.not.toThrow();
    });

    test('editCollection updates a collection', async () => {
      const updatedCollection = { ...mockCollection, [consts.COLLECTIONS_NAME_COLUMN]: 'Updated Collection' };
      await expect(peerbit.editCollection({ collectionId: 'test-id', collection: updatedCollection })).resolves.not.toThrow();
    });

    test('listenForCollections returns collections and sets up subscription', async () => {
      const mockCallback = jest.fn();
      await peerbit.listenForCollections({ f: mockCallback });
      
      expect(mockCallback).toHaveBeenCalledWith(expect.arrayContaining([
        expect.objectContaining({
          collection: expect.objectContaining({
            id: 'test-id-1',
          }),
          contributor: 'test-creator-1',
          site: mockSiteId,
        }),
        expect.objectContaining({
          collection: expect.objectContaining({
            id: 'test-id-2',
          }),
          contributor: 'test-creator-2',
          site: mockSiteId,
        }),
      ]));
    });
  });

  describe('Site management', () => {
    test('trustSite adds a trusted site', async () => {
      const siteId = 'trusted-site-id';
      await expect(peerbit.trustSite({ siteName: 'Trusted Site', siteId })).resolves.not.toThrow();
    });

    test('untrustSite removes a trusted site', async () => {
      const siteId = 'trusted-site-id';
      await expect(peerbit.untrustSite({ siteId })).resolves.not.toThrow();
    });

    test('followTrustedSites returns trusted sites and sets up subscription', async () => {
      const mockCallback = jest.fn();
      await peerbit.followTrustedSites({ f: mockCallback });
      
      expect(mockCallback).toHaveBeenCalledWith(expect.arrayContaining([
        expect.objectContaining({
          id: 'test-id-1',
          data: expect.any(Object),
        }),
        expect.objectContaining({
          id: 'test-id-2',
          data: expect.any(Object),
        }),
      ]));
    });
  });

  describe('Content moderation', () => {
    test('blockRelease blocks a release', async () => {
      const cid = 'blocked-content-cid';
      await expect(peerbit.blockRelease({ cid })).resolves.not.toThrow();
    });

    test('unblockRelease unblocks a release', async () => {
      await expect(peerbit.unblockRelease({ id: 'blocked-id' })).resolves.not.toThrow();
    });

    test('featureRelease features a release', async () => {
      const releaseId = 'featured-release-id';
      const siteId = 'featured-site-id';
      await expect(peerbit.featureRelease({ releaseId, siteId })).resolves.not.toThrow();
    });
  });

  describe('Category management', () => {
    const mockCategory = {
      [consts.CONTENT_CATEGORIES_CATEGORY_ID]: 'test-category-id',
      [consts.CONTENT_CATEGORIES_DISPLAY_NAME]: 'Test Category',
      [consts.CONTENT_CATEGORIES_FEATURED]: true,
      [consts.CONTENT_CATEGORIES_METADATA_SCHEMA]: {
        testField: {
          type: 'string',
          description: 'Test field',
        },
      },
    };

    test('addCategory adds a category', async () => {
      await expect(peerbit.addCategory(mockCategory)).resolves.not.toThrow();
    });

    test('listenForContentCategories returns categories and sets up subscription', async () => {
      const mockCallback = jest.fn();
      await peerbit.listenForContentCategories({ f: mockCallback });
      
      expect(mockCallback).toHaveBeenCalledWith(expect.arrayContaining([
        expect.objectContaining({
          id: 'test-id-1',
          contentCategory: expect.any(Object),
        }),
        expect.objectContaining({
          id: 'test-id-2',
          contentCategory: expect.any(Object),
        }),
      ]));
    });
  });

  describe('Profile management', () => {
    test('changeName changes the user name', async () => {
      const name = 'Test User';
      const language = 'en';
      await expect(peerbit.changeName({ name, language })).resolves.not.toThrow();
    });

    test('initializeProfile initializes the user profile', async () => {
      await expect(peerbit.initializeProfile()).resolves.not.toThrow();
    });
  });

  describe('IPFS file handling', () => {
    test('getIPFSFile retrieves a file from IPFS', async () => {
      const id = 'test-ipfs-file-id';
      const mockBuffer = Buffer.from('test file content');
      
      jest.spyOn(peerbit, 'getIPFSFile').mockResolvedValueOnce({
        buffer: mockBuffer,
        size: mockBuffer.length,
      });
      
      const result = await peerbit.getIPFSFile({ id });
      expect(result).toEqual({
        buffer: mockBuffer,
        size: mockBuffer.length,
      });
    });
  });

  describe('Error handling', () => {
    test('handles errors when adding a release', async () => {
      jest.spyOn(peerbit, 'addRelease').mockRejectedValueOnce(new Error('Failed to add release'));
      
      await expect(peerbit.addRelease({} as Release)).rejects.toThrow('Failed to add release');
    });

    test('handles errors when removing a release', async () => {
      jest.spyOn(peerbit, 'removeRelease').mockRejectedValueOnce(new Error('Failed to remove release'));
      
      await expect(peerbit.removeRelease('test-id')).rejects.toThrow('Failed to remove release');
    });
  });
});
