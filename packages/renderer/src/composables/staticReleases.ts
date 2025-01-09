import {ref, type Ref} from 'vue';

export type ItemStatus = 'pending' | 'approved' | 'rejected' | 'deleted';

export interface ItemContent {
  id: string;
  category: string;
  contentCID: string;
  name: string;
  description?: string;
  thumbnail?: string;
  sourceSite: string;
  status: ItemStatus;
  metadata?: {
    author?: string;
    duration?: string;
    releaseYear?: number | string;
  };
}

export interface FeaturedItem extends ItemContent {
  classification: string;
  cover: string;
  rating: number;
  startTime: number;
  endTime: number;
}

const staticFeaturedReleases: Ref<Array<FeaturedItem>> = ref([
  {
    id: '1',
    classification: 'PG',
    category: 'video',
    contentCID: '/test-video.mp4',
    cover: '/mock/movie-rip.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    metadata: {
      releaseYear: '2008',
      duration: '1h 26m',
    },
    rating: 4.5,
    name: 'RiP!: A Remix Manifesto',
    description:
      'Join filmmaker Brett Gaylor and mashup artist Girl Talk as they explore copyright and content creation in the digital age. In the process they dissect the media landscape of the 21st century and shatter the wall between users and producers.',
    status: 'approved',
    startTime: 0,
    endTime: 1,
  },
  {
    id: '2',
    classification: 'PG',
    category: 'video',
    contentCID: 'QmPjxGcAYBv1fbwWSA2Zu4rHFN21DpFTKpQivXk9Kozqqe',
    cover: '/mock/movie-aaronsw.jpg',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    metadata: {
      duration: '1h 45m',
      releaseYear: '2014',
    },
    rating: 4,
    name: "The Internet's Own Boy: The Story of Aaron Swartz",
    description:
      "The Internet's Own Boy follows the story of programming prodigy and information activist Aaron Swartz. [Audio currently needs fixing, which will be resolved soon.]",
    status: 'approved',
    startTime: 0,
    endTime: 1,
  },
  {
    id: '3',
    classification: 'Unrated',
    category: 'video',
    contentCID: 'QmPSGARS6emPSEf8umwmjdG8AS7z7o8Nd36258B3BMi291',
    cover: '/mock/featured-tpbafk-fanart.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    metadata: {
      duration: '1h 26m',
      releaseYear: '2012',
    },
    rating: 4.5,
    name: 'TPB AFK: The Pirate Bay Away From Keyboard',
    description:
      'The Pirate Bay Away From Keyboard is a documentary film about the file sharing website The Pirate Bay. [Audio currently needs fixing, which will be resolved soon.]',
    status: 'approved',
    startTime: 0,
    endTime: 1,
  },
  {
    id: '4',
    classification: 'PG',
    category: 'music',
    contentCID: 'QmU6WhM6h3uvnicXcCQPgYpwrg9Moz68nVGWBeaYca2bMv',
    cover: '/mock/music-mapleridge.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    metadata: {
      author: 'Swear and Shake',
      duration: '1h 26m',
      releaseYear: '2015',
    },
    rating: 4.5,
    name: 'Maple Ridge',
    description:
      'One of our favourite folk albums, and an early inspiration for the Riff.CC project.',
    status: 'approved',
    startTime: 0,
    endTime: 1,
  },
]);

