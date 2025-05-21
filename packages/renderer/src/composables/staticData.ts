import { ref, type Ref } from 'vue';
import {
  ID_PROPERTY,
  RELEASE_NAME_PROPERTY,
  RELEASE_CATEGORY_ID_PROPERTY,
  RELEASE_CONTENT_CID_PROPERTY,
  RELEASE_THUMBNAIL_CID_PROPERTY,
  RELEASE_METADATA_PROPERTY,
  FEATURED_RELEASE_ID_PROPERTY,
  FEATURED_START_TIME_PROPERTY,
  FEATURED_END_TIME_PROPERTY,
  FEATURED_PROMOTED_PROPERTY,
  CONTENT_CATEGORY_DISPLAY_NAME_PROPERTY,
  CONTENT_CATEGORY_FEATURED_PROPERTY,
  CONTENT_CATEGORY_METADATA_SCHEMA_PROPERTY,
} from '@riffcc/lens-sdk';
import type { ContentCategoryData, ContentCategoryMetadata } from '@riffcc/lens-sdk';
import type { ReleaseItem, FeaturedReleaseItem } from '/@/types';


const staticFeaturedReleases: Ref<FeaturedReleaseItem[]> = ref([
  {
    [ID_PROPERTY]: '1',
    [FEATURED_RELEASE_ID_PROPERTY]: '5',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: true,
  },
  {
    [ID_PROPERTY]: '2',
    [FEATURED_RELEASE_ID_PROPERTY]: '10',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: true,
  },
  {
    [ID_PROPERTY]: '3',
    [FEATURED_RELEASE_ID_PROPERTY]: '11',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: true,
  },
  {
    [ID_PROPERTY]: '4',
    [FEATURED_RELEASE_ID_PROPERTY]: '7',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: true,
  },
  {
    [ID_PROPERTY]: '5',
    [FEATURED_RELEASE_ID_PROPERTY]: '1',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '6',
    [FEATURED_RELEASE_ID_PROPERTY]: '2',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '7',
    [FEATURED_RELEASE_ID_PROPERTY]: '3',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '8',
    [FEATURED_RELEASE_ID_PROPERTY]: '4',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '9',
    [FEATURED_RELEASE_ID_PROPERTY]: '6',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '10',
    [FEATURED_RELEASE_ID_PROPERTY]: '8',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '11',
    [FEATURED_RELEASE_ID_PROPERTY]: '9',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '12',
    [FEATURED_RELEASE_ID_PROPERTY]: '12',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '13',
    [FEATURED_RELEASE_ID_PROPERTY]: '13',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '14',
    [FEATURED_RELEASE_ID_PROPERTY]: '14',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '15',
    [FEATURED_RELEASE_ID_PROPERTY]: '15',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '16',
    [FEATURED_RELEASE_ID_PROPERTY]: '16',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '17',
    [FEATURED_RELEASE_ID_PROPERTY]: '17',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '18',
    [FEATURED_RELEASE_ID_PROPERTY]: '18',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '19',
    [FEATURED_RELEASE_ID_PROPERTY]: '19',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '20',
    [FEATURED_RELEASE_ID_PROPERTY]: '20',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '21',
    [FEATURED_RELEASE_ID_PROPERTY]: '21',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
  {
    [ID_PROPERTY]: '22',
    [FEATURED_RELEASE_ID_PROPERTY]: '22',
    [FEATURED_START_TIME_PROPERTY]: '2025-01-01T00:00',
    [FEATURED_END_TIME_PROPERTY]: '2026-01-01T00:00',
    [FEATURED_PROMOTED_PROPERTY]: false,
  },
]);

const staticReleases: Ref<ReleaseItem<{[key: string]: unknown}>[]> = ref([
  {
    [ID_PROPERTY]: '1',
    [RELEASE_NAME_PROPERTY]: 'Pure Pwnage',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'tvShow',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/tvshow-purepwnage.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Geoff Lapaire',
      seasons: 1,
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLz',
    },
  },
  {
    [ID_PROPERTY]: '2',
    [RELEASE_NAME_PROPERTY]: 'Pioneer One',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'tvShow',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/tvshow-pioneerone.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Josh Bernhard and Bracey Smith',
      seasons: 2,
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwLUAABqczyLNFziLyAmujzG2A',
    },
  },
  {
    [ID_PROPERTY]: '3',
    [RELEASE_NAME_PROPERTY]: 'Flash Gordon',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'tvShow',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/tvshow-flashgordon.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Unknown',
      seasons: 1,
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '4',
    [RELEASE_NAME_PROPERTY]: 'The Beverley Hillbillies',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'tvShow',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/tvshow-beverleyhillbillies.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Paul Henning',
      seasons: '~1.6',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },

  },
  {
    [ID_PROPERTY]: '5',
    [RELEASE_NAME_PROPERTY]: 'RiP!: A Remix Manifesto',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'movie',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/movie-rip-poster.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Brett Gaylor',
      cover: '/mock/movie-rip.png',
      classification: 'PG',
      description:
        'Join filmmaker Brett Gaylor and mashup artist Girl Talk as they explore copyright and content creation in the digital age. In the process they dissect the media landscape of the 21st century and shatter the wall between users and producers.',
      duration: '1h 26m',
      releaseYear: '2008',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXUAABqczyLNFziLwjs4WMzwAmujzG2AL',
    },
  },
  {
    [ID_PROPERTY]: '6',
    [RELEASE_NAME_PROPERTY]: "Let's Kick Fire",
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmQ5mZFnruyqA4tzwguKJ9e4wLigokE2pQE3e99u3YK8vg',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-letskickfire.jpg',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Adam McHeffey',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmAmujzG2ALUAABqczyLNFziLwGURrXjs4WMzw',
    },
  },
  {
    [ID_PROPERTY]: '7',
    [RELEASE_NAME_PROPERTY]: 'Maple Ridge',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmU6WhM6h3uvnicXcCQPgYpwrg9Moz68nVGWBeaYca2bMv',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-mapleridge.webp',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Swear and Shake',
      cover: '/mock/music-mapleridge.webp',
      description:
        'One of our favourite folk albums, and an early inspiration for the Riff.CC project.',
      songs: 10,
      releaseYear: '2012',
      sourceSite: '/orbitdb/zdpuAwQJUzwAmujzG2ALUAABqczyLNFziLwpaVmGURrXjs4WM',
    },
  },
  {
    [ID_PROPERTY]: '8',
    [RELEASE_NAME_PROPERTY]: 'The Slip',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmR9hcaUqC7saAj8jjpkCwqa9bChmMJ3Mca17sRn6oiR2F',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-theslip.jpg',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Nine Inch Nails',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },

  },
  {
    [ID_PROPERTY]: '9',
    [RELEASE_NAME_PROPERTY]: 'IN RAINBOWS',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmWnDNcn7WCcuemYzRTBdJRMzSMgR8Hf6xtPiWLShtqucv',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-inrainbows.jpg',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Radiohead',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '10',
    [RELEASE_NAME_PROPERTY]: "The Internet's Own Boy: The Story of Aaron Swartz",
    [RELEASE_CATEGORY_ID_PROPERTY]: 'movie',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmPjxGcAYBv1fbwWSA2Zu4rHFN21DpFTKpQivXk9Kozqqe',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: 'bafkreiemmzezvfmeueaeuqfewtf4d6fiuvjqnh4xulgwuugfknmh3abfxi',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Brian Knappenberger',
      cover: 'bafkreiemmzezvfmeueaeuqfewtf4d6fiuvjqnh4xulgwuugfknmh3abfxi',
      classification: 'PG',
      description:
        "The Internet's Own Boy follows the story of programming prodigy and information activist Aaron Swartz.",
      duration: '1h 45m',
      releaseYear: '2014',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '11',
    [RELEASE_NAME_PROPERTY]: 'TPB AFK: The Pirate Bay Away from Keyboard',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'movie',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmPSGARS6emPSEf8umwmjdG8AS7z7o8Nd36258B3BMi291',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: 'bafkreiemqveqhpksefhup46d77iybtatf2vb2bgyak4hfydxaz5hxser34',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Simon Klose',
      cover: 'bafkreiemqveqhpksefhup46d77iybtatf2vb2bgyak4hfydxaz5hxser34',
      classification: 'Unrated',
      description:
        'The Pirate Bay Away From Keyboard is a documentary film about the file sharing website The Pirate Bay.',
      duration: '1h 26m',
      releaseYear: '2013',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '12',
    [RELEASE_NAME_PROPERTY]: 'Cosmos Laundromat: First Cycle',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'movie',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmYVCbux1BK5Z2eJjwr5pJayZiQhp2TAUdCNUostkFkwee',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/movie-cosmoslaundromat.webp',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Unknown',
      releaseYear: '2015',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '13',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'Qmb9XpBQnw1vataDeWTh4jAnPMgNfKGyV7KWFz7uCvYHNd',
    [RELEASE_NAME_PROPERTY]: 'Ghosts I-IV',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-ghosts-i-iv.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Nine Inch Nails',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '14',
    [RELEASE_NAME_PROPERTY]: 'Night of the Living Dead',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'movie',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmWeLCFA27vv91r6Jxu1C4PZTXj4mXpam6GGQzM6cS8FYD',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/movie-nightofthelivingdead.webp',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'George A. Romero',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '15',
    [RELEASE_NAME_PROPERTY]: 'All Day',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmZAYJ1eQtTgMCcM2xxXiLjERqnbaseX1wMvuRbddqhaMj',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-allday.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Girl Talk',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '16',
    [RELEASE_NAME_PROPERTY]: 'Story of Ohm^',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmcvUHaHp7bpnvs31Nka7rSQ2KEuWcgDSwK3V1wsRqMqns',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-storyofohm.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'paniq',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '17',
    [RELEASE_NAME_PROPERTY]: 'Bye Bye Fishies',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmZE5FLsfNDLvXpruXFehoGL3H1EUpbRpszcoFvSXx1iKd',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-byebyefishies.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'OK! Crazy Fiction Lady',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '18',
    [RELEASE_NAME_PROPERTY]: 'Everything You Should Know',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmbQ6JUzJPXaMdh5HBBZsczGLzhgz5DaFmUbRFZByZggRq',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-everythingyoushouldknow.jpg',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Silence is Sexy',
      releaseYear: 2006,
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '19',
    [RELEASE_NAME_PROPERTY]: 'OK! Crazy Fiction Lady',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmUD6WSCQcyyBGdwEqUiQBivU8QXR8psx5eiuqv3BqK76M',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-okcfl.png',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'OK! Crazy Fiction Lady',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '20',
    [RELEASE_NAME_PROPERTY]: 'Beyond Good and Evil',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmSPWyFztzp3wntTyBLR5P3xc35wYekaUZ9YyzHtYRu7Ky',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-paniq-bgae.jpg',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'paniq',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '21',
    [RELEASE_NAME_PROPERTY]: "Guess Who's A Mess",
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'QmNXPf83zcKpqp3nDFtjYuAcTWLqsLZkANbNmcH3YZSs34',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-guesswhosamess.webp',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Brad Sucks',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
  {
    [ID_PROPERTY]: '22',
    [RELEASE_NAME_PROPERTY]: 'Extended Play^',
    [RELEASE_CATEGORY_ID_PROPERTY]: 'music',
    [RELEASE_CONTENT_CID_PROPERTY]: 'Qme72tWtGJfQnUnWoadTb3PxkfQGAYziiAjf4hvqraokF9',
    [RELEASE_THUMBNAIL_CID_PROPERTY]: '/mock/music-swearandshake-extendedplay.webp',
    [RELEASE_METADATA_PROPERTY]: {
      author: 'Swear and Shake',
      sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    },
  },
]);

const staticContentCategories: ContentCategoryData<ContentCategoryMetadata>[] = [
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
    staticFeaturedReleases,
    staticReleases,
    staticContentCategories,
  };
};
