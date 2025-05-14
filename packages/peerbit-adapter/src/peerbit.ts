import { TypedEmitter } from 'tiny-typed-emitter';
import { v4 as uuidv4 } from 'uuid';

class Document {
  static async createDocumentCollection<T>(_options: CreateDocumentCollectionOptions): Promise<DocumentCollection<T>> {
    return {
      put: async (value: T, _id?: string) => _id || 'mock-id',
      delete: async (_id: string) => {},
      get: async (_id: string) => ({ id: _id, value: {} as T, creator: 'mock-creator' }),
      getAll: async () => [],
    };
  }
}

interface DocumentCollection<T> {
  put: (value: T, id?: string) => Promise<string>;
  delete: (id: string) => Promise<void>;
  get: (id: string) => Promise<{ id: string; value: T; creator?: string }>;
  getAll: () => Promise<Array<{ id: string; value: T; creator?: string }>>;
}

interface CreateDocumentCollectionOptions {
  peer: Peer;
  indexes?: string[];
}

class Pubsub {
  constructor(_options: { peer: Peer }) {}
  
  async subscribe(_topic: string, _callback: (message: Record<string, unknown>) => void): Promise<() => Promise<void>> {
    return async () => {};
  }
  
  async publish(_topic: string, _message: Record<string, unknown>): Promise<void> {}
}

class Peer {
  constructor(_options: { protocols: string[] }) {}
  
  async start(): Promise<void> {}
  async stop(): Promise<void> {}
}

import { 
  RIFFCC_PROTOCOL,
  TRUSTED_SITES_SITE_ID_COL,
  TRUSTED_SITES_NAME_COL,
  BLOCKED_RELEASES_RELEASE_ID_COLUMN,
  FEATURED_RELEASES_RELEASE_ID_COLUMN,
  FEATURED_RELEASES_START_TIME_COLUMN,
  FEATURED_RELEASES_END_TIME_COLUMN,
  FEATURED_PROMOTED_COLUMN,
} from './consts';

import type {
  Release,
  ReleaseWithId,
  Collection,
  CollectionWithId,
  FeaturedRelease,
  BlockedRelease,
  TrustedSite,
  ContentCategory,
  ContentCategoryWithId,
  ContentCategoryMetadataField,
} from './types';

type ReleasesCollection = DocumentCollection<Release>;
type CollectionsCollection = DocumentCollection<Collection>;
type TrustedSitesCollection = DocumentCollection<TrustedSite>;
type FeaturedReleasesCollection = DocumentCollection<FeaturedRelease>;
type BlockedReleasesCollection = DocumentCollection<BlockedRelease>;
type ContentCategoriesCollection = DocumentCollection<ContentCategory<ContentCategoryMetadataField>>;

interface PeerbitEvents {
  'site configured': (args: {
    siteId: string;
  }) => void;
}

type forgetFunction = () => Promise<void>;

import type { PeerbitInterface } from './peerbit.interface';

export class Peerbit implements PeerbitInterface {
  siteId: string;
  peer: Peer;
  pubsub: Pubsub;
  events: TypedEmitter<PeerbitEvents>;
  
  private accountId: string;
  private deviceId: string;
  private peerId: string;
  private profileData: Record<string, unknown>;
  private isModerator: boolean;
  private canUpload: boolean;
  
  private releasesCollection?: ReleasesCollection;
  private collectionsCollection?: CollectionsCollection;
  private trustedSitesCollection?: TrustedSitesCollection;
  private featuredReleasesCollection?: FeaturedReleasesCollection;
  private blockedReleasesCollection?: BlockedReleasesCollection;
  private contentCategoriesCollection?: ContentCategoriesCollection;
  
  private forgetFns: forgetFunction[] = [];
  
  constructor({
    siteId,
  }: {
    siteId: string;
  }) {
    this.events = new TypedEmitter<PeerbitEvents>();
    this.siteId = siteId;
    
    this.accountId = uuidv4();
    this.deviceId = uuidv4();
    this.peerId = uuidv4();
    this.profileData = {};
    this.isModerator = true; // Default to true for prototype
    this.canUpload = true; // Default to true for prototype
    
    this.peer = new Peer({
      protocols: [RIFFCC_PROTOCOL],
    });
    
    this.pubsub = new Pubsub({ peer: this.peer });
    
    this._init();
  }
  
