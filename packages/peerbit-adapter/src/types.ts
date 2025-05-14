import type {
  BLOCKED_RELEASES_RELEASE_ID_COLUMN,
  COLLECTIONS_AUTHOR_COLUMN,
  COLLECTIONS_CATEGORY_COLUMN,
  COLLECTIONS_METADATA_COLUMN,
  COLLECTIONS_NAME_COLUMN,
  COLLECTIONS_RELEASES_COLUMN,
  COLLECTIONS_THUMBNAIL_COLUMN,
  CONTENT_CATEGORIES_CATEGORY_ID,
  CONTENT_CATEGORIES_DISPLAY_NAME,
  CONTENT_CATEGORIES_FEATURED,
  CONTENT_CATEGORIES_METADATA_SCHEMA,
  FEATURED_PROMOTED_COLUMN,
  FEATURED_RELEASES_END_TIME_COLUMN,
  FEATURED_RELEASES_RELEASE_ID_COLUMN,
  FEATURED_RELEASES_START_TIME_COLUMN,
  RELEASES_AUTHOR_COLUMN,
  RELEASES_CATEGORY_COLUMN,
  RELEASES_COVER_COLUMN,
  RELEASES_FILE_COLUMN,
  RELEASES_METADATA_COLUMN,
  RELEASES_NAME_COLUMN,
  RELEASES_THUMBNAIL_COLUMN,
  TRUSTED_SITES_NAME_COL,
  TRUSTED_SITES_SITE_ID_COL,
} from './consts';

export type Release<T = string> = {
  [RELEASES_NAME_COLUMN]: string;
  [RELEASES_FILE_COLUMN]: string;
  [RELEASES_AUTHOR_COLUMN]: string;
  [RELEASES_CATEGORY_COLUMN]: string;
  [RELEASES_THUMBNAIL_COLUMN]?: string;
  [RELEASES_COVER_COLUMN]?: string;
  [RELEASES_METADATA_COLUMN]?: T;
};

export type ReleaseWithId<T = string> = {
  id: string;
  release: Release<T>;
};

export type Collection = {
  [COLLECTIONS_NAME_COLUMN]: string;
  [COLLECTIONS_AUTHOR_COLUMN]?: string;
  [COLLECTIONS_THUMBNAIL_COLUMN]?: string;
  [COLLECTIONS_METADATA_COLUMN]?: string;
  [COLLECTIONS_CATEGORY_COLUMN]: string;
  [COLLECTIONS_RELEASES_COLUMN]: string;
};

export type CollectionWithId = {
  collection: Collection;
  id: string;
};

export type FeaturedRelease = {
  [FEATURED_RELEASES_RELEASE_ID_COLUMN]: string;
  [FEATURED_RELEASES_START_TIME_COLUMN]: string;
  [FEATURED_RELEASES_END_TIME_COLUMN]: string;
  [FEATURED_PROMOTED_COLUMN]: boolean;
};

export type BlockedRelease = {
  [BLOCKED_RELEASES_RELEASE_ID_COLUMN]: string;
};

export type TrustedSite = {
  [TRUSTED_SITES_SITE_ID_COL]: string;
  [TRUSTED_SITES_NAME_COL]: string;
};

export type ContentCategoryMetadataField = Record<
  string,
  {
    type: 'string' | 'number' | 'array';
    description: string;
    options?: string[];
  }
>;

export type ContentCategory<T = string> = {
  [CONTENT_CATEGORIES_CATEGORY_ID]: string;
  [CONTENT_CATEGORIES_DISPLAY_NAME]: string;
  [CONTENT_CATEGORIES_FEATURED]?: boolean;
  [CONTENT_CATEGORIES_METADATA_SCHEMA]: T;
};

export type ContentCategoryWithId<T = string> = {
  id: string;
  contentCategory: ContentCategory<T>;
};

export type ConfigMode = 'vite' | 'json';

export type PeerbitConfig = {
  siteId: string;
};

export type PossiblyIncompletePeerbitConfig = Partial<PeerbitConfig>;
