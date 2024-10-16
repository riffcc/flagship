<template>
  <v-container class="fill-height">
    <featured-slider :featured-list="featuredReleases" />
    <content-section
      title="Featured"
      :items="staticData['featured-various']"
      layout="list"
      description="Various featured content of mixed media"
      :show-view-all="true"
    />
    <content-section
      title="Featured Music"
      :items="staticData['featured-music']"
      layout="grid"
      :show-view-all="true"
    />
    <v-alert
      type="info"
      class="mt-4"
      color="black"
      text-color="white"
    >
      Riff.CC: We're still adding UI support for TV shows, but below you can see what TV will look like on this platform.
    </v-alert>
    <content-section
      title="TV Shows"
      :items="staticData['tv-popular-shows']"
      layout="card"
      :show-navigation="true"
    />
  </v-container>
</template>


<script setup lang="ts">
import FeaturedSlider from '/@/components/home/featuredSlider.vue';
import ContentSection from '/@/components/home/contentSection.vue';
// import {suivre as follow} from '@constl/vue';
// import {useOrbiter} from '/@/plugins/orbiter/utils';


export interface ItemContent {
  id: string;
  category?: 'audio' | 'video';
  contentCID?: string;
  title: string;
  subtitle?: string;
  thumbnail?: string;
  metadata?: {
    author?: string;
    description?: string;
    duration?: string;
    releaseYear?: number | string;
  }
}

export interface FeaturedItem extends ItemContent {
  classification: string;
  cover: string;
  rating: number;
}