  async _init() {
    await this.peer.start();
    
    await this.initializeCollections();
    
    await this.subscribeToUpdates();
  }
  
  async initializeCollections() {
    const options: CreateDocumentCollectionOptions = {
      peer: this.peer,
      indexes: [], // Add indexes as needed
    };
    
    this.releasesCollection = await Document.createDocumentCollection<Release>(options);
    this.collectionsCollection = await Document.createDocumentCollection<Collection>(options);
    this.trustedSitesCollection = await Document.createDocumentCollection<TrustedSite>(options);
    this.featuredReleasesCollection = await Document.createDocumentCollection<FeaturedRelease>(options);
    this.blockedReleasesCollection = await Document.createDocumentCollection<BlockedRelease>(options);
    this.contentCategoriesCollection = await Document.createDocumentCollection<ContentCategory<ContentCategoryMetadataField>>(options);
  }
  
  async subscribeToUpdates() {
    await this.pubsub.subscribe(`${this.siteId}.releases`, (_message: Record<string, unknown>) => {
    });
    
    await this.pubsub.subscribe(`${this.siteId}.collections`, (_message: Record<string, unknown>) => {
    });
    
    await this.pubsub.subscribe(`${this.siteId}.trusted_sites`, (_message: Record<string, unknown>) => {
    });
  }
  
  
  async addRelease(release: Release): Promise<void> {
    if (!this.releasesCollection) {
      throw new Error('Releases collection not initialized');
    }
    
    await this.releasesCollection.put(release);
    
    await this.pubsub.publish(`${this.siteId}.releases`, {
      type: 'add',
      data: release,
    });
  }
  
  async removeRelease(releaseId: string): Promise<void> {
    if (!this.releasesCollection) {
      throw new Error('Releases collection not initialized');
    }
    
    await this.releasesCollection.delete(releaseId);
    
    await this.pubsub.publish(`${this.siteId}.releases`, {
      type: 'remove',
      id: releaseId,
    });
  }
  
  async editRelease({
    release,
    releaseId,
  }: {
    release: Partial<Release>;
    releaseId: string;
  }): Promise<void> {
    if (!this.releasesCollection) {
      throw new Error('Releases collection not initialized');
    }
    
    const existingRelease = await this.releasesCollection.get(releaseId);
    if (!existingRelease) {
      throw new Error(`Release with ID ${releaseId} not found`);
    }
    
    const updatedRelease = { ...existingRelease.value, ...release };
    await this.releasesCollection.put(updatedRelease, releaseId);
    
    await this.pubsub.publish(`${this.siteId}.releases`, {
      type: 'update',
      id: releaseId,
      data: updatedRelease,
    });
  }
  
  async listenForReleases({
    f,
  }: {
    f: (releases: { release: ReleaseWithId; contributor: string; site: string }[]) => void;
  }): Promise<forgetFunction> {
    if (!this.releasesCollection) {
      throw new Error('Releases collection not initialized');
    }
    
    const releases = await this.releasesCollection.getAll();
    const mappedReleases = releases.map(doc => ({
      release: {
        id: doc.id,
        release: doc.value,
      },
      contributor: doc.creator || 'unknown',
      site: this.siteId,
    }));
    
    
    const allReleases = [...mappedReleases];
    
    f(allReleases);
    
    const unsubscribe = await this.pubsub.subscribe(`${this.siteId}.releases`, async (_message: Record<string, unknown>) => {
      const updatedReleases = await this.releasesCollection?.getAll() || [];
      const mappedUpdatedReleases = updatedReleases.map(doc => ({
        release: {
          id: doc.id,
          release: doc.value,
        },
        contributor: doc.creator || 'unknown',
        site: this.siteId,
      }));
      
      f([...mappedUpdatedReleases]);
    });
    
    this.forgetFns.push(unsubscribe);
    return unsubscribe;
  }
  
