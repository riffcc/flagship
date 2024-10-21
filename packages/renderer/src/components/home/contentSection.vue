<template>
  <v-container class="mt-4">
    <v-container class="d-flex justify-between w-100">
      <p class="text-h5 font-weight-bold mb-6 mr-auto">{{ title }}</p>
      <template v-if="showNavigation">
        <v-btn
          icon="mdi-menu-left"
          density="compact"
          variant="tonal"
        ></v-btn>
        <v-btn
          icon="mdi-menu-right"
          density="compact"
          variant="tonal"
        ></v-btn>
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
        <v-hover
          v-slot="{props, isHovering}"
          open-delay="150"
          close-delay="150"
        >
          <v-card
            v-bind="props"
          >
            <v-img
              :src="item.thumbnail"
              height="160px"
              cover
              gradient="to bottom, rgba(0,0,0,.4), rgba(0,0,0,.41)"
            >
              <v-card-item>
                <p class="text-subtitle-1">{{ item.title }}</p>
                <p class="text-subtitle-2">{{ item.subtitle }}</p>
                <div
                  v-if="isHovering"
                  class="position-absolute top-0 bottom-0 right-0 d-flex flex-column justify-center mr-2 ga-1"
                >
                  <v-btn
                    size="small"
                    density="comfortable"
                    icon="mdi-share-variant"
                  ></v-btn>
                  <v-btn
                    size="small"
                    density="comfortable"
                    icon="mdi-heart"
                  ></v-btn>
                  <v-btn
                    size="small"
                    density="comfortable"
                    icon="mdi-plus"
                  ></v-btn>
                </div>
              </v-card-item>
              <v-card-actions>
                <v-btn
                  color="primary"
                  variant="flat"
                  rounded="0"
                  prepend-icon="mdi-play"
                  size="small"
                  class="mt-3 mb-1 text-none"
                  text="Play now"
                  @click="
                    router.push({
                      path: '/release',
                      query: {
                        category: item.category,
                        contentCID: item.contentCID,
                      },
                    })
                  "
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
          class="mx-auto cursor-pointer"
          color="transparent"
          width="170px"
          @click="
            router.push({
              path: '/release',
              query: {
                category: item.category,
                contentCID: item.contentCID,
                title: item.title,
                thumbnail: item.thumbnail,
                author: item.metadata?.author,
                description: item.metadata?.description,
                releaseYear: item.metadata?.releaseYear,
              },
            })
          "
        >
          <v-img
            :src="item.thumbnail"
            width="170px"
            cover
            aspect-ratio="1"
          ></v-img>
          <p class="text-body-2 text-center mt-1">
            {{ item.category === 'video' ? item.title : item.metadata?.author }}
          </p>
          <p class="text-caption text-center text-medium-emphasis">
            {{ item.category === 'video' ? item.metadata?.releaseYear : item.title }}
          </p>
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
        <v-hover
          v-slot="{props, isHovering}"
          open-delay="150"
          close-delay="150"
        >
          <v-sheet
            v-bind="props"
            color="transparent"
            class="pa-1"
            position="relative"
          >
            <v-img
              :src="item.thumbnail"
              width="240px"
              class="mx-auto"
              cover
              :aspect-ratio="1"
              @click="
                router.push({
                  path: '/release',
                  query: {
                    category: item.category,
                    contentCID: item.contentCID,
                    title: item.title,
                    thumbnail: item.thumbnail,
                    author: item.metadata?.author,
                    description: item.metadata?.description,
                    releaseYear: item.metadata?.releaseYear,
                  },
                })
              "
            ></v-img>
            <v-icon
              v-if="isHovering"
              :size="72"
              icon="mdi-play"
              color="primary"
              class="position-absolute top-0 left-0 right-0 bottom-0 ma-auto cursor-pointer"
            ></v-icon>
          </v-sheet>
        </v-hover>
        <p class="text-body-2 text-center">{{ item.metadata?.author }}</p>
        <p class="text-caption text-center text-medium-emphasis">{{ item.title }}</p>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import type {ItemContent} from '/@/views/homePage.vue';
import {useRouter} from 'vue-router';
const router = useRouter();

interface Props {
  title: string;
  items: ItemContent[];
  layout: 'card' | 'list' | 'grid';
  showNavigation?: boolean;
  showViewAll?: boolean;
}
defineProps<Props>();
</script>
