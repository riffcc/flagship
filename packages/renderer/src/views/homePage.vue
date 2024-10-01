<template>
  <v-container class="fill-height">
    <featured-slider :featured-list="featuredReleases" />
    <content-section
      title="TV Popular Shows"
      :items="staticData['tv-popular-shows']"
      layout="card"
      :show-navigation="true"
    />
    <content-section
      title="Featured Music"
      :items="staticData['featured-music']"
      layout="list"
      :show-view-all="true"
    />
    <content-section
      title="Latest Movies"
      :items="staticData['latest-movies']"
      layout="grid"
      :show-view-all="true"
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
  'featured-music': [
    {
      id: '1',
      title: 'Nine Inch Nails',
      subtitle: 'The Slip',
      thumbnail: '/mock/featured-music-5.png',
    },
    {
      id: '2',
      title: 'Adam McHeffey',
      subtitle: "Let' Kick Fire",
      thumbnail: '/mock/featured-music-4.png',
    },
    {
      id: '3',
      title: 'Swear and Sheke',
      subtitle: 'Maple Ridge',
      thumbnail: '/mock/featured-music-3.png',
    },
    {
      id: '4',
      title: 'Nine Inch Nails',
      subtitle: 'The Slip',
      thumbnail: '/mock/featured-music-2.png',
    },
    {
      id: '5',
      title: 'Adam McHeffey',
      subtitle: "Let' Kick Fire",
      thumbnail: '/mock/featured-music-1.png',
    },
  ],
  'latest-movies': [
      {
        id: '1',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie1.png',
      },
      {
        id: '2',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie2.png',
      },
      {
        id: '3',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie3.png',
      },
      {
        id: '4',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie4.png',
      },
      {
        id: '5',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie5.png',
      },
      {
        id: '6',
        title: 'Adam McHeffey',
        subtitle: "Let' Kick Fire",
        thumbnail: '/mock/latest-movie6.png',
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

