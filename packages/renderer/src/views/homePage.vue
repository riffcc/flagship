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
    >
      <v-col
        v-for="item in categorizedStaticReleases['featured-various']"
        :key="item.id"
      >
        <content-card
          :background-image="item.thumbnail"
          cursor-pointer
          :subtitle="item.category === 'movie' ? `(${item.metadata?.releaseYear})` : item.name"
          :title="item.category === 'movie' ? item.name : item.metadata?.author ?? ''"
          :width="xs ? '10.5rem' : '12rem'"
          @click="router.push(`/release/${item.id}`)"
        >
        </content-card>
      </v-col>
    </content-section>
    <content-section
      v-if="categorizedStaticReleases['featured-music'].length > 0"
      title="Featured Music"
    >
      <v-col
        v-for="item in categorizedStaticReleases['featured-music']"
        :key="item.id"
      >
        <content-card
          :background-image="item.thumbnail"
          cursor-pointer
          hovering-children
          :subtitle="item.metadata?.author ?? ''"
          :title="item.name"
          :width="xs ? '10.5rem' : '15rem'"
          @click="router.push(`/release/${item.id}`)"
        >
          <template #hovering>
            <v-icon
              size="4.5rem"
              icon="mdi-play"
              color="primary"
              class="position-absolute top-0 left-0 right-0 bottom-0 ma-auto"
            ></v-icon>
          </template>
        </content-card>
      </v-col>
    </content-section>
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
      :navigation="true"
    >
      <v-col
        v-for="item in categorizedStaticReleases['tv-shows']"
        :key="item.id"
      >
        <content-card
          background-gradient="to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)"
          :background-image="item.thumbnail"
          height="10rem"
          hovering-children
          overlapping
          :subtitle="`${item.metadata?.seasons} Seasons`"
          :title="item.name"
          width="17rem"
        >
          <template #hovering>
            <div class="position-absolute top-0 bottom-0 right-0 d-flex flex-column justify-center mr-2 ga-1">
              <v-btn
                size="small"
                color="grey-lighten-3"
                density="comfortable"
                icon="mdi-share-variant"
              ></v-btn>
              <v-btn
                size="small"
                color="grey-lighten-3"
                density="comfortable"
                icon="mdi-heart"
              ></v-btn>
              <v-btn
                size="small"
                color="grey-lighten-3"
                density="comfortable"
                icon="mdi-plus"
              ></v-btn>
            </div>
          </template>
          <template #actions>
            <v-btn
              color="primary"
              rounded="0"
              prepend-icon="mdi-play"
              size="small"
              class="position-absolute bottom-0 rigth-0 text-none ml-4 mb-10"
              text="Play now"
              @click="router.push(`/release/${item.id}`)"
            ></v-btn>
          </template>
        </content-card>
      </v-col>
    </content-section>
  </v-container>
</template>

<script setup lang="ts">
import {computed} from 'vue';
import {useDisplay} from 'vuetify';
import {useRouter} from 'vue-router';
import {suivre as follow} from '@constl/vue';
import ContentSection from '/@/components/home/contentSection.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import FeaturedSlider from '/@/components/home/featuredSlider.vue';
import InitiateModDBs from '/@/components/initiateModDBs.vue';
import {useDevStatus} from '/@/composables/devStatus';
import type {FeaturedItem, ItemContent} from '/@/composables/staticReleases';
import {useStaticReleases} from '/@/composables/staticReleases';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import { filterActivedFeature } from '/@/utils';

const router = useRouter();
const {xs} = useDisplay();
const {orbiter} = useOrbiter();
const {status} = useDevStatus();
const {staticFeaturedReleases, staticReleases} = useStaticReleases();

const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));

const featuredReleases = computed<Array<FeaturedItem>>(() => {

  // Note : this is a quick hack. We are using all releases from Orbiter as "featured releases".
  // TODO: Add option for featuring releases, and then modify below to show only these
  if (status.value === 'static') return staticFeaturedReleases.value.filter(fr => filterActivedFeature(fr));
  else {
    return (orbiterReleases.value || []).map((r): FeaturedItem => {
      return {
        id: (staticFeaturedReleases.value.length + 1).toString(),
        releaseId: r.release.id,
        startTime: '2025-01-01T00:00',
        endTime: '2026-01-01T00:00',
      };
    }).filter(fr => filterActivedFeature(fr));
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
