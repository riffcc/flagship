<template>
  <div class="d-flex align-center justify-space-between w-100 mt-12 mb-4">
    <p class="text-h6 text-sm-h5 font-weight-bold">{{ title }}</p>
    <template v-if="showNavigation">
      <div>
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
      </div>
    </template>
    <v-btn
      v-if="showViewAll"
      variant="text"
      :size="xs ? 'small' : 'default'"
      slim
      density="comfortable"
      text="View All"
      class="text-none"
    ></v-btn>
  </div>
  <v-row
    v-if="layout === 'list'"
  >
    <v-col
      v-for="item in items"
      :key="item.id"
    >
      <v-sheet
        class="cursor-pointer mx-auto"
        color="transparent"
        :width="xs ? '172px' : '192px'"
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
          width="100%"
          cover
          aspect-ratio="1"
        ></v-img>
        <p class="text-caption text-sm-subtitle-1 text-center mt-1">
          {{ item.category === 'video' ? item.title : item.metadata?.author }}
        </p>
        <p class="text-caption text-sm-subtitle-1 text-center text-medium-emphasis">
          {{ item.category === 'video' ? item.metadata?.releaseYear : item.title }}
        </p>
      </v-sheet>
    </v-col>
  </v-row>
  <v-row
    v-if="layout === 'grid'"
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
          class="cursor-pointer mx-auto"
          color="transparent"
          :width="xs ? '172px' : '240px'"
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
          <v-sheet
            color="transparent"
            position="relative"
          >
            <v-img
              :src="item.thumbnail"
              cover
              :aspect-ratio="1"
            ></v-img>
            <v-icon
              v-if="isHovering"
              :size="72"
              icon="mdi-play"
              color="primary"
              class="position-absolute top-0 left-0 right-0 bottom-0 ma-auto"
            ></v-icon>
          </v-sheet>
          <p class="text-caption text-sm-subtitle-1 text-center mt-1">
            {{ item.metadata?.author }}
          </p>
          <p class="text-caption text-sm-subtitle-1 text-center text-medium-emphasis">
            {{ item.title }}
          </p>
        </v-sheet>
      </v-hover>
    </v-col>
  </v-row>
  <v-row
    v-if="layout === 'card'"
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
          width="272px"
          height="154px"
          class="mx-auto"
        >
          <v-img
            :src="item.thumbnail"
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
            </v-card-item>
            <v-card-actions>
              <v-btn
                color="primary"
                variant="flat"
                rounded="0"
                prepend-icon="mdi-play"
                size="small"
                class="text-none"
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
</template>

<script setup lang="ts">
import { useDisplay } from 'vuetify';
import type {ItemContent} from '/@/views/homePage.vue';
import {useRouter} from 'vue-router';
const router = useRouter();
const {xs} = useDisplay();
interface Props {
  title: string;
  items: ItemContent[];
  layout: 'card' | 'list' | 'grid';
  showNavigation?: boolean;
  showViewAll?: boolean;
}
defineProps<Props>();
</script>
