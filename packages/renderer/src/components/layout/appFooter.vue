<template>
  <v-footer position="relative">
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
              subtitle="Riff.CC is a collaborative effort to create a free and open-source platform for music, videos, and creative content."
              class="pa-0"
            ></v-list-item>
            <br />
            <v-list-item
              subtitle="Early tech demo, work in progress. No warranties, here be dragons, but enjoy."
              class="pa-0"
            ></v-list-item>
            <br />
            <v-list-item class="pa-0">
              <template #subtitle>
                <p>
                  Items marked with <strong>^</strong> are <i>only partially available</i>, whether
                  due to licencing or lost content. All content made available under legally free
                  licences - specific credits will soon be available.
                </p>
              </template>
            </v-list-item>
            <br />
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
                <template v-if="key === 'explore'">
                  <v-list-item
                    v-for="item in featuredContentCategories"
                    :key="item.id"
                    :subtitle="item.displayName"
                    class="mb-2 pl-1"
                    min-height="12px"
                    height="24px"
                    @click="router.push(`/featured/${item.id}`)"
                  ></v-list-item>
                </template>
              </v-list>
            </div>
          </v-sheet>
        </v-col>
      </v-row>
    </v-container>
    <v-btn
      icon="$chevron-up"
      density="comfortable"
      color="primary-darken-1"
      rounded="0"
      class="position-absolute bottom-0 right-0 mb-2 mr-2"
      @click="scrollToTop"
    >
      <v-icon />
    </v-btn>
  </v-footer>
  <v-sheet
    color="primary-darken-1"
    height="64px"
    class="d-flex align-center items-center justify-center"
  >
    <v-chip variant="text">
      <template #prepend>
        <img
          src="/cc.svg"
          alt="Creative Commons License"
          class="mr-1"
          style="filter: invert(100%) sepia(0%) saturate(7438%) hue-rotate(78deg) brightness(109%) contrast(95%)"
          width="20"
          height="20"
        />
      </template>
      e cinere surgemus.
    </v-chip>
  </v-sheet>
</template>

<script setup lang="ts">
import {useRouter} from 'vue-router';
import {navigationMap} from '/@/constants/navigation';
import { computed } from 'vue';
import { useContentCategoriesQuery } from '/@/plugins/lensService/hooks';

const router = useRouter();

const { data: contentCategories } = useContentCategoriesQuery();

const featuredContentCategories = computed(() => contentCategories.value?.filter(c => c.featured));
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