  async listenForSiteReleases({
    f,
    siteId,
    desiredNResults = 1000,
  }: {
    f: (releases: { release: ReleaseWithId; contributor: string }[]) => void;
    siteId?: string;
    desiredNResults?: number;
  }): Promise<forgetFunction> {
    const targetSiteId = siteId || this.siteId;
    
    if (targetSiteId === this.siteId) {
      if (!this.releasesCollection) {
        throw new Error('Releases collection not initialized');
      }
      
      const releases = await this.releasesCollection.getAll();
      const mappedReleases = releases.slice(0, desiredNResults).map(doc => ({
        release: {
          id: doc.id,
          release: doc.value,
        },
        contributor: doc.creator || 'unknown',
      }));
      
      f(mappedReleases);
      
      const unsubscribe = await this.pubsub.subscribe(`${this.siteId}.releases`, async (_message: Record<string, unknown>) => {
        const updatedReleases = await this.releasesCollection?.getAll() || [];
        const mappedUpdatedReleases = updatedReleases.slice(0, desiredNResults).map(doc => ({
          release: {
            id: doc.id,
            release: doc.value,
          },
          contributor: doc.creator || 'unknown',
        }));
        
        f(mappedUpdatedReleases);
      });
      
      this.forgetFns.push(unsubscribe);
      return unsubscribe;
    } else {
      f([]);
      
      const noop = async () => {};
      return noop;
    }
  }
  
  async addCollection(collection: Collection): Promise<void> {
    if (!this.collectionsCollection) {
      throw new Error('Collections collection not initialized');
    }
    
    await this.collectionsCollection.put(collection);
    
    await this.pubsub.publish(`${this.siteId}.collections`, {
      type: 'add',
      data: collection,
    });
  }
  
  async removeCollection(collectionId: string): Promise<void> {
    if (!this.collectionsCollection) {
      throw new Error('Collections collection not initialized');
    }
    
    await this.collectionsCollection.delete(collectionId);
    
    await this.pubsub.publish(`${this.siteId}.collections`, {
      type: 'remove',
      id: collectionId,
    });
  }
  
  async editCollection({
    collection,
    collectionId,
  }: {
    collection: Partial<Collection>;
    collectionId: string;
  }): Promise<void> {
    if (!this.collectionsCollection) {
      throw new Error('Collections collection not initialized');
    }
    
    const existingCollection = await this.collectionsCollection.get(collectionId);
    if (!existingCollection) {
      throw new Error(`Collection with ID ${collectionId} not found`);
    }
    
    const updatedCollection = { ...existingCollection.value, ...collection };
    await this.collectionsCollection.put(updatedCollection, collectionId);
    
    await this.pubsub.publish(`${this.siteId}.collections`, {
      type: 'update',
      id: collectionId,
      data: updatedCollection,
    });
  }
  
  async listenForCollections({
    f,
  }: {
    f: (collections: { collection: CollectionWithId; contributor: string; site: string }[]) => void;
  }): Promise<forgetFunction> {
    if (!this.collectionsCollection) {
      throw new Error('Collections collection not initialized');
    }
    
    const collections = await this.collectionsCollection.getAll();
    const mappedCollections = collections.map(doc => ({
      collection: {
        id: doc.id,
        collection: doc.value,
      },
      contributor: doc.creator || 'unknown',
      site: this.siteId,
    }));
    
    f(mappedCollections);
    
    const unsubscribe = await this.pubsub.subscribe(`${this.siteId}.collections`, async (_message: Record<string, unknown>) => {
      const updatedCollections = await this.collectionsCollection?.getAll() || [];
      const mappedUpdatedCollections = updatedCollections.map(doc => ({
        collection: {
          id: doc.id,
          collection: doc.value,
        },
        contributor: doc.creator || 'unknown',
        site: this.siteId,
      }));
      
      f(mappedUpdatedCollections);
    });
    
    this.forgetFns.push(unsubscribe);
    return unsubscribe;
  }
  
  getSiteId(): string {
    return this.siteId;
  }
  
