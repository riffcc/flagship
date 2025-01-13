import {ref, type Ref} from 'vue';

export type ItemStatus = 'pending' | 'approved' | 'rejected' | 'deleted';

export interface ItemContent {
  id: string;
  category: string;
  contentCID: string;
  cover?: string;
  name: string;
  metadata?: {
    author?: string;
    classification?: string;
    description?: string;
    duration?: string;
    rating?: {
      value: number;
      source: string;
    };
    releaseYear?: number | string;
    seasons?: number | string;
    songs?: number;
  };
  sourceSite: string;
  status: ItemStatus;
  thumbnail?: string;
}

export interface FeaturedItem {
  id: string;
  releaseId: string;
  startTime: string;
  endTime: string;
}

const staticFeaturedReleases: Ref<Array<FeaturedItem>> = ref([
  {
    id: '1',
    releaseId: '5',
    startTime: '2025-01-01T00:00',
    endTime: '2026-01-01T00:00',
  },
  {
    id: '2',
    releaseId: '10',
    startTime: '2025-01-01T00:00',
    endTime: '2026-01-01T00:00',
  },
  {
    id: '3',
    releaseId: '11',
    startTime: '2025-01-01T00:00',
    endTime: '2026-01-01T00:00',
  },
  {
    id: '4',
    releaseId: '7',
    startTime: '2025-01-01T00:00',
    endTime: '2026-01-01T00:00',
  },
]);

