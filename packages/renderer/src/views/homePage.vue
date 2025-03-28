<template>
  <v-container class="fill-height pb-16">
    <v-sheet
      v-if="!(featuredReleases.length > 0) && !(releases.length > 0)"
      color="transparent"
      class="d-flex flex-column mx-auto"
      max-width="16rem"
    >
      <p class="text-white text-center mb-2">No content here. Please upload a release first.</p>
      <v-btn
        color="primary-darken-1"
        @click="router.push('/upload')"
      >
        Go to Upload
      </v-btn>
    </v-sheet>
    <template v-else>
      <featured-slider
        v-if="featuredReleases.length > 0"
        :featured-list="featuredReleases"
      />
      <content-section
        v-if="categorizedReleases['featured-various'].length > 0"
        title="Featured"
      >
        <v-col
          v-for="item in categorizedReleases['featured-various']"
          :key="item.id"
        >
          <content-card
            :background-image="parseUrlOrCid(item.thumbnail)"
            cursor-pointer
            :subtitle="item.category === 'movie' ? item.metadata['releaseYear'] ? `(${item.metadata['releaseYear']})` : undefined : item.name"
            :title="item.category === 'movie' ? item.name : item.author"
            :width="xs ? '10.5rem' : '12rem'"
            :source-site="item.sourceSite"
            @click="router.push(`/release/${item.id}`)"
          >
          </content-card>
        </v-col>
      </content-section>
      <content-section
        v-if="categorizedReleases['featured-music'].length > 0"
        title="Featured Music"
      >
        <v-col
          v-for="item in categorizedReleases['featured-music']"
          :key="item.id"
        >
          <content-card
            :background-image="parseUrlOrCid(item.thumbnail)"
            cursor-pointer
            hovering-children
            :subtitle="item.author ?? ''"
            :title="item.name"
            :width="xs ? '10.5rem' : '15rem'"
            :source-site="item.sourceSite"
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
        v-if="categorizedReleases['tv-shows'].length > 0"
        type="info"
        class="mt-8 mb-n8"
        color="black"
        text-color="white"
      >
        Riff.CC: We're still adding UI support for TV shows, but below you can see what TV will look
        like on this platform.
      </v-alert>
      <content-section
        v-if="categorizedReleases['tv-shows'].length > 0"
        title="TV Shows"
        :navigation="true"
      >
        <v-col
          v-for="item in categorizedReleases['tv-shows']"
          :key="item.id"
        >
          <content-card
            background-gradient="to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)"
            :background-image="parseUrlOrCid(item.thumbnail)"
            height="10rem"
            hovering-children
            overlapping
            :subtitle="item.metadata['seasons'] ? `${item.metadata['seasons']} Seasons` : undefined"
            :title="item.name"
            :source-site="item.sourceSite"
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
    </template>
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
import {useStaticStatus} from '../composables/staticStatus';
import type {FeaturedReleaseItem, ReleaseItem} from '/@/@types/release';
import {useStaticReleases} from '/@/composables/staticReleases';
import {useOrbiter} from '/@/plugins/orbiter/utils';
import { filterActivedFeature, parseUrlOrCid } from '/@/utils';

const router = useRouter();
const {xs} = useDisplay();
const {orbiter} = useOrbiter();
const {staticStatus} = useStaticStatus();
const {staticFeaturedReleases, staticReleases} = useStaticReleases();

const orbiterReleases = follow(orbiter.listenForReleases.bind(orbiter));

const orbiterFeaturedReleases = follow(orbiter.listenForSiteFeaturedReleases.bind(orbiter));

const releases = computed<ReleaseItem[]>(() => {
  if (staticStatus.value === 'static') return staticReleases.value;
  else {
    return (orbiterReleases.value || []).map((r) => {
      return {
        id: r.release.id,
        name: r.release.release.contentName,
        contentCID: r.release.release.file,
        category: r.release.release.category,
        author: r.release.release.author,
        thumbnail: r.release.release.thumbnail,
        cover: r.release.release.cover,
        metadata: JSON.parse(r.release.release.metadata as string),
        sourceSite: r.site,
      };
    }) as ReleaseItem[];
  }
});

const featuredReleases = computed<FeaturedReleaseItem[]>(() => {
  if (staticStatus.value === 'static') return staticFeaturedReleases.value.filter(fr => filterActivedFeature(fr));
  else {
    return (orbiterFeaturedReleases.value || []).map((fr): FeaturedReleaseItem => {
      return {
        id: fr.id,
        releaseId: fr.featured.releaseId,
        startTime: fr.featured.startTime,
        endTime: fr.featured.endTime,
      };
    }).filter(fr => filterActivedFeature(fr));
  }
});

function categorizeItems(items: ReleaseItem[], limit: number = 8) {
  const result: Record<string, ReleaseItem[]> = {
    'featured-music': [],
    'tv-shows': [],
    'featured-various': [],
  };
  if (!(items.length > 0)) return result;
  const addedItems = new Set<string>(); // Track all added items to avoid duplication

  // Helper to add items without duplicates and respect limits
  function addToCategory(targetArray: ReleaseItem[], item: ReleaseItem, categoryLimit: number) {
    if (
      targetArray.length < categoryLimit && item.id &&
      !addedItems.has(item.id)
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

const categorizedReleases = computed(() => categorizeItems(staticStatus.value === 'static' ? staticReleases.value : releases.value));
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