  async trustSite({
    siteId,
    siteName,
  }: {
    siteName: string;
    siteId: string;
  }): Promise<string> {
    if (!this.trustedSitesCollection) {
      throw new Error('Trusted sites collection not initialized');
    }
    
    const trustedSite: TrustedSite = {
      [TRUSTED_SITES_SITE_ID_COL]: siteId,
      [TRUSTED_SITES_NAME_COL]: siteName,
    };
    
    const result = await this.trustedSitesCollection.put(trustedSite);
    
    await this.pubsub.publish(`${this.siteId}.trusted_sites`, {
      type: 'add',
      data: trustedSite,
    });
    
    return result.toString();
  }
  
  async untrustSite({ siteId }: { siteId: string }): Promise<void> {
    if (!this.trustedSitesCollection) {
      throw new Error('Trusted sites collection not initialized');
    }
    
    const trustedSites = await this.trustedSitesCollection.getAll();
    const siteEntry = trustedSites.find(site => site.value[TRUSTED_SITES_SITE_ID_COL] === siteId);
    
    if (siteEntry) {
      await this.trustedSitesCollection.delete(siteEntry.id);
      
      await this.pubsub.publish(`${this.siteId}.trusted_sites`, {
        type: 'remove',
        id: siteEntry.id,
      });
    }
  }
  
  async followTrustedSites({
    f,
  }: {
    f: (sites: { id: string; data: TrustedSite }[]) => void;
  }): Promise<forgetFunction> {
    if (!this.trustedSitesCollection) {
      throw new Error('Trusted sites collection not initialized');
    }
    
    const trustedSites = await this.trustedSitesCollection.getAll();
    const mappedSites = trustedSites.map(doc => ({
      id: doc.id,
      data: doc.value,
    }));
    
    f(mappedSites);
    
    const unsubscribe = await this.pubsub.subscribe(`${this.siteId}.trusted_sites`, async (_message: Record<string, unknown>) => {
      const updatedSites = await this.trustedSitesCollection?.getAll() || [];
      const mappedUpdatedSites = updatedSites.map(doc => ({
        id: doc.id,
        data: doc.value,
      }));
      
      f(mappedUpdatedSites);
    });
    
    this.forgetFns.push(unsubscribe);
    return unsubscribe;
  }
  
  async blockRelease({ cid }: { cid: string }): Promise<string> {
    if (!this.blockedReleasesCollection) {
      throw new Error('Blocked releases collection not initialized');
    }
    
    const blockedRelease: BlockedRelease = {
      [BLOCKED_RELEASES_RELEASE_ID_COLUMN]: cid,
    };
    
    const result = await this.blockedReleasesCollection.put(blockedRelease);
    
    await this.pubsub.publish(`${this.siteId}.blocked_releases`, {
      type: 'add',
      data: blockedRelease,
    });
    
    return result.toString();
  }
  
  async unblockRelease({ id }: { id: string }): Promise<void> {
    if (!this.blockedReleasesCollection) {
      throw new Error('Blocked releases collection not initialized');
    }
    
    await this.blockedReleasesCollection.delete(id);
    
    await this.pubsub.publish(`${this.siteId}.blocked_releases`, {
      type: 'remove',
      id,
    });
  }
  
  async followBlockedReleases({
    f,
  }: {
    f: (blockedReleases: { id: string; data: BlockedRelease }[]) => void;
  }): Promise<forgetFunction> {
    if (!this.blockedReleasesCollection) {
      throw new Error('Blocked releases collection not initialized');
    }
    
    const blockedReleases = await this.blockedReleasesCollection.getAll();
    const mappedReleases = blockedReleases.map(doc => ({
      id: doc.id,
      data: doc.value,
    }));
    
    f(mappedReleases);
    
    const unsubscribe = await this.pubsub.subscribe(`${this.siteId}.blocked_releases`, async (_message: Record<string, unknown>) => {
      const updatedReleases = await this.blockedReleasesCollection?.getAll() || [];
      const mappedUpdatedReleases = updatedReleases.map(doc => ({
        id: doc.id,
        data: doc.value,
      }));
      
      f(mappedUpdatedReleases);
    });
    
    this.forgetFns.push(unsubscribe);
    return unsubscribe;
  }
  
