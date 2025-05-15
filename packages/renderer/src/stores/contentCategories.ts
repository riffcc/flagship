// Removed: import { ContentCategoryWithId } from './contentCategories';
import { defineStore } from 'pinia';
import { computed } from 'vue';

export const CONTENT_CATEGORIES_CATEGORY_ID = 'categoryId';
export const CONTENT_CATEGORIES_DISPLAY_NAME = 'displayName';
export const CONTENT_CATEGORIES_FEATURED = 'featured';
export const CONTENT_CATEGORIES_METADATA_SCHEMA = 'metadataSchema';

export type ContentCategory<T = string> = {
  [CONTENT_CATEGORIES_CATEGORY_ID]: string;
  [CONTENT_CATEGORIES_DISPLAY_NAME]: string;
  [CONTENT_CATEGORIES_FEATURED]?: boolean;
  [CONTENT_CATEGORIES_METADATA_SCHEMA]: T;
};

export type ContentCategoryMetadataField = Record<string, {
  type: 'string' | 'number' | 'array';
  description: string;
  options?: string[];
}>;

export type ContentCategoryWithId<T = string> = {
  id: string;
  contentCategory: ContentCategory<T>;
};

export const DEFAULT_CONTENT_CATEGORIES: ContentCategory<ContentCategoryMetadataField>[] = [
  {
    categoryId: 'music',
    displayName: 'Music',
    featured: true,
    metadataSchema: {
      description: {
        type: 'string',
        description: 'Brief description of the music content',
      },
      totalSongs: {
        type: 'number',
        description: 'Total number of songs in this category',
      },
      totalDuration: {
        type: 'string',
        description: 'Total duration of all songs (e.g., in HH:MM:SS format)',
      },
      genres: {
        type: 'array',
        description: 'List of genres represented in this category',
      },
      tags: {
        type: 'string',
        description: 'Tags associated with the music release',
      },
      musicBrainzID: {
        type: 'string',
        description: 'MusicBrainz identifier for the release',
      },
      albumTitle: {
        type: 'string',
        description: 'Title of the album',
      },
      releaseYear: {
        type: 'number',
        description: 'Year of release',
      },
      releaseType: {
        type: 'string',
        description: 'Type of music release',
        options: [
          'Album',
          'Soundtrack',
          'EP',
          'Anthology',
          'Compilation',
          'Single',
          'Live Album',
          'Remix',
          'Bootleg',
          'Interview',
          'Mixtape',
          'Demo',
          'Concert Recording',
          'DJ Mix',
          'Unknown',
        ],
      },
      fileFormat: {
        type: 'string',
        description: 'Audio file format',
        options: ['MP3', 'FLAC', 'AAC', 'AC3', 'DTS'],
      },
      bitrate: {
        type: 'string',
        description: 'Audio bitrate (e.g., 320kbps)',
      },
      mediaFormat: {
        type: 'string',
        description: 'Physical media format if applicable',
        options: ['CD', 'DVD', 'Vinyl', 'Soundboard', 'SACD', 'DAT', 'WEB', 'Blu-Ray'],
      },
    },
  },
  {
    categoryId: 'video',
    displayName: 'Videos',
    metadataSchema: {
      title: {
        type: 'string',
        description: 'Title of the video',
      },
      description: {
        type: 'string',
        description: 'Brief description of the video content',
      },
      duration: {
        type: 'string',
        description: 'Length of the video (e.g., HH:MM:SS)',
      },
      resolution: {
        type: 'string',
        description: 'Video resolution (e.g., 1920x1080)',
      },
      format: {
        type: 'string',
        description: 'File format of the video (e.g., mp4, mov)',
      },
      tags: {
        type: 'array',
        description: 'User-defined tags for searchability (e.g., tutorial, vlog, funny)',
      },
      uploader: {
        type: 'string',
        description: 'Name or ID of the uploader/creator',
      },
      uploadDate: {
        type: 'string',
        description: 'Date the video was uploaded (e.g., YYYY-MM-DD)',
      },
      sourceUrl: {
        type: 'string',
        description: 'Original URL if sourced from an online platform (e.g., YouTube link)',
      },
    },
  },
  {
    categoryId: 'movie',
    displayName: 'Movies',
    featured: true,
    metadataSchema: {
      description: {
        type: 'string',
        description: 'Brief description of the movie',
      },
      resolution: {
        type: 'string',
        description: 'Video resolution (e.g., 1920x1080)',
      },
      format: {
        type: 'string',
        description: 'File format of the video (e.g., mp4, mov)',
      },
      genres: {
        type: 'array',
        description: 'Genres associated with the video (e.g., action, drama)',
      },
      tags: {
        type: 'array',
        description: 'User-defined tags for searchability (e.g., funny, tutorial)',
      },
      posterCID: {
        type: 'string',
        description: 'Content ID for the movie poster',
      },
      TMDBID: {
        type: 'string',
        description: 'The Movie Database identifier',
      },
      IMDBID: {
        type: 'string',
        description: 'Internet Movie Database identifier',
      },
      releaseType: {
        type: 'string',
        description: 'Type of movie release',
      },
      releaseYear: {
        type: 'number',
        description: 'Year of release',
      },
      classification: {
        type: 'string',
        description: 'Content rating/classification (e.g., PG-13)',
      },
      duration: {
        type: 'string',
        description: 'Length of the movie',
      },
    },
  },
  {
    categoryId: 'tvShow',
    displayName: 'TV Shows',
    featured: true,
    metadataSchema: {
      description: {
        type: 'string',
        description: 'Brief description of the TV show',
      },
      seasons: {
        type: 'number',
        description: 'Number of seasons in the TV show',
      },
      totalEpisodes: {
        type: 'number',
        description: 'Total number of episodes aired across all seasons',
      },
      genres: {
        type: 'array',
        description: 'Genres associated with the TV show (e.g., comedy, sci-fi)',
      },
      firstAiredYear: {
        type: 'number',
        description: 'Year the TV show first aired',
      },
      status: {
        type: 'string',
        description: 'Current status of the TV show',
        options: ['Returning Series', 'Ended', 'Canceled', 'In Production', 'Pilot', 'Unknown'],
      },
      TMDBID: {
        type: 'string',
        description: 'The Movie Database identifier for the TV show',
      },
      IMDBID: {
        type: 'string',
        description: 'Internet Movie Database identifier for the TV show',
      },
      posterCID: {
        type: 'string',
        description: 'Content ID for the TV show poster',
      },
      classification: {
        type: 'string',
        description: 'Content rating/classification (e.g., TV-MA, TV-14)',
      },
      network: {
        type: 'string',
        description: 'Original television network or streaming service',
      },
      averageEpisodeDuration: {
        type: 'string',
        description: 'Average duration of an episode (e.g., ~45 min, 00:45:00)',
      },
    },
  },
];

export const useContentCategoriesStore = defineStore('contentCategories', () => {

  const contentCategories = computed<ContentCategoryWithId<ContentCategoryMetadataField>[]>(() => { // Applied fix here
    return DEFAULT_CONTENT_CATEGORIES.map((dcc) => ({
      id: dcc.categoryId,
      contentCategory: dcc,
    }));
  });

  const featuredContentCategories = computed(() => {
    // Type of 'cc' will now be correctly inferred as ContentCategoryWithId<ContentCategoryMetadataField>
    return contentCategories.value.filter(cc => cc.contentCategory.featured);
  });

  return {
    contentCategories,
    featuredContentCategories,
  };
});
