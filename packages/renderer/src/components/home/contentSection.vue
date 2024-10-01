<template>
  <v-container class="mt-4">
    <v-container class="d-flex justify-between w-100">
      <p class="text-h5 font-weight-bold mb-6 mr-auto">{{ title }}</p>
      <template v-if="showNavigation">
        <v-btn
          icon
          class="justify-self-end"
        >
          <v-icon
            icon="fas fa-caret-left"
            size="x-small"
          ></v-icon>
        </v-btn>
        <v-btn
          icon
          class="justify-self-end"
        >
          <v-icon
            icon="fas fa-caret-right"
            size="x-small"
            color="primary"
          ></v-icon>
        </v-btn>
      </template>
      <v-btn
        v-if="showViewAll"
        variant="text"
        text="View All"
        class="text-none"
      ></v-btn>
    </v-container>

    <v-row
      v-if="layout === 'card'"
      justify="center"
    >
      <v-col
        v-for="item in items"
        :key="item.id"
      >
        <v-hover v-slot="{ props, isHovering }">
          <v-card
            v-bind="props"
            position="relative"
          >
            <v-img
              :src="item.thumbnail"
              height="100%"
              cover
              gradient="to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)"
            >
              <v-card-item>
                <v-sheet class="d-flex bg-transparent">
                  <div>
                    <p class="text-subtitle-1">{{ item.title }}</p>
                    <p class="text-subtitle-2">{{ item.subtitle }}</p>
                  </div>
                  <v-sheet
                    v-if="isHovering"
                    position="absolute"
                    location="right"
                    class="d-flex mr-2 flex-column w-auto bg-transparent"
                  >
                    <v-btn
                      size="x-small"
                      class="my-1"
                      color="white"
                      icon
                    >
                      <v-icon
                        icon="fas fa-share-nodes"
                        size="small"
                      ></v-icon>
                    </v-btn>
                    <v-btn
                      size="x-small"
                      class="my-1"
                      color="white"
                      icon
                    >
                      <v-icon
                        icon="fas fa-heart"
                        size="small"
                      ></v-icon>
                    </v-btn>
                    <v-btn
                      size="x-small"
                      class="my-1"
                      color="white"
                      icon
                    >
                      <v-icon
                        icon="fas fa-plus"
                        size="small"
                      ></v-icon>
                    </v-btn>
                  </v-sheet>
                </v-sheet>
              </v-card-item>
              <v-card-actions>
                <v-btn
                  color="primary"
                  variant="flat"
                  rounded="0"
                  prepend-icon="fas fa-play"
                  size="small"
                  class="mt-3 mb-1 text-none"
                  text="Play Now"
                ></v-btn>
              </v-card-actions>
            </v-img>
          </v-card>
        </v-hover>
      </v-col>
    </v-row>
    <v-row
      v-if="layout === 'list'"
      justify="center"
    >
      <v-col
        v-for="item in items"
        :key="item.id"
      >
        <v-sheet
          class="mx-auto"
          width="170px"
        >
          <v-img
            :src="item.thumbnail"
            width="170px"
            cover
            aspect-ratio="1"
          ></v-img>
          <p class="text-body-2 text-center">{{ item.title }}</p>
          <p class="text-caption text-center text-medium-emphasis">{{ item.subtitle }}</p>
        </v-sheet>
      </v-col>
    </v-row>
    <v-row
      v-if="layout === 'grid'"
      justify="center"
    >
      <v-col
        v-for="item in items"
        :key="item.id"
      >
        <v-sheet
          color="transparent"
          class="pa-1"
          position="relative"
        >
          <v-img
            :src="item.thumbnail"
            width="240px"
            class="mx-auto"
            cover
            :aspect-ratio="3 / 3.5"
          ></v-img>
          <v-btn
            position="absolute"
            size="x-large"
            location="center"
            icon
            variant="solo"
          >
            <v-icon
              icon="fas fa-play"
              size="x-large"
              color="primary"
            ></v-icon>
          </v-btn>
        </v-sheet>
        <p class="text-body-2 text-center">{{ item.title }}</p>
        <p class="text-caption text-center text-medium-emphasis">{{ item.subtitle }}</p>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import type { ItemContent } from '/@/views/homePage.vue';


interface Props {
  title: string;
  items: ItemContent[];
  layout: 'card' | 'list' | 'grid';
  showNavigation?: boolean;
  showViewAll?: boolean;
}
defineProps<Props>();
</script>
