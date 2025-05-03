<template>
  <v-container class="fill-height pb-16">
    <v-sheet
      v-if="isLoading"
      color="transparent"
      class="d-flex w-100 fill-height align-center justify-center"
    >
      <v-progress-circular
        indeterminate
        color="primary"
      ></v-progress-circular>
    </v-sheet>
    <v-sheet
      v-else-if="noContent || tvShowReleases.length === 0"
      color="transparent"
      class="d-flex flex-column mx-auto"
      max-width="16rem"
    >
      <p class="text-white text-center mb-2">No TV show content found.</p>
      <!-- Optional: Add an upload button if desired -->
      <!-- <v-btn color="primary-darken-1" @click="router.push('/upload')">Upload TV Show</v-btn> -->
    </v-sheet>
    <template v-else>
       <content-section
        title="TV Shows"
        :navigation="true"
      >
        <v-col
          v-for="item in tvShowReleases"
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
import ContentSection from '/@/components/home/contentSection.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import { parseUrlOrCid } from '/@/utils';
import { useReleasesStore } from '../stores/releases';
import { storeToRefs } from 'pinia';

const router = useRouter();
const {xs} = useDisplay(); // Keep xs if needed for responsive layout in the card

const releasesStore = useReleasesStore();
const {releases, isLoading, noContent} = storeToRefs(releasesStore);

// Filter releases specifically for TV shows
const tvShowReleases = computed(() => {
  return releases.value.filter(item => item.category === 'tvShow');
});
</script>