  async featureRelease({
    cid,
    releaseId,
    siteId: _siteId,
    startTime,
    endTime,
    promoted,
  }: {
    cid?: string;
    releaseId?: string;
    siteId?: string;
    startTime?: string;
    endTime?: string;
    promoted?: boolean;
  }): Promise<string> {
    if (!this.featuredReleasesCollection) {
      throw new Error('Featured releases collection not initialized');
    }
    
    const actualCid = cid || releaseId || '';
    const actualStartTime = startTime || new Date().toISOString();
    const actualEndTime = endTime || new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString();
    const actualPromoted = promoted !== undefined ? promoted : false;
    
    const featuredRelease: FeaturedRelease = {
      [FEATURED_RELEASES_RELEASE_ID_COLUMN]: actualCid,
      [FEATURED_RELEASES_START_TIME_COLUMN]: actualStartTime,
      [FEATURED_RELEASES_END_TIME_COLUMN]: actualEndTime,
      [FEATURED_PROMOTED_COLUMN]: actualPromoted,
    };
    
    const result = await this.featuredReleasesCollection.put(featuredRelease);
    
    await this.pubsub.publish(`${this.siteId}.featured_releases`, {
      type: 'add',
      data: featuredRelease,
    });
    
    return result.toString();
  }
  
  async addCategory(category: ContentCategory<ContentCategoryMetadataField>): Promise<string> {
    if (!this.contentCategoriesCollection) {
      throw new Error('Content categories collection not initialized');
    }
    
    const result = await this.contentCategoriesCollection.put(category);
    
    await this.pubsub.publish(`${this.siteId}.content_categories`, {
      type: 'add',
      data: category,
    });
    
    return result.toString();
  }
  
  async listenForContentCategories({
    f,
  }: {
    f: (categories: ContentCategoryWithId<ContentCategoryMetadataField>[]) => void;
  }): Promise<forgetFunction> {
    if (!this.contentCategoriesCollection) {
      throw new Error('Content categories collection not initialized');
    }
    
    const categories = await this.contentCategoriesCollection.getAll();
    const mappedCategories = categories.map(doc => ({
      id: doc.id,
      contentCategory: doc.value,
    }));
    
    f(mappedCategories);
    
    const unsubscribe = await this.pubsub.subscribe(`${this.siteId}.content_categories`, async (_message: Record<string, unknown>) => {
      const updatedCategories = await this.contentCategoriesCollection?.getAll() || [];
      const mappedUpdatedCategories = updatedCategories.map(doc => ({
        id: doc.id,
        contentCategory: doc.value,
      }));
      
      f(mappedUpdatedCategories);
    });
    
    this.forgetFns.push(unsubscribe);
    return unsubscribe;
  }
  
  async close() {
    for (const forget of this.forgetFns) {
      await forget();
    }
    
    await this.peer.stop();
  }

  async getAccountId(): Promise<string> {
    return this.accountId;
  }

  async getDeviceId(): Promise<string> {
    return this.deviceId;
  }

  async getPeerId(): Promise<string> {
    return this.peerId;
  }

  async listenForAccountId(f: (accountId: string) => void): Promise<forgetFunction> {
    f(this.accountId);
    return async () => {};
  }

  async listenForNameChange(f: (names: Record<string, string>) => void, _options?: { accountId?: string }): Promise<forgetFunction> {
    const names = this.profileData.names as Record<string, string> || {};
    f(names);
    return async () => {};
  }

  async changeName({ name, language }: { name: string; language: string }): Promise<void> {
    if (!this.profileData.names) {
      this.profileData.names = {};
    }
    (this.profileData.names as Record<string, string>)[language] = name;
  }

  async initializeProfile(): Promise<void> {
    this.profileData = {
      names: {},
      photo: null,
    };
  }

  async listenForProfilePhotoChange(f: (photo: string | null) => void, _options?: { accountId?: string }): Promise<forgetFunction> {
    f(this.profileData.photo as string | null);
    return async () => {};
  }

