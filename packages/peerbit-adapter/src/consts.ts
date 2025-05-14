import type { ContentCategory, ContentCategoryMetadataField } from './types';

export const TRUSTED_SITES_TABLE_KEY = 'trustedSites';
export const TRUSTED_SITES_SITE_ID_COL = 'siteId';
export const TRUSTED_SITES_NAME_COL = 'siteName';

export const FEATURED_RELEASES_TABLE_KEY = 'featuredReleases';
export const FEATURED_RELEASES_RELEASE_ID_COLUMN = 'releaseId';
export const FEATURED_RELEASES_START_TIME_COLUMN = 'startTime';
export const FEATURED_RELEASES_END_TIME_COLUMN = 'endTime';
export const FEATURED_PROMOTED_COLUMN = 'promoted';

export const BLOCKED_RELEASES_TABLE_KEY = 'blockedReleases';
export const BLOCKED_RELEASES_RELEASE_ID_COLUMN = 'releaseId';

export const CONTENT_CATEGORIES_TABLE_KEY = 'contentCategories';
export const CONTENT_CATEGORIES_CATEGORY_ID = 'categoryId';
export const CONTENT_CATEGORIES_DISPLAY_NAME = 'displayName';
export const CONTENT_CATEGORIES_FEATURED = 'featured';
export const CONTENT_CATEGORIES_METADATA_SCHEMA = 'metadataSchema';

export const RELEASES_FILE_COLUMN = 'file';
export const RELEASES_AUTHOR_COLUMN = 'author';
export const RELEASES_NAME_COLUMN = 'contentName';
export const RELEASES_METADATA_COLUMN = 'metadata';
export const RELEASES_THUMBNAIL_COLUMN = 'thumbnail';
export const RELEASES_CATEGORY_COLUMN = 'category';
export const RELEASES_COVER_COLUMN = 'cover';

export const COLLECTIONS_RELEASES_COLUMN = 'releases';
export const COLLECTIONS_AUTHOR_COLUMN = 'author';
export const COLLECTIONS_NAME_COLUMN = 'contentName';
export const COLLECTIONS_METADATA_COLUMN = 'metadata';
export const COLLECTIONS_THUMBNAIL_COLUMN = 'thumbnail';
export const COLLECTIONS_CATEGORY_COLUMN = 'category';

export const RELEASES_DB_TABLE_KEY = 'releases';
export const COLLECTIONS_DB_TABLE_KEY = 'collections';

export const CONFIG_FILE_NAME = '.peerbit-config.json';
export const DEFAULT_PEERBIT_DIR = '.peerbit';

export const RIFFCC_PROTOCOL = '/riffcc/1.0.0';

export const DEFAULT_CONTENT_CATEGORIES: ContentCategory<ContentCategoryMetadataField>[] = [
  {
    categoryId: 'music',
    displayName: 'Music',
    featured: true,
    metadataSchema: {
      artist: {
        type: 'string',
        description: 'Artist name',
      },
      album: {
        type: 'string',
        description: 'Album name',
      },
      genre: {
        type: 'string',
        description: 'Music genre',
      },
      year: {
        type: 'number',
        description: 'Release year',
      },
    },
  },
  {
    categoryId: 'video',
    displayName: 'Video',
    metadataSchema: {
      director: {
        type: 'string',
        description: 'Director name',
      },
      duration: {
        type: 'number',
        description: 'Duration in minutes',
      },
      genre: {
        type: 'string',
        description: 'Video genre',
      },
    },
  },
  {
    categoryId: 'image',
    displayName: 'Image',
    metadataSchema: {
      photographer: {
        type: 'string',
        description: 'Photographer name',
      },
      resolution: {
        type: 'string',
        description: 'Image resolution',
      },
    },
  },
  {
    categoryId: 'document',
    displayName: 'Document',
    metadataSchema: {
      author: {
        type: 'string',
        description: 'Document author',
      },
      pages: {
        type: 'number',
        description: 'Number of pages',
      },
    },
  },
  {
    categoryId: 'software',
    displayName: 'Software',
    metadataSchema: {
      developer: {
        type: 'string',
        description: 'Software developer',
      },
      version: {
        type: 'string',
        description: 'Software version',
      },
      platform: {
        type: 'string',
        description: 'Supported platforms',
      },
    },
  },
];
