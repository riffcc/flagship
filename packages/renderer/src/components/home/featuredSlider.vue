<template>
  <v-carousel
    v-model="slide"
    hide-delimiters
    height="400px"
  >
    <template #prev="{props: prevProps}">
      <v-sheet
        v-if="featuredItems.length > 1"
        color="transparent"
        width="64px"
        class="position-relative h-100"
      >
        <v-img
          v-if="!xs"
          :src="parseUrlOrCid(previousSlideImage)"
          height="100%"
          position="right"
          gradient="rgba(33,33,33,.6), rgba(33,33,33,.4)"
          cover
        ></v-img>
        <v-btn
          v-bind="prevProps"
          :style="{zIndex: 1000}"
          position="absolute"
          location="center"
          variant="plain"
        >
        </v-btn>
      </v-sheet>
    </template>
    <template #next="{props: nextProps}">
      <v-sheet
        v-if="featuredItems.length > 1"
        color="transparent"
        width="64px"
        class="position-relative h-100"
      >
        <v-img
          v-if="!xs"
          :src="parseUrlOrCid(nextSlideImage)"
          height="100%"
          position="left"
          gradient="rgba(33,33,33,.6), rgba(33,33,33,.4)"
          cover
        ></v-img>
        <v-btn
          v-bind="nextProps"
          :style="{zIndex: 1000}"
          position="absolute"
          location="center"
          variant="plain"
        >
        </v-btn>
      </v-sheet>
    </template>
    <v-carousel-item
      v-for="featuredItem in featuredItems"
      :key="featuredItem.id"
      :src="parseUrlOrCid(featuredItem.cover ?? featuredItem.thumbnail)"
      cover
      gradient="to right, rgba(0,0,0,.8), rgba(0,0,0,.01)"
    >
      <v-container
        class="fill-height"
        :style="showDefederation && featuredItem.sourceSite ? `border: 1px solid ${getSiteColor(featuredItem.sourceSite)};` : ''"
      >
        <v-row
          justify="center"
          class="px-md-4"
        >
          <v-col
            cols="10"
            sm="9"
            md="8"
            lg="6"
          >
            <v-sheet
              color="transparent"
            >
              <h5 class="text-h5 text-sm-h4">
                {{ featuredItem.name }}
              </h5>
              <template v-if="['music'].includes(featuredItem.category)">
                <p class="text-body-2 text-sm-body-1">
                  {{ featuredItem.author }}
                </p>
                <v-chip
                  v-if="featuredItem.metadata['totalSongs'] && featuredItem.metadata['releaseYear']"
                  class="opacity-100 px-0 text-medium-emphasis mt-2"
                  density="comfortable"
                  disabled
                  variant="text"
                >
                  {{ featuredItem.metadata['totalSongs'] }} Songs • {{ featuredItem.metadata['releaseYear'] }}
                </v-chip>
              </template>

              <v-chip-group
                v-if="['movie'].includes(featuredItem.category)"
              >
                <v-chip
                  v-if="featuredItem.metadata['classification']"
                  class="opacity-100"
                  density="comfortable"
                  disabled
                  label
                >
                  {{ featuredItem.metadata['classification'] }}
                </v-chip>
                <v-chip
                  v-if="featuredItem.metadata['duration'] && featuredItem.metadata['releaseYear']"
                  density="comfortable"
                  disabled
                  class="opacity-100 text-medium-emphasis"
                  variant="text"
                >
                  {{ featuredItem.metadata['duration'] }} • {{ featuredItem.metadata['releaseYear'] }}
                </v-chip>
              </v-chip-group>
              <p
                v-if="featuredItem.metadata?.description"
                class="text-subtitle-2 text-medium-emphasis mt-4"
                style="line-height: 1.1em"
              >
                {{ featuredItem.metadata.description }}
              </p>
              <div class="d-flex mt-8">
                <v-btn
                  color="primary"
                  rounded="0"
                  prepend-icon="mdi-play"
                  class="text-none mr-4"
                  text="Play now"
                  @click="router.push(`/release/${featuredItem.id}`)"
                ></v-btn>
              </div>
            </v-sheet>
          </v-col>
          <v-col
            cols="12"
            md="2"
          >
            <!-- TODO: Add preview button
            <v-btn
              variant="plain"
              class="text-none text-h6"
              :ripple="false"
              size="x-large"
            >
              <template #prepend>
                <v-icon
                  icon="far fa-circle-play"
                  class="mb-1"
                  size="small"
                ></v-icon>
              </template>
              Preview
            </v-btn>
          -->
          </v-col>
        </v-row>
      </v-container>
    </v-carousel-item>
  </v-carousel>
</template>

<script setup lang="ts">
import {computed, ref} from 'vue';
import {useRouter} from 'vue-router';
import {useDisplay} from 'vuetify';
import {parseUrlOrCid} from '/@/utils';
import {useShowDefederation} from '/@/composables/showDefed';
import { useSiteColors } from '/@/composables/siteColors';
import { useReleasesStore } from '/@/stores/releases';
import { storeToRefs } from 'pinia';


const router = useRouter();
const {showDefederation} = useShowDefederation();
const {xs} = useDisplay();
const slide = ref(0);
const releasesStore = useReleasesStore();
const {releases, featuredReleases} = storeToRefs(releasesStore);

const featuredItems = computed(() => {
  const featuredReleasesIds = featuredReleases.value.map(fr => fr.releaseId);
  return releases.value.filter(r => r.id && featuredReleasesIds.includes(r.id));
});
const previousSlideImage = computed(() => {
  const previousIndex = slide.value === 0 ? featuredItems.value.length - 1 : slide.value - 1;
  return featuredItems.value[previousIndex].cover ?? featuredItems.value[previousIndex].thumbnail;
});

const nextSlideImage = computed(() => {
  const nextIndex = slide.value === featuredItems.value.length - 1 ? 0 : slide.value + 1;
  return featuredItems.value[nextIndex].cover ?? featuredItems.value[nextIndex].thumbnail;
});

const {getSiteColor} = useSiteColors();

</script>
