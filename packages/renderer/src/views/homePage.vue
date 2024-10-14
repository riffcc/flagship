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

export interface FeaturedItem {
  id: string;
  classification: string;
  category?: 'audio' | 'video';
  contentCID?: string;
  cover: string;
  date: string;
  description: string;
  duration: string;
  name: string;
  rating: number;
}

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
    releaseYear?: number | string;
  }
}

const featuredReleases: Array<FeaturedItem> = [
  {
    id: '1',
    classification: 'PG',
    category: 'video',
    contentCID: 'Qmcw6w7pE4J5cxD9ygsp7kgnUyxMYc2UHxsnTj6sTxUuX3',
    cover: '/mock/movie-aaronsw.jpg',
    date: '2014',
    duration: '1h 45m',
    description: 'The Internet\'s Own Boy follows the story of programming prodigy and information activist Aaron Swartz.',
    name: 'The Internet\'s Own Boy: The Story of Aaron Swartz',
    rating: 4,
  },
  {
    id: '2',
    classification: 'PG',
    category: 'video',
    contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
    cover: '/mock/movie-rip.png',
    date: '2008',
    duration: '1h 26m',
    description: 'Join filmmaker Brett Gaylor and mashup artist Girl Talk as they explore copyright and content creation in the digital age. In the process they dissect the media landscape of the 21st century and shatter the wall between users and producers.',
    name: 'RiP!: A Remix Manifesto',
    rating: 4.5,
  },
  {
    id: '3',
    classification: 'Unrated',
    category: 'video',
    contentCID: 'QmabExd2NYTdDb2kX9tVAdf1LwzgJnyivknbCnF3nuG289',
    cover: '/mock/featured-tpbafk-fanart.png',
    date: '2012',
    duration: '1h 26m',
    description: 'The Pirate Bay Away From Keyboard is a documentary film about the file sharing website The Pirate Bay.',
    name: 'TPB AFK: The Pirate Bay Away From Keyboard',
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
      thumbnail: '/mock/music-letskickfire.jpg',
    },
    {
      id: '3',
      category: 'audio',
      title: 'Maple Ridge',
      metadata: {
        author: 'Swear and Shake',
      },
      thumbnail: '/mock/music-mapleridge.webp',
    },
    {
      id: '4',
      category: 'audio',
      title: 'The Slip',
      metadata: {
        author: 'Nine Inch Nails',
      },
      thumbnail: '/mock/music-theslip.jpg',
    },
    {
      id: '5',
      category: 'audio',
      title: 'IN RAINBOWS',
      metadata: {
        author: 'Radiohead',
      },
      thumbnail: '/mock/music-inrainbows.jpg',
    },
    {
      id: '6',
      category: 'video',
      contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      title: 'The Internet\'s Own Boy: The Story of Aaron Swartz',
      metadata: {
        releaseYear: '(2014)',
      },
      thumbnail: '/mock/movie-aaronsw.jpg',
    },
    {
      id: '7',
      category: 'video',
      contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      title: 'TPB AFK: The Pirate Bay Away from Keyboard',
      metadata: {
        releaseYear: '(2012)',
      },
      thumbnail: '/mock/movie-tbpafk.webp',
    },
    {
      id: '8',
      category: 'video',
      contentCID: 'QmTWWUmvC9txvE7aHs9xHd541qLx3ax58urvx3Kb3SFK2Q',
      title: 'Cosmos Laundromat',
      metadata: {
        releaseYear: '(2015)',
      },
      thumbnail: '/mock/movie-cosmoslaundromat.webp',
    },
    {
      id: '9',
      category: 'audio',
      title: 'Ghosts I-IV',
      metadata: {
        author: 'Nine Inch Nails',
      },
      thumbnail: '/mock/music-ghosts-i-iv.png',
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
        thumbnail: '/mock/music-mapleridge.webp',
      },
      {
        id: '2',
        category: 'audio',
        metadata: {
          author: 'Adam McHeffey',
        },
        title: "Let's Kick Fire",
        thumbnail: '/mock/music-letskickfire.jpg',
      },
      {
        id: '3',
        category: 'audio',
        metadata: {
          author: 'Nine Inch Nails',
        },
        title: 'The Slip',
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
        thumbnail: '/mock/music-ghosts-i-iv.png',
      },
      {
        id: '6',
        category: 'audio',
        metadata: {
          author: 'paniq',
        },
        title: 'Beyond Good and Evil',
        thumbnail: '/mock/music-paniq-bgae.jpg',
      },
      {
        id: '7',
        category: 'audio',
        metadata: {
          author: 'Brad Sucks',
        },
        title: 'Guess Who\'s A Mess',
        thumbnail: '/mock/music-guesswhosamess.webp',
      },

      {
        id: '8',
        category: 'audio',
        metadata: {
          author: 'Hello Madness',
        },
        title: 'Life and light after dusk',
        thumbnail: '/mock/music-lightandlightafterdusk.webp',
      },
      {
        id: '9',
        category: 'audio',
        metadata: {
          author: 'OK! Crazy Fiction Lady',
        },
        title: 'OK! Crazy Fiction Lady',
        thumbnail: '/mock/music-okcfl.png',
      },
      {
        id: '10',
        category: 'audio',
        metadata: {
          author: 'OK! Crazy Fiction Lady',
        },
        title: 'Bye Bye Fishies',
        thumbnail: '/mock/music-byebyefishies.png',
      },
      {
        id: '11',
        category: 'audio',
        metadata: {
          author: 'paniq',
        },
        title: 'Story of Ohm',
        thumbnail: '/mock/music-storyofohm.png',
      },
      {
        id: '12',
        category: 'audio',
        metadata: {
          author: 'Girl Talk',
        },
        title: 'All Day',
        thumbnail: '/mock/music-allday.png',
      },
  ],
};
// const {orbiter} = useOrbiter();

// const releases = follow(orbiter.listenForReleases.bind(orbiter));
</script>