const staticReleases: Ref<Array<ItemContent>> = ref([
  {
    id: '1',
    category: 'tvShow',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'Pure Pwnage',
    description: '1 Season^',
    thumbnail: '/mock/tvshow-purepwnage.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '2',
    category: 'tvShow',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'Pioneer One',
    description: '2 Seasons',
    thumbnail: '/mock/tvshow-pioneerone.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '3',
    category: 'tvShow',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'Flash Gordon',
    description: '1 Seasons',
    thumbnail: '/mock/tvshow-flashgordon.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '4',
    category: 'tvShow',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'The Beverley Hillbillies',
    description: '~1.6 Seasons^',
    thumbnail: '/mock/tvshow-beverleyhillbillies.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '5',
    category: 'video',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    name: 'RiP!: A Remix Manifesto',
    metadata: {
      releaseYear: '(2008)',
    },
    thumbnail: '/mock/movie-rip-poster.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '6',
    category: 'music',
    name: "Let's Kick Fire",
    metadata: {
      author: 'Adam McHeffey',
    },
    contentCID: 'QmQ5mZFnruyqA4tzwguKJ9e4wLigokE2pQE3e99u3YK8vg',
    thumbnail: '/mock/music-letskickfire.jpg',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '7',
    category: 'music',
    name: 'Maple Ridge',
    contentCID: 'QmU6WhM6h3uvnicXcCQPgYpwrg9Moz68nVGWBeaYca2bMv',
    metadata: {
      author: 'Swear and Shake',
    },
    thumbnail: '/mock/music-mapleridge.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '8',
    category: 'music',
    name: 'The Slip',
    contentCID: 'QmR9hcaUqC7saAj8jjpkCwqa9bChmMJ3Mca17sRn6oiR2F',
    metadata: {
      author: 'Nine Inch Nails',
    },
    thumbnail: '/mock/music-theslip.jpg',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '9',
    category: 'music',
    name: 'IN RAINBOWS',
    contentCID: 'QmWnDNcn7WCcuemYzRTBdJRMzSMgR8Hf6xtPiWLShtqucv',
    metadata: {
      author: 'Radiohead',
    },
    thumbnail: '/mock/music-inrainbows.jpg',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '10',
    category: 'video',
    contentCID: 'QmPjxGcAYBv1fbwWSA2Zu4rHFN21DpFTKpQivXk9Kozqqe',
    name: "The Internet's Own Boy: The Story of Aaron Swartz",
    metadata: {
      releaseYear: '(2014)',
    },
    thumbnail: '/mock/movie-aaronsw.jpg',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '11',
    category: 'video',
    contentCID: 'QmPSGARS6emPSEf8umwmjdG8AS7z7o8Nd36258B3BMi291',
    name: 'TPB AFK: The Pirate Bay Away from Keyboard',
    metadata: {
      releaseYear: '(2012)',
    },
    thumbnail: '/mock/movie-tbpafk.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '12',
    category: 'video',
    contentCID: 'QmYVCbux1BK5Z2eJjwr5pJayZiQhp2TAUdCNUostkFkwee',
    name: 'Cosmos Laundromat: First Cycle',
    metadata: {
      releaseYear: '(2015)',
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
    category: 'video',
    contentCID: 'QmWeLCFA27vv91r6Jxu1C4PZTXj4mXpam6GGQzM6cS8FYD',
    name: 'Night of the Living Dead',
    metadata: {
      releaseYear: '(1968)',
    },
    thumbnail: '/mock/movie-nightofthelivingdead.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '15',
    category: 'music',
    metadata: {
      author: 'Girl Talk',
    },
    name: 'All Day',
    contentCID: 'QmZAYJ1eQtTgMCcM2xxXiLjERqnbaseX1wMvuRbddqhaMj',
    description: 'All Day',
    thumbnail: '/mock/music-allday.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '16',
    category: 'music',
    metadata: {
      author: 'paniq',
    },
    name: 'Story of Ohm^',
    contentCID: 'QmcvUHaHp7bpnvs31Nka7rSQ2KEuWcgDSwK3V1wsRqMqns',
    thumbnail: '/mock/music-storyofohm.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '17',
    category: 'music',
    metadata: {
      author: 'OK! Crazy Fiction Lady',
    },
    name: 'Bye Bye Fishies',
    contentCID: 'QmZE5FLsfNDLvXpruXFehoGL3H1EUpbRpszcoFvSXx1iKd',
    description: 'Bye Bye Fishies',
    thumbnail: '/mock/music-byebyefishies.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '18',
    category: 'music',
    name: 'Everything You Should Know',
    thumbnail: '/mock/music-everythingyoushouldknow.jpg',
    contentCID: 'QmbQ6JUzJPXaMdh5HBBZsczGLzhgz5DaFmUbRFZByZggRq',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
    metadata: {
      author: 'Silence is Sexy',
      releaseYear: 2006,
    },
  },
  {
    id: '19',
    category: 'music',
    metadata: {
      author: 'OK! Crazy Fiction Lady',
    },
    name: 'OK! Crazy Fiction Lady',
    contentCID: 'QmUD6WSCQcyyBGdwEqUiQBivU8QXR8psx5eiuqv3BqK76M',
    description: 'OK! Crazy Fiction Lady',
    thumbnail: '/mock/music-okcfl.png',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '20',
    category: 'music',
    metadata: {
      author: 'paniq',
    },
    name: 'Beyond Good and Evil',
    contentCID: 'QmSPWyFztzp3wntTyBLR5P3xc35wYekaUZ9YyzHtYRu7Ky',
    thumbnail: '/mock/music-paniq-bgae.jpg',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '21',
    category: 'music',
    metadata: {
      author: 'Brad Sucks',
    },
    name: "Guess Who's A Mess",
    contentCID: 'QmNXPf83zcKpqp3nDFtjYuAcTWLqsLZkANbNmcH3YZSs34',
    thumbnail: '/mock/music-guesswhosamess.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
  {
    id: '22',
    category: 'music',
    metadata: {
      author: 'Swear and Shake',
    },
    name: 'Extended Play^',
    contentCID: 'Qme72tWtGJfQnUnWoadTb3PxkfQGAYziiAjf4hvqraokF9',
    thumbnail: '/mock/music-swearandshake-extendedplay.webp',
    sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
    status: 'approved',
  },
]);

export const useStaticReleases = () => {
  return {
    staticFeaturedReleases,
    staticReleases,
  };
};