const featuredReleases: Array<FeaturedItem> = [
  {
    id: '1',
    classification: 'PG',
    category: 'video',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    cover: '/mock/movie-rip.png',
    metadata: {
      description: 'Join filmmaker Brett Gaylor and mashup artist Girl Talk as they explore copyright and content creation in the digital age. In the process they dissect the media landscape of the 21st century and shatter the wall between users and producers.',
      duration: '1h 26m',
      releaseYear: '2008',
    },
    title: 'RiP!: A Remix Manifesto',
    rating: 4.5,
  },
  {
    id: '2',
    classification: 'PG',
    category: 'video',
    contentCID: 'Qmcw6w7pE4J5cxD9ygsp7kgnUyxMYc2UHxsnTj6sTxUuX3',
    cover: '/mock/movie-aaronsw.jpg',
    metadata: {
      description: 'The Internet\'s Own Boy follows the story of programming prodigy and information activist Aaron Swartz. [Audio currently needs fixing, which will be resolved soon.]',
      duration: '1h 45m',
      releaseYear: '2014',
    },
    title: 'The Internet\'s Own Boy: The Story of Aaron Swartz',
    rating: 4,
  },
  {
    id: '3',
    classification: 'Unrated',
    category: 'video',
    contentCID: 'QmPSGARS6emPSEf8umwmjdG8AS7z7o8Nd36258B3BMi291',
    cover: '/mock/featured-tpbafk-fanart.png',
    metadata: {
      description: 'The Pirate Bay Away From Keyboard is a documentary film about the file sharing website The Pirate Bay. [Audio currently needs fixing, which will be resolved soon.]',
      duration: '1h 26m',
      releaseYear: '2012',
    },
    title: 'TPB AFK: The Pirate Bay Away From Keyboard',
    rating: 4.5,
  },
  {
    id: '4',
    classification: 'PG',
    category: 'audio',
    contentCID: 'QmU6WhM6h3uvnicXcCQPgYpwrg9Moz68nVGWBeaYca2bMv',
    cover: '/mock/music-mapleridge.webp',
    metadata: {
      author: 'Swear and Shake',
      description: 'One of our favourite folk albums, and an early inspiration for the Riff.CC project.',
      duration: '1h 26m',
      releaseYear: '2015',
    },
    title: 'Maple Ridge',
    rating: 4.5,
  },
];
const staticData: {[key: string]: Array<ItemContent>} = {
  'tv-popular-shows': [
    {
      id: '1',
      category: 'video',
      contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      title: 'Pure Pwnage',
      subtitle: '1 Season^',
      thumbnail: '/mock/tvshow-purepwnage.png',
    },
    {
      id: '2',
      category: 'video',
      contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      title: 'Pioneer One',
      subtitle: '2 Seasons',
      thumbnail: '/mock/tvshow-pioneerone.png',
    },
    {
      id: '3',
      category: 'video',
      contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      title: 'Flash Gordon',
      subtitle: '1 Seasons',
      thumbnail: '/mock/tvshow-flashgordon.png',
    },
    {
      id: '4',
      category: 'video',
      contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      title: 'The Beverley Hillbillies',
      subtitle: '~1.6 Seasons^',
      thumbnail: '/mock/tvshow-beverleyhillbillies.png',
    },
  ],
  'featured-various': [
    {
      id: '1',
      category: 'video',
      contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      title: 'RiP!: A Remix Manifesto',
      metadata: {
        releaseYear: '(2008)',
      },
      thumbnail: '/mock/movie-rip-poster.png',
    },
    {
      id: '2',
      category: 'audio',
      title: "Let's Kick Fire",
      metadata: {
        author: 'Adam McHeffey',
      },
      contentCID: 'QmQ5mZFnruyqA4tzwguKJ9e4wLigokE2pQE3e99u3YK8vg',
      thumbnail: '/mock/music-letskickfire.jpg',
    },
    {
      id: '3',
      category: 'audio',
      title: 'Maple Ridge',
      contentCID: 'QmU6WhM6h3uvnicXcCQPgYpwrg9Moz68nVGWBeaYca2bMv',
      metadata: {
        author: 'Swear and Shake',
      },
      thumbnail: '/mock/music-mapleridge.webp',
    },
    {
      id: '4',
      category: 'audio',
      title: 'The Slip',
      contentCID: 'QmR9hcaUqC7saAj8jjpkCwqa9bChmMJ3Mca17sRn6oiR2F',
      metadata: {
        author: 'Nine Inch Nails',
      },
      thumbnail: '/mock/music-theslip.jpg',
    },
    {
      id: '5',
      category: 'audio',
      title: 'IN RAINBOWS',
      contentCID: 'QmWnDNcn7WCcuemYzRTBdJRMzSMgR8Hf6xtPiWLShtqucv',
      metadata: {
        author: 'Radiohead',
      },
      thumbnail: '/mock/music-inrainbows.jpg',
    },
    {
      id: '6',
      category: 'video',
      contentCID: 'Qmcw6w7pE4J5cxD9ygsp7kgnUyxMYc2UHxsnTj6sTxUuX3 ',
      title: 'The Internet\'s Own Boy: The Story of Aaron Swartz',
      metadata: {
        releaseYear: '(2014)',
      },
      thumbnail: '/mock/movie-aaronsw.jpg',
    },
    {
      id: '7',
      category: 'video',
      contentCID: 'QmPSGARS6emPSEf8umwmjdG8AS7z7o8Nd36258B3BMi291',
      title: 'TPB AFK: The Pirate Bay Away from Keyboard',
      metadata: {
        releaseYear: '(2012)',
      },
      thumbnail: '/mock/movie-tbpafk.webp',
    },
    {
      id: '8',
      category: 'video',
      contentCID: 'QmYVCbux1BK5Z2eJjwr5pJayZiQhp2TAUdCNUostkFkwee',
      title: 'Cosmos Laundromat: First Cycle',
      metadata: {
        releaseYear: '(2015)',
      },
      thumbnail: '/mock/movie-cosmoslaundromat.webp',
    },
    {
      id: '9',
      category: 'audio',
      contentCID: 'Qmb9XpBQnw1vataDeWTh4jAnPMgNfKGyV7KWFz7uCvYHNd',
      title: 'Ghosts I-IV',
      metadata: {
        author: 'Nine Inch Nails',
      },
      thumbnail: '/mock/music-ghosts-i-iv.png',
    },
    {
      id: '10',
      category: 'video',
      contentCID: 'QmWeLCFA27vv91r6Jxu1C4PZTXj4mXpam6GGQzM6cS8FYD',
      title: 'Night of the Living Dead',
      metadata: {
        releaseYear: '(1968)',
      },
      thumbnail: '/mock/movie-nightofthelivingdead.webp',
    },
  ],
  'featured-music': [
      {
        id: '1',
        category: 'audio',
        metadata: {
          author: 'Swear and Shake',
        },
        title: 'Maple Ridge',
        contentCID: 'QmU6WhM6h3uvnicXcCQPgYpwrg9Moz68nVGWBeaYca2bMv',
        subtitle: 'Maple Ridge',
        thumbnail: '/mock/music-mapleridge.webp',
      },
      {
        id: '2',
        category: 'audio',
        metadata: {
          author: 'Adam McHeffey',
        },
        title: "Let's Kick Fire",
        contentCID: 'QmQ5mZFnruyqA4tzwguKJ9e4wLigokE2pQE3e99u3YK8vg',
        subtitle: "Let's Kick Fire",
        thumbnail: '/mock/music-letskickfire.jpg',
      },
      {
        id: '3',
        category: 'audio',
        metadata: {
          author: 'Nine Inch Nails',
        },
        title: 'The Slip',
        contentCID: 'QmR9hcaUqC7saAj8jjpkCwqa9bChmMJ3Mca17sRn6oiR2F',
        subtitle: 'The Slip',
        thumbnail: '/mock/music-theslip.jpg',
      },
      {
        id: '4',
        category: 'audio',
        title: 'Everything You Should Know',
        thumbnail: '/mock/music-everythingyoushouldknow.jpg',
        contentCID: 'QmbQ6JUzJPXaMdh5HBBZsczGLzhgz5DaFmUbRFZByZggRq',
        metadata: {
          author: 'Silence is Sexy',
          releaseYear: 2006,
        },
      },
      {
        id: '5',
        category: 'audio',
        metadata: {
          author: 'Nine Inch Nails',
        },
        title: 'Ghosts I-IV',
        contentCID: 'Qmb9XpBQnw1vataDeWTh4jAnPMgNfKGyV7KWFz7uCvYHNd',
        thumbnail: '/mock/music-ghosts-i-iv.png',
      },
      {
        id: '6',
        category: 'audio',
        metadata: {
          author: 'paniq',
        },
        title: 'Beyond Good and Evil',
        contentCID: 'QmSPWyFztzp3wntTyBLR5P3xc35wYekaUZ9YyzHtYRu7Ky',
        thumbnail: '/mock/music-paniq-bgae.jpg',
      },
      {
        id: '7',
        category: 'audio',
        metadata: {
          author: 'Brad Sucks',
        },
        title: 'Guess Who\'s A Mess',
        contentCID: 'QmNXPf83zcKpqp3nDFtjYuAcTWLqsLZkANbNmcH3YZSs34',
        thumbnail: '/mock/music-guesswhosamess.webp',
      },
      {
        id: '8',
        category: 'audio',
        metadata: {
          author: 'Swear and Shake',
        },
        title: 'Extended Play^',
        contentCID: 'Qme72tWtGJfQnUnWoadTb3PxkfQGAYziiAjf4hvqraokF9',
        thumbnail: '/mock/music-swearandshake-extendedplay.webp',
      },
      {
        id: '9',
        category: 'audio',
        metadata: {
          author: 'OK! Crazy Fiction Lady',
        },
        title: 'OK! Crazy Fiction Lady',
        contentCID: 'QmUD6WSCQcyyBGdwEqUiQBivU8QXR8psx5eiuqv3BqK76M',
        subtitle: 'OK! Crazy Fiction Lady',
        thumbnail: '/mock/music-okcfl.png',
      },
      {
        id: '10',
        category: 'audio',
        metadata: {
          author: 'OK! Crazy Fiction Lady',
        },
        title: 'Bye Bye Fishies',
        contentCID: 'QmZE5FLsfNDLvXpruXFehoGL3H1EUpbRpszcoFvSXx1iKd',
        subtitle: 'Bye Bye Fishies',
        thumbnail: '/mock/music-byebyefishies.png',
      },
      {
        id: '11',
        category: 'audio',
        metadata: {
          author: 'paniq',
        },
        title: 'Story of Ohm^',
        contentCID: 'QmcvUHaHp7bpnvs31Nka7rSQ2KEuWcgDSwK3V1wsRqMqns',
        thumbnail: '/mock/music-storyofohm.png',
      },
      {
        id: '12',
        category: 'audio',
        metadata: {
          author: 'Girl Talk',
        },
        title: 'All Day',
        contentCID: 'QmZAYJ1eQtTgMCcM2xxXiLjERqnbaseX1wMvuRbddqhaMj',
        subtitle: 'All Day',
        thumbnail: '/mock/music-allday.png',
      },
  ],
};
// const {orbiter} = useOrbiter();

// const releases = follow(orbiter.listenForReleases.bind(orbiter));
</script>
<!--
      {
        id: '8',
        category: 'audio',
        metadata: {
          author: 'Hello Madness',
        },
        title: 'Life and light after dusk',
        thumbnail: '/mock/music-lightandlightafterdusk.webp',
      },
-->