  async followIsModerator(f: (isModerator: boolean) => void): Promise<forgetFunction> {
    f(this.isModerator);
    return async () => {};
  }

  async followCanUpload(f: (canUpload: boolean) => void): Promise<forgetFunction> {
    f(this.canUpload);
    return async () => {};
  }

  async inviteModerator(accountId: string): Promise<void> {
    console.log(`Invited ${accountId} to be a moderator`);
  }

  async getNetworkConnections(): Promise<Record<string, unknown>[]> {
    return [];
  }

  async getConnectedDevices(): Promise<Record<string, unknown>[]> {
    return [];
  }

  async getConnectedAccounts(): Promise<Record<string, unknown>[]> {
    return [];
  }

  async getIPFSFile(_options: { id: string }): Promise<{ buffer: Buffer; size: number }> {
    return {
      buffer: Buffer.from(new ArrayBuffer(0)),
      size: 0,
    };
  }

  async editCategory(categoryId: string, category: ContentCategory<ContentCategoryMetadataField>): Promise<void> {
    if (!this.contentCategoriesCollection) {
      throw new Error('Content categories collection not initialized');
    }
    
    await this.contentCategoriesCollection.put(category, categoryId);
    
    await this.pubsub.publish(`${this.siteId}.content_categories`, {
      type: 'update',
      id: categoryId,
      data: category,
    });
  }

  async removeCategory(categoryId: string): Promise<void> {
    if (!this.contentCategoriesCollection) {
      throw new Error('Content categories collection not initialized');
    }
    
    await this.contentCategoriesCollection.delete(categoryId);
    
    await this.pubsub.publish(`${this.siteId}.content_categories`, {
      type: 'remove',
      id: categoryId,
    });
  }

  async editFeaturedRelease({
    id,
    releaseId,
    startTime,
    endTime,
    promoted,
  }: {
    id: string;
    releaseId: string;
    startTime: string;
    endTime: string;
    promoted: boolean;
  }): Promise<void> {
    if (!this.featuredReleasesCollection) {
      throw new Error('Featured releases collection not initialized');
    }
    
    const featuredRelease: FeaturedRelease = {
      [FEATURED_RELEASES_RELEASE_ID_COLUMN]: releaseId,
      [FEATURED_RELEASES_START_TIME_COLUMN]: startTime,
      [FEATURED_RELEASES_END_TIME_COLUMN]: endTime,
      [FEATURED_PROMOTED_COLUMN]: promoted,
    };
    
    await this.featuredReleasesCollection.put(featuredRelease, id);
    
    await this.pubsub.publish(`${this.siteId}.featured_releases`, {
      type: 'update',
      id,
      data: featuredRelease,
    });
  }

  async listenForSiteFeaturedReleases({
    f,
    siteId,
  }: {
    f: (releases: { id: string; data: FeaturedRelease }[]) => void;
    siteId?: string;
  }): Promise<forgetFunction> {
    const targetSiteId = siteId || this.siteId;
    
    if (targetSiteId === this.siteId && this.featuredReleasesCollection) {
      const featuredReleases = await this.featuredReleasesCollection.getAll();
      const mappedReleases = featuredReleases.map(doc => ({
        id: doc.id,
        data: doc.value,
      }));
      
      f(mappedReleases);
      
      const unsubscribe = await this.pubsub.subscribe(`${this.siteId}.featured_releases`, async (_message: Record<string, unknown>) => {
        const updatedReleases = await this.featuredReleasesCollection?.getAll() || [];
        const mappedUpdatedReleases = updatedReleases.map(doc => ({
          id: doc.id,
          data: doc.value,
        }));
        
        f(mappedUpdatedReleases);
      });
      
      this.forgetFns.push(unsubscribe);
      return unsubscribe;
    } else {
      f([]);
      
      const noop = async () => {};
      return noop;
    }
  }
}

export const createPeerbit = async ({
  siteId,
}: {
  siteId: string;
}) => {
  const peerbit = new Peerbit({ siteId });
  return { peerbit };
};
