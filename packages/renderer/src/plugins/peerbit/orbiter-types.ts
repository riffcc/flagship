// Mock of the orbiter types and constants
// This replaces @riffcc/orbiter imports for Peerbit compatibility

export namespace types {
  export interface ReleaseWithId<T = string> {
    release: {
      id?: string;
      release: {
        contentName: string;
        file: string;
        category: string;
        author: string;
        thumbnail?: string;
        cover?: string;
        metadata?: string;
      };
    };
    site?: string;
  }

  export interface FeaturedReleaseWithId {
    id: string;
    featured: {
      releaseId: string;
      startTime: string;
      endTime: string;
      promoted: boolean;
    };
  }

  export interface ContentCategoryWithId {
    id: string;
    category: {
      name: string;
      description?: string;
      icon?: string;
    };
  }

  export interface ContentMetadataWithId {
    id: string;
    metadata: {
      name: string;
      type: string;
      options?: string[];
      required: boolean;
    };
  }

  export interface SiteInfo {
    name: string;
    description: string;
    logo?: string;
    banner?: string;
    url?: string;
    theme?: {
      primary?: string;
      secondary?: string;
      accent?: string;
    };
  }
}

export const consts = {
  METADATA_FIELD_TYPES: ['text', 'number', 'date', 'select', 'boolean', 'url'],
  MAX_FILE_SIZE: 1000000000, // 1GB
  MAX_NAME_LENGTH: 100,
  MAX_DESCRIPTION_LENGTH: 1000
}; 