<template>
  <v-footer
    class="bg-background-darken-2"
    position="relative"
  >
    <v-container
      class="fill-height"
      fluid
    >
      <v-row class="px-2 px-sm-6 px-md-16 py-10 align-center">
        <v-col
          cols="12"
          md="3"
        >
          <v-sheet>
            <v-img
              height="90px"
              inline
              width="90px"
              src="src/assets/logo.svg"
            ></v-img>
            <v-list-item
              subtitle="Riff.CC is a dynamic platform for music, videos, and creative content. Discover, share, and enjoy a wide variety of multimedia in one place, with personalized recommendations and a vibrant community of creators."
              class="pa-0"
            ></v-list-item>
          </v-sheet>
        </v-col>
        <v-col
          cols="12"
          md="6"
        >
          <v-sheet
            class="d-flex flex-wrap justify-space-evenly"
          >
            <div
              v-for="(section, key) in navigationMap.appFooter"
              :key="key"
            >
              <p class="text-h5 mb-2">{{ key.charAt(0).toUpperCase() + key.slice(1) }}</p>
              <v-list max-width="200px">
                <v-list-item
                  v-for="item in section"
                  :key="item.label"
                  :subtitle="item.label"
                  :ripple="false"
                  class="mb-2 px-0"
                  min-height="12px"
                  height="24px"
                  @click="() => $router.push(item.path)"
                >
                  <template #prepend>
                    <div class="pr-2 d-flex pb-1">
                      <v-icon
                        icon="fas fa-circle"
                        size="8px"
                        color="primary"
                      ></v-icon>
                    </div>
                  </template>
                </v-list-item>
              </v-list>
            </div>
          </v-sheet>
        </v-col>
        <v-col
          cols="12"
          md="3"
        >
          <v-sheet>
            <p class="text-h5 mb-4">Download App</p>
            <v-list-item
              subtitle="lorem ispusa asdmasd msmd lsdk ls askwe"
              class="px-0 mb-4"
              min-height="25px"
            >
              <template #prepend>
                <div class="pr-2 d-flex pb-1">
                  <v-icon
                    icon="fas fa-circle"
                    size="8px"
                    color="primary"
                  ></v-icon>
                </div>
              </template>
            </v-list-item>
            <div class="d-flex">
              <v-img
                width="120px"
                inline
                class="mr-2"
                src="/mock/app-store.png"
              ></v-img>
              <v-img
                width="120px"
                inline
                src="/mock/play-store.png"
              ></v-img>
            </div>
          </v-sheet>
        </v-col>
      </v-row>
    </v-container>
    <v-btn
      position="absolute"
      icon
      color="primary"
      rounded="0"
      location="bottom right"
      class="mb-2 mr-4"
      @click="scrollToTop"
    >
      <v-icon icon="fas fa-chevron-up" />
    </v-btn>
  </v-footer>
  <v-sheet
    color="primary"
    height="64px"
    class="d-flex align-center justify-center"
  >
    <p class="text-subtitle-2">
      <img
        src="https://mirrors.creativecommons.org/presskit/icons/cc.svg"
        alt="Creative Commons License"
        width="17em"
        style="margin-bottom: -0.25em;"
      /> e cinere surgemus.
    </p>
  </v-sheet>
</template>

<script setup lang="ts">
import { navigationMap } from '/@/constants/navigation';

const scrollToTop = () => {
  window.scrollTo({
    top: 0,
    behavior: 'smooth',
  });
};
</script>
