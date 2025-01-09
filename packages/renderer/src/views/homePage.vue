<template>
  <v-container class="fill-height pb-12">
    <InitiateModDBs />
    <featured-slider
      v-if="featuredReleases.length > 0"
      :featured-list="featuredReleases"
    />
    <content-section
      v-if="categorizedStaticReleases['featured-various'].length > 0"
      title="Featured"
      :items="categorizedStaticReleases['featured-various']"
      layout="list"
      description="Various featured content of mixed media"
      :show-view-all="true"
    />
    <content-section
      v-if="categorizedStaticReleases['featured-music'].length > 0"
      title="Featured Music"
      :items="categorizedStaticReleases['featured-music']"
      layout="grid"
      :show-view-all="true"
    />
    <v-alert
      v-if="categorizedStaticReleases['tv-shows'].length > 0"
      type="info"
      class="mt-8 mb-n8"
      color="black"
      text-color="white"
    >
      Riff.CC: We're still adding UI support for TV shows, but below you can see what TV will look
      like on this platform.
    </v-alert>
    <content-section
      v-if="categorizedStaticReleases['tv-shows'].length > 0"
      title="TV Shows"
      :items="categorizedStaticReleases['tv-shows']"
      layout="card"
      :show-navigation="true"
    />
  </v-container>
</template>

<script setup lang="ts">
import ContentSection from '/@/components/home/contentSection.vue';
import FeaturedSlider from '/@/components/home/featuredSlider.vue';
import {useDevStatus} from '/@/composables/devStatus';

import {suivre as follow} from '@constl/vue';
import {computed} from 'vue';
import InitiateModDBs from '../components/initiateModDBs.vue';
import type {FeaturedItem, ItemContent} from '/@/composables/staticReleases';
import {useStaticReleases} from '/@/composables/staticReleases';
import {useOrbiter} from '/@/plugins/orbiter/utils';

const {orbiter} = useOrbiter();
const {status} = useDevStatus();
const {staticFeaturedReleases, staticReleases} = useStaticReleases();

const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));

const featuredReleases = computed<Array<FeaturedItem>>(() => {
  // Note : this is a quick hack. We are using all releases from Orbiter as "featured releases".
  // TODO: Add option for featuring releases, and then modify below to show only these
  if (status.value === 'static') return staticFeaturedReleases.value;
  else {
    return (orbiterReleases.value || []).map((r): FeaturedItem => {
      return {
        id: r.release.id,
        category: r.release.release.category,
        contentCID: r.release.release.file,
        name: r.release.release.contentName,
        thumbnail: r.release.release.thumbnail,
        sourceSite: '/orbitdb/zdpuAwQJUpaVmGURrXjs4WMzwAmujzG2ALUAABqczyLNFziLw',
        classification: 'Unknown', // TODO
        cover: r.release.release.cover,
        rating: 1, // TODO,
        status: 'approved',
        startTime: 0,
        endTime: 1,
      };
    });
  }
});

function categorizeItems(items: ItemContent[], limit: number = 8) {
  const result: Record<string, ItemContent[]> = {
    'featured-music': [],
    'tv-shows': [],
    'featured-various': [],
  };

  const addedItems = new Set<string>(); // Track all added items to avoid duplication

  // Helper to add items without duplicates and respect limits
  function addToCategory(targetArray: ItemContent[], item: ItemContent, categoryLimit: number) {
    if (
      targetArray.length < categoryLimit &&
      !addedItems.has(item.id) &&
      item.status === 'approved'
    ) {
      targetArray.push(item);
      addedItems.add(item.id);
    }
  }

  // Separate items by category
  const musicItems = items.filter(item => item.category === 'music');
  const tvShowItems = items.filter(item => item.category === 'tvShow');
  const variousItems = items.filter(
    item => item.category !== 'tvShow' && item.category !== 'music',
  );

  // Add items to "tv-shows"
  tvShowItems.forEach(item => addToCategory(result['tv-shows'], item, limit));

  // Add items to "featured-music"
  musicItems.forEach(item => addToCategory(result['featured-music'], item, limit));

  // Add items to "featured-various"
  variousItems.forEach(item => addToCategory(result['featured-various'], item, limit));

  // Fill "featured-various" with leftovers, ensuring no duplicates
  musicItems.concat(tvShowItems).forEach(item => {
    addToCategory(result['featured-various'], item, limit);
  });

  return result;
}

const categorizedStaticReleases = computed(() => categorizeItems(staticReleases.value));
</script>
<!--
      {
        id: '8',
        category: 'audio',
        metadata: {
          author: 'Hello Madness',
        },
        name: 'Life and light after dusk',
        thumbnail: '/mock/music-lightandlightafterdusk.webp',
      },
-->
