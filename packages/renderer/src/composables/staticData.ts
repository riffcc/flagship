import {
  ID_PROPERTY,
  CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY,
  CONTENT_CATEGORY_FEATURED_PROPERTY,
  CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY,
} from '@riffcc/lens-sdk';
import type { ContentCategoryData, ContentCategoryMetadata } from '@riffcc/lens-sdk';

const staticContentCategories: Omit<ContentCategoryData<ContentCategoryMetadata>, 'siteAddress'>[] = [
  {
    [ID_PROPERTY]: 'music',
    [CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY]: 'Music',
    [CONTENT_CATEGORY_FEATURED_PROPERTY]: true,
    [CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY]: {
      author: {
        type: 'string',
        description: 'Name of the author or creator of the music content',
      },
      cover: {
        type: 'string',
        description: 'URL or path to the cover image of the music content',
      },
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
    [ID_PROPERTY]: 'video',
    [CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY]: 'Videos',
    [CONTENT_CATEGORY_FEATURED_PROPERTY]: false,
    [CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY]: {
      author: {
        type: 'string',
        description: 'Name of the author or creator of the video content',
      },
      cover: {
        type: 'string',
        description: 'URL or path to the cover image of the video content',
      },
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
    [ID_PROPERTY]: 'movie',
    [CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY]: 'Movies',
    [CONTENT_CATEGORY_FEATURED_PROPERTY]: true,
    [CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY]: {
      description: {
        type: 'string',
        description: 'Brief description of the movie',
      },
      author: {
        type: 'string',
        description: 'Name of the author or creator of the video content',
      },
      cover: {
        type: 'string',
        description: 'URL or path to the cover image of the video content',
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
    [ID_PROPERTY]: 'tvShow',
    [CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY]: 'TV Shows',
    [CONTENT_CATEGORY_FEATURED_PROPERTY]: true,
    [CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY]: {
      description: {
        type: 'string',
        description: 'Brief description of the TV show',
      },
      author: {
        type: 'string',
        description: 'Name of the author or creator of the tv show content',
      },
      cover: {
        type: 'string',
        description: 'URL or path to the cover image of the tv show content',
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

export const useStaticData = () => {
  return {
    staticContentCategories,
  };
};
