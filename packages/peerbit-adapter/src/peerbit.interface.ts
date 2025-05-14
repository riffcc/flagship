import type { 
  Release, 
  ReleaseWithId, 
  Collection, 
  CollectionWithId, 
  ContentCategory, 
  ContentCategoryWithId, 
  ContentCategoryMetadataField,
  TrustedSite,
  FeaturedRelease,
  BlockedRelease,
} from './types';

type ForgetFunction = () => Promise<void>;

export interface PeerbitInterface {
  siteId: string;
  
  close(): Promise<void>;
  getSiteId(): string;
  getAccountId(): Promise<string>;
  getDeviceId(): Promise<string>;
  getPeerId(): Promise<string>;
  
  // Release management
  addRelease(release: Release): Promise<void>;
  removeRelease(releaseId: string): Promise<void>;
  editRelease(params: { releaseId: string; release: Release }): Promise<void>;
  listenForReleases(options: { f: (releases: { release: ReleaseWithId; contributor: string; site: string }[]) => void }): Promise<ForgetFunction>;
  listenForSiteReleases(options: { 
    f: (releases: { release: ReleaseWithId; contributor: string }[]) => void,
    siteId?: string,
    desiredNResults?: number
  }): Promise<ForgetFunction>;
  
  // Collection management
  addCollection(collection: Collection): Promise<void>;
  removeCollection(collectionId: string): Promise<void>;
  editCollection(params: { collectionId: string; collection: Collection }): Promise<void>;
  listenForCollections(options: { f: (collections: { collection: CollectionWithId; contributor: string; site: string }[]) => void }): Promise<ForgetFunction>;
  
  // Site management
  trustSite(options: { siteName: string; siteId: string }): Promise<string>;
  untrustSite(options: { siteId: string }): Promise<void>;
  followTrustedSites(options: { f: (sites: { id: string; data: TrustedSite }[]) => void }): Promise<ForgetFunction>;
  
  blockRelease(options: { cid: string }): Promise<string>;
  unblockRelease(options: { id: string }): Promise<void>;
  followBlockedReleases(options: { f: (blockedReleases: { id: string; data: BlockedRelease }[]) => void }): Promise<ForgetFunction>;
  
  featureRelease(options: { 
    cid?: string, 
    releaseId?: string, 
    siteId?: string,
    startTime?: string,
    endTime?: string,
    promoted?: boolean
  }): Promise<string>;
  editFeaturedRelease(options: { 
    id: string, 
    cid?: string,
    releaseId?: string, 
    siteId?: string,
    startTime?: string,
    endTime?: string,
    promoted?: boolean
  }): Promise<void>;
  listenForSiteFeaturedReleases(options: { 
    f: (featuredReleases: { id: string; data: FeaturedRelease }[]) => void,
    siteId: string
  }): Promise<ForgetFunction>;
  
  // Category management
  addCategory(category: ContentCategory<ContentCategoryMetadataField>): Promise<string>;
  editCategory(categoryId: string, category: ContentCategory<ContentCategoryMetadataField>): Promise<void>;
  removeCategory(categoryId: string): Promise<void>;
  listenForContentCategories(options: { f: (categories: ContentCategoryWithId<ContentCategoryMetadataField>[]) => void }): Promise<ForgetFunction>;
  
  // Profile management
  changeName(options: { name: string; language: string }): Promise<void>;
  initializeProfile(): Promise<void>;
  listenForNameChange(f: (names: Record<string, string>) => void, _options?: { accountId?: string }): Promise<ForgetFunction>;
  listenForProfilePhotoChange(f: (photo: string | null) => void, _options?: { accountId?: string }): Promise<ForgetFunction>;
  listenForAccountId(f: (accountId: string) => void): Promise<ForgetFunction>;
  
  // Access management
  followIsModerator(f: (isModerator: boolean) => void): Promise<ForgetFunction>;
  followCanUpload(f: (canUpload: boolean) => void): Promise<ForgetFunction>;
  inviteModerator(accountId: string): Promise<void>;
  
  // Network management
  getNetworkConnections(): Promise<Record<string, unknown>[]>;
  getConnectedDevices(): Promise<Record<string, unknown>[]>;
  getConnectedAccounts(): Promise<Record<string, unknown>[]>;
  
  // File management
  getIPFSFile(options: { id: string }): Promise<{ buffer: Buffer; size: number }>;
}
