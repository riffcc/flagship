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
              src="/logo.svg"
            ></v-img>
            <v-list-item
              subtitle="Riff.CC is a dynamic platform for music, videos, and creative content. Discover, share, and enjoy a wide variety of multimedia in one place."
              class="pa-0"
            ></v-list-item>
            <br />
            <v-list-item
              subtitle="This is an early tech demo, work in progress. It is intended only as a proof of concept but may be interesting and enjoyable. No warranties are given, here be dragons, but enjoy."
              class="pa-0"
            ></v-list-item>
            <br />
            <v-list-item class="pa-0">
              <template #subtitle>
                <p>
                  Items marked with <strong>%</strong> have broken audio or other playback issues
                  which will be resolved soon.
                </p>
                <p>
                  Items marked with <strong>^</strong> are <i>only partially available</i>, whether
                  due to licencing or lost content. All content made available under legally free
                  licences - specific credits will soon be available.
                </p>
              </template>
            </v-list-item>
            <br />
            <v-list-item
              subtitle="The Riff.CC project is a collaborative effort to create a free and open-source platform for music, videos, and creative content. We believe in the power of community and the importance of sharing knowledge and experiences. So Riff.CC's software allows you to build whatever you want, and link it with other sites that share your interests, allowing you to cooperatively build up a content library while each retains sovereignty over their library's specific content."
              class="pa-0"
            ></v-list-item>
          </v-sheet>
        </v-col>
        <v-col
          cols="12"
          md="9"
        >
          <v-sheet class="d-flex flex-wrap justify-space-evenly">
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
                  class="mb-2 pl-1"
                  min-height="12px"
                  height="24px"
                  @click="item.path === '/contact' ? openEmailClient() : router.push(item.path)"
                ></v-list-item>
              </v-list>
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
        style="margin-bottom: -0.25em"
      />
      e cinere surgemus.
    </p>
  </v-sheet>
</template>

<script setup lang="ts">
import {useRouter} from 'vue-router';
import {navigationMap} from '/@/constants/navigation';

const router = useRouter();

const scrollToTop = () => {
  window.scrollTo({
    top: 0,
    behavior: 'smooth',
  });
};

const openEmailClient = () => {
  window.location.href = 'mailto:wings@riff.cc';
};
</script>
