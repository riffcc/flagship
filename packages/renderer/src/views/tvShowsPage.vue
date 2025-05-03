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
      v-else-if="noContent || tvSeries.length === 0"  
      color="transparent"
      class="d-flex flex-column mx-auto"
      max-width="16rem"
    >
      <p class="text-white text-center mb-2">No TV Series found.</p> 
      <!-- Optional: Link to admin or upload if appropriate -->
      <!-- <v-btn color="primary-darken-1" @click="router.push('/admin')">Manage Series</v-btn> -->
    </v-sheet>
    <template v-else>
      <content-section title="TV Shows">
        <v-col
          v-for="item in tvSeries" 
          :key="item.id"
          cols="auto" 
        >
          <content-card
            :background-image="parseUrlOrCid(item.thumbnail || item.cover)" 
            cursor-pointer
            hovering-children
            :subtitle="item.description?.substring(0, 50) + (item.description && item.description.length > 50 ? '...' : '') ?? ''" 
            :title="item.name"
            :width="xs ? '10.5rem' : '15rem'"
            :source-site="item.sourceSite"
            @click="router.push(`/tv-show/${item.id}`)" 
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
    </template>
  </v-container>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useDisplay } from 'vuetify';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import ContentSection from '/@/components/home/contentSection.vue';
import ContentCard from '/@/components/misc/contentCard.vue';
import { parseUrlOrCid } from '/@/utils';
// import { useReleasesStore } from '../stores/releases'; // No longer needed here
import { useTvSeriesStore } from '/@/stores/tvSeries'; // Import the new TV Series store

const router = useRouter();
const { xs } = useDisplay();

// const releasesStore = useReleasesStore(); // No longer needed here
// const { releases, isLoading, noContent } = storeToRefs(releasesStore); // No longer needed here

const tvSeriesStore = useTvSeriesStore(); // Use the TV Series store
const { tvSeries, isLoading, noContent } = storeToRefs(tvSeriesStore); // Get series data

// Filter releases specifically for TV shows - No longer needed here
// const tvShowReleases = computed(() => {
//   return releases.value.filter(item => item.category === 'tvShow');
// });
</script>