const staticReleases: Ref<Array<ItemContent>> = ref([
  {
    id: '1',
    category: 'tvShow',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'Pure Pwnage',
    metadata: {
      seasons: 1,
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/tvshow-purepwnage.png',
  },
  {
    id: '2',
    category: 'tvShow',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'Pioneer One',
    metadata: {
      seasons: 2,
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/tvshow-pioneerone.png',
  },
  {
    id: '3',
    category: 'tvShow',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'Flash Gordon',
    metadata: {
      seasons: 1,
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/tvshow-flashgordon.png',
  },
  {
    id: '4',
    category: 'tvShow',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'The Beverley Hillbillies',
    metadata: {
      seasons: '~1.6',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/tvshow-beverleyhillbillies.png',
  },
  {
    id: '5',
    category: 'movie',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    cover: '/mock/movie-rip.png',
    name: 'RiP!: A Remix Manifesto',
    metadata: {
      classification: 'PG',
      description:
      'Join filmmaker Brett Gaylor and mashup artist Girl Talk as they explore copyright and content creation in the digital age. In the process they dissect the media landscape of the 21st century and shatter the wall between users and producers.',
      duration: '1h 26m',
      rating: {
        value: 4,
        source: 'IMDb',
      },
      releaseYear: '2008',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/movie-rip-poster.png',
  },
  {
    id: '6',
    category: 'music',
    name: "Let's Kick Fire",
    metadata: {
      author: 'Adam McHeffey',
    },
    contentCID: 'QmQ5mZFnruyqA4tzwguKJ9e4wLigokE2pQE3e99u3YK8vg',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-letskickfire.jpg',
  },
  {
    id: '7',
    category: 'music',
    contentCID: 'QmU6WhM6h3uvnicXcCQPgYpwrg9Moz68nVGWBeaYca2bMv',
    cover: '/mock/music-mapleridge.webp',
    name: 'Maple Ridge',
    metadata: {
      author: 'Swear and Shake',
      description:
      'One of our favourite folk albums, and an early inspiration for the Riff.CC project.',
      songs: 10,
      releaseYear: '2012',
    },
    status: 'approved',
    thumbnail: '/mock/music-mapleridge.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
  },
  {
    id: '8',
    category: 'music',
    name: 'The Slip',
    contentCID: 'QmR9hcaUqC7saAj8jjpkCwqa9bChmMJ3Mca17sRn6oiR2F',
    metadata: {
      author: 'Nine Inch Nails',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-theslip.jpg',
  },
  {
    id: '9',
    category: 'music',
    name: 'IN RAINBOWS',
    contentCID: 'QmWnDNcn7WCcuemYzRTBdJRMzSMgR8Hf6xtPiWLShtqucv',
    metadata: {
      author: 'Radiohead',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-inrainbows.jpg',
  },
  {
    id: '10',
    category: 'movie',
    contentCID: 'QmPjxGcAYBv1fbwWSA2Zu4rHFN21DpFTKpQivXk9Kozqqe',
    cover: '/mock/movie-aaronsw.jpg',
    name: "The Internet's Own Boy: The Story of Aaron Swartz",
    metadata: {
      classification: 'PG',
      description:
      "The Internet's Own Boy follows the story of programming prodigy and information activist Aaron Swartz.",
      duration: '1h 45m',
      rating: {
        value: 4,
        source: 'IMDb',
      },
      releaseYear: '2014',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/movie-aaronsw.jpg',
  },
  {
    id: '11',
    category: 'movie',
    contentCID: 'QmPSGARS6emPSEf8umwmjdG8AS7z7o8Nd36258B3BMi291',
    cover: '/mock/featured-tpbafk-fanart.png',
    name: 'TPB AFK: The Pirate Bay Away from Keyboard',
    metadata: {
      classification: 'Unrated',
      description:
      'The Pirate Bay Away From Keyboard is a documentary film about the file sharing website The Pirate Bay.',
      duration: '1h 26m',
      rating: {
        value: 3.7,
        source: 'IMDb',
      },
      releaseYear: '2013',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/movie-tbpafk.webp',
  },
  {
    id: '12',
    category: 'movie',
    contentCID: 'QmYVCbux1BK5Z2eJjwr5pJayZiQhp2TAUdCNUostkFkwee',
    name: 'Cosmos Laundromat: First Cycle',
    metadata: {
      releaseYear: '2015',
    },
    thumbnail: '/mock/movie-cosmoslaundromat.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '13',
    category: 'music',
    contentCID: 'Qmb9XpBQnw1vataDeWTh4jAnPMgNfKGyV7KWFz7uCvYHNd',
    name: 'Ghosts I-IV',
    metadata: {
      author: 'Nine Inch Nails',
    },
    thumbnail: '/mock/music-ghosts-i-iv.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '14',
    category: 'movie',
    contentCID: 'QmWeLCFA27vv91r6Jxu1C4PZTXj4mXpam6GGQzM6cS8FYD',
    name: 'Night of the Living Dead',
    metadata: {
      releaseYear: '1968',
    },
    thumbnail: '/mock/movie-nightofthelivingdead.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '15',
    category: 'music',
    contentCID: 'QmZAYJ1eQtTgMCcM2xxXiLjERqnbaseX1wMvuRbddqhaMj',
    name: 'All Day',
    metadata: {
      author: 'Girl Talk',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-allday.png',
  },
  {
    id: '16',
    category: 'music',
    contentCID: 'QmcvUHaHp7bpnvs31Nka7rSQ2KEuWcgDSwK3V1wsRqMqns',
    name: 'Story of Ohm^',
    metadata: {
      author: 'paniq',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-storyofohm.png',
  },
  {
    id: '17',
    category: 'music',
    contentCID: 'QmZE5FLsfNDLvXpruXFehoGL3H1EUpbRpszcoFvSXx1iKd',
    name: 'Bye Bye Fishies',
    metadata: {
      author: 'OK! Crazy Fiction Lady',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-byebyefishies.png',
  },
  {
    id: '18',
    category: 'music',
    contentCID: 'QmbQ6JUzJPXaMdh5HBBZsczGLzhgz5DaFmUbRFZByZggRq',
    name: 'Everything You Should Know',
    metadata: {
      author: 'Silence is Sexy',
      releaseYear: 2006,
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-everythingyoushouldknow.jpg',
  },
  {
    id: '19',
    category: 'music',
    contentCID: 'QmUD6WSCQcyyBGdwEqUiQBivU8QXR8psx5eiuqv3BqK76M',
    name: 'OK! Crazy Fiction Lady',
    metadata: {
      author: 'OK! Crazy Fiction Lady',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-okcfl.png',
  },
  {
    id: '20',
    category: 'music',
    contentCID: 'QmSPWyFztzp3wntTyBLR5P3xc35wYekaUZ9YyzHtYRu7Ky',
    name: 'Beyond Good and Evil',
    metadata: {
      author: 'paniq',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-paniq-bgae.jpg',
  },
  {
    id: '21',
    category: 'music',
    contentCID: 'QmNXPf83zcKpqp3nDFtjYuAcTWLqsLZkANbNmcH3YZSs34',
    name: "Guess Who's A Mess",
    metadata: {
      author: 'Brad Sucks',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-guesswhosamess.webp',
  },
  {
    id: '22',
    category: 'music',
    contentCID: 'Qme72tWtGJfQnUnWoadTb3PxkfQGAYziiAjf4hvqraokF9',
    name: 'Extended Play^',
    metadata: {
      author: 'Swear and Shake',
    },
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    thumbnail: '/mock/music-swearandshake-extendedplay.webp',
  },
]);

export const useStaticReleases = () => {
  return {
    staticFeaturedReleases,
    staticReleases,
  };
};
