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
          :src="previousSlideImage"
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
          :src="nextSlideImage"
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
      :src="featuredItem.cover ?? featuredItem.thumbnail"
      cover
      gradient="to right, rgba(0,0,0,.8), rgba(0,0,0,.01)"
    >
      <v-container
        class="fill-height"
        :style="showDefederation ? `border: 1px solid ${lensColorHash(featuredItem)};` : ''"
      >
        <v-row
          justify="center"
          align="center"
          justify-sm="space-around"
          class="px-sm-12"
        >
          <v-col
            cols="12"
            sm="7"
            md="6"
            lg="5"
          >
            <v-sheet
              color="transparent"
              class="my-10"
            >
              <p class="mb-4 text-h5 text-lg-h4">
                {{
                  featuredItem.category === 'music'
                    ? `${featuredItem.name} - ${featuredItem.metadata?.author}`
                    : featuredItem.name
                }}
              </p>
              <div class="d-flex align-center ga-2">
                <v-chip label>
                  {{ featuredItem.metadata?.classification }}
                </v-chip>
                <v-chip
                  variant="text"
                  class="text-medium-emphasis"
                >
                  {{ featuredItem.metadata?.duration }} • {{ featuredItem.metadata?.releaseYear }}
                </v-chip>
              </div>
              <p
                class="text-subtitle-2 text-medium-emphasis mt-2 mb-4"
                style="line-height: 1.1em"
              >
                {{ featuredItem.metadata?.description }}
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
import {base16} from 'multiformats/bases/base16';
import {CID} from 'multiformats/cid';
import {computed, ref} from 'vue';
import {useRouter} from 'vue-router';
import {useDisplay} from 'vuetify';
import {useShowDefederation} from '/@/composables/showDefed';
import {type FeaturedItem, type ItemContent, useStaticReleases} from '/@/composables/staticReleases';

const props = defineProps<{
  featuredList: Array<FeaturedItem>;
}>();

const router = useRouter();
const {showDefederation} = useShowDefederation();
const {staticReleases} = useStaticReleases();
const {xs} = useDisplay();
const slide = ref(0);

const featuredItems = computed(() => {
  const featuredIds = props.featuredList.map(f => f.releaseId);
  return staticReleases.value.filter(sr => featuredIds.includes(sr.id));
});

const previousSlideImage = computed(() => {
  const previousIndex = slide.value === 0 ? featuredItems.value.length - 1 : slide.value - 1;
  return featuredItems.value[previousIndex].cover ?? featuredItems.value[previousIndex].thumbnail;
});

const nextSlideImage = computed(() => {
  const nextIndex = slide.value === featuredItems.value.length - 1 ? 0 : slide.value + 1;
  return featuredItems.value[nextIndex].cover ?? featuredItems.value[nextIndex].thumbnail;
});


// Colors
const lensColorHash = (featured: ItemContent): string => {
  const idSite = featured.sourceSite.replace('/orbitdb/', '');
  return '#' + CID.parse(idSite).toString(base16.encoder).slice(-6);
};
</script>
