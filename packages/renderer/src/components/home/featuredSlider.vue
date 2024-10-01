<template>
  <v-carousel
    hide-delimiters
    height="400px"
  >
    <template #prev="{ props }">
      <v-btn
        v-bind="props"
        variant="solo"
      >
      </v-btn>
    </template>
    <template #next="{ props }">
      <v-btn
        v-bind="props"
        variant="solo"
      >
      </v-btn>
    </template>
    <v-carousel-item
      v-for="featured in featuredList"
      :key="featured.name"
      :src="featured.cover"
      cover
      gradient="to right, rgba(0,0,0,.80), rgba(0,0,0,.01)"
    >
      <!-- <v-div class="d-flex align-center justify-center justify-md-start h-100"> -->
      <v-container class="fill-height">
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
              <p class="mb-4 text-h5 text-lg-h4">{{ featured.name }}</p>
              <div class="d-flex align-center">
                <v-rating
                  :model-value="featured.rating"
                  readonly
                  half-increments
                  size="small"
                  color="primary"
                />
                <p class="ml-4">{{ `${featured.rating} (Imbd)` }}</p>
              </div>
              <div class="d-flex align-center">
                <p class="ml-4 bg-background.lighten-2 pa-1 mr-2">{{ featured.classification }}</p>
                <p class="text-subtitle-2 text-medium-emphasis mr-2">{{ featured.duration }}</p>
                <v-icon
                  style="font-size: 8px;"
                  icon="fas fa-circle"
                ></v-icon>
                <p class="text-subtitle-2 text-medium-emphasis ml-2">{{ featured.date }}</p>
              </div>
              <p
                class="text-subtitle-2 text-medium-emphasis mb-4"
                style="line-height: 1.1em;"
              >
                {{ featured.description }}
              </p>
              <div class="d-flex mt-8">
                <v-btn
                  color="primary"
                  rounded="0"
                  prepend-icon="fas fa-play"
                  class="text-none mr-4"
                  text="Play Now"
                ></v-btn>
              </div>
            </v-sheet>
          </v-col>
          <v-col
            cols="12"
            md="2"
          >
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
              Watch Trailer
            </v-btn>
          </v-col>
        </v-row>
      </v-container>
    <!-- </v-div> -->
    </v-carousel-item>
  </v-carousel>
</template>

<script setup lang="ts">
import type { FeaturedItem } from '/@/views/homePage.vue';

interface Props {
  featuredList: Array<FeaturedItem>
}
defineProps<Props>();

</script>
