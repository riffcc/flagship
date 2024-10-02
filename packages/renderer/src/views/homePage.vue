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
  cover: string;
  date: string;
  description: string;
  duration: string;
  name: string;
  rating: number;
}

export interface ItemContent {
  id: string;
  title: string;
  subtitle: string;
  thumbnail?: string;
}

const featuredReleases: Array<FeaturedItem> = [
  {
    id: '1',
    classification: 'PG',
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
      title: 'Zombie World',
      subtitle: '2 Seasons',
      thumbnail: '/mock/tv-show-1.png',
    },
    {
      id: '2',
      title: 'Zombie World',
      subtitle: '2 Seasons',
      thumbnail: '/mock/tv-show-2.png',
    },
    {
      id: '3',
      title: 'Zombie World',
      subtitle: '2 Seasons',
      thumbnail: '/mock/tv-show-3.png',
    },
    {
      id: '4',
      title: 'Zombie World',
      subtitle: '2 Seasons',
      thumbnail: '/mock/tv-show-4.png',
    },
  ],
  'featured-various': [
    {
      id: '1',
      title: 'RiP!: A Remix Manifesto',
      subtitle: '(2008)',
      thumbnail: '/mock/movie-rip-poster.png',
    },
    {
      id: '2',
      title: 'Adam McHeffey',
      subtitle: "Let's Kick Fire",
      thumbnail: '/mock/music-letskickfire.jpg',
    },
    {
      id: '3',
      title: 'Swear and Shake',
      subtitle: 'Maple Ridge',
      thumbnail: '/mock/music-mapleridge.webp',
    },
    {
      id: '4',
      title: 'Nine Inch Nails',
      subtitle: 'The Slip',
      thumbnail: '/mock/music-theslip.jpg',
    },
    {
      id: '5',
      title: 'Radiohead',
      subtitle: 'IN RAINBOWS',
      thumbnail: '/mock/music-inrainbows.jpg',
    },
    {
      id: '6',
      title: 'The Internet\'s Own Boy: The Story of Aaron Swartz',
      subtitle: '(2014)',
      thumbnail: '/mock/movie-aaronsw.jpg',
    },
    {
      id: '7',
      title: 'TPB AFK: The Pirate Bay Away from Keyboard',
      subtitle: '(2012)',
      thumbnail: '/mock/movie-tbpafk.webp',
    },
    {
      id: '8',
      title: 'Silence is Sexy',
      subtitle: 'Everything You Should Know',
      thumbnail: '/mock/music-everythingyoushouldknow.jpg',
    },
    {
      id: '9',
      title: 'Nine Inch Nails',
      subtitle: 'Ghosts I-IV',
      thumbnail: '/mock/music-ghosts-i-iv.png',
    },
  ],
  'featured-music': [
      {
        id: '1',
        title: 'Swear and Shake',
        subtitle: 'Maple Ridge',
        thumbnail: '/mock/music-mapleridge.webp',
      },
      {
        id: '2',
        title: 'Adam McHeffey',
        subtitle: "Let's Kick Fire",
        thumbnail: '/mock/music-letskickfire.jpg',
      },
      {
        id: '3',
        title: 'Nine Inch Nails',
        subtitle: 'The Slip',
        thumbnail: '/mock/music-theslip.jpg',
      },
      {
        id: '4',
        title: 'Silence is Sexy',
        subtitle: 'Everything You Should Know',
        thumbnail: '/mock/music-everythingyoushouldknow.jpg',
      },
      {
        id: '5',
        title: 'Nine Inch Nails',
        subtitle: 'Ghosts I-IV',
        thumbnail: '/mock/music-ghosts-i-iv.png',
      },
      {
        id: '6',
        title: 'paniq',
        subtitle: 'Beyond Good and Evil',
        thumbnail: '/mock/music-paniq-bgae.jpg',
      },
      {
        id: '7',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie7.png',
      },

      {
        id: '8',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie8.png',
      },
      {
        id: '9',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie9.png',
      },
      {
        id: '10',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie10.png',
      },
      {
        id: '11',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie11.png',
      },
      {
        id: '12',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie12.png',
      },
  ],
};
// const {orbiter} = useOrbiter();

// const releases = follow(orbiter.listenForReleases.bind(orbiter));
</script>

