import type {
  COLLECTIONS_AUTHOR_COLUMN,
  COLLECTIONS_METADATA_COLUMN,
  COLLECTIONS_NAME_COLUMN,
  COLLECTIONS_RELEASES_COLUMN,
  COLLECTIONS_THUMBNAIL_COLUMN,
  COLLECTIONS_CATEGORY_COLUMN,
  COLLECTIONS_STATUS_COLUMN,
  RELEASES_AUTHOR_COLUMN,
  RELEASES_FILE_COLUMN,
  RELEASES_METADATA_COLUMN,
  RELEASES_NAME_COLUMN,
  RELEASES_THUMBNAIL_COLUMN,
  RELEASES_STATUS_COLUMN,
  RELEASES_CATEGORY_COLUMN,
  TRUSTED_SITES_NAME_COL,
  TRUSTED_SITES_SITE_ID_COL,
  FEATURED_RELEASES_END_TIME_COLUMN,
  FEATURED_RELEASES_RELEASE_ID_COLUMN,
  FEATURED_RELEASES_START_TIME_COLUMN,
  BLOCKED_RELEASES_RELEASE_ID_COLUMN,
} from './consts';

export const variableIdKeys = [
  'trustedSitesSiteIdVariableId',
  'trustedSitesNameVariableId',
  'releasesContentNameVar',
  'releasesFileVar',
  'releasesThumbnailVar',
  'releasesAuthorVar',
  'releasesMetadataVar',
  'releasesCategoryVar',
  'releasesStatusVar',
  'collectionsNameVar',
  'collectionsAuthorVar',
  'collectionsThumbnailVar',
  'collectionsMetadataVar',
  'collectionsCategoryVar',
  'collectionsReleasesVar',
  'collectionsStatusVar',
  'featuredReleasesReleaseIdVar',
  'featuredReleasesStartTimeVar',
  'featuredReleasesEndTimeVar',
  'blockedReleasesReleaseIdVar',
] as const;

export type VariableIds = Record<(typeof variableIdKeys)[number], string>;

export type possiblyIncompleteVariableIds = Partial<VariableIds>;

export type Release = {
  [RELEASES_NAME_COLUMN]: string;
  [RELEASES_FILE_COLUMN]: string;
  [RELEASES_AUTHOR_COLUMN]: string;
  [RELEASES_THUMBNAIL_COLUMN]?: string;
  [RELEASES_METADATA_COLUMN]?: string;
  [RELEASES_CATEGORY_COLUMN]: string;
  [RELEASES_STATUS_COLUMN]: string;
};

export type ReleaseWithId = {
  release: Release;
  id: string;
};

export type Collection = {
  [COLLECTIONS_NAME_COLUMN]: string;
  [COLLECTIONS_AUTHOR_COLUMN]?: string;
  [COLLECTIONS_THUMBNAIL_COLUMN]?: string;
  [COLLECTIONS_METADATA_COLUMN]?: string;
  [COLLECTIONS_CATEGORY_COLUMN]: string;
  [COLLECTIONS_RELEASES_COLUMN]: string;
  [COLLECTIONS_STATUS_COLUMN]: string;
};

export type FeaturedRelease = {
  [FEATURED_RELEASES_RELEASE_ID_COLUMN]: string;
  [FEATURED_RELEASES_START_TIME_COLUMN]: string;
  [FEATURED_RELEASES_END_TIME_COLUMN]: string;
};

export type BlockedRelease = {
  [BLOCKED_RELEASES_RELEASE_ID_COLUMN]: string;
}

export type CollectionWithId = {
  collection: Collection;
  id: string;
};

export type TrustedSite = {
  [TRUSTED_SITES_SITE_ID_COL]: string;
  [TRUSTED_SITES_NAME_COL]: string;
};


export interface ReleaseMetadata {
  description?: string;
  license?: string;
}

export interface MusicReleaseMetadata {
  tags?: string;
  musicBrainzID?: string;
  albumTitle?: string;
  initialReleaseYear?: string;
  releaseType?: string;
  fileFormat?: string;
  bitrate?: string;
  mediaFormat?: string;
}

export interface MovieReleaseMetadata {
  posterCID?: string;
  TMDBID?: string;
  IMDBID?: string;
  releaseType?: string;
}
